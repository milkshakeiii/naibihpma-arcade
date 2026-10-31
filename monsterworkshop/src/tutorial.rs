//! First-minute objectives shown in the HUD. `game.rs` decides when each one is met; most
//! checks look at world state rather than single events, so doing things early still counts.

pub const STAGES: [&str; 10] = [
    "WALK WITH THE ARROW KEYS OR WASD. WALK INTO THINGS TO PUSH THEM.",
    "STAND ON A RIPE COTTON PLANT, FACE THE BIN, AND PRESS SPACE TO HARVEST.",
    "PUSH A COTTON BOLL INTO THE COTTON BIN.",
    "THE BIN'S CHUTE FEEDS THE WORKBENCH: PUSH A BOLL ONTO ITS PAD, STAND AT THE BENCH, PRESS SPACE.",
    "PUSH THE SLIVERS EAST INTO THE SPINNERY AND SPIN THEM INTO THREAD.",
    "PUSH THE THREAD TO THE LOOM HOUSE IN THE SOUTHEAST AND WEAVE FABRIC.",
    "PUSH THE FABRIC INTO THE CLOTH STORE BELOW THE LOOM HOUSE.",
    "TEACH A MONSTER PART OF THE JOB: R, DO IT, BACK TO THE FLAG, R.",
    "PRESS P TO LOOP IT. THEN TAB TO THE OTHER MONSTER AND TEACH THE NEXT PART.",
    "PINEWATER WEAVES ON ITS OWN. F1 SHOWS CONTROLS.",
];

pub const DONE: usize = STAGES.len() - 1;

pub const HELP: [&str; 9] = [
    "ARROWS / WASD   WALK, PUSH",
    "SPACE / E       USE: HARVEST, CRAFT",
    "R               RECORD / SAVE ROUTINE",
    "BACKSPACE       CANCEL RECORDING",
    "P               LOOP ROUTINE / STOP",
    "TAB             SWITCH MONSTER",
    "F1              THIS HELP",
    "F2              CRT SCANLINES",
    "ESC             QUIT",
];

#[derive(Default)]
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
