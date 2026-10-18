# blender-mcp-rs Tool Reference Catalog

`blender-mcp-rs` provides **37 reflective MCP tools** categorized into Scene Inspection, Viewport & Camera Control, Code Execution, Asset Library Search & Downloader, Geometry & CSG, Game Pipeline & Rigging, and Procedural Synthesis.

---

## 1. Scene Inspection & Diagnostics

### `get_scene_info` (Alias: `list_scene_items`)
Retrieves the complete scene hierarchy, active collections, selected objects, materials, cameras, and lights.
- **Parameters**: None
- **Returns**: Hierarchical scene graph with transforms, types, visibility, and counts.

### `get_scene_csg`
Extracts the semantic spatial Constructive Solid Geometry (CSG) graph in `rr-csg-0.1+3d` format, including socket anchors (`SOCKET_*`) and collision hulls (`UCX_*`).
- **Parameters**:
  - `source` *(string, optional, default: "active_scene")*: Scene identifier.
- **Returns**: Graph of objects, semantic attributes, bounding boxes, and socket transforms.

### `describe_node_type`
Introspects Blender shader, geometry, or compositor node definitions and sockets.
- **Parameters**:
  - `bl_idname` *(string, required)*: Internal node bl_idname (e.g. `ShaderNodeBsdfPrincipled`).
  - `property_overrides` *(object, optional)*: Specific socket attributes to inspect.

### `bpy_api_lookup`
Performs fuzzy or exact semantic search against Blender's `bpy` Python API documentation and property signatures.
- **Parameters**:
  - `query` *(string, required)*: API symbol, operator, or type to inspect.

### `check_missing_files`
Scans the current `.blend` file for broken image paths, missing external textures, or unlinked libraries.
- **Parameters**: None

### `inspect_semantic_asset` (Alias: `inspect_semantic_asset_tool`)
Inspects a packaged `.glb` container for embedded semantic metadata (`extras.dead_hand`), LOD tiers, material slots, and physics colliders.
- **Parameters**:
  - `file_path` *(string, required)*: Path to `.glb` file.

### `audit_asset` (Alias: `audit_game_asset_tool`)
Empirically audits a `.glb` asset against game engine performance budgets (triangles, materials, draw calls, texel density).
- **Parameters**:
  - `file_path` *(string, required)*: Path to `.glb` file.

### `search_mentions`
Performs fast fuzzy autocomplete search across Blender scene objects, materials, and collections, returning `@mention` tokens and semantic URIs.
- **Parameters**:
  - `query` *(string, optional, default: "")*: Fuzzy search query.
  - `limit` *(integer, optional, default: 30)*: Maximum candidates to return.
- **Returns**: Formatted list of matches with `name`, `kind`, `uri` (`mention://objects/*`), and `description`.

---

## 2. Viewport & Camera Control

### `look` (Alias: `look_at`)
Positions the viewport camera looking at an object or coordinate using spherical coordinates (azimuth, elevation, distance).
- **Parameters**:
  - `target` *(string, optional, default: "active")*: Object name or "active".
  - `distance` *(float, optional, default: 3.0)*: Distance in meters.
  - `azimuth` *(float, optional, default: 45.0)*: Azimuth angle in degrees.
  - `elevation` *(float, optional, default: 30.0)*: Elevation angle in degrees.

### `render_viewport_snapshot`
Renders an instant high-resolution OpenGL snapshot of the active 3D viewport.
- **Parameters**:
  - `width` *(integer, optional, default: 1080)*: Resolution width.
  - `height` *(integer, optional, default: 1080)*: Resolution height.

### `render_viewport_to_path`
Renders the current 3D viewport directly to a designated file path.
- **Parameters**:
  - `output_path` *(string, required)*: Destination file path.

### `render_thumbnail_to_path`
Renders an anti-aliased studio thumbnail with transparent contact shadow plane.
- **Parameters**:
  - `output_path` *(string, required)*: Destination PNG file path.
  - `max_dimension` *(integer, optional, default: 320)*: Maximum pixel dimension.
  - `samples` *(integer, optional, default: 16)*: Render samples.

### `get_screenshot_of_area`
Captures a screenshot of a specific UI area (e.g. Shader Editor, 3D Viewport, Outliner).
- **Parameters**:
  - `area_type` *(string, optional, default: "VIEW_3D")*: Target space type.
  - `output_path` *(string, optional)*: Destination file path.

