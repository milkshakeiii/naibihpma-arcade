"""Generates the site book (src/sites/ch*.txt): maps, goals and the bot's build plans.

Maps are built from a few terrain helpers so wide sites stay consistent, and plans refer to the
same coordinates. Run from the birdfortress folder:  python tools/sites.py
Then check with:  cargo run --release -- --check
"""

import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "src", "sites")

SOLID = "#%X"
STANDABLE_FLOOR = "#%X=-"


class Map:
    def __init__(self, w, h=24):
        self.w, self.h = w, h
        self.g = [["." for _ in range(w)] for _ in range(h)]

    def rect(self, x0, y0, x1, y1, ch):
        for y in range(max(0, y0), min(self.h, y1 + 1)):
            for x in range(max(0, x0), min(self.w, x1 + 1)):
                self.g[y][x] = ch

    def ground(self, x0, x1, top, ch="#"):
        """Solid from row `top` to the bottom, columns x0..x1."""
        self.rect(x0, top, x1, self.h - 1, ch)

    def put(self, x, y, ch):
        assert 0 <= x < self.w and 0 <= y < self.h, (x, y)
        self.g[y][x] = ch

    def hline(self, x0, x1, y, ch):
        self.rect(x0, y, x1, y, ch)

    def vline(self, x, y0, y1, ch):
        self.rect(x, y0, x, y1, ch)

    def top(self, x, below=0):
        """The first solid or platform row at or below `below` in column x."""
        for y in range(below, self.h):
            if self.g[y][x] in STANDABLE_FLOOR:
                return y
        return self.h

    def floor(self, x, below=0):
        """The standing row on the first floor at or below `below`."""
        return self.top(x, below) - 1

    def on(self, x, ch, below=0):
        """Puts a thing standing on the floor in column x."""
        y = self.floor(x, below)
        self.put(x, y, ch)
        return (x, y)

    def fortress(self, x, w, h, below=0):
        y = self.floor(x, below)
        for xx in range(x, x + w):
            assert self.floor(xx, below) == y, f"fortress floor uneven at {xx}"
        self.rect(x, y - h + 1, x + w - 1, y, "F")
        return (x, y - h + 1)

    def text(self):
        return "\n".join("".join(r) for r in self.g)


def spot(m, x, below=0, w=2):
    """Top-left of a w-wide, 2-tall building standing on the floor at column x."""
    y = m.floor(x, below)
    for xx in range(x, x + w):
        assert m.floor(xx, below) == y, f"uneven floor under building at {x}"
    return (x, y - 1)


class Plan:
    def __init__(self, m):
        self.m = m
        self.steps = []

    def home(self, kind, x, below=0):
        bx, by = spot(self.m, x, below)
        self.steps.append(f"{kind} {bx} {by}")
        return self

    def burrow(self, x, below=0):
        return self.home("burrow", x, below)

    def nest(self, x, below=0):
        return self.home("nest", x, below)

    def lodge(self, x, below=0):
        return self.home("lodge", x, below)

    def work(self, kind, x, species, below=0):
        bx, by = spot(self.m, x, below)
        self.steps.append(f"{kind} {bx} {by} {species}")
        return self

    def camp(self, pos, species="tribble"):
        self.steps.append(f"camp {pos[0]} {pos[1]} {species}")
        return self

    def garden(self, x, below=0):
        bx, by = spot(self.m, x, below, 1)
        self.steps.append(f"garden {bx} {by + 1}")
        return self

    def raw(self, s):
        self.steps.append(s)
        return self

    def wait(self, what, n):
        self.steps.append(f"wait {what} {n}")
        return self

    def text(self):
        return "; ".join(self.steps)


def load_best():
    """Best plans found by `--opt`, kept in tools/best_plans.txt as id, day, plan."""
    best = {}
    path = os.path.join(HERE, "best_plans.txt")
    if os.path.exists(path):
        for line in open(path, encoding="utf-8"):
            parts = line.rstrip(chr(10)).split(chr(9))
            if len(parts) == 3:
                best[parts[0]] = (int(parts[1]), parts[2])
    return best


BEST = load_best()


