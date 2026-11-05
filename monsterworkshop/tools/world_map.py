"""A schematic world map, in screens: what's built, and the plan for the rest. `python tools/world_map.py OUT.png`."""
import sys

from PIL import Image, ImageDraw, ImageFont

OUT = sys.argv[1]
CW, CH = 96, 60          # one screen
COLS, ROWS = 12, 10
MX, MY = 40, 70          # margins
LEG_W = 470

colors = {
    'cloth': (66, 110, 54), 'hills': (71, 95, 69), 'kiln': (99, 114, 56), 'plains': (73, 129, 57),
    'coast': (212, 196, 148), 'capital': (150, 120, 160), 'barren': (110, 98, 84), 'sea': (40, 140, 160),
    'river': (60, 110, 170), 'none': (24, 30, 26),
}
grid = [['none'] * COLS for _ in range(ROWS)]
built = set()


def fill(kind, cells, is_built=False):
    for (x, y) in cells:
        grid[y][x] = kind
        if is_built:
            built.add((x, y))


def rect(x0, x1, y0, y1):
    return [(x, y) for y in range(y0, y1 + 1) for x in range(x0, x1 + 1)]


# Built: the Cloth Reaches and the high pass.
fill('cloth', rect(0, 6, 0, 1) + rect(3, 4, 2, 3), True)
fill('barren', [(5, 2)], True)
# Proposed.
fill('cloth', [(0, 2), (1, 2)])                     # room to grow: fields and the pumpkin patch
fill('barren', [(2, 2), (2, 3)])                    # the river gorge south out of Tinless
fill('hills', rect(6, 10, 2, 4))
fill('kiln', rect(2, 5, 4, 6))
fill('capital', rect(6, 7, 5, 6))
fill('barren', [(8, 5)])                            # the range between the hills and the coast
fill('plains', rect(0, 4, 7, 9))
fill('barren', [(5, 7), (5, 8)])                    # a salt marsh between the plains and the coast
fill('coast', rect(8, 10, 6, 9) + [(6, 7), (7, 7), (6, 8), (7, 8), (9, 5)])
fill('sea', [(11, y) for y in range(5, 10)] + [(6, 9), (7, 9), (5, 9), (10, 5)])

img = Image.new('RGB', (MX * 2 + COLS * CW + LEG_W, MY + ROWS * CH + 70), (18, 14, 22))
d = ImageDraw.Draw(img)
try:
    font = ImageFont.truetype('consola.ttf', 13)
    big = ImageFont.truetype('consolab.ttf', 15)
    title = ImageFont.truetype('consolab.ttf', 20)
except OSError:
    font = big = title = ImageFont.load_default()


def cell(x, y):
    return MX + x * CW, MY + y * CH


for y in range(ROWS):
    for x in range(COLS):
        kind = grid[y][x]
        x0, y0 = cell(x, y)
        d.rectangle([x0, y0, x0 + CW - 1, y0 + CH - 1], fill=colors[kind], outline=(30, 24, 34))
        if (x, y) in built:
            d.rectangle([x0 + 2, y0 + 2, x0 + CW - 3, y0 + CH - 3], outline=(250, 236, 160), width=2)

# The pine river: out of Tinless, south through the gorge and the kiln banks into the plains.
river = [(2.5, 0.2), (2.5, 2.0), (2.6, 3.5), (3.4, 4.6), (3.2, 6.0), (2.6, 7.2), (2.2, 9.9)]
# Its branch: out of the kiln banks east, past the marsh (its mouth makes the marsh) to the sea.
branch = [(3.25, 5.7), (4.2, 6.5), (5.0, 7.3), (5.45, 8.2), (5.6, 9.2)]
# The Rust Brook: rises in the Tin Gulch, through the springs, past the copper cut, down Iron
# Brook, and west out of the hills to join the pine river in the kiln banks.
brook = [(10.4, 2.4), (9.6, 3.2), (8.6, 3.3), (7.6, 3.7), (6.6, 4.4), (5.5, 4.6), (4.4, 4.5), (3.4, 4.6)]
for line, width in [(river, 6), (branch, 5), (brook, 3)]:
    d.line([(MX + x * CW, MY + y * CH) for x, y in line], fill=colors['river'], width=width)

# Roads, between the towns.
roads = [
    [(0.5, 0.6), (6.6, 0.6)],                       # the road across the Cloth Reaches (built)
    [(3.6, 0.6), (3.6, 2.6), (5.9, 2.6)],           # down to Temple Town and out the pass (built)
    [(5.9, 2.6), (8.2, 2.6), (8.2, 4.5)],           # into the hills
    [(3.8, 3.8), (3.8, 5.5), (6.5, 5.5)],           # Temple Town south to the kiln banks and the capital
    [(6.5, 5.5), (6.5, 7.5), (9.5, 7.5)],           # the capital south-east to the coast
    [(3.8, 5.5), (2.5, 8.3)],                       # the kiln banks down to the plains
]
for road in roads:
    d.line([(MX + x * CW, MY + y * CH) for x, y in road], fill=(196, 164, 112), width=3)

# The tunnel under the range, the hills to the coast.
tx0, ty0, tx1, ty1 = MX + 8.6 * CW, MY + 4.6 * CH, MX + 9.4 * CW, MY + 6.4 * CH
d.line([(tx0, ty0), (tx1, ty1)], fill=(20, 16, 24), width=4)
for t in range(0, 11):
    px = tx0 + (tx1 - tx0) * t / 10
    py = ty0 + (ty1 - ty0) * t / 10
    d.ellipse([px - 2, py - 2, px + 2, py + 2], fill=(200, 200, 210))

