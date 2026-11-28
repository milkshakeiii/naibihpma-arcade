# Naibihpma Hold, document 03: the marsh's dangers

Draft 1, 2026-11-27. The third of five design documents. It covers what threatens the Hold, what each threat wants, what it carries, and what the Hold has against it.

**Where it comes from:**
- **RockCollector's Testario scenario.** It has graveyards of skeletons that drop the three orbs, a small graveyard with one early wave, and a crypt of ghouls that must be destroyed to win. Its waves come at set ticks, and wave creatures hunt, then raid any building.
- **The demo.** Herons guard their nests, call for help and raid the Hold. Gorf's stone rises in the marsh, and its wisps drain dew.
- **Majesty's shape.** Trouble comes from four kinds of source. Each monster wants something particular. A lair holds a cache. Building next to danger works as a decoy.

Every number here is a draft, and none has been run yet. Document 02's simulation covers the economy only. It should grow to include these enemies and be tuned against them before anything is built, so the danger curve below is a target, not a result.

## Four sources of trouble

1. **Lairs.**
   - Heron nests breed herons, and each sends raids on its own schedule.
   - Destroying a nest is the only lasting fix, and nobody can be ordered to do it. You post a reward, or a goliath frog finds the nest on its own.
2. **Wanderers.**
   - Snakes and crows come in from the edges of the map.
   - Killing them doesn't stop more coming. It only thins them.
3. **Trouble that grows with the Hold.**
   - Leech pools appear in the water as you build beside it.
   - Gorf's Stones rise as the Hold gets bigger.
   - The Hold's success is what calls them.
4. **Gorf's Hollow.** The sorcerer's own lair, guarded by snapping turtles. Destroying it wins the scenario.

## How enemies decide

Enemies use RockCollector's behaviour lists, as the Hold's folk do, with a fixed list each. **What it wants** is the first line of each list. It's the motive you learn to plan around.

- **Target priority.** As in RockCollector, an enemy attacks creatures in range first, then the nearest building. So a cheap building between a nest and the Hall is a decoy, and a puddle dug toward a nest is also a path for herons.
- **Danger.** Each enemy has a difficulty of 10 + its level, which is what heroes weigh when they decide to fight or flee (document 01).
- **Day and night.** Each enemy has its hours:
  - Herons and snakes are about by day.
  - Wisps come out at night.
  - Crows fly at dusk and dawn.
  - Turtles and Gorf keep no hours.

## The enemies

| Enemy | Health (level 1, per level) | Attributes | Attack | Moves | Comes from |
|---|---|---|---|---|---|
| Heron | 9 (+3) | dex +4, wis +2 | beak 1d6, melee, cooldown 25 | flies, 8 ticks a square | heron nests |
| Snake | 10 (+3) | str +2, dex +2 | bite 1d6, melee; swallow (see below) | 10 ticks a square, through reeds and water | the map's edges, by day |
| Crow | 5 (+2) | dex +6 | peck 1d3 | flies, 6 ticks a square | the crow roost |
| Wisp | 6 (+2) | int +2 | none; drains dew | floats, 12 ticks a square, through anything | Gorf's Stones, at night; husks left out |
| Snapping turtle | 40 (+3) | str +8, con +8; shell: +10 defence, dodge and resistance | jaws 2d8, melee, cooldown 40 | 20 ticks a square | Gorf's Hollow |
| Gorf | 120 (level 10) | int +6, wis +6, con +4 | the curse, binding, the croak (see below) | stays in the Hollow until it's breached | Gorf's Hollow |

Herons port the Testario skeletons, snapping turtles port the ghouls, and Gorf's Hollow ports the crypt.

### Herons

Real herons stand in the shallows and spear frogs, newts and fish. In the album they are Gorf's birds.

- **What a heron wants:** folk in the open, and above all a newt carrying dew, the slowest and most visible prey.
- **By day,** a nest's herons fly out to hunt up to 12 squares from it, and land where folk are picking or carrying. Testario's skeletons hunted within 10. One or two stay to brood. This is the doc 01 trade-off: the nests are nearly empty by day.
- **At night,** they roost on the nest, all of them. A full nest is five herons standing close together, which is exactly what the axolotls' pond light wants.
- **Calling.** A heron that's hit calls, and every heron from its nest within 15 squares comes, as in the demo.
- **Wading.** A heron that lands on a plot tramples it back to bare.
- **What it carries.** As in Testario, each heron may carry an orb: one chance in six each of Sun, Clargent or Meteor, and one in two of nothing. **New:** the drop is rolled when the heron hatches, so a dream can see it (document 01). In the fiction, herons swallow the fallen stars of the Star Pond the way other birds swallow grit.
- **Numbers that matter.**
  - Dexterity +4 makes them hard to hit with slings, which roll against dodge.
  - Spears roll against their poor defence, so spears do better than slings.
  - Their 9 health is three good sling hits.
  - Venom hardly matters against herons. A venomiron spear's 4d6 kills a 9-health heron in a hit or two, before its poison counts for anything.

