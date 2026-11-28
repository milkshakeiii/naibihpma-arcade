# Naibihpma Hold, document 01: the amphibians

Draft 2, 2026-11-27. Henry reviewed it the same day and kept it as the first iteration: his decisions are at the end. Nothing here is built yet, except where it says *(demo)*. Document 02 retunes several of the numbers here, and where they differ, document 02 wins.

This is the first of five design documents. It covers who lives in the Hold, what they gather, make, wield and suffer, and how they decide. Prices, the enemies and the Elder's tools each get a later document, and appear here only where they touch the amphibians.

**The direction:** the amphibians are a port of RockCollector's Triblings. Most of RockCollector's rules carry over (`GAME_RULES.md` in milkshakeiii/RockCollector). Majesty's reward loop goes on top of them, as in the demo.

**Where the detail comes from:** mostly from RockCollector's own systems:
- resource spots that ripen beside their buildings
- materials that pass through several hands
- crafting limited by the maker's skill
- abilities with ranges and timings
- conditions that stack
- gear that changes how a creature fights

From Majesty, the Hold takes the broad behaviours that suit the animals, and leaves its fantasy specifics behind.

Times are in RockCollector's ticks, as in its data. A creature walks one square in 10 ticks. Document 02 turns ticks into Hold time.

**What changed from draft 1:**
- New sections: **Resource spots**, **Abilities**, **Conditions** and **Gear**, with the Triblings' numbers.
- **The economy** now says who makes what, where, and at what skill.
- **Habits** no longer maps each behaviour onto a Majesty hero.
- **Majesty as the floor** has become a **density check** for each kind.
- **Slips in the Triblings data** is new, with a fix for each.
- Dreaming is back in a new form: the dreamer wakes with a vision (see "Dreams at the Fairy Ring"). The old form copied the Elves' gambling and only taxed the rarest material.
- The questions are revised.

Every story and "what follows" below has been checked by hand against the rules. Document 02 adds a simulation that checks them properly.

## Where each part comes from

| Source | What it brings |
|---|---|
| **RockCollector, kept** | <ul><li>the tick simulation and thinking queue</li><li>six attributes (8 + 1d6), skills that grow with use, and d20 rolls against defence, dodge and resistance</li><li>conditions with stacks, and death when an attribute reaches 0</li><li>abilities unlocked by level; experience needed = level²</li><li>behaviour priority lists with danger thresholds</li><li>resource spots beside buildings, planting, growth and harvesting</li><li>crafting at workshops, gated by the maker's skill</li><li>buildings raised from delivered materials, starting at 10% health; spawning that costs items</li><li>stock levels (requests), transport routes and claiming</li><li>equipment slots and levels</li></ul> |
| **The Triblings, ported** | The whole tree: four materials, three orbs, ten kinds of creature, sixteen buildings, their resource spots, and the gear, abilities and conditions between them. Each Tribling role becomes an amphibian whose real biology fits it. |
| **Majesty, broad behaviours** | <ul><li>rewards paid in dew and set aside when posted *(demo)*</li><li>heroes with purses who buy their own gear, and a tithe for the hero's house</li><li>heroes who weigh chances instead of following fixed lists</li><li>a few habits that suit the animals: following to heal, learning at a building, and the fallen brought back</li></ul> |
| **New, for you to approve** | <ul><li>drying skin, and damp ground that ripens spots</li><li>bigger plots, and tending</li><li>day and night</li><li>visible growth</li><li>friends and grudges</li><li>husks and wisps</li><li>losing a home and moving in</li><li>Twen's bonus</li><li>the hero habits and their chances</li><li>dreams and visions (approved)</li></ul>Each is marked **New** below. |

## Pillars

1. **A village that lives without you.** Every amphibian has a name, attributes and a temperament, and every hero has a purse. Each is always doing something for a reason you can read on screen.
2. **You steer by wanting, never by ordering.** You have RockCollector's levers (stock levels, hatching, placing buildings, transport routes) plus rewards. Nobody obeys; they respond.
3. **Dew goes round.** Dew is the Triblings' berry and Majesty's gold at once. It is gathered, spent on building, hatching and rewards, spent again by heroes in your shops, and carried home by folk.
4. **The bodies are the rules.** Each species does its Tribling job because of a real trait of its animal.
5. **The detail is in the numbers.** What makes each kind distinct is what it can pick, make, wield, cause and suffer, and how those numbers meet other kinds' numbers.
6. **The album is the cast.** Naibihpma is "amphibian" backwards, and so are its people. Twen (newt) gathers dew, Daot (toad) works the alchemy, and Gorf (frog) is the enemy. The Star Pond is where the mages live.

## The Elder

You are the Elder of the Hold: a giant salamander too old and too big to leave the pool beneath the Hall. That is why you can't move anyone.

You have RockCollector's four levers and Majesty's one. Each lever has a trade-off, and every species and rule below should change at least one of them. The game pauses (P) *(demo)*, and every lever works while paused.

**Rewards.** Post dew on a target *(demo)*, and heroes decide whether it's worth it.
- *Trade-off:* dew on a reward is dew you can't build with. The time of day changes both the cost and the danger. By day, the herons are out hunting, so their nests hold only a few brooders, but the way there is dry. By night, the marsh is wet and every heron is home on its nest.
- *Example:* 40 dew on the east nest at noon draws two toads, the only heroes who'll cross the dry ground, against two herons. At dusk the same 40 dew draws five takers, against five herons.

**Stock levels.** Tell any building how much of each thing to keep, up to its limit (see "The economy"). This is RockCollector's main lever. It decides what the Hold picks, carries and makes.
- *Trade-off:* every material has rival uses. Cap ingots make iron spears for the toads, plates for the bullfrogs, or iron shovels that let newts dig and fight.
- *Example:* keeping 5 iron spears at Toad Hall means the Shell Forge waits for ingots.

The defaults keep the basics running, so you touch this lever only to choose between rival uses.

**Building.** Choose what is built where, and folk raise it from delivered materials.
- *Trade-off:* a building brings its resource spots with it, and they ripen only on damp ground (**New**, see "Resource spots"). Building by water feeds the Hold, but puts folk where the herons hunt. And which branch you build next decides what the Hold becomes. The Star Pond needs a Sun Orb. The Spawning Pool needs a Clargent Orb, 10 fungiwood and 50 dew.
- *Example:* a newt house by the stream adds four newts and two dew ferns that ripen all day, under the herons' flight line.

**Hatching.** Hatch a creature at a building that houses its kind, paying its cost *(demo, as hiring)*. In the Triblings data:
- Newts, toads, the Chorus Toad, bullfrogs and goliath frogs cost nothing but time once their building stands.
- Caecilians, tree frogs and salamanders cost a drop of dew each.
- An axolotl costs a Sun Orb.
- The Olm costs one of each orb.

Document 02 may add dew costs.
- *Trade-off:* orbs are the scarcest thing in the Hold. Every Sun Orb either hatches an axolotl now or waits toward the Olm.

**Transport routes.** Let one building take another's stock, even stock that building wants for itself.
- *Example:* route the Chorus Log to the Shell Forge, so the Log's cap ingots become plates before spears.

## The cast

The Triblings' ten creature kinds, ported.

