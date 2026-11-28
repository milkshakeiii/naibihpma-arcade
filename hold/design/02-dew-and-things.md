# Naibihpma Hold, document 02: dew and things

Draft 1, 2026-11-27. The second of five design documents. Document 01 said who the amphibians are and what they can do. This one sets the numbers: time, spots, items, buildings, prices and pace.

The numbers were tuned with a simulation of the Hold's economy, `sim/econ.py` (see "The simulation" at the end). Where this document and document 01 differ, this one wins: document 01 gave the Triblings' own numbers, and some of those had to change.

## Time

- **One tick is one frame.** The Hold runs at 15 frames a second, as AGENTS.md requires and as the demo does. RockCollector's numbers are in ticks, so they carry over as they are.
- **A square is a 16-pixel tile,** as in the demo. A screen shows 25 squares across and 15 down, above the panel.
- **Walking.** A creature walks a square in 10 ticks: 1.5 squares, or 24 pixels, a second. Crossing a screen takes about 17 seconds.
- **A day and a night take 5,400 ticks,** or six minutes. Day is the first half and night the second. Dawn is when the day starts, and noon is a quarter of the way through.

Some timings in seconds, for a feel:

| Thing | Ticks | Seconds |
|---|---|---|
| A newt's pick at a fern | 60 | 4 |
| A caecilian's pick or planting | 100 | 6.7 |
| Mending by hand (1 health) | 40 | 2.7 |
| Digging with a shovel (2 to 4 health) | 10 | 0.7 |
| A spot ripening | 1,000 | 67 |
| A building's own spot coming back | 600 | 40 |
| Most crafting | 150 | 10 |
| A plate | 1,000 | 67 |
| A hatching | 500 | 33 |

## Spots

| Spot | Ripens in | Work to pick | Difficulty | Gives | Planting skill, difficulty |
|---|---|---|---|---|---|
| Dew fern | 1,000 | 10 | 10 | 15 dew | (comes with its building) |
| Red fungus | 1,000 | 6 | 15 | 2 fungiwood | 0, 10 |
| White fungus | 1,000 | 9 | 20 | 1 cap ingot | 5, 15 |
| Hybrid fungus | 1,000 | 12 | 25 | 1 spotted cap | 8, 25 |
| Glow-spore puffball | 1,000, at night only | 6 | 15 | 1 glow spores | (comes with the Star Pond) |

Changes from the Triblings' numbers, and why:
- **A fern gives 15 dew.** The berry was one item. Dew is also money, so it needs finer steps. Building costs are scaled to match (see "Buildings").
- **A morel takes 6 work and gives 2 fungiwood,** where the data had 10 and 1. Fungiwood goes into nearly everything, and at the data's rate the simulation's Hold waited 25 minutes for its second cave.
- **An ironcap takes 9 work,** where the data had 15. **A toadstool takes 12,** where the data had 25. Both stay slow, and spotted caps stay the slowest material: the first one arrives at about 21 minutes.
- **Fungiculture picks 2 work a success.** The data gave it no harvest amount. Document 01's fix said 1, and this replaces it.

Gathering still picks 1 work a success, so a fern takes a newt about 70 seconds, and each of the Hall's four ferns gives dew about once every two and a half minutes.

## Making things

The recipes include the slips document 01 fixed. Crafting levels are lower than the Triblings' (see the note after the table).

| Item | Made from | Maker | Skill needed | Time | Price |
|---|---|---|---|---|---|
| Wooden shovel | fungiwood | tree frog | 0 | 150 | 6 |
| Iron shovel | fungiwood, cap ingot | salamander | 0 | 150 | 15 |
| Venomwood shovel | fungiwood, spotted cap | tree frog | 2 | 150 | 30 |
| Slingshot | fungiwood, cap ingot | tree frog | 0 | 150 | 10 |
| Venomiron catapult | fungiwood, cap ingot, spotted cap | tree frog | 2 | 150 | 40 |
| Iron spear | fungiwood, cap ingot | salamander | 2 | 150 | 25 |
| Venomwood spear | fungiwood, spotted cap | tree frog | 3 | 150 | 45 |
| Venomiron spear | fungiwood, cap ingot, spotted cap, Meteor Orb | salamander | 5 | 150 | 70 |
| Chanfron | cap ingot | salamander | 3 | 1,000 | 20 |
| Peytral | 2 cap ingots | salamander | 3 | 1,000 | 30 |
| Croupiere | 2 cap ingots | salamander | 3 | 1,000 | 30 |
| Venomiron chanfron | cap ingot, spotted cap | salamander | 4 | 1,000 | 60 |
| Dew bead | fungiwood, 2 dew | tree frog | 1 | 150 | 12 |
| Glow charm | fungiwood, glow spores, 2 dew | tree frog | 3 | 150 | 30 |
| Star pearl | 2 glow spores, Meteor Orb | Daot | 8 | 300 | 50 |

