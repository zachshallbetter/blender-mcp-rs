# BMCP-007: Procedural Armature & Skinning Pipeline

**Epic:** EPIC-7 (Rigging & Animation Retargeting Tools)  
**Owner:** A (Autonomous Agent)  
**Issue:** [#7](https://github.com/zachshallbetter/blender-mcp-rs/issues/7)  
**Status:** done  
**Target:** 2026-10-17  

---

## 1. Problem Statement
Character and prop animation in 3D game pipelines requires standard bone armatures and automatic vertex group assignment (heat weighting / envelope weighting). The existing `rig_game_item` tool only sets up basic mechanical parenting.

## 2. Scope of Changes
1. `blender-mcp-core`:
   - Enhance `rig_game_item` tool or add `generate_armature_rig` accepting:
     - `target_object`: String
     - `rig_template`: "biped", "quadruped", "mechanical_door", "prop_container"
     - `bind_mesh`: bool
2. `addon/blender_mcp/__init__.py`:
   - Implement armature generation with bone chains and optional Automatic Weights parenting (`ARMATURE_AUTO`).
3. Verification:
   - Unit tests for parameter verification and syntax checking.
