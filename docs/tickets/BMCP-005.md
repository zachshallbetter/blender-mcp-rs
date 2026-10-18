# BMCP-005: Interactive Web/HTML Viewport Resource (`blender://viewport`)

**Epic:** EPIC-5 (Interactive Web/HTML Viewport Resource)  
**Owner:** A (Autonomous Agent)  
**Status:** todo  
**Target:** 2026-10-17  

---

## 1. Problem Statement
Clients supporting MCP Apps / visual UI (such as ChatGPT, Claude Desktop, Antigravity) can render interactive HTML/WebGL canvases when exposed as an MCP Resource with URI `blender://viewport`.

## 2. Scope of Changes
1. `blender-mcp-protocol` & `blender-mcp-core`:
   - Implement `resources/list` returning `blender://viewport` ("Live 3D Viewport").
   - Implement `resources/read` returning interactive HTML viewer with canvas and camera orbit / raycast pick triggers.
2. Tools:
   - Wire `open_viewport`, `viewport_latest`, `viewport_capture`, and `viewport_pick` endpoints.
3. Verification:
   - Test JSON-RPC requests for `resources/list` and `resources/read`.
