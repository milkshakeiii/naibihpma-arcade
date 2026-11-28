//! A scripted Elder for balance checks: plays the lesson the way it's taught, then puts one
//! reward at a time on the weakest nest, raises it while too few heroes take it, and hires
//! whenever there's dew to spare. It reacts every ten seconds, slower than a person.

use crate::sim::*;

/// A plain player: the lesson first, then rewards on every nest, raised while nobody takes them,
/// and hiring whenever there's dew to spare.
pub fn elder(w: &mut World, t: u32) {
    let nests: Vec<u32> = w
        .bldgs
        .iter()
        .filter(|b| b.alive && b.kind == BKind::Nest)
        .map(|b| b.id)
        .collect();
    let guilds: Vec<u32> = w
        .bldgs
        .iter()
        .filter(|b| b.alive && matches!(b.kind, BKind::Lodge | BKind::ToadHall))
        .map(|b| b.id)
        .collect();
    if w.calm {
        // The lesson: the weak nest, then a toad, then the next nest until the toad goes.
        let weak = w
            .bldgs
            .iter()
            .find(|b| b.weak)
            .map(|b| (b.id, b.alive))
            .unwrap();
        if weak.1 {
            w.post(Tgt::Bldg(weak.0));
            return;
        }
        if !w.units.iter().any(|u| u.kind == Kind::Toad) {
            w.hire(guilds[1]);
            return;
        }
        match w.flag_on(Tgt::Bldg(nests[0])).map(|f| f.id) {
            None => {
                w.post(Tgt::Bldg(nests[0]));
            }
            Some(f) => {
                let toad = w.units.iter().find(|u| u.kind == Kind::Toad).unwrap().id;
                if w.flag(f).unwrap().takers.contains(&toad) {
                    w.end_calm();
                } else if t % 15 == 0 {
                    w.raise(f);
                }
            }
        }
        return;
    }
    if t % 150 != 0 {
        return;
    }
    for &g in &guilds {
        let free = w.dew - w.reserved();
        if free >= 45 {
            w.hire(g);
        }
    }
    // Focus: one reward at a time, on the weakest nest or Gorf's stone; raise it while too few
    // take it.
    let focus = w
        .bldgs
        .iter()
        .filter(|b| b.alive && b.kind.enemy())
        .min_by_key(|b| b.hp)
        .map(|b| b.id);
    if let Some(n) = focus {
        match w.flag_on(Tgt::Bldg(n)).map(|f| f.id) {
            None => {
                w.post(Tgt::Bldg(n));
            }
            Some(f) => {
                if w.flag(f).unwrap().takers.len() < 2 && w.flag(f).unwrap().reward < 120 {
                    w.raise(f);
                }
            }
        }
    }
}
