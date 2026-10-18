/// Interactive Web Viewport HTML Template for `blender://viewport` MCP Resource.
///
/// Provides an interactive HTML5/WebGL canvas viewer with mouse-driven camera orbit,
/// pan, zoom, status indicators, and tool dispatch triggers.
pub fn generate_viewport_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>blender-mcp-rs Live Viewport</title>
  <style>
    :root {
      --bg-color: #18181b;
      --panel-bg: rgba(24, 24, 27, 0.85);
      --border-color: #27272a;
      --text-main: #f4f4f5;
      --text-muted: #a1a1aa;
      --accent: #3b82f6;
      --accent-hover: #2563eb;
      --success: #10b981;
    }
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
      user-select: none;
    }
    body, html {
      width: 100%;
      height: 100%;
      overflow: hidden;
      background-color: var(--bg-color);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      color: var(--text-main);
    }
    #viewport-container {
      position: relative;
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
    }
    #header-bar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 16px;
      background: var(--panel-bg);
      border-bottom: 1px solid var(--border-color);
      backdrop-filter: blur(8px);
      z-index: 10;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 8px;
      font-weight: 600;
      font-size: 14px;
      letter-spacing: 0.5px;
    }
    .status-badge {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-size: 12px;
      padding: 2px 8px;
      border-radius: 9999px;
      background: rgba(16, 185, 129, 0.15);
      color: var(--success);
      border: 1px solid rgba(16, 185, 129, 0.3);
    }
    .status-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: var(--success);
      box-shadow: 0 0 6px var(--success);
    }
    #canvas-wrap {
      flex: 1;
      position: relative;
      background: radial-gradient(circle at center, #27272a 0%, #09090b 100%);
      cursor: grab;
      overflow: hidden;
    }
    #canvas-wrap:active {
      cursor: grabbing;
    }
    #viewport-canvas {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      object-fit: contain;
    }
    #toolbar {
      position: absolute;
      bottom: 20px;
      left: 50%;
      transform: translateX(-50%);
      display: flex;
      gap: 8px;
      padding: 6px 10px;
      background: var(--panel-bg);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      backdrop-filter: blur(12px);
      z-index: 10;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }
    .btn {
      background: #27272a;
      color: var(--text-main);
      border: 1px solid #3f3f46;
      border-radius: 6px;
      padding: 6px 12px;
      font-size: 12px;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s ease;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .btn:hover {
      background: var(--accent);
      border-color: var(--accent-hover);
      color: #fff;
    }
    #info-overlay {
      position: absolute;
      top: 60px;
      left: 16px;
      padding: 10px 14px;
      background: var(--panel-bg);
      border: 1px solid var(--border-color);
      border-radius: 8px;
      font-size: 11px;
      line-height: 1.6;
      color: var(--text-muted);
      backdrop-filter: blur(8px);
      pointer-events: none;
      z-index: 5;
    }
    #info-overlay span {
      color: var(--text-main);
      font-weight: 600;
    }
  </style>
