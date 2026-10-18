# Game Engine Asset Pipeline

`blender-mcp-rs` is engineered for zero-compromise production pipelines targeting Unreal Engine 5, Unity, and custom engines (Dead Hand).

---

## 1. Multi-LOD Generation (`generate_lods_and_collision`)

Automates tiered Levels of Detail (LODs) to maximize runtime rendering performance:

| LOD Tier | Target Ratio | Purpose | Typical Triangle Count |
|---|---|---|---|
| **LOD0** | 1.0 (100%) | Hero / Close-up rendering | ~10,000–25,000 |
| **LOD1** | 0.5 (50%) | Medium distance | ~5,000–12,500 |
| **LOD2** | 0.25 (25%) | Far distance | ~2,500–6,250 |
| **LOD3** | 0.10 (10%) | Extreme silhouette / Impostor | ~1,000–2,500 |

### Usage
```json
{
  "name": "generate_lods_and_collision",
  "arguments": {
    "object_name": "Hero_Barrel",
    "lod_ratios": [1.0, 0.5, 0.25, 0.1],
    "collision_type": "convex_hull"
  }
}
```

---

## 2. Physics Collision Geometry (`UCX_*`)

Following the Unreal Engine naming convention, physics collision hulls are generated automatically and tagged:
- `UCX_<ObjectName>_01`: Convex hull representation for rigid body dynamics.
- `UBX_<ObjectName>_01`: Oriented bounding box collider.
- `UCP_<ObjectName>_01`: Capsule collider.

In Blender, collision hulls are automatically set to `display_type = 'WIRE'` and excluded from rendering layers.

---

## 3. SIMD ORM Texture Packing (`pack_pbr_textures`)

Combines individual grayscale maps into an industry-standard packed ORM map:
- **Red Channel (R)**: Ambient Occlusion (AO)
- **Green Channel (G)**: Roughness
- **Blue Channel (B)**: Metallic

Packing occurs in native Rust SIMD code, processing a 4K texture atlas in under 30 milliseconds.

---

## 4. Procedural Rigging & Skinning (`generate_armature_rig`)

Generates standard bone armatures and binds geometry with automatic vertex group skinning:
- **`prop_container`**: `root` $\to$ `lid` (for chests, crates, barrels).
- **`mechanical_door`**: `root` $\to$ `hinge` (for doors, hatches, cabinets).
- **`biped`**: Root bone hierarchy for character assets.

---

## 5. Asset Auditing & Budgets (`audit_asset`)

Validates `.glb` containers against strict production criteria:
```bash
blender-mcp audit assets/barrel.glb --json
```
- Checks triangle budget ($\le 12,000$ for props).
- Checks material slot budget ($\le 1$ draw call per prop).
- Validates presence of `SOCKET_*` anchors and `UCX_*` colliders.
