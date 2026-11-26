//! A text picture of the board, for watching what a bot did.

use crate::geom::*;
use crate::map::SHAPES;
use crate::sim::Sim;

const RAIL: [char; 16] = [
    ' ', '╵', '╶', '└', '╷', '│', '┌', '├', '╴', '┘', '─', '┴', '┐', '┤', '┬', '┼',
];
const CARS: [char; 8] = ['○', '◇', '□', '△', '☆', '+', '♡', '♧'];

pub fn render(sim: &Sim) -> String {
    let mut s = String::new();
    let secs = sim.tick / 15;
    s += &format!(
        "{}:{:02}  score {}  x{}  aboard {}  pieces {}/{}  {}{}{}\n",
        secs / 60,
        secs % 60,
        sim.stats.score,
        sim.mult(),
        sim.cars
            .iter()
            .map(|&c| CARS[c as usize])
            .collect::<String>(),
        sim.free(),
        sim.p.stock,
        if sim.express { "EXPRESS " } else { "" },
        if sim.stalled { "STALLED " } else { "" },
        if sim.terminus_open() {
            "TERMINUS OPEN"
        } else {
            ""
        },
    );
    for y in 0..GH {
        for x in 0..GW {
            let p = Pos::new(x, y);
            let i = p.idx();
            let c = if let Some(k) = sim.body.iter().position(|&b| b == p) {
                if k == 0 {
                    '@'
                } else {
                    sim.cars.get(k - 1).map_or('·', |&c| CARS[c as usize])
                }
            } else if let Some(si) = sim.station(p) {
                SHAPES[sim.stations[si].shape as usize]
            } else if p == sim.terminus {
                'T'
            } else if sim.blocked[i] {
                '▒'
            } else if sim.mask[i] != 0 {
                RAIL[sim.mask[i] as usize]
            } else if sim.station_at[i].is_some() {
                '+'
            } else {
                '.'
            };
            s.push(c);
        }
        s.push('\n');
    }
    for st in &sim.stations {
        if sim.tick >= st.appear {
            s += &format!(
                "  {} ({},{}) waiting {}\n",
                SHAPES[st.shape as usize],
                st.pos.x,
                st.pos.y,
                st.queue
                    .iter()
                    .map(|&c| SHAPES[c as usize])
                    .collect::<String>()
            );
        }
    }
    s
}
