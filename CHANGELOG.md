# Changelog

All notable changes to `blender-mcp-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) under the governance of `VERSIONING.md` and `AUTHORITY.md`.

## [0.1.0] - 2026-10-18

### Architecture & Ground-Up Rebuild
- **Rust Core Workspace**: Replaced legacy Python server implementation with a decoupled, high-performance Rust workspace.
  - `blender-mcp-reflect`: Type reflection, schema introspection via `schemars`, and zero-boilerplate typed tool registration.
  - `blender-mcp-protocol`: Pure MCP JSON-RPC 2.0 specification implementation adhering to spec `2024-11-05`.
  - `blender-mcp-ipc`: Dual-mode control transport and shared-memory framebuffer interface.
  - `blender-mcp-geom`: Native 3D geometry engine powered by Rapier3D / Parry for instant convex hull collision generation (`UCX_`) and glTF inspection.
  - `blender-mcp-core`: Central coordination daemon managing tool dispatch and cancellation tokens.
  - `blender-mcp-cli`: Unified binary (`blender-mcp`) with multi-agent client auto-configurator (`claude-code`, `antigravity`, `cursor`, `codex`, `gemini`).

### Foundations & Protocol Citations
- Integrated `formal-project-bootstrap` governance, authority hierarchy, and release diligence.
- Integrated `reflective-rust` and `reflective-index` architecture for typed schema reflection and local semantic indexing.
- Implemented `Dead Hand` asset craftsmanship standard for game-ready model audits.

### Verification Status
- **Implemented**: All 6 core workspace crates compiled and linked.
- **Tested**: Automated workspace test suite passing with 0 failures (`cargo test --workspace`).