### `focus_object`
Frames and focuses an object in the 3D viewport (`view3d.view_selected`).
- **Parameters**:
  - `name` *(string, required)*: Target object name.
  - `allow_edits` *(boolean, optional, default: true)*: Unhide if hidden.

### `focus_object_data`
Focuses and expands object data properties in the Properties workspace.
- **Parameters**:
  - `name` *(string, required)*: Target object name.

### `jump_to_tab_by_space_type`
Switches active Blender workspace to the one containing the requested space type (e.g. `NODE_EDITOR`, `VIEW_3D`).
- **Parameters**:
  - `space_type` *(string, required)*: Space identifier.
### `viewport_pick`
Raycasts normalized screen coordinates $(x, y)$ from the active 3D camera into the scene, selecting the hit object in Blender and returning exact world hit position, distance, surface normal, and polygon material.
- **Parameters**:
  - `x` *(float, required)*: Normalized horizontal screen coordinate (0.0 to 1.0).
  - `y` *(float, required)*: Normalized vertical screen coordinate (0.0 to 1.0).
  - `seq` *(integer, optional, default: 0)*: Client sequence number.
- **Returns**: Hit status, `object_name`, `location: [x, y, z]`, `normal: [nx, ny, nz]`, `distance`, `face_index`, and `material`.

---

## 3. High-Frequency Viewport Streaming (Zero-Copy SHM)

### `start_viewport_stream`
Starts live background viewport rendering into the shared memory ring buffer (`/tmp/blender_mcp_shm_viewport`).
- **Parameters**:
  - `shm_path` *(string, optional, default: "/tmp/blender_mcp_shm_viewport")*: Memory map path.
  - `fps` *(integer, optional, default: 30, max: 60)*: Target frame rate.
  - `capacity` *(integer, optional, default: 16777216)*: Buffer size in bytes.

### `stop_viewport_stream`
Stops live viewport streaming and unmaps the shared memory buffer.
- **Parameters**: None

### `read_viewport_frame`
Reads the latest frame from the shared memory buffer with sub-microsecond latency.
- **Parameters**:
  - `shm_path` *(string, optional, default: "/tmp/blender_mcp_shm_viewport")*: Memory map path.
  - `include_base64` *(boolean, optional, default: false)*: Whether to include base64 payload.

---

## 4. Live Spatial CSG & Sockets

### `apply_scene_csg`
Executes live CSG boolean operations or snaps objects to semantic sockets.
- **Parameters**:
  - `operation` *(string, required)*: One of `"union"`, `"difference"`, `"intersect"`, or `"snap_to_socket"`.
  - `target_object` *(string, required)*: Target receiving modifier or anchor.
  - `operand_object` *(string, required)*: Operand object to apply or snap.
  - `socket_name` *(string, optional)*: Socket name on target (e.g. `"SOCKET_grip_primary"`).
  - `solver` *(string, optional, default: "FAST")*: Boolean solver mode (`"FAST"` or `"EXACT"`).

---

## 5. Game Pipeline, Collision & Rigging

### `generate_lods_and_collision`
Generates tiered Levels of Detail (LOD0 through LOD3) via decimation modifiers and creates physics colliders (`UCX_*`).
- **Parameters**:
  - `object_name` *(string, required)*: Target mesh name.
  - `lod_ratios` *(array of floats, optional, default: [1.0, 0.5, 0.25, 0.1])*: Decimation ratios.
  - `collision_type` *(string, optional, default: "convex_hull")*: `"convex_hull"`, `"box"`, or `"none"`.

### `generate_armature_rig`
Generates bone armatures and binds geometry with automatic vertex group skinning.
- **Parameters**:
  - `target_object` *(string, required)*: Target mesh to rig.
  - `rig_template` *(string, optional, default: "prop_container")*: `"prop_container"`, `"mechanical_door"`, or `"biped"`.
  - `bind_mesh` *(boolean, optional, default: true)*: Parent mesh with Armature deform modifier.

### `rig_game_item` (Alias: `rig_game_item_tool`)
Sets up game-ready hierarchy, physics collision hulls, and semantic sockets for items/props.
- **Parameters**:
  - `object_name` *(string, required)*: Object to rig.
  - `collider_type` *(string, optional, default: "convex")*: `"convex"`, `"box"`, or `"capsule"`.
  - `sockets` *(array of strings, optional)*: Semantic socket anchors.

