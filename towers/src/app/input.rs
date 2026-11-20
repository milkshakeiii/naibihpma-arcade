#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Key {
    Num1 = 0,
    Num2 = 1,
    Num3 = 2,
    Num4 = 3,
    Num5 = 4,
    Num6 = 5,
    Num7 = 6,
    Q = 7,
    E = 8,
    U = 9,
    X = 10,
    M = 11,
    T = 12,
    P = 13,
    D = 14,
    V = 15,
    H = 16,
    F = 17,
    Space = 18,
    Tab = 19,
    Enter = 20,
    Esc = 21,
    Up = 22,
    Down = 23,
    Left = 24,
    Right = 25,
    F2 = 26,
    F3 = 27,
    F11 = 28,
    R = 29,
}

impl Key {
    pub const COUNT: usize = 30;
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub left_down: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub down: [bool; Key::COUNT],
    pub pressed: [bool; Key::COUNT],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_x: -1,
            mouse_y: -1,
            left_down: false,
            left_pressed: false,
            right_pressed: false,
            down: [false; Key::COUNT],
            pressed: [false; Key::COUNT],
        }
    }
}

impl InputState {
    pub fn set_key(&mut self, key: Key, is_down: bool) {
        let idx = key as usize;
        if is_down && !self.down[idx] {
            self.pressed[idx] = true;
        }
        self.down[idx] = is_down;
    }

    pub fn set_mouse_pos(&mut self, lx: i32, ly: i32) {
        self.mouse_x = lx;
        self.mouse_y = ly;
    }

    pub fn set_left_button(&mut self, is_down: bool) {
        if is_down && !self.left_down {
            self.left_pressed = true;
        }
        self.left_down = is_down;
    }

    pub fn latch_right_click(&mut self) {
        self.right_pressed = true;
    }

    #[inline]
    pub fn just_pressed(&self, key: Key) -> bool {
        self.pressed[key as usize]
    }

    pub fn end_tick(&mut self) {
        self.pressed.fill(false);
        self.left_pressed = false;
        self.right_pressed = false;
    }
}
