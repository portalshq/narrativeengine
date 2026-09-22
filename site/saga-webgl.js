// =============================================================================
// SAGA.XYZ STANDALONE WEBGL ENGINE
// Self-contained ES module — no React, no frameworks, just Three.js r184
// =============================================================================
//
// HTML requirements:
//   1. Add an import map BEFORE this script tag:
//      <script type="importmap">
//      {"imports":{
//        "three":"https://unpkg.com/three@0.184.0/build/three.module.js",
//        "three/addons/":"https://unpkg.com/three@0.184.0/examples/jsm/"
//      }}
//      </script>
//   2. Load this script as a module:
//      <script type="module" src="saga-webgl.js"></script>
//
// The page must contain:
//   <div class="fixed inset-0 z-(--z-webgl)"><canvas class="size-full"></canvas></div>
//   Elements with data-webgl-marker="scrollFrom|scrollTo|colorRamps" attributes
//   data-webgl-position="0.0" on scrollFrom/scrollTo markers to set animation position
// =============================================================================


// =============================================================================
// SECTION 1: DYNAMIC IMPORT MAP INJECTION
// =============================================================================
{
  if (!document.querySelector('script[type="importmap"]')) {
    const s = document.createElement('script');
    s.type = 'importmap';
    s.textContent = JSON.stringify({
      imports: {
        'three': 'https://unpkg.com/three@0.184.0/build/three.module.js',
        'three/addons/': 'https://unpkg.com/three@0.184.0/examples/jsm/'
      }
    });
    document.head.prepend(s);
  }
}


// =============================================================================
// SECTION 2: MODULE IMPORTS
// =============================================================================

// Publish the page-scoped theme bridge before the CDN imports begin. This
// keeps the PX theme contract available even when a browser blocks external
// module fetches; once Three.js is available, the engine below attaches to the
// same bridge and drives the actual canvas animation.
if (typeof window !== "undefined" && !window.__sagaPxTheme) {
  window.__sagaPxTheme = {
    setActive(on) {
      window.__sagaPendingPxTheme = !!on;
      if (sagaEngineInstance) sagaEngineInstance.setPxThemeActive(!!on);
    },
  };
}

const THREE = await import('three');
const { GLTFLoader }   = await import('three/addons/loaders/GLTFLoader.js');
const { DRACOLoader }   = await import('three/addons/loaders/DRACOLoader.js');
const { EffectComposer } = await import('three/addons/postprocessing/EffectComposer.js');
const { RenderPass }    = await import('three/addons/postprocessing/RenderPass.js');
const { ShaderPass }    = await import('three/addons/postprocessing/ShaderPass.js');
const { UnrealBloomPass } = await import('three/addons/postprocessing/UnrealBloomPass.js');


// =============================================================================
// SECTION 3: COLOR RAMP & BACKGROUND CONFIGURATIONS
// =============================================================================

let DEFAULT_BG_COLOR1 = "#0E115F";
let DEFAULT_BG_COLOR2 = "#726DD2";

let DEFAULT_RAMP1 = [
  { stop: 0, color: "#0E115F" },
  { stop: 0.148, color: "#0E115F" },
  { stop: 0.381, color: "#0E115F" },
  { stop: 0.673, color: "#726DD2" },
  { stop: 0.891, color: "#726DD2" },
  { stop: 0.992, color: "#DD30C9" },
];

let DEFAULT_RAMP2 = [
  { stop: 0, color: "#053A68" },
  { stop: 0.3, color: "#3A87CB" },
  { stop: 0.6, color: "#bbc4cf" },
  { stop: 0.8, color: "#4470cc" },
  { stop: 1, color: "#6162cd" },
];

// Snapshots of the global defaults. The PX page theme temporarily reassigns the
// DEFAULT_* bindings above and restores these on unmount, so other pages are
// never affected.
const ORIGINAL_BG_COLOR1 = DEFAULT_BG_COLOR1;
const ORIGINAL_BG_COLOR2 = DEFAULT_BG_COLOR2;
const ORIGINAL_RAMP1 = DEFAULT_RAMP1;
const ORIGINAL_RAMP2 = DEFAULT_RAMP2;
const ORIGINAL_MESH_COLOR1 = "#B6F2FF";
const ORIGINAL_MESH_COLOR2 = "#274fff";
const ORIGINAL_MESH_MIDDLE = "#B6F2FF";

// PX-exclusive palette. Main: black. Secondary: #efdc3d. Third: the same
// yellow at 42% opacity — written here as rgba(239,220,61,0.42) composited
// over the black field (#645c1a) wherever the shader needs an opaque stop.
const PX_BG_COLOR1 = "#000000";
const PX_BG_COLOR2 = "#efdc3d";
const PX_RAMP1 = [
  { stop: 0.008, color: "#efdc3d" },
  { stop: 0.109, color: "#645c1a" },
  { stop: 0.327, color: "#efdc3d" },
  { stop: 0.619, color: "#000000" },
  { stop: 0.852, color: "#efdc3d" },
  { stop: 1, color: "#000000" },
];
const PX_RAMP2 = [
  { stop: 0, color: "#000000" },
  { stop: 0.2, color: "#645c1a" },
  { stop: 0.4, color: "#efdc3d" },
  { stop: 0.7, color: "#645c1a" },
  { stop: 1, color: "#000000" },
];
const PX_MESH_COLOR1 = "#efdc3d";
const PX_MESH_COLOR2 = "#000000";
const PX_MESH_MIDDLE = "#efdc3d";

const SECTION_RAMPS = [
  {
    ramp1: [
      { stop: 0, color: "#0E115F" },
      { stop: 0.148, color: "#ffffff" },
      { stop: 0.381, color: "#0E115F" },
      { stop: 0.673, color: "#726DD2" },
      { stop: 0.891, color: "#726DD2" },
      { stop: 0.992, color: "#DD30C9" },
    ],
    ramp2: [
      { stop: 0, color: "#053A68" },
      { stop: 0.3, color: "#3A87CB" },
      { stop: 0.6, color: "#bbc4cf" },
      { stop: 0.8, color: "#4470cc" },
      { stop: 1, color: "#6162cd" },
    ],
    bg1: "#0E115F", bg2: "#DD30C9",
  },
  {
    ramp1: [
      { stop: 0, color: "#0E115F" },
      { stop: 0.148, color: "#ffffff" },
      { stop: 0.381, color: "#0E115F" },
      { stop: 0.673, color: "#726DD2" },
      { stop: 0.891, color: "#726DD2" },
      { stop: 0.992, color: "#DD30C9" },
    ],
    ramp2: [
      { stop: 0.5, color: "#c6a07c" },
      { stop: 0.6, color: "#c243a7" },
      { stop: 0.65, color: "#c1afbd" },
      { stop: 0.75, color: "#c1afbd" },
      { stop: 0.8, color: "#bc39ae" },
      { stop: 0.95, color: "#a12394" },
    ],
    bg1: "#c6a07c", bg2: "#bc39ae",
  },
  {
    ramp1: [
      { stop: 0, color: "#0E115F" },
      { stop: 0.148, color: "#ffffff" },
      { stop: 0.381, color: "#0E115F" },
      { stop: 0.673, color: "#726DD2" },
      { stop: 0.891, color: "#726DD2" },
      { stop: 0.992, color: "#DD30C9" },
    ],
    ramp2: [
      { stop: 0, color: "#053A68" },
      { stop: 0.3, color: "#3A87CB" },
      { stop: 0.6, color: "#bbc4cf" },
      { stop: 0.8, color: "#4470cc" },
      { stop: 1, color: "#6162cd" },
    ],
    bg1: "#053A68", bg2: "#6162cd",
  },
  {
    ramp1: [
      { stop: 0, color: "#0E115F" },
      { stop: 0.148, color: "#ffffff" },
      { stop: 0.381, color: "#0E115F" },
      { stop: 0.673, color: "#726DD2" },
      { stop: 0.891, color: "#726DD2" },
      { stop: 0.992, color: "#DD30C9" },
    ],
    ramp2: [
      { stop: 0, color: "#5889c9" },
      { stop: 0.148, color: "#8bbfaf" },
      { stop: 0.381, color: "#2d6657" },
      { stop: 0.673, color: "#8bbfaf" },
      { stop: 0.891, color: "#5889c9" },
      { stop: 0.992, color: "#5889c9" },
    ],
    bg1: "#5889c9", bg2: "#8bbfaf",
  },
];

