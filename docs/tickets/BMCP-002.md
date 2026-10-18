# BMCP-002: Polyhaven CC0 Live Asset Search & HTTP Downloader

**Epic:** EPIC-2 (Live Asset Library Integration)  
**Owner:** A (Autonomous Agent)  
**Status:** todo  
**Target:** 2026-10-17  

---

## 1. Problem Statement
Currently, `search_assets` and `import_asset` provide mock proxy responses. Users expect live searching and downloading of CC0 textures, models, and HDRIs from Polyhaven without needing API keys.

## 2. Scope of Changes
1. `crates/blender-mcp-core`:
   - Implement `polyhaven.rs` HTTP client querying `https://api.polyhaven.com/assets` with category, type, and search filters.
   - Cache results in memory / local temp cache.
2. `import_asset`:
   - Query Polyhaven file download URL for specified resolution (1k, 2k, 4k).
   - Download `.gltf` / `.hdr` / `.png` texture maps to local cache.
   - Dispatch `bpy` script to live Blender instance via IPC to import model or build Principled BSDF PBR material.
3. Verification:
   - Add unit tests for Polyhaven search response parsing.
   - Test live roundtrip with mock and network fallbacks.