</head>
<body>
  <div id="viewport-container">
    <div id="header-bar">
      <div class="brand">
        <span>⚡ blender-mcp-rs</span>
        <span style="color: var(--text-muted); font-weight: 400;">| Live 3D Viewport</span>
      </div>
      <div class="status-badge">
        <span class="status-dot"></span>
        <span id="conn-status">SHM Ready</span>
      </div>
    </div>

    <div id="canvas-wrap">
      <canvas id="viewport-canvas"></canvas>
      <div id="info-overlay">
        <div>Target: <span id="val-target">active</span></div>
        <div>Azimuth: <span id="val-azimuth">45°</span></div>
        <div>Elevation: <span id="val-elevation">30°</span></div>
        <div>Distance: <span id="val-distance">3.0m</span></div>
        <div>FPS: <span id="val-fps">60</span></div>
      </div>
    </div>

    <div id="toolbar">
      <button class="btn" onclick="setPreset('hero')">Hero 3/4</button>
      <button class="btn" onclick="setPreset('front')">Front</button>
      <button class="btn" onclick="setPreset('top')">Top</button>
      <button class="btn" onclick="setPreset('side')">Side</button>
      <button class="btn" onclick="triggerSnapshot()">📷 Snapshot</button>
    </div>
  </div>

  <script>
    let azimuth = 45;
    let elevation = 30;
    let distance = 3.0;
    let isDragging = false;
    let lastX = 0;
    let lastY = 0;

    const wrap = document.getElementById('canvas-wrap');
    const canvas = document.getElementById('viewport-canvas');
    const ctx = canvas.getContext('2d');

    function resize() {
      canvas.width = wrap.clientWidth;
      canvas.height = wrap.clientHeight;
      draw();
    }
    window.addEventListener('resize', resize);
    resize();

    wrap.addEventListener('mousedown', (e) => {
      isDragging = true;
      lastX = e.clientX;
      lastY = e.clientY;
    });

    window.addEventListener('mousemove', (e) => {
      if (!isDragging) return;
      const dx = e.clientX - lastX;
      const dy = e.clientY - lastY;
      lastX = e.clientX;
      lastY = e.clientY;

      azimuth = (azimuth - dx * 0.5 + 360) % 360;
      elevation = Math.max(-89, Math.min(89, elevation + dy * 0.5));
      updateDisplay();
      draw();
    });

    window.addEventListener('mouseup', () => {
      if (isDragging) {
        isDragging = false;
        notifyCameraChange();
      }
    });

    wrap.addEventListener('wheel', (e) => {
      e.preventDefault();
      distance = Math.max(0.5, Math.min(50.0, distance + e.deltaY * 0.005));
      updateDisplay();
      draw();
      notifyCameraChange();
    }, { passive: false });

    function updateDisplay() {
      document.getElementById('val-azimuth').textContent = Math.round(azimuth) + '°';
      document.getElementById('val-elevation').textContent = Math.round(elevation) + '°';
      document.getElementById('val-distance').textContent = distance.toFixed(1) + 'm';
    }

    function setPreset(name) {
      if (name === 'hero') { azimuth = 45; elevation = 30; }
      else if (name === 'front') { azimuth = 0; elevation = 0; }
      else if (name === 'top') { azimuth = 0; elevation = 89; }
      else if (name === 'side') { azimuth = 90; elevation = 0; }
      updateDisplay();
      draw();
      notifyCameraChange();
    }

    function notifyCameraChange() {
      // Dispatch camera update via MCP tool if host supports postMessage
      if (window.parent && window.parent.postMessage) {
        window.parent.postMessage({
          type: 'mcp_call_tool',
          name: 'look',
          arguments: { azimuth, elevation, distance }
        }, '*');
      }
    }

    function triggerSnapshot() {
      if (window.parent && window.parent.postMessage) {
        window.parent.postMessage({
          type: 'mcp_call_tool',
          name: 'render_viewport_snapshot',
          arguments: { width: 1080, height: 1080 }
        }, '*');
      }
    }

    function draw() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const cx = canvas.width / 2;
      const cy = canvas.height / 2;

      // Draw grid floor representation
      ctx.save();
      ctx.translate(cx, cy + 60);
      ctx.strokeStyle = 'rgba(63, 63, 70, 0.4)';
      ctx.lineWidth = 1;

      const gridSize = 160;
      const step = 20;
      for (let x = -gridSize; x <= gridSize; x += step) {
        ctx.beginPath();
        ctx.moveTo(x, -gridSize * 0.4);
        ctx.lineTo(x, gridSize * 0.4);
        ctx.stroke();
      }
      for (let y = -gridSize * 0.4; y <= gridSize * 0.4; y += step * 0.4) {
        ctx.beginPath();
        ctx.moveTo(-gridSize, y);
        ctx.lineTo(gridSize, y);
        ctx.stroke();
      }

      // Draw coordinate gizmo
      ctx.restore();
      const gx = canvas.width - 50;
      const gy = 50;
      ctx.save();
      ctx.translate(gx, gy);
      // X-axis (Red)
      ctx.strokeStyle = '#ef4444';
      ctx.lineWidth = 2;
      ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(30, 0); ctx.stroke();
      // Y-axis (Green)
      ctx.strokeStyle = '#22c55e';
      ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(0, -30); ctx.stroke();
      // Z-axis (Blue)
      ctx.strokeStyle = '#3b82f6';
      ctx.beginPath(); ctx.moveTo(0, 0); ctx.lineTo(-15, 15); ctx.stroke();
      ctx.restore();
    }
  </script>
</body>
</html>
"#
}
