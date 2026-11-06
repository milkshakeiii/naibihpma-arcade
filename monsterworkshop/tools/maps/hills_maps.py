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

for name, g in [('cliff_lift', c), ('copper_cut', k)]:
    rows = [''.join(r) for r in g]
    assert all(len(r) == W for r in rows)
    (DATA / f'{name}.map').write_text('\n'.join(rows) + '\n', encoding='utf-8', newline='\n')
    print(name)
    print('\n'.join(rows))
