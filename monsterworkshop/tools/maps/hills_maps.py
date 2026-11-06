"""Builds the Ore Hills' first screens from the image model's variants Henry picked (2026-11-05):
the Cliff Lift from the Stone Stair's variant 1 (tools/briefs/hills_stair.txt), without its
stair, since the road goes the long way round, and the Copper Cut from its variant 2
(hills_copper.txt). Wild screens are open to their neighbors, as in the first province, where
only the villages are hedged round (Henry): the Cliff Lift on its west (to the pass) and south (to
Hemp Hollow), the Copper Cut on its west and east (Verdigris and Verdigris East will bring their
own hedges). The Copper Cut's south side is the heights' edge, two rows of rock, the ore vine
down it to Bellows. Vines' feet, goods laid out and Old Patina's loop are in the regions' data in
src/world.rs.

    python tools/maps/hills_maps.py
"""
from pathlib import Path

W, H = 25, 16
DATA = Path(__file__).resolve().parents[2] / 'data'


def blank():
    g = [['.'] * W for _ in range(H)]
    for x in range(W):
        g[0][x] = g[H - 1][x] = '#'
    for y in range(H):
        g[y][0] = g[y][W - 1] = '#'
    return g


def put(g, ch, *cells):
    for (x, y) in cells:
        assert 0 <= x < W and 0 <= y < H, (x, y)
        g[y][x] = ch


def box(g, ch, x0, y0, x1, y1):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(g, ch, (x, y))


# --- The Cliff Lift: open to the pass and to Hemp Hollow.
c = blank()
box(c, '.', 0, 1, 0, 15)
box(c, '.', 1, 15, 23, 15)
box(c, 'Λ', 8, 1, 8, 4)          # the ridge: the heights' west side
box(c, 'Λ', 8, 5, 23, 9)         # the face, its foot along row 9
box(c, 'ʃ', 21, 5, 21, 9)        # the lift vine, up the face
box(c, ',', 19, 4, 24, 4)        # the high road, from the vine's head east into Verdigris
box(c, ',', 0, 5, 5, 5)          # the road in from the pass,
box(c, ',', 5, 6, 5, 15)         # and on south, the long way round
box(c, ',', 6, 12, 17, 12)       # a path east to the quarry and the vine's foot
box(c, '▒', 18, 10, 23, 15)      # the quarry's floor
put(c, '▒', (14, 12), (15, 12), (16, 12), (17, 12), (16, 13), (17, 13))
put(c, 'ĝ', (18, 11), (20, 11), (22, 11), (19, 12), (21, 12))
box(c, 'Ł', 14, 10, 16, 11)      # the stonecutter's lean-to
put(c, '▪', (16, 13), (17, 13), (16, 14), (17, 14), (23, 10))
box(c, 'ʂ', 22, 14, 23, 14)      # the sledge
put(c, '∴', (7, 6), (7, 7), (7, 9), (9, 10), (12, 10))
put(c, '!', (19, 3), (13, 11))
put(c, '-', (10, 2), (11, 2))
put(c, 'ɑ', (13, 2))
put(c, 'Y', (2, 2), (3, 2), (14, 2), (20, 1), (1, 10), (1, 11), (2, 11), (1, 12), (2, 12), (1, 13), (7, 13), (14, 14), (2, 14), (9, 15))
put(c, '@', (4, 4), (21, 1), (22, 1), (22, 2), (23, 2), (3, 14), (11, 13), (12, 13), (0, 2))
put(c, 's', (1, 6), (4, 13))
assert c[10][21] == '▒', 'the vine stands on open ground'

