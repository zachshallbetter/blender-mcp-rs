# BMCP-003: Live Shared Memory Viewport Streaming Hook in Companion Addon

**Epic:** EPIC-3 (Live Shared Memory Viewport Streaming)  
**Owner:** A (Autonomous Agent)  
**Status:** todo  
**Target:** 2026-10-17  

---

## 1. Problem Statement
The Rust crate `blender-mcp-ipc` implements a sub-microsecond `SharedMemoryViewport` memory-mapped ring buffer using `memmap2`. However, the companion Blender addon currently relies on writing snapshot files to `/tmp/blender_mcp_snapshot.png`.

## 2. Scope of Changes
1. `addon/blender_mcp/__init__.py`:
   - Add a background modal timer or post-render draw handler utilizing Python's `mmap` module or ctypes.
   - Open `/tmp/blender_mcp_shm_viewport` (or platform equivalent).
   - Write raw BGRA/RGBA pixels directly to the shared memory buffer with the 48-byte `FrameHeader` (`magic: 0x4652414D`).
2. Add IPC command `start_viewport_stream` / `stop_viewport_stream` to toggle 60 FPS viewport rendering.
3. Verification:
   - Roundtrip test between Blender Python mmap writer and Rust `SharedMemoryViewport` reader.