const GRADE_CURVE_X = 0.8;
const GRADE_CURVE_Y = 0.6;


// =============================================================================
// SECTION 4: UTILITY FUNCTIONS
// =============================================================================

function easeValue(t, easing) {
  t = Math.max(0, Math.min(1, t));
  switch (easing) {
    case "linear": default: return t;
    case "easeIn": return t * t;
    case "easeOut": return 1 - (1 - t) * (1 - t);
    case "easeInOut": return t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2;
    case "easeInCubic": return t * t * t;
    case "easeOutCubic": return 1 - Math.pow(1 - t, 3);
    case "easeInOutCubic":
      return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }
}

function smoothstep(edge0, edge1, x) {
  const t = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

function getElementScrollTop(el) {
  return el.getBoundingClientRect().top + window.scrollY;
}

function isVisibleWebGLMarker(el) {
  if (!el.isConnected) return false;
  const style = window.getComputedStyle(el);
  if (style.display === "none" || style.visibility === "hidden") return false;
  return el.getClientRects().length > 0;
}

function getVisibleWebGLMarkers() {
  return [...document.querySelectorAll("[data-webgl-marker]")].filter(isVisibleWebGLMarker);
}


// =============================================================================
// SECTION 5: COLOR RAMP SYSTEM
// =============================================================================

function parseColorStops(stops) {
  return stops.map(s => ({ stop: s.stop, color: new THREE.Color(s.color) }));
}

function interpolateColor(stops, t) {
  t = Math.max(0, Math.min(1, t));
  if (t <= stops[0].stop) return stops[0].color.clone();
  const last = stops[stops.length - 1];
  if (t >= last.stop) return last.color.clone();
  for (let i = 0; i < stops.length - 1; i++) {
    const a = stops[i], b = stops[i + 1];
    if (t >= a.stop && t <= b.stop) {
      const f = (t - a.stop) / (b.stop - a.stop);
      return a.color.clone().lerp(b.color, f);
    }
  }
  return last.color.clone();
}

function generateGradientTexture(stops, width = 1024) {
  const data = new Uint8Array(4 * width);
  const sorted = [...stops].sort((a, b) => a.stop - b.stop);
  for (let i = 0; i < width; i++) {
    const t = width > 1 ? i / (width - 1) : 0;
    const c = interpolateColor(sorted, t);
    const idx = 4 * i;
    data[idx]     = Math.round(255 * c.r);
    data[idx + 1] = Math.round(255 * c.g);
    data[idx + 2] = Math.round(255 * c.b);
    data[idx + 3] = 255;
  }
  const tex = new THREE.DataTexture(data, 1, width, THREE.RGBAFormat, THREE.UnsignedByteType);
  tex.minFilter = THREE.LinearFilter;
  tex.magFilter = THREE.LinearFilter;
  tex.wrapS = THREE.ClampToEdgeWrapping;
  tex.wrapT = THREE.ClampToEdgeWrapping;
  tex.needsUpdate = true;
  return tex;
}

function createColorRamp(stops) {
  const parsed = parseColorStops(stops);
  const texture = generateGradientTexture(parsed);
  return { stops: parsed, texture, transition: null };
}

function blendTextures(fromTex, toTex, mix) {
  const h = fromTex.image.height;
  const w = fromTex.image.width;
  const a = fromTex.image.data;
  const b = toTex.image.data;
  const out = new Uint8Array(a.length);
  const t = smoothstep(0, 1, mix);
  for (let i = 0; i < a.length; i += 4) {
    out[i]     = Math.round(a[i]     + (b[i]     - a[i])     * t);
    out[i + 1] = Math.round(a[i + 1] + (b[i + 1] - a[i + 1]) * t);
    out[i + 2] = Math.round(a[i + 2] + (b[i + 2] - a[i + 2]) * t);
    out[i + 3] = 255;
  }
  const tex = new THREE.DataTexture(out, w, h, THREE.RGBAFormat, THREE.UnsignedByteType);
  tex.minFilter = THREE.LinearFilter;
  tex.magFilter = THREE.LinearFilter;
  tex.wrapS = THREE.ClampToEdgeWrapping;
  tex.wrapT = THREE.ClampToEdgeWrapping;
  tex.needsUpdate = true;
  return tex;
}


// =============================================================================
// SECTION 6: TIMER CLASS
// =============================================================================

class Timer {
  constructor() {
    this._prev = 0;
    this._curr = 0;
    this._start = performance.now();
    this._delta = 0;
    this._elapsed = 0;
    this._timescale = 0.95;
    this._onVis = this._onVis.bind(this);
  }

  connect() {
    document.addEventListener("visibilitychange", this._onVis, false);
  }

  disconnect() {
    document.removeEventListener("visibilitychange", this._onVis);
  }

  _onVis() {
    if (!document.hidden) this._curr = performance.now() - this._start;
  }

  getDelta()   { return this._delta / 1000; }
  getElapsed() { return this._elapsed / 1000; }

  update(now) {
    if (document.hidden) { this._delta = 0; return; }
    this._prev = this._curr;
    this._curr = (now ?? performance.now()) - this._start;
    this._delta = (this._curr - this._prev) * this._timescale;
    this._elapsed += this._delta;
  }

  dispose() { this.disconnect(); }
}


// =============================================================================
// SECTION 7: ANIMATION RIG CLASS
// =============================================================================

class AnimationRig {
  constructor(root, clips = []) {
    this.root = root;
    this.mixer = new THREE.AnimationMixer(root);
    this.clips = clips;
    this.actions = new Map();
    this.clipsByName = new Map();
    this.morphMeshes = new Map();

    this._collectMorphMeshes(root);
    for (const clip of clips) {
      this.clipsByName.set(clip.name, clip);
      this.actions.set(clip.name, this.mixer.clipAction(clip));
    }
  }

  _collectMorphMeshes(obj) {
    if (obj.isMesh && obj.morphTargetDictionary) {
      this.morphMeshes.set(obj.name, obj);
    }
    for (const child of obj.children) this._collectMorphMeshes(child);
  }

  scrubClip(name, progress) {
    const action = this.actions.get(name);
    const clip = this.clipsByName.get(name);
    if (!action || !clip || clip.duration === 0) return;
    const t = Math.min(1, Math.max(0, progress));
    if (!action.isRunning()) {
      action.enabled = true;
      action.setEffectiveWeight(action.getEffectiveWeight() || 1);
      action.play();
    }
    action.paused = true;
    action.setEffectiveTimeScale(0);
    action.time = t * clip.duration;
    this.mixer.update(0);
  }

  update(delta) { this.mixer.update(delta); }

  dispose() {
    this.mixer.stopAllAction();
    this.actions.clear();
    this.morphMeshes.clear();
  }
}


// =============================================================================
// SECTION 8: SHADER DEFINITIONS
// =============================================================================

const FULLSCREEN_VS = `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`;

const BACKGROUND_VS = `varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}`;

const BACKGROUND_FS = `varying vec2 vUv;
uniform vec3 uColor1;
uniform vec3 uColor2;
uniform vec3 uColor1To;
uniform vec3 uColor2To;
uniform float uBackgroundMix;
uniform float uTime;

vec3 hash(vec3 p) {
  p = vec4(dot(p, vec3(127.1, 311.7, 74.7)),
           dot(p, vec3(269.5, 183.3, 246.1)),
           dot(p, vec3(113.5, 271.9, 124.6)),
           dot(p, vec3(42.1, 114.7, 311.3))).xyz;
  return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
}

float perlinNoise(vec3 p) {
  vec3 i = floor(p);
  vec3 f = fract(p);
  vec3 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  return mix(mix(mix(dot(hash(i + vec3(0,0,0)), f - vec3(0,0,0)),
                     dot(hash(i + vec3(1,0,0)), f - vec3(1,0,0)), u.x),
                 mix(dot(hash(i + vec3(0,1,0)), f - vec3(0,1,0)),
                     dot(hash(i + vec3(1,1,0)), f - vec3(1,1,0)), u.x), u.y),
             mix(mix(dot(hash(i + vec3(0,0,1)), f - vec3(0,0,1)),
                     dot(hash(i + vec3(1,0,1)), f - vec3(1,0,1)), u.x),
                 mix(dot(hash(i + vec3(0,1,1)), f - vec3(0,1,1)),
                     dot(hash(i + vec3(1,1,1)), f - vec3(1,1,1)), u.x), u.y), u.z);
}

void main() {
  float noise = perlinNoise(vec3(vUv.x*5., vUv.y*5., uTime));
  vec3 fromColor = mix(uColor1, uColor2, noise);
  vec3 toColor = mix(uColor1To, uColor2To, noise);
  vec3 color = mix(fromColor, toColor, smoothstep(0.0, 1.0, uBackgroundMix));
  gl_FragColor = vec4(color, 1.0);
}`;

const MESH_VS = `#include <common>
#include <morphtarget_pars_vertex>

varying vec2 vUv;
varying vec3 vPosition;

void main() {
  vUv = uv;
  #include <begin_vertex>
  #include <morphtarget_vertex>
  vPosition = transformed;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(transformed, 1.0);
}`;

const MESH_FS = `precision highp float;
precision highp sampler2D;

varying vec2 vUv;
uniform vec3 uColor1;
uniform vec3 uColor2;
uniform vec3 uMiddleColor;
uniform float uTime;
uniform float uHueShift;
uniform float uSaturation;
uniform sampler2D uColorRamp1;
uniform sampler2D uColorRamp1To;
uniform float uColorRamp1Mix;
uniform sampler2D uColorRamp2;
uniform sampler2D uColorRamp2To;
uniform float uColorRamp2Mix;
varying vec3 vPosition;

const float PI = 3.14159265359;

vec3 hash(vec3 p) {
  p = vec4(dot(p, vec3(127.1, 311.7, 74.7)),
           dot(p, vec3(269.5, 183.3, 246.1)),
           dot(p, vec3(113.5, 271.9, 124.6)),
           dot(p, vec3(42.1, 114.7, 311.3))).xyz;
  return -1.0 + 2.0 * fract(sin(p) * 43758.5453123);
}

float perlinNoise(vec3 p) {
  vec3 i = floor(p);
  vec3 f = fract(p);
  vec3 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  return mix(mix(mix(dot(hash(i + vec3(0,0,0)), f - vec3(0,0,0)),
                     dot(hash(i + vec3(1,0,0)), f - vec3(1,0,0)), u.x),
                 mix(dot(hash(i + vec3(0,1,0)), f - vec3(0,1,0)),
                     dot(hash(i + vec3(1,1,0)), f - vec3(1,1,0)), u.x), u.y),
             mix(mix(dot(hash(i + vec3(0,0,1)), f - vec3(0,0,1)),
                     dot(hash(i + vec3(1,0,1)), f - vec3(1,0,1)), u.x),
                 mix(dot(hash(i + vec3(0,1,1)), f - vec3(0,1,1)),
                     dot(hash(i + vec3(1,1,1)), f - vec3(1,1,1)), u.x), u.y), u.z);
}

float blenderNoise(vec3 p, float scale, float detail, float roughness) {
  p *= scale;
  float total = 0.0;
  float amplitude = 1.0;
  float maxValue = 0.0;
  int octaves = int(clamp(detail, 1.0, 16.0));
  for (int i = 0; i < octaves; i++) {
    float n = perlinNoise(p) * 0.5 + 0.5;
    total += n * amplitude;
    maxValue += amplitude;
    p *= 2.0;
    amplitude *= roughness;
  }
  return total / maxValue;
}

float blenderWave(vec3 p, float scale, float distortion, float detail, float roughness, float phaseShift) {
  float coord = p.x;
  if (distortion > 0.0) {
    float noise_distortion = blenderNoise(p, 1.0, detail, roughness);
    coord += distortion * (noise_distortion * 2.0 - 1.0);
  }
  coord = coord * scale + phaseShift;
  float wave = sin(coord);
  return wave * 0.5 + 0.5;
}

vec3 middleGradient(vec3 base, vec3 c1, vec3 c2, float wave) {
  float n = wave * perlinNoise(vec3(vUv.x, 1., (vUv.y - uTime*0.04) * .2) * 50.) + .2;
  vec3 baseColor = base;
  vec3 color = c1;
  float center = 0.48;
  color = mix(color, baseColor, smoothstep(center - .3, center - .2, n));
  vec3 grad = mix(baseColor, color, smoothstep(center - .2, center - .1, vUv.y));
  grad = mix(grad, baseColor, smoothstep(center + .1, center + .2, vUv.y));
  vec3 temp = grad;
  grad = mix(grad, c1, smoothstep(center - .1, center, vUv.y));
  grad = mix(grad, temp, smoothstep(center, center + .1, vUv.y));
  return grad;
}

vec3 rgb2hsv(vec3 c) {
  vec4 k = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
  vec4 p = mix(vec4(c.bg, k.wz), vec4(c.gb, k.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
  vec4 k = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + k.xyz) * 6.0 - k.www);
  return c.z * mix(k.xxx, clamp(p - k.xxx, 0.0, 1.0), c.y);
}

vec3 hueShift(vec3 color, float shift) {
  vec3 hsv = rgb2hsv(color);
  hsv.x = fract(hsv.x + shift);
  return hsv2rgb(hsv);
}

vec3 applySaturation(vec3 color, float saturation) {
  float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
  return mix(vec3(luma), color, saturation);
}

vec3 sampleColorRamp(sampler2D rampFrom, sampler2D rampTo, float rampMix, float t) {
  vec2 uv = vec2(0.5, t);
  vec3 from = texture2D(rampFrom, uv).rgb;
  vec3 to = texture2D(rampTo, uv).rgb;
  return mix(from, to, smoothstep(0.0, 1.0, rampMix));
}

void main() {
  vec3 color = vec3(0.0, 0.0, 0.0);
  float waveSpeed1 = .3;
  float noiseSpeedZ = .1;
  float noiseSpeedY = .2;
  float waveShift = perlinNoise(vec3(sin(vUv.x*PI*2.), sin(mod(uTime*waveSpeed1, PI*2.)), vUv.y)) * 0.5;
  float wave = sin(vUv.x * PI * 10. + waveShift) * 0.5 + 0.5;
  wave = smoothstep(1., 0.2, wave);
  float otherNoise = 1. - perlinNoise(vec3(sin(vUv.x*PI*2.), vUv.y-uTime*noiseSpeedY, uTime*noiseSpeedZ*2.));
  float colorVal = mix(otherNoise, wave, 0.5);
  color = vec3(colorVal);
  vec3 ramp1 = sampleColorRamp(uColorRamp1, uColorRamp1To, uColorRamp1Mix, colorVal);
  vec3 ramp2 = sampleColorRamp(uColorRamp2, uColorRamp2To, uColorRamp2Mix, colorVal);
  color = mix(ramp1, ramp2, vUv.y);
  color = middleGradient(color, uColor1, uColor2, wave);
  color = applySaturation(color, uSaturation);
  gl_FragColor = vec4(color, 1.0);
}`;

const CHROMATIC_ABERRATION = {
  uniforms: {
    tDiffuse: { value: null },
    uFactor:  { value: 0.05 },
  },
  vertexShader: FULLSCREEN_VS,
  fragmentShader: `
uniform sampler2D tDiffuse;
uniform float uFactor;
varying vec2 vUv;
void main() {
  vec2 uv = vUv;
  vec2 dir = uv - 0.5;
  float dist = length(dir);
  vec2 offset = dir * dist * uFactor;
  offset = clamp(offset, -0.05, 0.05);
  uv = (vUv - 0.5) * .9 + 0.5;
  float r = texture2D(tDiffuse, uv + offset).r;
  float g = texture2D(tDiffuse, uv).g;
  float b = texture2D(tDiffuse, uv - offset).b;
  gl_FragColor = vec4(r, g, b, 1.0);
}`,
};

const COLOR_GRADING = {
  uniforms: {
    tDiffuse:        { value: null },
    uCurvePoint:     { value: new THREE.Vector2(GRADE_CURVE_X, GRADE_CURVE_Y) },
    uLuminanceNoise: { value: 0.08 },
    uChromaNoise:    { value: 0.01 },
    uBrightness:     { value: 0.02 },
    uBlurTexel:      { value: new THREE.Vector2() },
  },
  vertexShader: FULLSCREEN_VS,
  fragmentShader: `
precision lowp float;
precision lowp sampler2D;
uniform sampler2D tDiffuse;
uniform vec2 uCurvePoint;
uniform float uLuminanceNoise;
uniform float uChromaNoise;
uniform float uBrightness;
uniform vec2 uBlurTexel;
varying vec2 vUv;

vec3 sampleDiffuse(vec2 uv) {
  vec2 o = uBlurTexel;
  if (dot(o, o) < 1e-8) return texture2D(tDiffuse, uv).rgb;
  vec3 c = texture2D(tDiffuse, uv).rgb;
  c += texture2D(tDiffuse, uv + vec2(o.x, 0.0)).rgb;
  c += texture2D(tDiffuse, uv - vec2(o.x, 0.0)).rgb;
  c += texture2D(tDiffuse, uv + vec2(0.0, o.y)).rgb;
  c += texture2D(tDiffuse, uv - vec2(0.0, o.y)).rgb;
  return c * 0.2;
}

float hash(vec2 p) {
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}

float curveLuma(float luma, vec2 point) {
  float x = point.x;
  float y = point.y;
  if (luma <= x) return (y / x) * luma;
  return y + ((1.0 - y) / (1.0 - x)) * (luma - x);
}

float ign(vec2 px) {
  return fract(52.9829189 * fract(dot(px, vec2(0.06711056, 0.00583715))));
}

void main() {
  vec3 color = sampleDiffuse(vUv).rgb;
  float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
  float mapped = clamp(curveLuma(luma, uCurvePoint), 0.0, 1.0);
  float gain = luma > 1e-5 ? mapped / luma : 1.0;
  color *= gain;
  vec2 px = gl_FragCoord.xy;
  color += (hash(px) - 0.5) * uLuminanceNoise;
  color.r += (hash(px + 17.0) - 0.5) * uChromaNoise;
  color.b += (hash(px + 43.0) - 0.5) * uChromaNoise;
  color += uBrightness;
  color += (ign(gl_FragCoord.xy) - 0.5) * (1.0 / 255.0);
  gl_FragColor = vec4(clamp(color, 0.0, 1.0), 1.0);
}`,
};


// =============================================================================
// SECTION 9: MAIN WEBGL ENGINE CLASS
// =============================================================================

class SagaEngine {
  constructor(canvas) {
    this.canvas = canvas;

    this.renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: false,
      powerPreference: "high-performance",
      alpha: false,
    });
    this.renderer.toneMapping = THREE.NoToneMapping;

    this.scene = new THREE.Scene();
    this.camera = null;
    this.baseCameraFov = null;
    this.designAspect = 1.5;

    this.timer = new Timer();
    this.rig = null;
    this.animationFrameId = null;

    this._dracoLoader = new DRACOLoader();
    this._dracoLoader.setDecoderPath(new URL("./draco/gltf/", import.meta.url).href);
    this._gltfLoader = new GLTFLoader();
    this._gltfLoader.setDRACOLoader(this._dracoLoader);

    this.colorRamp1 = createColorRamp(DEFAULT_RAMP1);
    this.colorRamp2 = createColorRamp(DEFAULT_RAMP2);

    this.backgroundColors = {
      color1: new THREE.Color(DEFAULT_BG_COLOR1),
      color2: new THREE.Color(DEFAULT_BG_COLOR2),
      transition: null,
    };

    this.backgroundUniforms = {
      uColor1:        { value: this.backgroundColors.color1 },
      uColor2:        { value: this.backgroundColors.color2 },
      uColor1To:      { value: this.backgroundColors.color1.clone() },
      uColor2To:      { value: this.backgroundColors.color2.clone() },
      uBackgroundMix: { value: 0 },
      uTime:          { value: 0 },
    };

    this.backgroundMat = new THREE.ShaderMaterial({
      uniforms: this.backgroundUniforms,
      vertexShader: BACKGROUND_VS,
      fragmentShader: BACKGROUND_FS,
    });

    this.meshUniforms = {
      uMiddleColor:   { value: new THREE.Color("#B6F2FF") },
      uColor1:        { value: new THREE.Color("#B6F2FF") },
      uColor2:        { value: new THREE.Color("#274fff") },
      uColorRamp1:    { value: this.colorRamp1.texture },
      uColorRamp1To:  { value: this.colorRamp1.texture },
      uColorRamp1Mix: { value: 0 },
      uColorRamp2:    { value: this.colorRamp2.texture },
      uColorRamp2To:  { value: this.colorRamp2.texture },
      uColorRamp2Mix: { value: 0 },
      uTime:          { value: 0 },
      uHueShift:      { value: 0 },
      uSaturation:    { value: 1 },
    };

    this.meshMat = new THREE.ShaderMaterial({
      uniforms: this.meshUniforms,
      vertexShader: MESH_VS,
      fragmentShader: MESH_FS,
    });

    this.postEffects = null;
    this.animationPosition = 0;
    this.startPosition = 0;
    this.positionAnimation = null;
    this.onReady = null;
  }

  resize = () => {
    const bounds = this.canvas.parentElement?.getBoundingClientRect();
    const viewport = window.visualViewport;
    const w = Math.max(1, Math.ceil(bounds?.width || viewport?.width || window.innerWidth));
    const h = Math.max(1, Math.ceil(bounds?.height || viewport?.height || window.innerHeight));
    const dpr = Math.min(window.devicePixelRatio, 2);

    this.renderer.setPixelRatio(dpr);
    this.renderer.setSize(w, h);

    if (this.camera) {
      const aspect = w / h;
      this.camera.aspect = aspect;
      if (this.baseCameraFov != null && aspect > this.designAspect) {
        const fovRad = 2 * Math.atan(
          Math.tan(this.baseCameraFov * Math.PI / 180 / 2) * (this.designAspect / aspect)
        );
        this.camera.fov = 180 * fovRad / Math.PI;
      } else if (this.baseCameraFov != null) {
        this.camera.fov = this.baseCameraFov;
      }
      this.camera.updateProjectionMatrix();
    }

    if (this.postEffects) {
      this.postEffects.setSize(w, h, dpr);
    }
  };

  tick = (timestamp) => {
    this.animationFrameId = requestAnimationFrame(this.tick);
    this.timer.update(timestamp);

    const elapsed = this.timer.getElapsed();
    const delta = this.timer.getDelta();

    this.meshUniforms.uTime.value = elapsed;
    this.backgroundUniforms.uTime.value = elapsed;

    this._updateColorRampTransition(elapsed);
    this._updateBackgroundColorTransition(elapsed);
    this._updatePositionAnimation(elapsed);

    if (this.rig) this.rig.update(delta);

    if (this.camera) {
      if (this.postEffects) {
        this.postEffects.render(delta);
      } else {
        this.renderer.render(this.scene, this.camera);
      }
    }
  };

  applyLoadedModel(gltf) {
    this.scene.add(gltf.scene);

    gltf.scene.traverse((obj) => {
      if (obj.isMesh) {
        if (obj.name === "background") obj.material = this.backgroundMat;
        if (obj.name === "mesh") obj.material = this.meshMat;
      }
    });

    this.rig = new AnimationRig(gltf.scene, gltf.animations);

    if (gltf.cameras[0] instanceof THREE.PerspectiveCamera) {
      this.camera = gltf.cameras[0];
      this.baseCameraFov = this.camera.fov;

      if (this.postEffects) this.postEffects.dispose();
      this.postEffects = createPostEffects(
        this.renderer, this.scene, this.camera,
        window.innerWidth, window.innerHeight
      );
    }

    this.resize();
    if (this.onReady) this.onReady();
    this.applyAnimationPosition(this.startPosition);
  }

  init(options) {
    this.onReady = options?.onReady ?? null;
    this.resize();
    window.addEventListener("resize", this.resize);
    window.visualViewport?.addEventListener("resize", this.resize);
    window.visualViewport?.addEventListener("scroll", this.resize);
    this.timer.connect();
    this.tick(performance.now());

    this._gltfLoader.load(new URL("./models/scene.glb", import.meta.url).href,
      (gltf) => this.applyLoadedModel(gltf),
      undefined,
      (err) => console.error("SagaEngine: Failed to load model", err)
    );
  }

  setStartPosition(pos) {
    const t = Math.min(1, Math.max(0, pos));
    this.animationPosition = t;
    this.applyAnimationPosition(t);
  }

  setAnimationPosition(pos) {
    const t = Math.min(1, Math.max(0, pos));
    this.animationPosition = t;
    if (this.positionAnimation) {
      this.positionAnimation.to = t;
      return;
    }
    this.applyAnimationPosition(t);
  }

  animateToPosition(target, duration, easing = "easeInOut") {
    const t = Math.min(1, Math.max(0, target));
    if (duration <= 0) {
      this.setAnimationPosition(t);
    } else {
      this.positionAnimation = {
        from: this.animationPosition,
        to: t,
        duration: duration,
        start: this._now(),
        easing: easing,
      };
    }
  }

  _now() {
    this.timer.update();
    return this.timer.getElapsed();
  }

  applyAnimationPosition(pos) {
    if (this.rig) this.rig.scrubClip("camera_action", pos);
    if (this.rig) this.rig.scrubClip("shape_key_action", pos);
  }

  _updatePositionAnimation(elapsed) {
    if (!this.positionAnimation) return;
    const { from, to, duration, start, easing } = this.positionAnimation;
    const progress = Math.min(1, (elapsed - start) / duration);
    const pos = from + (to - from) * easeValue(progress, easing);
    this.animationPosition = pos;
    this.applyAnimationPosition(pos);
    if (progress >= 1) {
      this.animationPosition = to;
      this.positionAnimation = null;
    }
  }

  setMeshSaturation(val) {
    this.meshUniforms.uSaturation.value = val;
  }

  setBackgroundColors(c1, c2) {
    this.backgroundColors.transition = null;
    this.backgroundColors.color1.set(c1);
    this.backgroundColors.color2.set(c2);
    this._syncBgColorUniforms(0);
  }

  animateBackgroundColors(c1, c2, duration = 1.2) {
    let mix = this.backgroundUniforms.uBackgroundMix.value;
    let to1 = this.backgroundUniforms.uColor1To.value;
    let to2 = this.backgroundUniforms.uColor2To.value;

    if (this.backgroundColors.transition) {
      if (mix > 0) {
        const t = smoothstep(0, 1, mix);
        this.backgroundColors.color1.lerp(to1, t);
        this.backgroundColors.color2.lerp(to2, t);
      }
      this.backgroundColors.transition = null;
    }

    const target1 = new THREE.Color(c1);
    const target2 = new THREE.Color(c2);

    this.backgroundUniforms.uColor1.value = this.backgroundColors.color1;
    this.backgroundUniforms.uColor2.value = this.backgroundColors.color2;
    this.backgroundUniforms.uColor1To.value = target1;
    this.backgroundUniforms.uColor2To.value = target2;
    this.backgroundUniforms.uBackgroundMix.value = 0;

    this.backgroundColors.transition = {
      color1To: target1,
      color2To: target2,
      duration: duration,
      start: this._now(),
    };
  }

  _syncBgColorUniforms(mix) {
    const { color1, color2 } = this.backgroundColors;
    this.backgroundUniforms.uColor1.value = color1;
    this.backgroundUniforms.uColor2.value = color2;
    this.backgroundUniforms.uColor1To.value = color1;
    this.backgroundUniforms.uColor2To.value = color2;
    this.backgroundUniforms.uBackgroundMix.value = mix;
  }

  // PX-exclusive theme. Swaps the live uniforms immediately and repoints the
  // module-level DEFAULT_* bindings so scroll-driven transitions also use the
  // PX palette while active. Restores everything when turned off, so no other
  // page ever sees these colors.
  setPxThemeActive(on) {
    const active = !!on;
    const bg1 = active ? PX_BG_COLOR1 : ORIGINAL_BG_COLOR1;
    const bg2 = active ? PX_BG_COLOR2 : ORIGINAL_BG_COLOR2;
    const ramp1 = active ? PX_RAMP1 : ORIGINAL_RAMP1;
    const ramp2 = active ? PX_RAMP2 : ORIGINAL_RAMP2;
    DEFAULT_BG_COLOR1 = bg1;
    DEFAULT_BG_COLOR2 = bg2;
    DEFAULT_RAMP1 = ramp1;
    DEFAULT_RAMP2 = ramp2;

    this._applyRampImmediate(this.colorRamp1, ramp1, {
      from: "uColorRamp1", to: "uColorRamp1To", mix: "uColorRamp1Mix",
    });
    this._applyRampImmediate(this.colorRamp2, ramp2, {
      from: "uColorRamp2", to: "uColorRamp2To", mix: "uColorRamp2Mix",
    });
    this.setBackgroundColors(bg1, bg2);
    this.meshUniforms.uColor1.value.set(active ? PX_MESH_COLOR1 : ORIGINAL_MESH_COLOR1);
    this.meshUniforms.uColor2.value.set(active ? PX_MESH_COLOR2 : ORIGINAL_MESH_COLOR2);
    this.meshUniforms.uMiddleColor.value.set(active ? PX_MESH_MIDDLE : ORIGINAL_MESH_MIDDLE);
  }

  _applyRampImmediate(slot, stops, uniforms) {
    if (slot.transition) {
      slot.transition.pendingTexture.dispose();
      slot.transition = null;
    }
    const seen = new Set();
    for (const tex of [slot.texture, this.meshUniforms[uniforms.from].value, this.meshUniforms[uniforms.to].value]) {
      if (tex && !seen.has(tex)) {
        seen.add(tex);
        tex.dispose();
      }
    }
    const next = createColorRamp(stops);
    slot.stops = next.stops;
    slot.texture = next.texture;
    this.meshUniforms[uniforms.from].value = next.texture;
    this.meshUniforms[uniforms.to].value = next.texture;
    this.meshUniforms[uniforms.mix].value = 0;
  }

  _updateBackgroundColorTransition(elapsed) {
    const t = this.backgroundColors.transition;
    if (!t) return;
    const { duration, start, color1To, color2To } = t;
    const progress = Math.min(1, (elapsed - start) / duration);
    this.backgroundUniforms.uBackgroundMix.value = progress;
    if (progress >= 1) {
      this.backgroundColors.color1.copy(color1To);
      this.backgroundColors.color2.copy(color2To);
      this._syncBgColorUniforms(0);
      this.backgroundColors.transition = null;
    }
  }

  transitionColorRamps(ramp1Stops, ramp2Stops, duration = 1.2) {
    this._startRampTransition(this.colorRamp1, ramp1Stops, duration, {
      from: "uColorRamp1", to: "uColorRamp1To", mix: "uColorRamp1Mix",
    });
    this._startRampTransition(this.colorRamp2, ramp2Stops, duration, {
      from: "uColorRamp2", to: "uColorRamp2To", mix: "uColorRamp2Mix",
    });
  }

  _startRampTransition(slot, newStops, duration, uniforms) {
    const currentMix = this.meshUniforms[uniforms.mix].value;
    const fromTex = this.meshUniforms[uniforms.from].value;
    const toTex = this.meshUniforms[uniforms.to].value;

    if (slot.transition) {
      if (currentMix > 0) {
        const blended = blendTextures(fromTex, toTex, currentMix);
        fromTex.dispose();
        if (toTex !== fromTex) toTex.dispose();
        slot.texture = blended;
      } else {
        slot.transition.pendingTexture.dispose();
      }
      slot.transition = null;
    }

    slot.stops = parseColorStops(newStops);
    const newTexture = generateGradientTexture(slot.stops);

    this.meshUniforms[uniforms.from].value = slot.texture;
    this.meshUniforms[uniforms.to].value = newTexture;
    this.meshUniforms[uniforms.mix].value = 0;

    slot.transition = {
      pendingTexture: newTexture,
      duration: duration,
      start: this._now(),
    };
  }

  _updateColorRampTransition(elapsed) {
    this._updateRampSlot(elapsed, this.colorRamp1, {
      from: "uColorRamp1", to: "uColorRamp1To", mix: "uColorRamp1Mix",
    });
    this._updateRampSlot(elapsed, this.colorRamp2, {
      from: "uColorRamp2", to: "uColorRamp2To", mix: "uColorRamp2Mix",
    });
  }

  _updateRampSlot(elapsed, slot, uniforms) {
    if (!slot.transition) return;
    const { duration, pendingTexture, start } = slot.transition;
    const progress = Math.min(1, (elapsed - start) / duration);
    this.meshUniforms[uniforms.mix].value = progress;
    if (progress < 1) return;

    const oldTex = slot.texture;
    slot.texture = pendingTexture;
    this.meshUniforms[uniforms.from].value = pendingTexture;
    this.meshUniforms[uniforms.to].value = pendingTexture;
    this.meshUniforms[uniforms.mix].value = 0;
    if (oldTex !== pendingTexture) oldTex.dispose();
    slot.transition = null;
  }

  dispose() {
    if (this.animationFrameId != null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    if (this.postEffects) { this.postEffects.dispose(); this.postEffects = null; }
    this._disposeRampSlot(this.colorRamp1);
    this._disposeRampSlot(this.colorRamp2);
    window.removeEventListener("resize", this.resize);
    window.visualViewport?.removeEventListener("resize", this.resize);
    window.visualViewport?.removeEventListener("scroll", this.resize);
    if (this.rig) { this.rig.dispose(); this.rig = null; }
    this.positionAnimation = null;
    this.startPosition = 0;
    this.timer.disconnect();
    this.renderer.dispose();
    this._dracoLoader.dispose();
  }

  _disposeRampSlot(slot) {
    if (slot.transition) {
      slot.transition.pendingTexture.dispose();
      slot.transition = null;
    }
    slot.texture.dispose();
  }
}