labels = {
    (1, 0.2): 'PINEWATER', (2.6, 1.1): 'TINLESS', (3.5, 0.2): 'THATCH HILLS', (4.5, 1.1): 'DYE MEADOWS',
    (5.95, 0.2): 'TANGLEMERE', (3.95, 2.15): 'TEMPLE TOWN', (5.5, 2.15): 'HIGH PASS',
    (8.5, 2.15): 'THE ORE HILLS', (3.5, 4.15): 'THE KILN BANKS', (7.0, 5.15): 'THE CAPITAL',
    (2.5, 7.15): 'THE PADDY PLAINS', (9.0, 6.15): 'THE SALT COAST', (2.3, 2.6): 'GORGE',
    (8.5, 5.1): 'RANGE', (5.5, 7.6): 'MARSH (NO ROAD)', (0.95, 2.35): 'FIELDS', (11.5, 7.4): 'SEA',
}
for (x, y), text in labels.items():
    px, py = MX + x * CW, MY + y * CH
    w = d.textlength(text, font=big)
    d.text((px - w / 2 + 1, py + 1), text, fill=(10, 8, 12), font=big)
    d.text((px - w / 2, py), text, fill=(250, 246, 232), font=big)

# Gates, where a province opens, and what each wants.
gates = [((5.95, 2.6), '1'), ((3.8, 4.0), '2'), ((6.0, 5.5), '3'), ((2.9, 7.0), '4'), ((7.5, 7.5), '5')]
for (x, y), n in gates:
    px, py = MX + x * CW, MY + y * CH
    d.rectangle([px - 9, py - 9, px + 9, py + 9], fill=(170, 60, 48), outline=(250, 236, 200), width=2)
    d.text((px - 4, py - 8), n, fill=(255, 255, 255), font=big)

# Special places, by letter.
places = [
    ((4.2, 3.4), 'B'), ((7.2, 5.8), 'P'), ((6.6, 6.2), 'M'), ((9.7, 8.6), 'C'), ((10.2, 7.9), 'L'),
    ((9.6, 3.6), 'F'), ((7.2, 3.4), 'V'), ((4.6, 5.6), 'K'), ((7.45, 6.55), 'S'), ((4.0, 6.4), 'V'),
    ((7.7, 4.3), 'X'), ((9.0, 4.2), 'O'), ((10.4, 6.6), 'O'), ((6.3, 0.75), 'O'), ((1.3, 1.4), 'H'),
    ((8.0, 2.95), 'H'), ((2.9, 5.0), 'H'), ((1.6, 8.0), 'H'), ((8.6, 7.0), 'H'),
]
for (x, y), ch in places:
    px, py = MX + x * CW, MY + y * CH
    d.ellipse([px - 10, py - 10, px + 10, py + 10], fill=(250, 236, 160), outline=(20, 16, 24), width=2)
    w = d.textlength(ch, font=big)
    d.text((px - w / 2, py - 8), ch, fill=(20, 16, 24), font=big)

d.text((MX, 20), 'MONSTER WORKSHOP: THE WORLD, IN SCREENS (A PROPOSAL)', fill=(250, 236, 160), font=title)
d.text((MX, MY + ROWS * CH + 14), 'Each cell is one 25x16-tile screen. Outlined in gold: built (19 screens). Everything else is proposed.', fill=(200, 196, 186), font=font)

lx, ly = MX * 2 + COLS * CW - 10, MY
legend = [
    ('GATES (each counts a rate of goods)', None),
    ('1 the high pass: cloth (built)', 'g'), ('2 south of Temple Town: coal or charcoal', 'g'),
    ('3 the capital road: bricks and tiles', 'g'), ('4 into the plains: sickles from the hills', 'g'),
    ('5 to the coast: metal pots, for brine', 'g'), ('', None),
    ('PLACES', None),
    ('B the Temple Bell (built)', 'p'), ('P the palace, its wings going up', 'p'), ('M the hall of masterworks', 'p'),
    ('C the charity boat, at the port', 'p'), ('L the Large Mirror, a lighthouse', 'p'),
    ('F the foundry town (statue, mirror)', 'p'), ('S the statue: in the palace garden', 'p'),
    ('K the dragon kiln (keep it fired)', 'p'), ('X the great furnace (keep it fed)', 'p'),
    ('V a conveyor vine (cliff lift, clay cliff)', 'p'), ('O an outfitter', 'p'), ('H a gathering hall', 'p'),
    ('', None),
    ('OTHER', None), ('dotted: the tunnel under the range,', None), ('  open once the coast is', None), ('blue: the pine river, its branch to the sea,', None), ('  and the Rust Brook out of the hills', None), ('tan: roads', None),
    ('', None),
    ('SCREENS', None), ('Cloth Reaches 19 + 2 fields', None), ('Ore Hills 15', None), ('Kiln Banks 12', None),
    ('Capital 4', None), ('Paddy Plains 15', None), ('Salt Coast 17 + sea', None), ('Barren 6', None),
    ('About 90 in all', None),
]
for text, kind in legend:
    fnt = big if kind is None and text.isupper() else font
    d.text((lx, ly), text, fill=(250, 236, 160) if fnt is big else (226, 222, 210), font=fnt)
    ly += 19 if text else 10
img.save(OUT)
print(img.size)