### Snakes

Real grass snakes and garter snakes eat toads, frogs and newts. Garter snakes are famous for being immune to newt poison.

- **What a snake wants:**
  - a dug-in toad, which it can smell and dig out
  - eggs in a building's pool while a hatching is under way
  - any small creature alone
- **Immune to newt skin.** Newt poison (document 01) never lands on a snake. Every other predator thinks twice about newts, but snakes don't.
- **Swallowing.** A snake that hits a newt, caecilian, tree frog or axolotl at a quarter health or less swallows it whole. The swallowed creature is out of play and not yet dead.
  - Kill the snake within 1,000 ticks and it comes out alive.
  - Otherwise it dies inside the snake. Kill the snake later and the husk falls out, so it can still be regrown.
- **Eggs.** A snake that reaches a building while it's hatching eats the eggs. The hatching is lost, and so is what you paid for it.
- **When they come.** One at a time, from the nearest map edge, every 2 to 3 minutes by day after the eighth minute. They carry nothing.

### Crows

Crows aren't Gorf's. They come for what's left lying around.

- **What a crow wants:**
  - anything lying on the ground for more than 30 seconds: dropped dew, a cave's picked fungus, an orb
  - the purse in a husk
- **The roost.**
  - A crow carries what it takes to the crow roost at the map's edge. The roost keeps everything it's brought.
  - Destroying the roost spills it all, so that's a cache worth a reward.
- **When they come.** At dusk and dawn, one or two at a time, but only once there's something to take.
- **Numbers that matter.** Crows are fast and hard to hit, and they have only 5 health. Slings and lightning are the answer.

This is where document 01's carrying order pays off. Newts deliver before they gather, so fungus doesn't lie long. If the caves are far from anything that wants their fungus, crows learn the way.

### Wisps

Wisps are forest spirits trapped in Gorf's procession, the "music-box procession" of the album's Gorf track.

- **What a wisp wants:** dew. It drifts to the nearest ripe fern or the nearest building holding dew, and hangs there draining 1 dew every 150 ticks. It carries what it drains. A stone's three wisps can take about 50 dew in a night if nothing stops them.
- **Where they come from:**
  - Each Gorf's Stone lets out a wisp every 600 ticks at night, up to three.
  - A husk left out for half a day becomes one (document 01).
- **Breaking one.** Any hit breaks a wisp. It drops the dew it drained.
  - If it was a husk's spirit, it goes home to the Star Pond to be regrown.
  - If it was one of the stone's own wisps, it fades.
- **The bullfrog's call.** A bullfrog calling from the Spawning Pool at night draws every wisp within 20 squares to the pool and away from the ferns (document 01). The bullfrog then smashes them.
- **At dawn** the stone's own wisps go back into their stone. Freed husk-wisps stay put.

### Snapping turtles

Snapping turtles eat frogs, and their shells make them the marsh's armour.

- **What a turtle wants:** to guard Gorf's Hollow. Turtles stay within 8 squares of it, attack anything that comes nearer, and never raid.
- **The shell.** +10 defence, dodge and resistance, as the Testario ghoul had.
  - Slings, spears and pond light mostly fail.
  - **New:** the Hold guards Poisoned with constitution. The data names no guarding attribute. Turtles' constitution is high, so poison seldom lands on them.
  - When it does land, it softens them. Defence rolls on constitution, so every two stacks of Poisoned cost a turtle a point of defence.
- **Answers:**
  - Bullfrogs' Double Smash, swung by +6 strength.
  - The Olm, whose Alter makes two allies stronger still.
  - Many attackers at once.
- **What each carries.** **New:** each turtle carries an orb for certain, rolled at hatching like a heron's.
- **Trampling.** A turtle tramples any plot it walks over, though it rarely leaves the Hollow.

### Gorf

Gorf, the malevolent sorcerer of the album, is a frog. Gorf stays in the Hollow until it's breached, which means its health falls below half, and then comes out to fight within 15 squares of it.

| Gorf's spell | What it does | Range | Recharge |
|---|---|---|---|
| The curse | Turns a 5 by 5 patch of ground into cursed ground for 6,000 ticks | 20 | 1,500 |
| Binding | Turns the nearest husk into a wisp at once | 15 | 600 |
| The croak | Every hero within 7 squares rolls against resistance, or is Frightened: flees for 200 ticks, whatever its temperament | 7 | 900 |
| Summoning | Calls the Hollow's turtles back to Gorf | anywhere | 1,200 |

Gorf has no drop. Destroying Gorf's Hollow wins the scenario, and Gorf goes with it.

## Lairs

