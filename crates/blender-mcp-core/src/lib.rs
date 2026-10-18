use std::sync::Arc;

use blender_mcp_ipc::BlenderIpcClient;
use blender_mcp_protocol::*;
use blender_mcp_reflect::{ToolDefinition, ToolRegistry};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub struct ServerContext {
    pub registry: ToolRegistry,
    pub ipc: Arc<BlenderIpcClient>,
}

#[derive(Deserialize, JsonSchema)]
struct AuditAssetParams {
    /// Absolute path to the .glb / .gltf file to audit
    file_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct ExecuteCodeParams {
    /// Python code to execute inside the live Blender instance
    code: String,
}

#[derive(Deserialize, JsonSchema)]
struct RigItemParams {
    /// Target object or root node name to rig
    object_name: String,
    /// Type of physics collider: "convex", "box", or "capsule"
    #[serde(default = "default_collider")]
    collider_type: String,
    /// List of semantic socket names to anchor (e.g. ["SOCKET_grip_primary", "SOCKET_muzzle"])
    #[serde(default)]
    sockets: Vec<String>,
}

fn default_collider() -> String {
    "convex".to_string()
}

pub fn create_default_registry(ipc: Arc<BlenderIpcClient>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 1. audit_asset tool (native SIMD & glTF parsing)
    registry.register_fn(
        "audit_asset",
        "Empirically audits a 3D asset (.glb) against game engine performance budgets (triangles, materials, UVs)",
        |params: AuditAssetParams| async move {
            blender_mcp_geom::ModelAuditor::audit_glb(&params.file_path)
                .map_err(|e| format!("Audit failed: {}", e))
        },
    );

    // 2. execute_blender_code tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "execute_blender_code",
        "Executes Python code directly inside the live Blender runtime via sub-millisecond IPC",
        move |params: ExecuteCodeParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "execute_code",
                    "params": { "code": params.code }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("IPC execution failed: {}", e))
            }
        },
    );

    // 3. rig_game_item tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "rig_game_item",
        "Generates semantic sockets, articulation constraints, and lightweight physics collision hulls",
        move |params: RigItemParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "rig_game_item",
                    "params": {
                        "object_name": params.object_name,
                        "collider_type": params.collider_type,
                        "sockets": params.sockets,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Rigging failed: {}", e))
            }
        },
    );

    registry
}

pub struct BlenderMcpServer {
    context: ServerContext,
}

impl BlenderMcpServer {
    pub fn new(host: &str, port: u16) -> Self {
        let ipc = Arc::new(BlenderIpcClient::new(host, port));
        let registry = create_default_registry(Arc::clone(&ipc));
        Self {
            context: ServerContext { registry, ipc },
        }
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.context.registry.list_tools()
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "initialize" => {
                let res = InitializeResult {
                    protocol_version: LATEST_MCP_VERSION.to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability { list_changed: Some(false) }),
                    },
                    server_info: Implementation {
                        name: "blender-mcp-rs".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };
                JsonRpcResponse::success(req.id, serde_json::to_value(res).unwrap())
            }
            "tools/list" => {
                let tools = self.list_tools();
                let val = serde_json::json!({ "tools": tools });
                JsonRpcResponse::success(req.id, val)
            }
            "tools/call" => {
                let params = req.params.unwrap_or(serde_json::Value::Null);
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

                match self.context.registry.dispatch(tool_name, arguments).await {
                    Ok(val) => {
                        let text = serde_json::to_string_pretty(&val).unwrap_or_default();
                        JsonRpcResponse::success(req.id, serde_json::to_value(CallToolResult::success(text)).unwrap())
                    }
                    Err(err) => {
                        JsonRpcResponse::success(req.id, serde_json::to_value(CallToolResult::error(err.to_string())).unwrap())
                    }
                }
            }
            _ => JsonRpcResponse::error(req.id, -32601, format!("Method '{}' not found", req.method)),
        }
    }

    /// Runs the stdio MCP JSON-RPC server loop over stdin/stdout.
    pub async fn run_stdio(&self) -> Result<(), anyhow::Error> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = tokio::io::stdout();

        info!("blender-mcp-rs stdio server running");

        while let Some(line) = reader.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(req) => {
                    let resp = self.handle_request(req).await;
                    let resp_str = serde_json::to_string(&resp)?;
                    stdout.write_all(resp_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    error!("Invalid JSON-RPC request: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_initialize_and_tools_list() {
        let server = BlenderMcpServer::new("127.0.0.1", 9999);

        // Test initialize
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "initialize".to_string(),
            params: None,
        };
        let init_resp = server.handle_request(init_req).await;
        assert!(init_resp.error.is_none());
        assert!(init_resp.result.is_some());

        // Test tools/list
        let list_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(2),
            method: "tools/list".to_string(),
            params: None,
        };
        let list_resp = server.handle_request(list_req).await;
        assert!(list_resp.error.is_none());
        let res_val = list_resp.result.unwrap();
        let tools = res_val.get("tools").unwrap().as_array().unwrap();
        assert!(tools.len() >= 3);

        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"audit_asset"));
        assert!(tool_names.contains(&"execute_blender_code"));
        assert!(tool_names.contains(&"rig_game_item"));
    }
}
