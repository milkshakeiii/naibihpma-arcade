//! The browser's way in: plain exported functions over one `Game`, with no JavaScript glue
//! library. Pixels and text are handed over as pointers into WebAssembly memory.

use hold::game::{Game, In, Screen};
use std::cell::UnsafeCell;

struct Slot(UnsafeCell<Option<Game>>, UnsafeCell<(i32, i32)>);

// SAFETY: this WebAssembly module is single-threaded and JavaScript calls one export at a time.
unsafe impl Sync for Slot {}

static SLOT: Slot = Slot(UnsafeCell::new(None), UnsafeCell::new((0, 0)));

fn game() -> &'static mut Game {
    // SAFETY: see `Slot`; no export holds this reference across calls.
    unsafe { (*SLOT.0.get()).get_or_insert_with(|| Game::new(1)) }
}

/// Start over with this seed.
#[no_mangle]
pub extern "C" fn hd_init(seed: u32) {
    // SAFETY: see `Slot`.
    unsafe { *SLOT.0.get() = Some(Game::new(seed as u64)) }
}

/// The pointer moved to logical framebuffer pixel (x, y).
#[no_mangle]
pub extern "C" fn hd_move(x: i32, y: i32) {
    game().push(In::Move(x, y));
}

/// Mouse button `b` (0 left, 2 right) went down or up at logical pixel (x, y).
#[no_mangle]
pub extern "C" fn hd_button(b: i32, down: i32, x: i32, y: i32) {
    let b = b.clamp(0, 2) as u8;
    game().push(if down != 0 {
        In::Down(b, x, y)
    } else {
        In::Up(b, x, y)
    });
}

/// A special key went down: 1 Tab, 2 Enter, 3 Esc, 4 Backspace, 5-8 arrows, 9 F2.
#[no_mangle]
pub extern "C" fn hd_key(code: i32, shift: i32) {
    game().push(In::Key(code as u8, shift != 0));
}

/// A printable character was typed.
#[no_mangle]
pub extern "C" fn hd_char(code: u32) {
    if let Some(c) = char::from_u32(code) {
        game().push(In::Char(c));
    }
}

/// One 15 FPS frame.
#[no_mangle]
pub extern "C" fn hd_frame() {
    game().frame();
}

/// The frame as RGBA at exact integer scale `n`: (400 n) x (300 n) pixels.
#[no_mangle]
pub extern "C" fn hd_present(n: i32) -> *const u8 {
    game().present(n.max(1) as usize).as_ptr()
}

/// The pointer as RGBA at scale `n`; its size is then `hd_cursor_w()` x `hd_cursor_h()`.
#[no_mangle]
pub extern "C" fn hd_cursor(n: i32) -> *const u8 {
    let (px, w, h) = game().cursor(n.max(1) as usize);
    // SAFETY: see `Slot`.
    unsafe { *SLOT.1.get() = (w as i32, h as i32) };
    px.as_ptr()
}

#[no_mangle]
pub extern "C" fn hd_cursor_w() -> i32 {
    // SAFETY: see `Slot`.
    unsafe { (*SLOT.1.get()).0 }
}

#[no_mangle]
pub extern "C" fn hd_cursor_h() -> i32 {
    // SAFETY: see `Slot`.
    unsafe { (*SLOT.1.get()).1 }
}

/// Pause the game (the page was hidden).
#[no_mangle]
pub extern "C" fn hd_pause() {
    game().pause();
}

/// 1 while the soundtrack should play.
#[no_mangle]
pub extern "C" fn hd_wants_music() -> i32 {
    game().wants_music() as i32
}

/// 1 while the game wants typed text, so the page should hand over every key.
#[no_mangle]
pub extern "C" fn hd_typing() -> i32 {
    game().typing() as i32
}

/// Sound effects since the last call, as bits (`game::sfx`).
#[no_mangle]
pub extern "C" fn hd_take_sfx() -> u32 {
    std::mem::take(&mut game().sfx)
}

/// How many feedback entries are waiting to be sent.
#[no_mangle]
pub extern "C" fn hd_outbox_len() -> i32 {
    game().outbox.len() as i32
}

/// The first waiting feedback entry (UTF-8 JSON): pointer, then its length in bytes.
#[no_mangle]
pub extern "C" fn hd_outbox_ptr() -> *const u8 {
    game()
        .outbox
        .first()
        .map_or(std::ptr::null(), |s| s.as_ptr())
}

#[no_mangle]
pub extern "C" fn hd_outbox_size() -> i32 {
    game().outbox.first().map_or(0, |s| s.len() as i32)
}

/// Drop the first waiting feedback entry (it has been sent).
#[no_mangle]
pub extern "C" fn hd_outbox_pop() {
    let g = game();
    if !g.outbox.is_empty() {
        g.outbox.remove(0);
    }
}

/// 0 title, 1 playing, 2 the end card.
#[no_mangle]
pub extern "C" fn hd_screen() -> i32 {
    match game().screen {
        Screen::Title => 0,
        Screen::Play => 1,
        Screen::End => 2,
    }
}
