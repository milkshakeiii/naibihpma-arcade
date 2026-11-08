# Bird Fortress

A DOS-style logistics optimization game built on the concept of
[milkshakeiii/birdfortress2](https://github.com/milkshakeiii/birdfortress2): a side-view colony
where tribbles, dodos and apes haul goods, and **what a creature carries decides how high it can
jump, how fast it walks and what it can climb**. Rendering, palette, cadence and the mouse pointer
follow the repo's [AGENTS.md](../AGENTS.md) rules: a 400x300 indexed framebuffer, 64-colour VGA
palette, 15 FPS, exact integer scaling, F2 scanlines and a pixel-art hardware cursor.

```bash
cargo run --release              # fullscreen
cargo run --release -- --windowed
```

## Fun thesis

Every creature sees a different map, and that map changes with its load. You lay out a colony
(depots, workshops, springboards, ladders, planks) on hand-designed terrain. You hire crews for
two-stop routes and watch it run. Then you shave the three scores, each of which pulls toward a
different design:

- **Ticks**: how fast the fortress gets everything. Rewards parallel supply chains and fast
  species, and is capped by regrowing sources, single docks and workshop batch times.
- **Cost**: seeds spent on creatures and buildings. Rewards cheap tribbles routed cleverly, and the
  odd springboard instead of a pricier creature.
- **Crew**: total creatures. Rewards direct routes and infrastructure over relays.

## Rules in brief

| | Empty | Light | Medium | Heavy | Cost |
|---|---|---|---|---|---|
| Tribble | jump 3, pace 1 | J3 P1 | J1 P3 | J0 P8 | 2 |
| Dodo | J2 P1 | J2 P1 | J2 P2 | J1 P4 | 3 |
| Ape | J2 P1 | J2 P1 | J2 P1 | J1 P2 | 5 |

- **Jump:** rise up to J tiles, move up to J+1 tiles sideways at the top, then fall. Walking off an
  edge falls any height. Falling costs 1 tick a tile, climbing twice the pace.
- **Climbing:** apes climb vines and ladders with anything. Tribbles and dodos use ladders only
  empty or light, and never vines.
- **Goods:** berry, egg, wicker and pot are light; twig, clay and brick are medium; stone is heavy.
  Eggs and pots are fragile, so their carrier never falls more than 1 tile.
- **Workshops:** the mason turns 1 stone into 1 brick (8 ticks). The weaver turns 2 twigs into 1
  wicker (10 ticks). The kiln turns 1 clay and 1 twig into 1 pot (12 ticks).
- **Routes:** a creature waits at its pickup, takes one good when the drop-off has room, carries it
  the fastest way it can, hands it over, and walks back empty (often by another way). Every place
  has one dock (the fortress has two); a pickup or handover holds it for 2 ticks.
- **Infrastructure:** a springboard adds +2 jump to anyone leaping from it. A ladder costs 1 seed
  per rung. A plank is a one-way board that costs 2 seeds a tile.

## Levels

19 sites in four chapters. Chapters I to III teach one rule per site and finish with an open
puzzle; chapter IV is three full optimization puzzles.

| Chapter | Sites |
|---|---|
| I. Meadow | First Haul, The Bluff, Berry Patch, Handoff, Springboard, **Meadow Market** |
| II. Quarry | Heavy Stone, Stonecutter, Ladders, **Quarry Road**, **The Crag** |
| III. Canopy | Eggshells, The Weaver, Bridges, The Kiln, **The Great Tree** |
| IV. Bastion | **Twin Rivers**, **The Eyrie**, **Bird Fortress** |

Each site has a goal for each score. A goal earns a star, and each best is kept separately. The
results screen plots your score against a histogram of every distinct solution the solver found.

## Controls

The left mouse button uses the selected tool and the right button cancels or erases. **1-9** pick
tools, **X** erases, **Space** runs or pauses, **S** steps one tick, **Tab** changes speed, **R**
resets, **Z** undoes, **V** shows all paths and **I** shows the site note again. **F1** opens the
rules, **F2** toggles scanlines and **F3** sets the music level. Everything also has an on-screen
button.

## Balancing tools

The simulation is deterministic and headless, so levels are designed and balanced from the
command line:

```bash
cargo run --release -- --check            # every level's reference solution still works
cargo run --release -- --show 2-4         # ASCII map with each route's loaded (*) and empty (o) path
cargo run --release -- --show 2-4 "B MASON 2 18;R TRIBBLE s1 b0 STONE 1;..."
cargo run --release -- --solve 2-4 80 300 # random starts + hill-climbing per score; prints the frontier
cargo run --release -- --histo            # regenerate src/levels/histo.txt for the results screen
cargo run --release -- --shots screenshots
```

Levels live in `src/levels/ch*.txt` as 40x24 ASCII maps (see `level.rs` for the legend). The
solver's best solutions for each score are written as comments into `histo.txt`. The goals are set
from them.

## Files

- `birdfortress_save.txt` holds progress, per-score bests and your last layout per site.
- `birdfortress_playtest.log` records every run, solve and stall with layouts (gitignored).
- Music is the s1 *Abstract Objects* masters, read from `assets/music` or the album folder. The game
  runs silently if they're missing.
