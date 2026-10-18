use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub client_type: String,
    pub config_path: PathBuf,
    pub is_configured: bool,
}

pub fn get_client_config_path(name: &str, scope: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = Path::new(&home);

    match (name, scope) {
        ("claude-code", "project") => PathBuf::from(".mcp.json"),
        ("claude-code", _) => home_path.join(".claude/mcp.json"),
        ("claude-desktop", _) => {
            #[cfg(target_os = "macos")]
            {
                home_path.join("Library/Application Support/Claude/claude_desktop_config.json")
            }
            #[cfg(not(target_os = "macos"))]
            {
                home_path.join(".config/Claude/claude_desktop_config.json")
            }
        }
        ("cursor", "project") => PathBuf::from(".cursor/mcp.json"),
        ("cursor", _) => home_path.join(".cursor/mcp.json"),
        ("antigravity", "project") => PathBuf::from(".agents/mcp_config.json"),
        ("antigravity", _) => home_path.join(".gemini/config/mcp_config.json"),
        ("codex", _) => home_path.join(".codex/config.toml"),
        ("gemini", "project") => PathBuf::from(".gemini/mcp_config.json"),
        ("gemini", _) => home_path.join(".config/gemini/mcp.json"),
        _ => PathBuf::from(format!(".{}_mcp.json", name)),
    }
}

pub fn configure_client(name: &str, scope: &str) -> Result<PathBuf, std::io::Error> {
    let config_path = get_client_config_path(name, scope);

    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    if name == "codex" {
        let snippet = "\n[mcp_servers.blender]\ncommand = \"blender-mcp\"\nargs = [\"serve\"]\n";
        let mut content = if config_path.exists() {
            std::fs::read_to_string(&config_path)?
        } else {
            String::new()
        };
        if !content.contains("[mcp_servers.blender]") {
            content.push_str(snippet);
            std::fs::write(&config_path, content)?;
        }
    } else {
        let mut root: serde_json::Value = if config_path.exists() {
            let data = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&data).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let server_entry = serde_json::json!({
            "command": "blender-mcp",
            "args": ["serve"]
        });

        let key = "mcpServers";

        if root.get(key).is_none() {
            root[key] = serde_json::json!({});
        }

        root[key]["blender"] = server_entry;
        let formatted = serde_json::to_string_pretty(&root)?;
        std::fs::write(&config_path, formatted)?;
    }

    Ok(config_path)
}

pub fn list_detected_clients() -> Vec<ClientInfo> {
    let clients = vec![
        ("claude-code", "cli"),
        ("claude-desktop", "desktop"),
        ("cursor", "editor"),
        ("antigravity", "ide"),
        ("codex", "cli"),
        ("gemini", "cli"),
    ];

    clients
        .into_iter()
        .map(|(name, ctype)| {
            let cfg = get_client_config_path(name, "user");
            let is_configured = if cfg.exists() {
                let s = std::fs::read_to_string(&cfg).unwrap_or_default();
                s.contains("blender-mcp")
            } else {
                false
            };
            ClientInfo {
                name: name.to_string(),
                client_type: ctype.to_string(),
                config_path: cfg,
                is_configured,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_paths() {
        let p_claude = get_client_config_path("claude-code", "project");
        assert_eq!(p_claude, PathBuf::from(".mcp.json"));

        let p_cursor = get_client_config_path("cursor", "project");
        assert_eq!(p_cursor, PathBuf::from(".cursor/mcp.json"));

        let p_anti = get_client_config_path("antigravity", "project");
        assert_eq!(p_anti, PathBuf::from(".agents/mcp_config.json"));
    }
}
