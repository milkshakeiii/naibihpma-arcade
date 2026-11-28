#!/usr/bin/env python3
"""Naibihpma Hold: a headless model of the Hold's economy, for document 02.

It plays RockCollector's Triblings rules as ported in document 01, with the fixes
Henry took, and the numbers proposed in document 02. There are no enemies: orbs
arrive at fixed times, and heroes earn dew at a fixed rate, standing in for
rewards and drops until document 03 sets them. A scripted Elder builds in a
fixed order and hatches as each building allows.

    python3 -I econ.py                 # five seeds, summary table
    python3 -I econ.py --seed 3 -v     # one seed, with the timeline

One tick is one 15 FPS frame. A creature walks one square in 10 ticks.
"""

import argparse
import heapq
import math
import random
from collections import Counter

TPS = 15
DAY = 5400            # a day and a night, six minutes
DUSK = 2700           # night runs from here to the end of the day
NOON = 1350
WALK = 10             # ticks per square
CARRY_DEW = 10        # dew a carrier can hold; every other item is one per trip
HORIZON = 45 * 60 * TPS

# ---------------------------------------------------------------- the numbers

# Spots: ripen time, work to pick, difficulty to pick, what it gives and how much,
# planting skill and difficulty (planted kinds), and when it ripens.
SPOTS = {
    "fern":  dict(ripen=1000, work=10, diff=10, gives="dew", n=15, skill="gathering", when="damp"),
    "red":   dict(ripen=1000, work=6, diff=15, gives="fungiwood", n=2, skill="fungiculture",
                  plant=0, plant_diff=10, when="always"),
    "white": dict(ripen=1000, work=9, diff=20, gives="cap ingot", n=1, skill="fungiculture",
                  plant=5, plant_diff=15, when="always"),
    "hybrid": dict(ripen=1000, work=12, diff=25, gives="spotted cap", n=1, skill="fungiculture",
                   plant=8, plant_diff=25, when="always"),
    "puff":  dict(ripen=1000, work=6, diff=15, gives="glow spores", n=1, skill="fungiculture",
                  when="night"),
}
BONUS_REGROW = 600    # a building's own spot comes back this long after it's picked bare

GATHER_COOLDOWN, GATHER_AMOUNT = 60, 1
FUNGI_COOLDOWN, FUNGI_AMOUNT = 100, 2
TEND_SPEED = 1.5
MEND = (1, 40)                     # by hand: health, ticks
DIG_COOLDOWN = 10                  # with a shovel, from level 5: the shovel's mend amount
HATCH_PURSE = 10                   # dew a hero hatchling carries out
BUILDERS = 4                       # how many can work on one building at once
REPAIR_CHUNK = 150                 # a builder re-thinks this often

# Items: slot, slot level, recipe, maker's skill, crafting level, ticks, price, extras.
ITEMS = {
    "wooden shovel":  dict(slot="shovel", lvl=1, inputs={"fungiwood": 1}, skill="woodcrafting", craft=0, time=150, price=6, mend=2),
    "iron shovel":    dict(slot="shovel", lvl=2, inputs={"fungiwood": 1, "cap ingot": 1}, skill="ironcrafting", craft=0, time=150, price=15, mend=3),
    "venomwood shovel": dict(slot="shovel", lvl=3, inputs={"fungiwood": 1, "spotted cap": 1}, skill="woodcrafting", craft=2, time=150, price=30, mend=4),
    "slingshot":      dict(slot="sling", lvl=1, inputs={"fungiwood": 1, "cap ingot": 1}, skill="woodcrafting", craft=0, time=150, price=10),
    "venomiron catapult": dict(slot="sling", lvl=2, inputs={"fungiwood": 1, "cap ingot": 1, "spotted cap": 1}, skill="woodcrafting", craft=2, time=150, price=40),
    "iron spear":     dict(slot="spear", lvl=2, inputs={"fungiwood": 1, "cap ingot": 1}, skill="ironcrafting", craft=2, time=150, price=25),
    "venomwood spear": dict(slot="spear", lvl=3, inputs={"fungiwood": 1, "spotted cap": 1}, skill="woodcrafting", craft=3, time=150, price=45),
    "venomiron spear": dict(slot="spear", lvl=4, inputs={"fungiwood": 1, "cap ingot": 1, "spotted cap": 1, "Meteor Orb": 1}, skill="ironcrafting", craft=5, time=150, price=70),
    "chanfron":       dict(slot="head", lvl=1, inputs={"cap ingot": 1}, skill="ironcrafting", craft=3, time=1000, price=20),
    "peytral":        dict(slot="chest", lvl=1, inputs={"cap ingot": 2}, skill="ironcrafting", craft=3, time=1000, price=30),
    "croupiere":      dict(slot="rump", lvl=1, inputs={"cap ingot": 2}, skill="ironcrafting", craft=3, time=1000, price=30),
    "venomiron chanfron": dict(slot="head", lvl=2, inputs={"cap ingot": 1, "spotted cap": 1}, skill="ironcrafting", craft=4, time=1000, price=60),
    "dew bead":       dict(slot="bauble", lvl=1, inputs={"fungiwood": 1, "dew": 2}, skill="woodcrafting", craft=1, time=150, price=12),
    "glow charm":     dict(slot="bauble", lvl=2, inputs={"fungiwood": 1, "glow spores": 1, "dew": 2}, skill="woodcrafting", craft=3, time=150, price=30),
    "star pearl":     dict(slot="bauble", lvl=3, inputs={"glow spores": 2, "Meteor Orb": 1}, skill="enchanting", craft=8, time=300, price=50),
}

