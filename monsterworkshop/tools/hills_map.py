"""The Ore Hills' plan, screen by screen (a proposal): `python tools/hills_map.py OUT.png`."""
import sys

from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
CW, CH = 330, 210
COLS, ROWS = 5, 3
MX, MY = 30, 70
X0, Y0 = 6, 2  # the hills' first screen in the world

# Each screen: its name, its kind (colors it), and what's there.
screens = {
    (6, 2): ('THE STONE STAIR', 'cliff', ['the way in: the road from the', '  pass climbs a cliff', 'the cliff-lift vine hauls', '  goods up beside it', 'a hard-stone quarry below']),
    (7, 2): ('COPPER CUT', 'mine', ['the most copper (the boast)', 'terraced pits, copper seams', 'rails and carts, ore heaps', 'the copper miner', '  (prospecting), his board']),
    (8, 2): ('VERDIGRIS', 'town', ["the hills' town square", 'the gathering hall', 'order boards, a well', 'the smithy: forge, anvils', 'the smith (blacksmithing)']),
    (9, 2): ('VERDIGRIS EAST', 'town', ['houses and yards', 'piles of spare picks and', '  shovels, for loops', 'a tool drop-off', 'the road on to the gulch']),
    (10, 2): ('THE TIN GULCH', 'mine', ['the only tin in the world', '  (the boast)', 'a narrow gorge, tin seams', 'the Rust Brook rises here', "a prospector's camp"]),
    (6, 3): ('CHARCOAL WOOD', 'wood', ['dark pines: pine logs, sticks', 'charcoal mounds, smoking', 'the charcoal burner (firing)', "the stonemason's yard", '  (stone carving)']),
    (7, 3): ('HEMP HOLLOW', 'field', ['hemp fields; a retting pond', '  on the Rust Brook', 'the hemp house: fiber, sacks,', "  the miners' coarse cloth", 'the hemp weaver (textiles)']),
    (8, 3): ('ALUM SPRINGS', 'springs', ['colored mineral springs:', '  alum, green vitriol', 'a brine well (well salt,', '  boiled in a strong pan)', 'pyrite in the rocks']),
    (9, 3): ('BELLOWS', 'town', ['the smelters: ore and', '  charcoal to ingots; bronze', "the miners' outfitter", '  (the +2 belt)', 'an assay office (ingots)']),
    (10, 3): ('THE FOUNDRY', 'works', ['casts the Great Statue and', '  the Large Mirror', 'pots, sycee, dies (molds', '  from the kiln banks)', 'the founder (casting)']),
    (6, 4): ('IRON BROOK', 'mine', ['iron seams by the Rust Brook,', '  red with iron', 'pines, a sawpit', 'boulders', '']),
    (7, 4): ('THE GREAT FURNACE', 'works', ['a towering furnace: kept fed', '  with ore and charcoal for', '  a set time, it gives', '  something special', '(continuous supply)']),
    (8, 4): ('THE DEEP DRIFT', 'mine', ["the tunnel's mouth: under", '  the range to the coast', '  (opens with the coast)', 'deep seams: silver', 'a timbered adit, lanterns']),
    (9, 4): ('SLAG SLOPES', 'wild', ["Bellows' slag heaps", 'scree, hard stone, pyrite', 'a lookout over the range', '', '']),
    (10, 4): ('THE HIGH SCREE', 'wild', ['wild heights, wild hemp', "a hermit's cairn: one +3", '  piece, given here', 'boulders, wind', '']),
}
colors = {'wood': (40, 70, 56), 'field': (86, 104, 62), 'town': (92, 96, 104), 'mine': (78, 70, 64), 'cliff': (96, 92, 84),
          'springs': (64, 104, 110), 'works': (110, 76, 58), 'wild': (70, 82, 68)}

img = Image.new('RGB', (MX * 2 + COLS * CW, MY + ROWS * CH + 60), (18, 14, 22))
d = ImageDraw.Draw(img)
try:
    font = ImageFont.truetype('consola.ttf', 14)
    big = ImageFont.truetype('consolab.ttf', 16)
    title = ImageFont.truetype('consolab.ttf', 20)
except OSError:
    font = big = title = ImageFont.load_default()

for (x, y), (name, kind, lines) in screens.items():
    px, py = MX + (x - X0) * CW, MY + (y - Y0) * CH
    d.rectangle([px, py, px + CW - 1, py + CH - 1], fill=colors[kind], outline=(24, 20, 28), width=2)
    d.text((px + 10, py + 8), name, fill=(250, 236, 160), font=big)
    for k, line in enumerate(lines):
        d.text((px + 12, py + 34 + k * 19), line, fill=(236, 232, 220), font=font)


def at(x, y):
    """A point in screen units (world screens, fractions allowed)."""
    return MX + (x - X0) * CW, MY + (y - Y0) * CH


# Roads: in from the pass along the north, down through the springs to the drift, east to Bellows
# and the foundry, west to the copper cut, the stone stair and the furnace.
roads = [
    [(6.0, 2.31), (6.02, 2.31), (6.02, 2.86), (10.5, 2.86)],
    [(8.9, 2.86), (8.9, 4.9)],
    [(8.9, 3.86), (10.5, 3.86)],
    [(8.9, 3.86), (6.9, 3.86)],
    [(7.9, 3.86), (7.9, 4.86)],
    [(6.9, 3.86), (6.9, 4.86)],
]
# The Rust Brook: rises in the Tin Gulch, through the springs, past the copper cut, down Iron Brook
# and west out of the hills (bridged where the roads cross it).
brook = [(10.86, 2.7), (10.86, 3.72), (7.0, 3.72), (6.86, 4.72), (6.0, 4.72)]
d.line([at(x, y) for x, y in brook], fill=(60, 110, 170), width=7)
for road in roads:
    d.line([at(x, y) for x, y in road], fill=(196, 164, 112), width=5)
bx, by = at(6.0, 4.75)
d.text((bx + 8, by + 8), 'THE RUST BROOK, WEST TO THE PINE RIVER', fill=(170, 200, 236), font=font)
# The entrance from the high pass, and the tunnel south.
ex, ey = at(6.0, 2.31)
d.polygon([(ex - 26, ey - 10), (ex - 6, ey), (ex - 26, ey + 10)], fill=(170, 60, 48))
d.text((ex - 24, ey - 30), 'FROM', fill=(250, 236, 200), font=font)
tx, ty = at(8.9, 4.9)
d.rectangle([tx - 10, ty - 6, tx + 10, ty + 6], fill=(10, 8, 12), outline=(200, 200, 210))
d.text((tx - 190, ty - 9), 'TUNNEL TO THE COAST', fill=(220, 220, 228), font=font)

d.text((MX, 22), 'THE ORE HILLS, SCREEN BY SCREEN (A PROPOSAL)', fill=(250, 236, 160), font=title)
d.text((MX, MY + ROWS * CH + 16), 'Each cell is one 25x16-tile screen. Tan lines are roads. Names are working names.', fill=(200, 196, 186), font=font)
img.save(OUT)
print(img.size)
