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

**v1.5:** equipment, as physical crafted wearables (see below).

## Features

### Recipes
There is no recipe menu. What's on a workshop's pads decides what it makes. When two recipes take the same inputs, a signboard on the workshop wall shows the current one, and walking into the board switches it. Empty pads show a dithered ghost of what belongs there, as they do in the prototype.

### Tools
Workshops have tool pegs on the wall, which you fill by pushing tools onto them, the same way pads work. Each tool shows a small durability bar. When a tool breaks, a looping monster stops with a "? + hammer" bubble.

### Hiring
There is no character-creation screen. A **hiring hall** in town holds applicants. Each one is already rolled with a monster type and 3 transferable skills. Walk up to an applicant and their bubble shows type, skills, stats and renown cost. Space hires them. You choose which applicant to hire, not which boxes to tick, and the roster refreshes over time.

### Renown and goals
Renown is a HUD counter. You spend it in the world: hiring at the hall, and contributing to **town project boards**. A project board shows what goods it wants and a progress bar, and finished projects unlock things like the road to the second town or new workshops. A final "great work", for example a temple bell, ends 1.0.

### Delivery and shares
Delivering a good is a small celebration. "+12 RENOWN" rises from the drop-off, then smaller credits fly to each monster that contributed ("MOGG 6, BRUNA 4"). This is how the shares system becomes visible. All renown still goes to the one commune.

### Roster
Tab cycles monsters. A monster that's off-screen gets an **edge marker**: an arrow at the screen edge with its state icon (looping, waiting, idle), so a stuck monster is noticeable from anywhere. Holding Tab shows a strip of portraits with states in the HUD.

### Skills
Skill growth is shown as it happens. A small "+SPINNING" floats up when a skill improves, and a skilled monster visibly works faster: the progress bar fills quicker. Exact numbers are on the monster's inspect card.

### Items and quality
Quality is shown on the sprite: brighter is better, and exceptional items sparkle. Tools carry a durability bar. The inspect card gives the details.

### The inspect card
Hold the inspect key (to be chosen by feel, e.g. Shift) to show a card anchored next to the tile you're facing. If that tile is empty, the card describes the monster you're controlling.
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

## Crafting math (M1)

These rules are ported from the Django and Python versions, with the changes agreed on 2026-10-30. All the numbers are first guesses to tune in play. If quality going a little wild now and then turns out to be fun, we keep some of it.

### Quality roll
This runs every time a monster finishes a good that has quality, including gathering. Where a recipe has no inputs or no tools, that average counts as 1.0.

1. **Material term** = average input quality × primary skill × secondary-skill average.
2. **Tool term** = weighted average tool quality × specific skill × ability factor. The ability factor is the relevant ability divided by the difficulty, at most 1.2.
3. **Mean = (material term + tool term) / 2.** *Changed:* the original added the two terms. Expert chains then compounded without limit: a Metal Hammer made with the previous Metal Hammer reached quality 159 after 40 generations.
4. **Safety cap on the mean.** Above 1.0 the mean is compressed so that it approaches 2.0 (`QUALITY_MEAN_CAP`). It rarely kicks in; it is only there to stop runaway loops, for example once equipment arrives.
5. **Roll** = mean + normal noise with spread **σ = 0.1 + 0.3 × destabilizer average** (*changed* from 0.1 + 0.1 ×). The roll itself is not capped, only floored at 0. Outliers above 1 still happen, carry into downstream goods, and then fade back.
6. **Final** = roll, plus the WIS/STR nudge when it is below 1.

**Visible tiers:** quality ≥ 1.25 sparkles as *exceptional*, and ≥ 1.5 is a *masterwork*. Below that, brightness shows quality. For an expert with a mean of 0.97, σ = 0.4 gives an exceptional result about one craft in 4 and a masterwork about one in 11.

### Destabilizers
A destabilizer is a neighboring craft whose habits misfire on this job. It is never the good's own primary or secondary skill. A monster cross-trained in neighboring crafts works streakier: more masterworks, and more flops. Prestige goods get two: palace tiles, palace bricks, palace glaze, the temple bell and the great statue. All 60 goods with quality have destabilizers in `data/good_types.json`. Examples: dyes are destabilized by Painting, clay goods by Casting, smithed tools by Casting, and Thread by Sericulture.

### Transferable-skill pruning (watching this)
Each matching transferable skill drops the weakest secondary skill and the two lowest tool-weight units. When everything has been dropped, the average counts as 1.0, as originally written. It stays this way: it works, and getting around secondary skills is intended. One side effect to watch in play is that a well-matched monster also ignores tool quality on many recipes.

**Fallback if that becomes a problem:** the "stand-in" rule. Each secondary skill dropped by a matching transferable skill counts as the monster's primary skill, and each dropped tool slot counts as the best tool. Secondary skills can still be bypassed completely, but only as far as the monster has mastered the craft, and junk tools stop counting as perfect.

### Learning and forgetting
Unchanged from the original, plus the Python port's two fixes: skills never read below 0, and learning counts per craft rather than per whole multi-batch task. Mastery takes roughly 100–200 crafts, about 30–40 minutes of a looping monster at prototype speeds. Each monster has a single forgetting counter that drains every skill it isn't practicing, so monsters specialize.

## Milestones

- **M1, economy:** port the rules from sokobanmonstermakers, using its 92 tests as the spec, with the changes in Crafting math above. The tech tree now lives in `data/good_types.json`; complete its data for the first chain. Add quality on sprites, skill popups, the inspect card with the workshop forecast, delivery credits, the renown counter and the hiring hall.
- **M2, second town:** roads and signposts, wagons, town project boards, edge markers and the Tab roster strip.
- **M3, content and art:** chains 2–3 and the great work. Replace placeholder sprites with final pixel art under the AGENTS.md palette rules.
- **M4, ship:** itch.io build and outside playtesters.
- **v1.5:** equipment.

## Open questions

- **Saving:** a local-first game needs a save file (world, monsters, routines). Format and autosave cadence are still to decide.
- **Offline progress:** either monsters only work while the game runs, or the game catches up a capped amount of time on launch. The current lean is only-while-running for 1.0.
- **Applicants:** how many are offered at once, how often they refresh, and how cost scales with type and skills.
- **Inspect key:** which key feels best. Try it in play.
- **Hauling and wagon skills** used to learn from transport tasks that had a duration. In the grid world they need a new trigger, for example learning per tile hauled.
- **Tech-tree data to fix in M1:**
  - About 80 required tags are provided by no good.
  - 4 recipes have a tag split into letters.
  - Bricks and Ash Glaze have nested tool weights.
  - Indigo lists Chemistry as both its primary and a secondary skill.
  - The second "Saw" is really a pick.
  - The transferable-skill bonus to primary learning is defined only for hauling and wagon driving.
