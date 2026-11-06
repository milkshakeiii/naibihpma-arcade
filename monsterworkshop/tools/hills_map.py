"""The Ore Hills' plan, screen by screen: `python tools/hills_map.py OUT.png`.

The second draft (2026-11-05): vines carry goods straight to where they're used, and the road
goes the long way round (Henry). The heights are shaded: the hills' heights lightest, the valley
darkest.
"""
import sys

from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
CW, CH = 330, 210
COLS, ROWS = 5, 3
MX, MY = 30, 70
X0, Y0 = 6, 2  # the hills' first screen in the world

# Each screen: its name, how high it lies, what's there, and whether it moved in this draft.
screens = {
    (6, 2): ('THE CLIFF LIFT', 'low', ['the way in: the pass arrives', '  on low ground below a cliff', 'the lift vine: goods up to', "  Verdigris's smithy", 'a granite quarry at its foot'], 'renamed'),
    (7, 2): ('VERDIGRIS', 'high', ["the hills' town square", 'the gathering hall, boards', 'the smithy at its west end,', '  where the lift vine lands', 'the top of the Stone Stair'], 'moved'),
    (8, 2): ('COPPER CUT', 'high', ['the most copper (the boast)', 'terraced pits, rails, carts', 'the copper miner, his board', 'the ore vine: down its south', '  cliff into the smelters'], 'moved'),
    (9, 2): ('VERDIGRIS EAST', 'high', ['houses and yards', 'piles of spare picks and', '  shovels, for loops', 'a tool drop-off', 'the ramp down to the springs'], ''),
    (10, 2): ('THE TIN GULCH', 'high', ['the only tin in the world', '  (the boast)', 'a narrow gorge, tin seams', 'the Rust Brook rises here', "a prospector's camp"], ''),
    (6, 3): ('HEMP HOLLOW', 'low', ['hemp fields; a retting pond', '  on the Rust Brook', 'the hemp house: fiber, sacks', 'the hemp weaver (textiles)', 'the road from the way in'], 'moved'),
    (7, 3): ('CHARCOAL WOOD', 'mid', ['a pine ridge: logs, sticks', 'charcoal mounds, the burner', "the stonemason's yard", 'THE STONE STAIR up its north', '  cliff; charcoal vine east'], 'moved'),
    (8, 3): ('BELLOWS', 'low', ['the smelters, where the ore', '  and charcoal vines land', "the miners' outfitter (+2)", 'an assay office (ingots)', 'the road south to the drift'], 'moved'),
    (9, 3): ('ALUM SPRINGS', 'low', ['colored mineral springs:', '  alum, green vitriol', 'a brine well (well salt)', 'pyrite in the rocks', 'the ramp up to Verdigris E.'], 'moved'),
    (10, 3): ('THE FOUNDRY', 'low', ['casts the Great Statue and', '  the Large Mirror', 'pots, sycee, dies (molds', '  from the kiln banks)', 'the founder (casting)'], ''),
    (6, 4): ('IRON BROOK', 'low', ['iron seams by the Rust Brook,', '  red with iron', 'pines, a sawpit', 'boulders', ''], ''),
    (7, 4): ('THE GREAT FURNACE', 'mid', ['a towering furnace: kept fed', '  with ore and charcoal for', '  a set time, it gives', '  something special', '(continuous supply)'], ''),
    (8, 4): ('THE DEEP DRIFT', 'low', ["the tunnel's mouth: under", '  the range to the coast', '  (opens with the coast)', 'deep seams: silver', 'a timbered adit, lanterns'], ''),
    (9, 4): ('SLAG SLOPES', 'mid', ["Bellows' slag heaps", 'scree, hard stone, pyrite', 'a lookout over the range', '', ''], ''),
    (10, 4): ('THE HIGH SCREE', 'high', ['wild heights, wild hemp', "a hermit's cairn: one +3", '  piece, given here', 'boulders, wind', ''], ''),
}
levels = {'high': (108, 104, 96), 'mid': (82, 90, 80), 'low': (52, 70, 64)}

img = Image.new('RGB', (MX * 2 + COLS * CW, MY + ROWS * CH + 96), (18, 14, 22))
d = ImageDraw.Draw(img)
try:
    font = ImageFont.truetype('consola.ttf', 14)
    big = ImageFont.truetype('consolab.ttf', 16)
    title = ImageFont.truetype('consolab.ttf', 20)
except OSError:
    font = big = title = ImageFont.load_default()

for (x, y), (name, level, lines, change) in screens.items():
    px, py = MX + (x - X0) * CW, MY + (y - Y0) * CH
    d.rectangle([px, py, px + CW - 1, py + CH - 1], fill=levels[level], outline=(24, 20, 28), width=2)


def text_over(x, y, line, fill, f, back):
    """Text on a box of its cell's color, so the lines under it don't cross it."""
    w = d.textlength(line, font=f)
    d.rectangle([x - 2, y, x + w + 2, y + 17], fill=back)
    d.text((x, y), line, fill=fill, font=f)


def at(x, y):
    """A point in screen units (world screens, fractions allowed)."""
    return MX + (x - X0) * CW, MY + (y - Y0) * CH


