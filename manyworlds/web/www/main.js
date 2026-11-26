// The browser shell for Many Worlds Line. Everything that is the game (rules, drawing, the
// tutorial) is in WebAssembly; this file only:
//  - sizes the canvas in true physical pixels and shows the 400x300 frame at the largest exact
//    integer scale N, centred on black (the frame arrives already expanded to N x N blocks),
//  - makes the hardware cursor from the game's pointer sprite at scale N,
//  - latches every input event as it arrives and hands them over in order,
//  - runs the game at a steady 15 FPS, with the song's playback position as the clock,
//  - plays a few quiet sound effects.

const FB_W = 400, FB_H = 300, TICK_MS = 1000 / 15;

const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d', { alpha: false });
const song = document.getElementById('song');

const bytes = await (await fetch('manyworlds_web.wasm')).arrayBuffer();
const { instance } = await WebAssembly.instantiate(bytes, {});
const w = instance.exports;
w.mw_init(289);

// --- the song is the clock ---------------------------------------------------------------
let songOk = false;
function songReady() {
  songOk = Number.isFinite(song.duration) && song.duration > 30;
  if (songOk) w.mw_set_length(Math.floor(song.duration));
}
song.addEventListener('loadedmetadata', songReady);
// A cached song may have loaded while the WebAssembly was still on its way.
if (song.readyState >= 1) songReady();
// Without the music file the leg still runs, on a silent clock of the same length.
const silent = { ms: 0, running: false };

let audioGen = 0;
function syncAudio() {
  const gen = w.mw_audio_gen();
  if (gen !== audioGen) {
    audioGen = gen;
    silent.ms = 0;
    if (songOk) song.currentTime = 0;
  }
  const want = w.mw_wants_audio() === 1;
  silent.running = want;
  if (!songOk) return;
  if (want && song.paused && !song.ended) song.play().catch(() => {});
  if (!want && !song.paused) song.pause();
}

function songTick() {
  return songOk ? Math.floor(song.currentTime * 15) : Math.floor(silent.ms / TICK_MS);
}

// --- physical pixels and the integer scale ------------------------------------------------
let N = 1, offX = 0, offY = 0, dpr = 1, image = null;

function layout() {
  dpr = window.devicePixelRatio || 1;
  const W = Math.round(window.innerWidth * dpr);
  const H = Math.round(window.innerHeight * dpr);
  canvas.width = W;
  canvas.height = H;
  N = Math.max(1, Math.min(Math.floor(W / FB_W), Math.floor(H / FB_H)));
  offX = Math.floor((W - FB_W * N) / 2);
  offY = Math.floor((H - FB_H * N) / 2);
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, W, H);
  image = ctx.createImageData(FB_W * N, FB_H * N);
  makeCursor();
  draw();
}

function draw() {
  const ptr = w.mw_present(N);
  image.data.set(new Uint8ClampedArray(w.memory.buffer, ptr, FB_W * N * FB_H * N * 4));
  ctx.putImageData(image, offX, offY);
}

// The pointer is the OS cursor, built from the game's sprite with every logical pixel an exact
// N x N block of physical pixels, fully opaque or fully clear. It is never drawn into the frame.
function makeCursor() {
  const ptr = w.mw_cursor(N);
  const cw = w.mw_cursor_w(), ch = w.mw_cursor_h();
  const c = document.createElement('canvas');
  c.width = cw;
  c.height = ch;
  const px = new Uint8ClampedArray(w.memory.buffer.slice(ptr, ptr + cw * ch * 4));
  c.getContext('2d').putImageData(new ImageData(px, cw, ch), 0, 0);
  const url = c.toDataURL('image/png');
  canvas.style.cursor = '';
  canvas.style.cursor = `image-set(url("${url}") ${dpr}x) 0 0, auto`;
  if (!canvas.style.cursor) canvas.style.cursor = `-webkit-image-set(url("${url}") ${dpr}x) 0 0, auto`;
  if (!canvas.style.cursor) canvas.style.cursor = `url("${url}") 0 0, auto`;
}