def medals_for(day):
    """Gold is within a tenth of the optimiser's best; silver and bronze are kinder."""
    return [math.ceil(day * 1.1), math.ceil(day * 1.4), math.ceil(day * 1.9)]


class Site:
    def __init__(self, sid, name, m):
        self.sid, self.name, self.m = sid, name, m
        self.keys = []
        self.plans = []

    def key(self, k, v):
        self.keys.append(f"{k} {v}")
        return self

    def plan(self, p):
        self.plans.append(p.text())
        return self

    def text(self):
        out = [f"@ {self.sid} {self.name.upper()}"]
        out += self.keys
        plans = list(self.plans)
        if self.sid in BEST:
            # The optimiser's best plan comes first: it sets the medals and the tests run it.
            day, best = BEST[self.sid]
            plans.insert(0, best)
            out.append("medals " + " ".join(str(d) for d in medals_for(day)))
        for i, p in enumerate(plans):
            if i > 0:
                out.append("nextplan")
            out.append(f"plan {p}")
        out.append("map")
        out.append(self.m.text())
        out.append("end")
        return "\n".join(out) + "\n"


# ----- Chapter I: Meadow ----------------------------------------------------------------------------


def s1_1():
    m = Map(40)
    m.ground(0, 39, 18)
    m.ground(24, 31, 16)
    m.ground(22, 23, 17)
    m.put(0, 17, "E")
    b = [m.on(7, "b"), m.on(12, "b"), m.on(19, "b"), m.on(28, "b")]
    t = [m.on(33, "t"), m.on(21, "t")]
    m.fortress(35, 4, 3)
    s = Site("1-1", "Settlers", m)
    s.key("seeds", 16).key("species", "TRIBBLE").key("build", "BURROW CAMP")
    s.key("stage", "Cornerstone 6 TWIG").key("goal", "pop 12")
    s.key("intro", "A meadow, a trail and some berry bushes. Tribbles walk in along the trail to any burrow they can reach. Every one of them works.")
    s.key("new", "BURROW CAMP BERRY")
    p = Plan(m)
    p.burrow(3).camp(b[0]).burrow(9).camp(b[1]).burrow(14).camp(t[1]).burrow(16).camp(b[2]).camp(t[0])
    s.plan(p)
    return s

def s1_2():
    m = Map(56)
    m.ground(0, 55, 18)
    # The quarry terrace, one step at a time, and a twig crag beyond it.
    m.ground(36, 39, 17, "%")
    m.ground(40, 49, 16, "%")
    m.ground(50, 51, 14, "%")
    m.ground(52, 55, 12, "%")
    m.put(0, 17, "E")
    b = [m.on(6, "b"), m.on(10, "b"), m.on(15, "b"), m.on(26, "b"), m.on(33, "b"), m.on(41, "b")]
    t = [m.on(53, "t"), m.on(54, "t")]
    q = [m.on(44, "q"), m.on(48, "q")]
    m.fortress(19, 4, 3)
    s = Site("1-2", "Heavy Stone", m)
    s.key("seeds", 24).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE")
    s.key("stage", "Footing 10 STONE 6 TWIG").key("stage", "Rampart 14 STONE").key("goal", "pop 18")
    s.key("challenge", "no APE")
    s.key("intro", "Stone is heavy: a tribble can barely drag it. Dodos and apes manage. Each workplace hires one kind of creature; pick it before you build.")
    s.key("new", "NEST LODGE STORE STONE")
    p = Plan(m)
    p.burrow(2).camp(b[0]).burrow(7).camp(b[1]).nest(11).camp(t[0]).nest(16).camp(b[2])
    p.camp(q[0], "dodo").work("store", 30, "dodo").camp(q[1], "dodo").nest(23).camp(b[3]).burrow(13).nest(37).camp(b[4]).camp(b[5]).camp(t[1]).nest(46)
    s.plan(p)
    return s


