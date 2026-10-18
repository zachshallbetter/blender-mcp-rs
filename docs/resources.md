# blender-mcp-rs Resource Architecture

`blender-mcp-rs` implements the Model Context Protocol (MCP) **Resources** specification (`resources/list` and `resources/read`), enabling AI host applications (Claude Desktop, ChatGPT / Codex, Google Antigravity, Cursor) to directly display live visual interfaces and structured scene contexts.

---

## Available Resources

| URI | Name | MIME Type | Description |
|---|---|---|---|
| `blender://viewport` | Blender Live Viewport | `text/html` | Interactive HTML5/WebGL canvas viewer with mouse orbit, pan, zoom, snapshot triggers, and raycast picking. |
| `blender://scene_summary` | Blender Scene CSG Summary | `application/json` | JSON serialization of the active Blender scene hierarchy, materials, cameras, and semantic sockets. |

---

## 1. `blender://viewport` (Interactive Web Viewport)

### Overview
`blender://viewport` serves an embedded HTML5 WebGL application designed to render inside AI agent webview panels, desktop modal containers, or embedded browser frames.

### Features
- **Mouse & Trackpad Camera Orbit**: Click-and-drag orbits azimuth (0°–360°) and elevation (-89° to +89°).
- **Mouse Wheel Zoom**: Smooth distance zoom control (0.5m to 50.0m).
- **Preset Camera Orientations**:
  - **Hero 3/4**: Azimuth 45°, Elevation 30°
  - **Front**: Azimuth 0°, Elevation 0°
  - **Top**: Azimuth 0°, Elevation 89°
  - **Side**: Azimuth 90°, Elevation 0°
- **Snapshot Trigger**: Dispatches camera captures directly to the MCP server.
- **Bi-Directional PostMessage API**: Communicates with the parent host frame via standard JSON-RPC tool dispatch messages (`look`, `render_viewport_snapshot`, `focus_object`).

### Client Integration Example

When querying via JSON-RPC:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/read",
  "params": {
    "uri": "blender://viewport"
  }
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "contents": [
      {
        "uri": "blender://viewport",
        "mimeType": "text/html",
        "text": "<!DOCTYPE html>..."
      }
    ]
  }
}
```

---

## 2. `blender://scene_summary` (Scene CSG Context)

### Overview
`blender://scene_summary` delivers a structured semantic view of the active Blender scene. Agents read this resource to understand the spatial layout before performing complex operations.

### Data Structure
```json
{
  "status": "success",
  "objects": [
    {
      "name": "Hero_Barrel",
      "type": "MESH",
      "location": [0.0, 0.0, 0.0],
      "dimensions": [0.8, 0.8, 1.2],
      "materials": ["M_Wood_Atlas"],
      "sockets": ["SOCKET_grip_primary"],
      "colliders": ["UCX_Hero_Barrel_01"]
    }
  ],
  "active_camera": "Agent_Camera",
  "unit_settings": {
    "system": "METRIC",
    "scale": 1.0
  }
}
```