// =============================================================================
// SECTION 10: POST-PROCESSING PIPELINE
// =============================================================================

function createPostEffects(renderer, scene, camera, width, height) {
  const dpr = renderer.getPixelRatio();

  const composer = new EffectComposer(renderer);

  composer.addPass(new RenderPass(scene, camera));

  const chromaticPass = new ShaderPass(CHROMATIC_ABERRATION);
  composer.addPass(chromaticPass);

  const bloomPass = new UnrealBloomPass(
    new THREE.Vector2(width, height), 0.03, 0.885, 0
  );
  composer.addPass(bloomPass);

  const gradePass = new ShaderPass(COLOR_GRADING);
  composer.addPass(gradePass);

  function updateBlurTexel(w, h, pr) {
    gradePass.uniforms.uBlurTexel.value.set(1.5 / (w * pr), 1.5 / (h * pr));
  }

  updateBlurTexel(width, height, dpr);

  return {
    render(delta) {
      bloomPass.strength = 0.03 * Math.sin(0.001 * performance.now()) + 0.1;
      composer.render(delta);
    },
    setSize(w, h, pr) {
      if (pr !== undefined) composer.setPixelRatio(pr);
      composer.setSize(w, h);
      updateBlurTexel(w, h, pr ?? renderer.getPixelRatio());
    },
    dispose() {
      composer.dispose();
    },
  };
}


