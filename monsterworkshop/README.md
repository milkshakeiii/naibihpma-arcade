# Monster Workshop

Monsters push goods around a grid world, craft by standing at a work spot, and learn a job by watching you do it once. The feel prototype proved that core; M1 built the real economy on it, and M2 is adding the second town. Pinewater has the whole cotton cloth chain: harvest cotton, card it into slivers at the workbench, spin thread in the spinnery, weave fabric in the loom house. Every good carries real quality, variety, history and shares. See [`DESIGN.md`](DESIGN.md) for the full design. Rendering follows [`../AGENTS.md`](../AGENTS.md): 400x300 indexed 64-color VGA palette, exact integer scaling, F2 scanlines, 15 FPS ticks with 250 Hz input latching.

The rules and tech tree came from the earlier `django` and `sokobanmonstermakers` versions; they now live here.

## Run

```bash
cargo run --release
```

```bash
cargo run --release -- --windowed
```

Every world runs on a server. Playing alone starts one inside the game that only this computer can reach. To play together, one person runs a server and the others connect to it:

```bash
cargo run --release -- --server --port 7777 --world shared.json
```

```bash
cargo run --release -- --connect HOST:7777
```

The server saves the world every minute; type `STOP` in its window to save and stop it. Your game remembers who you are with `%APPDATA%\MonsterWorkshop\player.key`, so you get your own commune back whenever you reconnect. To play two players on one computer, give each window a name: `--connect HOST:7777 --player ANNA`. The server also builds and runs on Linux.

The server prints a line every minute on how it's keeping up: players and monsters, ticks a second (15 is keeping up), how long the world and the players' screens take each tick, and what it sends.

To load-test a server, run a busy made-up world as big as a full server's (98 screens, 50 players' communes, 535 looping monsters) and play it with bots, each in its own terminal:

```bash
cargo run --release -- --server --busy
```

```bash
cargo run --release -- --bots 50 --connect 127.0.0.1
```

To feel a slow connection on a fast one, add `--lag MS`: every round trip takes that many milliseconds longer, whether you're on a server or playing alone (the window's title says so). Try `--lag 150` for a far-away server.

## Controls

| Key | Action |
| :-- | :-- |
| Arrows / WASD | Walk. Walk into an item to push it one tile |
| Space | Use: harvest a ripe plant you stand on (the bolls come out in front of you, one at a time), craft at a work spot, welcome the monster you face at the gathering hall, lead the wagon's pumpkin (and let go), open and shut a wagon's tailgate from behind, unpack a kit you face, or take a gathering tool you face into a free hand |
| E (hold) | Examine: a card about whatever you face, or the workshop forecast at a work spot. Press Tab while holding E for the details page |
| Shift + arrow | Pull: back away from the item in front, dragging it along. Very slow; it's for getting unstuck |
| R | Start recording a routine (a flag marks the start). Press again on the flag to save |
| Backspace | Cancel the recording |
| P | Loop the saved routine (start from the flag) or stop it |
| Tab | Switch monster. Hold it to see every monster and what it is doing |
| C | Character sheet: the monster, its hands and its loop (Tab turns the page; up and down choose a hand, Space puts its tool down ahead; C or Esc closes) |
| F1 | Controls |
| F2 | CRT scanlines on/off |
| Esc | Save and quit (on the title screen: quit; with the character sheet open: close it) |

The game opens on a title screen listing your worlds; pick one or start a new one. Worlds save to `%APPDATA%\MonsterWorkshop\worlds\` every minute and when you quit.

The world is 7×2 flip screens: walk off an edge and the view flips to the next screen, while every monster keeps working everywhere. Pinewater, the weaving town, is in the west; the road east crosses the pine river by a rope ferry (once the town has made rope for it), passes through the wilderness, and reaches Tanglemere, the dyeing town, where a kiln, an indigo house and a dye house turn Pinewater's cloth indigo blue. Monsters on other screens show as markers at the screen edge. The bottom panel teaches the chain step by step, then asks you to teach each monster part of it.

## How it works

- **Moves are answered on the tick you press.** The logical tile changes at once and a 2-tick slide (3 ticks when pushing) plays it out. A tap during a slide is buffered. A held key starts walking on its own only after about a quarter second (`HOLD_REPEAT_TICKS`), so a tap is always exactly one tile.
- **Routines are recorded actions, not positions.** Each step remembers whether it pushed something and what. On replay a monster waits (and says why) when the item it expects isn't there yet, or when something else blocks it. It never pushes something it didn't push while recording. A routine has to end on its start flag, so it always loops cleanly.
- **Feedback:** dust when pushing, a bump and thud when blocked, a pop when harvesting, a flash and click when an item lands on a pad, fluff, a progress bar and the wheel turning while spinning, a sparkle and chime when thread appears, bins bouncing and counting deposits. Monsters you aren't steering show what they're waiting for as a small icon.

## Files

- `data/good_types.json`, `data/skills.json`, `data/monster_types.json`: the tech tree, skills and monster types, embedded in the executable.
- `data/pinewater.map`, `data/wilderness.map`, `data/tanglemere.map`: the regions as character grids, laid side by side; `src/data.rs` and `src/world.rs` load them.
- `src/econ.rs`: the crafting rules (quality, quantity, time, learning, value, shares), with `econ_tests.rs`.
- `src/game.rs`: monsters, input, pushing, record/replay and tutorial progress, with `game_tests.rs`.
- `src/work.rs`: timed work: harvesting, crafting at workshops, chutes and output mats.
- `src/tutorial.rs`: objective texts and the help sheet.
- `src/render.rs`, `src/sprites.rs`, `src/palette.rs`, `src/framebuffer.rs`: indexed rendering. Sprites are code-authored placeholders.
- `src/audio.rs`: synthesized sound effects through waveOut.

## Tests and screenshots

```bash
cargo test
```

Tests cover the crafting rules, the whole Fabric chain played by hand, and two monsters looping it for 400 simulated seconds.

```bash
cargo run --release -- --export-screenshots screenshots
```
