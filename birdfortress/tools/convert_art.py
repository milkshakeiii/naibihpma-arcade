"""Converts painted art into indexed VGA images for the game.

Each painting is cropped, downscaled with a box filter, given a small palette (median cut),
quantized to 6-bit DAC precision, and ordered-dithered with a 4x4 Bayer matrix. Writes
`NAME.png` (8-bit grayscale whose values are palette slots 0..N-1) and `NAME.pal`
(one "r g b" per slot) into assets/art/.

Backdrops are dimmed and desaturated so the playfield stays readable on top of them.

    python tools/convert_art.py SOURCE_DIR
"""
import colorsys
import os
import sys

import numpy as np
from PIL import Image

BAYER4 = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]], dtype=np.float32) / 16.0 - 0.5

# name: (source file, width, height, colours, dim, dither spread)
JOBS = {
    "title": ("test_tree.jpg", 400, 300, 46, False, 22.0),
    "meadow": ("meadow.jpg", 400, 240, 10, True, 40.0),
    "quarry": ("quarry.jpg", 400, 240, 10, True, 40.0),
    "canopy": ("canopy.jpg", 400, 240, 10, True, 40.0),
    "bastion": ("bastion.jpg", 400, 240, 10, True, 40.0),
    "sky": ("sky.jpg", 400, 240, 10, True, 40.0),
}


def crop_to(im, w, h):
    sw, sh = im.size
    target = w / h
    if sw / sh > target:
        cw = int(sh * target)
        x0 = (sw - cw) // 2
        return im.crop((x0, 0, x0 + cw, sh))
    ch = int(sw / target)
    return im.crop((0, 0, sw, ch))


def dim(rgb):
    out = np.empty_like(rgb)
    flat = rgb.reshape(-1, 3) / 255.0
    res = out.reshape(-1, 3)
    for i, (r, g, b) in enumerate(flat):
        hh, ss, vv = colorsys.rgb_to_hsv(r, g, b)
        ss *= 0.8
        vv = 0.10 + 0.70 * vv
        res[i] = np.array(colorsys.hsv_to_rgb(hh, ss, vv)) * 255.0
    return out


def convert(src_dir, out_dir, name, job):
    src, w, h, n, dimmed, spread = job
    im = Image.open(os.path.join(src_dir, src)).convert("RGB")
    im = crop_to(im, w, h).resize((w, h), Image.BOX)
    rgb = np.asarray(im, dtype=np.float32)
    if dimmed:
        small = rgb.copy()
        rgb = dim(small)
    # Palette from a median cut of the (dimmed) image, snapped to 6-bit channels.
    pal_img = Image.fromarray(rgb.clip(0, 255).astype(np.uint8)).quantize(colors=n, method=Image.Quantize.MEDIANCUT)
    pal = np.array(pal_img.getpalette()[: n * 3], dtype=np.float32).reshape(n, 3)
    pal = (pal.astype(np.int32) & 0xFC).astype(np.float32)
    # Sort dark to light so slots read as a ramp.
    order = np.argsort(pal.sum(axis=1))
    pal = pal[order]
    ys, xs = np.mgrid[0:h, 0:w]
    jitter = BAYER4[ys % 4, xs % 4][..., None] * spread
    p = rgb + jitter
    d = ((p[:, :, None, :] - pal[None, None, :, :]) ** 2).sum(axis=3)
    idx = d.argmin(axis=2).astype(np.uint8)
    Image.fromarray(idx, mode="L").save(os.path.join(out_dir, name + ".png"), optimize=True)
    with open(os.path.join(out_dir, name + ".pal"), "w") as f:
        for r, g, b in pal.astype(int):
            f.write(f"{r} {g} {b}\n")
    # A preview of the result in true colour.
    prev = pal[idx].astype(np.uint8)
    Image.fromarray(prev).resize((w * 2, h * 2), Image.NEAREST).save(os.path.join(src_dir, name + "_preview.png"))
    print(name, "done")


if __name__ == "__main__":
    src_dir = sys.argv[1]
    out_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "art")
    os.makedirs(out_dir, exist_ok=True)
    for name, job in JOBS.items():
        convert(src_dir, out_dir, name, job)
