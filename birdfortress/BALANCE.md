# Balance log (spoilers)

This is how the sites were tuned. **It describes working plans, so skip it if you want to find
your own.**

## Method

The colony is deterministic, so every site was built and checked headlessly:

1. **Maps** come from `tools/sites.py`, which builds terrain from helpers (ground, steps, rivers
   one row below the banks, cloud bridges, vines). `--map ID` prints a map with coordinates.
2. **Plans:** each site has a hand-written bot plan, an ordered build list run as soon as each step
   is affordable. `--plan ID` prints a day-by-day line: seeds, residents, workers per species, homes
   per tier, fortress progress, and every building that's stuck and why. `--dump ID DAYS` lists
   every building and creature. Most map fixes came from reading those notes.
3. **The optimiser** (`--opt ID ITERS SEED`) hill-climbs a plan: it swaps steps, moves steps
   earlier, changes who a workplace hires, and drops steps. A change is kept if the plan still
   wins, no later. 600 iterations per site, all sites in parallel, took a few minutes.
4. **Medals:** gold is the optimiser's best day x 1.1, silver x 1.4 and bronze x 1.9 (rounded up).
   The bot builds the moment it can afford to and only at fixed spots, so a player who places better
   can beat gold, but a player who hesitates won't reach it.
5. **The tests** run every site's best plan to a win before its bronze day. They also check that
   every source, trail and fortress stands on a floor.

## Bot results (round 3)

| Site | Hand plan | Optimised | Medals (gold/silver/bronze) |
|---|---|---|---|
| 1-1 Settlers | 11 | 7 | 8 / 10 / 14 |
| 1-2 Heavy Stone | 25 | 16 | 18 / 23 / 31 |
| 1-3 Cottages | 26 | 14 | 16 / 20 / 27 |
| 2-1 The Mesa | 42 | 29 | 32 / 41 / 56 |
| 2-2 Long Haul | 41 | 32 | 36 / 45 / 61 |
| 2-3 Kiln Country | 115 | 30 | 33 / 42 / 57 |
| 3-1 First Winter | 52 | 36 | 40 / 51 / 69 |
| 3-2 The Canopy | 92 | 36 | 40 / 51 / 69 |
| 4-1 Manor Lane | 36 | 29 | 32 / 41 / 56 |
| 4-2 The Bastion | 62 | 57 | 63 / 80 / 109 |
| 5-1 Cloudwalk | 27 | 20 | 22 / 28 / 38 |
| 5-2 Skyreach | 104 | 65 | 72 / 91 / 124 |

The gap between the hand plans and the optimised ones is the "skill ceiling" the framework asks
for: the same buildings in a better order, with better hiring, finish weeks earlier. The biggest
gaps came on sites with hard ordering choices:

- **Kiln Country:** the hand plan built fish camps the town couldn't use yet.
- **The Canopy:** the optimised plan reaches the larder stage before the first autumn.

## Checking the framework: does build order matter?

DESIGN.md's "compounding" lever says two orderings of the same buildings should finish days
apart. `--spread 40` takes each site's best plan and runs the same buildings in 40 random orders.
Planks and ladders keep their relative order, and store orders go last so every plan is valid.

| Site | Best | Random orders: min / median / max | Never finish (of 40) |
|---|---|---|---|
| 1-1 | 7 | 7 / 10 / 16 | 0 |
| 1-2 | 16 | 24 / 36 / 99 | 2 |
| 1-3 | 14 | 15 / 18 / 329 | 34 |
| 2-1 | 29 | 40 / 77 / 174 | 2 |
| 2-2 | 32 | 58 / 67 / 102 | 37 |
| 2-3 | 30 | 33 / 57 / 279 | 13 |
| 3-1 | 36 | 58 / 91 / 225 | 10 |
| 3-2 | 36 | 56 / 86 / 200 | 4 |
| 4-1 | 29 | 42 / 91 / 310 | 24 |
| 4-2 | 57 | none finished | 40 |
| 5-1 | 20 | 40 / 140 / 332 | 30 |
| 5-2 | 65 | 114 / 207 / 330 | 19 |

