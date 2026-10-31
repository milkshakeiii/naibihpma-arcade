//! First-minute objectives shown in the HUD. `game.rs` decides when each one is met; most
//! checks look at world state rather than single events, so doing things early still counts.

use serde::{Deserialize, Serialize};

pub const STAGES: [&str; 14] = [
    "WALK WITH THE ARROW KEYS OR WASD. WALK INTO THINGS TO PUSH THEM.",
    "STAND ON A RIPE COTTON PLANT, FACE THE BIN, AND PRESS SPACE TO HARVEST.",
    "PUSH A COTTON BOLL INTO THE COTTON BIN.",
    "THE BIN'S CHUTE FEEDS THE WORKBENCH: PUSH A BOLL ONTO ITS PAD, STAND AT THE BENCH, PRESS SPACE.",
    "PUSH THE SLIVERS EAST INTO THE SPINNERY AND SPIN THEM INTO THREAD.",
    "PUSH THE THREAD TO THE LOOM HOUSE IN THE SOUTHEAST AND WEAVE FABRIC.",
    "PUSH THE FABRIC INTO THE CLOTH STORE BELOW THE LOOM HOUSE TO EARN RENOWN.",
    "TEACH A MONSTER PART OF THE JOB: R, DO IT, BACK TO THE FLAG, R.",
    "PRESS P TO LOOP IT. THEN TAB TO THE OTHER MONSTER AND TEACH THE NEXT PART.",
    "SAVE UP RENOWN, THEN WELCOME A HELPER AT THE GATHERING HALL IN THE SOUTHWEST.",
    "THE FERRY EAST NEEDS ROPE: PUSH TWO THREAD ONTO THE BENCH BY THE EAST GATE AND TWIST THEM.",
    "PUSH THREE ROPES INTO THE FERRY'S BOARD, OUT THE EAST GATE BY THE RIVER.",
    "THE FERRY RUNS! RIDE IT ACROSS AND FOLLOW THE ROAD EAST TO TANGLEMERE.",
    "TANGLEMERE'S CLOTH HALL TAKES FABRIC TOO. F1 SHOWS CONTROLS.",
];

pub const DONE: usize = STAGES.len() - 1;

pub const HELP: [&str; 11] = [
    "ARROWS / WASD   WALK, PUSH",
    "SHIFT + ARROW   PULL (SLOW)",
    "SPACE           USE: HARVEST, CRAFT",
    "E (HOLD)        EXAMINE (TAB: MORE)",
    "R               RECORD / SAVE ROUTINE",
    "BACKSPACE       CANCEL RECORDING",
    "P               LOOP ROUTINE / STOP",
    "TAB             SWITCH MONSTER (HOLD: ROSTER)",
    "F1              THIS HELP",
    "F2              CRT SCANLINES",
    "ESC             QUIT",
];

#[derive(Default, Serialize, Deserialize)]
pub struct Tutorial {
    pub stage: usize,
    pub steps: u32,
}

/// Splits text into lines of at most `max` characters on word boundaries.
pub fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = lines.last_mut().expect("at least one line");
        if !cur.is_empty() && cur.len() + 1 + word.len() > max {
            lines.push(word.to_string());
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_objective_fits_two_hud_lines() {
        for s in STAGES {
            assert!(wrap(s, 64).len() <= 2, "too long for the HUD: {s}");
        }
    }
}