// =============================================================================
// SECTION 11: SCROLL HIJACK SYSTEM
// =============================================================================
//
// Only the overview section's sub-blocks (01-04) are fixed and blurred.
// Everything else scrolls normally: hero, lead, CTA, footer.
//
// Layout:
//   [Hero     — normal scroll]
//   [Lead     — normal scroll]
//   [Overview — FIXED in viewport, sub-blocks blur in/out]
//   [Spacer   — replaces overview's natural height for scroll]
//   [CTA      — normal scroll]
//   [Footer   — normal scroll, top gradient]
// =============================================================================

class ScrollSystem {
  constructor(engine) {
    this.engine = engine;
    this.scrollFromEls = [];
    this.scrollToEls = [];
    this.overviewSection = null;
    this.subBlocks = [];
    this.overviewTop = 0;
    this.scrollZoneHeight = 0;
    this.lastColorRampIdx = -1;
    this.rafId = 0;
    this.scrollProgressBar = null;
    this.markerScrollTops = new Map();

    this._parseMarkers();
    this._identifyOverview();
    this._buildScrollFrame();
    this._createProgressBar();
    this._applyStyles();
    this._bind();

    requestAnimationFrame(() => this._update());
  }

  _parseMarkers() {
    const all = getVisibleWebGLMarkers();
    for (const el of all) {
      const type = el.getAttribute("data-webgl-marker");
      if (type === "scrollFrom")      this.scrollFromEls.push(el);
      else if (type === "scrollTo")   this.scrollToEls.push(el);
    }
  }

