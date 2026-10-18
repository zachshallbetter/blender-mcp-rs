---
name: blender-mcp
description: High-performance reflective 3D spatial agent runtime for Blender. Use for auditing models, rigging sockets & Rapier3D colliders, packing ORM textures, and crafting game props.
---

# blender-mcp Instructions

Use the `blender-mcp` MCP server tools:
1. `audit_asset`: Empirically audit a .glb asset for polycount and material budget.
2. `rig_game_item`: Auto-place semantic sockets (`SOCKET_grip_primary`, `SOCKET_muzzle`) and generate physics colliders (`UCX_`, `UBX_`, `UCP_`).
3. `pack_pbr_textures`: Pack AO, Roughness, and Metallic textures into game-ready ORM maps.
4. `craft_hero_prop`: Procedurally craft hero props conforming to the 7-step Barrel Protocol.
5. `execute_blender_code`: Execute live Python code in Blender.
6. `get_scene_info`: Introspect the real-time Blender scene.
