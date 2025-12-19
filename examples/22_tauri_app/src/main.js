// Tauri Screen Capture Example with WebGL Rendering
// Uses screencapturekit-rs via Tauri commands

const { invoke } = window.__TAURI__.core;

// State
let displays = [];
let windows = [];
let gl = null;
let glProgram = null;
let glTexture = null;

// DOM Elements
const statusEl = document.getElementById('status');
const displaysListEl = document.getElementById('displays-list');
const windowsListEl = document.getElementById('windows-list');
const previewContainerEl = document.getElementById('preview-container');
const previewInfoEl = document.getElementById('preview-info');

// WebGL shaders
const vertexShaderSource = `
  attribute vec2 a_position;
  attribute vec2 a_texCoord;
  varying vec2 v_texCoord;
  void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_texCoord = a_texCoord;
  }
`;

const fragmentShaderSource = `
  precision mediump float;
  varying vec2 v_texCoord;
  uniform sampler2D u_texture;
  void main() {
    vec4 color = texture2D(u_texture, v_texCoord);
    // BGRA to RGBA swap (blue and red channels)
    gl_FragColor = vec4(color.b, color.g, color.r, color.a);
  }
`;

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
  setupEventListeners();
  initWebGL();
  await refreshStatus();
  await refreshContent();
});

function setupEventListeners() {
  document.getElementById('btn-screenshot-display').addEventListener('click', captureDisplay);
  document.getElementById('btn-refresh').addEventListener('click', refreshContent);
}

// Initialize WebGL
function initWebGL() {
  const canvas = document.getElementById('preview-canvas');
  gl = canvas.getContext('webgl', { preserveDrawingBuffer: true });
  
  if (!gl) {
    console.error('WebGL not supported');
    return;
  }

  // Compile shaders
  const vertexShader = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vertexShader, vertexShaderSource);
  gl.compileShader(vertexShader);

  const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fragmentShader, fragmentShaderSource);
  gl.compileShader(fragmentShader);

  // Link program
  glProgram = gl.createProgram();
  gl.attachShader(glProgram, vertexShader);
  gl.attachShader(glProgram, fragmentShader);
  gl.linkProgram(glProgram);
  gl.useProgram(glProgram);

  // Set up geometry (fullscreen quad)
  const positions = new Float32Array([
    -1, -1,  0, 1,
     1, -1,  1, 1,
    -1,  1,  0, 0,
     1,  1,  1, 0,
  ]);

  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

  const positionLoc = gl.getAttribLocation(glProgram, 'a_position');
  const texCoordLoc = gl.getAttribLocation(glProgram, 'a_texCoord');

  gl.enableVertexAttribArray(positionLoc);
  gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 16, 0);
  gl.enableVertexAttribArray(texCoordLoc);
  gl.vertexAttribPointer(texCoordLoc, 2, gl.FLOAT, false, 16, 8);

  // Create texture
  glTexture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, glTexture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
}

// Render RGBA data with WebGL
function renderWithWebGL(rgbaData, width, height) {
  if (!gl) return;

  const canvas = document.getElementById('preview-canvas');
  
  // Calculate aspect ratio preserving dimensions
  const containerWidth = previewContainerEl.clientWidth - 20;
  const containerHeight = 400;
  const aspectRatio = width / height;
  
  let canvasWidth, canvasHeight;
  if (containerWidth / containerHeight > aspectRatio) {
    canvasHeight = containerHeight;
    canvasWidth = canvasHeight * aspectRatio;
  } else {
    canvasWidth = containerWidth;
    canvasHeight = canvasWidth / aspectRatio;
  }
  
  canvas.width = width;
  canvas.height = height;
  canvas.style.width = `${canvasWidth}px`;
  canvas.style.height = `${canvasHeight}px`;
  canvas.style.display = 'block';

  gl.viewport(0, 0, width, height);

  // Upload texture data
  gl.bindTexture(gl.TEXTURE_2D, glTexture);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, rgbaData);

  // Draw
  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
}

// Status
async function refreshStatus() {
  try {
    const status = await invoke('get_status');
    statusEl.textContent = status;
    statusEl.classList.remove('error');
  } catch (err) {
    statusEl.textContent = `Error: ${err}`;
    statusEl.classList.add('error');
  }
}

