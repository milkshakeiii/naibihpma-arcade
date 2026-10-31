# Monster Workshop — design

The feel prototype is the quality bar. It has no menus, yet new players learn the controls in a minute, and you can always tell what every monster is doing and why. Every feature below has to keep that.

## The rule: the lightest layer that works

1. **The world.** If something can be a place or an object you walk to, it is.
2. **Speech bubbles,** shown just in time, explain why something failed and what to do next. Monsters you aren't steering only show a compact icon ("? + cotton").
3. **The HUD** shows one objective line, one status line for the controlled monster, and a few counters.
4. **The inspect card.** Hold a key to see a small card beside whatever you're facing. Release to dismiss. It never covers the screen.
5. **Full screens** are only for things outside play: title, settings, help.

During play the target is zero full-screen menus. Every action gets visible and audible feedback. The first time you meet a system, a monster comments on it and the HUD gives one objective, the same way the prototype's tutorial works.

## Scope

**1.0** is a free itch.io build that runs locally, with no online server. It keeps renown with shares, and skill learning. Content is 2–3 production chains across 2 towns and the wilderness between them.

**Cut from 1.0:** upkeep, cost multiplier, monster aging.

**Later, a persistent world.** The end product may also offer a persistent multiplayer world. To keep that possible, the simulation stays headless and separate from rendering, as it is in the prototype. World rules work like survival-game server settings:
- **Time scale:** fast for single-player, slow for a persistent world.
- **Monsters working while you're away:** off, or catch up a capped amount of time.

### Saving
- **Worlds, like a survival game.** Each world is its own save, with its own settings (time scale, offline progress). The title screen lists your worlds; that is a full screen, which is allowed outside play.
- **What gets saved:** simulation state only:
  - monsters: position, stats, skills, routines and loop progress;
  - items: variety tags, quality, durability, raw-material history, shares;
  - plants, bins, and workshops, including crafts in progress;
  - renown, project progress, applicants and the tutorial stage;
  - the game clock and the random-number state.

  The base map comes from the game's data, so a save stores only what has changed; from M2 that includes player-built workshops. Display-only state such as particles and bubbles is not saved.
