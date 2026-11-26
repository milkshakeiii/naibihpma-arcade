//! The browser's way in: plain exported functions over one `Game`, with no JavaScript glue
//! library. Pixels are handed over as pointers into WebAssembly memory; `www/main.js` reads them.

use manyworlds::game::{Game, In, Key, Screen};
use std::cell::UnsafeCell;

struct Slot(UnsafeCell<Option<Game>>, UnsafeCell<(i32, i32)>);

// SAFETY: this WebAssembly module is single-threaded and JavaScript calls one export at a time.
unsafe impl Sync for Slot {}

static SLOT: Slot = Slot(UnsafeCell::new(None), UnsafeCell::new((0, 0)));

fn game() -> &'static mut Game {
    // SAFETY: see `Slot`; no export holds this reference across calls.
    unsafe { (*SLOT.0.get()).get_or_insert_with(|| Game::new(289)) }
}

/// Start over with a leg as long as a song of `seconds`.
#[no_mangle]
pub extern "C" fn mw_init(seconds: i32) {
    // SAFETY: see `Slot`.
    unsafe { *SLOT.0.get() = Some(Game::new(seconds)) }
}

/// The song's real length, once the browser has read it.
#[no_mangle]
pub extern "C" fn mw_set_length(seconds: i32) {
    game().set_length(seconds);
}

/// The pointer moved to logical framebuffer pixel (x, y).
#[no_mangle]
pub extern "C" fn mw_move(x: i32, y: i32) {
    game().push(In::Move(x, y));
}

/// Mouse button `b` (0 left, 2 right) went down or up at logical pixel (x, y).
#[no_mangle]
pub extern "C" fn mw_button(b: i32, down: i32, x: i32, y: i32) {
    let b = b.clamp(0, 2) as u8;
    game().push(if down != 0 {
        In::Down(b, x, y)
    } else {
        In::Up(b, x, y)
    });
}

/// A key went down or up; codes are listed in `Key::from_code`.
#[no_mangle]
pub extern "C" fn mw_key(code: i32, down: i32) {
    if let Some(k) = Key::from_code(code) {
        game().push(In::Key(k, down != 0));
    }
}

/// The song's playback position, in 15 FPS ticks.
#[no_mangle]
pub extern "C" fn mw_song_tick(t: i32) {
    game().set_song_tick(t);
}

/// One 15 FPS frame.
#[no_mangle]
pub extern "C" fn mw_frame() {
    game().frame();
}

/// The frame as RGBA at exact integer scale `n`: (400 n) x (300 n) pixels.
#[no_mangle]
pub extern "C" fn mw_present(n: i32) -> *const u8 {
    game().present(n.max(1) as usize).as_ptr()
}

/// The pointer as RGBA at scale `n`; its size is then `mw_cursor_w()` x `mw_cursor_h()`.
#[no_mangle]
pub extern "C" fn mw_cursor(n: i32) -> *const u8 {
    let (px, w, h) = game().cursor(n.max(1) as usize);
    // SAFETY: see `Slot`.
    unsafe { *SLOT.1.get() = (w as i32, h as i32) };
    px.as_ptr()
}

#[no_mangle]
pub extern "C" fn mw_cursor_w() -> i32 {
    // SAFETY: see `Slot`.
    unsafe { (*SLOT.1.get()).0 }
}

#[no_mangle]
pub extern "C" fn mw_cursor_h() -> i32 {
    // SAFETY: see `Slot`.
    unsafe { (*SLOT.1.get()).1 }
}

/// Pause the leg (the page was hidden).
#[no_mangle]
pub extern "C" fn mw_pause() {
    game().pause();
}

/// 1 while the song should be playing.
#[no_mangle]
pub extern "C" fn mw_wants_audio() -> i32 {
    game().wants_audio() as i32
}

/// Changes whenever the song should start again from the top.
#[no_mangle]
pub extern "C" fn mw_audio_gen() -> i32 {
    game().audio_gen as i32
}

/// Sound effects since the last call, as bits (`game::sfx`); 0 while sound effects are off.
#[no_mangle]
pub extern "C" fn mw_take_sfx() -> i32 {
    let g = game();
    let bits = std::mem::take(&mut g.sfx);
    if g.sound {
        bits as i32
    } else {
        0
    }
}

/// 0 title, 1 playing, 2 the leg is over.
#[no_mangle]
pub extern "C" fn mw_screen() -> i32 {
    match game().screen {
        Screen::Title => 0,
        Screen::Play => 1,
        Screen::End => 2,
    }
}