# Creature kinds: attribute bonuses (str dex con int wis cha), and what they do.
KINDS = {
    "newt":       dict(attrs=(-4, 0, 2, 0, 0, 0), role="carrier"),
    "caecilian":  dict(attrs=(-4, 0, 2, 0, 2, 0), role="grower"),
    "tree frog":  dict(attrs=(-2, 2, 0, 0, 0, 0), role="maker", craft="woodcrafting"),
    "salamander": dict(attrs=(0, 2, 0, 2, 0, 0), role="maker", craft="ironcrafting"),
    "daot":       dict(attrs=(0, 0, 0, 4, 4, 2), role="maker", craft="enchanting", start_skill=8),
    "toad":       dict(attrs=(2, 0, 2, -2, -2, 0), role="hero", slots=("spear", "sling")),
    "axolotl":    dict(attrs=(-2, -2, -2, 2, 2, 2), role="hero", slots=("bauble",)),
    "bullfrog":   dict(attrs=(6, 0, 8, 0, 0, 0), role="hero", slots=("shovel", "shovel2", "head", "chest", "rump")),
    "goliath":    dict(attrs=(4, 0, 4, 2, 2, 2), role="hero", slots=("shovel",)),
    "olm":        dict(attrs=(6, 6, 6, 6, 6, 6), role="hero", slots=("bauble", "spear", "head")),
}
SKILL_ATTR = {"gathering": 1, "fungiculture": 4, "woodcrafting": 1, "ironcrafting": 3, "enchanting": 3}

# Buildings: health, cost, which building it's built from, its spots and plot,
# who it houses, what it makes, and the stock it keeps by default.
B = {
    "Hall":         dict(hp=1000, cost={}, src=None, spots=[("fern", 4, "damp")], houses={"newt": 8}, keeps={"wooden shovel": 2}),
    "Burrow":       dict(hp=60, cost={"dew": 20, "fungiwood": 4}, src="Hall", spots=[("red", 1, "cave")],
                         houses={"caecilian": 2}, keeps={"dew": 10}),
    "Twig Loft":    dict(hp=60, cost={"dew": 10}, src="Hall", spots=[("red", 1, "cave")], houses={"tree frog": 2},
                         makes=["wooden shovel"], keeps={"fungiwood": 3, "wooden shovel": 3}),
    "Morel Cave":   dict(hp=60, cost={"dew": 20, "fungiwood": 4}, src="Burrow", plot=("red", 4), houses={"caecilian": 2}),
    "Fairy Ring":   dict(hp=40, cost={"fungiwood": 1, "cap ingot": 1}, src="Burrow", plot=("hybrid", 2)),
    "Chorus Log":   dict(hp=250, cost={"dew": 50, "fungiwood": 10}, src="Hall", houses={},
                         keeps={"wooden shovel": 3, "iron shovel": 2}),
    "Ember Log":    dict(hp=60, cost={"dew": 10}, src="Chorus Log", houses={"salamander": 2},
                         makes=["iron shovel"], keeps={"fungiwood": 1, "cap ingot": 1, "iron shovel": 2}),
    "Ironcap Cave": dict(hp=60, cost={"dew": 20, "fungiwood": 4}, src="Chorus Log", plot=("white", 4), houses={"caecilian": 2}),
    "Toad Hall":    dict(hp=150, cost={"fungiwood": 5}, src="Chorus Log", houses={"toad": 5},
                         makes=["slingshot", "iron spear"],
                         keeps={"fungiwood": 2, "cap ingot": 2, "slingshot": 3, "iron spear": 2}),
    "Shell Forge":  dict(hp=150, cost={"cap ingot": 5}, src="Toad Hall", makes=["chanfron", "peytral", "croupiere"],
                         keeps={"cap ingot": 3, "chanfron": 1, "peytral": 1, "croupiere": 1}),
    "Cauldron":     dict(hp=150, cost={"cap ingot": 5}, src="Toad Hall", makes=["venomwood shovel", "venomwood spear"],
                         keeps={"fungiwood": 2, "spotted cap": 2, "venomwood shovel": 1, "venomwood spear": 2}),
    "Star Pond":    dict(hp=250, cost={"Sun Orb": 1}, src="Hall", spots=[("puff", 2, "night")], houses={"axolotl": 4},
                         makes=["dew bead", "glow charm"],
                         keeps={"fungiwood": 2, "dew": 4, "glow spores": 2, "dew bead": 2, "glow charm": 1}),
    "Newt House":   dict(hp=60, cost={"dew": 10, "fungiwood": 2}, src="Star Pond", spots=[("fern", 2, "open")],
                         houses={"newt": 4}),
    "Spawning Pool": dict(hp=250, cost={"Clargent Orb": 1, "fungiwood": 10, "dew": 250}, src="Star Pond",
                          houses={"bullfrog": 2, "goliath": 2}, keeps={"dew": 20}),
    "Daot's Grotto": dict(hp=150, cost={"dew": 50, "fungiwood": 5, "cap ingot": 2}, src="Star Pond",
                          houses={"daot": 1},
                          makes=["venomiron catapult", "venomiron spear", "venomiron chanfron", "star pearl"],
                          keeps={"fungiwood": 1, "cap ingot": 2, "spotted cap": 2, "glow spores": 2, "Meteor Orb": 1,
                                 "venomiron catapult": 1, "venomiron spear": 1, "venomiron chanfron": 1, "star pearl": 1}),
    "Olm's Cave":   dict(hp=250, cost={"Clargent Orb": 1, "dew": 200}, src="Spawning Pool", houses={"olm": 1}),
}

