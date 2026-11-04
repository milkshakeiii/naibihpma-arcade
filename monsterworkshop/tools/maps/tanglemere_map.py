"""Builds data/tanglemere.map from the image model's variants Henry picked (2026-11-04): the dye
house's corner from variant 3 (tools/briefs/tangle_nw.txt), the indigo house, cloth hall and
outfitter from variant 2 (tangle_ne), the kiln and the lake shore from variant 1 (tangle_sw),
the indigo field from variant 1 (tangle_se). Function wins where they differ: the road keeps its
row, the well stands off it, and the path south meets the indigo field's path. The outfitter's
racks and Old Thimble are placed by the region's data in src/world.rs, not here.

    python tools/maps/tanglemere_map.py
"""
from pathlib import Path

W, H = 50, 32
g = [['.'] * W for _ in range(H)]


def put(x, y, c):
    assert 0 <= x < W and 0 <= y < H, (x, y)
    g[y][x] = c


def fill(x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, c)


def many(c, *spots):
    for x, y in spots:
        put(x, y, c)


def building(x0, y0, x1, y1):
    """Walls round a plank floor."""
    fill(x0, y0, x1, y1, 'W')
    fill(x0 + 1, y0 + 1, x1 - 1, y1 - 1, '_')


# The hedge round the town; the road in from the west and across.
fill(0, 0, W - 1, 0, '#')
fill(0, H - 1, W - 1, H - 1, '#')
fill(0, 0, 0, H - 1, '#')
fill(W - 1, 0, W - 1, H - 1, '#')
fill(0, 9, 47, 9, ',')

# --- Northwest: arriving from Pinewater, the dye house, the well square. ---
many('Y', (3, 1), (5, 1), (2, 2), (2, 3), (3, 3), (4, 3), (7, 3), (1, 6), (2, 6), (14, 12))
fill(9, 2, 12, 2, 'ð')                     # cloth drying on two lines
fill(8, 5, 12, 5, 'ð')
many('+', (13, 3), (13, 4))                # firewood for the vats
building(14, 2, 21, 7)                     # the dye house
put(17, 7, 'D')
put(20, 3, 'g')
many('p', (16, 4), (18, 4))
put(17, 4, 'V')
put(20, 4, 'o')
put(15, 6, '+')
put(19, 6, 'ö')                            # a tub of madder steeping
put(20, 6, 'i')
put(17, 8, ',')
many('@', (22, 2), (22, 3), (22, 5), (8, 7), (10, 11), (15, 8))
put(22, 7, 'ö')
put(2, 8, 's')                             # the signpost to Pinewater
many('π', (5, 8), (7, 8))                  # the town's timber gate, its beam overhead
# The well square: cobbles round the well, the road running across it.
many('░', (9, 8), (10, 8), (13, 8), (10, 10), (11, 10), (12, 10))
fill(9, 9, 13, 9, '░')
fill(11, 7, 12, 8, '>')
many('*', (8, 8), (10, 7), (13, 7), (8, 10), (9, 11), (12, 11), (14, 10))
fill(16, 10, 17, 10, '-')
many('!', (18, 10), (6, 14))
many('ŏ', (14, 8), (19, 10), (20, 10), (21, 10), (8, 15), (9, 15))
many('e', (4, 12), (10, 13), (5, 15))      # the elder bushes
fill(7, 10, 7, 16, ',')                    # the path south to the kiln

# --- Northeast: the indigo house, the cloth hall, a little park, the outfitter. ---
building(25, 2, 33, 6)                     # the indigo house
put(29, 6, 'D')
put(29, 3, 'I')
fill(27, 4, 31, 4, 'p')
put(31, 5, 'o')
many(',', (29, 7), (29, 8))
many('Y', (34, 2), (34, 3), (34, 4))
many('i', (34, 5), (34, 6), (30, 7))
fill(25, 7, 28, 8, 'ø')                    # barrels of indigo and madder
fill(31, 7, 34, 7, 'ð')
fill(31, 8, 34, 8, 'ð')
building(35, 2, 45, 6)                     # the cloth hall
put(39, 6, 'D')
put(39, 4, 'T')
many('¶', (36, 3), (37, 3), (38, 3), (44, 3))
fill(40, 3, 41, 3, '÷')
many('¤', (42, 2), (43, 2))
fill(43, 5, 44, 5, '-')
fill(36, 8, 37, 8, '-')
many(',', (39, 7), (39, 8))
fill(42, 7, 43, 8, '>')
fill(36, 10, 36, 15, ',')                  # the path south to the indigo field
# The park: pines, lanterns, benches, standing stones, flower beds.
many('Y', (25, 10), (26, 10), (33, 10), (34, 10), (25, 11), (34, 11), (26, 13), (31, 13), (25, 14), (26, 14), (25, 15), (26, 15), (27, 15), (32, 15))
many('!', (28, 10), (31, 10), (28, 14), (31, 14))
fill(29, 10, 30, 10, '-')
fill(29, 14, 30, 14, '-')
many('@', (28, 12), (30, 12), (29, 13))
many('*', (26, 11), (33, 11), (25, 12), (26, 12), (33, 12), (34, 12), (25, 13), (33, 13), (34, 13), (33, 14))
building(39, 10, 47, 15)                   # the outfitter
put(41, 10, 'D')
many('¬', (40, 10), (42, 10))              # the awning, over the door
put(43, 11, 'ì')
put(44, 11, 'ï')
many('¶', (45, 11), (46, 11))
fill(45, 12, 46, 12, '÷')
put(38, 11, '§')
many('Y', (48, 2), (48, 5), (48, 7), (48, 11), (48, 12), (48, 14))

