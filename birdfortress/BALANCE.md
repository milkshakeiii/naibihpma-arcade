# Balance log (spoilers)

This is how the sites were tuned. **It contains solutions, so don't read it before playing.**

## Method

The simulation is deterministic, so every site was checked headlessly:

1. `--show ID "layout"` draws each route's loaded (`*`) and empty (`o`) path in ASCII. Each map was
   iterated until the intended constraint actually bit. For example, in 2-1 the vine had to sit
   where an ape can reach its foot, and in 3-3 and 4-x the water had to drop one row so planks can
   attach to the banks.
2. `--solve ID starts climbs` builds random layouts (random sources, 0-2 relay depots per leg,
   workshops, infrastructure, and a species that can run each leg) and then hill-climbs each score
   separately. It reports the best layout per score and the Pareto front.
3. `--histo` runs the solver per chapter and stores per-score histograms of every distinct valid
   layout it found (about 30 for 1-1, about 8,000 for 1-3). The results screen draws them.
4. The goals are the solver's best per score (ticks rounded up to a multiple of 5). They are
   replaced by a hand-found layout where one beats the solver.

## Findings

- **Ticks are always a crew-heavy corner.** The fastest layouts use 6 to 12 creatures. The supply
  limits (regrowing bushes and piles, one dock per place, workshop batch times) keep this from
  being pure spam: past the supply rate, extra creatures only queue. The fastest designs pull from
  several sources and use parallel workshops.
- **Cost and crew usually share an optimum on the teaching sites** (1-1 to 1-5, 2-1, 3-1). They
  split on the open sites, where infrastructure trades seeds for fewer relays.
- **The solver found tricks I didn't plan.** In 3-3 it bridged the lake with planks on every other
  tile; jumps clear the 1-wide holes, so the bridge costs 18 instead of 26. In 4-3 two planks act
  as stepping stones across a 4-wide stream.
- **I found one trick the solver missed.** In 2-3, a springboard on the mesa's step replaces the
  upper ladder (11 seeds instead of 14), so the cost goal is 11. A gold "<" on the results screen
  means you beat every solution the solver found.
- **Species niches as tuned:**
  - Tribbles own light goods (jump 3, pace 1, cheapest) and the empty return up 3-high steps.
  - Dodos own medium goods up 2-high steps.
  - Apes own heavy goods and anything up vines or ladders.
  - Every species is the best choice on at least one leg somewhere.

## Round 2 (2026-11-08)

- **Distinct valid solutions per site** (one solver pass, 40 random starts with 150 hill-climbs per
  score):

  | Chapter | Counts |
  |---|---|
  | I | 27, 103, 1559, 589, 1999, 1664 |
  | II | 28, 4407, 115, 509, 1265 |
  | III | 102, 361, 106, 144, 186 |
  | IV | 965, 206, 198 |
  | V | 416, 5959, 1416 |

  The lowest are the one-rule teaching sites (1-1, 2-1), which is fine.
- **Ticks-cost frontiers have 3 to 21 points.** Each site's combo goal is the knee: the point
  closest to the best of both after normalising, excluding the two extremes.
- **Clouds** split each trip by load. In 5-1, a dodo with twigs must go down and around under the
  clouds (78 ticks) but walks home across them in 30. Weaving on the near island makes the goods
  light enough to cross both ways. Where the weaver goes is the site's main decision.
- **Moonwell's fastest layouts** pull eggs from both nests and bricks from three apes at once (49
  ticks). The cheapest uses one tribble per job and a single ape on the vine (14 seeds).

## Open questions for the next round

- Should one site cap the crew, so the ticks goal can't be reached by headcount alone?
- Workshops could take a creature to run them (birdfortress2 had worker slots). That would add a
  labour trade-off.
- Houses and food from birdfortress2 (creatures come from nests that must be fed) aren't in yet.
  They would turn some sites into economy puzzles rather than pure logistics.