# Where each building goes, in squares from the Hall.
POS = {
    "Hall": (0, 0), "Burrow": (8, 4), "Twig Loft": (-7, 4), "Morel Cave": (12, 8), "Chorus Log": (0, 10),
    "Ironcap Cave": (-10, 12), "Toad Hall": (6, 16), "Ember Log": (-4, 16), "Fairy Ring": (14, 14),
    "Star Pond": (-14, -2), "Cauldron": (10, 22), "Shell Forge": (-2, 22), "Newt House": (-18, 6),
    "Spawning Pool": (-20, -10), "Daot's Grotto": (-24, 0), "Olm's Cave": (-28, -14),
}

BUILD_ORDER = ["Burrow", "Twig Loft", "Morel Cave", "Chorus Log", "Ironcap Cave", "Toad Hall", "Ember Log",
               "Fairy Ring", "Star Pond", "Newt House", "Shell Forge", "Cauldron", "Daot's Grotto",
               "Spawning Pool", "Olm's Cave"]

FULL_ORDER = list(BUILD_ORDER)

HATCH_COST = {"caecilian": {"dew": 5}, "tree frog": {"dew": 5}, "salamander": {"dew": 5}, "newt": {},
              "toad": {}, "bullfrog": {}, "goliath": {}, "axolotl": {"Sun Orb": 1},
              "olm": {"Sun Orb": 1, "Clargent Orb": 1, "Meteor Orb": 1}, "daot": {}}
HATCH_TIME = 500

START_STOCK = {"dew": 60, "fungiwood": 6, "cap ingot": 3, "wooden shovel": 4}
# The first scenario starts with a small Hold already standing (document 05).
START_BUILT = ["Burrow", "Twig Loft", "Chorus Log", "Toad Hall"]
START_FOLK = [("caecilian", "Burrow", 2), ("tree frog", "Twig Loft", 2), ("toad", "Toad Hall", 2)]

# Stand-ins until document 03: when orbs arrive, and what heroes earn.
ORBS = [(8, "Sun Orb"), (14, "Clargent Orb"), (18, "Meteor Orb"), (21, "Sun Orb"), (25, "Sun Orb"),
        (28, "Clargent Orb"), (30, "Meteor Orb"), (33, "Sun Orb"), (35, "Clargent Orb"), (38, "Meteor Orb"),
        (40, "Sun Orb")]
MAX_AXOLOTLS_BEFORE_OLM = 2
REWARDS_PER_MIN = 15        # the Elder's rewards, out of the Hall's dew above a reserve
DROPS_PER_HERO_MIN = 3      # new dew heroes pick up from what they kill
TITHE = 10                  # percent of each reward that goes to the hero's house
RESERVE = 40

# ---------------------------------------------------------------- the model


def mod(score):
    return score // 2 - 5


def roll(rng, m, diff):
    d = rng.randint(1, 20)
    return d == 20 or d + m >= diff


def phase(t):
    return "day" if t % DAY < DUSK else "night"


