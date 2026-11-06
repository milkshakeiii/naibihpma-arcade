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
put(w, 'ȵ', (7, 7), (12, 7))     # each mound's choice board: charcoal, or pitch from pine
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

# --- Verdigris, the hills' town, from variant 2 (tools/briefs/hills_verdigris.txt): hedged round
# as a village, but open on the west, where the Cliff Lift's own hedge stands; the high road through
# it east and west; a parapet along the cliff's edge, open where the Stone Stair comes up. The
# smithy at the west end, a few steps from where the lift vine sets goods down; the smith's forge
# across the road; the square with its well, flower beds and notice board; the gathering hall; two
# copper-roofed cottages north of the road (the model stood them on the road; they moved up a row).
v = blank()
box(v, '.', 0, 1, 0, 15)
box(v, ',', 0, 4, 24, 4)         # the high road
put(v, ',', (15, 15), (16, 15))  # the top of the Stone Stair
box(v, '░', 15, 11, 16, 14)      # up from it to the square
box(v, 'ɐ', 0, 14, 14, 14)       # the parapet, along the cliff's edge, into the corner
put(v, '#', (0, 15))             # and the hedge under it
box(v, 'ɐ', 17, 14, 23, 14)
box(v, 'W', 1, 6, 10, 11)        # the smithy, a door to the road and one behind
box(v, '_', 2, 7, 9, 10)
put(v, 'D', (5, 6), (6, 11))
put(v, ',', (5, 5), (6, 12))
box(v, ',', 6, 13, 14, 13)       # the path round from its back door
put(v, 'ʍ', (5, 8))
put(v, 'p', (3, 8), (4, 8), (6, 8), (7, 8), (3, 9))
put(v, 'o', (5, 9))
put(v, 'g', (2, 7), (9, 7), (2, 10), (9, 10))
put(v, 'ȵ', (8, 9))              # its choice board: what it makes when the goods could make several
put(v, 'ʍ', (5, 2))              # the smith's own forge, out in the open
put(v, 'p', (4, 2), (6, 2))
put(v, 'o', (5, 3))
put(v, 'G', (2, 2), (8, 2))      # his copper and his charcoal
put(v, 'b', (8, 3))              # his order board
put(v, 'N', (4, 1))
box(v, ':', 3, 5, 4, 5)          # the quenching trough
put(v, 'ǥ', (8, 5), (4, 12))     # grindstones
box(v, ':', 7, 12, 8, 12)        # a second trough, out back
put(v, 'ŧ', (9, 12))
box(v, '░', 11, 5, 17, 10)       # the square
box(v, '>', 13, 6, 14, 7)        # its well
put(v, '*', (11, 6), (11, 7), (17, 6), (17, 7), (17, 9), (17, 10), (11, 10), (12, 10), (13, 10), (14, 10))
box(v, 'ņ', 13, 9, 14, 9)        # the notice board
box(v, 'W', 18, 6, 23, 11)       # the gathering hall
box(v, '_', 19, 7, 22, 10)
put(v, 'D', (18, 8))
put(v, 'a', (20, 8), (21, 9))
box(v, 'ȼ', 12, 1, 14, 3)        # two cottages,
box(v, 'ȼ', 18, 1, 20, 3)
box(v, ']', 15, 2, 17, 2)        # washing between them,
box(v, '-', 15, 3, 16, 3)        # and a bench
put(v, 'i', (11, 3), (21, 3))
box(v, '-', 12, 12, 13, 12)      # a bench looking out
put(v, 's', (23, 3), (18, 13))
put(v, '!', (10, 5), (19, 5))
put(v, 'Y', (2, 1), (0, 1), (23, 1), (22, 1), (1, 12), (2, 12), (1, 13), (23, 12), (22, 12), (23, 13))
put(v, '@', (3, 1), (2, 3), (0, 3), (21, 12), (22, 13), (3, 13))
assert v[2][1] == '.' and v[2][3] == '.', "the smith's copper is pushed from the west"
assert v[13][6] == ',', "the smithy's back door has a way out"

