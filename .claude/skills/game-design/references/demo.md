# Building a demo Henry can judge

## The bar

He set polish at 4 out of 5: placeholder graphics don't let him judge, and the last gray box was rejected partly for having no immersion. That makes demos more expensive, so build fewer and finish them.

A demo has:
- **His real track as the soundtrack,** from the first second of play. Failure never cuts or restarts it, and nothing in play depends on hearing it.
- **Pixel art in the repo's style** (`AGENTS.md`): 400×300, indexed palette at 6-bit precision, crisp 1-pixel dark outlines on foreground sprites against darker, recessed backgrounds, and no alpha blending. The art carries the album's world.
- **Sound effects:** a few quiet synthesized effects, with timbres that suit the album's world.
- **A guided first minute** inside the game, with the core idea showing up in that minute.
- **About five minutes of play,** then an end card.
- **Feedback keys** that save his verdict and notes to a file you can read.
- **Input as he asked:** keyboard-only on 2026-11-26. Show the keys on screen.

## Reuse the stack

The Many Worlds Line build already solved the platform. While it's on the `manyworlds-line` branch, copy from that branch:

| File | What it gives you |
|---|---|
| `manyworlds/src/game/fb.rs` | The indexed framebuffer: rectangles, Bresenham lines, outlined shapes, a 5×7 font, and `present()`, which expands to exact N×N blocks with scanlines on the bottom row. |
| `manyworlds/src/game/art.rs` | The palette at `& 0xFC` precision, and sprites authored as text grids. |
| `manyworlds/web/src/lib.rs` | WebAssembly exports with no glue library. |
| `manyworlds/web/www/main.js` | Integer scaling on black, the hardware cursor, latched input, the song's playback position as the 15 FPS clock, pausing when the tab is hidden, and WebAudio effects. |
| `manyworlds/web/build.sh` | Builds the wasm, and extracts one track from the album zip to AAC with ffmpeg. |

Put each demo in its own top-level folder with the same layout. The existing `.gitignore` already keeps `**/assets/music/` and audio files out of git. Add the demo's `web/www/*.wasm` to it as well. Never commit music.

To serve a demo and collect feedback, use `scripts/demo_server.py` in place of `python -m http.server`:

```bash
python3 -I .claude/skills/game-design/scripts/demo_server.py --dir <demo>/web/www --port 8765 --feedback <demo>/feedback.jsonl
```

Add a matching entry to `.claude/launch.json` so the browser pane can start it.

## Pixel art at polish 4

- Draw sprites as text grids in code, one character per palette index, the way `art.rs` does. That makes them easy for you to edit and review.
- Give each material a short ramp of three or four shades, plus the dark outline. Keep backgrounds darker and lower in contrast than anything the player can touch.
- Use few sprites, each drawn well: the player's thing, two or three kinds of world object, and one landmark that says where you are.
- Animate at 2 to 4 frames on the 15 FPS tick, with no smoothing.
- Look at it at real pixels. Browser screenshots arrive scaled down, so add a small `snap` binary that runs a scripted scene headlessly and writes the framebuffer at 3x as a PPM. Convert it with `ffmpeg -i x.ppm x.png` and read the PNG. `hold/src/bin/snap.rs` is an example. Fix what reads badly before Henry sees it.

## Checks the build needs

- **A scripted player for balance.** Before tuning by feel, write a crude bot that plays the way the guided minute teaches, reacting about every ten seconds, and print the state every 30 seconds over several seeds. `hold/src/bot.rs` and `hold/src/bin/probe.rs` are an example. Aim the demo at a few minutes for a person, which is about half that for the bot. When the numbers look wrong, print the deaths or failures one by one: the cause is usually a single behaviour, not the stats.
- **A debug stepper in the page.** The built-in browser pane often runs hidden, and a hidden page gets no animation frames. Behind `#debug`, expose a function that runs n frames and redraws (`window.hdStep(n)` in `hold/web/www/main.js`), so you and the fresh player can let time pass on purpose.
- **Tests for the lesson.** Play the guided minute in a test with mouse input and again with keys only, so a change can't quietly break it.

## Sound

- **Few and quiet.** Keep effects quieter than the music, and use only as many as needed to confirm actions.
- **Timbres that suit the world:** bright square-wave chirps for the retro-futurist city, soft plucks for the forest, dry clicks and sweeps for the star survey. Describe the choice in plain words in the concept page.
- **Not tuned or timed to the track.** You can't hear whether that worked.
- **The soundtrack keeps playing.** When a track ends while the player is still there, start the album's next one. A level never ends because its track did.

## The guided first minute

Write it as a script before you build it: second by second, what appears, what the player does, and how the game shows it worked.

- **From the first second,** the goal and a progress indicator are on screen.
- **One rule at a time.** The game waits for the player while each is taught, and never runs a timer during teaching.
- **Teach by doing.** A ghost or hint shows the action, the player does it, and the game answers visibly and audibly.
- **Keep words short and few,** on screen in the game's font, such as "← → STEER". No paragraphs.
- **End the minute in real play,** with the core idea happening.

## Feedback keys

A Feedback button (F) pauses the game and opens a small bar in the framebuffer. Don't use Tab for it, because Tab usually selects things. In the bar:
- **K** keep, **C** change, **D** drop
- **N** starts a typed note, which Enter saves
- **Esc** returns to the game

The end card shows the same choices.

The same bar appears on the end card.

Each entry is a POST to `/feedback` with JSON, which `demo_server.py` appends to the feedback file:

```js
fetch('/feedback', { method: 'POST', body: JSON.stringify({
  kind: 'note',              // keep | change | drop | note
  text: 'too slow to start', // typed text, or ''
  song_time: song.currentTime,
  state: w.mw_state_line ? readStateLine() : ''  // a one-line summary the game can provide
}) });
```

Read the file when he says he's done. His notes carry the song time and the game state, so you can see what was happening when he wrote each one.

## Before handing it over

1. The fresh-player test (SKILL.md, step 7) passes: the agent states the goal and the rules correctly.
2. You've watched the first minute yourself in the browser, at real speed, with sound.
3. Nothing beside the demo needs reading. Give Henry one line: how to start it, and the keys.