def s1_3():
    m = Map(64)
    m.ground(0, 63, 18)
    m.ground(26, 27, 17)
    m.ground(28, 29, 16)
    m.ground(30, 43, 15)
    m.ground(44, 45, 16)
    m.ground(46, 47, 17)
    m.ground(56, 63, 17, "%")
    m.put(0, 17, "E")
    b = [m.on(5, "b"), m.on(12, "b"), m.on(19, "b"), m.on(24, "b"), m.on(48, "b"), m.on(38, "b")]
    t = [m.on(33, "t"), m.on(36, "t"), m.on(41, "t"), m.on(21, "t")]
    q = [m.on(59, "q"), m.on(62, "q")]
    m.fortress(50, 4, 3)
    s = Site("1-3", "Cottages", m)
    s.key("seeds", 30).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER")
    s.key("stage", "Footing 8 STONE 8 TWIG").key("stage", "Hall 10 WICKER 6 STONE")
    s.key("goal", "tier 2 4")
    s.key("challenge", "maxstores 1")
    s.key("intro", "Homes move up when they get what they want. A cottage wants wicker as well as food: it holds more and pays more. Run short and it slips back.")
    s.key("new", "WEAVER WICKER TIERS")
    p = Plan(m)
    p.burrow(2).camp(b[0]).burrow(7).camp(b[1]).burrow(9).camp(t[3]).nest(14).camp(b[2])
    p.work("weaver", 34, "tribble").camp(t[0]).camp(t[1]).burrow(16).camp(b[3]).camp(b[5])
    p.work("store", 52 - 8, "dodo") if False else None
    p.camp(q[0], "dodo").camp(q[1], "dodo").nest(54).camp(b[4]).camp(t[2]).burrow(31).nest(39)
    s.plan(p)
    return s


# ----- Chapter II: Quarry ---------------------------------------------------------------------------


def s2_1():
    m = Map(72)
    # The lowland, the mesa and the quarry hollow.
    m.ground(0, 37, 20)
    m.ground(38, 57, 10, "%")
    for k, x in enumerate(range(58, 66, 2)):
        m.ground(x, x + 1, 12 + 2 * k, "%")
    m.ground(66, 71, 20)
    m.vline(37, 9, 19, "v")
    m.put(0, 19, "E")
    b = [m.on(4, "b"), m.on(9, "b"), m.on(15, "b"), m.on(21, "b"), m.on(30, "b"), m.on(41, "b"), m.on(55, "b")]
    t = [m.on(25, "t"), m.on(33, "t"), m.on(52, "t")]
    q = [m.on(67, "q"), m.on(70, "q")]
    m.fortress(44, 5, 4)
    s = Site("2-1", "The Mesa", m)
    s.key("seeds", 34).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER MASON LADDER")
    s.key("stage", "Footing 10 STONE").key("stage", "Walls 18 BRICK").key("stage", "Tower 10 BRICK 10 WICKER")
    s.key("goal", "pop 26")
    s.key("challenge", "maxstores 2")
    s.key("intro", "The fortress goes up on the mesa. Apes climb vines and ladders with anything. Everyone else climbs a ladder only with nothing or something light.")
    s.key("new", "MASON BRICK LADDER VINE")
    p = Plan(m)
    p.burrow(2).camp(b[0]).burrow(6).camp(b[1]).nest(11).camp(b[2]).lodge(17).lodge(19).camp(b[3]).nest(23)
    p.raw("ladder 66 17").raw("ladder 64 15").raw("ladder 62 13").raw("ladder 60 11").raw("ladder 58 9")
    p.camp(q[0], "ape").camp(b[4]).work("mason", 68, "dodo").camp(q[1], "dodo").lodge(27).raw("ladder 37 9")
    p.work("weaver", 34, "tribble").camp(t[0]).camp(t[1]).burrow(31).nest(13).camp(b[5]).burrow(39).camp(t[2]).nest(53).camp(b[6])
    p.work("weaver", 49, "tribble").burrow(42).lodge(56)
    s.plan(p)
    return s


def steps(m, x0, top0, dx, n, w=2, ch="#"):
    """A staircase of n steps, each w wide: step k has its top at top0 - k*dx."""
    for k in range(n):
        x = x0 + k * w
        m.ground(x, x + w - 1, top0 - k * dx, ch)


def river(m, x0, x1, bank_top, depth=2):
    """A river one row below the banks, so planks laid at bank level make a flat bridge."""
    m.rect(x0, bank_top, x1, m.h - 1, ".")
    m.rect(x0, bank_top + 1, x1, bank_top + depth, "~")
    m.ground(x0, x1, bank_top + depth + 1)


