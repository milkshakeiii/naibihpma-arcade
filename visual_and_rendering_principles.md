# Visual, Rendering, & Simulation Principles

This document codifies the core technical and aesthetic principles shared across projects in this repository so that all rendering, level design, UI, and animation work remains strictly faithful to early-1990s DOS/VGA pixel art (inspired by classics such as *Cosmo's Cosmic Adventure*) while displaying cleanly on modern high-resolution monitors.

---

## 1. Native `400×300` Logical Canvas (4:3 Square-Pixel Grid)

- **Logical Resolution:** `400×300` pixels (`FB_W = 400`, `FB_H = 300`) — a crisp 4:3 square-pixel logical grid (typically split between an active playfield viewport and a classic beveled Apogee-style status/HUD panel along the bottom edge).
- **Integer Tile & Sprite Grids:** Tiles and grounded sprites use exact `1:1` integer pixel dimensions (e.g., `20×20` or `24×24`), with grounded character/creature feet planted strictly on the bottom-most row (`row = h - 1`, zero empty transparent rows underneath) and snapped directly to `tile_row * tile_size - sprite_h`.

---

## 2. Zero Sub-Pixel Interpolation

- **100% Integer Coordinates:** Every entity position (`px`, `py`), camera offset (`cam_x`, `cam_y`), projectile, particle, hitbox boundary, and Bresenham tether line uses integer (`i32`) coordinates snapped directly to the `400×300` pixel grid.
- **No Bilinear Filtering or Runtime Fractional Transforms:** Sprites are never rotated, scaled by fractional amounts, or bilinearly filtered at runtime.
- **No Alpha Blending:** The framebuffer stores raw palette indices rather than RGBA values. Shadows, glass enclosures, and dim overlays use classic DOS `2×2` Bayer checkerboard dithering (`Framebuffer::rect_dither`).
- **Pixel-Crisp Typography:** All text uses an `8×8` (`5×7` glyph body with `6px` kerning) bitmap arcade font rendered exclusively at integer pixel scales (`1×`, `2×`).

---

## 3. Exact Integer-Multiple Fullscreen Presentation

To prevent uneven `3px`/`4px` pixel widths and scrolling shimmer on modern flat-panel displays, the window never stretches the framebuffer by a fractional ratio:

1. **DPI-Aware Physical Query:** On startup, the window layer calls `SetProcessDPIAware()` so Windows reports true physical monitor pixels rather than virtualized DPI-scaled coordinates.
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

- **64-Entry Indexed LUT:** Every pixel in the logical `Framebuffer` holds an index in `0..63` (`0` = sprite transparency, `1` = `1px` dark hull outline).
- **18-Bit (`6-Bit-Per-Channel`) VGA DAC Quantization:** All palette RGB channels are masked with `& 0xFC` (64 discrete steps per channel), matching IBM VGA hardware DAC precision.
- **Moody Readability & Zero Flashing UI:**
  - Foreground sprites enforce a `1px` dark hull outline (index `1`) against recessed, darker twilight playfield backdrops so characters, terrain edges, and hazards remain unmistakable.
  - UI text, borders, and prompts **never strobe or flash**; hardware palette cycling is reserved solely for slow, atmospheric background ambience.

---

## 6. Discrete 15 FPS Cadence with 250 Hz Input Latching

- **15 FPS Simulation & Render Tick (`66.667 ms`):** All physics, AI, animations, camera steps, and frame blits advance in discrete `15 Hz` ticks, preserving the crisp, deliberate frame cadence of early-1990s DOS shareware games.
- **250 Hz Input Edge Polling (`4 ms`):** Between `15 FPS` ticks, the main loop polls keyboard state every `4 ms` and latches rising-edge presses (`InputState`) until the next tick fires, ensuring quick taps are never missed.