| Tribling | Hold | Lives in | Job | Why this animal |
|---|---|---|---|---|
| Tribling | **Newt** (folk) | The Hall, newt houses | Gathers, carries, mends, defends; digs and fights with a shovel from level 5 | Small and tough-skinned. Twen, the album's dew collector, is a newt. |
| Grower | **Caecilian** (folk) | The Burrow | Plants, tends and picks fungus | Legless and nearly blind, living in soil. Fungus caves are home. |
| Crafter | **Tree frog** (folk) | The Twig Loft | Woodcrafting | Toe pads and nimble fingers |
| Smith | **Fire salamander** (folk) | The Ember Log | Ironcrafting | Folklore says salamanders are born from fire |
| Mayor | **The Chorus Toad** (one) | The Chorus Log | Defends, wanders; its call Bolsters from level 5 | Its call leads the evening chorus |
| Fighter | **Toad** (hero) | Toad Hall | Spear and slingshot | Sturdy, and dries slowly |
| Wiseling | **Axolotl** (hero) | The Star Pond | Lightning, pond light, regrowth | The Aztec god Xolotl, god of fire and lightning, became an axolotl to escape death. Real axolotls regrow limbs. |
| Clargeling | **Bullfrog** (hero, two squares wide) | The Spawning Pool | Two shovels and three plates | The heavyweight of the marsh |
| Strongling | **Goliath frog** (hero) | The Spawning Pool | Smash, Heavy Repair | Real goliath frogs move heavy stones to build their nesting pools |
| Dragon | **The Olm** (one) | The Olm's Cave | Bite, spear, pond light, Alter | Slovenians once took olms washed out of caves for baby dragons. Blind, it senses by smell and electric field, and lives over a century. |

Their numbers, from the Triblings data:

| Kind | Health at level 1 (then per level) | Attributes (added to 8 + 1d6) | Abilities (from level) | Gear slots |
|---|---|---|---|---|
| Newt | 6 (+3) | str −4, con +2 | Gathering, Mending, Bite (1); Shovel blow, Digging (5) | shovel |
| Caecilian | 8 (+3) | str −4, con +2, wis +2 | Fungiculture (1) | none |
| Tree frog | 8 (+3) | str −2, dex +2 | Woodcrafting (1) | none |
| Fire salamander | 8 (+3) | dex +2, int +2 | Ironcrafting (1) | none |
| Chorus Toad | 12 (+6) | con +2, cha +2 | Shovel blow, Digging (1); the call (5) | shovel (see "Slips") |
| Toad | 12 (+4) | str +2, con +2, int −2, wis −2 | Spear and sling (1) | spear, sling (see "Slips") |
| Axolotl | 6 (+2) | str, dex, con −2; int, wis, cha +2 | Lightning (1), Pond light (3), Regrowth (5) | bauble |
| Bullfrog | 30 (+10) | str +6, con +8 | Double Smash (1) | two shovels, head, chest, rump |
| Goliath frog | 30 (+10) | str +4, con +4; int, wis, cha +2 | Smash, Heavy Repair (1) | shovel |
| The Olm | 30 (+10) | +6 to every attribute | Bite, Spear and sling, Pond light, Alter (1) | bauble, spear, head |

Each of those numbers does something:
- A newt's −4 strength makes its bite feeble. A shovel at level 5 is what turns it into a fighter.
- An axolotl's −2 constitution is why drying is so dangerous to it (see "Conditions").
- A level-1 bullfrog has five newts' worth of health.

Folk never take rewards, and fight only when cornered or defending. When danger is near they flee to their house, using RockCollector's danger thresholds. Folk take gear from the Hold's stock for free; heroes take rewards, carry purses and pay for their gear.

**Levels.** Each level adds the kind's health per level and unlocks its abilities at set levels, and skills grow with use, all as in RockCollector.

**Losing a home.** As in RockCollector, a creature whose building is destroyed loses its home. **New:** if its kind has another house with room, it moves in. Otherwise it waits by the ruin, and if you rebuild in time, the new building takes it back.

### Notes on each

**Newts.** The Hall houses 8, and each newt house 4 more. A lost newt is replaced in 500 ticks. Newts keep the Tribling priorities: equip, defend, flee, repair, deliver, harvest, rest, idle. Two things follow from that order:
- Delivering comes before harvesting, so carrying dew home comes before gathering more.
- Repairing comes before delivering, so after a raid the dew stops until the buildings are mended, even if nothing was stolen.

Also:
- **New: Twen** is the first newt, a named character who knows every fern. While Twen lives, every newt gets +2 to gathering rolls.
- **New:** newt skin is poison. A creature that bites or swallows a newt rolls against Poisoned, 2 stacks.
- Newts are also the tax collectors (see "Where the dew goes").

**Caecilians.** Grower priorities: flee, plant, harvest, rest, idle, with **New** tending added before rest. They plant red fungus in the Morel Cave, white fungus in the Ironcap Cave, and hybrid fungus in the Fairy Ring. Planting needs a fungiculture skill of 0, 5 and 8 for the three. Picking gets harder with each fungus, and planting and picking share one skill (see "Slips"). So the caecilian skilled enough to plant a toadstool is also the one best at picking it. When danger comes, they flee into the ground.

**Tree frogs and fire salamanders.** They keep the Crafter priorities: flee, repair, deliver, craft, rest, idle. Each walks to whichever workshop wants an item it can make.
- Tree frogs make wooden shovels, slingshots, the venomwood gear and the venomiron catapult.
- Salamanders make iron shovels, iron spears, plates, and the venomiron spear and chanfron.
- Each item has a crafting level that the maker's skill must reach (see "Gear").

**The Chorus Toad.** There is one.
- It defends anywhere within 40 squares of the Chorus Log, four times the range of a hero, and wanders as far.
- Resting at the Chorus Log heals twice as fast as anywhere else.
- From level 5, its call Bolsters one ally within 7 squares, once every 100 ticks.
- **New:** at dusk it calls from the Chorus Log. This is a sound effect, never tied to the music. Heroes setting out on a long march may stop there first to be Bolstered (see "Habits").

**Toads.** They keep the Fighter priorities: equip, defend, flee, fight, raid, hunt, deliver, rest, wander. Equip becomes Buy (see "Where the dew goes"), and the reward lines are added.
- **Two weapons.** A toad has two weapon slots, a spear and a sling, as the Fighter behaviour asks (though the items' slot names don't match it: see "Slips"). It uses the sling beyond one square and the spear up close.
- **Unarmed is helpless.** A toad with neither weapon can't fight at all, because its only attack needs one. So a new toad's first errand is to buy a weapon with the dew it hatched with.
- **New:** bold toads buy a spear first and timid ones a sling.
- **New: digging in.** A steady or timid toad that's hurt far from home may dig in where it stands instead of fleeing.
  - Digging takes a few seconds, and it can be struck while it digs.
  - Underground it's hidden and heals slowly, shown as a small mound. Snakes can dig it out.
  - It comes up when it's well, or when the danger has passed.

**Axolotls.** They're frail, with +2 intelligence, wisdom and charisma, and hatch only by spending a Sun Orb.
- Energy Beam becomes Xolotl's lightning.
- Blinding Blast becomes a flash of pond light.
- Regeneration becomes their own regrowth, given to an ally.
- **New:** their gills dry four times faster than a toad's skin. Add their −2 constitution, and they fight from the water's edge.

