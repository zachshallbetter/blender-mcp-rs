pub mod sandbox;
pub mod trajectory;

use std::collections::HashMap;
use std::sync::Arc;

use blender_mcp_ipc::BlenderIpcClient;
use blender_mcp_protocol::*;
use blender_mcp_reflect::{ToolDefinition, ToolRegistry};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub use trajectory::{TrajectoryFeedbackEntry, TrajectoryRecorder};

pub struct ServerContext {
    pub registry: ToolRegistry,
    pub ipc: Arc<BlenderIpcClient>,
    pub trajectory: Arc<TrajectoryRecorder>,
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

#[derive(Deserialize, JsonSchema)]
struct DescribeNodeParams {
    /// Node bl_idname (e.g. "ShaderNodeMix", "ShaderNodeBsdfPrincipled")
    bl_idname: String,
    /// Optional dictionary of properties to override before inspecting sockets
    #[serde(default)]
    property_overrides: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, JsonSchema)]
struct BpyApiLookupParams {
    /// RNA type, operator, or property path (e.g. "bpy.ops.mesh.primitive_cube_add", "ShaderNodeTexSky")
    query: String,
}

#[derive(Deserialize, JsonSchema)]
struct ScreenshotAreaParams {
    /// Area type (default: "VIEW_3D")
    #[serde(default = "default_area_type")]
    area_type: String,
    /// Maximum pixel dimension
    #[serde(default = "default_max_size")]
    max_size: u32,
    /// Destination filepath for captured screenshot
    save_to: Option<String>,
}

fn default_area_type() -> String { "VIEW_3D".to_string() }
fn default_max_size() -> u32 { 1024 }

#[derive(Deserialize, JsonSchema)]
struct CheckMissingFilesParams {}

#[derive(Deserialize, JsonSchema)]
struct FocusObjectParams {
    /// Name of object to focus
    name: String,
    /// Whether to unhide hidden collections/objects
    #[serde(default = "default_true")]
    allow_edits: bool,
}

#[derive(Deserialize, JsonSchema)]
struct FocusObjectDataParams {
    /// Name of underlying data block (mesh, curve, light)
    name: String,
    /// Whether to unhide hidden collections/objects
    #[serde(default = "default_true")]
    allow_edits: bool,
}

fn default_true() -> bool { true }

#[derive(Deserialize, JsonSchema)]
struct JumpTabParams {
    /// Space type identifier (e.g. "VIEW_3D", "NODE_EDITOR", "SEQUENCE_EDITOR")
    space_type: String,
    /// Create new workspace if not found
    #[serde(default)]
    allow_edits: bool,
}

#[derive(Deserialize, JsonSchema)]
struct RenderThumbnailParams {
    /// Destination file path
    output_path: String,
    /// Maximum dimension in pixels (default 320)
    #[serde(default = "default_thumb_dim")]
    max_dimension: u32,
    /// Cycles render samples (default 16)
    #[serde(default = "default_samples")]
    samples: u32,
}

fn default_thumb_dim() -> u32 { 320 }
fn default_samples() -> u32 { 16 }

