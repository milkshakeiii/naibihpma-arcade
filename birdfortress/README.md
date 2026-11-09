# Bird Fortress

A DOS-style side-view city builder built on the concept of
[milkshakeiii/birdfortress2](https://github.com/milkshakeiii/birdfortress2), in the line of
*Pharaoh* and *Zeus*. Tribbles, dodos and apes settle wild country, work, haul and grow a town,
and raise the Bird Fortress. **What a creature carries decides how high it can jump, how fast it
walks and what it can climb**, so each species sees a different map, and that map changes with its
load.

Rendering, palette, cadence and the mouse pointer follow the repo's [AGENTS.md](../AGENTS.md)
rules: a 400x300 indexed framebuffer, a 64-colour VGA palette, 15 FPS, exact integer scaling, F2
scanlines and a pixel-art hardware cursor.

```bash
cargo run --release              # fullscreen
cargo run --release -- --windowed
```

Round 3 replaced the earlier route-puzzle game with this city builder. [DESIGN.md](DESIGN.md) has
the framework for depth and content that drove the change, and the design.

## How it plays

Real time, with pause and 1x, 2x, 4x and 8x speeds. A day is 8 seconds at 1x. A year is four
six-day seasons.

- **Homes:** burrows (tribbles), nests (dodos) and lodges (apes). Settlers walk in from the trail
  at the map's edge, so a home its species can't reach stays empty. The first settlers come anyway;
  after that a home fills only while it has food. Run out for two days and someone leaves.
- **Every resident is a worker.** Each workplace hires one species, which you choose. A new hire
  walks over from a home, so a workplace that no home of that species can reach never gets staffed.
- **Tax:** homes pay seeds every day, and you build with seeds.
- **Delivery is automatic,** like *Pharaoh*'s cart pushers:
  - A worker with a good takes it to the nearest place that wants it, within 100 ticks of walking,
    going by its own species' map with that load.
  - It goes to homes, workshops and the fortress first, stores last. Stores never feed stores.
- **Stores** hold 24 goods, 8 of a kind. Their carriers supply everyone in reach. A store told to
  GET a good fetches it from other stores and takes nothing else; that's how goods travel far.
  REFUSE keeps a good out.
- **Camps** go on a source: a berry bush, nut tree, fishing spot, twig pile, quarry or clay bank.
  Sources run down and grow back.
- **Workshops:** the weaver makes wicker from 2 twigs, the kiln a pot from clay and a twig, and the
  mason a brick from a stone.
- **Housing tiers:**
  - Shack: food.
  - Cottage: food and wicker.
  - House: two kinds of food, wicker and pots.
  - Manor: all that, bricks, and a pleasant spot.

  More tier, more residents, more tax. Run short and a home slips back.
- **Pleasantness:** kilns, masons, quarries and clay pits are noisy for 3 tiles around. Gardens are
  pleasant.
- **Seasons:** berries and fish stop growing in winter. Nut trees fruit only in autumn.
- **The fortress** goes up in stages, each wanting particular goods. Bricks go to manors too.
- **Infrastructure** changes the species' maps:
  - Ladders: apes climb with anything, others only empty or light. A ladder can be lashed to a vine.
  - Planks.
  - Springboards: +2 jump.

## Movement

| | Empty | Light | Medium | Heavy | Home holds |
|---|---|---|---|---|---|
| Tribble | jump 3, pace 1 | J3 P1 | J1 P3 | J0 P8 | 3 to 8 |
| Dodo | J2 P1 | J2 P1 | J2 P2 | J1 P4 | 2 to 6 |
| Ape | J2 P1 | J2 P1 | J2 P1 | J1 P2 | 2 to 5 |

- **Weights:** berry, wicker and pot are light; nut, fish, twig, clay and brick are medium; stone is
  heavy.
- **Pots are fragile:** whoever carries one won't fall more than a tile.
- **Clouds** hold a creature only when it carries nothing or something light. Heavier loads fall
  through.

## Sites

12 sites in five chapters, from 40 to 128 tiles wide. The map scrolls with the arrows, the mouse
wheel, the screen's edge or the minimap in the header.

| Chapter | Sites |
|---|---|
| I. Meadow | Settlers, Heavy Stone, Cottages |
| II. Quarry | The Mesa, Long Haul, Kiln Country |
| III. Canopy | First Winter, The Canopy |
| IV. Bastion | Manor Lane, The Bastion |
| V. Skyreach | Cloudwalk, Skyreach |

Each site has:

- **Goals:** the fortress plus population or housing-tier targets.
- **Gold, silver and bronze days.** Gold is within 10% of the best plan the optimiser found, so
  it's a hard target.
- **A challenge**, such as "never more than 2 stores", "no apes" or "no home ever slips".

## Teaching

- **The first site has a coach.** It points at each step: a burrow near the trail, a camp on a
  berry bush, then one on a twig pile for the fortress.
- **Site cards** show what's new with icons, then the goals, medal days and challenge.
- **One-time tips** appear at the building they're about: hunger, no free workers, a workplace no
  home can reach, goods nobody in reach takes, the first winter, autumn nuts, moving up a tier,
  slipping back.
- **Every building says what it's waiting for** on its card and in the status line. A marker over
  it shows what it lacks: a red-edged box with the missing good, or the species it can't hire.
- **Picking a workplace tool shows its reach.** Dots mark everywhere that species can walk from the
  spot within delivery range; clicking a workplace shows the same.
- **Rules (F1):** six short pages.

## Controls

- **Mouse:** the left button builds, or looks at something. The right button puts the tool away,
  or closes the card.
- **Keys:**
  - 1-9 pick tools; X demolishes (half the seeds back).
  - T, D and A choose who a workplace hires.
  - Space pauses; Tab changes speed.
  - Arrows scroll; H jumps back to the trail.
  - F1 rules, F2 scanlines, F3 music level.

Everything also has an on-screen button.

Leaving a site keeps the run. On return the site card offers to go on from the same day: the run
is saved as its list of actions and replayed.

## Balancing tools

The colony is deterministic and fast to simulate headlessly: 65 days of the biggest site run in
about 0.2 seconds.

```bash
cargo run --release -- --check            # every site's bot plans, the day they finish, the medals
cargo run --release -- --plan 2-1         # one plan with a day-by-day log
cargo run --release -- --dump 2-1 30      # every building and creature after 30 days of the plan
cargo run --release -- --opt 2-1 600      # hill-climb the plan's order, hiring and length
cargo run --release -- --map 2-1          # the map with coordinates
cargo run --release -- --shots screenshots
python tools/sites.py                     # regenerate src/sites/ch*.txt
```

Sites are generated by `tools/sites.py`: terrain helpers, sources, goals and the bot's build
plans. `tools/best_plans.txt` keeps the optimiser's best plan per site. The medals are derived
from it, and the tests run it to a win. [BALANCE.md](BALANCE.md) has the method and findings.

## Files

- `birdfortress_save.txt` holds progress, best days, challenges and runs in progress.
- `birdfortress_playtest.log` records every build, stage, win and leave with timestamps
  (gitignored).
- Music is the s5 *Virtual History* masters: *A Myth* on the menus, then one era per chapter. The
  files are read from `assets/music` or the album folder, and the game runs silently if they're
  missing.
- `assets/art/` holds the title painting and chapter backdrops. They were painted by an image
  model and converted to indexed, dithered images by `tools/convert_art.py`.