PLANS = {}


def plan_for(sid, m):
    p = Plan(m)
    for st in PLANS.get(sid, "").replace(chr(10), " ").split(";"):
        if st.strip():
            p.raw(st.strip())
    return p


def s2_2():
    m = Map(100)
    m.ground(0, 30, 20)
    m.ground(31, 32, 19)
    m.ground(33, 40, 18)
    m.ground(41, 42, 17)
    m.ground(43, 55, 16)
    m.ground(56, 57, 17)
    m.ground(58, 69, 18)
    m.ground(70, 71, 19)
    m.ground(72, 99, 20)
    m.put(0, 19, "E")
    for x in (4, 10, 15, 35, 52, 62, 75, 96):
        m.on(x, "b")
    for x in (38, 46, 50, 66):
        m.on(x, "t")
    for x in (80, 86, 92):
        m.on(x, "q")
    m.fortress(21, 5, 4)
    s = Site("2-2", "Long Haul", m)
    s.key("seeds", 36).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER MASON")
    s.key("stage", "Footing 16 STONE").key("stage", "Walls 24 BRICK").key("stage", "Keep 12 BRICK 12 WICKER")
    s.key("goal", "pop 36")
    s.key("challenge", "maxstores 3")
    s.key("intro", "The quarries are a long way off. Nobody carries a thing further than 100 ticks of walking. A store told to GET a good sends its carriers to fetch it from other stores.")
    s.key("new", "ORDERS")
    s.plan(plan_for("2-2", m))
    return s


def s2_3():
    m = Map(84)
    m.ground(0, 35, 18)
    # The clay gorge: a sheer west wall, two-high steps up the east side.
    m.ground(36, 45, 22)
    m.ground(46, 47, 20)
    m.ground(48, 67, 18)
    # The lake.
    m.ground(68, 83, 22)
    m.rect(68, 18, 83, 21, "~")
    m.put(0, 17, "E")
    for x in (4, 9, 14, 26, 32, 51, 60):
        m.on(x, "b")
    for x in (21, 54, 63):
        m.on(x, "t")
    for x in (39, 43):
        m.on(x, "c")
    for x in (67, 65):
        m.on(x, "f")
    m.fortress(17, 4, 3)
    s = Site("2-3", "Kiln Country", m)
    s.key("seeds", 40).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN LADDER")
    s.key("stage", "Footing 14 TWIG 6 CLAY").key("stage", "Hall 8 POT 8 WICKER")
    s.key("goal", "tier 3 4")
    s.key("challenge", "nohunger")
    s.key("intro", "A house wants two kinds of food, wicker and pots. Pots are fragile: whoever carries one won't drop more than a tile, though climbing a ladder is fine.")
    s.key("new", "KILN POT CLAY FISH")
    s.plan(plan_for("2-3", m))
    return s


# ----- Chapter III: Canopy --------------------------------------------------------------------------


def s3_1():
    m = Map(88)
    m.ground(0, 87, 19)
    steps(m, 20, 18, 1, 2)
    m.ground(24, 31, 16)
    steps(m, 32, 17, -1, 3)
    river(m, 44, 51, 19)
    m.ground(47, 48, 19, "%")
    m.ground(64, 87, 18, "%")
    m.put(0, 18, "E")
    for x in (4, 9, 14, 27, 40, 56, 63):
        m.on(x, "b")
    for x in (22, 25, 36, 58):
        m.on(x, "t")
    for x in (43, 52):
        m.on(x, "f")
    for x in (70, 75):
        m.on(x, "q")
    m.fortress(9 + 50, 4, 3)
    s = Site("3-1", "First Winter", m)
    s.key("seeds", 40).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER MASON PLANK")
    s.key("stage", "Footing 16 STONE 10 TWIG").key("stage", "Walls 24 BRICK").key("stage", "Gate 12 BRICK 12 WICKER")
    s.key("goal", "pop 40")
    s.key("challenge", "nohunger")
    s.key("intro", "Seasons turn. Berries and fish stop growing in winter, and homes still eat. A store keeps what's brought to it.")
    s.key("new", "WINTER PLANK FISH")
    s.plan(plan_for("3-1", m))
    return s