# --- The Copper Cut: open to Verdigris and Verdigris East, rock on its south.
k = blank()
box(k, '▒', 0, 1, 24, 13)
box(k, 'Λ', 0, 14, 24, 15)       # the heights' edge, over Bellows
box(k, ',', 0, 4, 24, 4)         # the high road, west to Verdigris and east to Verdigris East
put(k, '©', (3, 2), (4, 2))      # Old Patina's own
box(k, '©', 11, 2, 17, 2)
put(k, 'N', (2, 2))
put(k, 'b', (2, 3))
box(k, 'Ш', 5, 5, 21, 5)         # the pit's north wall,
box(k, 'Ш', 2, 6, 3, 13)         # its west wall,
box(k, 'Ш', 22, 5, 23, 13)       # its east wall,
box(k, 'Ш', 5, 10, 21, 11)       # and the wall between its terraces
box(k, '≡', 4, 5, 4, 7)          # steps down from the road,
put(k, '≡', (10, 5))
box(k, '≡', 4, 10, 4, 11)        # and between the terraces
box(k, '≡', 14, 10, 14, 11)
box(k, 'Ĥ', 7, 5, 8, 6)          # the miners' shed, in the wall
box(k, '©', 5, 8, 13, 8)         # the upper terrace's seams
box(k, '©', 5, 12, 13, 12)       # the lower terrace's
box(k, '=', 11, 6, 20, 6)
put(k, '&', (13, 6), (17, 6))
box(k, 'ω', 16, 7, 19, 9)        # the windlass over the shaft
box(k, 'ʬ', 18, 2, 20, 3)        # spoil heaps, on the rim and in the bottom
box(k, 'ʬ', 18, 12, 20, 13)
put(k, 'ŧ', (6, 7), (21, 13))
put(k, 'i', (9, 7))
put(k, '!', (6, 3), (15, 9), (20, 9))
put(k, 'Y', (1, 1), (2, 1), (7, 1), (13, 1), (19, 1), (20, 1), (21, 1), (23, 1), (1, 2), (22, 2), (1, 5), (0, 9), (1, 12), (24, 8))
put(k, '@', (5, 1), (6, 1), (21, 2), (2, 5), (0, 2), (24, 11))
put(k, 'ʃ', (16, 14), (16, 15))  # the ore vine, down the rock into Bellows
assert k[13][16] == '▒', "the ore vine's foot is open ground"

# --- Hemp Hollow, from variant 2 (tools/briefs/hills_hemp.txt): the hemp house on the retting
# pond, with a door on each side (Henry): north onto the road along its back, south onto the road
# by the pond. The brook comes in at the right edge on the rows Charcoal Wood's leaves by, fills the
# pond, and goes out of the bottom edge down Iron Brook; the lower road leaves on the row Charcoal
# Wood's comes in on. Open to the Cliff Lift above, Charcoal Wood beside it and Iron Brook below.
h = blank()
box(h, '.', 1, 0, 24, 0)
box(h, '.', 1, 15, 24, 15)
box(h, '.', 24, 1, 24, 14)
box(h, ',', 5, 0, 5, 15)         # the road from the Cliff Lift, on south to Iron Brook
box(h, ',', 6, 11, 24, 11)       # the lower road, past the pond to Charcoal Wood
box(h, ',', 15, 4, 24, 4)        # the road along the house's back, to Charcoal Wood's stair
box(h, 'w', 13, 12, 20, 14)      # the retting pond,
box(h, 'w', 21, 13, 24, 14)      # fed by the Rust Brook from Charcoal Wood,
put(h, 'w', (12, 13), (12, 14), (12, 15), (13, 15))  # and on down Iron Brook
put(h, 'ɽ', (15, 12), (17, 13), (19, 12), (16, 14))  # hemp soaking, under stones
put(h, '¥', (13, 12), (20, 14), (22, 13), (12, 14))
box(h, 'x', 6, 2, 14, 2)         # the field's fence,
box(h, 'x', 6, 9, 14, 9)
box(h, 'x', 6, 3, 6, 8)
box(h, 'x', 14, 3, 14, 8)
put(h, '.', (10, 9))             # and its gate
for y in (3, 5, 7):
    put(h, 'ĥ', (7, y), (9, y), (11, y), (13, y))
