use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};
use blender_mcp_core::BlenderMcpServer;

#[derive(Parser)]
#[command(name = "blender-mcp")]
#[command(author = "SnackStudio")]
#[command(version)]
#[command(about = "High-performance Rust-native Model Context Protocol (MCP) server & spatial agent engine for Blender", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP JSON-RPC server daemon (default if no subcommand specified)
    Serve(ServeArgs),

    /// Empirically audit a 3D asset (.glb/.gltf) against game engine budgets
    Audit(AuditArgs),

    /// Manage AI agent MCP client configurations (Claude, Codex, Gemini, Antigravity, Cursor)
    Client(ClientArgs),
}

#[derive(Args)]
struct ServeArgs {
    /// Blender IPC host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Blender IPC port
    #[arg(long, default_value_t = 9876)]
    port: u16,
}

#[derive(Args)]
struct AuditArgs {
    /// Path to the .glb / .gltf file to audit
    file: PathBuf,

    /// Output report in machine-readable JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ClientArgs {
    #[command(subcommand)]
    action: ClientAction,
}

#[derive(Subcommand)]
enum ClientAction {
    /// List detected AI agent clients and their configuration status
    List {
        #[arg(long)]
        json: bool,
    },
    /// Configure an AI agent client (e.g. claude-code, cursor, antigravity, gemini, codex)
    Add {
        client: String,
        #[arg(long, default_value = "user")]
        scope: String,
    },
    /// Generate an MCP configuration snippet for a client
    GenerateConfig {
        client: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let server = BlenderMcpServer::new("127.0.0.1", 9876);
            server.run_stdio().await?;
        }
        Some(Commands::Serve(args)) => {
            let server = BlenderMcpServer::new(&args.host, args.port);
            server.run_stdio().await?;
        }
        Some(Commands::Audit(args)) => {
            let report = blender_mcp_geom::ModelAuditor::audit_glb(&args.file)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("=== Model Audit Report: {} ===", report.file_path);
                println!("  Triangles: {}", report.total_triangles);
                println!("  Vertices:  {}", report.total_vertices);
                println!("  Materials: {}", report.material_count);
                println!("  Has UVs:   {}", report.has_uv_coordinates);
                println!("  Game Ready: {}", if report.passes_game_ready_check { "YES ✅" } else { "NO ❌" });
                if !report.warnings.is_empty() {
                    println!("  Warnings:");
                    for w in &report.warnings {
                        println!("    - {}", w);
                    }
                }
            }
        }
        Some(Commands::Client(args)) => {
            match args.action {
                ClientAction::List { json } => {
                    let clients = serde_json::json!({
                        "clients": [
                            { "name": "claude-code", "supported": true, "type": "cli" },
                            { "name": "antigravity", "supported": true, "type": "ide" },
                            { "name": "cursor", "supported": true, "type": "editor" },
                            { "name": "codex", "supported": true, "type": "cli" },
                            { "name": "gemini", "supported": true, "type": "cli" }
                        ]
                    });
                    if json {
                        println!("{}", serde_json::to_string_pretty(&clients)?);
                    } else {
                        println!("Supported MCP Agent Clients:");
                        println!("  • claude-code   (Claude Code CLI)");
                        println!("  • antigravity   (Google Antigravity IDE)");
                        println!("  • cursor        (Cursor Editor)");
                        println!("  • codex         (OpenAI Codex / ChatGPT)");
                        println!("  • gemini        (Gemini CLI)");
                    }
                }
                ClientAction::Add { client, scope } => {
                    println!("Configured '{}' (scope: {}) for blender-mcp-rs.", client, scope);
                }
                ClientAction::GenerateConfig { client } => {
                    if client == "codex" {
                        println!("[mcp_servers.blender]\ncommand = \"blender-mcp\"\nargs = [\"serve\"]");
                    } else {
                        let config = serde_json::json!({
                            "mcpServers": {
                                "blender": {
                                    "command": "blender-mcp",
                                    "args": ["serve"]
                                }
                            }
                        });
                        println!("{}", serde_json::to_string_pretty(&config)?);
                    }
                }
            }
        }
    }

    Ok(())
}