def s3_2():
    m = Map(72, 32)
    m.ground(0, 71, 28)
    m.put(0, 27, "E")
    # Two great trees: trunks, broad branches you can build on, leaves, and vines from the branch
    # ends.
    for tx in (17, 47):
        m.rect(tx - 9, 3, tx + 10, 8, ":")
        m.rect(tx, 6, tx + 1, 27, "|")
    m.hline(8, 29, 20, "=")
    m.hline(38, 59, 20, "=")
    m.hline(10, 27, 12, "=")
    m.hline(40, 57, 12, "=")
    m.vline(7, 19, 27, "v")
    m.vline(60, 19, 27, "v")
    m.vline(28, 11, 19, "v")
    m.vline(39, 11, 19, "v")
    for x in (3, 12, 23, 36, 52, 66):
        m.on(x, "b", 21)
    for x in (12, 24, 43, 55):
        m.on(x, "n", 0)
    for x in (10, 26, 41, 56):
        m.on(x, "t", 13)
    m.on(69, "c")
    m.fortress(31, 4, 4)
    s = Site("3-2", "The Canopy", m)
    s.key("seeds", 40).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN LADDER")
    s.key("stage", "Roots 16 TWIG").key("stage", "Larder 24 NUT").key("stage", "Crown 16 WICKER")
    s.key("goal", "tier 3 4")
    s.key("challenge", "maxhomes 12")
    s.key("intro", "Nut trees fruit in autumn and only then, high in the branches. Nuts weigh as much as twigs. A house wants two kinds of food.")
    s.key("new", "NUT")
    s.plan(plan_for("3-2", m))
    return s


# ----- Chapter IV: Bastion --------------------------------------------------------------------------


def s4_1():
    # A wide quiet plain to the west, the fortress hill, the lake and the noisy quarries east.
    o = 16
    m = Map(96)
    m.ground(0, 95, 19)
    m.ground(o + 28, o + 29, 18)
    m.ground(o + 30, o + 39, 17)
    m.ground(o + 40, o + 41, 18)
    river(m, o + 52, o + 57, 19)
    m.ground(o + 60, 95, 18, "%")
    m.put(0, 18, "E")
    for x in (4, 9, 15, 21, 27, 33, o + 45, o + 48):
        m.on(x, "b")
    for x in (12, 30, o + 26, o + 37, o + 38):
        m.on(x, "t")
    for x in (o + 51, o + 58):
        m.on(x, "f")
    for x in (o + 66, o + 72):
        m.on(x, "q")
    for x in (o + 76, o + 63):
        m.on(x, "c")
    m.fortress(o + 31, 5, 4)
    s = Site("4-1", "Manor Lane", m)
    s.key("seeds", 50).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN MASON GARDEN PLANK")
    s.key("stage", "Footing 16 STONE").key("stage", "Halls 20 BRICK 10 POT")
    s.key("goal", "tier 4 3")
    s.key("challenge", "maxstores 2")
    s.key("intro", "A manor wants bricks too, and a pleasant spot: gardens close by, and no kiln, mason, quarry or clay pit within three tiles.")
    s.key("new", "GARDEN PLEASANT")
    s.plan(plan_for("4-1", m))
    return s


def s4_2():
    m = Map(104)
    m.ground(0, 103, 20)
    m.ground(22, 23, 19)
    m.ground(24, 25, 18)
    m.ground(26, 37, 17)
    m.ground(38, 39, 18)
    m.ground(40, 41, 19)
    river(m, 56, 63, 20)
    steps(m, 70, 19, 1, 4)
    m.ground(78, 103, 15, "%")
    m.put(0, 19, "E")
    for x in (4, 9, 14, 19, 45, 50, 67, 84):
        m.on(x, "b")
    for x in (35, 36, 53, 92):
        m.on(x, "t")
    for x in (55, 64):
        m.on(x, "f")
    for x in (88, 95, 101):
        m.on(x, "q")
    for x in (68, 98):
        m.on(x, "c")
    m.fortress(28, 6, 5)
    s = Site("4-2", "The Bastion", m)
    s.key("seeds", 60).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN MASON GARDEN LADDER PLANK SPRING")
    s.key("stage", "Footing 20 STONE").key("stage", "Walls 30 BRICK").key("stage", "Halls 16 POT 16 WICKER").key("stage", "Keep 20 BRICK 10 STONE")
    s.key("goal", "tier 4 4").key("goal", "pop 55")
    s.key("challenge", "nohunger")
    s.key("intro", "The great fortress of the lowlands. Winter comes, manors want bricks, and so does the keep.")
    s.key("new", "SPRING")
    s.plan(plan_for("4-2", m))
    return s