def dist(a, b):
    return max(abs(a[0] - b[0]), abs(a[1] - b[1]))


class Spot:
    def __init__(self, kind, bld, ground, bonus):
        self.kind, self.bld, self.ground, self.bonus = kind, bld, ground, bonus
        self.state = "growing" if bonus else "empty"   # empty, growing, ripe, gone
        self.progress = 0.0
        self.work = 0
        self.claim = None
        self.tender = None
        self.gone_until = 0

    def damp_now(self, t):
        when = SPOTS[self.kind]["when"]
        if when == "night" or self.ground == "night":
            return phase(t) == "night"
        if self.ground == "open":
            return phase(t) == "night"
        return True


class Bldg:
    def __init__(self, name, built):
        d = B[name]
        self.name, self.pos = name, POS[name]
        self.max_hp = d["hp"]
        self.hp = d["hp"] if built else d["hp"] // 10
        self.stock = Counter()
        self.ground = Counter()
        self.keeps = dict(d.get("keeps", {}))
        self.res = Counter()          # reserved for a build or a hatching
        self.incoming = Counter()
        self.outgoing = Counter()
        self.repairers = set()
        self.pending_hp = 0
        self.crafter = None
        self.spots = []
        for kind, n, ground in d.get("spots", []):
            for _ in range(n):
                self.spots.append(Spot(kind, self, ground, True))
        if "plot" in d:
            kind, n = d["plot"]
            for _ in range(n):
                self.spots.append(Spot(kind, self, "cave", False))
        for item in d.get("makes", []):
            for inp in ITEMS[item]["inputs"]:
                self.keeps.setdefault(inp, 1)
        self.residents = Counter()
        self.hatching = None          # (kind, ready tick)
        self.hres = None

    @property
    def done(self):
        return self.hp >= self.max_hp

    def want(self, item):
        if self.name == "Hall" and item == "dew":
            return 0
        return self.keeps.get(item, 0) + self.res[item] - self.stock[item] - self.incoming[item]

    def spare(self, item):
        if self.name == "Hall" and item == "dew":
            return self.stock[item] - self.outgoing[item] - self.res[item]
        return self.stock[item] - self.outgoing[item] - self.keeps.get(item, 0) - self.res[item]


class Creature:
    _ids = 0

    def __init__(self, kind, home, rng, t):
        Creature._ids += 1
        self.id = Creature._ids
        self.kind, self.home = kind, home
        self.pos = home.pos
        bonus = KINDS[kind]["attrs"]
        self.attrs = [8 + rng.randint(1, 6) + b for b in bonus]
        self.skills = Counter()
        if KINDS[kind].get("start_skill"):
            self.skills[KINDS[kind]["craft"]] = KINDS[kind]["start_skill"]
        self.level, self.xp = 1, 0
        self.gear = {}
        self.purse = 0
        self.trait = rng.choice(["bold", "greedy", "steady", "timid"])
        self.task = None
        self.carry = None
        self.born = t

    def skill(self, name, world):
        m = math.floor(self.skills[name]) + mod(self.attrs[SKILL_ATTR[name]])
        if name == "gathering" and world.twen_alive:
            m += 2
        return m

    def gain(self, skill, enc, world):
        if skill:
            self.skills[skill] += max(0.0, math.log(max(enc - 8, 1))) / (self.skills[skill] + 1)
        xp = (enc - 9) ** 2
        self.xp += xp
        if self.xp >= self.level ** 2:
            self.level += 1
            self.xp = 0


