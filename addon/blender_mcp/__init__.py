bl_info = {
    "name": "blender-mcp-rs Companion",
    "author": "Zachary Shallbetter / SnackStudio",
    "version": (0, 1, 0),
    "blender": (4, 0, 0),
    "location": "View3D > Sidebar > MCP-RS",
    "description": "Companion daemon for blender-mcp-rs native Rust spatial agent server",
    "category": "Development",
}

import bpy
import json
import socket
import threading
import math
import os

_server_sock = None
_server_thread = None
_running = False

import mathutils
import tempfile

def handle_client(conn):
    buffer = b""
    try:
        while _running:
            data = conn.recv(8192)
            if not data:
                break
            buffer += data
            if b"\n" in buffer:
                line, buffer = buffer.split(b"\n", 1)
                req = json.loads(line.decode("utf-8"))
                res = dispatch_command(req)
                conn.sendall(json.dumps(res).encode("utf-8") + b"\n")
    except Exception as e:
        pass
    finally:
        conn.close()

def dispatch_command(req):
    cmd_type = req.get("type")
    params = req.get("params", {})

    if cmd_type == "get_scene_info":
        return get_scene_info()
    elif cmd_type == "execute_code":
        return execute_code(params.get("code", ""))
    elif cmd_type == "rig_game_item":
        return rig_game_item(params)
    elif cmd_type == "craft_hero_prop":
        return craft_hero_prop(params)
    elif cmd_type == "look":
        return look_at(params)
    elif cmd_type == "render_viewport_snapshot":
        return render_viewport_snapshot(params)
    else:
        return {"status": "error", "message": f"Unknown command: {cmd_type}"}

def look_at(params):
    target_name = params.get("target", "active")
    dist = params.get("distance", 3.0)
    azimuth = math.radians(params.get("azimuth", 45.0))
    elevation = math.radians(params.get("elevation", 30.0))

    target_obj = bpy.context.active_object if target_name == "active" else bpy.data.objects.get(target_name)
    target_loc = target_obj.location if target_obj else mathutils.Vector((0.0, 0.0, 0.0))

    cam = bpy.context.scene.camera
    if not cam:
        cam_data = bpy.data.cameras.new("Agent_Camera")
        cam = bpy.data.objects.new("Agent_Camera", cam_data)
        bpy.context.scene.collection.objects.link(cam)
        bpy.context.scene.camera = cam

    x = target_loc.x + dist * math.cos(elevation) * math.sin(azimuth)
    y = target_loc.y - dist * math.cos(elevation) * math.cos(azimuth)
    z = target_loc.z + dist * math.sin(elevation)
    cam.location = (x, y, z)

    direction = target_loc - cam.location
    rot_quat = direction.to_track_quat('-Z', 'Y')
    cam.rotation_euler = rot_quat.to_euler()

    return {
        "status": "success",
        "camera": cam.name,
        "location": list(cam.location),
        "target": target_name,
        "distance": dist,
    }

def render_viewport_snapshot(params):
    width = params.get("width", 1080)
    height = params.get("height", 1080)

    bpy.context.scene.render.resolution_x = width
    bpy.context.scene.render.resolution_y = height
    bpy.context.scene.render.image_settings.file_format = 'PNG'

    temp_path = os.path.join(tempfile.gettempdir(), "blender_mcp_snapshot.png")
    bpy.context.scene.render.filepath = temp_path
    bpy.ops.render.opengl(write_still=True)

    file_size = os.path.getsize(temp_path) if os.path.exists(temp_path) else 0

    return {
        "status": "success",
        "snapshot_path": temp_path,
        "width": width,
        "height": height,
        "size_bytes": file_size,
    }

def get_scene_info():
    objects = []
    for obj in bpy.context.scene.objects:
        obj_data = {
            "name": obj.name,
            "type": obj.type,
            "location": list(obj.location),
            "rotation": list(obj.rotation_euler),
            "scale": list(obj.scale),
        }
        if obj.type == 'MESH' and obj.data:
            obj_data["vertices"] = len(obj.data.vertices)
            obj_data["polygons"] = len(obj.data.polygons)
            obj_data["materials"] = [slot.material.name for slot in obj.material_slots if slot.material]
        objects.append(obj_data)

    return {
        "status": "success",
        "objects": objects,
        "object_count": len(objects),
        "active_object": bpy.context.active_object.name if bpy.context.active_object else None,
    }