  _identifyOverview() {
    this.overviewSection = document.querySelector(
      'main [data-slice-type="overview"]'
    );
    if (!this.overviewSection) return;

    const inner = this.overviewSection.querySelector(".relative");
    if (!inner) return;

    // Try multiple selectors to find sub-blocks
    this.subBlocks = inner.querySelectorAll(
      ':scope > .space-y-fluid-\\[106\\,212\\] > .ui-grid, :scope > div > .ui-grid'
    );
    if (this.subBlocks.length === 0) {
      this.subBlocks = inner.querySelectorAll(".ui-grid.relative");
    }
    if (this.subBlocks.length === 0) {
      // Try finding direct children that are grids
      this.subBlocks = inner.querySelectorAll(':scope > .space-y-fluid-\\[106\\,212\\] > div > .ui-grid');
    }
    if (this.subBlocks.length === 0) {
      // Fallback: find all ui-grid elements within the space-y container
      const spaceContainer = inner.querySelector('.space-y-fluid-\\[106\\,212\\]');
      if (spaceContainer) {
        this.subBlocks = spaceContainer.querySelectorAll('.ui-grid');
      }
    }
    if (this.subBlocks.length === 0) {
      // Final fallback: all ui-grid elements
      this.subBlocks = inner.querySelectorAll('.ui-grid');
    }
    
    // Add overview-sub-block class to identified elements
    this.subBlocks.forEach((block, i) => {
      block.classList.add('overview-sub-block');
    });
  }

