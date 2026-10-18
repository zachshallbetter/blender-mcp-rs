---
description: Standards, asset craftsmanship doctrine, latency targets, and zero-compromise engineering rules
globs: ["**/*"]
---

# Standards & Engineering Doctrine

This document defines the core standards, performance budgets, and non-negotiables for `blender-mcp-rs`.

---

## 1. Sub-Millisecond Latency Budget

- Control plane MCP request-to-response latency must remain under **1 millisecond** ($< 1.0\text{ ms}$) on local transport.
- Viewport streaming must sustain **60 FPS** uncompressed BGRA/RGBA frames via zero-copy shared memory (`SharedMemoryViewport`).

---

## 2. Asset Craftsmanship Standard (The "Barrel Protocol")

All 3D tools and asset generators in this repository must enforce:
1. **Material Slot Budget**: $\le 2$ material slots per game asset.
2. **Explicit UV Mapping**: Direct quad UV loop mapping to game texture atlases. Never apply global cylindrical projections to non-cylindrical shapes.
3. **Physical 3D Relief**: Genuine physical geometry (plank grooves, chamfered rims, extruded hardware, 3D raised rivets with metallic highlights).
4. **Collision Hull Standards**: Lightweight convex hulls (`UCX_`), bounding boxes (`UBX_`), or capsules (`UCP_`) powered by Rapier3D / Parry.
5. **glTF Node Tree Preservation**: Modular child nodes parented under a root Empty node. Always export with `export_apply=False`.

---

## 3. Strict Authorship & Licensing Purity

- Project authorship: **Zachary Shallbetter** (`zachshallbetter`).
- Licensing: Dual MIT OR Apache-2.0. Clean-room implementation only.
- Zero references to extraneous commercial software or third-party organizations.
