"""Builds data/pinewater.map: Pinewater rearranged with a cloth chain along each street (Henry
picked this arrangement, B, on 2026-11-06). Field, carding bench, spinnery, loom house and cloth
store run west to east along the north road, and again along the south street, so the two chains
never cross or share a door. Between the streets: the gathering hall by the first field, where
everyone starts, homes and a green. Old Spoke's yard, the wagon bench by its gate, Old Sawgrass's
house and the builders' bench keep their tiles. The tests walk a frozen copy of the village as it
was (data/tests/pinewater.map); `Game::live` builds this one.

    python tools/maps/pinewater_map.py
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
    fill(x0, y0, x1, y1, 'W')
    fill(x0 + 1, y0 + 1, x1 - 1, y1 - 1, '_')


def rows(x0, y0, *lines):
    for dy, line in enumerate(lines):
        for dx, c in enumerate(line):
            if c != ' ':
                put(x0 + dx, y0 + dy, c)


# The hedge round the village, open where the north road leaves for the ferry and the south road
# for the bridge.
fill(0, 0, W - 1, 0, '#')
fill(0, H - 1, W - 1, H - 1, '#')
fill(0, 0, 0, H - 1, '#')
fill(W - 1, 0, W - 1, H - 1, '#')

# --- The streets. ---
fill(1, 9, 49, 9, ',')                     # the north road, out east to the ferry
fill(1, 17, 47, 17, ',')                   # the south street
fill(16, 10, 16, 16, ',')                  # a lane between them by the green
fill(36, 10, 36, 16, ',')                  # and another past the well
fill(36, 18, 36, 26, ',')                  # on south between the second spinnery and loom house
fill(36, 27, 49, 27, ',')                  # the south road, out east to the bridge

# --- The north street: the first chain, as every player first learns it. ---
for y in (4, 6, 8):
    fill(3, y, 8, y, 'f')                  # the cotton field
put(2, 8, '1')                             # Mogg
put(11, 9, '2')                            # Bruna, by the chute
put(12, 8, 'C')                            # the cotton bin, its chute on the road
put(12, 9, 'm')
many('g', (13, 8), (15, 8))
put(13, 9, 'p')                            # the carding bench
put(14, 9, 'B')
put(15, 9, 'o')
fill(11, 10, 15, 10, ',')
building(27, 2, 35, 8)                     # the spinnery, its wide door on the road
many('D', (31, 8), (32, 8))
put(31, 6, 'S')
put(31, 7, 'p')
put(32, 7, 'o')
many('g', (32, 5), (33, 5))
building(38, 2, 46, 8)                     # the loom house beside it
many('D', (42, 8), (43, 8))
put(41, 7, 'L')
put(42, 7, 'p')
put(43, 7, 'o')
put(40, 7, 'g')
put(47, 8, 'T')                            # the cloth store at the street's end
put(48, 8, 's')                            # TANGLEMERE
many('p', (45, 10), (46, 10))              # the gate bench, where thread is twisted into rope
put(47, 10, 'B')
put(47, 11, 'o')

# --- Between the streets: the gathering hall by the first field, homes, the green, a well. ---
building(2, 11, 10, 15)
put(6, 11, 'D')
put(6, 10, ',')
many('a', (4, 13), (6, 13), (8, 13))
fill(18, 2, 20, 4, 'Ȣ')
fill(19, 5, 19, 8, ',')
fill(21, 11, 23, 13, 'Ȣ')
fill(22, 14, 22, 16, ',')
fill(30, 12, 31, 13, '>')                  # the well
fill(38, 12, 40, 14, 'Ȣ')
fill(39, 15, 39, 16, ',')

# --- The south street: the second chain. ---
for y in (18, 20):
    fill(2, y, 7, y, 'f')                  # the second field
put(11, 18, 'C')
put(11, 17, 'm')
many('g', (12, 18), (14, 18))
put(12, 17, 'p')                           # its carding bench
put(13, 17, 'B')
put(14, 17, 'o')
building(27, 18, 35, 24)                   # the second spinnery
many('D', (31, 18), (32, 18))
put(31, 20, 'S')
put(31, 19, 'p')
put(32, 19, 'o')
many('g', (32, 21), (33, 21))
building(38, 18, 46, 24)                   # the second loom house
many('D', (42, 18), (43, 18))
put(41, 19, 'L')
put(42, 19, 'p')
put(43, 19, 'o')
put(40, 19, 'g')
put(47, 18, 'T')                           # the second cloth store

# --- Old Spoke's yard, the wagon bench by its gate and a home: as they were. ---
rows(15, 19,
     'xxxxxxxxx',
     'xiWWWW..x',
     'xdWWWWG.x',
     'xzG.G...Q',
     'x.......Q',
     'x..ppp..x',
     ',..pJo.bx',
     'x..pN...x',
     'xy.....ti',
     'xxxxxxxxx')
fill(9, 18, 9, 25, ',')                    # the lane down to the yard's gate
fill(9, 25, 14, 25, ',')
rows(6, 27, 'pppJppp', ' ggog')
fill(2, 22, 4, 24, 'Ȣ')
fill(3, 25, 8, 25, ',')

# --- Old Sawgrass's house, the builders' bench, the pumpkins and the town's wagon: as they were. ---
rows(25, 27,
     'WWWWWW',
     'W_c__D  pppZgg',
     'W____W  pppog',
     'WWWWWW              UP')
many('v', (41, 29), (42, 29), (43, 29))    # the pumpkins, behind the little flowerbed

# --- Dressed after the image model's variants Henry picked (2026-11-06): north-west 1, north-east 1,
# south-west 2, south-east 1. Every bench, bin and door keeps two clear tiles round it; berry bushes
# became clipped shrubs, so nothing looks pickable; a gateway stands over a lane, since one can't
# span a road running across the screen.

# North-west: the field fenced, open towards the bin, a scarecrow; pines and shrubs along the
# hedge; a cottage garden in flower; a bench and lantern by the hall; a great tree on the green.
many('Y', (1, 1), (3, 1), (1, 2), (1, 4), (1, 5), (1, 6), (9, 1), (10, 1), (11, 1))
many('ŏ', (2, 1), (1, 3), (1, 8))
fill(2, 2, 10, 2, 'x')
fill(2, 3, 2, 7, 'x')
fill(10, 3, 10, 5, 'x')
put(9, 5, 'ȥ')
many('x', (16, 1), (17, 1), (21, 1), (22, 1), (23, 1))
fill(16, 2, 16, 6, 'x')
fill(23, 2, 23, 6, 'x')
many('x', (17, 6), (18, 6), (20, 6), (21, 6), (22, 6))
many('*', (17, 3), (17, 4), (17, 5), (18, 5), (22, 2), (22, 3), (21, 5), (22, 5))
put(21, 4, 'i')
put(9, 10, '!')
many('-', (20, 8), (21, 8))                # a bench looks south, so over the road, not at the hall's wall
fill(13, 12, 15, 14, '(')
many('-', (11, 14), (12, 14))
put(20, 12, '+')
many('*', (19, 13), (20, 13))

# North-east: shade trees and flowers along the hedge, rain barrels and lanterns between the
# workshops, a bench, the well and a trough, the cottage in a fenced flower garden.
fill(25, 1, 26, 2, 'E')
fill(25, 4, 26, 5, 'E')
put(25, 7, '*')
put(26, 8, 'i')
many('*', (27, 1), (28, 1), (44, 1), (45, 1), (47, 3), (48, 3), (47, 4), (48, 4))
fill(47, 1, 48, 2, 'E')
many('!', (36, 3), (37, 5))
many('i', (36, 7), (37, 8))
many('-', (30, 11), (31, 11))
many('ȣ', (32, 13), (33, 13))
fill(37, 10, 42, 10, 'x')
fill(37, 11, 37, 15, 'x')
fill(42, 11, 42, 15, 'x')
many('x', (38, 15), (40, 15), (41, 15))
fill(41, 11, 41, 14, '*')                  # beside the cottage, clear of its roof
fill(47, 14, 48, 15, 'E')

# South-west: the field fenced below and beside, open to the street and towards the bin, a
# scarecrow in the fence; a barrel, flower boxes and a woodpile by the cottage; the well and a
# bench on the green; pines in the corner and shrubs along the hedge below the yard.
fill(1, 18, 1, 21, 'x')
fill(1, 21, 8, 21, 'x')
fill(8, 19, 8, 20, 'x')
put(6, 21, 'ȥ')
put(1, 24, 'i')
many('*', (5, 24), (6, 24), (7, 24), (1, 26))
many('+', (2, 26), (3, 26))
fill(12, 22, 13, 23, '>')
many('-', (11, 24), (12, 24))
many('Y', (1, 27), (2, 28), (1, 29), (3, 29), (22, 29), (23, 30))
many('ŏ', (16, 30), (17, 30), (19, 30), (20, 30))

# South-east: woodpiles and a bench by the second spinnery, a washing line, crates and barrels by
# the loom house, a timber gateway and lanterns where the lane meets the south road, flowers by
# the hedge.
many('+', (26, 22), (26, 23), (26, 24), (27, 25), (28, 25))
many('-', (29, 25), (30, 25))
many(']', (39, 25), (40, 25), (41, 25), (42, 25), (43, 25), (44, 25))
put(46, 25, '[')
put(47, 25, 'i')
put(48, 21, '[')
many('i', (48, 22), (48, 23))
many('Ȥ', (35, 26), (37, 26))
many('!', (34, 26), (38, 26))
many('*', (48, 25), (48, 26), (48, 28), (48, 29), (48, 30), (42, 30), (43, 30))

out = Path(__file__).resolve().parents[2] / 'data' / 'pinewater.map'
out.write_bytes(('\n'.join(''.join(r) for r in g) + '\n').encode('utf-8'))
print('wrote', out)
