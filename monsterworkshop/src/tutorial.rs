//! First-minute objectives shown in the HUD. `game.rs` decides when each one is met; most
//! checks look at world state rather than single events, so doing things early still counts.

pub const STAGES: [&str; 11] = [
    "WALK WITH THE ARROW KEYS OR WASD. WALK INTO THINGS TO PUSH THEM.",
    "PUSH COTTON FROM THE BIN'S CHUTE INTO THE SPINNERY, ONTO BOTH PADS.",
    "STAND ON THE SPINNING WHEEL AND PRESS SPACE TO SPIN THREAD.",
    "PUSH THE THREAD OUT THE EAST DOOR AND INTO THE THREAD BIN.",
    "NOW TEACH MOGG THE JOB. PRESS R TO RECORD. A FLAG MARKS WHERE YOU STARTED.",
    "DO THE WHOLE JOB AGAIN, STAND ON THE FLAG, PRESS R TO SAVE. BACKSPACE CANCELS.",
    "PRESS P TO SET MOGG LOOPING THE ROUTINE.",
    "PRESS TAB TO TAKE CONTROL OF BRUNA. MOGG KEEPS WORKING.",
    "COTTON RAN OUT. STAND ON A RIPE PLANT FACING THE BIN, PRESS SPACE, PUSH THE BOLL IN.",
    "TEACH BRUNA TOO: R, GATHER AND DELIVER, BACK TO THE FLAG, R, THEN P.",
    "THE YARD RUNS ITSELF. RE-RECORD A LOOP TO MAKE IT FASTER. F1 SHOWS CONTROLS.",
];

pub const DONE: usize = STAGES.len() - 1;

pub const HELP: [&str; 9] = [
    "ARROWS / WASD   WALK, PUSH",
    "SPACE / E       USE: HARVEST, SPIN",
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