  _buildScrollFrame() {
    if (!this.overviewSection || this.subBlocks.length === 0) return;

    this._cacheMarkerPositions();

    this.overviewTop = getElementScrollTop(this.overviewSection);
    // Use distance from first scrollFrom to first scrollTo as scroll zone height
    // This matches saga.xyz behavior where scroll zone = marker distance
    if (this.scrollFromEls.length > 0 && this.scrollToEls.length > 0) {
      const firstFromTop = this._cachedScrollTop(this.scrollFromEls[0]);
      const firstToTop = this._cachedScrollTop(this.scrollToEls[0]);
      this.scrollZoneHeight = firstToTop - firstFromTop;
    } else if (this.scrollFromEls.length > 0) {
      const firstFromTop = this._cachedScrollTop(this.scrollFromEls[0]);
      this.scrollZoneHeight = this.overviewTop - firstFromTop;
    } else {
      // Fallback to viewport-based calculation
      this.scrollZoneHeight = (this.subBlocks.length + 4) * window.innerHeight;
    }

    const spacer = document.createElement("div");
    spacer.className = "saga-scroll-spacer";
    spacer.setAttribute("aria-hidden", "true");
    spacer.style.cssText =
      "height:" + this.scrollZoneHeight + "px;pointer-events:none;";

    if (this.overviewSection.nextSibling) {
      this.overviewSection.parentNode.insertBefore(
        spacer, this.overviewSection.nextSibling
      );
    } else {
      this.overviewSection.parentNode.appendChild(spacer);
    }

    // Keep overview section static, only fix sub-blocks
    // this.overviewSection.style.position = "fixed";
    // this.overviewSection.style.inset = "0";
    // this.overviewSection.style.zIndex = "50";
    // this.overviewSection.style.willChange = "filter, opacity";
    // this.overviewSection.style.pointerEvents = "none";
    // this.overviewSection.style.overflow = "hidden";

    const inner = this.overviewSection.querySelector(".relative");
    if (inner) {
      inner.style.position = "absolute";
      inner.style.inset = "0";
      inner.style.overflow = "hidden";
    }

    for (const block of this.subBlocks) {
      block.classList.add('overview-sub-block');
      block.style.willChange = "filter, opacity";
      block.style.opacity = "0";
      block.style.filter = "blur(8px)";

      const markers = block.querySelectorAll(":scope > .pointer-events-none");
      for (const m of markers) {
        m.style.position = "absolute";
        m.style.visibility = "hidden";
      }
    }
  }

