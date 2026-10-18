# blender-mcp-rs Ticket Board

The work in flight for `blender-mcp-rs`, who owns it, and what "done" means.
Namespace: `zachshallbetter` (`https://github.com/zachshallbetter/blender-mcp-rs`).

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

---

## Active Ticket Board

| # | Epic | Ticket | Owner | Status | Done When |
|---|---|---|---|---|---|
| **BMCP-001** | EPIC-1 | **Standardize namespace to `zachshallbetter` across entire repository.** Remove all lingering third-party or team names. Update `Cargo.toml`, `CITATION.cff`, `.zenodo.json`, `AGENTS.md`, `README.md`, `addon`, `synth`, and `llms.txt`. | A | done | All references point strictly to `https://github.com/zachshallbetter/blender-mcp-rs` and `Zachary Shallbetter`; 0 occurrences of extraneous organization names. |
| **BMCP-002** | EPIC-2 | **Polyhaven CC0 Live Asset Search & HTTP Downloader.** Wire real HTTP queries to `https://api.polyhaven.com/assets` in `search_assets` and automated download/import into Blender in `import_asset`. | A | todo | `search_assets(source="polyhaven", query="wood")` returns live assets; `import_asset` downloads and creates PBR material or model in live Blender. |
| **BMCP-003** | EPIC-3 | **Live Shared Memory Viewport Streaming Hook in Companion Addon.** Implement `gpu.types.GPUOffScreen` frame writing into `SharedMemoryViewport` memory-mapped ring buffer in `addon/blender_mcp`. | A | todo | Addon writes uncompressed BGRA/RGBA frames directly to memory map; client reads frames at 60 FPS with $\le 1$ ms latency. |
| **BMCP-004** | EPIC-4 | **Live Spatial CSG Boolean & Socket Snapping Tool.** Implement `apply_scene_csg` tool in `blender-mcp-core` allowing boolean union/difference/intersection and snapping objects to semantic sockets (`SOCKET_*`). | A | todo | Tool executes boolean modifier or matrix transform snapping object A's socket to object B's socket; empirically verified with tests. |
| **BMCP-005** | EPIC-5 | **Interactive Web/HTML Viewport Resource (`blender://viewport`).** Expose MCP resource `blender://viewport` returning live HTML5 Canvas/WebGL webview for client UIs. | A | todo | Server serves `resources/read` for `blender://viewport` with embedded interactive viewer and pick raycasting. |

---

## Completed Tickets

- **BMCP-001** (2026-10-17): Standardized all repository URLs (`https://github.com/zachshallbetter/blender-mcp-rs`), authors, manifests, citations, and doc comments strictly under `zachshallbetter` / `Zachary Shallbetter`.