**The venomiron catapult goes up** from 0 to 2, because it now takes a spotted cap.

**Why the crafting levels are lower.** The Triblings' levels (10 for an iron spear, 15 for a venomiron one) assume makers that learn far faster than RockCollector's formula lets them. Under the formula, a salamander would have to make about sixty iron shovels before its first spear. That would also use sixty cap ingots. Each level here is about a third of the data's, so a maker reaches the next item after a handful of the last one. The formula itself is kept.

**Workshop skills:** woodcrafting rolls on dexterity, ironcrafting on intelligence, and enchanting on intelligence. Daot hatches with enchanting 8.

## Buildings

| Building | Health | Costs | Built from | Houses (hatch cost) | Keeps by default |
|---|---|---|---|---|---|
| The Hall | 1,000 | | | 8 newts (free) | the rest of the dew; 2 wooden shovels |
| The Burrow | 60 | 20 dew, 4 fungiwood | Hall | 2 caecilians (5 dew each) | 10 dew |
| Twig Loft | 60 | 10 dew | Hall | 2 tree frogs (5 dew) | 3 fungiwood, 3 wooden shovels |
| Morel Cave | 60 | 20 dew, 4 fungiwood | Burrow | 2 caecilians (5 dew) | |
| Fairy Ring | 40 | 1 fungiwood, 1 cap ingot | Burrow | | |
| Chorus Log | 250 | 50 dew, 10 fungiwood | Hall | the Chorus Toad (free) | 3 wooden shovels, 2 iron shovels |
| Ember Log | 60 | 10 dew | Chorus Log | 2 salamanders (5 dew) | 1 fungiwood, 1 cap ingot, 2 iron shovels |
| Ironcap Cave | 60 | 20 dew, 4 fungiwood | Chorus Log | 2 caecilians (5 dew) | |
| Toad Hall | 150 | 5 fungiwood | Chorus Log | 5 toads (10 dew purse) | 2 fungiwood, 2 cap ingots, 3 slingshots, 2 iron spears |
| Shell Forge | 150 | 5 cap ingots | Toad Hall | | 3 cap ingots, 1 of each plate |
| Cauldron | 150 | 5 cap ingots | Toad Hall | | 2 fungiwood, 2 spotted caps, 1 venomwood shovel, 2 venomwood spears |
| Star Pond | 250 | 1 Sun Orb | Hall | 4 axolotls (Sun Orb, 10 dew purse) | 2 fungiwood, 4 dew, 2 glow spores, 2 dew beads, 1 glow charm |
| Newt House | 60 | 10 dew, 2 fungiwood | Star Pond | 4 newts (free) | |
| Spawning Pool | 250 | Clargent Orb, 10 fungiwood, 250 dew | Star Pond | 2 bullfrogs, 2 goliath frogs (10 dew purse) | 20 dew |
| Daot's Grotto | 150 | 50 dew, 5 fungiwood, 2 cap ingots | Star Pond | Daot (free) | 1 fungiwood, 2 cap ingots, 2 spotted caps, 2 glow spores, 1 Meteor Orb, 1 of each thing it makes |
| Olm's Cave | 250 | Clargent Orb, 200 dew | Spawning Pool | the Olm (one of each orb) | |