  _cacheMarkerPositions() {
    const allMarkers = [...this.scrollFromEls, ...this.scrollToEls];
    for (const el of allMarkers) {
      this.markerScrollTops.set(el, getElementScrollTop(el));
    }
  }

  _applyStyles() {
    if (document.getElementById("saga-scroll-styles")) return;
    const s = document.createElement("style");
    s.id = "saga-scroll-styles";
    s.textContent = [
      '[data-slice-type="hero"]{position:relative;z-index:2}',
      '.saga-scroll-spacer{position:relative;z-index:0}',
      '.saga-scroll-spacer ~ section{position:relative;z-index:2}',
      '/* Overview section stays static, sub-blocks get fixed positioning */',
      '.overview-sub-block {',
      '  position: fixed !important;',
      '  top: 0 !important;',
      '  left: 0 !important;',
      '  width: 100% !important;',
      '  height: 100vh !important;',
      '  display: flex !important;',
      '  flex-direction: column !important;',
      '  align-items: center !important;',
      '  justify-content: center !important;',
      '  overflow: hidden !important;',
      '  padding: 0 !important;',
      '  margin: 0 !important;',
      '  z-index: 10;',
      '  pointer-events: none;',
      '}',
      '.overview-sub-block > .col-span-full {',
      '  flex: 0 0 auto;',
      '  flex-direction: row !important;',
      '  width: 100%;',
      '}'
    ].join("\n");
    document.head.appendChild(s);
  }

  _createProgressBar() {
    const old = document.getElementById("saga-scroll-progress");
    if (old) old.remove();
    const bar = document.createElement("div");
    bar.id = "saga-scroll-progress";
    bar.setAttribute("aria-hidden", "true");
    bar.style.cssText =
      "position:fixed;top:0;left:0;height:1px;width:100%;" +
      "background:#ffffff;" +
      "z-index:10000;pointer-events:none;transform-origin:left;transform:translate3d(0px, 0px, 0px) scale(0, 1)";
    document.body.prepend(bar);
    this.scrollProgressBar = bar;
  }

  _bind() {
    const onScroll = () => {
      if (!this.rafId) {
        this.rafId = requestAnimationFrame(() => {
          this.rafId = 0;
          this._update();
        });
      }
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll, { passive: true });
  }

  _update() {
    this._updateOverviewSubBlocks();
    this._updateProgressBar();
    this._updateColorRampsFromProgress();
    this._updateScrollAnimation();
  }

  _updateOverviewSubBlocks() {
    if (!this.overviewSection || this.subBlocks.length === 0) return;

    const scrollY = window.scrollY;
    const scrollZoneStart = this._cachedScrollTop(this.scrollFromEls[0]);
    const scrollZoneEnd = this._cachedScrollTop(this.scrollToEls[0]);

    // Show blocks when we're in the scroll zone (from scrollFrom to scrollTo, inclusive)
    if (scrollY < scrollZoneStart || scrollY > scrollZoneEnd + 1) {
      this.overviewSection.style.opacity =
        scrollY < scrollZoneStart ? "1" : "0";
      this.overviewSection.style.pointerEvents = "none";
      for (const b of this.subBlocks) {
        b.style.opacity = "0";
        b.style.filter = "blur(10px)";
      }
      return;
    }

    this.overviewSection.style.opacity = "1";
    this.overviewSection.style.pointerEvents = "none";

    // In the scroll zone, show all blocks fully visible (matching saga.xyz behavior)
    for (const b of this.subBlocks) {
      b.style.opacity = "1";
      b.style.filter = "none";
      b.style.pointerEvents = "auto";
    }
  }

  _updateProgressBar() {
    if (!this.scrollProgressBar) return;
    const scrollY = window.scrollY;
    let progress = 0;
    if (scrollY >= this.overviewTop && this.scrollZoneHeight > 0) {
      progress = Math.min(1, Math.max(0,
        (scrollY - this.overviewTop) / this.scrollZoneHeight
      ));
    } else if (scrollY >= this.overviewTop + this.scrollZoneHeight) {
      progress = 1;
    }
    this.scrollProgressBar.style.transform = "translate3d(0px, 0px, 0px) scale(" + progress + ", 1)";
  }

  _updateColorRampsFromProgress() {
    if (this.subBlocks.length === 0) return;
    const scrollY = window.scrollY;
    const scrollEnd = this.overviewTop + this.scrollZoneHeight;
    let activeIdx = -1;

    if (scrollY >= this.overviewTop && scrollY < scrollEnd) {
      const scrollDistance = scrollY - this.overviewTop;
      activeIdx = Math.min(
        this.subBlocks.length - 1,
        Math.floor((scrollDistance / this.scrollZoneHeight) * this.subBlocks.length)
      );
    }

    if (activeIdx === this.lastColorRampIdx) return;
    this.lastColorRampIdx = activeIdx;

    if (activeIdx === -1) {
      this.engine.transitionColorRamps(DEFAULT_RAMP1, DEFAULT_RAMP2, 1.2);
      this.engine.animateBackgroundColors(DEFAULT_BG_COLOR1, DEFAULT_BG_COLOR2, 1.2);
    } else {
      const ramp = SECTION_RAMPS[Math.min(activeIdx, SECTION_RAMPS.length - 1)];
      this.engine.transitionColorRamps(ramp.ramp1, ramp.ramp2, 1.2);
      this.engine.animateBackgroundColors(ramp.bg1, ramp.bg2, 1.2);
    }
  }