- **Format: JSON**, with a version number and migrations on load. It is readable when debugging, easy to diff, and the same snapshot can later be what a persistent-world server stores. Goods are referenced by a stable `id` (like `cotton_gin`), never by display name, so renaming a good never breaks a save. The data needs those ids first: today goods have only names, and two are both called "Saw".
- **Where:** `%APPDATA%\MonsterWorkshop\worlds\`, so an itch install folder never has to be writable.
- **When:** every 60 seconds of play, and on quit, always between ticks. Each save is written to a temporary file and then renamed, and the previous two saves are kept as backups, so a crash mid-write can't lose a world.
- **Offline progress on load:** if the world's setting allows it, the game runs the missed time headless, capped at around 8 hours, behind a progress bar, before showing the world. The simulation is deterministic per tick, so catching up just means ticking fast.
- **Tested by a round trip:** save, load, run both copies for a few thousand ticks, and check they stay identical. That also guards the simulation's determinism, which the persistent world will rely on.

### First production chain
Cotton cloth: **Cotton Bolls → Cotton Slivers → Thread → Fabric.** The production chains come from *Tian Gong Kai Wu* (Song Yingxing, 1637). New goods that fill gaps in the tech tree are proposed from that book, and Henry approves them before they go in.

Added on 2026-10-30, so the chain can be built from gathering alone:
- **Raw wood varieties**, with values and densities from the first tech tree: Elder Sticks, Rhododendron Sticks, Hemlock Log, Oak Log, and Arrow Bamboo (the book's bamboo for weft quills).
- **Five carpentry tools** from the book's cotton section (乃服·布衣):
  - Cotton Gin (趕車): two hardwood rollers, which the book compares to the sugar-cane press;
  - Cotton Bow (彈弓): its string is ramie, where the book uses ox sinew, because the tree has no animal goods yet;
  - Spinning Wheel (紡車);
  - Spool (籰);
  - Waist Loom (腰機): the book's loom for cotton cloth.

Fixes made at the same time:
- Plain Fabric no longer needs a figure pattern. The book says plain is enough for cotton cloth. Figure patterns belong to a future Figured Silk made on a Draw Loom (花機).
- Cotton Slivers needs a *cotton* bow, so it no longer shares a tag with the iron-wire bow used to cut clay tiles.
- Cotton Seeds' tag was split into letters; it now asks for "cotton gin".
- Dyed Fabric now carries its dye color, which also makes Royal Blue Dyed Fabric's input reachable.

**The source stays implicit.** Player-facing text never names the book or the real places it describes. It shapes the world from underneath, as part of the game's mystique.

### World layout (starting world)
- **Flip-screen rooms.** Each screen is 25×16 tiles. When the controlled monster crosses an edge, the view flips to the next screen, as in classic DOS games. The simulation keeps running on every screen.
- **Pinewater, the weaving town (west)**, about 2×2 screens. It has the spinnery, the loom house, the hiring hall and a public workbench, with the cotton fields right outside. M1 can be played entirely here.
- **Wilderness (middle)**, 2–3 screens of road between the towns: forest (hemlock near the road, oak deeper in), a bamboo grove, scrub and meadow (sticks, ramie), and dye plants toward the east. This is where frontier workshops go. Rarer varieties sit farther from the road.
- **Tanglemere, the dyeing town (east)**, about 2×2 screens. It has the dye works, project boards, a public workbench and the great work. Fabric travels east to be dyed, which is what makes wagons matter.
- **Later regions** feel farther away and weightier: for example, hills beyond the dyeing town with copper and tin for the Temple Bell chain.
- Its two-town shape follows the book: weaving at one town, sizing and dyeing at another.
- **Naming:** place names quietly translate the meaning of the real places behind them. Songjiang ("pine river") becomes Pinewater, and Wuhu ("overgrown lake") becomes Tanglemere. Name the wilderness and later regions the same way.

### Music
Henry composes the soundtrack. The arcade's Star Pond already streams WAV music, so that code can be reused.

**v1.5:** equipment as physical crafted wearables; charity and great-works drop-offs; and an in-world quality sorter (see below).

## Features

### Recipes
There is no recipe menu. What's on a workshop's pads decides what it makes. When two recipes take the same inputs, a signboard on the workshop wall shows the current one, and walking into the board switches it. Empty pads show a dithered ghost of what belongs there, as they do in the prototype.

### Tools
Workshops have tool pegs on the wall, which you fill by pushing tools onto them, the same way pads work. Each tool shows a small durability bar. When a tool breaks, a looping monster stops with a "? + hammer" bubble.

### Workshops: town and frontier
There are two kinds of workshop, with a clear trade-off between convenience and being close to raw materials.
- **Town workshops** are permanent: they never wear out, have a fixed quality of around 0.8, and are far from raw materials. Their pegs hold basic permanent tools of modest quality, around 0.5, so a new player can craft straight away; bringing better tools improves the results. Nobody builds inside towns, which keeps them fixed and easy to read.
- **Frontier workshops** are built by players anywhere outside town limits except on roads, for example next to a resource. Their quality is rolled when they're built, like any crafted good, so a lucky build can beat the town's. They wear out: 1,000 durability at 8 per use, about 125 crafts. You repair one by pushing materials into it.
- **Building stays in the world.** Push a blueprint onto open ground and its footprint appears as a ghost outline. Push the materials in, and a monster builds it.
- **Reasons to build:**
  - shorter physical loops, because distance is time;
  - possibly higher quality;
  - workshop shares: builders earn weight-8 shares whenever *anyone* uses their workshop.

  Wear-out also clears abandoned workshops out of a persistent world.
- **Workbenches.** Recipes that don't need a workshop use a small, cheap bench placed anywhere. Benches wear out too, and towns have a few public ones.

### Hiring
There is no character-creation screen. A **hiring hall** in town holds applicants. Each one is already rolled with a monster type and 3 transferable skills. Walk up to an applicant and their bubble shows type, skills, stats and renown cost. Space hires them. You choose which applicant to hire, not which boxes to tick, and the roster refreshes over time.

### Renown and goals
Renown is a HUD counter. You spend it in the world: hiring at the hall, and contributing to **town project boards**. A project board shows what goods it wants and a progress bar. In 1.0 boards count goods of any quality, so a low-quality good always has somewhere useful to go. Finished projects unlock things like the road to the second town or new workshops. A final "great work", for example a temple bell, ends 1.0.

### Delivery and shares
Delivering a good is a small celebration. "+12 RENOWN" rises from the drop-off, then smaller credits fly to each monster that contributed ("MOGG 6, BRUNA 4"). This is how the shares system becomes visible. All renown still goes to the one commune.

### Roster
Tab cycles monsters. A monster that's off-screen gets an **edge marker**: an arrow at the screen edge with its state icon (looping, waiting, idle), so a stuck monster is noticeable from anywhere. Holding Tab shows a strip of portraits with states in the HUD.

### Skills
Skill growth is shown as it happens. A small "+SPINNING" floats up when a skill improves, and a skilled monster visibly works faster: the progress bar fills quicker. Exact numbers are on the monster's inspect card.

### Items and quality
Quality is shown on the sprite: brighter is better, and exceptional items sparkle. Tools carry a durability bar. The inspect card gives the details.

### Varieties (carried-over tags)
Goods come in varieties, like Dwarf Fortress materials: oak or hemlock, cotton or ramie, crimson or indigo. Each recipe lists which tags carry from each input into the output, and tags keep flowing through every later step, so crimson dye on cotton fabric makes a crimson cotton fabric.
- **Each raw variety is its own good**, with its own base value, density and rarity.
- **Value and weight follow automatically.** Every good records the raw materials in its history; value adds up their base values, and weight uses their average density. An oak loom is worth more and weighs more than a hemlock one. A tool's material does not pass into what it makes; only its quality does.
- **Color comes from one chosen input** per good: dyed goods take the dye's color, thread takes its fiber's, tools take their wood's. Each variety maps to a palette ramp, and the sprite is recolored by swapping palette indices, which AGENTS.md allows.
- **Names and the inspect card** show varieties: "Oak Loom", "Crimson Cotton Fabric".
- **Gathering:** which variety a gathering spot yields is part of the world-layout discussion. Rarity is already in the data for it, and rare varieties fit the wilderness.

### The inspect card
Hold **E** ("examine") to show a card anchored next to the tile you're facing. Use moves to Space only; in the prototype E also triggers Use. If that tile is empty, the card describes the monster you're controlling.
- **Item:** name, quality, durability (for tools), carried-over tags, and who contributed to it.
- **Monster:** name, type, ability scores, the 3 transferable skills, and the top applied skills as bars.
- **Workshop (the forecast):** hold inspect while standing on the work spot. The card shows, for the monster there (or the one you control):
  - current recipe and anything still missing;
  - expected quality and its typical range, plus the chance of an exceptional or masterwork result;
  - expected quantity and time per craft;
  - learning:
    - what each skill gains per craft: the specific skill, the primary skill and each secondary skill;
    - roughly how many more crafts until each one levels off;
    - what the craft costs in forgetting across every other skill;
  - what is holding quality back most, e.g. "SECONDARY SKILLS 0.52: HANDCRAFTS OR MUSIC WOULD SKIP THESE".

  It is computed from the items actually on the pads and pegs, so putting in a better tool updates it straight away. While a monster stands on a work spot, the HUD status line also previews its expected quality.

The card is drawn in the same beveled style as the help overlay. It never pauses the world.

### Wagons and roads
Wagons are fully physical. Press Space on a wagon's handle to hitch it, and it follows behind you. Push items onto its bed, where they stack visibly, and push them off at the destination. Roads leave the map edge at signposts naming the next town. Recorded loops can include wagon trips.

### Equipment (v1.5)
Gloves, glasses and the like are crafted goods from the tech tree, not bought with renown. Push one into a monster to equip it. It then shows on the sprite (a goblin in glasses), and its bonus appears on the inspect card.

### Quality goals and sorting (v1.5)
Two kinds of drop-off, after the "charity and great works" in the original rules:
- **Great works** pay the good's value, as delivery does in 1.0. Quality matters, and lucky deep goods shine.
- **Charity** scores every good as if its quality were average (around 0.5, tunable), whatever its real quality. Below that point charity pays more than a great work, so clearing out dim goods becomes worthwhile under the right conditions. Charity goals can also be bulk goals ("200 BRICKS FOR THE TOWN WALL").

Recorded loops push whatever they find, so they can't tell good items from bad. An in-world **quality sorter** fixes that: a machine that only lets items above a set quality through, so a loop can route dim goods to charity and bright ones to the great works.

Two uses for low-quality goods already exist in 1.0 without new rules: delivering them for less renown, and training. Learning doesn't depend on input quality, so junk materials are good practice for new hires.

## Crafting math (M1)

These rules are ported from the Django and Python versions, with the changes agreed on 2026-10-30. All the numbers are first guesses to tune in play. If quality going a little wild now and then turns out to be fun, we keep some of it.

### Quality roll
This runs every time a monster finishes a good that has quality, including gathering. Where a recipe has no inputs or no tools, that average counts as 1.0.

1. **Material term** = average input quality × primary skill × secondary-skill average.
2. **Tool term** = weighted average tool quality × specific skill × ability factor. The ability factor is the relevant ability divided by the difficulty, at most 1.2. When the recipe uses a workshop, the workshop counts as one more tool with weight 8, matching its share weight and its durability cost. *Changed:* the original rules said workshop quality affects production, but neither the Django nor the Python code used it.
3. **Mean = (material term + tool term) / 2.** *Changed:* the original added the two terms. Expert chains then compounded without limit: a Metal Hammer made with the previous Metal Hammer reached quality 159 after 40 generations.
4. **Safety cap on the mean.** Above 1.0 the mean is compressed so that it approaches 2.0 (`QUALITY_MEAN_CAP`). It rarely kicks in; it is only there to stop runaway loops, for example once equipment arrives.
5. **Roll** = mean + normal noise with spread **σ = 0.1 + 0.3 × destabilizer average** (*changed* from 0.1 + 0.1 ×). The roll itself is not capped, only floored at 0. Outliers above 1 still happen, carry into downstream goods, and then fade back.
6. **Final** = roll, plus the WIS/STR nudge when it is below 1.

**Visible tiers:** quality ≥ 1.25 sparkles as *exceptional*, and ≥ 1.5 is a *masterwork*. Below that, brightness shows quality. For an expert with a mean of 0.97, σ = 0.4 gives an exceptional result about one craft in 4 and a masterwork about one in 11.

The tiers are only visual; there are no extra bonuses. The reward for quality is already built in. A better good earns more renown when delivered, and much more for deep goods: at σ = 0.4, a 2-sigma lucky depth-4 good earns +194% while an unlucky one loses 80%. A better tool improves every craft it's used in for its whole life. Durability stays at 100 for tools and 1000 for workshops, whatever their quality.

### Destabilizers
A destabilizer is a neighboring craft whose habits misfire on this job. It is never the good's own primary or secondary skill. A monster cross-trained in neighboring crafts works streakier: more masterworks, and more flops. Prestige goods get two: palace tiles, palace bricks, palace glaze, the temple bell and the great statue. All 60 goods with quality have destabilizers in `data/good_types.json`. Examples: dyes are destabilized by Painting, clay goods by Casting, smithed tools by Casting, and Thread by Sericulture.

### Transferable-skill pruning (watching this)
Each matching transferable skill drops the weakest secondary skill and the two lowest tool-weight units. When everything has been dropped, the average counts as 1.0, as originally written. It stays this way: it works, and getting around secondary skills is intended. One side effect to watch in play is that a well-matched monster also ignores tool quality on many recipes.

**Fallback if that becomes a problem:** the "stand-in" rule. Each secondary skill dropped by a matching transferable skill counts as the monster's primary skill, and each dropped tool slot counts as the best tool. Secondary skills can still be bypassed completely, but only as far as the monster has mastered the craft, and junk tools stop counting as perfect.

### Decisions made while porting (M1 stage 1)
- **Refinement depth follows the Django original:** a good made only from raw materials is depth 1, and each later step adds one. The Python port counted one lower.
- **Quality is rolled with the skills the monster had when the craft started,** so the forecast card matches what actually happens. Learning is applied afterwards.
- **Learning crafts with transferable skills:** the original data defines which transferable skills speed up learning only for hauling and wagon driving. Crafts use the recipe's own transferable skills instead.
- **Monster aging is cut,** so ability scores don't change with age.
- **The simulation uses its own seeded random numbers,** so a saved world replays identically.

### How the rules play in the world (M1 stage 2)
- **Pace:** fast mode runs 4 game seconds per tick, so a real second is about a game minute. Harvesting cotton takes about 2–3 seconds, spinning 3–5, and weaving 20–40, depending on the monster.
- **Harvests keep loops exact.** A harvest gathers the whole rolled quantity at once. The plant then holds those goods and pops one out per use, and regrows once it is empty.
- **Workshops hold finished goods** until an output mat is free, then set them out one at a time.
- **Built-in tools show on the pegs** as faint outlines. A tool pushed onto a peg is used instead, and wears out.

### Learning and forgetting
Unchanged from the original, plus the Python port's two fixes: skills never read below 0, and learning counts per craft rather than per whole multi-batch task. Mastery takes roughly 100–200 crafts, about 30–40 minutes of a looping monster at prototype speeds. Each monster has a single forgetting counter that drains every skill it isn't practicing, so monsters specialize.

## Milestones

- **M1, economy:** port the rules from sokobanmonstermakers, using its 92 tests as the spec, with the changes in Crafting math above. The tech tree now lives in `data/good_types.json`; give every good a stable `id`, and complete the data for the first chain. Add saving and the world list. Add quality on sprites, skill popups, the inspect card with the workshop forecast, delivery credits, the renown counter and the hiring hall. Workshops in M1 are the town's permanent ones.
- **M2, second town:** roads and signposts, wagons, town project boards, building frontier workshops and workbenches, edge markers and the Tab roster strip.
- **M3, content and art:** chains 2–3 and the great work. Replace placeholder sprites with final pixel art under the AGENTS.md palette rules.
- **M4, ship:** itch.io build and outside playtesters.
- **v1.5:** equipment, charity and great-works drop-offs, and the quality sorter.

## Open questions

- **Screen-by-screen layouts** of the two towns and the wilderness, designed in M1 and M2.
- **Weight you can feel (proposal):** heavier items push a little slower, scaled against the monster's STR (for example 3 ticks for a normal push and 4 for a heavy one), and wagons get a weight capacity. Oak versus hemlock then becomes a trade-off between value and hauling effort.
- **Tune in play:** economy numbers (starting renown, hiring costs, monster count, project costs), applicant rules (how many, refresh rate, pricing), and the time-scale presets.
- **Hauling and wagon skills** are learned by doing. Each tile a monster pushes cargo, or pulls a loaded wagon, counts as a few seconds of hauling or wagon-driving task time, for both learning and forgetting.
- **Tech-tree data still to fix:**
  - About 80 required tags are still provided by no good, outside the Fabric chain.
  - 3 recipes have a tag split into letters.
  - Already fixed in M1 stage 1: stable ids for every good, the second "Saw" renamed "Pick", nested tool weights on Bricks and Ash Glaze, and Indigo listing Chemistry twice.
