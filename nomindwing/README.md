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

The proving ground is iterated in rounds. **[ROUNDS.md](ROUNDS.md)** has the goals, every round's additions, the verdicts and the parking lot. This is round 4.

1. **The order says where to go.** Each design has one: Hunt, Guard, Harvest or Hold.
2. **The parts fire on reflex.** Flak turrets fire at the nearest enemy in range whatever the order, so an armed miner shoots back while it mines. A hunter with no guns left stays beside the nearest friend that still has some; guards and miners with no guns run home when hit.
3. **Mass and thrust decide how to fly.** Integer inertia: velocity changes by whole quarter-pixels per tick, as fast as thrust over weight allows. A ship with at least 1 thrust per weight makes attack runs (dive, overshoot, loop back). A lance fires only straight ahead, like a rook, so a ship with one slides onto the target's row or column first and then dives along it ("LINING UP" on hover). Anything heavier holds at weapon range and lines its lances up the same way. Without engines it stays put.

**Ship rules (since round 3):** a ship needs at least one core, and all its pixels must join up into one piece, edge to edge. Corners don't count. Every engine needs a straight 3-wide way out of the ship for its exhaust, like a rook's move: in one of the four directions, the engine's own line and the lines either side of it must be empty right out past the edge of the ship. So there's always a straight line of fire to every engine; plate can't fold round one (round 4; in round 3 the way out could bend). Loose pixels and buried engines show red in the yard, and the ship can't fly until they're fixed. Extra cores cost more each: the second +8, the third +16, the fourth +32. A ship's cost is its pixels plus its extra cores. In a fight, a ship dies when its last core goes. If it's cut apart, every piece that still holds a core flies on as a ship of its own, and pieces without one break away.

**Parts**, in the yard's six categories (keys 1-6; press again for the category's next part). Power has no parts yet.

| Category | Part | Weight | Does |
|---|---|---|---|
| Hull & armor | Hull | 1 | Holds the ship together. Any hit breaks it. |
| | Plate | 2 | Flak bounces off it. Lances and cannon shells break it. |
| Weapons | Flak | 1 | Turret, 48 px, 1 damage. Leads its aim and spreads its fire over several targets. One shot per 1.3 s. |
| | Lance | 2 | Fires straight ahead only, 100 px, 2 damage. Punches through 6 pixels, or 3 of plate. One shot per 3 s. |
| | Cannon | 18 (3x3) | A big turret, 130 px, turning about 42° a second. Its shell blasts a 3x3 hole, plate and all. Goes for the big ships in range (anything at least 2/3 the size of the biggest), and of those the one its barrel reaches soonest, so it stays on its target. One shot per 3 s. |
| Drive | Engine | 1 | Thrust. Needs a straight 3-wide way out. |
| Industry | Scoop | 1 | Mines and carries 3. |
| Command | Core | 1 | At least one per ship; extras cost +8, +16, +32. The ship dies when its last core is gone. |

**Shipyard (round 4):** designs go up to 99x99, odd so the mirror has a centre column. The canvas zooms in whole steps with Q/E, the mouse wheel or the -/+ buttons: 1x is the ship at its real size, as in the arena; 2x shows the whole grid; 4x and 8x are for fine work, and at 8x each cell shows its part's shape. WASD or the arrows scroll. A design opens framed at the biggest zoom that shows all of it.

| Tool | Key | Does |
|---|---|---|
| Pencil | P | Left paints, right erases, drags leave no gaps. A cannon stamps a 3x3 block. |
| Rectangle | R | Drag a box to fill it; right-drag empties one. Cannons tile it in 3x3 blocks. |
| Fill | F | Fills a hole inside the ship, or swaps a patch of one part for another; right-click empties a patch. Open space that reaches the edge won't fill. |
| Copy / stamp | C, T | Drag a box to pick it up, or right-click a design's tab to pick up the whole design. Click to place it as ordinary pixels (nothing stays linked); T turns it, right-click puts it down. |
| Undo | U | Steps back through changes to the design (64 deep). |
| Mirror | M | Every tool also works on the other side of the centre column. |

The right-hand column is icons and numbers, with the details on hover: cost, flight style, gauges for top speed, turning and thrust per weight (the gold mark is 1 thrust per weight, where attack runs start), a tally of the parts the ship has, and the status line.

Stats change as pixels are shot off: lose your engines and you slow down. Enemies bigger than you are solid, so you can't fly into them and shoot them from inside their plate. An attack run peels off sideways before it would hit one, so small ships make strafing passes along a big ship's hull.

**Arena:** 480x360, seen through a 304x300 window. Scroll it with WASD, the arrows or the 1/6-scale minimap. Nine ready-made tests use the seven design slots, so editing a design changes the tests. Keys 1-7 pick the design to place. The arena is deterministic: R replays exactly the same fight. A fight ends when one side is wiped out, after 3 minutes, or as a stalemate (a draw) once 20 seconds pass with nobody firing, nothing shot off and no armed ship closing in. The result names any side with no working guns left. Hover a ship to see its order, what it's doing, its target and where it's heading. Faint trails show recent flight paths; gold barrels show where cannons point; white sparks are shots bouncing off plate.

**Headless tools:**
- `--pg-sim`: every test, every pair of the first five designs at about 600 px of cost a side, and the titan against frigates, destroyers and capitals at its own cost, each fought on 12 seeds (seed 1 is the fight the window replays). Draws are split out into stalemates, and splits count the ships that broke in two.
- `--pg-trace BLUE_SLOT N RED_SLOT N EVERY UNTIL`: ship-by-ship state. Use blue slot 99 to run test number RED_SLOT.
- `--export-screenshots DIR`: proving-ground screenshots.

Sessions append to `proving_ground.log` (each run's designs and result). Designs persist in `proving_ground_designs.txt`, each trimmed to its drawing with its place on the grid (round 2 and 3 files, which saved the whole 41x41 grid, load centred); round-1 designs (15x15, old rules) were moved aside to `proving_ground_designs_round1.txt`. A saved design still exactly like an old default is swapped for the current one when it loads. All three files are gitignored.

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
