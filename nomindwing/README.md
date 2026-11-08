# No Mind Wing: Toy 1

A DOS-style arcade RTS: arcade on top, RTS underneath. You fly your king (mothership) yourself. It is the root of a tree of factories, and you spread that tree across the map with seeds. Everything is made of pixels: rocks, ships and the money that prints them.

**The fun thesis (Henry's words):** the mothership is the king and the root node of a tree of factories. You spawn off little self-propagating wings strategically, like a resource-collection dispatch or a forward production base. It's "the creativity of designing a TCG deck combined with the strategic thrill of outmaneuvering someone in chess."

**What this toy tests:** is planting a forward base, and watching it eat a rock and start printing, as awesome as it is in your head? And does the AI king push back hard enough to make where you plant a real decision? There is no shipyard yet (the five cards are hand-made), no campaign, no music and no tutorial beyond the rules screen.

## Run

```
cargo run --release
cargo run --release -- --windowed
```

## Controls

| Input | Does |
|---|---|
| WASD / arrows | Fly your king. If it dies, you lose. |
| 1–5 or click a card | Print it. Seed cards then need a rock: click one on the map, minimap or tactical view. |
| Right-click / Esc | Cancel planting |
| Tab / TAC | Tactical view: the whole map at exactly 1/4 scale |
| Backspace / UNDO | Take the last card off the print queue (printed pixels come back) |
| P / PAUSE, F1 / HELP, F2 | Pause, rules, scanlines |

## Rules in one breath

- Pixels are money. Every design costs its pixel count, paid one pixel per frame as it prints.
- Collectors chew pixels off rocks and wrecks and haul them to the nearest factory.
- A seed flies to a rock and eats it into a base, pixel by pixel.
- A base keeps its pinned build order alive, rebuilding whatever dies:
  - **Outpost:** 2 collectors, 1 escort, and 1 outpost seed. The seed means outposts spread.
  - **Forward base:** 2 escorts and 3 raiders.
- Raiders wait at their factory until there are 3, then the pack hunts.
- Every colour is a part: yellow shoots, blue-white pushes, orange scoops, grey takes two hits, and a ship dies when its white core is gone.
- Shots knock pixels loose for anyone to collect.

## Tools

- `--sim N`: plays N AI-vs-AI matches and N AI-vs-idle-king matches headless, then prints the stats.
- `--trace SEED [--both]`: prints one match's state every 20 seconds.
- `--export-screenshots DIR`: 2x PNGs of the main screens.

Playtests append to `nomindwing_playtest.log` (gitignored): every card played, every base planted or lost, and a snapshot every 30 seconds.

## Tuning notes (2026-11-07)

- **Map:** 960x720. Every rock has a mirror twin through the centre. An earlier centre rock sat 0.35 px closer to one king, and that king won 5 of 6 AI matches.
- **Aiming:** gunners aim at a random surviving pixel. Aiming at the centre drilled a straight tunnel through a king that later shots flew through without touching anything.
- **Pacing:** the AI builds economy first, plants its forward base at about 1:50, prints no raiders before 1:40, and doesn't send its king before 3:20. An idle king falls at about 2:10. AI-vs-AI matches end in 3½–6½ minutes, roughly 50/50.
