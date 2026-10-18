# BMCP-004: Live Spatial CSG Boolean & Socket Snapping Tool

**Epic:** EPIC-4 (Live Spatial CSG Boolean Operations & Socket Snapping)  
**Owner:** A (Autonomous Agent)  
**Issue:** [#4](https://github.com/zachshallbetter/blender-mcp-rs/issues/4)  
**Status:** done  
**Target:** 2026-10-17  

---

## 1. Problem Statement
The server can read the scene semantic graph via `get_scene_csg` (`rr-csg-0.1+3d`), but does not have a dedicated tool to apply CSG operations (difference, union, intersect) or perform modular socket snapping between objects.

## 2. Scope of Changes
1. `blender-mcp-core`:
   - Register `apply_scene_csg` tool accepting:
     - `operation`: "union", "difference", "intersect", or "snap_to_socket"
     - `target_object`: String
     - `operand_object`: String
     - `socket_name`: Option<String>
2. `addon/blender_mcp/__init__.py`:
   - Implement `apply_scene_csg` handler in Blender Python using `BOOLEAN` modifier (Exact/Fast solver) or matrix transformation matching target socket world transform.
3. Verification:
   - Add unit tests verifying command dispatch and CSG parameter validation.
