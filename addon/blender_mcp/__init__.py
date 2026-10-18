bl_info = {
    "name": "blender-mcp-rs Companion",
    "author": "Zachary Shallbetter",
    "version": (0, 5, 0),
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
import mmap
import struct
import time
from contextlib import redirect_stdout, suppress

_server_sock = None
_server_thread = None
_running = False

_shm_mmap = None
_shm_file = None
_shm_seq = 0
_shm_streaming = False

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
    elif cmd_type == "get_scene_info":
        return get_scene_info()
    elif cmd_type == "list_scene_items":
        return list_scene_items()
    elif cmd_type == "get_object_resource":
        return get_object_resource(params.get("name", ""))
    elif cmd_type == "get_material_resource":
        return get_material_resource(params.get("name", ""))
    elif cmd_type == "get_collection_resource":
        return get_collection_resource(params.get("name", ""))
    elif cmd_type == "viewport_pick":
        return viewport_pick(params)
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
    elif cmd_type == "start_viewport_stream":
        return start_viewport_stream(params)
    elif cmd_type == "stop_viewport_stream":
        return stop_viewport_stream()
    elif cmd_type == "apply_scene_csg":
        return apply_scene_csg(params)
    elif cmd_type == "generate_lods_and_collision":
        return generate_lods_and_collision(params)
    elif cmd_type == "generate_armature_rig":
        return generate_armature_rig(params)
    else:
        return {"status": "error", "message": f"Unknown command: {cmd_type}"}

def get_addon_info():
    return {
        "status": "success",
        "name": bl_info["name"],
        "addon_version": list(bl_info["version"]),
        "protocol_version": "0.5.0",
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
            "start_viewport_stream",
            "stop_viewport_stream",
            "apply_scene_csg",
            "generate_lods_and_collision",
            "generate_armature_rig",
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

def list_scene_items():
    items = []
    for obj in bpy.context.scene.objects:
        items.append({
            "name": obj.name,
            "kind": "object",
            "type": obj.type,
            "uri": f"blender://objects/{obj.name}",
            "mention_uri": f"mention://objects/{obj.name}",
            "description": f"{obj.type} at ({obj.location.x:.2f}, {obj.location.y:.2f}, {obj.location.z:.2f})",
        })
    for mat in bpy.data.materials:
        items.append({
            "name": mat.name,
            "kind": "material",
            "type": "MATERIAL",
            "uri": f"blender://materials/{mat.name}",
            "mention_uri": f"mention://materials/{mat.name}",
            "description": f"Material (nodes: {mat.use_nodes}, users: {mat.users})",
        })
    for col in bpy.data.collections:
        items.append({
            "name": col.name,
            "kind": "collection",
            "type": "COLLECTION",
            "uri": f"blender://collections/{col.name}",
            "mention_uri": f"mention://collections/{col.name}",
            "description": f"Collection ({len(col.objects)} objects)",
        })
    return {
        "status": "success",
        "items": items,
        "count": len(items),
    }

def get_object_resource(name):
    obj = bpy.data.objects.get(name)
    if not obj:
        return {"status": "error", "message": f"Object not found: {name}"}
    return {
        "status": "success",
        "name": obj.name,
        "type": obj.type,
        "location": [round(v, 4) for v in obj.location],
        "rotation": [round(v, 4) for v in obj.rotation_euler],
        "scale": [round(v, 4) for v in obj.scale],
        "dimensions": [round(v, 4) for v in obj.dimensions],
        "visible": obj.visible_get(),
        "parent": obj.parent.name if obj.parent else None,
        "children": [c.name for c in obj.children],
        "modifiers": [{"name": m.name, "type": m.type} for m in obj.modifiers],
        "material_slots": [slot.material.name for slot in obj.material_slots if slot.material],
    }

def get_material_resource(name):
    mat = bpy.data.materials.get(name)
    if not mat:
        return {"status": "error", "message": f"Material not found: {name}"}
    nodes = []
    links = []
    if mat.use_nodes and mat.node_tree:
        for n in mat.node_tree.nodes:
            nodes.append({
                "name": n.name,
                "bl_idname": n.bl_idname,
                "label": n.label or n.name,
                "location": [round(v, 1) for v in n.location],
            })
        for l in mat.node_tree.links:
            links.append({
                "from_node": l.from_node.name,
                "from_socket": l.from_socket.name,
                "to_node": l.to_node.name,
                "to_socket": l.to_socket.name,
            })
    return {
        "status": "success",
        "name": mat.name,
        "use_nodes": mat.use_nodes,
        "users": mat.users,
        "nodes": nodes,
        "links": links,
    }

def get_collection_resource(name):
    col = bpy.data.collections.get(name)
    if not col:
        return {"status": "error", "message": f"Collection not found: {name}"}
    return {
        "status": "success",
        "name": col.name,
        "objects": [o.name for o in col.objects],
        "children": [c.name for c in col.children],
    }

def viewport_pick(params):
    x = float(params.get("x", 0.5))
    y = float(params.get("y", 0.5))
    seq = params.get("seq", 0)

    depsgraph = bpy.context.evaluated_depsgraph_get()
    scene = bpy.context.scene

    cam = scene.camera
    if cam:
        cam_eval = cam.evaluated_get(depsgraph)
        origin = cam_eval.matrix_world.translation
        view_mat = cam_eval.matrix_world
        direction = (view_mat.to_3x3() @ mathutils.Vector((x - 0.5, y - 0.5, -1.0))).normalized()
    else:
        origin = mathutils.Vector(((x - 0.5) * 10.0, (y - 0.5) * 10.0, 10.0))
        direction = mathutils.Vector((0.0, 0.0, -1.0))

    hit, location, normal, index, hit_obj, matrix = scene.ray_cast(depsgraph, origin, direction)

    if hit and hit_obj:
        for o in bpy.context.selected_objects:
            o.select_set(False)
        hit_obj.select_set(True)
        bpy.context.view_layer.objects.active = hit_obj

        mat_name = hit_obj.active_material.name if hit_obj.active_material else None
        return {
            "status": "success",
            "hit": True,
            "object_name": hit_obj.name,
            "location": [round(v, 4) for v in location],
            "normal": [round(v, 4) for v in normal],
            "distance": round((location - origin).length, 4),
            "face_index": index,
            "material": mat_name,
            "seq": seq,
        }
    else:
        return {
            "status": "success",
            "hit": False,
            "location": None,
            "seq": seq,
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

def start_viewport_stream(params):
    global _shm_mmap, _shm_file, _shm_seq, _shm_streaming
    shm_path = params.get("shm_path") or "/tmp/blender_mcp_shm_viewport"
    fps = max(1, min(params.get("fps", 30), 60))
    capacity = params.get("capacity", 16 * 1024 * 1024)

    if _shm_streaming:
        stop_viewport_stream()

    os.makedirs(os.path.dirname(os.path.abspath(shm_path)), exist_ok=True)
    _shm_file = open(shm_path, "a+b")
    _shm_file.seek(0, os.SEEK_END)
    if _shm_file.tell() < capacity:
        _shm_file.truncate(capacity)
    _shm_file.seek(0)
    _shm_mmap = mmap.mmap(_shm_file.fileno(), capacity)
    _shm_seq = 0
    _shm_streaming = True

    def stream_tick():
        global _shm_seq, _shm_mmap, _shm_streaming
        if not _shm_streaming or _shm_mmap is None:
            return None
        try:
            w = bpy.context.scene.render.resolution_x or 640
            h = bpy.context.scene.render.resolution_y or 480
            _shm_seq += 1
            ts = time.time_ns()
            data_size = w * h * 4
            if 48 + data_size <= len(_shm_mmap):
                header = struct.pack("<I4xQIII4xQQ", 0x4652414D, _shm_seq, w, h, 4, ts, data_size)
                _shm_mmap[0:48] = header
                _shm_mmap.flush()
        except Exception as e:
            print(f"[blender-mcp-rs] Stream tick error: {e}")
        return 1.0 / fps if _shm_streaming else None

    bpy.app.timers.register(stream_tick, first_interval=1.0 / fps)
    return {
        "status": "success",
        "streaming": True,
        "shm_path": shm_path,
        "fps": fps,
        "capacity": capacity,
    }

def stop_viewport_stream():
    global _shm_mmap, _shm_file, _shm_streaming
    _shm_streaming = False
    if _shm_mmap:
        try:
            _shm_mmap.close()
        except Exception:
            pass
        _shm_mmap = None
    if _shm_file:
        try:
            _shm_file.close()
        except Exception:
            pass
        _shm_file = None
    return {"status": "success", "streaming": False}

def apply_scene_csg(params):
    op = params.get("operation", "union").lower()
    target_name = params.get("target_object")
    operand_name = params.get("operand_object")
    socket_name = params.get("socket_name")
    solver = params.get("solver", "FAST").upper()

    target_obj = bpy.data.objects.get(target_name)
    if not target_obj:
        return {"status": "error", "message": f"Target object '{target_name}' not found"}

    if op == "snap_to_socket":
        operand_obj = bpy.data.objects.get(operand_name)
        if not operand_obj:
            return {"status": "error", "message": f"Operand object '{operand_name}' not found"}
        socket_obj = None
        for child in target_obj.children:
            if child.name == socket_name or (socket_name is None and child.name.startswith("SOCKET_")):
                socket_obj = child
                break
        if socket_obj:
            operand_obj.matrix_world = socket_obj.matrix_world.copy()
            return {
                "status": "success",
                "operation": "snap_to_socket",
                "target": target_name,
                "operand": operand_name,
                "socket": socket_obj.name,
                "location": list(operand_obj.location),
            }
        else:
            return {"status": "error", "message": f"Socket '{socket_name}' not found on '{target_name}'"}

    operand_obj = bpy.data.objects.get(operand_name)
    if not operand_obj:
        return {"status": "error", "message": f"Operand object '{operand_name}' not found"}

    op_map = {
        "union": "UNION",
        "difference": "DIFFERENCE",
        "intersect": "INTERSECT",
    }
    bool_op = op_map.get(op)
    if not bool_op:
        return {"status": "error", "message": f"Unsupported CSG operation: {op}. Expected union, difference, intersect, or snap_to_socket"}

    mod = target_obj.modifiers.new(name="CSG_Op", type='BOOLEAN')
    mod.operation = bool_op
    mod.object = operand_obj
    mod.solver = solver if solver in ("FAST", "EXACT") else "FAST"

    bpy.context.view_layer.objects.active = target_obj
    target_obj.select_set(True)
    try:
        bpy.ops.object.modifier_apply(modifier=mod.name)
    except Exception:
        pass

    poly_count = len(target_obj.data.polygons) if target_obj.type == 'MESH' else 0
    vert_count = len(target_obj.data.vertices) if target_obj.type == 'MESH' else 0

    return {
        "status": "success",
        "operation": op,
        "target": target_name,
        "operand": operand_name,
        "solver": solver,
        "polygons": poly_count,
        "vertices": vert_count,
    }

def generate_lods_and_collision(params):
    obj_name = params.get("object_name")
    lod_ratios = params.get("lod_ratios") or [1.0, 0.5, 0.25, 0.1]
    collision_type = (params.get("collision_type") or "convex_hull").lower()

    target_obj = bpy.data.objects.get(obj_name)
    if not target_obj or target_obj.type != 'MESH':
        return {"status": "error", "message": f"Mesh object '{obj_name}' not found"}

    lod_objects = []
    for idx, ratio in enumerate(lod_ratios):
        lod_name = f"{obj_name}_LOD{idx}"
        lod_mesh = target_obj.data.copy()
        lod_obj = bpy.data.objects.new(lod_name, lod_mesh)
        bpy.context.scene.collection.objects.link(lod_obj)
        lod_obj.matrix_world = target_obj.matrix_world.copy()

        if ratio < 0.999:
            dec = lod_obj.modifiers.new(name="Decimate", type='DECIMATE')
            dec.ratio = float(ratio)
            bpy.context.view_layer.objects.active = lod_obj
            try:
                bpy.ops.object.modifier_apply(modifier=dec.name)
            except Exception:
                pass
        lod_objects.append({
            "lod": idx,
            "name": lod_name,
            "ratio": ratio,
            "polygons": len(lod_obj.data.polygons),
        })

    collision_hull_name = f"UCX_{obj_name}"
    coll_obj = None
    if collision_type == "box":
        bpy.ops.mesh.primitive_cube_add()
        coll_obj = bpy.context.active_object
        coll_obj.name = collision_hull_name
        coll_obj.matrix_world = target_obj.matrix_world.copy()
        coll_obj.dimensions = target_obj.dimensions
    elif collision_type == "convex_hull":
        coll_mesh = target_obj.data.copy()
        coll_obj = bpy.data.objects.new(collision_hull_name, coll_mesh)
        bpy.context.scene.collection.objects.link(coll_obj)
        coll_obj.matrix_world = target_obj.matrix_world.copy()
        remesh = coll_obj.modifiers.new(name="ConvexDecimate", type='DECIMATE')
        remesh.ratio = 0.05
        bpy.context.view_layer.objects.active = coll_obj
        try:
            bpy.ops.object.modifier_apply(modifier=remesh.name)
        except Exception:
            pass
    if coll_obj:
        coll_obj.display_type = 'WIRE'

    return {
        "status": "success",
        "base_object": obj_name,
        "lods": lod_objects,
        "collision": collision_hull_name if coll_obj else None,
        "collision_type": collision_type,
    }

def generate_armature_rig(params):
    target_name = params.get("target_object")
    rig_template = (params.get("rig_template") or "prop_container").lower()
    bind_mesh = params.get("bind_mesh", True)

    target_obj = bpy.data.objects.get(target_name)
    if not target_obj:
        return {"status": "error", "message": f"Target object '{target_name}' not found"}

    arm_data = bpy.data.armatures.new(f"{target_name}_Armature")
    arm_obj = bpy.data.objects.new(f"{target_name}_Rig", arm_data)
    bpy.context.scene.collection.objects.link(arm_obj)
    arm_obj.matrix_world = target_obj.matrix_world.copy()

    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode='EDIT')

    bones = []
    if rig_template == "prop_container":
        root = arm_data.edit_bones.new("root")
        root.head = (0, 0, 0)
        root.tail = (0, 0, 0.5)
        lid = arm_data.edit_bones.new("lid")
        lid.parent = root
        lid.head = (0, 0, 0.5)
        lid.tail = (0, 0, 1.0)
        bones = ["root", "lid"]
    elif rig_template == "mechanical_door":
        root = arm_data.edit_bones.new("root")
        root.head = (0, 0, 0)
        root.tail = (0, 0, 0.5)
        hinge = arm_data.edit_bones.new("hinge")
        hinge.parent = root
        hinge.head = (-0.5, 0, 0)
        hinge.tail = (-0.5, 0, 1.0)
        bones = ["root", "hinge"]
    else:
        root = arm_data.edit_bones.new("root")
        root.head = (0, 0, 0)
        root.tail = (0, 0, 0.5)
        bones = ["root"]

    bpy.ops.object.mode_set(mode='OBJECT')

    if bind_mesh and target_obj.type == 'MESH':
        target_obj.parent = arm_obj
        mod = target_obj.modifiers.new(name="Armature", type='ARMATURE')
        mod.object = arm_obj
        for b in bones:
            if b not in target_obj.vertex_groups:
                vg = target_obj.vertex_groups.new(name=b)
                vg.add(list(range(len(target_obj.data.vertices))), 1.0, 'REPLACE')

    return {
        "status": "success",
        "armature": arm_obj.name,
        "rig_template": rig_template,
        "bones": bones,
        "bound": bind_mesh,
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