**Bullfrogs.** They're huge: 20 health and 10 more per level, +6 strength, +8 constitution, and two squares wide.
- They hold two shovels and wear the three plates (head, chest and rump), and Double Smash swings both shovels at two enemies at once.
- Plates add defence and dodge. Only the venomiron chanfron adds resistance, so magic gets through.
- They're the reason for the forge.

**Goliath frogs.** Strongling numbers.
- They smash, and Heavy Repair is the real goliath frog's habit of shifting stones. With a shovel they mend a building about thirty to fifty times as fast as a newt with bare hands.
- They look for nests to raid 50 squares away, where every other kind looks 3 (see "Habits").

**The Olm.** One per Hold. It costs one of each orb, has +6 to every attribute, and its Alter strengthens two allies at once.

**New: Daot the Enchanter.** One, at Daot's Grotto: the album's old toad mage. Daot is folk, and never fights or takes rewards. It has one ability, enchanting, a crafting skill no one else has, and it arrives already skilled. It makes the star pearl. If Daot dies, it's regrown like anyone else, and while it's gone no star pearls are made.

## Buildings and the tree

The Triblings' sixteen buildings, ported, in the order they unlock. The costs are the Triblings', and document 02 tunes them.

| Hold building (Tribling) | Unlocks | Costs | Houses | Spots |
|---|---|---|---|---|
| **The Hall** (Denhome), the start | The Burrow, Twig Loft, Chorus Log, Star Pond | | 8 newts | 4 dew ferns |
| **The Burrow** (Grower's Den) | Morel Cave, Fairy Ring | 4 dew, 4 fungiwood | 2 caecilians | 1 red fungus |
| **Morel Cave** (Fungiwood Cave) | | 4 dew, 4 fungiwood | | a plot of red fungus |
| **Fairy Ring** (Experiment Shack) | | 1 fungiwood, 1 cap ingot | | a plot of hybrid fungus. Toadstools really do grow in rings. |
| **Twig Loft** (Crafter's Den) | | 2 dew | 2 tree frogs | 1 red fungus |
| **Chorus Log** (Meeting Den) | Ironcap Cave, Toad Hall, Ember Log | 10 dew, 10 fungiwood | The Chorus Toad | |
| **Ember Log** (Smith's Den) | | 2 dew | 2 fire salamanders | |
| **Ironcap Cave** | | 4 dew, 4 fungiwood | | a plot of white fungus |
| **Toad Hall** (Armory) | Cauldron, Shell Forge | 5 fungiwood | 5 toads | |
| **Shell Forge** (Foundry) | | 5 cap ingots | | |
| **Cauldron** | | 5 cap ingots | | |
| **Star Pond** (House of Wisdom) | Newt House, Spawning Pool, Daot's Grotto | 1 Sun Orb | 4 axolotls | **New:** 2 glow-spore puffballs |
| **Newt House** (Tribling House) | | 2 dew, 2 fungiwood | 4 newts | 2 dew ferns |
| **Spawning Pool** (Nursery) | Olm's Cave | 1 Clargent Orb, 10 fungiwood, 50 dew | 4 bullfrogs and 4 goliath frogs | |
| **Olm's Cave** (Dragon's Den) | | 1 Clargent Orb, 40 dew | The Olm | |
| **Daot's Grotto** (Alchemist's Den) | | not set in the data | **New:** Daot the Enchanter, the album's old toad mage, who makes the star pearl | |

## Resource spots

RockCollector already has the pattern of Zeus's vineyards and orchards: plots beside their buildings that ripen, are picked, and come back.

### How a spot works

- **Spots come with buildings.** Some buildings have fixed spots beside them. A spot picked bare comes back 600 ticks later.
- **Plots are planted.** The fungus caves and the Fairy Ring each have a planting plot. A caecilian can plant there only if its skill has reached the level that fungus needs. Then it rolls d20 + skill against the planting difficulty.
- **Spots ripen.** A spot ripens 1,000 ticks after it appears:
  - a fern beads with dew
  - red fungus becomes a wood morel
  - white fungus becomes an ironcap
  - hybrid fungus becomes a toadstool
- **Picking is work.**
  - Each pick rolls d20 + skill against the spot's difficulty, and each success adds the picker's harvest amount.
  - When the total reaches the spot's work, the spot gives its item, and the picker gains experience at the spot's difficulty. So harder spots teach faster.
- **One picker per spot.** A picker claims its spot, so two newts never work one fern, and the panel shows whose is whose.
- **Only what's wanted.** Folk pick only what some building's stock level asks for. A ripe spot nobody wants waits.

| Spot | Where | Ripens into | Picked by | Difficulty | Work | Gives | Afterwards |
|---|---|---|---|---|---|---|---|
| Dew fern | 4 at the Hall, 2 at each newt house | a dewy fern | Newts (gathering, on dexterity) | 10 | 10 | dew | back to an unripe fern |
| Red fungus | 1 at the Burrow and 1 at the Twig Loft; planted in the Morel Cave (planting skill 0, difficulty 10) | a wood morel | Caecilians (fungiculture, on wisdom) | 15 | 10 | fungiwood | building spots come back; planted ones must be replanted |
| White fungus | planted in the Ironcap Cave (planting skill 5, difficulty 15) | an ironcap | Caecilians | 20 | 15 | cap ingot | replanted |
| Hybrid fungus | planted in the Fairy Ring (planting skill 8, difficulty 25) | a toadstool | Caecilians | 25 | 25 | spotted cap | replanted |
| **New:** Glow-spore puffball | 2 at the Star Pond | a glowing puffball | Caecilians | 15 | 10 | glow spores | back to an unripe puffball |

What those numbers mean:
- **The toadstool is slow twice over.** Only a caecilian of skill 8 can plant it. Even that caecilian picks it slowly, succeeding mostly on high rolls, and it needs 25 work. As the data stands, it's too slow. Document 02 tunes it, but spotted caps should stay the slow material.
- **The Twig Loft's morel** feeds the tree frogs' own carving, but only a caecilian can pick it. So the loft waits on the Burrow.
- **A gathering newt** succeeds on about half its rolls at first, and a fern needs ten successes. So the Hall's four ferns and eight newts set the pace of the early Hold.

### New: spots in the marsh

- **Damp ripens.** A spot ripens only while its ground is damp (see "Dry").
  - Ferns by water ripen all day and night.
  - Ferns on open ground ripen only at night, so they're ripe at dawn.
  - Caves are damp inside, so fungus ripens at any hour.
  - Glow-spore puffballs ripen only in the dark, and burst at dawn if nobody has picked them. They're the night crop, as open-ground dew is the dawn crop, so caecilians walk to the Star Pond at night while the herons roost.
- **Dew dries.** A ripe fern on open ground loses its dew at noon and goes back to unripe. So open-ground ferns are a dawn crop, picked before noon or lost. Ferns by water keep theirs.
- **Bigger plots.** In the data, each plot holds a single fungus, because planting needs even squares. In the Hold, each cave's plot holds four fungi and the Fairy Ring's two (draft; document 02 tunes the counts). That way a cave reads as a bed of fungus, the way Zeus's orchards read as rows of trees.
- **Tending.** A caecilian with nothing else to do curls beside a ripening fungus, which then ripens half again as fast. Tending teaches as repairing does. It makes the number of caecilians a choice: each one hatched (a drop of dew) speeds the caves.
- **Stages you can read.** Each spot has four sprites: bare, sprouting, ripening and ripe. Ripe toadstools glow faintly at night.
- **Trampled and cursed.** Gorf's creatures that walk over a plot trample it back to bare. Gorf's cursed ground is dry, so spots on it stop ripening. Document 03 says which creatures trample, and where the curse spreads.

## Abilities

The Triblings' abilities, renamed where the animal suggests it. Ranges are in squares. The cooldown is the wait after using an ability. The recharge is the least time between two uses.

| Hold name (Tribling name) | Who (from level) | What it does | Range | Cooldown, recharge | Rolls on |
|---|---|---|---|---|---|
| Gathering | Newts (1) | 1 work on a fern per success | 1 | 60 | gathering (dexterity) |
| Fungiculture | Caecilians (1) | Picks fungus; also plants (see "Slips") | 1 | 100 | fungiculture (wisdom) |
| Bite | Newts (1), the Olm (1) | 1d2 to one enemy | 1 | 15, 20 | biting (strength) |
| Shovel blow (Primitive Weapon Attack) | Newts (5), the Chorus Toad (1) | The shovel's damage to one enemy | 1 | 30, 30 | shovels |
| Spear and sling (Simple Weapon Attack) | Toads (1), the Olm (1) | The weapon's damage to one enemy | 1 or 4 | 20, 30 | spears or slingshots |
| Double Smash | Bullfrogs (1) | The shovels' damage to two enemies | 1 | 25, 25 | shovels |
| Smash | Goliath frogs (1) | Not defined in the data (see "Slips") | 1 | | shovels |
| Mending (Repair Den) | Newts (1) | Mends 1 health of a building | 1 | 40 | |
| Digging (Dig Den) | Newts (5), the Chorus Toad (1) | Mends with a shovel, 2 to 4 health by shovel | 1 | 10 | |
| Heavy Repair | Goliath frogs (1) | Mends with a shovel | 1 | 3 | |
| Woodcrafting, Ironcrafting | Tree frogs, salamanders (1) | Makes an item at a workshop | 1 | the item's crafting time | the craft |
| The call (Bolster) | The Chorus Toad (5) | Bolsters one ally | 7 | 0, 100 | motivation |
| Lightning (Energy Beam) | Axolotls (1) | 1d12 to one enemy | 7 | 40, 100 | energy (intelligence) |
| Pond light (Blinding Blast) | Axolotls (3), the Olm (1) | 1d6 and 2 stacks of Blinded to every enemy within 3 squares of the target | 7 | 40, 100 | energy (intelligence) |
| Regrowth (Regeneration) | Axolotls (5) | Gives one ally Regenerating | 7 | 10, 100 | healing (wisdom) |
| Alter | The Olm (1) | Gives two allies Altered (see "Slips") | 7 | 10, 100 | alteration |

How a blow lands, as in RockCollector: the attacker rolls d20 + skill, and the defender stops it if the roll is no more than 10 + its own skill. Melee is stopped by defence, slings by dodge, and lightning and pond light by resistance.

What follows from the numbers:
- **Shovels multiply mending.** A newt mending by hand restores 1 health every 40 ticks. With a shovel it's 8 to 16 times faster, and a goliath frog with a shovel 27 to 53 times faster. One goliath frog gets a raided Hold's dew moving again sooner than all its newts.
- **Pond light wants a crowd.** It hits everything within 3 squares of its target, and herons roosting together on a nest at night are exactly that.
- **Lightning goes round armour.** Its 1d12 is weaker than an iron spear's 3d6, and it recharges in 100 ticks where a spear strikes every 30. But it rolls against resistance instead of defence or dodge, so it lands on whatever blocks and dodges well.
- **Weapon abilities need a weapon.** No spear and no sling means no attack.

## Conditions

| Condition | Caused by | Each stack | Fades (ticks per stack) | Most stacks |
|---|---|---|---|---|
| Bolstered | The Chorus Toad's call | +4 strength, +4 constitution, +2 charisma | 2,000 | 1 |
| Poisoned | Venom gear (1 stack a hit from venomwood, 2 from venomiron); **New:** newt skin | −1 constitution | 400 | 8 (venomwood), 10 (venomiron) |
| Blinded | Pond light | −1 defence, −1 dodge | 300 | 10 |
| Regenerating | Regrowth | Heals steadily (a point every 50 ticks) | 500 | 10 |
| Altered | Alter | +4 strength, dexterity, constitution and intelligence. Acts 10% slower, but walks a square in 8 ticks instead of 10 | not set (see "Slips") | 2 |
| **New:** Dry | Dry ground (see "Dry") | −1 constitution, slower walking | Only on damp ground. Water clears it fast. | no limit |
| **New:** Dreaming | Eating a ripe toadstool (see "Dreams at the Fairy Ring") | Lies still and does nothing. A hit wakes it, with no vision. | 1,000 (draft) | 1 |

How they work, as in RockCollector:
- **Harmful conditions need a roll.** The attacker rolls d20 + skill against 10 + the target's level, plus protection from its items and conditions, plus its modifier in the attribute that guards against that condition.
- **Helpful conditions land without a roll.**
- **Fading.** Stacks fade in proportion to how many there are, so a heavy dose falls fast at first.
- **Death.** A creature dies when any attribute reaches 0, whatever its health.

What follows:
- **Constitution is where they meet.** Drying takes constitution, a creature dies when any attribute reaches 0, and defence rolls on constitution. So a drying hero also blocks less, long before it's in danger of dying. An axolotl, with −2 constitution, has the least room.
- **The call buys distance.** Bolster's +4 constitution is four more stacks of Dry a hero can carry, for 2,000 ticks. A toad Bolstered in the morning can reach a nest across dry ground at noon.
- **Poison softens.** Poisoned does no damage, but defence rolls on constitution. So every two stacks cost the target a point of defence, and every spear that follows lands more often. That's how venom wears down a snapping turtle's shell (document 03). A creature also dies when any attribute reaches 0, but at the most 10 stacks only one with a constitution of 10 or less can die of poison, and a venomiron spear has usually killed anything that small already.
- **Blind first, then strike.** Blinded lowers defence and dodge, so pond light makes every spear and sling that follows hit more often. Axolotls beside toads is a pairing the numbers reward.

## Gear

| Item | Slot (level) | Damage, range | Also | Made from | Maker (skill needed) | Workshop |
|---|---|---|---|---|---|---|
| Wooden shovel | shovel (1) | 2d2, 1 | mends 2 | fungiwood | tree frog (0) | Twig Loft |
| Iron shovel | shovel (2) | 3d4, 1 | mends 3 | fungiwood, cap ingot | salamander (6) | Ember Log |
| Venomwood shovel | shovel (3) | 3d4, 1 | mends 4; Poisoned, 1 stack a hit, up to 8 | fungiwood, spotted cap | tree frog (8) | Cauldron |
| Slingshot | sling (1) | 1d6, 4 | | fungiwood, cap ingot | tree frog (0) | Toad Hall |
| Venomiron catapult | sling (2) | 2d6, 4 | Poisoned, 2 a hit, up to 10 | fungiwood, cap ingot, spotted cap | tree frog (0) | Daot's Grotto |
| Iron spear | spear (2) | 3d6, 1 | | fungiwood, cap ingot | salamander (10) | Toad Hall |
| Venomwood spear | spear (3) | 3d6, 1 | Poisoned, 1 a hit, up to 8 | fungiwood, spotted cap | tree frog (12) | Cauldron |
| Venomiron spear | spear (4) | 4d6, 1 | Poisoned, 2 a hit, up to 10 | fungiwood, cap ingot, spotted cap, Meteor Orb | salamander (15) | Daot's Grotto |
| Chanfron | head (1) | | +4 defence, +4 dodge | 1 cap ingot | salamander (10), 1,000 ticks | Shell Forge |
| Peytral | chest (1) | | +4 defence, +4 dodge | 2 cap ingots | salamander (10), 1,000 ticks | Shell Forge |
| Croupiere | rump (1) | | +4 defence, +4 dodge | 2 cap ingots | salamander (10), 1,000 ticks | Shell Forge |
| Venomiron chanfron | head (2) | | +4 defence, dodge and resistance | cap ingot, spotted cap | salamander (10), 1,000 ticks | Daot's Grotto |
| **New:** Dew bead | bauble (1) | | halves the Dry its wearer gains | fungiwood, 2 dew | tree frog (2) | Star Pond |
| **New:** Glow charm | bauble (2) | | halves Dry gained; +2 energy | fungiwood, glow spores, 2 dew | tree frog (8) | Star Pond |
| **New:** Star pearl | bauble (3) | | halves Dry gained; +3 energy, +3 healing, +4 resistance | 2 glow spores, Meteor Orb | Daot (enchanting) | Daot's Grotto |

The rules, from RockCollector:
- **Trading up.** A creature swaps only for a higher level in the same slot.
- **The maker's skill gates the gear.** A maker can make an item only if its skill has reached the item's crafting level. Each item made teaches at that level + 10, so harder work teaches faster. The Hold's best gear waits on one skilled salamander.

What follows:
- **The first toads are slingers.** The only toad weapon a new Hold can make is the slingshot, because the iron spear needs a salamander of skill 10. Bold toads (who want a spear first) are poorly served until then.
- **Even the slingshot needs iron.** It takes a cap ingot, and white fungus needs planting skill 5. So the toads' first weapons wait on a caecilian's skill. Left as it is, new toads would sit unarmed for a long time, so document 05 starts the Hold with a few cap ingots.
- **Plates are slow.** Each takes a salamander 1,000 ticks, so dressing one bullfrog takes 3,000.
- **Only the venomiron chanfron resists magic.**
- **New: baubles.** The Triblings gave Wiselings and the Dragon a bauble slot that no item fitted. The Hold fills it with three:
  - Every bauble halves the Dry its wearer gains, so a bauble is what lets an axolotl follow a toad past the water's edge.
  - The higher ones add to the energy roll (lightning and pond light), and the star pearl to healing and resistance too.
  - The star pearl needs a Meteor Orb, so a Meteor Orb buys a venomiron spear, a star pearl, or part of the Olm.

## The economy

### What each building keeps and makes

Stock levels can be set from 0 up to each building's limit. These are the Triblings' limits, with two changes:
- **New:** the Hall keeps all the dew that no other building wants, because it's the treasury.
- Workshops also keep their own inputs (see "Slips").

| Building | Keeps, up to | Makes |
|---|---|---|
| The Hall | the rest of the dew, 10 fungiwood, 2 Sun Orbs | |
| The Burrow | 5 dew, 1 fungiwood, 1 cap ingot | |
| Twig Loft | 4 dew, 4 fungiwood, 4 wooden shovels | wooden shovels |
| Ember Log | 4 dew, fungiwood and cap ingots | iron shovels |
| Chorus Log | 5 each of dew, fungiwood, cap ingots, wooden shovels and iron shovels | |
| Toad Hall | 5 each of fungiwood, cap ingots, iron spears and slingshots | iron spears, slingshots |
| Shell Forge | 10 cap ingots, 5 of each plate | chanfrons, peytrals, croupieres |
| Cauldron | 5 each of fungiwood, spotted caps, venomwood shovels and venomwood spears | venomwood shovels and spears |
| Star Pond | 5 of each orb, 50 dew, 50 fungiwood; glow spores, and the dew beads and glow charms | dew beads, glow charms |
| Newt House | 5 dew | |
| Spawning Pool | 5 Clargent Orbs, 50 dew | |
| Olm's Cave | 5 of each orb | |
| Daot's Grotto | fungiwood, spotted caps, cap ingots, Meteor Orbs, glow spores, the three venomiron items and star pearls | venomiron spear, chanfron, catapult; star pearls |

### How stock levels move things

As in RockCollector:
- Folk pick only what's wanted, and carry only what's wanted.
- Makers make only what a workshop wants and lacks, and only if they have the skill.
- A carrier takes from the ground, or from any building holding more than it wants.
- A transport route lets one building take another's stock even when that building is short itself.
- **A new building is paid from the building it's built from.** The Newt House, the Spawning Pool and Daot's Grotto are paid from the Star Pond's stock, which is why the Star Pond keeps 50 dew and 50 fungiwood.
- **A hatching is paid from the hatching building's stock.** The Burrow's dew pays for its caecilians.

So the panel for any building can say why it's empty: "WANTS 2 CAP INGOTS. NONE SPARE. IRONCAP CAVE RIPENING."

### Where the dew goes

Dew is the Triblings' berry, so it builds and hatches. It's also Majesty's gold, so it pays rewards and buys gear. The loop:

1. **Gathering.** Newts pick dew from the ferns and carry it to the Hall.
2. **Spending.** You spend it three ways: on building, on hatching, and on rewards *(demo)*. Newts carry dew out of the Hall to any building that wants it for building or hatching. Rewards are paid from what stays in the Hall. So building and rewards still draw on one pool, which is the Elder's main trade-off.
3. **Earning.** A target falls, and its reward is split among those who hit it *(demo)*. Heroes also pick up what enemies drop. A tenth of each reward goes to the hero's house, so some dew comes home even from heroes who never shop.
4. **Buying.** RockCollector's Equip line becomes Buy for heroes. A hero looks for a higher-level item in its slots, as before, and takes it only if its purse can pay. The dew goes into the building that sold it.
5. **Collecting.** The Hall keeps the dew that no other building wants, so newts carry the dew from each shop to the Hall. That makes them the tax collectors. A transport route can send one shop's dew straight to another building.

The loop leaks in four places:
- A newt carrying dew is a target. If it's killed, the dew drops, and any hero passing can keep it.
- Heroes who die carry their purses with them (see "Husks").
- Gear costs materials your folk had to grow.
- Enemies raid buildings, and folk must mend what they break before they carry again.

A Hold that rewards generously gets rich heroes, busy forges and well-armed toads. One that hoards gets poor heroes, who fight with slingshots.

## New: amphibian rules

Each of these is an addition to RockCollector's rules, for you to keep or cut.

### Dry

Amphibians breathe partly through their skin and dry out on land. Dry is a RockCollector condition (see "Conditions").

**Two kinds of ground, drawn differently.**
- *Damp ground:* moss, fern and mud near water.
- *Dry ground:* open path, sand, and Gorf's cursed earth, which looks cracked.

**Stacks.**
- **Gaining.** A creature on dry ground gains stacks: slowly at night, faster by day, and never in rain.
- **Losing.** Damp ground stops new stacks and lets them fade. Water clears them fast: the stream, ponds, a building's pool, or a puddle.
- **Effects.** Each stack costs a point of constitution and slows walking. A creature with many stacks puts "find water" above everything except Flee. Drying takes constitution alone, so it meets Bolster and Alter in one place, and a drying creature also blocks less (see "Conditions").
- **Species differ.** Toads gain stacks at a third of a frog's rate, and axolotls at four times a toad's.

**Puddles.** Newts with shovels dig puddles where you mark them (document 04), and goliath frogs widen them into pools unasked (see "Habits"). A pool clears Dry faster than a puddle. Both last unless Gorf's curse spreads over them.

**On screen.** A creature with many stacks shows a small drop icon, the way the demo shows health bars only when they matter, and its panel gives the stack count. Gradual paling wouldn't read on sprites this small.

So the map's water decides how far heroes reach. A hero's asking price *(demo)* adds the dry ground on its path, so a far target across dry ground costs more by day than by night.

### Day and night

A day and a night take about six minutes (to be tuned), and the game pauses.

- **Night.** The ground is wet, and the map glows with fireflies, glowing spores and the stars in the Star Pond. The palette remaps only the background ramps, so sprites keep their colours and their dark outlines still stand out. Herons roost on their nests at night, so the nests are full.
- **Dawn.** The open-ground ferns are ripe, and their dew is gone by noon, so dawn is the gathering hour.
- **Day.** The open ground dries. Herons leave their nests to hunt the marsh, which endangers the folk and leaves the nests nearly empty.

So neither half of the day is simply better. Night makes it easy to reach a nest and hard to take it. Day makes it hard to reach and easy to take, and toads are the day's heroes.

**Every reward shows its candidates.** It lists which heroes are weighing it and why each one declines, for example "3 TOO DRY, 2 WANT MORE". The demo already names who wants more.

### Growth

- **Hatching.** It shows as eggs in the building's pool, then a tadpole that grows legs and hops out. That's RockCollector's spawn time made visible.
- **New: a purse to start.** Each hero hatchling carries out a few drops of dew, meant to buy its first weapon. They're paid from its building's stock, so a hatching costs that much dew, and a death never makes new dew.
- **Stages.** Each species has three sizes of sprite, for young, adult and old.
- **Levelling up.** On a level-up the creature sheds its skin and eats it, as real frogs and newts do.

### Ties

- **Friends.** Two heroes become friends when both are badly hurt in the same fight and both survive. A hero has at most one friend, so friendships stay rare enough to notice. Friends take rewards together, and wait for each other before setting out, as the demo waits for company. The wait has a time limit, as the demo's does.
- **Carrying home.** A hero carries a fallen friend's husk home on its own, with no reward needed.
- **Grudges.** If a brave hero's friend dies, it takes a grudge against the kind of enemy that killed it. A grudge lowers its asking price for that kind.

### Husks and wisps

A fallen amphibian leaves a husk, with its purse still in it.
- **Brought home.** If a friend or a rewarded hero carries the husk to the Star Pond, the axolotls regrow it, one level lower. You pay for the regrowing, and the hero keeps its purse.
- **Left out.** A husk left out for half a day loses its spirit to Gorf, though a higher-level hero's husk lasts longer. The spirit joins Gorf's procession as a wisp (document 03), and the purse drops where the husk lay.
- **Won back.** Breaking that wisp frees the spirit, which goes back to the Star Pond and can still be regrown.
- **Axolotls go home alone.** An axolotl's husk needs no carrying: it drifts back to the Star Pond by itself and waits there to be regrown, at the same price and one level lower.

So every loss is a debt you can repay, not a spiral.

## How a hero decides

A toad, axolotl, bullfrog, goliath frog or the Olm uses its Tribling priority list, with three changes:
- Buy moves below Defend and Flee, so heroes don't go shopping during a raid.
- The reward lines are added (marked +).
- Find water is added (New).

1. **Defend** the Hold
2. **Flee**, using RockCollector's danger thresholds and the demo's temperaments
3. **Find water**, if very dry (New)
4. **Buy** an upgrade it can afford (Equip, now paid for)
5. **Fight**, then, for goliath frogs only, **Repair** damaged buildings
6. **+ Keep at a reward** it has taken
7. **+ Take a reward** worth more than its asking price, counting friends who have taken it
8. **Raid**, then **Hunt**
9. **Deliver** (heroes carry dropped dew and orbs home), then **Rest**, then **Wander**

**Unrewarded hunting.** Heroes hunt and raid without any reward, as far as 100 squares from home.
- **Level limits.** RockCollector has limits on how far above or below itself a hero will hunt. The Triblings set them wide open (±999). **New:** the Hold sets real ones (draft: up to 2 levels above itself; document 02 tunes them).
- **What a reward does.** It sends heroes past their range, or at something their level limit or danger check would refuse.

You can always see which line a creature is on. The panel says it in one plain sentence: "GOING TO THE STREAM: DRY", "SAVING FOR AN IRON SPEAR", "WAITING FOR PIP AT THE BRIDGE".

## Habits

Each kind's habits come from its body and its numbers. Every habit and chance below is **New** design unless it says it comes from the data. The chances are drafts and will be tuned.

### How a hero chooses

Heroes don't follow a fixed list.
- **A chance for each line.** Each line of a hero's list has a chance of being considered whenever it thinks. It works down the list until it reaches a line it considers and can do. Majesty worked this way, according to the 2000 Prima guide.
- **When it rethinks.** It thinks again every so often, and whenever the danger near it changes (RockCollector's interrupt). So two toads make different choices on the same afternoon.
- **Folk keep fixed lists,** so the economy stays readable.
- **What sets the chances.** Each line's chance comes from the hero's kind, shifted by its temperament and attributes.

Draft chances for each hero kind, beyond the shared lines (defend, flee, find water, take a reward):

| Kind | What it considers unasked (draft chance) |
|---|---|
| **Toad** | <ul><li>Buy a weapon, if it has none (95)</li><li>Hunt enemies in sight (85)</li><li>Buy an upgrade it can afford (80)</li><li>Carry dropped dew or an orb home (70)</li><li>Go home (70)</li><li>Raid a nest within 3 squares (70)</li><li>Dig in when hurt, if steady or timid (60)</li><li>Eat moths under a glowing log at night (50)</li><li>Stop at the Chorus Log before a long march (40)</li><li>Eat a ripe toadstool when passing the Fairy Ring (15, more at low wisdom)</li></ul> |
| **Axolotl** | <ul><li>Learn at the Star Pond when an ability is due (90)</li><li>Soak in the Star Pond when dry (90)</li><li>Go home (85)</li><li>Follow and heal a toad (75)</li><li>Pond-light a crowd before the toads close in (70)</li><li>Fight from the water's edge (65)</li><li>Follow and heal a bullfrog (50)</li></ul> |
| **Bullfrog** | <ul><li>Fight anything that enters its territory (95)</li><li>Patrol its territory around the Spawning Pool (90)</li><li>Buy a plate (85)</li><li>Buy a second shovel (70)</li><li>Go home (60)</li><li>Raid (60)</li><li>**New:** call from the Spawning Pool at night (40), which draws Gorf's creatures to it and away from the folk</li></ul> |
| **Goliath frog** | <ul><li>Sense a nest far off and raid it (95)</li><li>Heavy Repair a damaged building (75)</li><li>Widen a puddle into a pool (60)</li><li>Escort a newt carrying dew (50)</li><li>Go home (40)</li></ul> |
| **The Olm** | <ul><li>Defend the Hold (95)</li><li>Alter two allies before a fight (95)</li><li>Sleep in its cave unless it senses danger near the Hold (90)</li><li>Pond-light the largest crowd (80)</li></ul> |

### Each kind's habit

- **Goliath frogs raid far.** In the data they look for raid targets 50 squares away, and everyone else looks 3. So goliath frogs find far nests and go after them unrewarded. That's useful, and a nuisance when you wanted a nest left alone.
- **Goliath frogs build pools.** Real ones shift stones to make nesting pools. Unasked, a goliath frog with a shovel widens a dug puddle into a pool, which clears Dry faster.
- **Bullfrogs hold ground.** Real bullfrogs defend territories. A bullfrog patrols a ring around the Spawning Pool and fights anything that enters it. It's a strong wall at home, and it rarely goes far without a large reward.
- **Axolotls wait at the water's edge.** They follow toads and bullfrogs to heal them, stop where the damp ground ends, and wait there, visibly, until their charge comes back.
- **Axolotls go home alone.** A fallen axolotl's husk drifts back to the Star Pond by itself to be regrown (see "Husks and wisps"), as the real animal regrows what it loses.
- **Toads are the day's heroes.** They dry at a third of a frog's rate, so the noon raid is theirs.
- **Toads dig in** (see "Notes on each"), and at night they idle under the glowing logs, eating moths.
- **The Olm sleeps.** Real olms can go years without food, and sense prey through the water by its electric field. The Hold's Olm wakes when it senses danger near the Hold, and is slow to wake.
- **Newts carry before they gather, and mend before they carry** (see "Notes on each").
- **Caecilians tend** when there's nothing to plant or pick (New).

### Who helps whom

- Axolotls follow toads and bullfrogs, to heal them and to blind what they fight.
- Goliath frogs escort newts carrying dew home.
- Young toads (levels 1 to 3) tag along behind an older toad from their hall, and share its experience.
- Friends travel together (see "Ties").
- The Chorus Toad Bolsters heroes who stop at the Chorus Log.

### What one kind makes for another

Each chain runs through RockCollector's materials, so the economy and the heroes are one system.
- **Fungus to iron.** Caecilians grow ironcap. Salamanders forge it into iron shovels, spears and plates, and toads, bullfrogs and goliath frogs buy them.
- **Venom.** Caecilians grow toadstools in the Fairy Ring. Tree frogs carve the caps into venomwood shovels and spears at the Cauldron, and salamanders make venomiron at Daot's Grotto. A hero armed with one poisons what it hits.
- **Wood.** The Twig Loft's own morel is picked by caecilians and carved by tree frogs.
- **Shovels.** Newts dig and fight with them from level 5. Goliath frogs need one to repair, and bullfrogs swing two.
- **Mending.** Newts mend before they carry, so goliath frogs and shovel-carrying newts are what get the dew moving again after a raid.
- **Orbs.** Gorf's creatures drop the orbs, heroes carry them home, and the Star Pond, the Spawning Pool, axolotls and the Olm need them.
- **Healing.** Regrowth keeps toads in the field, and the Star Pond regrows the fallen.
- **Bolster.** The Chorus Toad's call lets heroes carry more drying.

### Learning as they level

- An **axolotl** that reaches level 3 or 5 must go down into the Star Pond's deep water to learn pond light or regrowth. It pays dew for the lesson, which goes into the Star Pond's till.
- A **newt** reaching level 5 learns to dig and fight only if it has a shovel.
- The **Chorus Toad's** call comes at level 5.
- **Caecilians** reach fungiculture skills 5 and 8 by planting and picking, so the Fairy Ring waits on one skilled caecilian.
- **Makers** reach crafting levels by making things, so the best gear waits on one skilled salamander.
- **Weapon skills** rise with use for everyone, as in RockCollector.

### Dreams at the Fairy Ring

**New** (approved). A hero passing the Fairy Ring may stop to eat a ripe toadstool, and lies Dreaming. When it wakes, it has seen something.

- **Who dreams.** Only heroes, and mostly those low in wisdom. Toads, with −2 wisdom, dream most. Axolotls, with +2, hardly ever do.
- **Only an unclaimed toadstool.**
  - A toadstool that a caecilian has claimed for picking can't be eaten.
  - So your stock levels decide whether heroes dream. While the Cauldron or Daot's Grotto wants spotted caps, caecilians claim each toadstool soon after it ripens.
  - Stop wanting spotted caps, and the ripe ones stand there for toads to find.
- **Dreaming.** The hero lies on its back at the Ring for 1,000 ticks (draft), doing nothing. A hit wakes it at once, with no vision.
- **The vision.** On waking, it has seen one of two things. The panel says which, and the map marks it for a day:
  - **An orb carrier:** one of Gorf's creatures that carries an orb, marked in the orb's colour. This needs one more New rule: Gorf's creatures roll what they'll drop when they hatch, not when they die, so there's something true to see.
  - **The next raid:** which of Gorf's nests sends the next wave, and roughly when. These are RockCollector's wave times, and document 03 sets them.
- **The dreamer wants it.** It halves its asking price for whatever it dreamt of, so the vision comes with a willing hero.
- **What it costs:** a spotted cap, the Hold's slowest material; the hero's time; and a hero lying helpless at the Fairy Ring. The Fairy Ring's bigger plot holds two toadstools, so one dream doesn't stop the venom gear.
- **On screen.** The panel line reads, for example, "PIP DREAMT OF A HERON WITH A SUN ORB".

So where the Fairy Ring goes matters twice. Put it near Toad Hall, and the toads pass it more, and dream more.

### The fallen

A higher-level hero's husk lasts longer (see "Husks and wisps"). Something in the marsh scavenges husk purses, and document 03 says what.

## Stories this should make

Left alone, the rules above should produce stories like these:

- **Pip and the heron.** Pip the toad's friend is taken by a heron. Pip takes heron rewards cheap from then on, and saves for a venomiron spear.
- **The first spear.** For the Hold's first days the toads are all slingers, because its one salamander isn't yet skilled enough for a spear. The day it is, the bold toads empty their purses at Toad Hall.
- **The fairy ring.** The one caecilian skilled enough to plant the Fairy Ring is eaten. The venom gear stops until another one learns.
- **The dry spell.** A dry spell strands the axolotls at the pond's edge, and the toads have to go to the far nest alone.
- **The noon march.** The Chorus Toad's call Bolsters two toads in the morning. With four more points of constitution to dry out, they reach the east nest at noon, when its herons are out hunting.
- **The softened shell.** A snapping turtle shrugs off the toads' spears. Then one toad with a venomiron spear lands three poisonings, the turtle's defence drops, and the bullfrogs' smashes start getting through. The turtle's Sun Orb falls in the reeds, and the toad carries it home.
- **The dream.** You stop wanting spotted caps for a day. Pip finds a ripe toadstool standing in the Fairy Ring, eats it, and dreams of a heron carrying a Sun Orb. Pip wakes, and takes your reward on that heron for half its usual price.
- **Twen.** Twen is killed on the way home with the Shell Forge's takings. A passing bullfrog picks the dew up and buys a peytral with it.
- **The Olm.** The Olm wakes at last, after three orbs and a long wait for the Spawning Pool. Its Alter turns two worn-out toads into the strongest pair in the marsh.

## Density check

Majesty is the floor for how much each kind carries, not for which features it has. In the research (`majesty/heroes.md`), a Majesty hero class has:
- about six to eleven weighted habits
- a few abilities gained by level or bought
- something to spend its gold on
- two or three ties to other classes or buildings
- a weakness that players plan around

This checks each Hold kind against that.

| Kind | Habits | Abilities | Spends on, or uses | Ties to other kinds | Weakness |
|---|---|---|---|---|---|
| Newt | fixed list of 8 | gathering, mending, bite; shovel blow and digging at 5 | shovels from stock | carries for everyone; Twen's bonus; its skin poisons what eats it | −4 strength; herons by day |
| Caecilian | fixed list of 6, with tending | fungiculture | | feeds every maker; one skilled caecilian gates the Fairy Ring | slow at hard fungus |
| Tree frog | fixed list of 6 | woodcrafting | | makes the toads' first weapons and all the venomwood | skill gates |
| Fire salamander | fixed list of 6 | ironcrafting | | makes spears and plates | can make nothing when new (see "Slips") |
| Chorus Toad | fixed list of 7 | shovel blow, digging; the call at 5 | a shovel | Bolsters heroes before long marches | there's only one |
| Toad | 10 weighted | spear and sling | a weapon, then upgrades | axolotls follow it; young toads follow it; it dreams on the caecilians' toadstools | helpless unarmed, and its first weapon waits on iron |
| Axolotl | 7 weighted | lightning; pond light at 3; regrowth at 5 | lessons at the Star Pond; baubles | heals toads and blinds their targets | dries four times as fast; −2 constitution |
| Bullfrog | 7 weighted | Double Smash | plates, two shovels | axolotls follow it | slow to leave home; only one item resists magic |
| Goliath frog | 5 weighted | Smash, Heavy Repair | a shovel | escorts newts; mends after raids; builds pools | raids what you wanted left alone |
| The Olm | 4 weighted | bite, spear, pond light, Alter | bauble, spear, chanfron | Alters two allies | slow to wake; costs all three orbs |

**What the check shows.**
- The heroes are close to Majesty's density.
- Goliath frogs and the Olm are thinner. Many habits are reactions to enemies, so document 03 should add theirs.
- The folk run fixed lists by design, so the economy stays readable. Their character comes from their numbers and their spots instead.

**Majesty's broad behaviours the Hold keeps:**
- rewards set aside when posted
- purses, buying, and a tithe to the house
- heroes weighing chances
- heroes declining hopeless rewards (the danger thresholds, the asking price, and the Hold's new level limits do this)
- following to heal
- learning at a building
- the fallen brought back for a price
- homes lost and regained

**What the Hold doesn't copy:** temples and their exclusions, the Fairgrounds, the Elves' bargain and gambling halls, Statues and loyalty, Market Day and caravans. They rest on fantasy-kingdom tropes. If the Hold needs something in their place, it will come from its own systems.

## Slips in the Triblings data

Places where the data doesn't do what it seems meant to, with the fix for each. Henry took all of these fixes on 2026-11-27.

**Dead locks first.** These would stall the early Hold:
- **A new smith can make nothing, ever.** The easiest iron item needs skill 6, and a crafting skill grows only by crafting. *Fix (document 02):* a level-0 iron item, or salamanders that hatch with some skill.
- **Workshops can't keep their inputs.** Toad Hall, the Cauldron and the Ember Log make things from fungiwood, but can't keep any, and crafting needs the inputs in the workshop. *Fix:* every workshop keeps its own recipes' inputs.
- **Growers can't plant.** No ability carries the planting skill ("growing"), and Fungiculture has no harvest amount. *Fix:* Fungiculture plants as well as picks, with one shared skill on wisdom, and a harvest amount (document 02 sets it at 2). Then planting and picking both teach it.
- **The opening needs fungiwood before anyone can pick it.** The Burrow costs 4. *Fix:* the Hall starts with some (document 05).

**Missing or mismatched:**
- **Alter applies Regenerating, not Altered,** and Altered has no fading time. *Fix:* Alter applies Altered, fading at 1,000 ticks a stack (draft).
- **Smash isn't defined,** and the Strongling lists one unlock level for its two abilities. *Fix:* both at level 1. Smash hits one enemy for the shovel's damage, cooldown 20 (draft).
- **Nobody makes the iron shovel,** though the Meeting Den keeps five. *Fix:* the Ember Log makes it.
- **Slot names don't match.** The hero behaviours ask for spears, slingshots and shovels, but items are filed as "simple" and "primitive", so as written a spear and a sling would share one slot. The Mayor asks for "simple" gear, but its attack needs a shovel. *Fix:* name the slots shovel, spear, sling, head, chest, rump and bauble, and give the Chorus Toad a shovel.
- **The venomiron recipes ignore what Daot's Grotto keeps.** The spear and catapult take fungiwood and a cap ingot, and the chanfron a cap ingot alone, though the Alchemist's Den keeps spotted caps and Meteor Orbs. *Fix:* each venomiron item adds a spotted cap, and the spear a Meteor Orb as well.
- **The Dragon uses the Wiseling behaviour.** The unused Dragon behaviour has its gear slots. *Fix:* use the Dragon behaviour.
- **The wooden shovel lists two recipes,** fungiwood and another team's log. *Fix:* fungiwood only.
- **Lists that don't line up.** The Alchemist's Den has six stock items but three limits, and no building cost. The Dragon's hatching cost names three orbs but one amount. *Fix:* one of each orb for the Olm; document 02 sets the Grotto's.
- **Some buildings have no hatching times or resting.** The House of Wisdom, the Nursery and the Dragon's Den set neither. *Fix:* document 02 sets them.
- **The Fairy Ring's plot can be empty.** Its one-square plot holds nothing if the building lands on odd squares, since planting needs even ones. *Fix:* the bigger plots above, placed to fit.

## What comes next

Four more documents, then the build:
- **02 Dew and things:**
  - item prices, stock defaults and spawn costs
  - the recipes, with the slips fixed
  - ticks turned into Hold time
  - yields tuned, including the toadstool's slowness and the new smith
  - a headless simulation of the Triblings data plus the New rules, to check the pace of the early Hold and that the stories above really happen
- **03 The marsh's dangers:**
  - herons and their nests (ported from the graveyards and the crypt)
  - which creature drops which orb, rolled when it hatches so that dreams can see it
  - each nest's wave times
  - who tramples spots and scavenges husks
  - Gorf, the curse and the weather
- **04 The Elder's hand:**
  - the kinds of reward
  - marking puddles
  - the Ancient Magic
  - the panels and controls at 400×300, with mouse and keys
- **05 The first scenario:**
  - the map, and which track plays where
  - the starting stock
  - the guided start
  - the road to Gorf

## Henry's decisions, 2026-11-27

1. **The New rules:** all kept.
2. **The slips:** all the fixes taken.
3. **The bauble slot:** filled, with the dew bead, glow charm and star pearl (see "Gear").
4. **Kinds beyond the Triblings:** none for now. A playtest will show whether anything is missing.
5. **Length:** 30 to 60 minutes for the first scenario is right.
6. **Magic:** welcome. The folklore stays: Xolotl's lightning, salamanders born of fire, wisps, and the Star Pond's mages.
7. **Dreams:** back, with the vision.