# Cliffs: where the heights end, drawn as a heavy dark line.
cliffs = [
    [(6.6, 2.0), (6.6, 2.66), (7.0, 2.66)],  # the cliff lift's: low arrival below, Verdigris above
    [(7.0, 2.97), (9.0, 2.97)],  # the heights' edge over Charcoal Wood, Bellows and the springs
    [(7.97, 3.0), (7.97, 3.68)],  # Charcoal Wood's ridge, over Bellows
]
for c in cliffs:
    d.line([at(x, y) for x, y in c], fill=(20, 16, 18), width=9)
    d.line([at(x, y) for x, y in c], fill=(150, 140, 130), width=3)

# Roads: from the pass down into Hemp Hollow, east along the middle, up the Stone Stair to
# Verdigris; the high road east; the ramp from the springs; south to the drift and the furnace.
roads = [
    [(6.0, 2.82), (6.2, 2.82), (6.2, 3.74), (10.5, 3.74)],
    [(7.62, 3.74), (7.62, 2.76)],  # the Stone Stair
    [(7.62, 2.76), (10.5, 2.76)],  # the high road
    [(9.3, 2.76), (9.3, 3.74)],  # the ramp
    [(8.62, 3.74), (8.62, 4.9)],
    [(7.3, 3.74), (7.3, 4.62)],
    [(6.2, 3.74), (6.2, 4.62)],
]
# The Rust Brook: rises in the Tin Gulch, along the middle row's south, down Iron Brook.
brook = [(10.9, 2.6), (10.9, 3.88), (6.45, 3.88), (6.45, 4.8), (6.0, 4.8)]
d.line([at(x, y) for x, y in brook], fill=(60, 110, 170), width=7)
for road in roads:
    d.line([at(x, y) for x, y in road], fill=(196, 164, 112), width=5)



# Vines: green arrows, from where goods are set to where they're set down.
def arrow(points, label, lx, ly):
    pts = [at(x, y) for x, y in points]
    d.line(pts, fill=(10, 30, 10), width=11)
    d.line(pts, fill=(96, 200, 90), width=6)
    (x1, y1), (x2, y2) = pts[-2], pts[-1]
    ux, uy = (x2 - x1), (y2 - y1)
    n = max((ux * ux + uy * uy) ** 0.5, 1)
    ux, uy = ux / n, uy / n
    tip = (x2 + ux * 6, y2 + uy * 6)
    d.polygon([tip, (x2 - ux * 16 - uy * 11, y2 - uy * 16 + ux * 11), (x2 - ux * 16 + uy * 11, y2 - uy * 16 - ux * 11)], fill=(96, 200, 90), outline=(10, 30, 10))
    labels.append((at(lx, ly), label))


labels = []


arrow([(6.42, 2.9), (6.66, 2.9), (6.66, 2.6), (7.14, 2.6)], 'PASS GOODS UP', 6.05, 2.9)
arrow([(8.4, 2.84), (8.4, 3.14)], 'ORE DOWN', 8.45, 2.92)
arrow([(7.76, 3.65), (8.16, 3.65)], 'CHARCOAL', 7.7, 3.79)

for (x, y), (name, level, lines, change) in screens.items():
    px, py = MX + (x - X0) * CW, MY + (y - Y0) * CH
    text_over(px + 10, py + 8, name, (250, 236, 160), big, levels[level])
    if change:
        text_over(px + CW - 10 - d.textlength(change.upper(), font=font), py + 9, change.upper(), (255, 150, 120), font, levels[level])
    for k, line in enumerate(l for l in lines if l):
        text_over(px + 12, py + 32 + k * 19, line, (236, 232, 220), font, levels[level])
for (lx, ly), label in labels:
    text_over(lx, ly, label, (170, 240, 160), big, (10, 30, 10))
sx, sy = at(7.62, 3.0)
text_over(sx + 8, sy + 6, 'THE STONE STAIR', (250, 236, 200), font, (60, 50, 40))

# The entrance from the high pass, and the tunnel south.
ex, ey = at(6.0, 2.82)
d.polygon([(ex - 26, ey - 10), (ex - 6, ey), (ex - 26, ey + 10)], fill=(170, 60, 48))
d.text((ex - 24, ey - 30), 'FROM', fill=(250, 236, 200), font=font)
tx, ty = at(8.62, 4.9)
d.rectangle([tx - 10, ty - 6, tx + 10, ty + 6], fill=(10, 8, 12), outline=(200, 200, 210))

d.text((MX, 22), 'THE ORE HILLS: VINES STRAIGHT, THE ROAD THE LONG WAY ROUND (SECOND DRAFT)', fill=(250, 236, 160), font=title)
foot = MY + ROWS * CH + 14
d.text((MX, foot), 'Each cell is one 25x16-tile screen. Shade: light the heights, mid the ridges, dark the valley. Heavy lines are cliffs,', fill=(200, 196, 186), font=font)
d.text((MX, foot + 20), 'tan lines roads, blue the Rust Brook, green arrows the vines. Walking from the way in to Verdigris: 4 screens; by vine: one hop.', fill=(200, 196, 186), font=font)
d.text((MX, foot + 40), 'MOVED: swapped with each other since the first draft (Verdigris and Copper Cut; Hemp Hollow and Charcoal Wood; Bellows and Alum Springs).', fill=(255, 150, 120), font=font)
img.save(OUT)
print(img.size)