Lairs are buildings, as in RockCollector: they have health, they house their creatures, and the creatures rest and heal there.

| Lair | Health | Houses (hatching time) | Waves | Cache when destroyed |
|---|---|---|---|---|
| Heron nest | 150 | 5 herons (600) | every nest has its own schedule (see "Raids") | a 50% chance of an orb, plus the orbs of any herons inside |
| Lone nest | 60 | 1 heron (600) | one raid of 1 heron at 13 minutes, if it still stands | none |
| Gorf's Stone | 200 | 3 wisps (600, at night only) | none; it curses ground instead | dew equal to what its wisps drained and didn't lose |
| Leech pool | 80 | no creatures; it infests water | none | none |
| Crow roost | 60 | 3 crows (900) | none | everything the crows brought |
| Gorf's Hollow | 600 | 5 snapping turtles (1,200), Gorf | none | the scenario is won |

The lone nest ports Testario's small graveyard, with its single early wave.

**Herons heal fast at home.** A heron resting at its nest heals 3 health every 20 ticks, as Testario's skeletons did at their graveyards. A long fight at a nest therefore favours the herons, so a reward on a nest should be big enough to bring a party. The nest itself doesn't heal.

## Raids

Each heron nest has its own wave times and sizes, as Testario's graveyards had: theirs were 1, 2 and 4 skeletons. During a raid the herons use RockCollector's wave behaviour:
- **Hunt** anything within 6 squares on the way.
- **Raid** the nearest building, at any distance.
- **Wander** if there's nothing else.

Draft schedule for the first scenario (document 05 places the nests):

| Nest | Raids at (minutes) | Herons in each |
|---|---|---|
| Lone nest | 13 | 1 |
| Near nests (2) | 6, 16, 26, 36 | 1, 2, 3, 4 |
| Middle nests (2) | 11, 21, 31, 41 | 2, 2, 3, 4 |
| Far nests (2) | 18, 28, 38 | 3, 4, 5 |

A raid sends that many of the nest's herons. If the nest has fewer, it sends all it has, so killing herons between raids thins the next one. A destroyed nest sends no more raids.

**Dreams see raids.** The "next raid" vision (document 01) marks the nest whose raid is next and its time. A raid of four is worth knowing about a minute ahead.

## Trouble that grows with the Hold

### Leech pools

- **When they appear.** For every four buildings the Hold has within 6 squares of water, one leech pool appears, in the nearest stretch of water that doesn't have one.
- **Infested water** is drawn with dark flecks. A creature standing in it gains **Leeched** (**New** condition): it loses 1 health every 60 ticks, up to 3 stacks.
- **The mirror of Dry.** Leeched fades only on dry ground.
  - Water clears Dry but can give leeches.
  - Dry ground clears leeches but gives Dry.
  - A drying toad has to choose which pool to drink from.
- **Clearing one.** Heroes can raid a leech pool like any lair (health 80). It comes back after 10 minutes if the count of buildings still calls for it.
- **New: goliath frogs fill them.** A goliath frog that raids a leech pool fills it with stones, and that pool never comes back.

### Gorf's Stones

- **When they rise.** A Gorf's Stone rises when the Hold reaches 10, 13 and 16 buildings. In the first scenario that's at about 10, 20 and 30 minutes. Each rises at a point in the marsh 20 to 30 squares from the Hold's newest building.
- **What a stone does:**
  - It curses the ground around it. The cursed patch starts 3 squares across and grows by a square a minute, up to 15.
  - At night it lets out wisps.
- **Cursed ground** is dry ground (document 01), drawn cracked:
  - Creatures on it gain Dry by day and night alike.
  - Spots on it stop ripening.
  - Puddles on it dry up.
- **Destroying a stone** lifts its curse over a minute, and the ground goes back to what it was.

So the Hold's growth calls stones toward itself. Building fast without fighting lets the curse reach your ferns.

## Weather

**New.** The weather changes a few times a game. Each change is announced a minute ahead, by a tint in the sky palette and a line in the panel: "RAIN BY THE NEXT DUSK".

| Weather | How long | What it does |
|---|---|---|
| Rain | 1,500 ticks | No one gains Dry anywhere. Every fern ripens as if by water. Herons stay on their nests. Wisps can't leave their stones. |
| Dry spell | 3,000 ticks | Dry comes twice as fast. Open-ground ferns don't ripen. Puddles shrink by a square a minute. |
| Dawn mist | the first 900 ticks of a day | Every creature sees half as far, so herons find fewer folk and folk see danger later. |