// Refresh content lists
async function refreshContent() {
  try {
    statusEl.textContent = 'Refreshing...';
    
    // Load displays and windows in parallel
    const [displayData, windowData] = await Promise.all([
      invoke('list_displays'),
      invoke('list_windows'),
    ]);
    
    displays = displayData;
    windows = windowData;
    
    renderDisplays();
    renderWindows();
    await refreshStatus();
  } catch (err) {
    statusEl.textContent = `Error: ${err}`;
    statusEl.classList.add('error');
    console.error('Failed to refresh content:', err);
  }
}

// Render displays
function renderDisplays() {
  if (displays.length === 0) {
    displaysListEl.innerHTML = '<p class="placeholder">No displays found</p>';
    return;
  }
  
  displaysListEl.innerHTML = displays.map((d, i) => `
    <div class="item-card" data-display-id="${d.id}">
      <div>
        <div class="item-title">Display ${i + 1}</div>
        <div class="item-subtitle">${d.width} × ${d.height} • ID: ${d.id}</div>
      </div>
      <button class="capture-btn" onclick="captureDisplayById(${d.id})">Capture</button>
    </div>
  `).join('');
}

// Render windows
function renderWindows() {
  const visibleWindows = windows.filter(w => w.title && w.title.trim() !== '');
  
  if (visibleWindows.length === 0) {
    windowsListEl.innerHTML = '<p class="placeholder">No windows found</p>';
    return;
  }
  
  windowsListEl.innerHTML = visibleWindows.slice(0, 20).map(w => `
    <div class="item-card" data-window-id="${w.id}">
      <div>
        <div class="item-title">${escapeHtml(w.title || 'Untitled')}</div>
        <div class="item-subtitle">${escapeHtml(w.app_name || 'Unknown')} • ${Math.round(w.width)} × ${Math.round(w.height)}</div>
      </div>
      <button class="capture-btn" onclick="captureWindowById(${w.id})">Capture</button>
    </div>
  `).join('');
}

// Capture primary display
async function captureDisplay() {
  try {
    statusEl.textContent = 'Capturing display...';
    const result = await invoke('take_screenshot_display', { displayId: null });
    showScreenshot(result);
    statusEl.textContent = 'Screenshot captured!';
    statusEl.classList.remove('error');
  } catch (err) {
    statusEl.textContent = `Capture failed: ${err}`;
    statusEl.classList.add('error');
    console.error('Screenshot failed:', err);
  }
}

// Capture specific display
window.captureDisplayById = async function(displayId) {
  try {
    statusEl.textContent = `Capturing display ${displayId}...`;
    const result = await invoke('take_screenshot_display', { displayId });
    showScreenshot(result);
    statusEl.textContent = 'Screenshot captured!';
    statusEl.classList.remove('error');
  } catch (err) {
    statusEl.textContent = `Capture failed: ${err}`;
    statusEl.classList.add('error');
    console.error('Screenshot failed:', err);
  }
};

// Capture specific window
window.captureWindowById = async function(windowId) {
  try {
    statusEl.textContent = `Capturing window ${windowId}...`;
    const result = await invoke('take_screenshot_window', { windowId });
    showScreenshot(result);
    statusEl.textContent = 'Screenshot captured!';
    statusEl.classList.remove('error');
  } catch (err) {
    statusEl.textContent = `Capture failed: ${err}`;
    statusEl.classList.add('error');
    console.error('Screenshot failed:', err);
  }
};

// Display screenshot result using WebGL
function showScreenshot(result) {
  // Hide placeholder
  const placeholder = previewContainerEl.querySelector('.placeholder');
  if (placeholder) {
    placeholder.style.display = 'none';
  }

  // Decode base64 RGBA data
  const binaryString = atob(result.data);
  const rgbaData = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    rgbaData[i] = binaryString.charCodeAt(i);
  }

  // Render with WebGL
  renderWithWebGL(rgbaData, result.width, result.height);
  
  previewInfoEl.textContent = `${result.width} × ${result.height} pixels (WebGL rendered)`;
}

// Utility: escape HTML
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
