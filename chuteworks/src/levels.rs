//! The prototype's two halls, embedded at build time.

use crate::model::Level;

const SOURCES: [(&str, &str); 17] = [
    ("1-1", include_str!("../levels/1-1_first_coat.lvl")),
    ("1-2", include_str!("../levels/1-2_two_coats.lvl")),
    ("1-3", include_str!("../levels/1-3_slide.lvl")),
    ("1-4", include_str!("../levels/1-4_two_chutes.lvl")),
    ("1-5", include_str!("../levels/1-5_through_the_booth.lvl")),
    ("1-6", include_str!("../levels/1-6_mud.lvl")),
    ("1-7", include_str!("../levels/1-7_floor_paint.lvl")),
    ("1-8", include_str!("../levels/1-8_backwards.lvl")),
    ("2-1", include_str!("../levels/2-1_switch.lvl")),
    ("2-2", include_str!("../levels/2-2_split_before_you_paint.lvl")),
    ("2-3", include_str!("../levels/2-3_flip_flop.lvl")),
    ("2-4", include_str!("../levels/2-4_turn_taking.lvl")),
    ("2-5", include_str!("../levels/2-5_zipper.lvl")),
    ("2-6", include_str!("../levels/2-6_wrong_foot.lvl")),
    ("2-7", include_str!("../levels/2-7_swap_pairs.lvl")),
    ("2-8", include_str!("../levels/2-8_other_board.lvl")),
    ("2-9", include_str!("../levels/2-9_blue_goes_first.lvl")),
];

pub const HALL_NAMES: [&str; 2] = ["THE PAINT SHOP", "THE SWITCHYARD"];

pub fn all() -> Result<Vec<Level>, String> {
    SOURCES.iter().map(|(id, src)| Level::parse(id, src)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use crate::sim::{run, Fail, Jam, Outcome};

    #[test]
    fn every_stored_solution_solves_and_an_empty_board_does_not() {
        for lv in all().unwrap() {
            let layout = Layout::from_solution(&lv);
            let (out, chutes) = run(&lv, &layout);
            assert_eq!(out, Outcome::Won, "{} {}: {out:?}, chutes {chutes:?}", lv.id, lv.name);
            assert_ne!(run(&lv, &Layout::default()).0, Outcome::Won, "{} solves itself", lv.id);
        }
    }

    #[test]
    fn solutions_only_use_the_tray_on_free_tiles_of_allowed_boards() {
        for lv in all().unwrap() {
            let layout = Layout::from_solution(&lv);
            for p in &lv.solution {
                let tool = Tool::of(p.cell).unwrap();
                assert!(tool.allowed(p.board), "{}: {tool:?} on {:?}", lv.id, p.board);
                assert_eq!(lv.fixed(p.board, p.x, p.y), Cell::Floor, "{}: placed on a fixed tile at {},{}", lv.id, p.x, p.y);
                if let Some(n) = lv.tray_count(tool) {
                    assert!(layout.placed(tool) <= n, "{}: too many {tool:?}", lv.id);
                } else {
                    assert_eq!(tool, Tool::Belt, "{}: {tool:?} isn't in the tray", lv.id);
                }
            }
            // Exact trays: every counted piece is used.
            for s in &lv.tray {
                if let Some(n) = s.count {
                    assert_eq!(layout.placed(s.tool), n, "{}: tray {:?} not fully used", lv.id, s.tool);
                }
            }
        }
    }

    fn get(id: &str) -> Level {
        all().unwrap().into_iter().find(|l| l.id == id).unwrap()
    }

    #[test]
    fn through_the_booth_straight_lines_send_grey_to_the_orange_bin() {
        let lv = get("1-5");
        let mut l = Layout::default();
        for x in 1..8 {
            l.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
            l.set(Board::Left, x, 6, Some(Cell::Belt(Dir::E)));
        }
        assert!(matches!(run(&lv, &l).0, Outcome::Failed(Fail::WrongBin, 7, 1)));
    }

    #[test]
    fn mud_crossing_in_the_red_booth_ruins_the_yellow_line() {
        let lv = get("1-6");
        // Blue runs straight to chute 1; yellow takes red, then blue: three coats.
        let mut l2 = Layout::default();
        for x in 1..8 {
            l2.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
        }
        for x in 1..5 {
            l2.set(Board::Left, x, 6, Some(Cell::Belt(Dir::E)));
        }
        l2.set(Board::Left, 6, 6, Some(Cell::Belt(Dir::N)));
        l2.set(Board::Left, 6, 5, Some(Cell::Belt(Dir::N)));
        l2.set(Board::Left, 6, 4, Some(Cell::Belt(Dir::W)));
        l2.set(Board::Left, 5, 4, Some(Cell::Belt(Dir::W)));
        l2.set(Board::Left, 4, 4, Some(Cell::Belt(Dir::W)));
        assert!(matches!(run(&lv, &l2).0, Outcome::Jam(Jam::Mud, 3, 4)));
    }

    #[test]
    fn wrong_foot_one_color_per_chute_sends_blue_up_first() {
        let lv = get("2-6");
        let mut l = Layout::default();
        for x in 1..8 {
            l.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
            l.set(Board::Left, x, 6, Some(Cell::Belt(Dir::E)));
        }
        l.set(Board::Right, 2, 1, Some(Cell::Arrow(Dir::S)));
        l.set(Board::Right, 2, 3, Some(Cell::Arrow(Dir::E)));
        assert!(matches!(run(&lv, &l).0, Outcome::Failed(Fail::WrongBin, 5, 0)));
        // Starting the zipper on the blue side has the same problem.
        let mut sol = Layout::from_solution(&lv);
        sol.set(Board::Left, 4, 3, Some(Cell::Zipper(Dir::E, Dir::N)));
        assert!(matches!(run(&lv, &sol).0, Outcome::Failed(Fail::WrongBin, 5, 0)));
    }

    #[test]
    fn other_board_splitting_on_the_belts_has_nowhere_to_go() {
        let lv = get("2-8");
        let mut l = Layout::default();
        l.set(Board::Left, 1, 3, Some(Cell::Belt(Dir::E)));
        l.set(Board::Left, 2, 3, Some(Cell::Switch(Dir::N)));
        for (x, y, d) in [(2, 2, Dir::E), (3, 2, Dir::E), (4, 2, Dir::E), (5, 2, Dir::E), (6, 2, Dir::S), (6, 3, Dir::E), (7, 3, Dir::E)] {
            l.set(Board::Left, x, y, Some(Cell::Belt(d)));
        }
        for (x, y, d) in [(2, 4, Dir::E), (3, 4, Dir::E), (4, 4, Dir::E), (5, 4, Dir::E), (6, 4, Dir::N)] {
            l.set(Board::Left, x, y, Some(Cell::Belt(d)));
        }
        assert!(matches!(run(&lv, &l).0, Outcome::Jam(Jam::Merge, 6, 3)));
    }
}
