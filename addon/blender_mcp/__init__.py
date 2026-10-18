bl_info = {
    "name": "blender-mcp-rs Companion",
    "author": "Zachary Shallbetter",
    "version": (0, 4, 0),
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
import io
import tempfile
import traceback
from contextlib import redirect_stdout, suppress

_server_sock = None
_server_thread = None
_running = False

import mathutils

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
    except Exception:
        pass
    finally:
        conn.close()

def dispatch_command(req):
    cmd_type = req.get("type") or req.get("command")
    params = req.get("params", {})

    if cmd_type == "get_addon_info":
        return get_addon_info()
    elif cmd_type == "get_scene_info" or cmd_type == "list_scene_items":
        return get_scene_info()
    elif cmd_type == "execute_code":
        return execute_code(params.get("code", ""))
    elif cmd_type == "rig_game_item":
        return rig_game_item(params)
    elif cmd_type == "craft_hero_prop":
        return craft_hero_prop(params)
    elif cmd_type in ("look", "look_at"):
        return look_at(params)
    elif cmd_type == "render_viewport_snapshot":
        return render_viewport_snapshot(params)
    elif cmd_type == "describe_node_type":
        return describe_node_type(params.get("bl_idname", ""), params.get("property_overrides"))
    elif cmd_type == "bpy_api_lookup":
        return bpy_api_lookup(params.get("query", ""))
    elif cmd_type == "get_screenshot_of_area":
        return get_screenshot_of_area(params)
    elif cmd_type == "check_missing_files":
        return check_missing_files()
    elif cmd_type == "focus_object":
        return focus_object(params.get("name", ""), params.get("allow_edits", True))
    elif cmd_type == "focus_object_data":
        return focus_object_data(params.get("name", ""), params.get("allow_edits", True))
    elif cmd_type == "jump_to_tab_by_space_type":
        return jump_to_tab_by_space_type(params.get("space_type", "VIEW_3D"), params.get("allow_edits", False))
    elif cmd_type == "render_thumbnail_to_path":
        return render_thumbnail_to_path(params)
    elif cmd_type == "render_viewport_to_path":
        return render_viewport_to_path(params.get("output_path", "/tmp/blender_render.png"))
    else:
        return {"status": "error", "message": f"Unknown command: {cmd_type}"}

def get_addon_info():
    return {
        "status": "success",
        "name": bl_info["name"],
        "addon_version": list(bl_info["version"]),
        "protocol_version": "0.3.0",
        "blender_version": list(bpy.app.version),
        "author": bl_info["author"],
        "telemetry": "disabled_by_doctrine",
        "capabilities": [
            "execute_code",
            "get_scene_info",
            "look",
            "render_viewport_snapshot",
            "describe_node_type",
            "bpy_api_lookup",
            "get_screenshot_of_area",
            "check_missing_files",
            "focus_object",
            "focus_object_data",
            "jump_to_tab_by_space_type",
            "render_thumbnail_to_path",
            "render_viewport_to_path",
            "rig_game_item",
            "craft_hero_prop",
        ],
    }

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
        capture_buffer = io.StringIO()
        local_scope = {"bpy": bpy}
        with redirect_stdout(capture_buffer):
            exec(code, local_scope)
        return {
            "status": "success",
            "executed": True,
            "result": capture_buffer.getvalue() or "Code executed successfully",
        }
    except Exception as e:
        tb = traceback.format_exc()
        return {
            "status": "error",
            "exception_type": type(e).__name__,
            "message": str(e),
            "traceback": tb,
        }

def _describe_prop(prop):
    entry = {
        "identifier": prop.identifier,
        "name": prop.name,
        "type": prop.type,
        "description": prop.description,
    }
    for attr in ("is_required", "is_readonly", "is_argument_optional", "array_length"):
        v = getattr(prop, attr, None)
        if v is not None:
            entry[attr] = v
    if prop.type == 'ENUM':
        with suppress(Exception):
            entry["enum_items"] = [item.identifier for item in prop.enum_items]
        with suppress(Exception):
            entry["default"] = prop.default
    elif prop.type in ('FLOAT', 'INT'):
        with suppress(Exception):
            entry["default"] = list(prop.default_array) if getattr(prop, "array_length", 0) else prop.default
        for attr in ("hard_min", "hard_max", "soft_min", "soft_max", "unit"):
            v = getattr(prop, attr, None)
            if v is not None:
                entry[attr] = v
    elif prop.type == 'BOOLEAN':
        with suppress(Exception):
            entry["default"] = prop.default
    elif prop.type == 'STRING':
        with suppress(Exception):
            entry["default"] = prop.default
    return entry

def describe_node_type(bl_idname, property_overrides=None):
    node_cls = getattr(bpy.types, bl_idname, None)
    if node_cls is None or not (isinstance(node_cls, type) and issubclass(node_cls, bpy.types.Node)):
        candidates = [name for name in dir(bpy.types) if "Node" in name and bl_idname.lower() in name.lower()]
        return {"error": f"Unknown node type: {bl_idname}", "did_you_mean": sorted(candidates)[:15]}

    for tree_type in ["ShaderNodeTree", "GeometryNodeTree", "CompositorNodeTree"]:
        tmp_tree = None
        try:
            tmp_tree = bpy.data.node_groups.new(name="__mcp_tmp__", type=tree_type)
            node = tmp_tree.nodes.new(type=bl_idname)
            if property_overrides:
                for k, v in property_overrides.items():
                    with suppress(Exception):
                        setattr(node, k, v)

            base_props = set(bpy.types.Node.bl_rna.properties.keys())
            properties = [_describe_prop(p) for p in node.bl_rna.properties if p.identifier not in base_props]

            def map_sockets(sockets):
                out = []
                for idx, s in enumerate(sockets):
                    d = {
                        "index": idx,
                        "identifier": s.identifier,
                        "name": s.name,
                        "type": s.type,
                        "is_linked": s.is_linked,
                    }
                    if hasattr(s, "default_value"):
                        dv = s.default_value
                        if hasattr(dv, "__len__") and not isinstance(dv, str):
                            d["default_value"] = list(dv)
                        elif isinstance(dv, (int, float, str, bool)):
                            d["default_value"] = dv
                    out.append(d)
                return out

            res = {
                "bl_idname": bl_idname,
                "tree_type": tree_type,
                "properties": properties,
                "inputs": map_sockets(node.inputs),
                "outputs": map_sockets(node.outputs),
            }
            bpy.data.node_groups.remove(tmp_tree)
            return res
        except Exception:
            if tmp_tree:
                with suppress(Exception):
                    bpy.data.node_groups.remove(tmp_tree)
            continue

    return {"error": f"Could not instantiate node '{bl_idname}' in any node tree type"}

def bpy_api_lookup(query):
    q = (query or "").strip()
    if q.startswith("bpy."):
        q = q[4:]
    if q.startswith("ops."):
        parts = q[4:].split(".")
        if len(parts) >= 2:
            cat, op_name = parts[0], parts[1]
            op = getattr(getattr(bpy.ops, cat, None), op_name, None)
            if op:
                try:
                    rna = op.get_rna_type()
                    params = [_describe_prop(p) for p in rna.properties if p.identifier != "rna_type"]
                    return {
                        "kind": "operator",
                        "idname": f"bpy.ops.{cat}.{op_name}",
                        "label": rna.name,
                        "description": rna.description,
                        "parameters": params,
                    }
                except Exception as e:
                    return {"error": f"Could not introspect operator {q}: {e}"}
            return {"error": f"Unknown operator: bpy.ops.{cat}.{op_name}"}

    parts = [p for p in q.split(".") if p and p != "types"]
    if not parts:
        return {"error": "Empty query"}
    type_name = parts[0]
    node_cls = getattr(bpy.types, type_name, None)
    if node_cls is None:
        matches = sorted([n for n in dir(bpy.types) if type_name.lower() in n.lower()])
        return {"error": f"Unknown type: {type_name}", "did_you_mean": matches[:15]}

    if len(parts) == 1:
        properties = [_describe_prop(p) for p in node_cls.bl_rna.properties if p.identifier != "rna_type"]
        return {
            "kind": "type",
            "bl_idname": type_name,
            "description": getattr(node_cls.bl_rna, "description", ""),
            "properties": properties,
        }

    member_name = parts[1]
    prop = node_cls.bl_rna.properties.get(member_name)
    if prop:
        entry = _describe_prop(prop)
        entry["kind"] = "property"
        entry["owner_type"] = type_name
        return entry
    return {"error": f"'{type_name}' has no property named '{member_name}'"}

def get_screenshot_of_area(params):
    if getattr(bpy.app, "background", False):
        return {"error": "Screenshots are not available in background mode"}
    area_type = params.get("area_ui_type") or params.get("area_type") or "VIEW_3D"
    max_size = params.get("max_size", 1024)
    filepath = params.get("filepath") or params.get("save_to")

    window = getattr(bpy.context, "window", None)
    if not window or not window.screen:
        return {"error": "No active window/screen"}

    matching = [a for a in window.screen.areas if getattr(a, "ui_type", None) == area_type or getattr(a, "type", None) == area_type]
    if not matching:
        return {"error": f"No area matching '{area_type}' found"}
    area = max(matching, key=lambda a: a.width * a.height)

    target_path = filepath or os.path.join(tempfile.gettempdir(), f"mcp_area_{area_type}.png")
    override = getattr(bpy.context, "temp_override", None)
    if override:
        with bpy.context.temp_override(window=window, area=area):
            bpy.ops.screen.screenshot_area(filepath=target_path)
    else:
        bpy.ops.screen.screenshot_area(filepath=target_path)

    return {
        "status": "success",
        "area_type": area_type,
        "width": area.width,
        "height": area.height,
        "filepath": target_path,
    }

def check_missing_files():
    missing = []
    total_checked = 0

    def _visit(id_data, path, _placeholder):
        nonlocal total_checked
        total_checked += 1
        try:
            fp = bpy.path.abspath(path)
        except Exception:
            fp = path
        if not os.path.exists(fp):
            missing.append({
                "id_type": type(id_data).__name__,
                "id_name": getattr(id_data, "name", ""),
                "path": fp,
            })

    file_path_foreach = getattr(bpy.data, "file_path_foreach", None)
    if file_path_foreach:
        file_path_foreach(_visit, flags={"SKIP_PACKED", "SKIP_WEAK_REFERENCES", "RESOLVE_TOKEN"})
    else:
        for img in getattr(bpy.data, "images", []):
            if getattr(img, "source", None) == 'FILE' and not getattr(img, "packed_file", None):
                total_checked += 1
                fp = bpy.path.abspath(img.filepath)
                if not os.path.exists(fp):
                    missing.append({"id_type": "Image", "id_name": img.name, "path": fp})

    return {
        "status": "success",
        "missing_files": missing,
        "missing_count": len(missing),
        "total_checked": total_checked,
    }

def focus_object(name, allow_edits=True):
    obj = bpy.data.objects.get(name)
    if not obj:
        return {"error": f"Object '{name}' not found"}

    if allow_edits:
        if hasattr(obj, "hide_viewport"):
            obj.hide_viewport = False
        if hasattr(obj, "hide_get") and obj.hide_get():
            obj.hide_set(False)

    if getattr(bpy.context, "mode", "OBJECT") != "OBJECT":
        with suppress(Exception):
            bpy.ops.object.mode_set(mode="OBJECT")

    with suppress(Exception):
        bpy.ops.object.select_all(action="DESELECT")
    if hasattr(obj, "select_set"):
        obj.select_set(True)
    if getattr(bpy.context, "view_layer", None):
        bpy.context.view_layer.objects.active = obj

    for area in getattr(bpy.context.screen, "areas", []):
        if area.type == "VIEW_3D":
            for region in area.regions:
                if region.type == "WINDOW":
                    override = getattr(bpy.context, "temp_override", None)
                    if override:
                        with bpy.context.temp_override(area=area, region=region):
                            bpy.ops.view3d.view_selected()
                    else:
                        bpy.ops.view3d.view_selected()
                    break
            break

    return {
        "status": "success",
        "object": name,
        "type": obj.type,
        "location": list(obj.location),
    }

def focus_object_data(name, allow_edits=True):
    for obj in bpy.data.objects:
        if getattr(obj, "data", None) and getattr(obj.data, "name", None) == name:
            res = focus_object(obj.name, allow_edits)
            res["data_name"] = name
            return res
    return {"error": f"No object with data block named '{name}' found"}

def jump_to_tab_by_space_type(space_type, allow_edits=False):
    for ws in bpy.data.workspaces:
        for screen in ws.screens:
            for area in screen.areas:
                if getattr(area, "ui_type", "") == space_type or area.type == space_type:
                    bpy.context.window.workspace = ws
                    return {"status": "success", "workspace": ws.name, "space_type": space_type}
    return {"error": f"No workspace found with space type '{space_type}'"}

def render_thumbnail_to_path(params):
    out_path = params.get("output_path", "/tmp/thumb.png")
    max_dim = params.get("max_dimension", 320)
    orig_x = bpy.context.scene.render.resolution_x
    orig_y = bpy.context.scene.render.resolution_y
    orig_path = bpy.context.scene.render.filepath

    try:
        bpy.context.scene.render.resolution_x = max_dim
        bpy.context.scene.render.resolution_y = max_dim
        bpy.context.scene.render.filepath = os.path.abspath(out_path)
        bpy.ops.render.render(write_still=True)
        return {"status": "success", "filepath": out_path, "width": max_dim, "height": max_dim}
    except Exception as e:
        return {"error": f"Thumbnail render failed: {e}"}
    finally:
        bpy.context.scene.render.resolution_x = orig_x
        bpy.context.scene.render.resolution_y = orig_y
        bpy.context.scene.render.filepath = orig_path

def render_viewport_to_path(output_path):
    orig_path = bpy.context.scene.render.filepath
    try:
        os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
        bpy.context.scene.render.filepath = os.path.abspath(output_path)
        bpy.ops.render.render(write_still=True)
        return {"status": "success", "filepath": output_path}
    except Exception as e:
        return {"error": f"Scene render failed: {e}"}
    finally:
        bpy.context.scene.render.filepath = orig_path

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

    root_name = f"Hero_{archetype.capitalize()}"
    root = bpy.data.objects.new(root_name, None)
    bpy.context.scene.collection.objects.link(root)

    bpy.ops.mesh.primitive_cylinder_add(radius=0.4, depth=1.0, vertices=16)
    mesh_obj = bpy.context.active_object
    mesh_obj.name = f"{root_name}_Mesh"
    mesh_obj.parent = root

    grip = bpy.data.objects.new("SOCKET_grip_primary", None)
    grip.empty_display_type = 'ARROWS'
    grip.parent = root
    bpy.context.scene.collection.objects.link(grip)

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
    print("[blender-mcp-rs] Server stopped")

def register():
    start_server()

def unregister():
    stop_server()

if __name__ == "__main__":
    register()