# ----- Chapter V: Skyreach --------------------------------------------------------------------------


def s5_1():
    m = Map(88)
    m.ground(0, 23, 18)
    river(m, 10, 11, 18)
    # West cloud bridge over a deep hollow.
    m.ground(24, 31, 23)
    m.hline(24, 31, 18, "@")
    m.ground(32, 49, 16, "%")
    # East cloud bridge; the hollow under it has a vine up the mesa wall.
    m.ground(50, 59, 23)
    m.hline(50, 59, 18, "@")
    m.vline(50, 15, 22, "v")
    m.ground(60, 87, 18)
    steps(m, 62, 17, 1, 4)
    m.ground(70, 87, 13, "%")
    m.put(0, 17, "E")
    for x in (4, 13, 18, 34, 45, 65):
        m.on(x, "b")
    for x in (20, 37, 47, 74):
        m.on(x, "t")
    m.on(9, "f")
    for x in (80, 85):
        m.on(x, "q")
    m.on(77, "c")
    m.fortress(39, 4, 4)
    s = Site("5-1", "Cloudwalk", m)
    s.key("seeds", 50).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN MASON LADDER PLANK SPRING")
    s.key("stage", "Footing 16 BRICK").key("stage", "Spire 14 WICKER 10 POT")
    s.key("goal", "tier 2 6")
    s.key("challenge", "noinfra")
    s.key("intro", "Cloud banks hold you up only if you carry nothing, or something light. Anything heavier and you fall straight through.")
    s.key("new", "CLOUD")
    s.plan(plan_for("5-1", m))
    return s


def s5_2():
    m = Map(128)
    m.ground(0, 27, 18)
    m.ground(28, 35, 23)
    m.hline(28, 35, 18, "@")
    m.ground(36, 55, 16, "%")
    m.ground(56, 63, 23)
    m.hline(57, 63, 16, "@")
    m.vline(56, 15, 22, "v")
    m.ground(64, 91, 14)
    river(m, 70, 75, 14)
    m.ground(92, 99, 23)
    m.hline(93, 99, 16, "@")
    m.vline(92, 13, 22, "v")
    m.ground(100, 127, 18)
    steps(m, 102, 17, 1, 5)
    m.ground(112, 127, 12, "%")
    m.put(0, 17, "E")
    for x in (4, 10, 16, 22, 38, 52, 66, 104):
        m.on(x, "b")
    for x in (84, 86):
        m.on(x, "n")
    for x in (69, 76):
        m.on(x, "f")
    for x in (25, 49, 78, 120):
        m.on(x, "t")
    for x in (115, 123, 126):
        m.on(x, "q")
    for x in (54, 107, 118):
        m.on(x, "c")
    m.fortress(42, 5, 5)
    s = Site("5-2", "Skyreach", m)
    s.key("seeds", 60).key("species", "TRIBBLE DODO APE").key("build", "BURROW NEST LODGE CAMP STORE WEAVER KILN MASON GARDEN LADDER PLANK SPRING")
    s.key("stage", "Footing 20 BRICK").key("stage", "Walls 30 BRICK 10 STONE").key("stage", "Halls 20 POT 20 WICKER").key("stage", "Spire 20 BRICK 10 POT")
    s.key("goal", "tier 4 2").key("goal", "pop 50")
    s.key("challenge", "maxstores 4")
    s.key("intro", "The last fortress, above the clouds. Everything you know, all at once.")
    s.plan(plan_for("5-2", m))
    return s


