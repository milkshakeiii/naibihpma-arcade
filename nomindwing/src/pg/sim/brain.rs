//! How a side's ships choose how to fly: as their side's planner decides (ARCH, `arch.rs`), or
//! as their drawings suggest (PLAIN). The arena's brain buttons switch between the two.
//!
//! (A third brain, DRILLED, flew each ship at each enemy the way that had done best in drills run
//! before the fight, every design against every enemy design. It made nobody better, was never in
//! a match, and was taken out on 2026-11-14 with its drills, at Henry's word.)

/// How a side's ships fly at what they're fighting.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Ways {
    /// The way its drawing suggests, whatever the enemy.
    #[default]
    Drawn,
    /// The way its side's planner has decided for its design.
    Plan,
}

/// How a side's ships choose how to fly.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct Brain {
    pub ways: Ways,
}

/// The brains the arena's buttons step through, the one every side starts on first.
pub const BRAINS: [Brain; 2] = [Brain::ARCH, Brain::PLAIN];

impl Brain {
    /// No planning: each ship goes for the nearest enemy and flies at it as its drawing
    /// suggests. (This was CARD while a card carried a way of flying.)
    pub const PLAIN: Brain = Brain { ways: Ways::Drawn };
    /// The side's planner decides how each of its designs flies (`arch.rs`).
    pub const ARCH: Brain = Brain { ways: Ways::Plan };

    /// For a button: at most eight letters.
    pub fn short(self) -> &'static str {
        match self.ways {
            Ways::Drawn => "PLAIN",
            Ways::Plan => "ARCH",
        }
    }

    pub fn name(self) -> String {
        self.short().into()
    }
}
