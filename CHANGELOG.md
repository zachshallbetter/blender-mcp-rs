# Changelog

All notable changes to `blender-mcp-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) under the governance of `VERSIONING.md` and `AUTHORITY.md`.

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
