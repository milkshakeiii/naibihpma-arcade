# Bird Fortress design notes

Round 3 (2026-11-08). This file has three parts:

1. A framework for "depth" and "content".
2. Why rounds 1 and 2 fell short against that framework.
3. The new design, a side-view city builder in the line of *Pharaoh* and *Zeus*.

## 1. A framework for depth and content

### What players mean

- **"It has depth"**: I keep finding better ways to play. My choices push on each other. A new map
  makes me change my plan. When I lose, I can see what I'd do differently. After a win, I can
  still see room to play better.
- **"It's trivial"**: once I've looked at the map, the answer is obvious. Every part can be solved
  on its own. There's one right answer, and it's the first one I try.
- **"Interesting content"**: new situations that break my old habits. Not new skins on the same
  problem, and not one gimmick per level that I spot and dismiss.

### What designers mean

- **Depth is not complexity.** Complexity is what the player pays to learn and track the rules.
  Depth is what that buys: a large space of good, different strategies, and room to keep
  improving. The aim is the most depth for the least complexity
  ([Game Developer, "What is depth"](https://www.gamedeveloper.com/game-platforms/what-is-depth-and-how-do-you-add-it-to-your-game-);
  [Justin Gary, "Designing for Elegance"](https://justingarydesign.substack.com/p/designing-for-elegance);
  ["Depth from complexity"](https://gamedeveloper.com/design/depth-from-complexity-pt-2)).
- **Interesting decisions.** A choice is interesting when it's neither obvious nor a blind guess:
  "should I", not "can I". Interesting decisions come from one coherent system, not from "a
  collection of semi-related half-systems" (Keith Burgun in
  [Three perspectives on strategy game design](https://www.gamedeveloper.com/design/three-perspectives-on-strategy-game-design)).
- **Emergent depth over content depth.** Depth should come from how the rules interact, not from
  piling up maps and items ([Burgun's Clockwork criteria](https://gamedeveloper.com/design/criteria-for-strategy-game-design)).
- **Transparency.** In a rules-driven game, players must be able to see the rules, their inputs
  and the outcomes, or they can't reason about any of it (Soren Johnson, same article).
- **City builders get their depth from interlocking systems.** A placement decision affects
  population, the economy, services and transport all at once, so every choice spreads
  ([Bugnet, "How to design a city builder"](https://bugnet.io/blog/how-to-design-a-city-builder)).
- **The Impressions games** (*Caesar III*, *Pharaoh*, *Zeus*) build all of this on a few systems:
  - **Housing evolution:** a house climbs a ladder of tiers while its needs (goods, services,
    nice surroundings) are met, and falls back when they lapse.
  - **Walkers:** service and delivery walkers make *where* you build matter.
  - **A shared labour pool:** it couples every building to every other.
  - **Production chains** feed both houses and monuments.
  - **Seasonal events** (the Nile flood) force you to plan ahead
    ([Akhenaten wiki, housing tiers](https://dalerank.github.io/Akhenaten/wiki/player/buildings/housing.html);
    [Impressions City Building series](https://gamicus.fandom.com/wiki/City_Building_Series)).

### The checklist used here

Every scenario and mechanic is judged against these. Each one can be measured with the headless bot.

| Lever | Question | How it's checked |
|---|---|---|
| **Coupling** | Do decisions share constraints, so the best choice here depends on choices there? | Labour, seeds, storage and sources are all shared. Optimising parts separately should lose to optimising the whole. |
| **Compounding** | Does investing early pay back later, so build order matters? | Two orderings of the same buildings should finish days apart. |
| **Tension over time** | Does the colony face pressure that forces planning ahead? | Winter shuts some food sources, houses decay when unsupplied, and the fortress competes with houses for bricks. |
| **Spatial trade-offs** | Does one placement pull several ways at once? | Near the food or near the quarry? Close to the houses, or far enough away to keep the noise off them? |
| **Plural strategies** | Are there two or three different good plans? | The bot finds plans that are close in days but different in shape: tribble swarm vs ape economy, one hub vs many. |
| **Readability** | When something fails, can the player see why? | Every building says what it's waiting for. A reach overlay shows each species' map. |
| **Scale** | Is there enough to decide? | Big sites take 15 to 40 minutes and 40 to 120 placements, and the plan gets revised mid-game. |

**Content** means a new *situation*, not a new skin: a map whose geography makes a different plan
best, or a mechanic that multiplies the existing ones (clouds × load weight was a good one;
"this level has a springboard" was not). Each site gets one central tension that it states only
as a goal, never as a hint.

## 2. Diagnosis of rounds 1 and 2

The playtest log, sorted by the checklist:

- **Sites were solved in 20 to 80 seconds**, usually on the first run, with 1 to 3 routes.
- **No coupling.** Each leg of a route was its own small puzzle, and choosing a species was a
  lookup ("heavy means ape"). Nothing tied one route to another.
- **No compounding and no tension over time.** You set it up and watched. Nothing grew, decayed or
  changed with the seasons.
- **Scores were judged after the fact, not felt during play.** Seeds were a score, not a budget,
  so there was no scarcity to force a choice. The fastest layouts just added creatures.
- **One gimmick per site.** Springboard sites asked for a springboard, and ladder sites asked for a
  ladder.

The good parts to keep:

- The species-and-load movement model (every creature sees a different map).
- The hand-drawn side-view terrain, the art, the sound and the DOS presentation.

## 3. The new design

**Fantasy.** You lead a colony of tribbles, dodos and apes into wild country. You grow it from a few
nests into a town and raise the Bird Fortress, the monument each site is named for.

**Fun thesis.** Each species sees a different map, and that map changes with its load. The places
you put homes, camps, stores and workshops decide who can supply whom. The colony's growth spiral
decides how fast the fortress rises. The main questions are: which species do you house, where do
you put the hubs, when do you stop growing and push the fortress, and how do you get through
winter?

**Real time with pause**, like *Pharaoh*: you build while the colony runs. Pause, 1x, 2x, 4x and 8x
are on-screen buttons. A day is 120 ticks. A year is four seasons of six days each.

### Systems

1. **Nests and immigration.** A burrow (tribbles), nest (dodos) or hut (apes) is a home. Settlers
   walk in from the trail at the map's edge, so **a home their species can't reach stays empty.**
   The first residents arrive on their own. After that a home fills only while it has food.
2. **Housing evolution.** A home's tier sets how many live there and how much tax they pay.
   - Hut: food.
   - Cottage: food and wicker.
   - House: two kinds of food, wicker and pots.
   - Manor: two kinds of food, wicker, pots and bricks, plus a pleasant spot.

   A home that loses something it needs slips back a tier after two days. A starving hut loses a
   resident each day.
3. **Labour.** Every resident is a worker of its species. Each workplace hires one species, chosen
   by the player, and a new hire walks there from a home, so **a workplace that no home of that
   species can reach never gets staffed.** When there aren't enough workers, the newest workplaces
   go short first.
4. **Seeds.** Homes pay tax every day, more at higher tiers. Everything costs seeds, so the economy
   has to be built before the fortress.
5. **Walkers deliver on their own** (*Pharaoh*'s cart pushers):
   - A worker with a good looks for the nearest place that wants it, within a walking range,
     **going by its own species' map with that load**.
   - It carries the good there and walks back empty, by another way if it has to.
   - Consumers (homes, workshops, the fortress) come before stores, and stores never feed stores.
6. **Stores** hold 24 goods, 8 of a kind. Their carriers supply consumers within range. Each good
   has a setting: *accept*, *refuse*, or *get*. *Get* sends the store's own carriers to fetch that
   good from other stores, which is how you relay goods over long distances. A store with any *get*
   order takes only what it gets, so a relay stays clean.
7. **Camps** claim a berry bush, nut tree, fishing spot, twig pile, quarry or clay bank. Their
   workers harvest there and carry the harvest out. Sources run down and regrow, so a big colony
   needs many of them.
8. **Workshops**: the weaver makes wicker from 2 twigs, the kiln makes a pot from clay and a twig,
   and the mason makes a brick from stone. Each needs a worker inside to craft, and inputs
   delivered by others.
9. **Seasons.**
   - Berries grow from spring to autumn.
   - Nuts grow only in autumn.
   - Fish stop in winter.

   Winter has to be stockpiled for.
10. **Pleasantness** (from chapter IV). Workshops, quarries and clay pits are noisy for 3 tiles
    around them. Gardens are pleasant. A manor needs a pleasant spot, so industry wants to sit
    close to the homes for short walks but far from them for quiet.
11. **The fortress** is built in stages, each wanting specific goods. Bricks go to manors too, so
    the monument competes with the town.
12. **Infrastructure** works as before: ladders, planks and springboards change each species' map.

### Goals

Each site has a goal list. For example: finish the fortress and house 40. You win when every goal
is met, and you're rated by the day you finish.

- **Gold, silver and bronze days** are set from the best plan the bot found. Gold is a hard
  target.
- **One challenge per site** limits how you play, for example "no more than 2 stores" or "no apes".

### Readability

- **Every building says what it's waiting for**, in one short line: "Wants twigs. No dodo can reach
  a twig pile from here."
- **Choosing a species shows its map.** While placing or inspecting a workplace, the reach overlay
  shows where that species can go, empty and loaded.
- **Hungry or slipping homes** show the missing good over the roof in a red-edged box. It's
  static, with no flashing.
- **The header shows** seeds, the date and season, workers per species (busy/total) and the
  goals.
- **Just-in-time tips** appear at the building they're about. They teach rules, never the solution.

### How it's balanced

The colony is deterministic, so every site keeps one or more **bot plans**: ordered build lists
that run as soon as each step is affordable. The `--plan` harness prints a day-by-day timeline
(population, seeds, stage progress). An optimiser reorders the steps and swaps species to find the
best day, and that sets the medals. A test checks that every site's plan finishes.

## Rounds log

Each round names its additions and what to look for. Verdicts are the player's to give.

### Round 3: city builder (this round)

| Addition | Goal it serves | What to look for |
|---|---|---|
| Homes, settlers, labour | Coupling | Does running short of one species change what you build next? |
| Tax economy | Compounding | Does build order feel like it matters? |
| Housing tiers | Depth, coupling | Do you weigh growing the town against feeding the fortress? |
| Automatic walkers within range | Spatial trade-offs | Do store and workshop placements feel like real decisions? |
| Store orders (get/refuse) | Plural strategies | Are long-distance relays understandable? |
| Seasons | Tension over time | Is winter a planning problem rather than a surprise? |
| Pleasantness | Spatial trade-offs | Does the noise-vs-distance pull read clearly? |
| Fortress stages, bigger maps | Scale | Do sites take long enough to have a middle game? |

### Changes made while balancing

Each came from a bot run that stalled for a reason a player would also hit; [BALANCE.md](BALANCE.md)
has the details:

- Stores take 8 of a kind.
- Relay stores take only what they get, and lend down a gradient.
- Camps on renewable sources hire one worker.
- Ape lodges hold more.
- Taxes went down and prices up.
- Workshops craft while their workers deliver.

### Parking lot

- Lifts and moving platforms (birdfortress2 had both).
- Trade caravans that buy surpluses.
- Service walkers (a roaming healer or entertainer) for higher tiers.
- Saving mid-site.