box(h, 'W', 15, 5, 22, 9)        # the hemp house
box(h, '_', 16, 6, 21, 8)
put(h, 'ʜ', (18, 7))
put(h, 'p', (17, 7), (19, 7), (20, 7))
put(h, 'o', (18, 8))
put(h, 'g', (16, 6), (21, 6))
put(h, 'D', (18, 5), (18, 9))    # its doors, north and south
put(h, ',', (18, 10))
put(h, 's', (4, 2), (24, 3), (6, 10))
box(h, 'ɦ', 17, 1, 19, 1)        # hemp drying,
put(h, 'ʈ', (20, 1), (21, 2), (3, 4), (3, 8), (3, 12), (11, 12))
box(h, 'ɦ', 6, 13, 7, 13)
put(h, 'ƨ', (15, 2))
put(h, 'ş', (13, 10))
put(h, 'ŭ', (14, 10))
box(h, '-', 20, 10, 21, 10)      # benches by the water
box(h, '-', 19, 15, 20, 15)
put(h, 'Y', (1, 8), (2, 13), (3, 14), (23, 5), (17, 3), (22, 1))
put(h, '@', (22, 3), (9, 14))
assert h[4][18] == ',' and h[10][18] == ',', "both the hemp house's doors open onto a road"

# --- Charcoal Wood, from the second round's variant 1 (tools/briefs/hills_charcoal_2.txt), which
# kept the draft's layout (Henry: the draft was closest): a pine ridge, its north cliff up to
# Verdigris, climbed by the Stone Stair; its east edge a cliff over Bellows, crossed by the
# charcoal vine; open to Hemp Hollow, whose back road comes in along row 4 to the stair, and south
# over the brook to the Great Furnace. The brook comes in from Bellows and goes out to Hemp Hollow
# on the rows they carry it on.
w = blank()
box(w, '.', 0, 0, 24, 15)
box(w, 'Λ', 0, 0, 24, 2)         # the heights' edge, under Verdigris
box(w, '≡', 15, 0, 16, 2)        # the Stone Stair
box(w, 'Λ', 22, 3, 24, 9)        # the ridge's east edge, over Bellows
box(w, 'ʃ', 22, 6, 24, 6)        # the charcoal vine, east to the smelters
box(w, ',', 0, 11, 24, 11)       # the road, from Hemp Hollow on to Bellows
box(w, ',', 15, 3, 15, 10)       # up to the stair
put(w, ',', (16, 3))
box(w, ',', 0, 4, 14, 4)         # the back road from Hemp Hollow, to the stair
box(w, 'w', 0, 13, 24, 14)       # the Rust Brook
put(w, '╬', (7, 13), (7, 14))    # bridged for the road south
put(w, ',', (7, 12), (7, 15))
put(w, 'Ɱ', (5, 6), (10, 6))     # two charcoal mounds
put(w, 'p', (4, 6), (6, 6), (9, 6), (11, 6))
put(w, 'o', (5, 7), (10, 7))
put(w, 'ɭ', (3, 5), (3, 7), (11, 5))   # logs waiting for them,
put(w, 'ʉ', (4, 5), (6, 7), (11, 7))   # turf to cover them,
put(w, 'ʊ', (8, 8))                    # and the burners' shelter
put(w, 'Ɯ', (19, 9))             # the stonemason's bench
put(w, 'p', (18, 9), (20, 9))
put(w, 'o', (19, 10))
put(w, '▪', (17, 8), (18, 8), (20, 8), (21, 8), (21, 9))   # cut stone, leaving each pad a way in
put(w, '∴', (21, 10))
put(w, '8', (17, 3))             # a stone lantern at the stair's foot
put(w, 'Ƥ', (2, 3), (8, 3), (12, 3), (13, 8), (3, 9), (17, 5), (20, 3), (1, 7))
put(w, 'ƥ', (6, 3), (14, 6))
put(w, 'ʝ', (8, 9), (11, 9), (17, 7))
put(w, 's', (14, 10))
put(w, 'Y', (0, 8), (23, 12))
put(w, '¥', (4, 13), (17, 13), (18, 13), (20, 13), (1, 14), (2, 14), (3, 14), (4, 14), (9, 14), (20, 14))
assert w[6][21] == '.', "the charcoal vine's foot is open ground"
assert w[4][0] == ',' and w[4][14] == ',' and w[4][15] == ',', 'the back road meets the stair path'

for name, g in [('cliff_lift', c), ('copper_cut', k), ('hemp_hollow', h), ('charcoal_wood', w)]:
    rows = [''.join(r) for r in g]
    assert all(len(r) == W for r in rows)
    (DATA / f'{name}.map').write_text('\n'.join(rows) + '\n', encoding='utf-8', newline='\n')
    print(name)
    print('\n'.join(rows))
