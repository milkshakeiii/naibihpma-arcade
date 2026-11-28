# Naibihpma Hold

A five-minute demo of a Majesty-like village sim in the world of *The Enchanted Forest of Naibihpma*. You're the Elder of a newt and toad hold and can't give orders. You post dew rewards on heron nests and raiders, and each hero decides for themselves whether a reward is worth the risk. The album plays as the soundtrack, in order. Nothing in the game depends on hearing it.

## Play it

```
cd web
./build.sh ~/Downloads/"Henry Solberg - Many Worlds - 2026 Car-Friendly Remasters.zip"
cd ../..
python3 -I .claude/skills/game-design/scripts/demo_server.py --dir hold/web/www --port 8766 --feedback hold/feedback.jsonl
```

Then open http://localhost:8766. The `hold` entry in `.claude/launch.json` starts the same server.

`build.sh` needs the `wasm32-unknown-unknown` Rust target and, for the soundtrack, `ffmpeg`. Without the zip it builds the game only, and the game runs silently.

`demo_server.py` serves the page and appends the in-game feedback (keep, change, drop and notes) to `hold/feedback.jsonl`, with the time, the game state and the track that was playing. Both the music files and the feedback file are kept out of git.

## Controls

Majesty-style with the mouse: click things on the map to select them, click the panel's buttons, and right-click to back out. Every button also has a key, its yellow first letter.

| Action | Mouse | Keys |
|---|---|---|
| Select something | Click it | Tab / Shift+Tab steps through everything; arrows move a map cursor and Enter picks |
| Put a reward on a nest or creature | Select it, then Reward (or Reward, then click it) | R |
| Raise, lower or withdraw a reward | More, Less, Cancel | M, L, C (or + and -) |
| Hire a hero | Select a lodge or the toad hall, then Hire | H |
| Build | Build, choose, then click where | B, then L, T, S or W, then arrows and Enter |
| Pause | Pause | P or Space |
| Feedback | Feedback | F, then K, C, D, or N for a typed note |
| Scanlines | | F2 |

## How it plays

- **Dew** comes from the Elder's hall and the dew stills. It pays for buildings, for hiring, and for rewards. A posted reward is set aside, and paid out to the heroes who fought for it when the target falls. A reward nobody earned comes back.
- **Heroes judge each reward.** A hero weighs it against how dangerous the target looks, their own strength and health, their trait (brave, greedy, steady, timid) and how many others have already taken it. Company makes them braver. A hero turning a reward down shows what they'd want instead.
- **Heroes wait at the bridge** when a target is too strong for the party that has signed up so far. After 20 seconds they go anyway.
- **Hurt heroes heal** at the Star Pond, and herons let a fleeing hero go after a moment.
- **Herons** guard their nests, call more herons when a nest is attacked, and raid the hold every 40 to 60 seconds. Eighty seconds after the lesson, Gorf's stone rises in the marsh, and its wisps drain the dew stills.
- **The goal** is to destroy the three nests. The game is lost if the Elder's hall falls.

The first minute is a guided lesson that holds the world still while it waits for you. It walks you through a reward on the weak nest, watching Pip take it, hiring a toad, a reward on the next nest, and raising it until Bufo the timid toad takes it. After that the herons fight back.

## Files

| File | What it holds |
|---|---|
| `src/sim.rs` | The rules: heroes, herons, wisps, buildings, rewards, dew. One tick is one 15 FPS frame. |
| `src/map.rs` | The one-screen marsh, pathfinding, and the painted background. |
| `src/art.rs` | The palette (6-bit DAC, with slow colour cycling on water glints and spores) and all sprites. |
| `src/game.rs` | Screens, input, the panel, the guided minute, hints, feedback, and drawing. |
| `src/fb.rs` | The 400 x 300 indexed framebuffer, font, and exact integer-scale presentation with scanlines. |
| `src/bot.rs` | A scripted Elder for balance checks. |
| `src/bin/probe.rs` | `cargo run --release --bin probe [seed]` plays a game with the bot and prints the village every 30 s. |
| `src/bin/snap.rs` | `cargo run --release --bin snap -- <scene> out.ppm` writes an exact 3x screenshot of a scripted scene. |
| `web/` | The WebAssembly exports, the page, and `main.js` (scaling, cursor, input, soundtrack, sound effects, feedback). |

`cargo test --release` runs the rules tests, the lesson played with the mouse and with the keyboard, and a balance check: the bot clears the marsh in between a minute and a half and eight minutes on five seeds.
