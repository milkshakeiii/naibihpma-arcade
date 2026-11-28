# Naibihpma Hold

**v1**, built from the five design documents in `design/`. A Majesty-like village of amphibians in *The Enchanted Forest of Naibihpma*.

You're the Elder, a giant salamander too old to leave the pool under the Hall. You can't order anyone. You:
- post dew rewards
- set what each building keeps
- place buildings
- work the Ancient Magic

Newts, caecilians, tree frogs and fire salamanders run the economy by RockCollector's rules. Toads, axolotls, bullfrogs, goliath frogs and the Olm decide for themselves whether your rewards are worth the risk. The goal is to break Gorf's Hollow, far to the east. The album plays as the soundtrack, in order; nothing in the game depends on hearing it.

The five-minute demo Henry played on 2026-11-27 is kept in `demo-v0/`.

## Play it

```
cd web
./build.sh ~/Downloads/"Henry Solberg - Many Worlds - 2026 Car-Friendly Remasters.zip"
cd ../..
python3 -I .claude/skills/game-design/scripts/demo_server.py --dir hold/web/www --port 8766 --feedback hold/feedback.jsonl
```

Then open http://localhost:8766. The `hold` entry in `.claude/launch.json` starts the same server.
- `build.sh` needs the `wasm32-unknown-unknown` Rust target, and `ffmpeg` for the soundtrack. Without the zip, the game runs silently.
- In-game feedback (F9: keep, change, drop, or a note) is appended to `hold/feedback.jsonl`.

## Controls (document 04)

The mouse works as in Majesty:
- Click things to select them.
- Click the panel's buttons.
- Right-click to back out.

Every button also has a key, its yellow letter.

| Do | Mouse | Keys |
|---|---|---|
| Select | click it | Tab / Shift+Tab |
| Scroll | pointer at a screen edge, or click the minimap | hold W A S D, or the arrows; Home centres the Hall |
| A reward (Attack, Guard, Fetch) | select the target, then REWARD or GUARD | R or G, then M / L (or + / -) for more or less, C to cancel |
| An Explore reward | EXPLORE, then click the place | X |
| Build | select the building it's built from, then BUILD, choose, click where | B, a number, arrows and Enter |
| Stock levels | STOCK on a building | T, then up/down, + / -, R to reset to the defaults |
| Auto-hatch on or off | HATCH | H |
| Transport route | ROUTE, then click the building to feed | O (N removes it) |
| Puddle (5 dew) | PUDDLE on the Hall, then click dry ground | U |
| The Ancient Magic | the wand icon | F4, then 1 to 5 |
| Heroes, folk, rewards, log | the screens icon | F5, F6, F7, F8 |
| Pause, fast | the icons | P or Space, F3 |
| The menu (restart, hints) | right-click or Esc with nothing selected | Esc |
| Scanlines | | F2 |
| Playtest feedback | | F9 |

The first minute is a guided start. It walks through:
1. a reward on the lone nest
2. watching Pip take it and Bufo ask for more
3. raising it
4. ordering the Ironcap Cave from the Chorus Log

## What v1 changed from the documents

The rules were ported to Rust with the enemies in, and tuned with scripted Elders (`src/bot.rs`, `cargo run --release --bin probe`). These numbers moved, all in `src/data.rs`:
- **Herons:**
  - 6 health, dexterity +2, a 1d4 beak, four to a nest, a new heron every 1,800 ticks.
  - They notice prey only within 6 squares, and hunt within 10 of their nest (Testario's skeletons hunted within 10 and looked 5).
  - Their call reaches 8 squares.
  - Raids are smaller (1 to 3 herons), and raiders fly home after two minutes.
- **Hold buildings** have about two and a half times document 02's health, and demolition difficulty 15. Small raids were flattening them in seconds.
- **Costs:** the Spawning Pool costs 150 dew and the Olm's Cave 120.
- **Snapping turtles** have 30 health and a +6 shell; Gorf's Hollow has 400 health.
- **Enemies stop at level 6.** In RockCollector every block teaches, which made the turtles unbeatable.
- **Wisps** drain a dew every 300 ticks.
- **Leeches** do a point of damage every 300 ticks a stack, and drop off slowly once out of the water.
- **Folk** run from any predator close by. Only an armed newt (level 5, with a shovel) or the Chorus Toad defends against raiders.
- **Heroes** weigh the odds round their target, counting the heroes already there, so company makes them braver. They flee at 1.5 times their nerve.
- **The start** places Toad Hall and the Chorus Log away from the lone nest, and the near nests further north and south.
- **The early pace,** after the first fresh-player test:
  - a fern gives 20 dew;
  - the Hall starts with 100;
  - mending by hand takes 20 ticks a point;
  - heroes look at a new reward at once, rather than finishing their stroll first.
- **Puddles** are dug by any newt, faster with a shovel, before it goes back to carrying.
- **Creatures that stop to work, fight or rest take a square of their own** (Henry, 2026-11-28). Moving through each other is fine.
- **Heroes going home leave a fifth of their purse above 30 dew** (Majesty's guild share), so late-game dew comes back to the Hall.
- **Slings hit buildings** for their dice, where RockCollector gave ranged weapons no demolition multiplier.
- **The start shows more,** after the second fresh-player test:
  - the cliffs north of the Hold start explored, so the guided start can show where caves go;
  - while you place a building, every place it fits is marked in green;
  - the build menu says what each building is for;
  - the event line sits at the top of the map;
  - nights are a step lighter.

With the scripted Elder, which never casts a spell or digs a puddle:
- **Wins:** on the last run of ten seeds, five won (between 40 and 75 minutes), four survived 75 minutes without finishing, and one lost at 26 minutes.
- **The usual stopping point** is the end: the turtles round the Hollow, and dew running short for the last rewards.

## Files

| File | What it holds |
|---|---|
| `src/data.rs` | Every number: items, kinds, abilities, conditions, spots, buildings, spells |
| `src/terrain.rs` | The 100 x 45 marsh: tiles, damp and dry, the curse, pathfinding, fog |
| `src/world/` | The rules: the world and its tick (`mod.rs`), the economy (`econ.rs`), combat and conditions (`fight.rs`), minds and tasks (`think.rs`), the marsh's dangers (`marsh.rs`), and the Elder's commands and rewards (`elder.rs`) |
| `src/game.rs` | The screen: panel, windows, keys, the guided start, hints, feedback, title and end |
| `src/art.rs` | The palette and every sprite and tile |
| `src/fb.rs` | The 400 x 300 indexed framebuffer, font, and integer-scale presentation with scanlines |
| `src/bot.rs` | Scripted Elders, and the balance tests |
| `src/bin/probe.rs` | `cargo run --release --bin probe [seed] [-v]` plays the scenario headless and prints how it went |
| `src/bin/snap.rs` | `cargo run --release --bin snap -- <scene> out.ppm` writes a 2x screenshot |
| `src/bin/play.rs` | `cargo run --release --bin play -- moves.txt out.ppm` replays a script of clicks, keys and waits, and writes the screen where it ends. The fresh-player tests used it. |
| `design/` | The five design documents, the Majesty research, and the economy simulation they were tuned with |
| `web/` | The WebAssembly exports, the page, and `main.js` (scaling, cursor, input, soundtrack, sound effects, feedback) |

`cargo test --release` runs:
- the terrain checks
- the guided start, played with the mouse and keys
- a pass over every key and button
- the balance gate: a steady Elder keeps the Hall for 30 minutes on at least four of five seeds
