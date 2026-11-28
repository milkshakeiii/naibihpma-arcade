// The browser shell for Naibihpma Hold. Everything that is the game is in WebAssembly; this file:
//  - sizes the canvas in true physical pixels and shows the 400x300 frame at the largest exact
//    integer scale N, centred on black (the frame arrives already expanded to N x N blocks),
//  - makes the hardware cursor from the game's pointer sprite at scale N,
//  - latches every input event as it arrives and hands them over in order,
//  - runs the game at a steady 15 FPS,
//  - plays The Enchanted Forest of Naibihpma in album order as the soundtrack (it never steers
//    the game), plus a few quiet sound effects,
//  - posts Henry's in-game feedback to /feedback (scripts/demo_server.py writes it to a file).

const FB_W = 400, FB_H = 300, TICK_MS = 1000 / 15;

const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d', { alpha: false });
const song = document.getElementById('song');

const bytes = await (await fetch('hold_web.wasm')).arrayBuffer();
const { instance } = await WebAssembly.instantiate(bytes, {});
const w = instance.exports;
w.hd_init((Math.random() * 0xffffffff) >>> 0);

// --- the soundtrack -------------------------------------------------------------------------
const TRACKS = [
  'The Underbrush', 'Daot the Enchanter', 'Twen, Dew Collector',
  'Star Pond', 'Gorf, Dark Sorcerer', 'Ancient Magic',
];
let track = 0;
let musicOn = false;
function loadTrack(i) {
  track = i % TRACKS.length;
  song.src = `assets/music/forest-${track + 1}.m4a`;
}
loadTrack(0);
song.addEventListener('ended', () => {
  loadTrack(track + 1);
  if (musicOn) song.play().catch(() => {});
});
// Without the music files the game still runs, silently.
song.addEventListener('error', () => console.warn('no soundtrack file for', TRACKS[track]));

function syncMusic() {
  const want = w.hd_wants_music() === 1 && !document.hidden;
  if (want && !musicOn) {
    musicOn = true;
    song.play().catch(() => {});
  } else if (!want && musicOn) {
    musicOn = false;
    song.pause();
  }
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
  const ptr = w.hd_present(N);
  image.data.set(new Uint8ClampedArray(w.memory.buffer, ptr, FB_W * N * FB_H * N * 4));
  ctx.putImageData(image, offX, offY);
}

// The pointer is the OS cursor, built from the game's sprite with every logical pixel an exact
// N x N block of physical pixels, fully opaque or fully clear. It is never drawn into the frame.
function makeCursor() {
  const ptr = w.hd_cursor(N);
  const cw = w.hd_cursor_w(), ch = w.hd_cursor_h();
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
    w.hd_move(x, y);
  }
});
canvas.addEventListener('pointerdown', (e) => {
  canvas.setPointerCapture(e.pointerId);
  canvas.focus();
  const [x, y] = logical(e);
  w.hd_button(e.button, 1, x, y);
  gesture();
  e.preventDefault();
});
canvas.addEventListener('pointerup', (e) => {
  const [x, y] = logical(e);
  w.hd_button(e.button, 0, x, y);
});
canvas.addEventListener('contextmenu', (e) => e.preventDefault());

const SPECIAL = {
  Tab: 1, Enter: 2, NumpadEnter: 2, Escape: 3, Backspace: 4,
  ArrowUp: 5, ArrowDown: 6, ArrowLeft: 7, ArrowRight: 8, F2: 9,
  F3: 10, F4: 11, F5: 12, F6: 13, F7: 14, F8: 15, F9: 16, Home: 17,
};
// WASD scroll the view while held: down and up are sent as their own codes (18-21, 22-25), and
// the letter is still sent once as a character, for any button that uses it.
const SCROLL = { KeyW: 0, KeyA: 1, KeyS: 2, KeyD: 3 };
const held = new Set();
function releaseScroll() {
  for (const d of held) w.hd_key(22 + d, 0);
  held.clear();
}
window.addEventListener('keyup', (e) => {
  const d = SCROLL[e.code];
  if (d !== undefined && held.has(d)) {
    held.delete(d);
    w.hd_key(22 + d, 0);
  }
});
window.addEventListener('blur', releaseScroll);
window.addEventListener('keydown', (e) => {
  if (e.metaKey || e.ctrlKey || e.altKey) return; // leave browser shortcuts alone
  const d = SCROLL[e.code];
  if (d !== undefined && w.hd_typing() !== 1) {
    e.preventDefault();
    if (!e.repeat) {
      held.add(d);
      w.hd_key(18 + d, 0);
      w.hd_char(e.key.codePointAt(0));
    }
    gesture();
    return;
  }
  const code = SPECIAL[e.code] ?? SPECIAL[e.key];
  if (code) {
    e.preventDefault();
    if (!e.repeat || code >= 4) w.hd_key(code, e.shiftKey ? 1 : 0);
  } else if (e.key.length === 1) {
    e.preventDefault();
    if (!e.repeat || w.hd_typing() === 1) w.hd_char(e.key.codePointAt(0));
  } else {
    return;
  }
  gesture();
});
// A hidden page gets few or no frames: pause rather than run unseen.
document.addEventListener('visibilitychange', () => {
  if (document.hidden) w.hd_pause();
  syncMusic();
});
// For testing: #debug exposes the game's exports as window.hd, and window.hdStep(n) runs n
// frames and redraws (useful when the page gets no animation frames, e.g. a hidden pane).
if (location.hash === '#debug') {
  window.hd = w;
  window.hdStep = (n = 1) => {
    for (let i = 0; i < n; i++) {
      w.hd_frame();
      sfx(w.hd_take_sfx());
      sendFeedback();
    }
    draw();
  };
}

