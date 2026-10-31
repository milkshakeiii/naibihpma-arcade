# Monster Workshop — feel prototype

A small test of whether the core of *Monster Workshop* is fun: monsters push things around a grid, craft by standing at a workstation, and learn a job by watching you do it once. This is one yard, one recipe, and two monsters, with no economy. Rendering follows [`../AGENTS.md`](../AGENTS.md): 400x300 indexed 64-color VGA palette, exact integer scaling, F2 scanlines, 15 FPS ticks with 250 Hz input latching.

The full design (economy, skills, tech tree, towns) lives in the `sokobanmonstermakers` and `django` repos. This prototype only covers how the game feels to play.

## Run

```bash
cargo run --release
```

```bash
cargo run --release -- --windowed
```

## Controls

| Key | Action |
| :-- | :-- |
| Arrows / WASD | Walk. Walk into an item to push it one tile |
| Space / E | Use: harvest a ripe plant you stand on (the boll pops out in front of you), or spin at the wheel |
| R | Start recording a routine (a flag marks the start). Press again on the flag to save |
| Backspace | Cancel the recording |
| P | Loop the saved routine (start from the flag) or stop it |
| Tab | Switch monster |
| F1 | Controls |
| F2 | CRT scanlines on/off |
| Esc | Quit |

The bottom panel teaches the loop step by step: push cotton onto both pads, spin, deliver the thread, record the job, loop it, then switch to the second monster and teach it to keep the cotton bin full.

## How it works

- **Moves are answered on the tick you press.** The logical tile changes at once and a 2-tick slide (3 ticks when pushing) plays it out. A tap during a slide is buffered. A held key starts walking on its own only after about a quarter second (`HOLD_REPEAT_TICKS`), so a tap is always exactly one tile.
- **Routines are recorded actions, not positions.** Each step remembers whether it pushed something and what. On replay a monster waits (and says why) when the item it expects isn't there yet, or when something else blocks it. It never pushes something it didn't push while recording. A routine has to end on its start flag, so it always loops cleanly.
- **Feedback:** dust when pushing, a bump and thud when blocked, a pop when harvesting, a flash and click when an item lands on a pad, fluff, a progress bar and the wheel turning while spinning, a sparkle and chime when thread appears, bins bouncing and counting deposits. Monsters you aren't steering show what they're waiting for as a small icon.

## Files

- `src/world.rs`: the yard map as data, plus items, plants, bins, the workshop and particles.
- `src/game.rs`: monsters, input, pushing, record/replay, crafting and tutorial progress.
- `src/tutorial.rs`: objective texts and the help sheet.
- `src/render.rs`, `src/sprites.rs`, `src/palette.rs`, `src/framebuffer.rs`: indexed rendering. Sprites are code-authored placeholders.
- `src/audio.rs`: synthesized sound effects through waveOut.

## Tests and screenshots

```bash
cargo test
```

One test plays the whole tutorial from a script, then runs 200 simulated seconds and checks that both looping monsters keep delivering thread.

```bash
cargo run --release -- --export-screenshots screenshots
```