def execute_code(code):
    try:
        local_scope = {"bpy": bpy}
        exec(code, local_scope)
        return {"status": "success", "result": "Code executed successfully"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

def rig_game_item(params):
    obj_name = params.get("object_name")
    collider_type = params.get("collider_type", "convex")
    sockets = params.get("sockets", [])

    obj = bpy.data.objects.get(obj_name)
    if not obj:
        return {"status": "error", "message": f"Object '{obj_name}' not found"}

    created_sockets = []
    for s_name in sockets:
        empty = bpy.data.objects.new(s_name, None)
        empty.empty_display_type = 'ARROWS'
        empty.empty_display_size = 0.2
        empty.parent = obj
        bpy.context.scene.collection.objects.link(empty)
        created_sockets.append(s_name)

    # Collision hull generation
    prefix = "UCX_" if collider_type == "convex" else ("UBX_" if collider_type == "box" else "UCP_")
    hull_name = f"{prefix}{obj_name}_01"

    hull_obj = bpy.data.objects.get(hull_name)
    if not hull_obj and obj.type == 'MESH':
        hull_mesh = obj.data.copy()
        hull_obj = bpy.data.objects.new(hull_name, hull_mesh)
        hull_obj.parent = obj
        hull_obj.display_type = 'WIRE'
        bpy.context.scene.collection.objects.link(hull_obj)

    return {
        "status": "success",
        "object": obj_name,
        "sockets_created": created_sockets,
        "collider_node": hull_name,
        "collider_type": collider_type,
    }

def craft_hero_prop(params):
    archetype = params.get("archetype", "western_barrel")
    output_dir = params.get("output_dir", "./assets")
    os.makedirs(output_dir, exist_ok=True)

    # Create root container empty
    root_name = f"Hero_{archetype.capitalize()}"
    root = bpy.data.objects.new(root_name, None)
    bpy.context.scene.collection.objects.link(root)

    # Procedural geometry adhering to Barrel Protocol
    bpy.ops.mesh.primitive_cylinder_add(radius=0.4, depth=1.0, vertices=16)
    mesh_obj = bpy.context.active_object
    mesh_obj.name = f"{root_name}_Mesh"
    mesh_obj.parent = root

    # Semantic grip socket
    grip = bpy.data.objects.new("SOCKET_grip_primary", None)
    grip.empty_display_type = 'ARROWS'
    grip.parent = root
    bpy.context.scene.collection.objects.link(grip)

    # Collision hull
    hull = bpy.data.objects.new(f"UCX_{root_name}_01", mesh_obj.data.copy())
    hull.parent = root
    hull.display_type = 'WIRE'
    bpy.context.scene.collection.objects.link(hull)

    out_glb = os.path.join(output_dir, f"{archetype}.glb")
    return {
        "status": "success",
        "archetype": archetype,
        "root_node": root_name,
        "mesh_node": mesh_obj.name,
        "sockets": ["SOCKET_grip_primary"],
        "collider": hull.name,
        "output_path": out_glb,
    }

def start_server(host="127.0.0.1", port=9876):
    global _server_sock, _server_thread, _running
    if _running:
        return

    _server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    _server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    _server_sock.bind((host, port))
    _server_sock.listen(5)
    _running = True

    def listen_loop():
        while _running:
            try:
                conn, _ = _server_sock.accept()
                threading.Thread(target=handle_client, args=(conn,), daemon=True).start()
            except Exception:
                break

    _server_thread = threading.Thread(target=listen_loop, daemon=True)
    _server_thread.start()
    print(f"[blender-mcp-rs] Server listening on {host}:{port}")

def stop_server():
    global _server_sock, _running
    _running = False
    if _server_sock:
        try:
            _server_sock.close()
        except Exception:
            pass
        _server_sock = None

class BLENDER_MCP_OT_start(bpy.types.Operator):
    bl_idname = "blender_mcp.start_server"
    bl_label = "Start blender-mcp-rs Server"

    def execute(self, context):
        start_server()
        self.report({'INFO'}, "blender-mcp-rs server started on 127.0.0.1:9876")
        return {'FINISHED'}

class BLENDER_MCP_OT_stop(bpy.types.Operator):
    bl_idname = "blender_mcp.stop_server"
    bl_label = "Stop blender-mcp-rs Server"

    def execute(self, context):
        stop_server()
        self.report({'INFO'}, "blender-mcp-rs server stopped")
        return {'FINISHED'}

class BLENDER_MCP_PT_panel(bpy.types.Panel):
    bl_label = "blender-mcp-rs"
    bl_idname = "BLENDER_MCP_PT_panel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'MCP-RS'

    def draw(self, context):
        layout = self.layout
        layout.label(text="Rust Native Agent Bridge")
        if _running:
            layout.operator("blender_mcp.stop_server", text="Stop Server", icon='PAUSE')
        else:
            layout.operator("blender_mcp.start_server", text="Start Server", icon='PLAY')

def register():
    bpy.utils.register_class(BLENDER_MCP_OT_start)
    bpy.utils.register_class(BLENDER_MCP_OT_stop)
    bpy.utils.register_class(BLENDER_MCP_PT_panel)

def unregister():
    stop_server()
    bpy.utils.unregister_class(BLENDER_MCP_PT_panel)
    bpy.utils.unregister_class(BLENDER_MCP_OT_stop)
    bpy.utils.unregister_class(BLENDER_MCP_OT_start)

if __name__ == "__main__":
    register()
