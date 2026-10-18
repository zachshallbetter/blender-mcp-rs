pub mod addon_manager;
pub mod client_manager;

use blender_mcp_core::BlenderMcpServer;
use clap::{Args, Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "blender-mcp")]
#[command(author = "Zachary Shallbetter")]
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

    /// Pack Ambient Occlusion (R), Roughness (G), and Metallic (B) into an ORM texture
    Pbr(PbrArgs),

    /// Rig semantic sockets and physics collision hulls on an object
    Rig(RigArgs),

    /// Procedurally craft a game-ready hero prop conforming to the Barrel Protocol
    Craft(CraftArgs),

    /// Manage AI agent MCP client configurations (Claude, Codex, Gemini, Antigravity, Cursor)
    Client(ClientArgs),

    /// Export specialized agent skills and prompts for all major AI coding platforms
    ExportSkills(ExportSkillsArgs),

    /// Extract a formal 3D Compiler Semantic Graph (rr-csg-0.1+3d) from Blender
    Csg(CsgArgs),

    /// Generate multi-LOD decimation meshes and optional packed ORM textures
    Lods(LodsArgs),

    /// Manage companion Blender addon installation and status
    Addon(AddonArgs),
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
struct PbrArgs {
    /// Destination path for the packed ORM texture
    #[arg(long, short = 'o')]
    output: PathBuf,

    /// Path to Ambient Occlusion map
    #[arg(long)]
    ao: Option<PathBuf>,

    /// Path to Roughness map
    #[arg(long)]
    roughness: Option<PathBuf>,

    /// Path to Metallic map
    #[arg(long)]
    metallic: Option<PathBuf>,

    /// Output result in machine-readable JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RigArgs {
    /// Name of target object
    object_name: String,

    /// Type of physics collider: "convex", "box", or "capsule"
    #[arg(long, default_value = "convex")]
    collider: String,

    /// Comma-separated list of semantic sockets (e.g. "SOCKET_grip_primary,SOCKET_muzzle")
    #[arg(long)]
    sockets: Option<String>,

    /// Blender IPC host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Blender IPC port
    #[arg(long, default_value_t = 9876)]
    port: u16,
}

#[derive(Args)]
struct CraftArgs {
    /// Prop archetype (western_barrel, wooden_crate, lantern, revolver)
    archetype: String,

    /// Destination output directory
    #[arg(long, default_value = "./assets")]
    output_dir: PathBuf,

    /// Blender IPC host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Blender IPC port
    #[arg(long, default_value_t = 9876)]
    port: u16,
}

#[derive(Args)]
struct ExportSkillsArgs {
    /// Destination directory to export skills into
    #[arg(long, default_value = ".")]
    dest_dir: PathBuf,

    /// Target platform format: "all", "antigravity", "cursor", "claude", "gemini"
    #[arg(long, default_value = "all")]
    format: String,
}

#[derive(Args)]
struct CsgArgs {
    /// Source scene identifier (default: active_scene)
    #[arg(long, default_value = "active_scene")]
    source: String,

    /// Blender IPC host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Blender IPC port
    #[arg(long, default_value_t = 9876)]
    port: u16,

    /// Output report in machine-readable JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct LodsArgs {
    /// Name of target object in Blender
    object_name: String,

    /// Comma-separated decimation ratios (e.g. "1.0,0.5,0.25,0.1")
    #[arg(long, default_value = "1.0,0.5,0.25,0.1")]
    ratios: String,

    /// Type of physics collider: "convex_hull", "box", or "capsule"
    #[arg(long, default_value = "convex_hull")]
    collider: String,

    /// Pack Ambient Occlusion, Roughness, and Metallic textures into an ORM map
    #[arg(long)]
    pack_orm: bool,

    /// Path to Ambient Occlusion map
    #[arg(long)]
    ao: Option<PathBuf>,

    /// Path to Roughness map
    #[arg(long)]
    roughness: Option<PathBuf>,

    /// Path to Metallic map
    #[arg(long)]
    metallic: Option<PathBuf>,

    /// Output destination path for packed ORM map
    #[arg(long)]
    output_orm: Option<PathBuf>,

    /// Blender IPC host address
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Blender IPC port
    #[arg(long, default_value_t = 9876)]
    port: u16,

    /// Output result in machine-readable JSON
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
    GenerateConfig { client: String },
}

#[derive(Args)]
struct AddonArgs {
    #[command(subcommand)]
    action: AddonAction,
}