# --- Southwest: the kiln, and the lake's west shore. ---
building(3, 17, 11, 24)                    # the kiln house, a door at each end
many('D', (7, 17), (7, 24))
put(5, 20, 'o')
many('p', (6, 21), (8, 21))
put(7, 21, 'K')
many('+', (4, 18), (5, 18), (4, 23), (5, 23))
many('•', (9, 18), (4, 22))
many('i', (10, 18), (10, 23))
many('º', (9, 19), (9, 23))
put(6, 25, '!')
fill(7, 25, 12, 25, ',')                   # round the kiln and out to the jetty
fill(12, 17, 12, 24, ',')
fill(13, 23, 15, 23, ',')
many('q', (13, 19), (14, 21), (14, 25))    # the ramie
lake = {16: (23, 24), 17: (21, 24), 18: (20, 24), 19: (18, 24), 20: (17, 24), 21: (16, 24), 22: (16, 24), 23: (16, 24),
        24: (18, 24), 25: (18, 24), 26: (19, 24), 27: (20, 24), 28: (23, 24)}
for y, (x0, x1) in lake.items():
    fill(x0, y, x1, y, 'w')
fill(16, 23, 18, 23, '╪')                  # the jetty
fill(18, 22, 19, 22, 'ß')                  # a rowing boat tied up beside it
many('¥', (21, 17), (18, 19), (17, 20), (16, 21), (16, 22), (20, 27), (21, 27), (22, 27))
many('ò', (18, 20), (19, 20), (19, 21), (20, 19), (20, 24), (21, 24), (21, 25))
fill(20, 28, 21, 28, '-')
put(16, 26, '@')
many('Y', (15, 24), (15, 25), (16, 25), (13, 25), (15, 26), (15, 27), (16, 27), (5, 27), (6, 27), (9, 27), (10, 27), (11, 27),
     (6, 28), (13, 28), (4, 29), (6, 29), (8, 29), (12, 29), (16, 29), (17, 29), (18, 29), (18, 27), (19, 28),
     (4, 30), (8, 30), (9, 30), (13, 30), (17, 30), (19, 30), (23, 30), (24, 30))

# --- Southeast: the lake's east shore, the jetty, the indigo field. ---
lake = {16: 30, 17: 32, 18: 33, 19: 33, 20: 34, 21: 34, 22: 34, 23: 34, 24: 34, 25: 33, 26: 33, 27: 32, 28: 31}
for y, x1 in lake.items():
    fill(25, y, x1, y, 'w')
fill(32, 23, 34, 23, '╪')                  # the jetty
fill(36, 16, 36, 23, ',')                  # the path down from the cloth hall
many(',', (35, 23), (37, 23), (38, 23))
# The field, fenced, its gate on the west; the shed outside its corner.
fill(41, 18, 48, 18, 'x')
fill(48, 18, 48, 26, 'x')
fill(39, 26, 48, 26, 'x')
many('x', (39, 22), (39, 24), (39, 25), (40, 22), (41, 22))
fill(41, 18, 41, 22, 'x')
fill(39, 19, 40, 20, 'ħ')
for y in (19, 21, 23, 25):
    for x in (43, 45, 47):
        put(x, y, 'n')                     # the indigofera
put(44, 22, 'ŝ')
fill(40, 24, 41, 24, '-')
fill(40, 25, 41, 25, '≈')
many('@', (34, 16), (34, 17), (35, 18), (36, 25), (37, 25), (36, 26))
many('¥', (32, 17), (34, 21), (34, 24), (33, 26), (29, 28), (30, 28), (31, 28))
many('ò', (32, 20), (33, 20), (33, 21), (33, 25), (31, 26), (32, 26))
many('Y', (48, 16), (48, 17), (35, 27), (38, 27), (47, 27), (32, 28), (33, 28), (36, 28), (37, 28), (38, 28), (44, 28),
     (33, 29), (34, 29), (39, 29), (44, 29), (47, 29), (48, 29), (26, 30), (27, 30), (28, 30), (32, 30), (38, 30), (39, 30), (41, 30))

out = Path(__file__).resolve().parents[2] / 'data' / 'tanglemere.map'
out.write_bytes(('\n'.join(''.join(r) for r in g) + '\n').encode('utf-8'))
print('wrote', out)