Notes:
- **Dew costs are five times the Triblings' berry costs,** because a fern gives fifteen dew where a bush gave one berry. Daot's Grotto had no cost in the data; this one is new.
- **Health is lower than the Triblings'.** A building starts at a tenth of its health and is dug up the rest of the way, so health is how long it takes to build. At half the data's health (100 where a den had 200), a new Burrow still took one newt four minutes.
- **Every workshop keeps its own inputs,** as document 01's fix says.
- **Limits.** Each building's limits are document 01's, with dew multiplied by five, plus two wooden shovels at the Hall and two iron shovels at the Ember Log and the Chorus Log. **New:** a reservation for a building or a hatching may go over the limit, because the Spawning Pool's 250 dew is far above the Star Pond's 50.
- **New: the caves house caecilians.** Document 01 called fungus caves home. The Burrow's two caecilians couldn't keep four plots, two building spots and the puffballs going.
- **Spawning Pool numbers.** The Nursery housed four of each kind. The Spawning Pool houses two of each, so heroes don't outnumber the folk who arm them.

## Rules the simulation added

Running the Triblings' rules showed four places where they stall, plus one rule for limits (see "Buildings"). Each fix below is **New**.

1. **Up to four can build at once.** RockCollector lets only one creature claim a building to repair it. With one builder, a tree frog mending by hand could hold up the Chorus Log for ten minutes while newts with shovels stood idle. The limit is now four builders for a building, and still one picker for a spot and one maker for a workshop.
2. **The Hall keeps no fungiwood or orbs by default.** The Hall kept 10 fungiwood and 2 Sun Orbs in the data, so it held on to the first ten fungiwood and the Burrow couldn't get the four it needed. A building being built from the Hall reserves what it needs, so the Hall doesn't have to keep a stock.
3. **Tending teaches.** Caecilians pick only what's wanted. In one run nobody wanted fungiwood for twenty minutes, so no caecilian learned, and none reached the skill to plant ironcap. Tending now teaches fungiculture, at the same rate as repairing teaches a newt.
4. **Folk take shovels from the Hall and the Chorus Log only.** Otherwise newts emptied every shop of shovels before a bullfrog could buy one. The Hall keeps two wooden shovels, so newts can learn to dig before the Chorus Log exists.

## Money

- **Starting purse.** A hero hatches with 10 dew from its building's stock, enough for a slingshot.
- **Tithe.** A tenth of each reward goes to the hero's house. Newts carry it to the Hall with the shops' takings.
- **Rewards** start at 20 dew and step by 10 *(demo)*. Document 04 sets the kinds of reward.
- **Prices** are in the table in "Making things". A toad's full kit (sling, spear, then the venom versions) costs about 190 dew over a game.

**Income, for now.** Until document 03 sets the enemies, the simulation gives heroes 3 dew a minute each from drops, and the Elder spends 15 dew a minute on rewards whenever the Hall has more than 40 dew spare. Both are stand-ins.

## The pace

The first scenario starts with a small Hold already standing: the Hall, the Burrow, the Twig Loft, the Chorus Log and Toad Hall. It has 8 newts, 2 caecilians, 2 tree frogs, the Chorus Toad (left out of the simulation) and 2 toads with slingshots, and 60 dew, 6 fungiwood, 3 cap ingots and 4 wooden shovels. Document 05 says why: heroes have to be there in the first minute, and the full tree from the bare Hall takes thirteen to sixteen minutes to reach the first toad.

The scripted Elder then builds in this order: Morel Cave, Ironcap Cave, Ember Log, Fairy Ring, Star Pond, Newt House, Shell Forge, Cauldron, Daot's Grotto, Spawning Pool, Olm's Cave. It hatches whatever each building can house. Orbs arrive at fixed minutes: a Sun Orb at 8, Clargent at 14, Meteor at 18, then one about every three minutes. These are stand-ins. Under document 03's odds an orb takes about six heron kills, so the real schedule depends on the fighting.

These timings have no enemies in them. Raids, deaths and repairs will slow them, so they're the fastest the Hold can go. Over ten seeds:

