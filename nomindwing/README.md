# No Mind Wing

A DOS-style arcade RTS: arcade on top, RTS underneath. You fly your king (mothership) yourself. It is the root of a tree of factories, and you spread that tree across the map with seeds. Everything is made of pixels: rocks, ships and the money that prints them.

**The fun thesis (Henry's words):** the mothership is the king and the root node of a tree of factories. You spawn off little self-propagating wings strategically, like a resource-collection dispatch or a forward production base. It's "the creativity of designing a TCG deck combined with the strategic thrill of outmaneuvering someone in chess."

There are two toys so far:

- **Proving ground** (the default): draw ships, set them loose, and see whether they fly and fight the way their drawings suggest. Iterated in rounds, logged in [ROUNDS.md](ROUNDS.md).
- **Toy 1** (`--toy1`): the first playable match, testing the tree of factories.

```
cargo run --release
cargo run --release -- --windowed
cargo run --release -- --toy1
```

# Proving ground

Toy 1's playtest found good feel and no depth. The planned fix is that strategy flows through the army you design, so every ship has to be drawn, not picked. The hard part, and where the 2020 version stalled ("sketchy enemy mind stuff"), is ship AI that works for any drawing. This toy tests one answer: a three-layer brain where none of the layers knows what kind of ship it is.

The proving ground is iterated in rounds. **[ROUNDS.md](ROUNDS.md)** has the goals, every round's additions, the verdicts and the parking lot. This is round 2.

1. **The order says where to go.** Each design has one: Hunt, Guard, Harvest or Hold.
2. **The parts fire on reflex.** Flak turrets fire at the nearest enemy in range whatever the order, so an armed miner shoots back while it mines. A ship with no guns left runs home when hit.
3. **Mass and thrust decide how to fly.** Integer inertia: velocity changes by whole quarter-pixels per tick, as fast as thrust over weight allows. A ship with at least 1 thrust per weight makes attack runs (dive, overshoot, loop back). A lance fires only straight ahead, like a rook, so a ship with one slides onto the target's row or column first and then dives along it ("LINING UP" on hover). Anything heavier holds at weapon range and lines its lances up the same way. Without engines it stays put.

**Ship rules (round 2):** every ship has exactly one core, and every other pixel must join it through other pixels, edge to edge. Corners don't count. Loose pixels show red in the yard, and the ship can't fly until they're joined up. In a fight, any piece cut off from the core breaks away.

**Parts:**

| Part | Weight | Does |
|---|---|---|
| Hull | 1 | Holds the ship together. Any hit breaks it. |
| Plate | 2 | Flak bounces off it. Lances and cannon shells break it. |
| Flak | 1 | Turret, 48 px, 1 damage. Leads its aim and spreads its fire over several targets. One shot per 1.3 s. |
| Lance | 2 | Fires straight ahead only, 100 px, 2 damage. Punches through 6 pixels, or 3 of plate. One shot per 3 s. |
| Cannon | 18 (3x3) | A big turret, 130 px, turning about 42° a second. Its shell blasts a 3x3 hole, plate and all. Prefers the biggest target. One shot per 3 s. |
| Engine | 1 | Thrust. |
| Scoop | 1 | Mines and carries 3. |
| Core | 1 | Exactly one per ship. The ship dies when it's gone. |

Stats change as pixels are shot off: lose your engines and you slow down. Enemies bigger than you are solid, so you can't fly into them and shoot them from inside their plate. An attack run peels off sideways before it would hit one, so small ships make strafing passes along a big ship's hull.

**Arena:** 480x360, seen through a 304x300 window. Scroll it with WASD, the arrows or the 1/6-scale minimap. Seven ready-made tests use the six design slots, so editing a design changes the tests. The arena is deterministic: R replays exactly the same fight. Hover a ship to see its order, what it's doing, its target and where it's heading. Faint trails show recent flight paths; gold barrels show where cannons point; white sparks are shots bouncing off plate.

**Headless tools:**
- `--pg-sim`: every test, plus every pair of designs at about 600 px a side, each fought on 12 seeds (seed 1 is the fight the window replays).
- `--pg-trace BLUE_SLOT N RED_SLOT N EVERY UNTIL`: ship-by-ship state. Use blue slot 99 to run test number RED_SLOT.
- `--export-screenshots DIR`: proving-ground screenshots.

Sessions append to `proving_ground.log` (each run's designs and result). Designs persist in `proving_ground_designs.txt`; round-1 designs (15x15, old rules) were moved aside to `proving_ground_designs_round1.txt`. All three files are gitignored.

What the rounds found so far is in [ROUNDS.md](ROUNDS.md).

# Toy 1

## Controls

| Input | Does |
|---|---|
| WASD / arrows | Fly your king. If it dies, you lose. |
| 1–5 or click a card | Print it. Seed cards then need a rock: click one on the map, minimap or tactical view. |
| Right-click / Esc | Cancel planting |
| Tab / TAC | Tactical view: the whole map at exactly 1/4 scale |
| Backspace / UNDO | Take the last card off the print queue (printed pixels come back) |
| Right-click a queued card | Repeat it every time it finishes (blue frame); seeds can't repeat |
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