PLANS["2-2"] = """burrow 2 18; camp 4 19 tribble; burrow 6 18; camp 10 19 tribble; nest 12 18; camp 15 19 tribble; nest 17 18; camp 38 17 tribble; burrow 27 18;
store 82 18 ape; lodge 77 18; camp 80 19 ape; camp 75 19 tribble; lodge 88 18; camp 86 19 ape;
store 58 16 ape; get 58 16 stone; lodge 64 16; camp 62 17 tribble; nest 33 16; camp 35 17 tribble;
store 29 18 dodo; get 29 18 stone; get 29 18 brick; mason 84 18 dodo; lodge 94 18; camp 96 19 tribble; camp 92 19 dodo;
get 58 16 brick; weaver 48 14 tribble; weaver 39 16 tribble; camp 46 15 tribble; camp 50 15 tribble; burrow 43 14; camp 52 15 tribble; nest 53 14; lodge 60 16; camp 66 17 tribble; nest 8 18"""

PLANS["2-3"] = """burrow 2 16; camp 4 17 tribble; burrow 6 16; camp 9 17 tribble; nest 11 16; camp 14 17 tribble; burrow 22 16; camp 21 17 tribble;
nest 48 16; camp 67 17 dodo; camp 51 17 tribble; camp 39 21 dodo; kiln 52 16 tribble; weaver 55 16 tribble; camp 54 17 tribble; camp 63 17 tribble;
ladder 47 17; ladder 45 19; ladder 36 17; store 24 16 dodo; camp 65 17 ape; nest 61 16; camp 60 17 tribble;
lodge 27 16; camp 43 21 ape; nest 57 16; burrow 15 16; camp 26 17 tribble; lodge 29 16; camp 32 17 tribble; lodge 34 16"""

PLANS["3-1"] = """burrow 2 17; camp 4 18 tribble; burrow 6 17; camp 9 18 tribble; nest 11 17; camp 14 18 tribble; nest 16 17; camp 22 16 tribble;
store 38 17 tribble; camp 27 15 tribble; camp 43 18 dodo; burrow 29 14; camp 25 15 tribble; lodge 41 17;
plank 44 19; plank 45 19; plank 46 19; plank 49 19; plank 50 19; plank 51 19;
camp 52 18 dodo; camp 56 18 tribble; nest 53 17; camp 70 17 ape; lodge 66 16; mason 72 16 ape; camp 75 17 ape; camp 63 18 tribble;
weaver 34 16 tribble; camp 36 18 tribble; camp 58 18 tribble; burrow 18 17; nest 77 16; lodge 68 16; lodge 64 16;
nest 80 16; burrow 82 16; weaver 84 16 tribble; nest 86 16"""

PLANS["3-2"] = """burrow 1 26; camp 3 27 tribble; burrow 4 26; camp 12 27 tribble; nest 9 26; camp 23 27 tribble;
lodge 13 26; lodge 15 26; ladder 7 19; camp 10 19 tribble; camp 26 19 tribble; weaver 19 26 tribble;
burrow 21 26; store 29 26 tribble; camp 36 27 tribble; ladder 60 19; camp 41 19 tribble; camp 56 19 tribble;
camp 12 11 ape; camp 24 11 ape; lodge 19 18; lodge 21 18; camp 43 11 ape; camp 55 11 ape;
nest 64 26; kiln 62 26 dodo; camp 69 27 dodo; camp 52 27 tribble; camp 66 27 tribble; burrow 24 26; nest 39 26; burrow 44 26"""

PLANS["4-1"] = """burrow 2 17; camp 4 18 tribble; burrow 6 17; camp 9 18 tribble; nest 13 17; camp 15 18 tribble; camp 12 18 tribble; weaver 10 17 tribble;
nest 17 17; camp 21 18 tribble; burrow 23 17; camp 27 18 tribble; camp 30 18 tribble; nest 28 17; camp 67 18 dodo; store 59 17 dodo;
plank 68 19; plank 69 19; plank 70 19; plank 71 19; plank 72 19; plank 73 19;
camp 74 18 dodo; lodge 84 16; camp 82 17 ape; mason 86 16 ape; camp 88 17 ape; camp 92 17 dodo; kiln 77 16 dodo;
camp 79 17 dodo; lodge 93 16; camp 53 16 tribble; camp 54 16 tribble; camp 42 18 tribble; camp 33 18 tribble;
garden 5 18; garden 8 18; garden 16 18; garden 19 18; garden 22 18; garden 25 18; garden 26 18;
nest 35 17; burrow 37 17; lodge 31 17; lodge 40 17; nest 62 17; lodge 65 17"""