window.addEventListener('resize', layout);
matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`).addEventListener?.('change', layout);

// --- input: latch everything, in order -----------------------------------------------------
// Logical pixel = (physical - viewport offset) / N, using the offset and N of the last layout.
function logical(e) {
  return [Math.floor((e.clientX * dpr - offX) / N), Math.floor((e.clientY * dpr - offY) / N)];
}

canvas.addEventListener('pointermove', (e) => {
  const samples = e.getCoalescedEvents ? e.getCoalescedEvents() : [e];
  for (const s of samples.length ? samples : [e]) {
    const [x, y] = logical(s);
    w.mw_move(x, y);
  }
});
canvas.addEventListener('pointerdown', (e) => {
  canvas.setPointerCapture(e.pointerId);
  const [x, y] = logical(e);
  w.mw_button(e.button, 1, x, y);
  gesture();
  e.preventDefault();
});
canvas.addEventListener('pointerup', (e) => {
  const [x, y] = logical(e);
  w.mw_button(e.button, 0, x, y);
});
canvas.addEventListener('contextmenu', (e) => e.preventDefault());

const KEYS = { Space: 1, KeyE: 2, KeyP: 3, KeyR: 4, F2: 5, KeyM: 6, KeyL: 7, Escape: 8, Enter: 9 };
window.addEventListener('keydown', (e) => {
  const k = KEYS[e.code];
  if (!k) return;
  e.preventDefault();
  if (!e.repeat) w.mw_key(k, 1);
  gesture();
});
window.addEventListener('keyup', (e) => {
  const k = KEYS[e.code];
  if (k) w.mw_key(k, 0);
});
window.addEventListener('blur', () => w.mw_key(1, 0)); // let go of express
// A hidden page gets few or no frames while the song plays on, so pause instead of falling behind.
document.addEventListener('visibilitychange', () => {
  if (document.hidden) w.mw_pause();
});
// For testing: #debug exposes the game's exports as window.mw.
if (location.hash === '#debug') window.mw = w;

// Browsers only allow sound after the player has done something: start it from that gesture.
function gesture() {
  if (!ac) ac = new AudioContext();
  if (ac.state === 'suspended') ac.resume();
  if (songOk && song.paused && w.mw_screen() !== 1) song.play().then(() => syncAudio()).catch(() => {});
}

// --- quiet sound effects -------------------------------------------------------------------
let ac = null;
function tone(freq, start, dur, vol, type = 'triangle', to = freq) {
  const t = ac.currentTime + start;
  const o = ac.createOscillator();
  const g = ac.createGain();
  o.type = type;
  o.frequency.setValueAtTime(freq, t);
  o.frequency.linearRampToValueAtTime(to, t + dur);
  g.gain.setValueAtTime(vol, t);
  g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  o.connect(g).connect(ac.destination);
  o.start(t);
  o.stop(t + dur + 0.02);
}
function sfx(bits) {
  if (!ac || !bits) return;
  if (bits & 1) { tone(660, 0, 0.12, 0.05); tone(990, 0.07, 0.18, 0.05); }    // delivered
  if (bits & 2) tone(523, 0, 0.08, 0.03);                                      // boarded
  if (bits & 4) tone(140, 0, 0.25, 0.06, 'square', 90);                       // stall
  if (bits & 8) tone(110, 0, 0.4, 0.07, 'sawtooth', 50);                       // crash
  if (bits & 16) [784, 988, 1175].forEach((f, i) => tone(f, i * 0.09, 0.3, 0.05)); // lamp
  if (bits & 32) tone(1400, 0, 0.03, 0.02, 'square');                           // click
  if (bits & 64) [523, 659, 784, 1047].forEach((f, i) => tone(f, i * 0.12, 0.4, 0.05)); // last stop
  if (bits & 128) tone(1800, 0, 0.015, 0.012, 'square');                         // rail laid
}

// --- the 15 FPS loop -----------------------------------------------------------------------
layout();
let last = performance.now();
let acc = 0;
function loop(now) {
  const dt = now - last;
  last = now;
  if (silent.running) silent.ms += dt;
  acc = Math.min(acc + dt, 250);
  while (acc >= TICK_MS) {
    acc -= TICK_MS;
    syncAudio();
    w.mw_song_tick(songTick());
    w.mw_frame();
    sfx(w.mw_take_sfx());
    draw();
  }
  requestAnimationFrame(loop);
}
requestAnimationFrame(loop);