# --- Bellows, the smelters' town, from variant 1 (tools/briefs/hills_bellows.txt): below the
# Copper Cut's cliff, whose ore vine comes down it onto a path to the road, and Charcoal Wood's
# ridge, whose charcoal vine comes across a spur of rock, both setting their goods down in the
# walled smelters' yard. Pines on the heights in the corner; the assay office against the cliff,
# the miners' outfitter below it. Hedged round as a town, but where the road, the brook and the
# trunk road south come in.
b = blank()
box(b, 'Λ', 8, 0, 24, 2)         # the heights' edge, under the Copper Cut
box(b, 'Λ', 2, 2, 7, 2)
box(b, '#', 0, 0, 7, 1)          # a dark pine wood on the heights,
put(b, 'Y', (1, 1), (4, 1), (6, 1), (0, 2), (1, 2))   # its nearest trees standing out
box(b, 'ʃ', 16, 0, 16, 2)        # the ore vine, coming down the cliff
box(b, 'Λ', 0, 3, 3, 9)          # the ridge's spur, under Charcoal Wood
box(b, 'ʃ', 0, 6, 3, 6)          # the charcoal vine, coming across it
box(b, '▒', 4, 3, 15, 9)         # the smelters' yard,
box(b, 'ǁ', 3, 10, 15, 10)       # walled from the road,
put(b, '▒', (7, 10), (9, 10))    # with two gates
box(b, ',', 16, 3, 16, 10)       # the path from the ore vine down to the road
put(b, '.', (0, 10), (1, 10), (2, 10), (0, 12))
box(b, ',', 0, 11, 24, 11)       # the road, from Charcoal Wood on east
box(b, 'w', 0, 13, 24, 14)       # the Rust Brook
put(b, '╬', (16, 13), (16, 14))  # bridged for the trunk road south
put(b, ',', (16, 12), (16, 15))
put(b, 'ɕ', (7, 5), (11, 8))     # two smelters, nothing above them
put(b, 'p', (5, 5), (6, 5), (8, 5), (9, 5), (8, 6))
put(b, 'o', (7, 6))
put(b, 'p', (9, 8), (10, 8), (12, 8), (13, 8), (12, 9))
put(b, 'o', (11, 9))
box(b, 'w', 12, 4, 13, 5)        # the stirring pond, for wrought iron
put(b, 'ʚ', (10, 4), (11, 4), (11, 5), (12, 6), (13, 6))   # slag
box(b, ':', 14, 7, 15, 7)        # a quenching trough
put(b, 'ʇ', (4, 8), (4, 9))      # tongs and rakes
put(b, '!', (15, 3), (15, 9))
box(b, 'W', 18, 2, 23, 6)        # the assay office, against the cliff
box(b, '_', 19, 3, 22, 5)
put(b, 'D', (18, 4))
put(b, 'ȋ', (21, 4))
put(b, 'ʡ', (19, 5))             # its scales
box(b, 'W', 17, 7, 23, 10)       # the miners' outfitter
box(b, '_', 18, 8, 22, 9)
put(b, 'D', (20, 10))
put(b, 'ɧ', (18, 9), (22, 9))    # its shelves of gear, along the back
put(b, 's', (2, 12))
put(b, '¥', (5, 13), (6, 13), (12, 13), (14, 13), (18, 13), (19, 13), (21, 13), (22, 13), (14, 14), (15, 14), (20, 14), (21, 14))
assert b[3][16] == ',' and b[6][4] == '▒', "both vines set their goods down on open ground"
assert all(b[4][x] != 'p' for x in (7, 11)) and b[7][11] == '▒', 'nothing sits above a smelter'

# --- Iron Brook, from the second round's variant 3 (tools/briefs/hills_iron.txt): the first
# round drew the brook rust-red, which Henry found gruesome, so the second kept its water clear and
# its banks green. The Rust Brook comes down from Hemp Hollow on the columns it leaves the hollow
# by, wanders, and turns west out of the hills; iron in seams along its banks; bright pines on the
# far bank; a sawyard at the road's end, its choice board saying planks or rollers. Open to Hemp
# Hollow above and to the Great Furnace east; hedged where the hills end.
i = blank()
box(i, '.', 1, 0, 24, 0)
box(i, '.', 24, 1, 24, 14)
box(i, ',', 5, 0, 5, 9)          # the road down from Hemp Hollow,
box(i, ',', 6, 9, 13, 9)         # to the sawyard, and on to the brook,
box(i, 'w', 12, 0, 13, 1)        # the Rust Brook, down from the hollow,
box(i, 'w', 13, 2, 14, 4)        # wandering,
box(i, 'w', 14, 5, 15, 9)
box(i, 'w', 13, 10, 14, 11)
box(i, 'w', 0, 12, 13, 13)       # and away west out of the hills
put(i, '╬', (14, 9), (15, 9))    # bridged to the far bank's seams and pines
put(i, 'ɨ', (12, 2), (15, 3), (16, 5), (13, 7), (16, 8), (12, 10), (15, 11))   # iron in its banks
for x in (2, 4, 6, 8, 10):
    put(i, 'ɨ', (x, 11))
box(i, 'Ɖ', 7, 3, 9, 4)          # the sawyers' shed
put(i, 'ɮ', (10, 4), (7, 5))     # logs waiting,
put(i, 'ʓ', (8, 5), (9, 5))      # sawdust,
put(i, 'd', (10, 5))             # sawn planks,
put(i, 'z', (9, 6))              # a sawhorse
put(i, 'ʑ', (8, 7))              # the sawpit
put(i, 'p', (7, 7), (9, 7))
put(i, 'o', (8, 8))
put(i, 'ȵ', (10, 7))             # its choice board: planks, or a roller
put(i, 'º', (12, 7), (11, 8), (11, 10))
put(i, 'ɵ', (14, 3), (13, 4), (14, 7))   # stones in the brook,
put(i, 'ɤ', (15, 7))             # a fallen log across it,
put(i, '¥', (12, 0), (13, 2), (14, 6), (15, 5), (14, 10), (5, 12), (1, 13), (4, 13), (11, 13))
put(i, 'ɸ', (16, 3), (13, 5), (16, 10), (11, 11), (12, 8), (17, 7))   # ferns
put(i, 'Ƥ', (17, 1), (19, 3), (22, 2), (21, 5), (23, 4), (19, 7), (22, 8), (20, 10), (17, 11), (22, 12))
put(i, 'Y', (18, 14), (20, 14), (23, 14), (24, 13), (1, 14))
put(i, 'Y', (20, 0), (24, 0), (24, 3), (20, 6), (21, 7), (24, 5), (24, 9), (23, 10), (19, 12), (18, 13), (24, 11))   # the wood
put(i, '@', (3, 4), (17, 5), (23, 8), (21, 13))
put(i, 's', (4, 2))
assert i[0][12] in 'w¥' and i[0][13] in 'w¥', "the brook comes in where Hemp Hollow's goes out"

for name, g in [('cliff_lift', c), ('copper_cut', k), ('hemp_hollow', h), ('charcoal_wood', w), ('verdigris', v), ('bellows', b), ('iron_brook', i)]:
    rows = [''.join(r) for r in g]
    assert all(len(r) == W for r in rows)
    (DATA / f'{name}.map').write_text('\n'.join(rows) + '\n', encoding='utf-8', newline='\n')
    print(name)
    print('\n'.join(rows))