| Milestone | Median | Range |
|---|---|---|
| A new toad buys its slingshot | 1:16 | every seed the same |
| Ironcap Cave built | 4:52 | 4:22 to 5:29 |
| First cap ingot picked | 7:44 | 6:39 to 8:02 |
| Fairy Ring built | 8:44 | 7:45 to 10:06 |
| Star Pond built | 10:04 | 9:23 to 11:32 |
| First iron shovel made | 10:26 | 8:18 to 14:32 |
| First iron spear bought | 20:10 | 8:30 to 32:40 |
| First spotted cap picked | 21:20 | 18:23 to 23:42 |
| First axolotl | 21:54 | 21:43 to 22:32 |
| First dew bead bought | 22:14 | 22:03 to 23:12 |
| Daot's Grotto built | 22:17 | 18:51 to 24:13 |
| First star pearl made | 23:43 | 23:24 to 25:16 |
| Spawning Pool built | 25:03 | 21:54 to 26:25 |
| First bullfrog | 25:52 | 22:47 to 27:18 |
| First venomwood spear bought | 28:13 | 22:43 to 34:11 |
| Olm's Cave built | 29:47 | 29:32 to 30:29 |
| First glow charm bought | 30:38 | 29:05 to 32:01 |
| First peytral made | 30:51 | 28:35 to 35:16 |
| The Olm | 36:03 | 35:52 to 39:08 |
| First chanfron bought | 39:37 | 37:17 to 41:08 |
| First star pearl bought | 43:16 | 42:06 to 44:14 |
| First venomiron spear bought | 42:20 | 38:49 to 44:33 (3 seeds of 10) |

Dew into the Hall averages about 20 a minute for the first five minutes, about 30 from five to twenty-five minutes, and 55 to 85 after that.

What that means for play:
- **The first quarter hour** is fungus, iron and slingshots. Toads fight with slings, and the first iron goes to building the caves and the forge.
- **The iron spear comes anywhere from 8 to 33 minutes.** Two things decide it:
  - **The salamanders' rolled intelligence.** A salamander with a low modifier has to practise on iron shovels first, so the Ember Log and the Chorus Log keep two each by default.
  - **Where the first iron goes.** The Shell Forge and the Cauldron cost five cap ingots each, and slingshots take one apiece.

  That's the story "The first spear" in document 01, though in one seed of ten the venomwood spear came first.
- **The middle** (20 to 30 minutes) brings the Star Pond's axolotls, the Grotto, the Spawning Pool and venom. That's where the Hold changes character.
- **The Olm comes at about 36 minutes,** and the best gear after 40. That leaves the last part of a 45-minute game for the road to Gorf.
- **Late in the game the Hall is rich, under the stand-ins.** It holds about 560 dew at 35 minutes and 800 at 40, because the stand-in spends only 15 a minute on rewards and nothing is left to build after the Olm's Cave. Most toads also end with 80 to 150 dew they can't spend. Both say the late game needs more to spend on, and that's for tuning once the enemies are in the simulation.

Folk are idle a lot of the time:

| Folk | Share of time idle |
|---|---|
| Tree frogs | 79% |
| Salamanders | 73% |
| Newts | 46% |
| Caecilians | 13% |

That's deliberate slack for raids and rebuilding, which document 03 will use. On screen, idle folk rest at home or wander near it. If it looks too sleepy in play, the Hold can house one maker of each kind instead of two.

## The simulation

`hold/design/sim/econ.py` plays the rules of documents 01 and 02 without enemies or a map. Buildings sit at fixed squares, and walking is straight-line distance. The model covers:
- spots ripening and picking, with damp, night and dawn rules
- planting and tending
- carrying to stock levels
- crafting with skill gates
- building with up to four builders
- hatching, purses, buying and the tithe
- skills and levels, by RockCollector's formulas

To run it:
- `python3 -I econ.py` runs five seeds.
- `python3 -I econ.py --seed 3 -v` shows one seed's timeline.
- `python3 -I econ.py --bare` starts from the Hall alone.
- `python3 -I summary.py` gives the ten-seed table above.

What it leaves out:
- enemies, combat, Dry, raids and repairs after damage
- the reward decisions heroes make
- the map's real paths and water

Document 03 adds the enemies, and the simulation should grow with it before anything is built.

## What comes next

Document 03: the marsh's dangers.