### `pack_pbr_textures` (Alias: `generate_prop_lods_and_pbr_tool`)
Packs Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into a single ORM PNG texture atlas using native SIMD processing.
- **Parameters**:
  - `ao_path` *(string, optional)*: AO map file path.
  - `roughness_path` *(string, optional)*: Roughness map file path.
  - `metallic_path` *(string, optional)*: Metallic map file path.
  - `output_path` *(string, required)*: Destination ORM PNG path.

---

## 6. Live Asset Library & Downloaders

### `search_assets`
Searches online CC0 3D model, HDRI, and material repositories (Polyhaven, Sketchfab, Polypizza).
- **Parameters**:
  - `source` *(string, required)*: `"polyhaven"`, `"sketchfab"`, or `"polypizza"`.
  - `query` *(string, optional)*: Search term.
  - `limit` *(integer, optional, default: 20)*: Maximum results.

### `import_asset`
Downloads a remote asset and imports it directly into the active Blender scene.
- **Parameters**:
  - `source` *(string, required)*: Asset provider.
  - `asset_id` *(string, required)*: Asset ID.
  - `resolution` *(string, optional, default: "1k")*: Texture resolution (`"1k"`, `"2k"`, `"4k"`).

---

## 7. Procedural Asset Synthesis

### `craft_hero_prop` (Alias: `craft_hero_prop_tool`)
Synthesizes stylized game-ready hero props following the strict Craftsmanship Doctrine (physical grooves, chamfered rims, 3D studs, ORM materials).
- **Parameters**:
  - `archetype` *(string, required)*: `"western_barrel"`, `"wooden_crate"`, `"lantern"`, `"revolver"`.
  - `output_dir` *(string, optional, default: "./assets")*: Output directory.

### `synthesize_desert_flora` (Alias: `synthesize_desert_flora_tool`)
Procedurally synthesizes stylized desert vegetation (saguaro, prickly pear, barrel cactus, yucca).
- **Parameters**:
  - `species` *(string, required)*: Cactus/flora archetype.
  - `output_path` *(string, required)*: Output `.glb` path.
  - `arms` *(integer, optional, default: 2)*: Branch count.
  - `height` *(float, optional, default: 4.0)*: Height in meters.

### `synthesize_municipal_prop`
Synthesizes modular municipal street furniture and infrastructure.
- **Parameters**:
  - `archetype` *(string, required)*: `"hydrant"`, `"street_lamp"`, `"mailbox"`, `"bench"`.
  - `output_path` *(string, required)*: Output `.glb` path.

### `synthesize_biological_mat` (Alias: `synthesize_biological_mat_tool`)
Synthesizes organic procedural materials with full PBR channel generation.
- **Parameters**:
  - `tissue_type` *(string, required)*: `"cactus_skin"`, `"desert_bark"`, `"weathered_hide"`.
  - `output_path` *(string, required)*: Output texture path.

### `compose_assembly_grammar` (Alias: `compose_assembly_grammar_tool`)
Compiles semantic multi-part assembly grammars into a connected hierarchy.
- **Parameters**:
  - `grammar_json` *(string, required)*: Assembly grammar specification.
  - `output_path` *(string, required)*: Output `.glb` path.

### `compile_semantic_preset_tool`
Compiles an archetype preset directly into a game-ready asset container.
- **Parameters**:
  - `preset_name` *(string, required)*: Archetype identifier.
  - `output_path` *(string, required)*: Output `.glb` path.

---

## 8. Sandboxed Code Execution & Feedback

### `execute_blender_code`
Executes arbitrary Python code inside Blender within a secure AST sandbox preventing destructive operations (file deletion, unauthorized process spawning).
- **Parameters**:
  - `code` *(string, required)*: Python code to execute.

### `record_trajectory_feedback`
Records human-in-the-loop and agent trajectory feedback for iterative model learning.
- **Parameters**:
  - `feedback` *(string, required)*: `"accept"`, `"reject"`, `"undo"`, or `"correction"`.
  - `correction_text` *(string, optional)*: Correction description.
  - `step_index` *(integer, optional)*: Execution step number.
  - `user_prompt` *(string, optional)*: Original prompt.

### `get_addon_status`
Queries the live companion addon for version, Blender build, and active capabilities.
- **Parameters**: None

### `disable_telemetry`
Confirms and enforces zero-telemetry policy according to doctrine.
- **Parameters**: None