#[derive(Subcommand)]
enum AddonAction {
    /// Detect installed Blender versions and check companion addon status
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Install or update the companion addon for detected Blender versions
    Install {
        #[arg(long)]
        blender_version: Option<String>,
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
                println!(
                    "  Game Ready: {}",
                    if report.passes_game_ready_check {
                        "YES ✅"
                    } else {
                        "NO ❌"
                    }
                );
                if !report.warnings.is_empty() {
                    println!("  Warnings:");
                    for w in &report.warnings {
                        println!("    - {}", w);
                    }
                }
            }
        }
        Some(Commands::Pbr(args)) => {
            let res = blender_mcp_pbr::pack_orm(
                args.ao.as_deref(),
                args.roughness.as_deref(),
                args.metallic.as_deref(),
                &args.output,
            )?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!(
                    "Successfully packed ORM texture ({}x{}): {}",
                    res.width, res.height, res.output_path
                );
            }
        }
        Some(Commands::Rig(args)) => {
            let sockets: Vec<String> = args
                .sockets
                .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default();

            let ipc = blender_mcp_ipc::BlenderIpcClient::new(&args.host, args.port);
            let cmd = serde_json::json!({
                "type": "rig_game_item",
                "params": {
                    "object_name": args.object_name,
                    "collider_type": args.collider,
                    "sockets": sockets,
                }
            });
            match ipc.send_command(&cmd).await {
                Ok(resp) => println!("Rigging response: {}", serde_json::to_string_pretty(&resp)?),
                Err(e) => eprintln!("Rigging failed: {}", e),
            }
        }
        Some(Commands::Craft(args)) => {
            let ipc = blender_mcp_ipc::BlenderIpcClient::new(&args.host, args.port);
            let cmd = serde_json::json!({
                "type": "craft_hero_prop",
                "params": {
                    "archetype": args.archetype,
                    "output_dir": args.output_dir.display().to_string(),
                }
            });
            match ipc.send_command(&cmd).await {
                Ok(resp) => println!(
                    "Crafting response: {}",
                    serde_json::to_string_pretty(&resp)?
                ),
                Err(e) => eprintln!("Crafting failed: {}", e),
            }
        }
        Some(Commands::ExportSkills(args)) => {
            export_agent_skills(&args.dest_dir, &args.format)?;
        }
        Some(Commands::Csg(args)) => {
            let ipc = blender_mcp_ipc::BlenderIpcClient::new(&args.host, args.port);
            let cmd = serde_json::json!({
                "type": "get_scene_info",
                "params": {}
            });
            match ipc.send_command(&cmd).await {
                Ok(info) => {
                    let mut graph = blender_mcp_geom::SpatialCsgGraph::new(&args.source);
                    if let Some(objects) = info.get("objects").and_then(|v| v.as_array()) {
                        for obj in objects {
                            let name = obj
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let obj_type =
                                obj.get("type").and_then(|v| v.as_str()).unwrap_or("EMPTY");
                            let kind = if name.starts_with("SOCKET_") {
                                blender_mcp_geom::CsgNodeKind::Socket
                            } else if name.starts_with("UCX_")
                                || name.starts_with("UBX_")
                                || name.starts_with("UCP_")
                            {
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

                            let id = format!(
                                "{}:{}",
                                match kind {
                                    blender_mcp_geom::CsgNodeKind::Socket => "socket",
                                    blender_mcp_geom::CsgNodeKind::Collider => "collider",
                                    blender_mcp_geom::CsgNodeKind::Mesh => "mesh",
                                    blender_mcp_geom::CsgNodeKind::Camera => "camera",
                                    blender_mcp_geom::CsgNodeKind::Light => "light",
                                    _ => "obj",
                                },
                                name
                            );

                            graph.add_node(blender_mcp_geom::CsgNode {
                                id,
                                name: name.to_string(),
                                kind,
                                minimum_access: "public".to_string(),
                                transform: None,
                                triangle_count: obj
                                    .get("polygons")
                                    .and_then(|v| v.as_u64())
                                    .map(|c| c as usize),
                                metadata: None,
                            });
                        }
                    }
                    if args.json {
                        println!("{}", serde_json::to_string_pretty(&graph)?);
                    } else {
                        println!("=== Spatial CSG Graph: {} ===", graph.provenance.source);
                        println!("  Nodes: {}", graph.nodes.len());
                        println!("  Edges: {}", graph.edges.len());
                        for node in &graph.nodes {
                            println!("  • [{:?}] {}", node.kind, node.name);
                        }
                    }
                }
                Err(e) => eprintln!("Failed to query Blender scene for CSG: {}", e),
            }
        }
        Some(Commands::Lods(args)) => {
            let lod_ratios: Vec<f64> = args
                .ratios
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();

            let ipc = blender_mcp_ipc::BlenderIpcClient::new(&args.host, args.port);
            let cmd = serde_json::json!({
                "type": "generate_lods_and_collision",
                "params": {
                    "object_name": args.object_name,
                    "lod_ratios": lod_ratios,
                    "collision_type": args.collider,
                    "generate_colliders": true,
                }
            });

            let lod_resp = ipc.send_command(&cmd).await;

            let mut orm_result = None;
            if args.pack_orm {
                let out_path = args
                    .output_orm
                    .unwrap_or_else(|| PathBuf::from(format!("{}_ORM.png", args.object_name)));
                let pbr_res = blender_mcp_pbr::pack_orm(
                    args.ao.as_deref(),
                    args.roughness.as_deref(),
                    args.metallic.as_deref(),
                    &out_path,
                )?;
                orm_result = Some(pbr_res);
            }

            if args.json {
                let mut output = serde_json::json!({
                    "object_name": args.object_name,
                    "lod_generation": match &lod_resp {
                        Ok(v) => v.clone(),
                        Err(e) => serde_json::json!({ "error": e.to_string() }),
                    }
                });
                if let Some(orm) = &orm_result {
                    output["orm_packing"] = serde_json::to_value(orm)?;
                }
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                match lod_resp {
                    Ok(resp) => println!(
                        "LOD & Collision Generation: {}",
                        serde_json::to_string_pretty(&resp)?
                    ),
                    Err(e) => eprintln!(
                        "LOD generation IPC failed (is Blender running with companion addon?): {}",
                        e
                    ),
                }
                if let Some(orm) = &orm_result {
                    println!(
                        "Packed ORM Map ({}x{}): {}",
                        orm.width, orm.height, orm.output_path
                    );
                }
            }
        }
        Some(Commands::Client(args)) => match args.action {
            ClientAction::List { json } => {
                let clients = client_manager::list_detected_clients();
                if json {
                    println!("{}", serde_json::to_string_pretty(&clients)?);
                } else {
                    println!("Supported & Detected MCP Agent Clients:");
                    for c in &clients {
                        let status = if c.is_configured {
                            "configured ✅"
                        } else {
                            "not configured"
                        };
                        println!(
                            "  • {:<15} ({}) - {} [{}]",
                            c.name,
                            c.client_type,
                            c.config_path.display(),
                            status
                        );
                    }
                }
            }
            ClientAction::Add { client, scope } => {
                match client_manager::configure_client(&client, &scope) {
                    Ok(p) => println!(
                        "Successfully configured '{}' (scope: {}) at: {}",
                        client,
                        scope,
                        p.display()
                    ),
                    Err(e) => eprintln!("Failed to configure '{}': {}", client, e),
                }
            }
            ClientAction::GenerateConfig { client } => {
                if client == "codex" {
                    println!(
                        "[mcp_servers.blender]\ncommand = \"blender-mcp\"\nargs = [\"serve\"]"
                    );
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
        },
        Some(Commands::Addon(args)) => match args.action {
            AddonAction::Status { json } => {
                let installs = addon_manager::detect_blender_installations();
                if json {
                    println!("{}", serde_json::to_string_pretty(&installs)?);
                } else {
                    println!("Detected Blender Installations & Addon Status:");
                    if installs.is_empty() {
                        println!("  No installed Blender version directories detected.");
                    }
                    for inst in &installs {
                        let st = if inst.is_addon_installed {
                            "INSTALLED ✅"
                        } else {
                            "NOT INSTALLED ❌"
                        };
                        println!(
                            "  • Blender {:<4} | Addon: {:<17} | Path: {}",
                            inst.version,
                            st,
                            inst.addon_dir.display()
                        );
                    }
                }
            }
            AddonAction::Install { blender_version } => {
                match addon_manager::install_addon(blender_version.as_deref()) {
                    Ok(paths) => {
                        if paths.is_empty() {
                            println!("No matching Blender installation found to install addon.");
                        } else {
                            println!("Successfully installed blender-mcp-rs companion addon:");
                            for p in paths {
                                println!("  ✅ {}", p.display());
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to install addon: {}", e),
                }
            }
        },
    }

    Ok(())
}

fn export_agent_skills(dest_dir: &Path, format: &str) -> std::io::Result<()> {
    let skill_content = r#"---
name: blender-mcp
description: High-performance reflective 3D spatial agent runtime for Blender. Use for auditing models, rigging sockets & Rapier3D colliders, packing ORM textures, and crafting game props.
---

# blender-mcp Instructions

Use the `blender-mcp` MCP server tools:
1. `audit_asset`: Empirically audit a .glb asset for polycount and material budget.
2. `rig_game_item`: Auto-place semantic sockets (`SOCKET_grip_primary`, `SOCKET_muzzle`) and generate physics colliders (`UCX_`, `UBX_`, `UCP_`).
3. `pack_pbr_textures`: Pack AO, Roughness, and Metallic textures into game-ready ORM maps.
4. `craft_hero_prop`: Procedurally craft hero props conforming to the 7-step Barrel Protocol.
5. `execute_blender_code`: Execute live Python code in Blender.
6. `get_scene_info`: Introspect the real-time Blender scene.
"#;

    if format == "all" || format == "antigravity" {
        let p = dest_dir.join(".agents/skills/blender-mcp/SKILL.md");
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, skill_content)?;
        println!("Exported Antigravity skill: {}", p.display());
    }

    if format == "all" || format == "cursor" {
        let p = dest_dir.join(".cursor/rules/blender-mcp.mdc");
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, skill_content)?;
        println!("Exported Cursor rule: {}", p.display());
    }

    if format == "all" || format == "claude" {
        let p = dest_dir.join(".claude/skills/blender-mcp.md");
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, skill_content)?;
        println!("Exported Claude skill: {}", p.display());
    }

    if format == "all" || format == "gemini" {
        let p = dest_dir.join(".gemini/skills/blender-mcp.md");
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, skill_content)?;
        println!("Exported Gemini skill: {}", p.display());
    }

    Ok(())
}