Rain is when heroes can reach the far nests without water, and when the far nests are full. A dry spell is when the axolotls stay home (document 01's story "The dry spell").

## What the Hold has against each

| Threat | Good answers | Poor answers |
|---|---|---|
| Herons hunting by day | Toads, who dry slowly; spears more than slings; goliath frogs escorting newts | Axolotls far from water |
| A full nest at night | Pond light on the roost, then toads; a large reward to bring a party | A lone hero |
| Herons raiding | Bullfrogs holding the Spawning Pool; a decoy building on the raid's path; the Chorus Toad's wide defence | Leaving newts carrying in the open |
| Snakes | Any hero; keeping toads from digging in far out | Folk alone; newt poison |
| Crows | Slings and lightning; carrying fast; a reward on the roost | Spears (crows don't wait) |
| Wisps | The bullfrog's night call; any hit; Gorf's Stones destroyed | Leaving ripe ferns unpicked at night |
| Leech pools | Goliath frogs; short drinks; dry ground afterwards | Long soaks |
| Gorf's Stones | A reward early, before the curse grows | Waiting |
| Snapping turtles | Bullfrogs in plates; the Olm's Alter; many at once; venom, slowly | Slings, pond light |
| Gorf | Resistance against the croak (the venomiron chanfron, the star pearl); killing the turtles first | Heroes with low wisdom, who fail the croak |

## Danger over a game

Majesty's quests start relaxed and get harder on a clock, and so does the Hold. In the first scenario:
- **0 to 10 minutes.** Herons hunt by day. The near nests send their first single herons at 6 minutes. Snakes start at 8.
- **10 to 20 minutes.** The middle nests start their raids, and the lone nest sends its heron at 13. The first Gorf's Stone rises with the tenth building, at about 10 minutes. The first leech pools appear. Crows come once something is lying about.
- **20 to 35 minutes.** Raids of three and four. More stones. The far nests start at 18. The curse begins to touch the edges of the Hold. Dry spells.
- **35 minutes on.** The road to Gorf: the Hollow, its turtles, and Gorf once it's breached.

## Stories this should make

- **The lone heron.** If the guided start's reward hasn't taken the lone nest, at 13 minutes a single heron comes from it, straight for the newts carrying dew home. Pip, the only toad near, has a slingshot and the heron dodges every shot. Twen drops the dew and runs.
- **The swallowed newt.** A snake swallows a newt by the stream. Mott the bold toad is 20 squares away. The panel counts down. Mott gets there with 300 ticks to spare, and the newt comes out of the snake.
- **The crows' hoard.** The Morel Cave is far from the Hall, and fungiwood lies there between carriers. The crows find it. A reward on the roost at last spills twelve fungiwood, three purses and a Sun Orb.
- **The stone by the ferns.** You build fast and the thirteenth building calls a Gorf's Stone near the newt houses. Its curse spreads a square a minute toward the ferns, and the toads won't take the reward on it until it's 40 dew.
- **The leech or the dry.** A toad comes back from the east nest with four stacks of Dry. The nearest water has leeches. It drinks, picks up two stacks of Leeched, and then has to walk home across dry ground to shake them off.
- **The turtle wall.** Three toads die against Gorf's turtles. Then the bullfrogs come in plates, with the Olm Altering them, and smash two turtles in a minute.

## Density check

The same check as document 01, by kind. Majesty's monsters each have:
- a motive
- a habit that players plan around
- something they carry
- an answer the kingdom has to build toward

| Enemy | Motive | Habit | Carries | Answer |
|---|---|---|---|---|
| Heron | folk carrying dew | hunts by day, roosts at night, calls its nest | maybe an orb (seen in dreams) | spears, pond light at night |
| Snake | dug-in toads, eggs, small folk alone | swallows; immune to newt skin | nothing | any hero, quickly |
| Crow | things left lying, husk purses | dusk and dawn; flies home to its roost | what it took | slings, carrying fast, the roost's cache |
| Wisp | dew | night; hangs on ripe ferns | the dew it drained | the bullfrog's call, any hit |
| Snapping turtle | guarding the Hollow | never raids | an orb for certain | bullfrogs, the Olm |
| Gorf | the Hold's end | curses, binds husks, croaks | (the win) | resistance, the turtles first |
| Leech pool | anyone in water | grows with the Hold by water | nothing | goliath frogs |
| Gorf's Stone | the Hold's ferns | grows with the Hold | the dew its wisps drained | an early reward |

## Open choices

These are my defaults. Tell me if any should go another way.

1. **Snakes swallow folk, who can be saved within 1,000 ticks.** Without it, snakes would be ordinary fighters.
2. **Crows aren't Gorf's.** They're scavengers who belong to nobody, and their roost is a cache.
3. **Leech pools and Gorf's Stones grow with the Hold** (Majesty's sewers and graveyards), rather than arriving on a clock.

## What comes next

- **Document 04: the Elder's hand.**
  - the kinds of reward and how they're paid
  - marking puddles
  - the Ancient Magic
  - the panels and controls
- **Before the build,** the simulation adds these enemies, and the schedule above is tuned against it.