  _cachedScrollTop(el) {
    return this.markerScrollTops.has(el) ? this.markerScrollTops.get(el) : getElementScrollTop(el);
  }

  _updateScrollAnimation() {
    if (!this.engine.rig) return;
    const pairs = this._pairScrollMarkers();
    if (pairs.length === 0) return;

    const scrollY = window.scrollY;
    const threshold = window.innerHeight / 2;

    const firstFromScroll = this._cachedScrollTop(pairs[0].from) - threshold;
    if (scrollY < firstFromScroll) {
      this.engine.setAnimationPosition(pairs[0].fromPos);
      return;
    }

    for (let i = 0; i < pairs.length; i++) {
      const { from, to, fromPos, toPos, easing } = pairs[i];
      const fromScroll = this._cachedScrollTop(from) - threshold;
      const toScroll = this._cachedScrollTop(to) - threshold;

      if (scrollY < fromScroll) {
        this.engine.setAnimationPosition(i > 0 ? pairs[i - 1].toPos : fromPos);
        return;
      }

      if (scrollY <= toScroll) {
        if (toScroll <= fromScroll) {
          this.engine.setAnimationPosition(fromPos);
          return;
        }
        const raw = Math.max(0, Math.min(1,
          (scrollY - fromScroll) / (toScroll - fromScroll)
        ));
        this.engine.setAnimationPosition(
          fromPos + (toPos - fromPos) * easeValue(raw, easing)
        );
        return;
      }
    }

    this.engine.setAnimationPosition(pairs[pairs.length - 1].toPos);
  }

  _pairScrollMarkers() {
    const fromEls = [...this.scrollFromEls].sort(
      (a, b) => this._cachedScrollTop(a) - this._cachedScrollTop(b)
    );
    const toEls = [...this.scrollToEls].sort(
      (a, b) => this._cachedScrollTop(a) - this._cachedScrollTop(b)
    );

    const pairs = [];
    let fi = 0, ti = 0;
    while (fi < fromEls.length && ti < toEls.length) {
      const fromEl = fromEls[fi];
      const toEl = toEls[ti];
      if (this._cachedScrollTop(fromEl) < this._cachedScrollTop(toEl)) {
        const fromPos = parseFloat(fromEl.getAttribute("data-webgl-position")) || 0;
        const toPos = parseFloat(toEl.getAttribute("data-webgl-position")) || 1;
        const easing = fromEl.getAttribute("data-webgl-easing") || "easeInOut";
        pairs.push({ from: fromEl, to: toEl, fromPos, toPos, easing });
        fi++; ti++;
      } else {
        ti++;
      }
    }
    return pairs;
  }
}


// =============================================================================
// SECTION 12: INITIALIZATION
// =============================================================================

class ReactOwnedScrollSystem {
  constructor(engine) {
    this.engine = engine;
    this.scrollFromEls = [];
    this.scrollToEls = [];
    this.overviewSection = null;
    this.lastColorRampIdx = -2;
    this.rafId = 0;

    this._parseMarkers();
    this._bind();
    requestAnimationFrame(() => this._update());
  }

  _parseMarkers() {
    const all = getVisibleWebGLMarkers();
    this.scrollFromEls = [];
    this.scrollToEls = [];
    for (const el of all) {
      const type = el.getAttribute("data-webgl-marker");
      if (type === "scrollFrom") this.scrollFromEls.push(el);
      if (type === "scrollTo") this.scrollToEls.push(el);
    }
    this.overviewSection = document.querySelector('main [data-slice-type="overview"]');
  }

  _bind() {
    const schedule = () => {
      if (this.rafId) return;
      this.rafId = requestAnimationFrame(() => {
        this.rafId = 0;
        this._parseMarkers();
        this._update();
      });
    };
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule, { passive: true });
  }

  _scrollTop(el) {
    return el.getBoundingClientRect().top + window.scrollY;
  }

  _update() {
    this._updateScrollAnimation();
    this._updateColorRamps();
  }

  _updateScrollAnimation() {
    if (!this.engine.rig || this.scrollFromEls.length === 0 || this.scrollToEls.length === 0) return;
    const from = this.scrollFromEls[0];
    const to = this.scrollToEls[0];
    const threshold = window.innerHeight / 2;
    const start = this._scrollTop(from) - threshold;
    const end = this._scrollTop(to) - threshold;
    const fromPos = parseFloat(from.getAttribute("data-webgl-position")) || 0;
    const toPos = parseFloat(to.getAttribute("data-webgl-position")) || 1;
    const easing = from.getAttribute("data-webgl-easing") || "easeInOut";

    if (end <= start) {
      this.engine.setAnimationPosition(fromPos);
      return;
    }

    const raw = Math.max(0, Math.min(1, (window.scrollY - start) / (end - start)));
    this.engine.setAnimationPosition(fromPos + (toPos - fromPos) * easeValue(raw, easing));
  }

  _updateColorRamps() {
    if (!this.overviewSection) return;

    const top = this._scrollTop(this.overviewSection);
    const height = Math.max(1, this.overviewSection.offsetHeight - window.innerHeight);
    const progress = (window.scrollY - top) / height;
    const activeIdx = progress < 0 || progress > 1 ? -1 : Math.min(3, Math.floor(Math.max(0, progress) * 4));

    if (activeIdx === this.lastColorRampIdx) return;
    this.lastColorRampIdx = activeIdx;

    if (activeIdx < 0) {
      this.engine.transitionColorRamps(DEFAULT_RAMP1, DEFAULT_RAMP2, 1.2);
      this.engine.animateBackgroundColors(DEFAULT_BG_COLOR1, DEFAULT_BG_COLOR2, 1.2);
      return;
    }

    const ramp = SECTION_RAMPS[Math.min(activeIdx, SECTION_RAMPS.length - 1)];
    this.engine.transitionColorRamps(ramp.ramp1, ramp.ramp2, 1.2);
    this.engine.animateBackgroundColors(ramp.bg1, ramp.bg2, 1.2);
  }
}

let sagaEngineInstance = null;

if (typeof window !== "undefined") {
  // Page-scoped theme switch. Only the PX page calls this (via its
  // data-webgl-theme="px" marker); every other page keeps the defaults.
  window.__sagaPxTheme = {
    setActive(on) {
      window.__sagaPendingPxTheme = !!on;
      if (sagaEngineInstance) sagaEngineInstance.setPxThemeActive(!!on);
    },
  };
}

function bootstrap() {
  const canvas = document.querySelector(
    ".fixed.inset-0 canvas, .fixed canvas.size-full, [class*='fixed'] canvas"
  );
  if (!canvas) {
    console.error("SagaEngine: No canvas element found. Expected <canvas> inside a .fixed container.");
    return;
  }

  if (!canvas.getContext("webgl2") && !canvas.getContext("webgl")) {
    console.error("SagaEngine: WebGL is not supported in this browser.");
    return;
  }

  const engine = new SagaEngine(canvas);
  sagaEngineInstance = engine;
  engine.init({
    onReady() {
      console.log("SagaEngine: Model loaded, marker scroll system active.");
      new ReactOwnedScrollSystem(engine);
    },
  });
  if (window.__sagaPendingPxTheme === true || document.querySelector('[data-webgl-theme="px"]')) {
    engine.setPxThemeActive(true);
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap);
} else {
  bootstrap();
}
