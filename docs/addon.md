# Companion Blender Addon (`blender-mcp-rs Companion`)

The companion addon (`addon/blender_mcp/__init__.py`) runs as a lightweight daemon inside Blender 4.0+, listening on `127.0.0.1:9876` for high-speed IPC requests.

---

## Installation

### Method 1: Automatic CLI Installation (Recommended)
```bash
blender-mcp addon install
```

This copies the add-on into the user add-ons folder of each Blender version on this machine (including a Blender that has never been opened) and turns it on in that Blender's saved preferences, by running Blender once without a window. From then on the add-on loads every time Blender opens and listens on `127.0.0.1:9876`; there is nothing to enable by hand.

- If Blender is open, the command doesn't change its preferences (Blender saves its own preferences on quit and would undo the change). Quit Blender and run it again.
- `--no-enable` only copies the file; turn it on yourself under **Edit > Preferences > Add-ons**.
- `--blender-version 5.2` limits it to one version.

### Method 2: Manual Blender UI Installation
1. Open Blender.
2. Navigate to **Edit > Preferences > Add-ons**.
3. Click **Install...** (or **Install from Disk...** in Blender 4.2+).
4. Select `addon/blender_mcp/__init__.py` or the compressed `blender-mcp-companion.zip`.
5. Check the box to enable **Development: blender-mcp-rs Companion**.

### Method 3: Headless Execution / CI Pipelines
When launching Blender in background/headless mode:
```bash
blender -b --python addon/blender_mcp/__init__.py
```

---

## Verification
Verify the daemon is responsive from your shell:
```bash
# Check status using the CLI
blender-mcp addon status
```

Or query directly using Netcat / Telnet:
```bash
echo '{"type":"get_addon_info"}' | nc 127.0.0.1 9876
```

Expected response:
```json
{
  "status": "success",
  "name": "blender-mcp-rs Companion",
  "addon_version": [0, 5, 0],
  "protocol_version": "0.5.0",
  "blender_version": [4, 0, 0],
  "author": "Zachary Shallbetter",
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
    "generate_armature_rig"
  ]
}
```
