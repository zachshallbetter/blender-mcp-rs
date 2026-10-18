pub mod sandbox;

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
struct LookParams {
    /// Target object or location name to focus on
    #[serde(default = "default_focus_target")]
    target: String,
    /// Distance from target in meters
    #[serde(default = "default_camera_distance")]
    distance: f32,
    /// Azimuth angle in degrees (0 to 360)
    #[serde(default = "default_azimuth")]
    azimuth: f32,
    /// Elevation angle in degrees (-89 to 89)
    #[serde(default = "default_elevation")]
    elevation: f32,
}

fn default_focus_target() -> String { "active".to_string() }
fn default_camera_distance() -> f32 { 3.0 }
fn default_azimuth() -> f32 { 45.0 }
fn default_elevation() -> f32 { 30.0 }

#[derive(Deserialize, JsonSchema)]
struct ViewportSnapshotParams {
    /// Render resolution width
    #[serde(default = "default_res")]
    width: u32,
    /// Render resolution height
    #[serde(default = "default_res")]
    height: u32,
}

fn default_res() -> u32 { 1080 }

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

#[derive(Deserialize, JsonSchema)]
struct PackPbrParams {
    /// Optional path to Ambient Occlusion map
    ao_path: Option<String>,
    /// Optional path to Roughness map
    roughness_path: Option<String>,
    /// Optional path to Metallic map
    metallic_path: Option<String>,
    /// Destination path for packed ORM map (PNG)
    output_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct CraftHeroPropParams {
    /// Archetype name: "western_barrel", "wooden_crate", "lantern", "revolver"
    archetype: String,
    /// Destination directory for compiled .glb and textures
    #[serde(default = "default_output_dir")]
    output_dir: String,
}

fn default_output_dir() -> String {
    "./assets".to_string()
}

#[derive(Deserialize, JsonSchema)]
struct GetSceneInfoParams {}

#[derive(Deserialize, JsonSchema)]
struct GetSceneCsgParams {
    /// Source scene identifier or path (default: active_scene)
    #[serde(default = "default_csg_source")]
    source: String,
}

fn default_csg_source() -> String {
    "active_scene".to_string()
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

    // 2. pack_pbr_textures tool (SIMD ORM channel packing)
    registry.register_fn(
        "pack_pbr_textures",
        "Packs Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into a single game-ready ORM texture",
        |params: PackPbrParams| async move {
            blender_mcp_pbr::pack_orm(
                params.ao_path.as_deref(),
                params.roughness_path.as_deref(),
                params.metallic_path.as_deref(),
                &params.output_path,
            )
            .map_err(|e| format!("PBR packing failed: {}", e))
        },
    );

    // 3. execute_blender_code tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "execute_blender_code",
        "Executes Python code directly inside the live Blender runtime with sandboxing and sub-millisecond IPC",
        move |params: ExecuteCodeParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                sandbox::validate_blender_code(&params.code)
                    .map_err(|e| format!("Safe mode violation: {}", e))?;

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

    // 4. rig_game_item tool
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

    // 5. craft_hero_prop tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "craft_hero_prop",
        "Procedurally generates a game-ready hero prop conforming to the 7-step Barrel Protocol craftsmanship standard",
        move |params: CraftHeroPropParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "craft_hero_prop",
                    "params": {
                        "archetype": params.archetype,
                        "output_dir": params.output_dir,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Hero prop crafting failed: {}", e))
            }
        },
    );

    // 6. get_scene_info tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "get_scene_info",
        "Retrieves real-time scene hierarchy, meshes, active cameras, and material slots from Blender",
        move |_params: GetSceneInfoParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "get_scene_info",
                    "params": {}
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Scene query failed: {}", e))
            }
        },
    );

    // 7. get_scene_csg tool (rr-csg-0.1+3d Compiler Semantic Graph)
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "get_scene_csg",
        "Generates a formal 3D Compiler Semantic Graph (rr-csg-0.1+3d) with typed nodes, edges, sockets, and colliders",
        move |params: GetSceneCsgParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "get_scene_info",
                    "params": {}
                });
                let info = ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Scene query failed: {}", e))?;

                let mut graph = blender_mcp_geom::SpatialCsgGraph::new(&params.source);
                if let Some(objects) = info.get("objects").and_then(|v| v.as_array()) {
                    for obj in objects {
                        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let obj_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("EMPTY");
                        let kind = if name.starts_with("SOCKET_") {
                            blender_mcp_geom::CsgNodeKind::Socket
                        } else if name.starts_with("UCX_") || name.starts_with("UBX_") || name.starts_with("UCP_") {
                            blender_mcp_geom::CsgNodeKind::Collider
                        } else if obj_type == "MESH" {
                            blender_mcp_geom::CsgNodeKind::Mesh
                        } else if obj_type == "CAMERA" {
                            blender_mcp_geom::CsgNodeKind::Camera
                        } else if obj_type == "LIGHT" {
                            blender_mcp_geom::CsgNodeKind::Light
                        } else {
                            blender_mcp_geom::CsgNodeKind::Object
                        };

                        let tri_count = obj.get("polygons").and_then(|v| v.as_u64()).map(|c| c as usize);
                        let id = format!("{}:{}", match kind {
                            blender_mcp_geom::CsgNodeKind::Socket => "socket",
                            blender_mcp_geom::CsgNodeKind::Collider => "collider",
                            blender_mcp_geom::CsgNodeKind::Mesh => "mesh",
                            blender_mcp_geom::CsgNodeKind::Camera => "camera",
                            blender_mcp_geom::CsgNodeKind::Light => "light",
                            _ => "obj",
                        }, name);

                        graph.add_node(blender_mcp_geom::CsgNode {
                            id,
                            name: name.to_string(),
                            kind,
                            minimum_access: "public".to_string(),
                            transform: None,
                            triangle_count: tri_count,
                            metadata: None,
                        });
                    }
                }
                serde_json::to_value(&graph).map_err(|e| e.to_string())
            }
        },
    );

    // 8. synthesize_desert_flora tool
    registry.register_fn(
        "synthesize_desert_flora",
        "Procedurally synthesizes game-ready desert flora (Saguaro, Prickly Pear, Yucca, Desert Rock) with sockets and colliders",
        |params: blender_mcp_synth::FloraSynthParams| async move {
            let res = blender_mcp_synth::synthesize_desert_flora(&params);
            Ok(res)
        },
    );

    // 9. synthesize_municipal_prop tool
    registry.register_fn(
        "synthesize_municipal_prop",
        "Procedurally synthesizes game-ready municipal props (Western Barrel, Wooden Crate, Hitching Post, Lantern)",
        |params: blender_mcp_synth::PropSynthParams| async move {
            let res = blender_mcp_synth::synthesize_municipal_prop(&params);
            Ok(res)
        },
    );

    // 10. compose_assembly_grammar tool
    registry.register_fn(
        "compose_assembly_grammar",
        "Composes a multi-component spatial assembly grammar into a game-ready asset with sockets, colliders, and manifest",
        |grammar: blender_mcp_synth::AssemblyGrammar| async move {
            let res = blender_mcp_synth::compile_assembly_grammar(&grammar);
            Ok(res)
        },
    );

    // 11. look tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "look",
        "Positions and orbits the active 3D camera around a target object with azimuth, elevation, and distance",
        move |params: LookParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "look",
                    "params": {
                        "target": params.target,
                        "distance": params.distance,
                        "azimuth": params.azimuth,
                        "elevation": params.elevation,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Camera navigation failed: {}", e))
            }
        },
    );

    // 12. render_viewport_snapshot tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "render_viewport_snapshot",
        "Captures a high-resolution offscreen viewport or active camera snapshot for visual inspection",
        move |params: ViewportSnapshotParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "render_viewport_snapshot",
                    "params": {
                        "width": params.width,
                        "height": params.height,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Snapshot render failed: {}", e))
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
        assert_eq!(tools.len(), 12);

        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t.get("name").unwrap().as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"audit_asset"));
        assert!(tool_names.contains(&"pack_pbr_textures"));
        assert!(tool_names.contains(&"execute_blender_code"));
        assert!(tool_names.contains(&"rig_game_item"));
        assert!(tool_names.contains(&"craft_hero_prop"));
        assert!(tool_names.contains(&"get_scene_info"));
        assert!(tool_names.contains(&"get_scene_csg"));
        assert!(tool_names.contains(&"synthesize_desert_flora"));
        assert!(tool_names.contains(&"synthesize_municipal_prop"));
        assert!(tool_names.contains(&"compose_assembly_grammar"));
        assert!(tool_names.contains(&"look"));
        assert!(tool_names.contains(&"render_viewport_snapshot"));
    }
}