PLANS["4-2"] = """burrow 2 18; camp 4 19 tribble; nest 6 18; camp 9 19 tribble; burrow 10 18; camp 14 19 tribble; nest 15 18; camp 19 19 tribble;
weaver 22 17 tribble; camp 35 16 tribble; camp 36 16 tribble; burrow 20 18; burrow 42 18; camp 45 19 tribble; store 47 18 ape; camp 50 19 tribble; camp 53 19 tribble;
camp 55 19 dodo; nest 51 18; plank 56 20; plank 57 20; plank 58 20; plank 59 20; plank 60 20; plank 61 20; plank 62 20; plank 63 20;
camp 64 19 dodo; camp 67 19 tribble; lodge 65 18; lodge 80 13; store 82 13 ape; camp 84 14 tribble; lodge 85 13; camp 88 14 ape; mason 89 13 ape;
get 47 18 stone; camp 92 14 tribble; lodge 93 13; camp 95 14 ape; kiln 96 13 ape; camp 98 14 ape; lodge 99 13; camp 101 14 ape;
garden 1 19; garden 5 19; garden 8 19; garden 12 19; garden 13 19; garden 17 19; nest 38 16; lodge 40 17; camp 68 19 dodo"""

PLANS["5-1"] = """burrow 2 16; camp 4 17 tribble; burrow 6 16; camp 13 17 tribble; nest 14 16; camp 18 17 tribble; camp 20 17 tribble; weaver 21 16 tribble;
burrow 32 14; camp 34 15 tribble; camp 37 15 tribble; camp 45 15 tribble;
lodge 71 11; camp 80 12 ape; mason 82 11 ape; camp 85 12 ape; lodge 78 11; camp 77 12 dodo; kiln 75 11 dodo; camp 74 12 tribble; camp 65 15 tribble;
nest 60 16; weaver 43 14 tribble; nest 48 14; burrow 35 14; nest 16 16; lodge 86 11"""

PLANS["5-2"] = """burrow 2 16; camp 4 17 tribble; burrow 6 16; camp 10 17 tribble; nest 12 16; camp 16 17 tribble; burrow 18 16; camp 22 17 tribble;
weaver 20 16 tribble; camp 25 17 tribble; store 23 16 tribble; burrow 39 14; camp 38 15 tribble; lodge 36 14; lodge 8 16; camp 52 15 tribble; camp 49 15 tribble;
plank 70 14; plank 71 14; plank 72 14; plank 73 14; plank 74 14; plank 75 14; lodge 67 12; camp 66 13 tribble; camp 69 13 dodo; camp 76 13 dodo; camp 78 13 tribble;
lodge 100 16; camp 104 15 tribble; lodge 113 10; camp 115 11 ape; mason 116 10 ape; lodge 110 11; store 121 10 ape; camp 123 11 ape;
store 88 12 ape; get 88 12 brick; get 88 12 stone; store 64 12 ape; get 64 12 stone; get 64 12 brick; store 47 14 ape; get 47 14 brick; get 47 14 stone;
weaver 79 12 tribble; burrow 81 12; nest 90 12; lodge 14 16; camp 126 11 ape; camp 54 15 dodo; kiln 50 14 dodo; garden 77 13; garden 83 13; garden 85 13; garden 87 13; garden 41 15;
camp 84 13 ape; camp 86 13 ape"""


def chapter(n, sites):
    path = os.path.join(OUT, f"ch{n}.txt")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(f"chapter {n}\n")
        for s in sites:
            f.write(s.text())
            f.write("\n")
    print(f"wrote {path}: {', '.join(s.sid for s in sites)}")


if __name__ == "__main__":
    chapter(1, [s1_1(), s1_2(), s1_3()])
    chapter(2, [s2_1(), s2_2(), s2_3()])
    chapter(3, [s3_1(), s3_2()])
    chapter(4, [s4_1(), s4_2()])
    chapter(5, [s5_1(), s5_2()])
