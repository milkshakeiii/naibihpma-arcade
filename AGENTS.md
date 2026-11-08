# Faithful Old-School Low-Res Pixel Graphics Rules

Maintain strict authenticity to early-1990s DOS / EGA / VGA low-resolution pixel graphics (e.g., *Cosmo's Cosmic Adventure*) across all rendering, window presentation, UI, and sprite/tile assets. Do not introduce modern sub-pixel smoothing or fractional scaling.

## 1. Zero Sub-Pixel Interpolation
- **Integer-Only Logical Grid:** Store and render all positions, camera offsets, hitboxes, particles, bitmap fonts, and line primitives (integer Bresenham) using integer coordinates (`i32`) snapped directly to the logical framebuffer (e.g., `400x300` or `320x200`). Never render at fractional or sub-pixel offsets.
- **No Filtering or Runtime Transforms:** Never apply bilinear/trilinear filtering, anti-aliasing, sub-pixel font rendering, or runtime fractional sprite scaling/rotation. Keep OS blit modes set to nearest-neighbor (`COLORONCOLOR`).
- **No Alpha Blending:** Never use translucent RGBA alpha blending. Implement shadows, glass, and dim overlays strictly through palette index remapping or classic `2x2` Bayer / checkerboard dithering.

## 2. Exact Integer-Multiple Display Scaling
- **DPI-Aware Physical Pixels:** Query true physical monitor pixels (enable OS DPI awareness such as `SetProcessDPIAware()` before creating windows).
- **Exact `Nx` Scaling Only:** Compute the largest integer scale factor `N = max(1, min(screen_w / fb_w, screen_h / fb_h))`. Expand each logical pixel into an exact `N x N` physical pixel block and blit `1:1` (`src_w == dst_w`, `src_h == dst_h`) so every pixel on screen has identical width and height with zero camera-scroll shimmer.
- **Black CRT Bezel Borders:** Center the `(fb_w * N) x (fb_h * N)` viewport on the display and clear all surrounding pillarbox and letterbox margins to pure black.

## 3. Physical-Scale Toggleable CRT Scanlines
- **Apply at Final Integer Scale `N`:** Never bake scanlines into an intermediate resolution and stretch afterward (which causes uneven Moiré banding). For `N >= 2`, render the top `N - 1` physical sub-rows at full palette brightness and darken only the bottom physical sub-row (`sub_y = N - 1`) by ~20–25%.
- **Always Keep a Live Off-Toggle:** Always provide an immediate keyboard toggle (e.g., `F2`) to disable scanlines and render solid, undivided `N x N` pixel squares.

## 4. Indexed Hardware Palette & Visual Clarity
- **Indexed Framebuffer & Hardware DAC Precision:** Store pixels as palette indices (`u8`) resolved through a fixed lookup table quantized to hardware DAC precision (e.g., 64-color 6-bit-per-channel VGA via `channel & 0xFC`, index `0` = transparent).
- **Foreground/Background Separation & Calm UI:** Enforce crisp `1px` dark hull outlines around foreground sprites against darker, recessed background palette ramps. Avoid strobing or rapidly flashing UI text and borders; restrict palette cycling to subtle environmental ambience.

## 5. Pixel-Grounded Sprites & Discrete Frame Cadence
- **Zero Floating Feet:** Author grounded sprites with planted feet touching the bottom-most row of the sprite (`row = h - 1`, zero empty transparent rows underneath) and snap standing Y coordinates directly to `tile_row * tile_size - sprite_h` so characters never float 1–2 pixels above surfaces.
- **Discrete Frame Ticks + High-Hz Input Latching:** Advance simulation and animation in discrete classic frame ticks (e.g., `15 FPS` / `66.667 ms`) rather than variable-`dt` frame interpolation, while polling and latching rising-edge inputs at high frequency (e.g., `250 Hz`) between ticks so quick taps are never missed.

## 6. Mouse Pointer Exception (the only thing that moves faster than 15 FPS)
- **Never Raise the Frame Rate for Input Feel:** A pointer drawn into the framebuffer at `15 FPS` feels broken. Do not fix that by ticking or rendering faster. Keep simulation, animation and presentation at the classic cadence and special-case the pointer only, the way DOS mouse drivers drew the cursor independently of the game.
- **Pixel-Art Hardware Cursor:** Show the pointer as an OS hardware cursor built from the game's own pointer sprite: a `32bpp` `CreateDIBSection` where each logical pixel is an exact `N x N` block at the current integer scale `N`, palette colors (`channel & 0xFC`), index `0` fully transparent (alpha `0`, every other pixel alpha `255`, never partial), plus a zero `CreateBitmap` mask, combined with `CreateIconIndirect`. Return it from `WM_SETCURSOR` when the hit test is `HTCLIENT`. Rebuild it whenever `N` changes and `DestroyIcon` the old one. Never also draw the pointer into the framebuffer.
- **Poll the Mouse with the Keyboard:** Read position (`GetCursorPos` + `ScreenToClient`) and buttons in the same `250 Hz` loop, latching press and release edges. Map physical to logical as `(p - viewport_offset) / N` using the offset and `N` of the last present. Accumulate `WM_MOUSEWHEEL` deltas atomically and drain them on the tick. When a drag paints tiles, walk every tile between successive samples so fast strokes leave no gaps.
- **Discrete Responses Are Fine:** Hover highlights, tooltips and clicked results may wait for the next `15 FPS` frame; only the pointer itself must track the hand.
- **Clickable Controls Too:** Anything the mouse can drive gets an on-screen button as well as a key (e.g., run/pause, restart, speed for playback).
