# Changelog

All notable changes to `blender-mcp-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) under the governance of `VERSIONING.md` and `AUTHORITY.md`.

## [0.3.0] - 2026-10-17

### Added
- **Complete Procedural Synthesis Engine (`blender-mcp-synth`)**:
  - Desert flora synthesis: Saguaro cactus, Prickly Pear, Yucca, Desert Boulder with quad UV mapping.
  - Municipal and frontier props: Western barrels, wooden crates, hitching posts, kerosene lanterns.
  - Spatial Assembly Grammars: Multi-component composition with socket anchoring and Rapier3D collision hulls.
- **Safe Mode Code Sandbox (`blender-mcp-core::sandbox`)**:
  - Native token analysis blocking dangerous system calls (`os.system`, `subprocess`, `shutil.rmtree`, socket attacks) while preserving creative `bpy` scripting.
- **Visual Camera Navigation & Viewport Snapshots**:
  - Added reflective `look` tool for spherical camera orbit and active object tracking.
  - Added `render_viewport_snapshot` for high-resolution offscreen image capture.
- **Automated Blender Addon Lifecycle Manager (`blender-mcp-cli::addon_manager`)**:
  - Cross-platform detection for all installed Blender versions (4.0, 4.1, 4.2, 5.1, 5.2).
  - One-command addon installation (`blender-mcp addon install`) and inspection (`blender-mcp addon status`).
- **Complete 12-Tool Reflective MCP Suite**:
  - `audit_asset`, `pack_pbr_textures`, `execute_blender_code`, `rig_game_item`, `craft_hero_prop`, `get_scene_info`, `get_scene_csg`, `synthesize_desert_flora`, `synthesize_municipal_prop`, `compose_assembly_grammar`, `look`, `render_viewport_snapshot`.

### Verification Status
- **Implemented**: All 7 workspace crates (`reflect`, `protocol`, `ipc`, `geom`, `pbr`, `synth`, `core`, `cli`) and companion addon.
- **Tested**: 17 unit/integration tests passing 100% across workspace with zero clippy warnings.
- **Empirically Validated**: Tested addon installation into live Blender 5.1/5.2 paths, client configuration detection, and direct CLI command execution.

## [0.2.0] - 2026-10-17

### Added
- **Spatial Compiler Semantic Graph (`rr-csg-0.1+3d`)**:
  - Implemented formal 3D scene semantic graph protocol conforming to `reflective-rust` and `reflective-index` specifications.
  - Added reflective MCP tool `get_scene_csg` and direct CLI command `blender-mcp csg [--source <name>] [--json]`.
  - Typed spatial nodes: `scene`, `object`, `mesh`, `socket`, `collider`, `camera`, `light`, `material`.
  - Typed semantic edges: `contains`, `parents`, `anchors`, `collides_with`, `applies_material`.
- **Zero-Copy Shared Memory Viewport Streaming (`blender-mcp-ipc`)**:
  - Cross-platform memory-mapped ring buffer (`memmap2`) with sub-microsecond latency and zero CPU base64 encoding overhead.
  - Native `FrameHeader` with frame sequence tracking, timestamp nanoseconds, and buffer overflow protection.

### Verification Status
- **Implemented**: `SpatialCsgGraph`, `SharedMemoryViewport`, `get_scene_csg` tool, and `csg` CLI subcommand.
- **Tested**: 10 unit/integration tests passing 100% across workspace with zero clippy warnings.

## [0.1.0] - 2026-10-17

### Added
- **Unified Multi-Agent CLI**: Full suite of direct commands (`serve`, `audit`, `pbr`, `rig`, `craft`, `client`, `export-skills`).
- **One-Command Skill Exporter**: Automatic generation of optimized agent skills for Antigravity, Cursor, Claude Code, and Gemini CLI.
- **Companion Blender Addon**: Pure Python socket daemon bridging Blender 4.0+ viewport into the native Rust runtime.

### Verification Status
- **Implemented**: Complete 6-crate Rust workspace + companion addon.
- **Tested**: 8 unit/integration tests passing 100% with zero warnings under `cargo clippy --workspace --all-targets -- -D warnings`.
- **Empirically Validated**: Verified direct CLI execution for model auditing, PBR ORM packing, skill export, and client configuration.

## [0.1.0-beta.1] - 2026-10-17

### Added
- **Reflective 6-Tool Suite**: Dynamic type reflection and automated JSON Schema derivation for all core pipeline tools:
  - `audit_asset`
  - `pack_pbr_textures`
  - `execute_blender_code`
  - `rig_game_item`
  - `craft_hero_prop`
  - `get_scene_info`
- **Cancellation Token Support**: Instant termination of underlying Blender operations on agent tool cancellation.

## [0.1.0-alpha.1] - 2026-10-17

### Added
- **Native Geometry & Collision Engine (`blender-mcp-geom`)**:
  - Rapier3D / Parry convex hull decomposition (`UCX_`).
  - Axis-aligned bounding box collider calculation (`UBX_`).
  - Bounding capsule collider calculation (`UCP_`).
  - Game-ready glTF model audit with $\le 2$ material budget and UV channel 0 verification.
- **Native PBR Texture Packing (`blender-mcp-pbr`)**:
  - High-throughput channel packing for Ambient Occlusion (R), Roughness (G), and Metallic (B) maps into a single ORM texture.
