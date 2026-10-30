# Visual, Rendering, & Simulation Principles

This document codifies the core technical and aesthetic principles of **`naibihpma_arcade`** so that all rendering, level design, UI, and animation work remains strictly faithful to early-1990s DOS/VGA pixel art (inspired by classics such as *Cosmo's Cosmic Adventure*) while displaying cleanly on modern high-resolution monitors.

---

## 1. Native `400×300` Logical Canvas (4:3 Square-Pixel Grid)

- **Logical Resolution:** `400×300` pixels (`FB_W = 400`, `FB_H = 300` in [`src/framebuffer.rs`](../src/framebuffer.rs)).
  - **Playfield Viewport:** `400×236` (`PLAY_H = 236` in [`src/game.rs`](../src/game.rs)).
  - **Beveled Status Bar:** `400×64` along the bottom edge (`y = 236..300`), housing controls hints, character portrait, $\text{O}_2$ / tranq readout, album track display, gold/cargo counters, and the 6-slot specimen pack.
- **World Tile Grid:** `24×24` pixel tiles (`TILE_SZ = 24` in [`src/sprites.rs`](../src/sprites.rs)), arranged in `80×22` tile stages (`1920×528` logical pixels).

---

## 2. Zero Sub-Pixel Interpolation

- **100% Integer Coordinates:** Every entity position (`px`, `py`), camera offset (`cam_x`, `cam_y`), projectile, particle, hitbox boundary, and Bresenham tether line uses integer (`i32`) coordinates snapped directly to the `400×300` pixel grid.
- **No Bilinear Filtering or Runtime Fractional Transforms:** Sprites are never rotated, scaled by fractional amounts, or bilinearly filtered at runtime.
- **No Alpha Blending:** The framebuffer stores raw palette indices rather than RGBA values. Shadows, glass enclosures, and dim overlays use classic DOS `2×2` Bayer checkerboard dithering (`Framebuffer::rect_dither`).
- **Pixel-Crisp Typography:** All text uses an `8×8` (`5×7` glyph body with `6px` kerning) bitmap arcade font rendered exclusively at integer pixel scales (`1×`, `2×`).

---

## 3. Exact Integer-Multiple Fullscreen Presentation

To prevent uneven `3px`/`4px` pixel widths and scrolling shimmer on modern flat-panel displays, the window never stretches the framebuffer by a fractional ratio:

1. **DPI-Aware Physical Query:** On startup, [`src/main.rs`](../src/main.rs) calls `SetProcessDPIAware()` so Windows reports true physical monitor pixels rather than virtualized DPI-scaled coordinates.
2. **Largest Integer Multiple (`N×`):** Each frame, `Framebuffer::integer_scale_for_viewport(cw, ch)` selects the largest integer scale factor $N \ge 1$ that fits inside the monitor:
   $$N = \max\!\left(1,\; \min\!\left(\left\lfloor \frac{W_{\text{screen}}}{400} \right\rfloor,\; \left\lfloor \frac{H_{\text{screen}}}{300} \right\rfloor\right)\right)$$

   | Monitor Resolution | Integer Scale `N` | Active Viewport | Physical Block per Logical Pixel |
   | :--- | :---: | :---: | :---: |
   | `800×600` | `2×` | `800×600` | `2×2` |
   | `1280×720` / `1366×768` | `2×` | `800×600` | `2×2` |
   | `1920×1080` (1080p) | `3×` | `1200×900` | `3×3` |
   | `1920×1200` / `2560×1440` (1440p) | `4×` | `1600×1200` | `4×4` |
   | `2560×1600` | `5×` | `2000×1500` | `5×5` |
   | `3840×2160` (4K UHD) | `7×` | `2800×2100` | `7×7` |

3. **`1:1` Device Blit Centered in Black CRT Bezel Borders:** `Framebuffer::present_integer_scaled` expands the `400×300` indexed canvas directly into a `(400·N) × (300·N)` RGB buffer and blits it `1:1` (`src_w == dst_w`, `src_h == dst_h`) centered on the screen, clearing any surrounding pillarbox/letterbox margins to pure black (`BLACKNESS`).

---

## 4. Toggleable Physical-Scale CRT Scanlines (`[F2]`)

- **Live Toggle (`[F2]`):** Players can toggle CRT scanlines on or off at any time with **`F2`**.
  - **When Enabled (Default):** For any integer scale $N \ge 2$, the top $N - 1$ physical sub-rows of each logical pixel render at `100%` VGA brightness while the bottom physical sub-row (`sub_y = N - 1`) darkens by ~22% with slight cool-blue phosphor persistence (`R·198/256`, `G·198/256`, `B·204/256`). Because scanlines are generated directly at the target integer scale $N$ rather than stretched afterward, every scanline across the entire monitor has identical thickness and zero Moiré banding.
  - **When Disabled (`[F2]`):** Every logical pixel renders as a solid, undivided $N \times N$ square block.

---

## 5. Authentic 64-Color 6-Bit VGA Hardware Palette

- **64-Entry Indexed LUT:** Every pixel in [`Framebuffer`](../src/framebuffer.rs) holds an index in `0..63` (`0` = sprite transparency, `1` = `1px` dark hull outline).
- **18-Bit (`6-Bit-Per-Channel`) VGA DAC Quantization:** In [`VgaPalette::update_cycler`](../src/palette.rs), all RGB channels are masked with `& 0xFC` (64 discrete steps per channel), matching IBM VGA hardware DAC precision.
- **Moody Readability & Zero Flashing UI:**
  - Foreground sprites enforce a `1px` dark hull outline (index `1`) against recessed, darker twilight playfield backdrops (`make_recessed_playfield_bg`) so characters, terrain edges, and hazards remain unmistakable.
  - UI text, borders, and prompts **never strobe or flash**; hardware palette cycling is reserved solely for slow, atmospheric background bioluminescence (`Bank A: 48..51`).

---

## 6. Discrete 15 FPS Cadence with 250 Hz Input Latching

- **15 FPS Simulation & Render Tick (`66.667 ms`):** All physics, creature AI, animations, camera steps, and frame blits advance in discrete `15 Hz` ticks (`GameState::update_15fps_tick`), preserving the crisp, deliberate frame cadence of early-1990s DOS shareware platformers.
- **250 Hz Input Edge Polling (`4 ms`):** Between `15 FPS` ticks, [`src/main.rs`](../src/main.rs) polls keyboard state every `4 ms` and latches rising-edge presses (`InputState`) until the next tick fires, ensuring quick taps are never missed.