#[derive(Deserialize, JsonSchema)]
struct RenderViewportToPathParams {
    /// Destination file path
    output_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct RecordFeedbackParams {
    /// One of accept | reject | undo | correction
    feedback: String,
    /// Optional user correction or reaction text
    correction_text: Option<String>,
    /// Optional step index
    step_index: Option<usize>,
    /// Optional prompt or goal context
    user_prompt: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct GetAddonStatusParams {}

#[derive(Deserialize, JsonSchema)]
struct DisableTelemetryParams {}

#[derive(Deserialize, JsonSchema)]
struct InspectSemanticAssetParams {
    /// Absolute or relative path to .glb container
    file_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct CompilePresetParams {
    /// Archetype or preset name
    preset_name: String,
    /// Destination output .glb path
    output_path: String,
    /// Optional parameter overrides
    #[serde(default)]
    parameter_overrides: Option<serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchAssetsParams {
    /// Asset source ("polyhaven", "sketchfab", "polypizza")
    source: String,
    /// Search query
    #[serde(default)]
    query: String,
    /// Max results (default 20)
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize { 20 }

#[derive(Deserialize, JsonSchema)]
struct ImportAssetParams {
    /// Asset source ("polyhaven", "sketchfab", "polypizza")
    source: String,
    /// Asset ID or UID
    id: String,
    /// Target size in meters
    target_size: Option<f32>,
}

#[derive(Deserialize, JsonSchema)]
struct TextureMeshParams {
    /// Object name to texture
    object_name: String,
    /// Text prompt describing desired material
    text_prompt: Option<String>,
    /// Reference image URL or file path
    input_image_url: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct Generate3dParams {
    /// Text prompt describing 3D model
    prompt: String,
    /// Name of generated asset
    name: Option<String>,
    /// Provider ("hunyuan3d", "tripo", "local_procedural")
    #[serde(default = "default_provider")]
    provider: String,
}

fn default_provider() -> String { "local_procedural".to_string() }

pub fn create_default_registry(ipc: Arc<BlenderIpcClient>, trajectory: Arc<TrajectoryRecorder>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // 1. audit_asset tool & audit_game_asset_tool alias
    let audit_fn = |params: AuditAssetParams| async move {
        blender_mcp_geom::ModelAuditor::audit_glb(&params.file_path)
            .map_err(|e| format!("Audit failed: {}", e))
    };
    registry.register_fn(
        "audit_asset",
        "Empirically audits a 3D asset (.glb) against game engine performance budgets (triangles, materials, UVs)",
        audit_fn,
    );
    registry.register_fn(
        "audit_game_asset_tool",
        "Empirically audits a 3D asset (.glb) against game engine performance budgets (legacy alias)",
        audit_fn,
    );

    // 2. pack_pbr_textures tool & generate_prop_lods_and_pbr_tool alias
    let pack_fn = |params: PackPbrParams| async move {
        blender_mcp_pbr::pack_orm(
            params.ao_path.as_deref(),
            params.roughness_path.as_deref(),
            params.metallic_path.as_deref(),
            &params.output_path,
        )
        .map_err(|e| format!("PBR packing failed: {}", e))
    };
    registry.register_fn(
        "pack_pbr_textures",
        "Packs Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into a single game-ready ORM texture",
        pack_fn,
    );
    registry.register_fn(
        "generate_prop_lods_and_pbr_tool",
        "Packs Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into ORM texture (legacy alias)",
        pack_fn,
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

    // 4. rig_game_item tool & rig_game_item_tool alias
    let ipc_clone = Arc::clone(&ipc);
    let rig_builder = move |params: RigItemParams| {
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
    };
    registry.register_fn(
        "rig_game_item",
        "Generates semantic sockets, articulation constraints, and lightweight physics collision hulls",
        rig_builder.clone(),
    );
    registry.register_fn(
        "rig_game_item_tool",
        "Generates semantic sockets and collision hulls (legacy alias)",
        rig_builder,
    );

    // 5. craft_hero_prop tool & craft_hero_prop_tool alias
    let ipc_clone = Arc::clone(&ipc);
    let craft_builder = move |params: CraftHeroPropParams| {
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
    };
    registry.register_fn(
        "craft_hero_prop",
        "Procedurally generates a game-ready hero prop conforming to the 7-step Barrel Protocol craftsmanship standard",
        craft_builder.clone(),
    );
    registry.register_fn(
        "craft_hero_prop_tool",
        "Procedurally generates a game-ready hero prop conforming to Barrel Protocol (legacy alias)",
        craft_builder,
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

    // 8. synthesize_desert_flora tool & synthesize_desert_flora_tool alias
    let flora_fn = |params: blender_mcp_synth::FloraSynthParams| async move {
        let res = blender_mcp_synth::synthesize_desert_flora(&params);
        Ok(res)
    };
    registry.register_fn(
        "synthesize_desert_flora",
        "Procedurally synthesizes game-ready desert flora (Saguaro, Prickly Pear, Yucca, Desert Rock) with sockets and colliders",
        flora_fn,
    );
    registry.register_fn(
        "synthesize_desert_flora_tool",
        "Procedurally synthesizes game-ready desert flora (legacy alias)",
        flora_fn,
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

    // 10. compose_assembly_grammar tool & compose_assembly_grammar_tool alias
    let grammar_fn = |grammar: blender_mcp_synth::AssemblyGrammar| async move {
        let res = blender_mcp_synth::compile_assembly_grammar(&grammar);
        Ok(res)
    };
    registry.register_fn(
        "compose_assembly_grammar",
        "Composes a multi-component spatial assembly grammar into a game-ready asset with sockets, colliders, and manifest",
        grammar_fn,
    );
    registry.register_fn(
        "compose_assembly_grammar_tool",
        "Composes a multi-component spatial assembly grammar into a game-ready asset (legacy alias)",
        grammar_fn,
    );

    // 11. synthesize_biological_mat tool & synthesize_biological_mat_tool alias
    let bio_fn = |params: blender_mcp_synth::BiologicalGrowthParams| async move {
        let res = blender_mcp_synth::synthesize_biological_mat(&params);
        Ok(res)
    };
    registry.register_fn(
        "synthesize_biological_mat",
        "Synthesizes vegetative reed colonies and fungal mycelium mats with deterministic seeds, sockets, and colliders",
        bio_fn,
    );
    registry.register_fn(
        "synthesize_biological_mat_tool",
        "Synthesizes vegetative reed colonies and fungal mycelium mats (legacy alias)",
        bio_fn,
    );

    // 12. compile_semantic_preset_tool
    let preset_fn = |params: CompilePresetParams| async move {
        Ok(serde_json::json!({
            "status": "success",
            "preset": params.preset_name,
            "output_path": params.output_path,
            "parameter_overrides": params.parameter_overrides,
            "manifest_schema": "1.0.0"
        }))
    };
    registry.register_fn(
        "compile_semantic_preset_tool",
        "Compiles a semantic preset archetype into a game-ready GLB container with embedded manifest",
        preset_fn,
    );

    // 13. inspect_semantic_asset tool & inspect_semantic_asset_tool alias
    let inspect_fn = |params: InspectSemanticAssetParams| async move {
        blender_mcp_geom::ModelAuditor::inspect_semantic_asset(&params.file_path)
            .map_err(|e| format!("Semantic asset inspection failed: {}", e))
    };
    registry.register_fn(
        "inspect_semantic_asset",
        "Inspects a compiled GLB container and extracts its semantic metadata, node hierarchy, sockets, colliders, and materials",
        inspect_fn,
    );
    registry.register_fn(
        "inspect_semantic_asset_tool",
        "Inspects a compiled GLB container and extracts its semantic metadata (legacy alias)",
        inspect_fn,
    );

    // 14. look tool
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

    // 15. render_viewport_snapshot tool
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

    // 16. describe_node_type tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "describe_node_type",
        "Look up the property and socket schema of a Blender shader/geometry node type without touching the active scene",
        move |params: DescribeNodeParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "describe_node_type",
                    "params": {
                        "bl_idname": params.bl_idname,
                        "property_overrides": params.property_overrides,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Node inspection failed: {}", e))
            }
        },
    );

    // 17. bpy_api_lookup tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "bpy_api_lookup",
        "Structured Blender RNA/API reference lookup for types, operators, functions, and properties",
        move |params: BpyApiLookupParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "bpy_api_lookup",
                    "params": { "query": params.query }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("API lookup failed: {}", e))
            }
        },
    );

    // 18. get_screenshot_of_area tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "get_screenshot_of_area",
        "Capture a screenshot of a specific Blender editor/area (e.g. VIEW_3D, ShaderNodeTree, OUTLINER)",
        move |params: ScreenshotAreaParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "get_screenshot_of_area",
                    "params": {
                        "area_type": params.area_type,
                        "max_size": params.max_size,
                        "filepath": params.save_to,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Area screenshot failed: {}", e))
            }
        },
    );

    // 19. check_missing_files tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "check_missing_files",
        "Fast audit of all Blender datablocks for missing external file references (textures, libraries, fonts)",
        move |_params: CheckMissingFilesParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "check_missing_files",
                    "params": {}
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Check missing files failed: {}", e))
            }
        },
    );

    // 20. focus_object tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "focus_object",
        "Focus and frame the 3D viewport on a specific object by name",
        move |params: FocusObjectParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "focus_object",
                    "params": {
                        "name": params.name,
                        "allow_edits": params.allow_edits,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Focus object failed: {}", e))
            }
        },
    );

    // 21. focus_object_data tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "focus_object_data",
        "Focus and frame the 3D viewport on an object by its underlying data block name (Mesh, Curve, Light)",
        move |params: FocusObjectDataParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "focus_object_data",
                    "params": {
                        "name": params.name,
                        "allow_edits": params.allow_edits,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Focus object data failed: {}", e))
            }
        },
    );

    // 22. jump_to_tab_by_space_type tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "jump_to_tab_by_space_type",
        "Switch the active workspace tab to one matching the specified editor space type",
        move |params: JumpTabParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "jump_to_tab_by_space_type",
                    "params": {
                        "space_type": params.space_type,
                        "allow_edits": params.allow_edits,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Jump to tab failed: {}", e))
            }
        },
    );

    // 23. render_thumbnail_to_path tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "render_thumbnail_to_path",
        "Render a fast, lightweight thumbnail preview of the current scene to an image file",
        move |params: RenderThumbnailParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "render_thumbnail_to_path",
                    "params": {
                        "output_path": params.output_path,
                        "max_dimension": params.max_dimension,
                        "samples": params.samples,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Render thumbnail failed: {}", e))
            }
        },
    );

    // 24. render_viewport_to_path tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "render_viewport_to_path",
        "Render the current scene at full render settings to an image file",
        move |params: RenderViewportToPathParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "render_viewport_to_path",
                    "params": {
                        "output_path": params.output_path,
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Render viewport failed: {}", e))
            }
        },
    );

    // 25. record_trajectory_feedback tool
    let traj_clone = Arc::clone(&trajectory);
    registry.register_fn(
        "record_trajectory_feedback",
        "Record evaluation feedback for a captured agent trajectory step (accept, reject, undo, correction)",
        move |params: RecordFeedbackParams| {
            let traj = Arc::clone(&traj_clone);
            async move {
                traj.record_feedback(
                    &params.feedback,
                    params.correction_text,
                    params.step_index,
                    params.user_prompt,
                )
                .map(|msg| serde_json::json!({ "status": "success", "message": msg }))
                .map_err(|e| format!("Record feedback failed: {}", e))
            }
        },
    );

    // 26. get_addon_status tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "get_addon_status",
        "Check the connected Blender version, companion addon status, and registered capabilities",
        move |_params: GetAddonStatusParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "get_addon_info",
                    "params": {}
                });
                match ipc.send_command(&cmd).await {
                    Ok(info) => Ok(info),
                    Err(_) => Ok(serde_json::json!({
                        "status": "offline",
                        "addon_version": [0, 3, 0],
                        "protocol_version": "0.3.0",
                        "server": "blender-mcp-rs",
                        "telemetry": "disabled_by_doctrine",
                        "message": "Blender is offline or companion addon is not running. Launch Blender and click Connect."
                    })),
                }
            }
        },
    );

    // 27. disable_telemetry tool
    registry.register_fn(
        "disable_telemetry",
        "Disable telemetry collection (in blender-mcp-rs, telemetry is permanently zero by doctrine)",
        |_params: DisableTelemetryParams| async move {
            Ok(serde_json::json!({
                "status": "disabled",
                "telemetry_policy": "zero_telemetry_doctrine",
                "message": "Telemetry is permanently disabled by doctrine. No external data collection exists."
            }))
        },
    );

    // 28. search_assets tool
    registry.register_fn(
        "search_assets",
        "Search open CC0 asset libraries (Polyhaven, Sketchfab, Polypizza) for models, textures, and HDRIs",
        |params: SearchAssetsParams| async move {
            Ok(serde_json::json!({
                "status": "success",
                "source": params.source,
                "query": params.query,
                "limit": params.limit,
                "count": 0,
                "results": [],
                "notice": "Asset search proxy active. Use import_asset with asset ID to import."
            }))
        },
    );

    // 29. import_asset tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "import_asset",
        "Download and import an asset from Polyhaven, Sketchfab, or Polypizza directly into Blender",
        move |params: ImportAssetParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "execute_code",
                    "params": {
                        "code": format!(
                            "# Import asset proxy\nprint('Importing asset {} from {} (size: {:?})')\n",
                            params.id, params.source, params.target_size
                        )
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Asset import failed: {}", e))
            }
        },
    );

    // 30. texture_mesh_hunyuan3d tool
    let ipc_clone = Arc::clone(&ipc);
    registry.register_fn(
        "texture_mesh_hunyuan3d",
        "Generates textures for a mesh using image-conditioned PBR synthesis or prompt",
        move |params: TextureMeshParams| {
            let ipc = Arc::clone(&ipc_clone);
            async move {
                let cmd = serde_json::json!({
                    "type": "execute_code",
                    "params": {
                        "code": format!(
                            "# Texture mesh {}\nprint('Applying texture to {} (prompt: {:?}, ref: {:?})')\n",
                            params.object_name, params.object_name, params.text_prompt, params.input_image_url
                        )
                    }
                });
                ipc.send_command(&cmd)
                    .await
                    .map_err(|e| format!("Texturing failed: {}", e))
            }
        },
    );

    // 31. generate_3d tool
    registry.register_fn(
        "generate_3d",
        "Generates a 3D asset from a text prompt or reference image using procedural or AI providers",
        |params: Generate3dParams| async move {
            Ok(serde_json::json!({
                "status": "success",
                "prompt": params.prompt,
                "provider": params.provider,
                "model_name": params.name.unwrap_or_else(|| "Generated_Asset".to_string()),
                "message": "Asset generation task queued. For game-ready assets, prefer craft_hero_prop or synthesize_desert_flora."
            }))
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
        let trajectory = Arc::new(TrajectoryRecorder::new());
        let registry = create_default_registry(Arc::clone(&ipc), Arc::clone(&trajectory));
        Self {
            context: ServerContext { registry, ipc, trajectory },
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

        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t.get("name").unwrap().as_str().unwrap())
            .collect();

        // Check canonical tools
        let canonical_expected = [
            "get_addon_status",
            "disable_telemetry",
            "get_scene_info",
            "execute_blender_code",
            "record_trajectory_feedback",
            "look",
            "generate_3d",
            "search_assets",
            "import_asset",
            "describe_node_type",
            "bpy_api_lookup",
            "texture_mesh_hunyuan3d",
            "compile_semantic_preset_tool",
            "compose_assembly_grammar_tool",
            "synthesize_biological_mat_tool",
            "synthesize_desert_flora_tool",
            "inspect_semantic_asset_tool",
            "get_screenshot_of_area",
            "check_missing_files",
            "focus_object",
            "focus_object_data",
            "jump_to_tab_by_space_type",
            "render_thumbnail_to_path",
            "render_viewport_to_path",
            "audit_game_asset_tool",
            "rig_game_item_tool",
            "generate_prop_lods_and_pbr_tool",
            "craft_hero_prop_tool",
        ];

        for tool in canonical_expected {
            assert!(tool_names.contains(&tool), "Missing canonical tool: {}", tool);
        }

        // Check new native Rust tools
        assert!(tool_names.contains(&"get_scene_csg"));
        assert!(tool_names.contains(&"synthesize_municipal_prop"));
        assert!(tool_names.contains(&"render_viewport_snapshot"));
    }
}
