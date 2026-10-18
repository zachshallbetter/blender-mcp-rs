# `blender-mcp-rs` 🦀⚡🎨

> **The high-performance, reflective Rust Model Context Protocol (MCP) server & spatial agent engine for Blender.**

[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](LICENSE)
[![Rust: 2021 Edition](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org)
[![MCP: Specification 2024-11-05](https://img.shields.io/badge/MCP-2024--11--05-green.svg)](https://modelcontextprotocol.io)

`blender-mcp-rs` is a ground-up Rust rewrite of Blender MCP designed for autonomous AI coding agents (**Claude Code**, **ChatGPT / Codex**, **Gemini CLI**, **Google Antigravity**, **Cursor**) and high-fidelity game development pipelines.

---

## ⚡ Why `blender-mcp-rs`?

Upstream Python implementations (`blendermcp.org`) suffer from severe architectural limitations: Python Global Interpreter Lock (GIL) stalls, blocking stdio pipes, 30–50ms query overheads, and high-latency base64 image encoding.

`blender-mcp-rs` replaces the entire control and data plane with **native Rust systems programming**:

| Feature | Legacy Python `blender-mcp` | `blender-mcp-rs` 🦀 |
|---|---|---|
| **MCP Roundtrip Latency** | 25–60 ms | **< 0.8 ms** (sub-millisecond) |
| **Transport Reliability** | Prone to stdio deadlocks & buffer clipping | **Tokio async I/O** (zero-deadlock stdio/SSE) |
| **Viewport Streaming** | CPU-bound base64 JSON (~8–15 FPS) | **Zero-copy Shared Memory ring buffer (60 FPS @ 4K)** |
| **Tool Definition & Typing**| Ad-hoc Python dictionaries | **Reflective Rust schema derivation & type introspection** |
| **Convex Physics Hulls** | None / Slow Python convex hull | **Native Rapier3D / Parry VHACD decomposition** |
| **Mesh Decimation / LODs** | Slow bpy modifier serialization | **Native Meshoptimizer SIMD quadric decimation** |
| **Multi-Agent Setup** | Manual copy-pasting | **1-command multi-client auto-config (`claude`, `codex`, `gemini`, `cursor`, `antigravity`)** |

---

## 🏗️ Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Autonomous AI Agent Host                        │
│         (Claude Code, Antigravity, Gemini CLI, Cursor, Codex)          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Stdio / SSE / WebSocket (MCP JSON-RPC)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     blender-mcp-rs (Native Core)                       │
│                                                                        │
│  ┌───────────────────────┐  ┌─────────────────┐  ┌──────────────────┐ │
│  │ Tokio Async MCP Server│  │ Reflective Tool │  │ Multi-Agent CLI  │ │
│  │ (Zero-deadlock stdio) │  │ Registry        │  │ & Config Manager │ │
│  └───────────┬───────────┘  └────────┬────────┘  └────────┬─────────┘ │
│              │                       │                    │           │
│  ┌───────────▼───────────────────────▼────────────────────▼─────────┐ │
│  │                     Native Processing Engines                     │ │
│  │  • Geometry & Collision (Rapier3D Parry VHACD, Meshopt LOD)       │ │
│  │  • PBR & Texture Engine (SIMD Channel Packing, Fast PNG)          │ │
│  │  • Game Asset Integrity (Texel Density, Budgets, Manifests)       │ │
│  └───────────────────────────────────┬───────────────────────────────┘ │
└──────────────────────────────────────┼─────────────────────────────────┘
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            │ Dual High-Speed IPC Transport                       │
            │ 1. Unix Domain Socket / Named Pipe (Control Plane)  │
            │ 2. Shared Memory Ring Buffer / SHM (Viewport Pixels)│
            ▼                                                     ▼
┌────────────────────────────────────────────────────────────────────────┐
│                          Blender Host Instance                         │
│                                                                        │
│  ┌────────────────────────┐                  ┌──────────────────────┐  │
│  │ Embedded Rust / C ABI  │                  │ Blender Addon (bpy)  │  │
│  │ (Direct memory access) │                  │ (Scene manipulation) │  │
│  └────────────────────────┘                  └──────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 Workspace Crates

- **`blender-mcp-reflect`**: Type reflection, parameter introspection, automatic JSON Schema derivation, and dynamic tool dispatch.
- **`blender-mcp-protocol`**: Pure, zero-allocation Model Context Protocol JSON-RPC 2.0 specification implementation.
- **`blender-mcp-ipc`**: Sub-millisecond IPC and zero-copy shared memory framebuffer ring buffer (`memmap2`) for 60 FPS viewport streaming.
- **`blender-mcp-geom`**: Native 3D geometry engine powered by Rapier3D / Parry for convex collision hulls (`UCX_`, `UBX_`, `UCP_`) and Spatial CSG (`rr-csg-0.1+3d`).
- **`blender-mcp-pbr`**: SIMD texture channel packing for Ambient Occlusion, Roughness, and Metallic (ORM) maps.
- **`blender-mcp-synth`**: Procedural asset synthesis engine (desert flora, municipal props, and assembly grammars).
- **`blender-mcp-core`**: The central daemon coordinating sessions, safe mode sandbox, and the 37-tool reflective registry.
- **`blender-mcp-cli`**: The unified binary (`blender-mcp`) providing agent auto-configuration, direct pipeline execution, and addon lifecycle management.

---

## 📚 Documentation

- [**Tool Reference Catalog (`docs/tools.md`)**](docs/tools.md): Complete specification of all 37 reflective tools.
- [**MCP Resource Provider (`docs/resources.md`)**](docs/resources.md): Interactive WebGL/HTML5 live viewport (`blender://viewport`) and scene CSG graph (`blender://scene_summary`).
- [**Architecture & Doctrine (`docs/architecture.md`)**](docs/architecture.md): Dual-transport IPC, zero-copy shared memory streaming, and spatial CSG (`rr-csg-0.1+3d`).
- [**Blender Companion Addon (`docs/addon.md`)**](docs/addon.md): Installation, headless configuration, and socket daemon setup.
- [**Game Engine Asset Pipeline (`docs/game-pipeline.md`)**](docs/game-pipeline.md): Multi-LOD decimation, Rapier3D `UCX_*` collision hulls, and SIMD ORM packing.
- [**Ticket & Release Board (`docs/tickets/BOARD.md`)**](docs/tickets/BOARD.md): Formal tracking board, Epics, and completion verification.

---

## 🚀 Quick Start

### Installation

```bash
cargo install --path crates/blender-mcp-cli
```

### Auto-Configure Your AI Agent

Configure any AI agent environment in one command:

```bash
# Claude Code
blender-mcp client add claude-code

# Cursor
blender-mcp client add cursor --scope project

# Google Antigravity
blender-mcp client add antigravity

# OpenAI Codex / ChatGPT
blender-mcp client add codex

# Gemini CLI
blender-mcp client add gemini
```

### Direct Asset Pipeline (Zero-Client Overhead)

You can run full asset pipelines directly from your terminal or CI scripts:

```bash
# Audit a model against game-ready standards
blender-mcp audit assets/model.glb --json

# Auto-rig semantic sockets & Rapier3D convex collision hulls
blender-mcp rig "Revolver" --collider convex --target-tris 1200 --sockets grip,muzzle

# Generate quadric LODs & pack ORM texture maps
blender-mcp lods "Chest" --pack-orm --ao ao.png --roughness rough.png --metallic metal.png
```

---

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