class World:
    def __init__(self, seed, verbose=False, scenario=False):
        self.scenario = scenario
        self.rng = random.Random(seed)
        self.t = 0
        self.verbose = verbose
        self.log = []
        self.first = {}
        self.blds = {"Hall": Bldg("Hall", True)}
        for k, v in START_STOCK.items():
            self.blds["Hall"].stock[k] += v
        self.creatures = []
        self.q = []
        self.seq = 0
        self.twen_alive = True
        self.build_i = 0
        self.pending = None       # (name) being reserved at its source
        self.hall_income = Counter()
        self.idle = Counter()
        self.busy = Counter()
        self.sold = Counter()
        self.crafted = Counter()
        for _ in range(8):
            self.spawn("newt", self.blds["Hall"])
        if self.scenario:
            for name in START_BUILT:
                self.blds[name] = Bldg(name, True)
                if name in BUILD_ORDER:
                    BUILD_ORDER.remove(name)
            for kind, home, n in START_FOLK:
                for _ in range(n):
                    c = self.spawn(kind, self.blds[home])
                    if kind == "toad":
                        c.gear["sling"] = "slingshot"
        self.orbs = sorted((m * 60 * TPS, o) for m, o in ORBS)

    # -- bookkeeping
    def note(self, what):
        key = what
        if key not in self.first:
            self.first[key] = self.t
            if self.verbose:
                self.log.append(f"{clock(self.t)}  {what}")

    def spawn(self, kind, home):
        c = Creature(kind, home, self.rng, self.t)
        self.creatures.append(c)
        home.residents[kind] += 1
        self.schedule(c, self.t + 1)
        return c

    def schedule(self, c, when):
        self.seq += 1
        heapq.heappush(self.q, (when, self.seq, c.id, c))

    def built(self):
        return [b for b in self.blds.values() if b.done]

    # -- the Elder
    def elder(self):
        t = self.t
        # orbs arriving (heroes bring them home)
        while self.orbs and self.orbs[0][0] <= t:
            _, orb = self.orbs.pop(0)
            self.blds["Hall"].ground[orb] += 1
            self.note(f"{orb} found")
        # building, in order, one reservation at a time
        if self.pending is None and self.build_i < len(BUILD_ORDER):
            name = BUILD_ORDER[self.build_i]
            src = self.blds.get(B[name]["src"])
            if src and src.done:
                for k, v in B[name]["cost"].items():
                    src.res[k] += v
                self.pending = name
        if self.pending:
            name = self.pending
            src = self.blds[B[name]["src"]]
            cost = B[name]["cost"]
            if all(src.stock[k] >= v for k, v in cost.items()):
                for k, v in cost.items():
                    src.stock[k] -= v
                    src.res[k] -= v
                self.blds[name] = Bldg(name, False)
                self.note(f"{name} placed")
                self.pending = None
                self.build_i += 1
        # hatching
        for b in self.built():
            if b.hatching:
                kind, ready = b.hatching
                if t >= ready:
                    c = self.spawn(kind, b)
                    if KINDS[kind]["role"] == "hero":
                        c.purse = HATCH_PURSE
                    b.hatching = None
                    self.note(f"first {kind}")
                continue
            for kind, n in B[b.name].get("houses", {}).items():
                if b.residents[kind] >= n:
                    continue
                if kind == "axolotl" and (self.olm_needs_sun() or (
                        b.residents[kind] >= MAX_AXOLOTLS_BEFORE_OLM
                        and not any(c.kind == "olm" for c in self.creatures))):
                    continue
                cost = dict(HATCH_COST[kind])
                if KINDS[kind]["role"] == "hero":
                    cost["dew"] = cost.get("dew", 0) + HATCH_PURSE
                key = ("hatch", kind)
                if not getattr(b, "hres", None):
                    b.hres = cost
                    for k, v in cost.items():
                        b.res[k] += v
                if all(b.stock[k] >= v for k, v in b.hres.items()):
                    for k, v in b.hres.items():
                        b.stock[k] -= v
                        b.res[k] -= v
                    b.hres = None
                    b.hatching = (kind, t + HATCH_TIME)
                break
        # rewards and drops, every minute
        if t % (60 * TPS) == 0 and t > 0:
            heroes = [c for c in self.creatures if KINDS[c.kind]["role"] == "hero"]
            hall = self.blds["Hall"]
            need = sum(self.pending_cost().values()) if self.pending else 0
            pot = min(REWARDS_PER_MIN, hall.stock["dew"] - hall.res["dew"] - RESERVE - need)
            if heroes and pot > 0:
                hall.stock["dew"] -= pot
                share = pot // len(heroes)
                for h in heroes:
                    tithe = share * TITHE // 100
                    h.purse += share - tithe
                    h.home.stock["dew"] += tithe
            for h in heroes:
                h.purse += DROPS_PER_HERO_MIN

    def pending_cost(self):
        return Counter({k: v for k, v in B[self.pending]["cost"].items() if k == "dew"})

    def olm_needs_sun(self):
        return "Olm's Cave" in self.blds and not any(c.kind == "olm" for c in self.creatures)

    # -- spots
    def grow(self):
        t = self.t
        for b in self.blds.values():
            if not b.done:
                continue
            for s in b.spots:
                d = SPOTS[s.kind]
                if s.state == "gone" and t >= s.gone_until:
                    s.state, s.progress = "growing", 0
                if s.state == "growing" and s.damp_now(t):
                    s.progress += 15 * (TEND_SPEED if s.tender else 1)
                    if s.progress >= d["ripen"]:
                        s.state, s.work = "ripe", 0
                # dew dries at noon on open ground; puffballs burst at dawn
                if s.state == "ripe" and s.claim is None:
                    if s.ground == "open" and t % DAY == NOON - NOON % 15:
                        s.state, s.progress = "growing", 0
                        self.note("dew dried on an open fern")
                    if s.kind == "puff" and t % DAY == 0:
                        s.state, s.progress = "growing", 0
                        self.note("a puffball burst unpicked")

    # -- what's wanted
    def total_want(self, item):
        return sum(max(0, b.want(item)) for b in self.blds.values()) - sum(b.ground[item] for b in self.blds.values())

    def find_delivery(self, c):
        """The nearest source for the first unmet want, or spare dew for the Hall."""
        order = sorted(self.blds.values(), key=lambda b: (0 if b.res else 1, dist(c.pos, b.pos)))
        for dst in order:
            if not dst.done and not dst.res:
                continue
            for item in list(dst.keeps) + list(dst.res):
                w = dst.want(item)
                if w <= 0:
                    continue
                src = self.source(item, dst, c.pos)
                if src:
                    kind, sb = src
                    have = sb.ground[item] if kind == "ground" else sb.spare(item)
                    n = min(w, have, CARRY_DEW if item == "dew" else 1)
                    if n > 0:
                        return (item, n, kind, sb, dst)
        # spare dew goes to the Hall
        hall = self.blds["Hall"]
        best = None
        for b in self.blds.values():
            if b.ground["dew"] > 0:
                cand = ("ground", b, b.ground["dew"])
            elif b is not hall and b.spare("dew") > 0:
                cand = ("stock", b, b.spare("dew"))
            else:
                continue
            dd = dist(c.pos, b.pos)
            if best is None or dd < best[0]:
                best = (dd, cand)
        if best:
            kind, b, have = best[1]
            return ("dew", min(have, CARRY_DEW), kind, b, hall)
        return None

    def source(self, item, dst, pos):
        best = None
        for b in self.blds.values():
            if b is dst:
                continue
            for kind, have in (("ground", b.ground[item]), ("stock", b.spare(item) if b.done else 0)):
                if have > 0:
                    dd = dist(pos, b.pos)
                    if best is None or dd < best[0]:
                        best = (dd, (kind, b))
        if dst.ground[item] > 0:
            return ("ground", dst)
        return best[1] if best else None

    # -- creatures
    def think(self, c):
        role = KINDS[c.kind]["role"]
        if role == "carrier":
            return self.think_newt(c)
        if role == "grower":
            return self.think_grower(c)
        if role == "maker":
            return self.think_maker(c)
        return self.think_hero(c)

    def walk(self, c, to):
        d = dist(c.pos, to)
        c.pos = to
        return d * WALK

    def think_newt(self, c):
        # equip: a better shovel from the Hall's or the Chorus Log's stock
        cur = ITEMS[c.gear["shovel"]]["lvl"] if "shovel" in c.gear else 0
        for b in (self.blds.get("Hall"), self.blds.get("Chorus Log")):
            if not b or not b.done:
                continue
            for item in ("venomwood shovel", "iron shovel", "wooden shovel"):
                if ITEMS[item]["lvl"] > cur and b.stock[item] > b.outgoing[item]:
                    b.stock[item] -= 1
                    c.gear["shovel"] = item
                    return self.walk(c, b.pos) + 10
        # repair
        r = self.repair(c)
        if r:
            return r
        # deliver
        job = self.find_delivery(c)
        if job:
            return self.deliver(c, job)
        # harvest a ripe fern
        best = None
        for b in self.built():
            for s in b.spots:
                if s.kind == "fern" and s.state == "ripe" and s.claim is None:
                    dd = dist(c.pos, b.pos)
                    if best is None or dd < best[0]:
                        best = (dd, s)
        if best:
            return self.harvest(c, best[1], "gathering", GATHER_COOLDOWN, GATHER_AMOUNT)
        self.idle[c.kind] += 150
        return 150

    def repair(self, c):
        cands = [b for b in self.blds.values()
                 if b.hp + b.pending_hp < b.max_hp and len(b.repairers) < BUILDERS]
        if not cands:
            return 0
        b = min(cands, key=lambda b: dist(c.pos, b.pos))
        b.repairers.add(c.id)
        t = self.walk(c, b.pos)
        if c.level >= 5 and "shovel" in c.gear:
            amt, cd = ITEMS[c.gear["shovel"]]["mend"], DIG_COOLDOWN
        else:
            amt, cd = MEND
        left = b.max_hp - b.hp - b.pending_hp
        acts = max(1, min(-(-left // amt), REPAIR_CHUNK // cd))
        hp = min(left, acts * amt)
        b.pending_hp += hp
        for _ in range(acts):
            c.gain(None, 10, self)
        t += acts * cd

        def done(b=b, hp=hp, cid=c.id):
            b.pending_hp -= hp
            b.hp = min(b.max_hp, b.hp + hp)
            b.repairers.discard(cid)
            if b.done:
                self.note(f"{b.name} built")
        self.later(t, done)
        self.busy[c.kind] += t
        return t

    def deliver(self, c, job):
        item, n, kind, src, dst = job
        if kind == "ground":
            src.ground[item] -= n
        else:
            src.stock[item] -= n
        dst.incoming[item] += n
        t = self.walk(c, src.pos) + self.walk(c, dst.pos) + 10
        # it arrives when the carrier does
        self.later(t, lambda: self.arrive(dst, item, n))
        self.busy[c.kind] += t
        return t

    def arrive(self, dst, item, n):
        dst.incoming[item] -= n
        dst.stock[item] += n
        if dst.name == "Hall" and item == "dew":
            self.hall_income[self.t // (60 * TPS)] += n

    def later(self, dt, fn):
        self.pend.append((self.t + dt, fn))

    def harvest(self, c, s, skill, cooldown, amount):
        d = SPOTS[s.kind]
        s.claim = c.id
        t = self.walk(c, s.bld.pos)
        work = 0
        while work < d["work"]:
            if roll(self.rng, c.skill(skill, self), d["diff"]):
                work += amount
            t += cooldown
        c.gain(skill, d["diff"], self)

        def done(s=s, d=d):
            s.claim = None
            s.bld.ground[d["gives"]] += d["n"]
            if s.bonus:
                if s.kind in ("fern", "puff"):
                    s.state, s.progress = "growing", 0
                else:
                    s.state, s.gone_until = "gone", self.t + BONUS_REGROW
            else:
                s.state = "empty"
            self.note(f"first {d['gives']} picked")
        self.later(t, done)
        self.busy[c.kind] += t
        return t

    def think_grower(self, c):
        # plant an empty plot it has the skill for
        for b in sorted(self.built(), key=lambda b: dist(c.pos, b.pos)):
            for s in b.spots:
                if s.state == "empty" and s.claim is None:
                    d = SPOTS[s.kind]
                    if c.skill("fungiculture", self) >= d["plant"]:
                        s.claim = c.id
                        t = self.walk(c, b.pos)
                        while True:
                            t += FUNGI_COOLDOWN
                            if roll(self.rng, c.skill("fungiculture", self), d["plant_diff"]):
                                break
                        c.gain("fungiculture", d["plant_diff"], self)

                        def done(s=s):
                            s.claim, s.state, s.progress = None, "growing", 0
                            self.note(f"first {s.kind} fungus planted")
                        self.later(t, done)
                        self.busy[c.kind] += t
                        return t
        # pick a ripe fungus that's wanted
        best = None
        for b in self.built():
            for s in b.spots:
                if s.kind != "fern" and s.state == "ripe" and s.claim is None:
                    if self.total_want(SPOTS[s.kind]["gives"]) > 0:
                        dd = dist(c.pos, b.pos)
                        if best is None or dd < best[0]:
                            best = (dd, s)
        if best:
            return self.harvest(c, best[1], "fungiculture", FUNGI_COOLDOWN, FUNGI_AMOUNT)
        # tend
        for b in self.built():
            for s in b.spots:
                if s.kind != "fern" and s.state == "growing" and s.tender is None and s.damp_now(self.t):
                    s.tender = c.id
                    c.gain("fungiculture", 10, self)      # tending teaches, as repairing does
                    t = self.walk(c, b.pos) + 150
                    self.later(t, lambda s=s: setattr(s, "tender", None))
                    self.busy[c.kind] += t
                    return t
        self.idle[c.kind] += 150
        return 150

    def think_maker(self, c):
        r = self.repair(c) if c.kind != "daot" else 0
        if r:
            return r
        craft = KINDS[c.kind]["craft"]
        for b in sorted(self.built(), key=lambda b: dist(c.pos, b.pos)):
            if b.crafter is not None:
                continue
            for item in B[b.name].get("makes", []):
                it = ITEMS[item]
                if it["skill"] != craft or c.skill(craft, self) < it["craft"]:
                    continue
                if b.keeps.get(item, 0) <= b.stock[item]:
                    continue
                if not all(b.stock[k] - b.res[k] >= v for k, v in it["inputs"].items()):
                    continue
                for k, v in it["inputs"].items():
                    b.stock[k] -= v
                b.crafter = c.id
                t = self.walk(c, b.pos) + it["time"]
                c.gain(craft, it["craft"] + 10, self)

                def done(b=b, item=item):
                    b.stock[item] += 1
                    b.crafter = None
                    self.crafted[item] += 1
                    self.note(f"first {item} made")
                self.later(t, done)
                self.busy[c.kind] += t
                return t
        self.idle[c.kind] += 150
        return 150

    def think_hero(self, c):
        slots = KINDS[c.kind]["slots"]
        # bold toads want a spear first, timid ones a sling
        if c.kind == "toad":
            slots = ("sling", "spear") if c.trait == "timid" else ("spear", "sling")
        for slot in slots:
            real = "shovel" if slot == "shovel2" else slot
            cur = ITEMS[c.gear[slot]]["lvl"] if slot in c.gear else 0
            best = None
            for b in self.built():
                for item in B[b.name].get("makes", []):
                    it = ITEMS[item]
                    if it["slot"] == real and it["lvl"] > cur and it["price"] <= c.purse and b.stock[item] > 0:
                        if best is None or it["lvl"] > ITEMS[best[1]]["lvl"]:
                            best = (b, item)
            if best:
                b, item = best
                b.stock[item] -= 1
                b.stock["dew"] += ITEMS[item]["price"]
                c.purse -= ITEMS[item]["price"]
                c.gear[slot] = item
                self.sold[item] += 1
                self.note(f"first {item} bought")
                t = self.walk(c, b.pos) + 10
                c.pos = c.home.pos
                return t + dist(b.pos, c.home.pos) * WALK
        return 300

    # -- run
    def run(self, horizon=HORIZON):
        self.pend = []
        snaps = []
        while self.t < horizon:
            if self.t % 15 == 0:
                self.grow()
            self.elder()
            if self.pend:
                due = [p for p in self.pend if p[0] <= self.t]
                if due:
                    self.pend = [p for p in self.pend if p[0] > self.t]
                    for _, fn in sorted(due, key=lambda p: p[0]):
                        fn()
            while self.q and self.q[0][0] <= self.t:
                _, _, _, c = heapq.heappop(self.q)
                dt = self.think(c)
                self.schedule(c, self.t + max(1, dt))
            if self.t % (60 * TPS) == 0:
                snaps.append(self.snapshot())
            self.t += 1
        return snaps

    def snapshot(self):
        hall = self.blds["Hall"]
        kinds = Counter(c.kind for c in self.creatures)
        return dict(t=self.t, dew=hall.stock["dew"], fungiwood=sum(b.stock["fungiwood"] for b in self.blds.values()),
                    ingots=sum(b.stock["cap ingot"] for b in self.blds.values()), kinds=dict(kinds),
                    blds=sum(1 for b in self.blds.values() if b.done))


def clock(t):
    s = t // TPS
    return f"{s // 60:2d}:{s % 60:02d}"


MILESTONES = [
    "Burrow built", "Chorus Log built", "Toad Hall built", "first toad", "first slingshot bought",
    "first iron spear bought", "Ironcap Cave built", "first cap ingot picked", "Fairy Ring built",
    "first spotted cap picked", "Star Pond built", "first axolotl", "first dew bead bought",
    "first chanfron bought", "first venomwood spear bought", "Spawning Pool built", "first bullfrog",
    "Daot's Grotto built", "first glow charm bought", "first star pearl bought", "first venomiron spear bought",
    "Olm's Cave built", "first olm",
]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int)
    ap.add_argument("-v", action="store_true")
    ap.add_argument("--bare", action="store_true", help="start with the Hall alone, as RockCollector does")
    a = ap.parse_args()
    seeds = [a.seed] if a.seed is not None else [1, 2, 3, 4, 5]
    results = []
    for s in seeds:
        global BUILD_ORDER
        BUILD_ORDER = list(FULL_ORDER)
        w = World(s, verbose=a.v, scenario=not a.bare)
        snaps = w.run()
        results.append((s, w, snaps))
        if a.v:
            print("\n".join(w.log))
            print()
            for sn in snaps[::5]:
                print(f"{clock(sn['t'])} dew {sn['dew']:4d}  fungiwood {sn['fungiwood']:3d}  ingots {sn['ingots']:3d}  "
                      f"buildings {sn['blds']:2d}  {sn['kinds']}")
            inc = [w.hall_income[m] for m in range(45)]
            print("dew into the Hall per minute:", inc)
            print("sold:", dict(w.sold))
            for c in w.creatures:
                if KINDS[c.kind]["role"] == "hero":
                    print(f"  {c.kind:9s} {c.trait:7s} hatched {clock(c.born)} purse {c.purse:3d} gear {c.gear}")
            print("made:", dict(w.crafted))
            tot = {k: w.idle[k] / (w.idle[k] + w.busy[k]) for k in w.idle if w.idle[k] + w.busy[k]}
            print("idle share:", {k: f"{v:.0%}" for k, v in tot.items()})
    print(f"\n{'milestone':34s}" + "".join(f"  seed {s:<3d}" for s, _, _ in results))
    for m in MILESTONES:
        row = f"{m:34s}"
        for _, w, _ in results:
            row += f"  {clock(w.first[m]) if m in w.first else '   --':>8s}"
        print(row)


if __name__ == "__main__":
    main()
