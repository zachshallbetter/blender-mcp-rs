# blender-mcp-rs Ticket Board

Namespace: `zachshallbetter` (`https://github.com/zachshallbetter/blender-mcp-rs`).  
Project Board: [GitHub Project #41 (blender-mcp-rs)](https://github.com/users/zachshallbetter/projects/41)

Owners:
- **Z**: Zachary Shallbetter (`zachshallbetter`)
- **A**: Autonomous Agent (Antigravity)

Statuses:
- **todo**: Backlogged and specified
- **doing**: Active work in progress
- **review**: Built, tests passing, awaiting final signoff
- **done**: Committed, tagged, and empirically validated
- **blocked**: Awaiting dependency or external blocker

---

## Epics Overview

1. **EPIC-1: Namespace & Attribution Standardization (`/zachshallbetter`)**  
   Ensure all package metadata, repository URLs, documentation, CFF citations, Zenodo entries, manifests, and author fields are strictly attributed to `zachshallbetter`.
2. **EPIC-2: Live Asset Library Integration (Polyhaven, Sketchfab, Polypizza)**  
   Implement real asynchronous HTTP querying and automated downloading/material creation in Blender for CC0 models, HDRIs, and textures.
3. **EPIC-3: Live Shared Memory Viewport Streaming Hook**  
   Bridge Blender's GPU offscreen frame buffer into `SharedMemoryViewport` (`memmap2`) for 60 FPS zero-copy viewport streaming.
4. **EPIC-4: Live Spatial CSG Boolean Operations & Socket Snapping**  
   Add interactive CSG mutation capabilities (`apply_scene_csg`) to execute boolean operations and snap modular components to sockets.
5. **EPIC-5: Interactive Web/HTML Viewport Resource (`blender://viewport`)**  
   Implement MCP resource and tool endpoints for interactive web-based viewport viewing and pick raycasting.
6. **EPIC-6: Multi-LOD Generation & Collision Hull Pipeline**  
   Generate tiered Levels of Detail (LOD0-3) and game-ready convex/box colliders (`UCX_*`).
7. **EPIC-7: Rigging & Animation Retargeting Tools**  
   Generate standardized armatures with bone hierarchies and automatic vertex group skinning.

---

## Active Ticket Board

| # | Epic | Ticket | Owner | Status | Done When |
|---|---|---|---|---|---|
| [**BMCP-001**](https://github.com/zachshallbetter/blender-mcp-rs/issues/1) | EPIC-1 | **Standardize namespace to `zachshallbetter` across entire repository.** Remove all lingering third-party or team names. Update `Cargo.toml`, `CITATION.cff`, `.zenodo.json`, `AGENTS.md`, `README.md`, `addon`, `synth`, and `llms.txt`. | A | done | All references point strictly to `https://github.com/zachshallbetter/blender-mcp-rs` and `Zachary Shallbetter`; 0 occurrences of extraneous organization names. |
| [**BMCP-002**](https://github.com/zachshallbetter/blender-mcp-rs/issues/2) | EPIC-2 | **Polyhaven CC0 Live Asset Search & HTTP Downloader.** Wire real HTTP queries to `https://api.polyhaven.com/assets` in `search_assets` and automated download/import into Blender in `import_asset`. | A | done | `search_assets(source="polyhaven", query="wood")` returns live assets; `import_asset` downloads and creates PBR material or model in live Blender. |
| [**BMCP-003**](https://github.com/zachshallbetter/blender-mcp-rs/issues/3) | EPIC-3 | **Live Shared Memory Viewport Streaming Hook in Companion Addon.** Implement frame writing into `SharedMemoryViewport` memory-mapped ring buffer in `addon/blender_mcp` with `start_viewport_stream`, `stop_viewport_stream`, and `read_viewport_frame`. | A | done | Addon writes uncompressed frames directly to memory map; client reads frames with sub-microsecond latency. |
| [**BMCP-004**](https://github.com/zachshallbetter/blender-mcp-rs/issues/4) | EPIC-4 | **Live Spatial CSG Boolean & Socket Snapping Tool.** Implement `apply_scene_csg` tool in `blender-mcp-core` allowing boolean union/difference/intersection and snapping objects to semantic sockets (`SOCKET_*`). | A | done | Tool executes boolean modifier or matrix transform snapping object A's socket to object B's socket; verified with test suite. |
| [**BMCP-005**](https://github.com/zachshallbetter/blender-mcp-rs/issues/5) | EPIC-5 | **Interactive Web/HTML Viewport Resource (`blender://viewport`).** Expose MCP resource `blender://viewport` returning live HTML5 Canvas/WebGL webview for client UIs. | A | done | Server serves `resources/list` and `resources/read` for `blender://viewport` with embedded interactive viewer and pick raycasting. |
| [**BMCP-006**](https://github.com/zachshallbetter/blender-mcp-rs/issues/6) | EPIC-6 | **Game-Ready Multi-LOD Generation & Collision Hull Pipeline.** Implement `generate_lods_and_collision` tool for automatic decimation and `UCX_*` physics colliders. | A | done | Automatically creates LOD0-3 mesh hierarchy with decimate modifiers and convex collision hulls. |
| [**BMCP-007**](https://github.com/zachshallbetter/blender-mcp-rs/issues/7) | EPIC-7 | **Procedural Armature & Skinning Pipeline.** Implement `generate_armature_rig` for bone hierarchies and automatic weights parenting. | A | done | Generates standard biped/quadruped/mechanical bone rigs with vertex groups bound to geometry. |

---

## Completed Tickets

- **BMCP-001** (2026-10-17): Standardized all repository URLs (`https://github.com/zachshallbetter/blender-mcp-rs`), authors, manifests, citations, and doc comments strictly under `zachshallbetter` / `Zachary Shallbetter`.
- **BMCP-002** (2026-10-17): Implemented live Polyhaven CC0 asset querying (`search_assets`) and automated download & GLTF/texture scene loading (`import_asset`) in `crates/blender-mcp-core/src/polyhaven.rs`.
- **BMCP-003** (2026-10-17): Implemented high-frequency memory-mapped viewport streaming in `addon/blender_mcp` (`start_viewport_stream`, `stop_viewport_stream`) and zero-copy reader tool (`read_viewport_frame`) in `crates/blender-mcp-core`.
- **BMCP-004** (2026-10-17): Implemented live CSG operations tool (`apply_scene_csg`) for boolean difference, union, intersection, and socket matrix snapping in both Rust core and companion addon.
- **BMCP-005** (2026-10-17): Implemented MCP Resource protocols (`resources/list` and `resources/read`) exposing `blender://viewport` with an interactive HTML5/Canvas 3D viewport webview and `blender://scene_summary`.
- **BMCP-006** (2026-10-17): Implemented game-ready LOD pipeline (`generate_lods_and_collision`) with automated mesh decimation (LOD0-3) and physics collision hulls (`UCX_*`).
- **BMCP-007** (2026-10-17): Implemented procedural armature generator (`generate_armature_rig`) supporting prop containers, doors, and biped bone chains with vertex group skin binding.