// Browsers only allow sound after the player has done something: start it from that gesture.
function gesture() {
  if (!ac) ac = new AudioContext();
  if (ac.state === 'suspended') ac.resume();
}

// --- feedback ------------------------------------------------------------------------------
const decoder = new TextDecoder();
function sendFeedback() {
  while (w.hd_outbox_len() > 0) {
    const ptr = w.hd_outbox_ptr(), size = w.hd_outbox_size();
    const text = decoder.decode(new Uint8Array(w.memory.buffer, ptr, size));
    w.hd_outbox_pop();
    let entry;
    try { entry = JSON.parse(text); } catch { entry = { kind: 'note', text }; }
    entry.song = TRACKS[track];
    entry.song_time = Math.round(song.currentTime || 0);
    fetch('/feedback', { method: 'POST', body: JSON.stringify(entry) })
      .then((r) => { if (!r.ok) console.warn('feedback not saved (is demo_server.py serving?)', entry); })
      .catch(() => console.warn('feedback not saved', entry));
    console.log('feedback', entry);
  }
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
function thump(start, vol) {
  const t = ac.currentTime + start;
  const len = Math.floor(ac.sampleRate * 0.06);
  const buf = ac.createBuffer(1, len, ac.sampleRate);
  const d = buf.getChannelData(0);
  for (let i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * (1 - i / len);
  const s = ac.createBufferSource();
  const f = ac.createBiquadFilter();
  const g = ac.createGain();
  f.type = 'lowpass';
  f.frequency.value = 700;
  g.gain.value = vol;
  s.buffer = buf;
  s.connect(f).connect(g).connect(ac.destination);
  s.start(t);
}
function sfx(bits) {
  if (!ac || !bits) return;
  if (bits & 1) tone(1200, 0, 0.03, 0.02, 'square');                                   // click
  if (bits & 2) { tone(520, 0, 0.1, 0.04); tone(780, 0.06, 0.12, 0.035); }             // reward posted
  if (bits & 4) { tone(660, 0, 0.08, 0.035); tone(990, 0.07, 0.1, 0.035); }            // hero takes it
  if (bits & 8) tone(240, 0, 0.12, 0.03, 'triangle', 200);                             // hero declines
  if (bits & 16) thump(0, 0.05);                                                      // a blow lands
  if (bits & 32) [880, 1175, 1568].forEach((f, i) => tone(f, i * 0.06, 0.16, 0.03));   // paid
  if (bits & 64) { thump(0, 0.04); tone(392, 0.04, 0.18, 0.04); }                      // hired
  if (bits & 128) { tone(523, 0, 0.3, 0.035, 'sine'); tone(523, 0.35, 0.3, 0.03, 'sine'); } // raid bell
  if (bits & 256) { thump(0, 0.08); tone(196, 0, 0.4, 0.04, 'triangle', 120); }         // something falls
  if (bits & 512) [523, 659, 784].forEach((f, i) => tone(f, i * 0.08, 0.2, 0.03));     // built
  if (bits & 1024) tone(220, 0, 0.6, 0.04, 'triangle', 147);                           // a hero falls
  if (bits & 2048) [523, 659, 784, 1047].forEach((f, i) => tone(f, i * 0.12, 0.4, 0.04)); // won
  if (bits & 4096) tone(110, 0, 0.8, 0.05, 'sawtooth', 82);                            // the curse
  if (bits & 8192) [784, 988].forEach((f, i) => tone(f, i * 0.07, 0.15, 0.03));        // level up
  if (bits & 16384) tone(1400, 0, 0.06, 0.012, 'sine', 900);                           // toad bolt
  if (bits & 32768) tone(150, 0, 0.1, 0.03, 'square');                                 // can't
}

// --- the 15 FPS loop -----------------------------------------------------------------------
layout();
let last = performance.now();
let acc = 0;
function loop(now) {
  acc = Math.min(acc + (now - last), 250);
  last = now;
  while (acc >= TICK_MS) {
    acc -= TICK_MS;
    w.hd_frame();
    sfx(w.hd_take_sfx());
    sendFeedback();
    syncMusic();
    draw();
  }
  requestAnimationFrame(loop);
}
requestAnimationFrame(loop);
