# Star Pond — prototype: *The Underbrush*

A prototype of the first level of *Star Pond*. The level lasts exactly one song (s3-01 *The Underbrush*, 5:52), and the song's section map drives the world. You play Twen, a newt who gathers dew. You can't die; the song plays to the end and the run is tallied. Rendering follows [`../AGENTS.md`](../AGENTS.md): 400x300 indexed 64-color VGA palette, exact integer scaling, F2 scanlines, 15 FPS ticks with 250 Hz input latching.

## Run

```bash
cargo run --release
```

```bash
cargo run --release -- --windowed
```

The game looks for `1_the_underbrush_master.wav` (or `the_underbrush.wav`) in `assets/music/` next to the exe or the working directory, then `C:\Users\User\Desktop\albums\s3_thenechantedforestofnaibihpma\mastered_resonance\`, then `../creaturecollector/assets/music/`. Pass `--music <file.wav>` to choose one (44.1 kHz; 16/24-bit PCM or 32-bit float). Without a WAV the level runs silently on a wall clock. Audio files stay out of git.

## Controls

| Key | Action |
| :-- | :-- |
| Arrows / WASD | Move. Up climbs while clinging. Down drops through branches and lily pads |
| Z / Space / X / K | Jump. In water, each press is a swim stroke that can leap out at the surface |
| Press into a wall mid-air | Cling. Up climbs, jump wall-jumps, pressing away lets go |
| F2 | CRT scanlines on/off |
| Esc | Quit |
| F3 | Debug overlay: E/S marks exposed/shaded dew sites, evaporation bars, song time |
| PageDown / N | Skip to the next section (dev) |
| R | Restart the song (dev) |

## The rules (spoilers: this is the hidden depth)

- **Mood → weather.** Mist, cavern and swamp-night sections condense dew on their sites (`c` canopy, `v` cavern, `r` reeds). Sunny sections (the harp clearing and finale) shrink exposed drops by 1 every 8 s. A drop is exposed when nothing at all is above it in its column.
- **Echoes.** When a section ends, the game remembers which drops are still uncollected. When a later section echoes it (the canopy lament echoes the mist, the harp finale echoes the clearing), each remembered drop still present grows by +2, up to 5. A drop left untouched under the leaves from 0:30 until the finale is worth 5. Remembered drops show a faint glowing core.
- **The bog rises** from 3:14 to 3:26 and recedes after 3:55. Drops underwater dissolve.
- Grabbing everything as soon as it appears scores about 48. Planning around the echoes scores up to about 90. The tally shows dew gathered, dew grown by echoes, dew lost to the sun and the bog, and dew left behind. The best score is saved to `starpond_best.txt`.

## Files

- `levels/underbrush.lvl`: the whole level as data: section timeline (transcribed from `music_listening_notes.md`), water keyframes and the 60x24 tile map.
- `src/game.rs`: song-driven world (condense/evaporate/echo/flood) and Twen's movement.
- `src/audio.rs`: waveOut WAV streaming; the song clock comes from the device's playback position, so a long song never drifts.
- `src/render.rs`, `src/palette.rs`, `src/sprites.rs`, `src/framebuffer.rs`: indexed rendering, mood palettes with dithered palette wipes, HUD song-shape strip.

## Tests and screenshots

```bash
cargo test
```

```bash
cargo test -- --ignored
```

The ignored test needs an audio device and the Desktop master. It checks that the device clock follows playback and seeks.

```bash
cargo run --release -- --export-screenshots screenshots
```
