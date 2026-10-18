# BMCP-006: Game-Ready Multi-LOD Generation & Collision Hull Pipeline

**Epic:** EPIC-6 (Multi-LOD & Collision Hull Pipeline)  
**Owner:** A (Autonomous Agent)  
**Status:** done  
**Target:** 2026-10-17  

---

## 1. Problem Statement
Production game pipelines (Dead Hand, Unreal Engine 5, Unity) require tiered Levels of Detail (LOD0 through LOD3) and standard convex collision hulls (`UCX_`, `UBX_`, `UCP_`) for rigid bodies. Currently, `blender-mcp-geom` has collision hull computation in Rust, but it is not exposed as an integrated Blender pipeline tool.

## 2. Scope of Changes
1. `blender-mcp-core`:
   - Register `generate_lods_and_collision` tool accepting:
     - `object_name`: String
     - `lod_ratios`: Vec<f32> (e.g. `[1.0, 0.5, 0.25, 0.1]`)
     - `collision_type`: "convex_hull", "box", "capsule", or "none"
2. `addon/blender_mcp/__init__.py`:
   - Implement `generate_lods_and_collision` handler:
     - Duplicates mesh to `LOD0`, `LOD1`, `LOD2`, `LOD3` with Decimate modifiers.
     - Generates physics collision geometry named `UCX_<object_name>` with invisible wireframe display.
3. Verification:
   - Unit tests validating parameters and command formatting.
