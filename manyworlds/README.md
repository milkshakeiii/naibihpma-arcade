# Many Worlds Line

The rules from [`../DESIGN.md`](../DESIGN.md) as a deterministic simulation, four bots that play them under human speed limits (phase 0), and leg 1 as a playable gray-box in the browser (phase 1). Results are in DESIGN.md under "Phase 0 results" and "Phase 1 build".

## Play leg 1 in the browser

```
cd web
./build.sh ~/Downloads/"Henry Solberg - Many Worlds - 2026 Car-Friendly Remasters.zip"
python3 -m http.server 8765 --directory www
```

Then open http://localhost:8765. `build.sh` needs the `wasm32-unknown-unknown` Rust target (`rustup target add wasm32-unknown-unknown`) and, for the music, `ffmpeg`. Without the zip it builds the game only, and the leg runs on a silent clock. Add `#debug` to the address to get the game's exports as `window.mw` in the console.

## Bots and rules

```
cargo test --release                                   # rules tests
cargo run --release --bin gate                         # the bot gate at the current rules
cargo run --release --bin gate -- sweep [filter]       # the gate across rule variants
cargo run --release --bin gate -- diag [variant]       # one-skill bot variants
cargo run --release --bin gate -- show planner town 1 600 1800   # the board at those ticks
cargo run --release --bin gate -- show greedy gen3 2 stalls      # the board as stalls begin
```

| File | What it holds |
|---|---|
| `src/sim.rs` | The rules: rail, switches, the train, stations, passengers, scoring. One tick is one 15 FPS frame. |
| `src/map.rs` | The hand-made `town` map and generated maps (`gen1`, `gen2`, …). |
| `src/pilot.rs` | Human limits: reaction time, drag speed, one gesture at a time. |
| `src/plan.rs` | What bots can work out from the screen: the route ahead, the load, paths for new rail. |
| `src/bots/` | `random`, `greedy`, `bigloop`, `planner`. |
| `src/bin/gate.rs` | Runs the bots and prints the tables. |
| `src/show.rs` | Text pictures of the board. |
| `src/game/` | Leg 1: input, the tutorial, the HUD, and drawing into the 400 x 300 indexed framebuffer (`fb.rs`, `art.rs`). |
| `web/src/lib.rs` | The WebAssembly exports the browser calls. |
| `web/www/` | The page and `main.js`: integer scaling, the hardware cursor, input, the song as the clock, sound effects. |
