# blender-mcp-rs Architecture & System Doctrine

`blender-mcp-rs` replaces legacy Python MCP wrappers with an asynchronous, zero-allocation Rust systems architecture designed for sub-millisecond AI agent interaction.

---

## 1. Dual-Transport IPC Architecture

Legacy Blender MCP servers funnel image payloads through standard I/O (stdio) pipes, causing JSON serialization bloat, buffer truncations, and GIL bottlenecks in Blender Python. 

`blender-mcp-rs` decouples the **Control Plane** from the **Data Plane**:

```
┌────────────────────────┐                    ┌────────────────────────┐
│     AI Agent Host      │                    │     blender-mcp-rs     │
│  (Claude, Antigravity) │◄─── Stdio JSON ───►│      Native Daemon     │
└────────────────────────┘                    └───────────┬────────────┘
                                                          │
                        ┌─────────────────────────────────┴─────────────────────────────────┐
                        │                                                                   │
           1. Control Plane (IPC TCP/UDS)                                     2. Data Plane (SHM Buffer)
                        │                                                                   │
                        ▼                                                                   ▼
       ┌─────────────────────────────────┐                                ┌───────────────────────────────────┐
       │   Command Dispatch & Control    │                                │     Shared Memory Framebuffer     │
       │    (JSON-RPC over localhost)    │                                │    (/tmp/blender_mcp_shm_view)    │
       └────────────────┬────────────────┘                                └─────────────────┬─────────────────┘
                        │                                                                   │
                        ▼                                                                   ▼
       ┌────────────────────────────────────────────────────────────────────────────────────┴────────────────┐
       │                                     Blender Host Instance                                           │
       │                                                                                                     │
       │   • Companion Addon Daemon (bpy)                                                                    │
       │   • Memory-Mapped Ring Buffer Writer                                                                │
       └─────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Control Plane: Asynchronous Unix Domain Socket / TCP
- Handles remote procedure calls (`execute_code`, `apply_scene_csg`, `look`).
- Framed JSON stream with sub-millisecond response latency ($< 0.8\text{ ms}$).

### Data Plane: Zero-Copy Shared Memory Viewport (`SharedMemoryViewport`)
- Uses OS memory-mapped files (`memmap2`) with pre-allocated ring buffer capacity (16MB default).
- Uncompressed raw BGRA/RGBA pixels are written directly into memory by Blender.
- Reader clients consume frames with $\le 1\text{ ms}$ latency at up to 60 FPS in 4K resolution.

#### 48-Byte Frame Header Layout (`SHM_MAGIC: 0x4652414D`):
```
Offset  0: [4 bytes]  magic: u32 = 0x4652414D ('FRAM')
Offset  4: [4 bytes]  padding (alignment)
Offset  8: [8 bytes]  sequence: u64
Offset 16: [4 bytes]  width: u32
Offset 20: [4 bytes]  height: u32
Offset 24: [4 bytes]  channels: u32
Offset 28: [4 bytes]  padding (alignment)
Offset 32: [8 bytes]  timestamp_ns: u64
Offset 40: [8 bytes]  data_size: u64
Offset 48: [...]      Raw Pixel Payload
```

---

## 2. Spatial Semantic Graph (`rr-csg-0.1+3d`)

Rather than treating 3D scenes as loose vertex clouds, `blender-mcp-rs` models scenes as structured Constructive Solid Geometry graphs:
- **Root Nodes**: Empty containers defining object origin and orientation.
- **Mesh Nodes**: Geometric representations linked to modular materials.
- **Socket Anchors (`SOCKET_*`)**: Semantic connection points (e.g. `SOCKET_grip_primary`, `SOCKET_muzzle`).
- **Collision Hulls (`UCX_*`)**: Dedicated physics geometry decoupled from visual rendering.

---

## 3. Sandboxed Execution Doctrine

All arbitrary code execution is strictly governed by `blender_mcp_core::sandbox`:
- Disallows destructive system calls (`os.system`, `subprocess`, `shutil.rmtree`).
- Forbids network sockets outside the designated local loopback address.
- Logs all operations to the trajectory feedback journal.
