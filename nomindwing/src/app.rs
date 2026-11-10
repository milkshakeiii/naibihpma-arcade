//! What the window layer needs from a toy: events in, 15 FPS frames out.

use crate::framebuffer::Framebuffer;
use crate::palette::Pal;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Num(u8),
    /// A digit with Shift held.
    ShiftNum(u8),
    Letter(char),
    Tab,
    Space,
    Esc,
    Help,
    Back,
    Enter,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ev {
    Key(Key),
    /// A typed character (keyboard layout and repeat handled by the OS).
    Char(char),
    Move(i32, i32),
    Press { x: i32, y: i32, right: bool },
    Release { x: i32, y: i32, right: bool },
    /// Mouse wheel notches since the last tick: positive is away from the user.
    Wheel(i32),
}

pub trait App {
    fn event(&mut self, ev: Ev);
    /// Movement keys held right now: up, down, left, right.
    fn hold(&mut self, _dirs: [bool; 4]) {}
    fn tick(&mut self);
    fn draw(&self, fb: &mut Framebuffer);
    fn palette(&self) -> &Pal;
    /// 0 = arrow, 1 = crosshair.
    fn cursor_kind(&self) -> u8 {
        0
    }
    fn crt(&self) -> bool;
    fn toggle_crt(&mut self);
    fn quit(&self) -> bool;
}
