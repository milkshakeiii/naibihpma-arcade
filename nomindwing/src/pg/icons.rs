//! 7x7 one-colour icons for the yard: part categories, tools, gauges, flight styles, and the
//! small shape each part shows inside its colour so parts stay easy to tell apart as more arrive.

use super::design::{Cat, Flight, Part};
use crate::framebuffer::Framebuffer;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Cat(Cat),
    Part(Part),
    Pencil,
    Rect,
    Fill,
    Copy,
    Turn,
    Undo,
    Mirror,
    Clear,
    Speed,
    TurnRate,
    Thrust,
    Flight(Flight),
}

pub fn art(i: Icon) -> [&'static str; 7] {
    match i {
        Icon::Cat(Cat::Armor) => [".#####.", "#######", "#######", "#######", ".#####.", "..###..", "...#..."],
        Icon::Cat(Cat::Weapons) => ["...#...", ".#####.", ".#...#.", "##.#.##", ".#...#.", ".#####.", "...#..."],
        Icon::Cat(Cat::Drive) => ["...#...", "..###..", ".#####.", "..###..", "..###..", ".#.#.#.", "#..#..#"],
        Icon::Cat(Cat::Power) => ["....##.", "...##..", "..##...", ".#####.", "...##..", "..##...", ".##...."],
        Icon::Cat(Cat::Industry) => ["..#.#..", ".#####.", "##...##", ".#...#.", "##...##", ".#####.", "..#.#.."],
        Icon::Cat(Cat::Command) => [".......", "#..#..#", "##.#.##", "#######", "#######", ".#####.", "......."],
        Icon::Part(Part::Hull) => [".......", ".......", ".......", ".......", ".......", ".......", "......."],
        Icon::Part(Part::Plate) => [".......", ".#...#.", ".......", ".......", ".......", ".#...#.", "......."],
        Icon::Part(Part::Flak) => [".......", "...#...", "...#...", "..###..", "..###..", ".......", "......."],
        Icon::Part(Part::Lance) => ["...#...", "..###..", ".#.#.#.", "...#...", "...#...", "...#...", "......."],
        Icon::Part(Part::Cannon) => [".......", "..###..", ".#...#.", ".#...#.", ".#...#.", "..###..", "......."],
        Icon::Part(Part::Engine) => ["#.....#", ".#...#.", "..#.#..", "#..#..#", ".#...#.", "..#.#..", "...#..."],
        Icon::Part(Part::Scoop) => [".......", ".#...#.", ".#...#.", ".#...#.", "..###..", ".......", "......."],
        Icon::Part(Part::Core) => ["...#...", "..#.#..", ".#...#.", "#.....#", ".#...#.", "..#.#..", "...#..."],
        Icon::Pencil => [".....##", "....#.#", "...#.#.", "..#.#..", ".#.#...", "##.....", "#......"],
        Icon::Rect => ["#######", "#.#.#.#", "##.#.##", "#.#.#.#", "##.#.##", "#.#.#.#", "#######"],
        Icon::Fill => ["..#....", ".#.#...", "#...#..", "#....#.", ".#..#.#", "..##..#", "......#"],
        Icon::Copy => ["##.#.##", "#.....#", ".......", "#.....#", ".......", "#.....#", "##.#.##"],
        Icon::Turn => ["..###..", ".#...#.", "#.....#", "#.....#", "#...#.#", ".#...##", "...####"],
        Icon::Undo => ["..#....", ".##....", "######.", ".##...#", "..#...#", "......#", "..####."],
        Icon::Mirror => ["...#...", "#..#..#", "##.#.##", "###.###", "##.#.##", "#..#..#", "...#..."],
        Icon::Clear => ["#.....#", ".#...#.", "..#.#..", "...#...", "..#.#..", ".#...#.", "#.....#"],
        Icon::Speed => [".......", "#..#...", ".#..#..", "..#..#.", ".#..#..", "#..#...", "......."],
        Icon::TurnRate => ["..###..", ".#...#.", "#.....#", "#...###", "#....#.", ".#.....", "..##..."],
        Icon::Thrust => ["...#...", "..##...", "..###..", ".##.##.", ".#...#.", ".##.##.", "..###.."],
        Icon::Flight(Flight::Runs) => ["#......", ".#.....", "..#....", "...#..#", "....#.#", ".....##", "..#####"],
        Icon::Flight(Flight::StandOff) => ["...#...", "...#...", ".......", "##...##", ".......", "...#...", "...#..."],
        Icon::Flight(Flight::Still) => ["...#...", "..#.#..", "...#...", "...#...", "#..#..#", ".#.#.#.", "..###.."],
    }
}

/// Draws an icon with its top-left at (x, y).
pub fn draw(fb: &mut Framebuffer, i: Icon, x: i32, y: i32, col: u8) {
    for (dy, row) in art(i).iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            if ch == '#' {
                fb.pset(x + dx as i32, y + dy as i32, col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg::design::{CATS, PARTS};

    #[test]
    fn every_icon_is_seven_by_seven() {
        let mut all: Vec<Icon> = CATS.iter().map(|&c| Icon::Cat(c)).chain(PARTS.iter().map(|&p| Icon::Part(p))).collect();
        all.extend([Icon::Pencil, Icon::Rect, Icon::Fill, Icon::Copy, Icon::Turn, Icon::Undo, Icon::Mirror, Icon::Clear, Icon::Speed, Icon::TurnRate, Icon::Thrust]);
        all.extend([Flight::Runs, Flight::StandOff, Flight::Still].map(Icon::Flight));
        for i in all {
            assert!(art(i).iter().all(|r| r.chars().count() == 7), "{:?}", i);
        }
    }
}