Only the tutorial site barely cares about order. Everywhere else a random order of the right
buildings takes two to four times as long, or stalls for good, so order is where a lot of the
skill is.

The stalls are worth reading. Workplaces hire in the order they were built, so a camp built early
can hold the only free apes while the mason that needs them never gets a worker. The bot never
pauses a building to free its workers, but a player can. Some of these stalls would be recoverable
in play. They still show the coupling the design asked for: one early choice decides what the
whole colony can do.

## What tuning changed in the rules

Each of these came from a bot run that stalled for a reason a player would also hit:

- **Stores hold 8 of a kind (24 in all).** With one shared limit, a glut of berries filled a store
  and stone had nowhere to go. A per-good limit keeps one glut from blocking the rest. Orders still
  matter for relays.
- **A store with a GET order takes only what it gets.** Passing camps were filling relay stores
  with twigs and pots until bricks couldn't get in. This rule makes a relay store clean by default.
- **Relays lend downhill.** A GET store will fetch from another GET store only if that store holds
  clearly more (at least 2 more). Goods flow down a chain and never bounce between two stores.
  Without this, multi-hop relays (Skyreach) never moved anything.
- **Camps on bushes, trees, fishing spots and twig piles hire one worker; quarries, clay banks and
  nut trees hire two.** Slow renewable sources were wasting labour.
- **Ape lodges hold 2/3/4/5** (by tier), up from 1/2/3/4. Sites that need vines were short of apes
  for weeks.
- **Taxes went down and prices up.** Before, seeds stopped mattering by day 10; now the economy
  stays a constraint into the middle game.
- **Workshops craft whenever they have a worker hired.** Requiring a worker to be inside stalled
  every workshop whose workers were out delivering, and the status line misread it as a missing
  input.

## Findings per site

- **Heavy Stone:** one quarry is just within dodo range of the fortress (88 ticks loaded) and the
  other isn't, so the store is for the second quarry. The challenge ("no apes") is easy. It's
  there to show that dodos can do it.
- **The Mesa:**
  - Stone can't climb the 2-high steps out of the quarry hollow, since no one jumps 2 with a heavy
    load. It needs apes and ladders.
  - Bricks are medium, so dodos carry them up the steps. That makes where the mason goes the main
    decision.
  - Twigs can't go up the vine ladder with tribbles (medium), so the weaver belongs on whichever
    side its twigs are.
- **Long Haul:** the quarries are 60 tiles from the fortress. That's out of range for a stone
  carried by anyone, but in range for a brick carried by an ape (pace 1 when medium). Masons near
  the quarries replace a whole relay chain.
- **Kiln Country:**
  - Fish can't climb the gorge's sheer wall except with apes, so the houses (2 foods) end up in the
    east or get fed by apes.
  - Pots can't drop the 2-high steps, so pots need ladders, and climbing never breaks them.
- **First Winter:** food has to be in stores before day 19. The optimised plan wins on day 36,
  after surviving the first winter. The "no home ever slips" challenge is hard.
- **The Canopy:** nuts come only in autumn and only from the high branches (apes, or tribbles on
  ladders, who walk off the branch ends with them). The larder stage wants 24 at once.
- **Manor Lane:** manors need gardens next door and all the noisy industry far away. The plan puts
  industry across the lake and gardens between the western homes.
- **The Bastion:** everything at once. Bricks go to the keep and to manors alike. The binding
  constraint is twigs: wicker for every home plus the kilns.
- **Cloudwalk:**
  - Bricks can't cross clouds, so apes go down into the hollow and up the vine.
  - Pots cross clouds fine, but can't drop off the mesa onto them.
- **Skyreach:**
  - Bricks travel shelf, east hollow, vine, highland, planks, middle hollow, vine, mesa. That's
    four stores in a GET chain, exactly the challenge's limit.
  - The manors go on the highland, which has fish, nuts and quiet.

## Open questions for the next round

- Do automatic deliveries feel readable, or do players want to draw supply links?
- Is the walking range (100 ticks) the right size for the bigger maps?
- Should the fortress need builders (workers on site) as well as goods?
