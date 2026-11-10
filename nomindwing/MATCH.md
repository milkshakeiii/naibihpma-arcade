# The match: design plan (2026-11-09; Match round 1 built the same day)

The proving ground answered its question: ships drawn from parts fly and fight the way their drawings suggest. A counter triangle exists, power makes module choices matter, and a 97 px ship works beside 3 px ones. What it can't tell us is which of that matters in a game. Toy 1 found the feel ("amazing") and no depth ("nothing to it"). This is the match that has to find the depth.

**The fun we're aiming at** (Henry): the mothership is the king and the root of a tree of factories, spawning self-propagating wings strategically, "the creativity of designing a TCG deck combined with the strategic thrill of outmaneuvering someone in chess".

**What the first match must show:**
1. Planting factories and watching them grow and print gives the awe.
2. There's strategy beyond tug-of-war: where you expand, which links you guard, which factories you plant against what the enemy is fielding.
3. The pace leaves time to think. "RTSs that go too fast are all click-spamming and reflexes without strategy or advanced tactics."

---

## How a match plays (the first minutes)

You start with your king, a small bank of pixels and nothing else, with two rocks nearby. Your deck is your 7 designs. The map is a mirror image, so the AI starts the same way across from you.

You have the king print a seed for one of your factory designs, then click the closer rock. The seed flies there and starts eating the rock: your factory grows out of it pixel by pixel, from its core outward. Its first factory pixels start printing the moment they exist, slowly at first. The first thing on its build order is a scoop ship. It starts chewing the other rock and hauling pixels back to the nearest of your buildings, and you watch your bank tick up.

Meanwhile you fly the king, choose which rock to seed next, and keep an eye on what the enemy is planting. Their factory designs tell you what's coming. Your counter is to plant a different factory, or to have the king print something specific: the king can print anything, and its builds arrive by hyperspace.

The fight is over when a king dies.

---

## The rules

### Map and winning
- **Kill the enemy king to win.**
- **The map is mirrored through its centre,** like Toy 1, so neither king is ever a pixel closer to anything. In Toy 1, being 0.35 px closer to one rock won 5 of 6 matches.
- **The map is about 1200x900** (placeholder; Toy 1 was 960x720), scrolling, with a minimap. It's bigger for the slower pace and the bigger ships.
- **Rocks run from about 150 px to about 3,000 px.** The big ones are rare and central, so the biggest factories need contested ground.
- **No fog of war** for now.

### The deck
- **You bring your designs and can't edit them mid-match.**
- **Each design is either built or seeded.**
  - A **built** design is printed by a factory or the king.
  - A **seeded** design is grown from a seed on a rock. The yard locks the seed's core at the centre of the grid and shows how big a rock the design needs (its pixel count).
- **A design with factory pixels has a build order:** a looping list of built designs from the deck, set in the yard and fixed for the match. Any entry the full-grown design could never build is flagged.

### The king
- **You fly it** with WASD.
- **It prints anything in the deck,** with no size limit (the one exception to the factory rule), at a fixed rate.
  - Built designs arrive by hyperspace: a blue line sweeps across the ship and reveals it column by column, like Homeworld's hyperspace window.
  - Seeded designs come out as a seed, and then you click a rock.
- **It's a network node:** it stores pixels and links to your other nodes. Flying it away can split your own network.
- **For now, the king is a fixed design.** Designing your own king "could be quite fun" (Henry), so it's a feature for later.

### Seeds and growing factories
- **A seed is a fixed little kit:** a core and an engine, unarmed. It costs a set price and flies at fighter speed. Killing seeds in flight is the cheapest counterplay.
- **On the rock, it grows its design outward from the core.** Each new pixel eats one rock pixel. The pixel nearest the core grows first, so the factory is one connected, legal ship at every step.
- **Guns work while it grows.** They fire as soon as they appear.
- **Its factory pixels grow it first, then print.** While it grows, a seed eats at a base rate and every factory pixel grown so far adds its printing to the growth. A design with factory pixels near its core snowballs its own growth. Once it's whole (or the rock runs out), the factory pixels start on the build order. (Built this way so a factory pixel never does two jobs at once; see the judgement calls below.)
- **If the rock runs out first, the factory stays unfinished:** fewer factory pixels, so it's smaller and slower.
- **No claims, no ownership.** A rock is just pixels: anyone can mine it, and anyone's seed can eat it. Two seeds on one rock share it, and each grows from what it gets. The planted factory is the claim, and taking it back means destroying it.
- **Only the king prints seeds.** Toy 1's outposts printed their own seeds, so expansion compounded for free. Here expansion goes at the king's pace, one deliberate planting at a time. This throttle is what replaces claims.
- **A seeded design with engines can fly off its rock once grown,** as a mobile shipyard.

### Factories and printing
- **Size and speed both come from the factory-pixel count.** A factory can build ships up to 10 pixels per factory pixel. Each factory pixel prints one ship pixel every 4 s. So every factory takes the same 40 s to build the biggest ship it's allowed, and smaller ships come out faster.
- **Build order rules:**
  - It builds the next thing on its list that it's big enough for. Anything too big is skipped, and shown greyed out, until the factory grows into it.
  - **It pays as it prints:** a pixel at a time from its network's bank. The ship appears pixel by pixel, just outside the factory on the side facing the enemy king, growing from its core outward like a growing factory. A half-printed ship is really there and can be shot. It launches when it's complete.
  - The king's builds are the exception: they're printed out of sight and arrive whole by hyperspace.
  - Short of pixels, it stops half-printed and carries on when pixels arrive. It never skips to something cheaper.
  - Factories sharing a bank take turns, a pixel each.
  - A factory that loses factory pixels partway through a ship finishes it more slowly. The size limit only applies when it starts the next one.
- **Carriers:** factory pixels work on built ships too. A carrier prints from its own build order wherever it flies.
- **Factory pixels are a part** in the Industry category: weight 1, using 1 power each, like other modules.

### Mining and pixels
- **Scoop ships mine any rock.** About 2.5 px/s with one scoop and 5 px/s with two or more (the proving ground's current rates), plus travel time.
- **They haul to the nearest node of their own network.** A rock near your network is cheap to work. One near theirs means long, exposed trips past their guns. Territory comes from distance and guns, not a rule.
- **Loose pixels are anyone's:** from shots, kills and spilled stock. Once the rocks are gone, the economy runs on wreckage.

### Networks and split pools
- **Nodes are the king and anything with factory pixels** (factories and carriers). In round 1, every node has a fixed link range. Two of your nodes link when their ranges overlap, and links are drawn as dotted lines in your colour.
- **Pixels are stored at nodes.** A network's bank is everything its nodes hold. A delivery goes into the node it reached, and printing draws from the nearest nodes in the network first.
- **A split is physical:** cut a link and each half keeps the pixels sitting in its nodes. Reconnect and they pool again. You're told when a network splits, and hovering a factory shows its network's bank.
- **A destroyed node spills its stock** as loose pixels anyone can scoop.

### Ships
- **Everything printed flies itself by its design's order** (Hunt, Guard, Harvest, Hold), with the proving ground's three-layer brain. There's no unit control, so there's nothing to click-spam. The clicks are strategy: what the king prints, where seeds go, which factories run.

### The AI
- **A fixed deck and build orders.** It starts like you, with a king and a bank. It seeds rocks nearest-first, mines, and its factories send warships on Hunt.
- **It must expand and attack on a schedule.** Toy 1's passive AI was the pushover that made outpost spam look like a strategy.

### Multiplayer later
- **PvP is a core part of the vision.** The sim is whole numbers only and deterministic: the same inputs always replay the same match. That's what lockstep multiplayer needs (each machine runs the same sim from the same inputs). The match must keep that: no floating point, no frame-time effects, and all randomness from the seeded generator.

---

## Starting numbers (all placeholders, tuned toward 15–20 minute matches)

| What | Number | Why |
|---|---|---|
| Map | 1200 x 900 | Toy 1 was 960x720; slower pace, bigger ships |
| Starting bank | 300 px at the king | A seed and a few scoop ships |
| King's print rate | 6 px/s | A fighter in about 1 s, a 644 px capital in about 2 minutes |
| Hyperspace arrival | 2 s | Long enough to see, short enough not to matter |
| Seed | 20 px, fighter speed | Cheap enough to plant, dear enough not to spam |
| Seed's base eating rate | 4 px/s, plus the factory's own print rate | A 150 px factory with few factory pixels grows in about 35 s |
| Factory print rate | 1 ship pixel per factory pixel every 4 s | A 20-factory-pixel factory prints 5 px/s |
| Factory size limit | 10 px per factory pixel | Its biggest ship takes 40 s |
| Factory pixel | weight 1, uses 1 power | Like other modules |
| Link range | 80 px per node (king 120) | Links span about one rock gap |
| Scoop mining | 2.5 / 5 px/s at the rock | Current proving ground rates |
| Toy 1, for comparison | 15 px/s per factory; matches lasted 3½–6½ minutes | Too fast |

---

## Match round 1: what gets built

A bare toy before wrappers: everything the loop needs, nothing it doesn't.

| # | Addition | What to look for | Verdict |
|---|---|---|---|
| M1.1 | **The board:** a mirrored map of rocks, your king flown with WASD, kill the king to win. The proving ground's sim at map scale, with a minimap. | Does it feel like Toy 1 (the feel you liked), just slower? | |
| M1.2 | **The deck in the yard:** built or seeded per design, the factory part, and build orders edited in the yard. | Is designing a factory and its build order fun in itself? | |
| M1.3 | **Seeds grow factories** pixel by pixel out of their rock, their own factory pixels speeding the growth. | Is planting and watching a factory grow the awe? | |
| M1.4 | **Factories print** by the count rules, pay as they print, skip what's too big, wait when broke. Carriers work the same way. | Can you read what a factory will do next? | |
| M1.5 | **Mining and the network:** scoop ships haul to the nearest node, linked nodes pool, splits divide by where the pixels sit, destroyed nodes spill. | Does where you build start to matter? Is a split ever legible and fair? | |
| M1.6 | **The king's hyperspace builds:** anything in the deck, arriving behind a blue line. | Is the king the quick lever it should be, without making factories pointless? | |
| M1.7 | **An AI** with a fixed deck that expands and attacks. | Does it put up a fight, or is it Toy 1's pushover? | |
| M1.8 | **The pace,** with the numbers above. | Is there time to think? Does a match last 15–20 minutes? | |

**Built (2026-11-09).** From the yard, the MATCH button offers your 7 designs as your deck, or the starter deck; the AI always plays the starter deck. Keys: WASD fly the king, 1–7 or a card prints, a seeded card then wants a rock click (right-click or Esc cancels), Space pauses, F changes speed, F1 explains. Every match is logged to `proving_ground.log`: the start, a line a minute, and the result.

**Judgement calls while building** (say if any are wrong):
- **A growing factory doesn't print.** Its factory pixels help it grow instead, then start the build order once it's whole, so one pixel never does two jobs. The plan said both "it works while it grows" and "growth speeds up as it goes", which together would have counted every factory pixel twice. Guns do fire while it grows.
- **The starter deck** is the default fighter, lancer, frigate and capital, plus a **MINER** (the gunminer) and two seeded factories:
  - an **OUTPOST**: 121 px, 20 factory pixels, builds up to 200 px; its build order is miner, miner, fighter, fighter, lancer, frigate;
  - a **SHIPYARD**: 513 px, 88 factory pixels, builds up to 880 px; its build order is miner, frigate, frigate, capital.

  You can play it from the menu before you've drawn factories of your own.
- **The king** is a fixed 17x17 design with 3 cores, 5 flak and 4 engines. It flies at Toy 1's speed: 3 px a tick, straight from the keys, but it can't fly through bigger enemies or anything planted. Everything it prints arrives by hyperspace, seeds included.
- **Nodes** are the king and anything with factory pixels. They store pixels and link to each other when their reaches overlap (80 px each, the king's 120). A miner delivers to the nearest node of its own side, whichever network that's in.
- **Printed ships** appear beside their factory on the side facing the enemy king and launch when whole. If the factory dies first, the half-printed ship flies off as it is. One shot while printing is started again.
- **The AI**:
  - keeps its king home and backs it away from armed enemies;
  - plants a factory on the nearest free rock on its own half, aiming for one more every 75 s (outposts first, shipyards once it has two);
  - from 4:00, queues its biggest built ship every 2½ minutes.

  Its warships go on Hunt as they're printed, so attacks come as a trickle, not as timing attacks: that has to wait for switching orders mid-match.

**What the headless runs showed** (`--match-sim 12`, AI against AI with the starter deck):
- **Wins split 6–6** (7–5 after the self-review fixes below). The map is a mirror image and the sides play the same, so no side bias shows.
- **Matches lasted 7½ to 17½ minutes, about 10¾ in the middle** (9 to 13½, about 10½ in the middle, after the fixes). That's short of the 15–20 target.
- **Early ships come fast:** about 30 a side by 2:00, and 250–400 printed per side over a match.
- **Each side plants 4–6 factories.**
- **The kings rarely finish summoning a capital**, because the factories spend the bank as fast as the miners fill it.
- **Levers if it's too quick:** the factory print rate (one pixel per factory pixel every 4 s), the king's 6 px/s, mining, and the size of the map.
- **Proving ground unchanged:** all 26 of its matchups gave exactly the results they did before the match was added.

**Self-review before the first playtest (2026-11-09).** Henry asked for a check for subtle bugs like the fighters inside the big ship and the teleporting flak. A read-through plus probes that ran whole AI matches checking invariants every tick found these, all fixed, each with a test that fails without the fix:
- **Factories didn't take turns.** Income went to whichever printer was built first, so later factories starved for minutes; one miner sat at 1 of 9 px for over two minutes. Now everyone printing takes a pixel in turn, and who goes first moves round each tick. In the test, two yards fed 1 px a second print 12 or 13 ships each, where one used to print 25 and the other none.
- **A factory could grow through an enemy.** A seed sent to a rock where an enemy factory sat grew its factory straight through it. Now a pixel can't grow, or be printed, where an enemy pixel is: the factory grows round, and if it's wedged with nowhere to grow, its factory pixels print instead of waiting for ever.
- **Seeds could get stuck.** One whose rock was mined or eaten away before it arrived hovered for good, and the AI, seeing a seed in flight, never planted again. One whose target pixel ended up under an enemy factory pushed at the hull for good. Both now come down on the nearest free rock.
- **A ship could arrive by hyperspace inside an enemy.** A frigate was caught inside an arriving capital for 7 ticks. Now an arrival stays in hyperspace, beside its king wherever the king goes, until its spot is clear.
- **A shot that started a tick inside a pixel skipped that pixel.** That can happen when a pixel grows round a shot, a ship arrives round one, or a gun fires from inside an enemy. Now the shot meets the pixel it's in first (flak on plate bounces). This is in the shared sim, so it also applies in the proving ground: 9 of its 26 matchups moved by a ship or a few pixels, with no winner changed; 85 fighters against 3 destroyers went from 8 destroyer wins and 4 draws to 5 and 7.
- **A carrier's half-printed ship moved after shots were resolved,** so shots never saw it move. It now moves with everything else.
- **A factory could pay for a pixel it couldn't place,** and a growing one could eat a rock pixel without growing. Both now look first.
- **F1 did nothing in a match,** the AI treated a rock as taken if any ship was near it (its own miners included), and undo could leave a seeded design without its seed core.

**What the probes confirmed was already right:** the bank's books balance exactly every tick (what's mined plus the starting bank equals what's stored, spent and lost with destroyed nodes), nothing leaves the map, no store goes negative, the map is an exact mirror image, the same seed replays the same match, and a factory eats exactly one rock pixel for each pixel it grows.

**Hardening before the first playtest (2026-11-09).** Henry asked whether the code was fit to grow and to stop the same kinds of bug coming back, then said to harden it now. Nothing here was meant to change play; two more bugs turned up on the way and were fixed.

What was restructured:
- **One table defines every part** (`PART_INFO` in `design.rs`): its letter, name, category, weight, armor, power, colour, icon and help text. Adding a part is one row; the yard, the rules panel and the hover texts are generated from it.
- **One gate decides where pixels may be** (`sim/space.rs`). Every move, turn, carried ship, growing pixel, printed pixel and arrival goes through it. A test reads the sim's source and fails if any other line moves a ship.
- **All input reaches a match as commands** (`Cmd::Steer`, `Cmd::Print`), the AI's included, applied on the tick and logged. A test replays an AI match from its log into a match with no AI and gets the same match, which is what replays and lockstep PvP need.
- **The arena and the match draw the world with the same code** (`scene.rs`): backdrop, loose pixels, ships, shots, sparks, minimap ground.
- **A standing audit** (`sim/audit.rs`) says what's wrong with a fight as it stands: a ship in two pieces or off the map, stats that don't match its pixels, a bank that doesn't add up, two factories sharing a pixel. Soak tests run it every tick through every arena test, a whole AI match, and four matches driven by random commands that do what no AI would (kings charging each other, both sides planting on the same rocks, a carrier). In tests the sim also checks each moment as it happens: no pixel grows onto another, no move ends inside a bigger enemy, no shot flies through a living pixel, nothing arrives inside an enemy, nothing without thrust moves.
- **The audit was itself tested** by breaking nine rules on purpose, one at a time. The first try caught four of seven: the random matches never grew a factory against an enemy or printed beside one. They now measure how often each rule had something to stop and fail if it never did. All nine breaks are caught.

The two bugs the random matches found, both fixed:
- **A bigger ship pushed things without engines out from under it.** Fly a king over an enemy outpost and the outpost slid across the rock a pixel or two a tick; hulks with no engines left were pushed the same way. Now only something that can move itself gets out from under a bigger ship. A factory stays exactly where it is.
- **A side could plant its factories on top of each other.** Growth only made way for enemy pixels, so a second seed sent to the same rock grew straight through the first factory. Now a factory growing out of a rock makes way for anything of its own side that's standing there too: the second comes down beside the first and grows what it has room for (28 of 49 px in the test).

What moved: 6 of the proving ground's 26 matchups by a second or a few pixels, no winner changed (hulks are no longer pushed). AI matches play out differently from the point either rule first bites: over 36 seeds, blue 15 and red 21 (the first 12 went 8–4 to blue, so neither side is favoured beyond chance), lasting 7:27 to 18:41, about 10¾ minutes in the middle. Five of the 36 reached the 15–20 minute target.

Not done: the ship record is still one struct of 50-odd fields, shared by fighters, factories, kings and seeds. It hasn't caused a bug yet, and splitting it is a bigger change than this pass.

**Rulings after the hardening (2026-11-09), and what they changed.**
- **Whatever is planted is solid to every enemy ship** (Henry: "making planted factories solid makes sense"). Anything grown out of a rock and still standing where its seed came down, a factory mostly, stops every enemy ship whatever its size. Nothing flies over an enemy factory or sits on one, so the two no longer fire from inside each other's plate. A king steered straight at a factory stops flush against it; steered at an angle it slides along the plate and past. Your own ships still fly over your own factories. Henry finds it slightly odd that the factory is solid when the rock under it isn't, and accepts it for now.
- **Moves are walked a pixel at a time** (needed by the rule above). A move used to be checked only where it ended. So a ship stopped up to 2 px short of what it ran into, depending on where its steps fell, and a ship one pixel wide flying 3 px a tick could land beyond a wall one pixel thick without touching it. Now every pixel of a move is checked: ships stop flush at any speed, slide along whichever axis is free, and nothing skips a thin wall. A turn that would swing a ship into something planted is refused, and the way out from under a bigger ship never leads through one.
- **A seeded design can have engines and guns** (Henry: yes). It grows out of its rock like a factory, planted and solid while it grows, then lifts off whole and is a ship like any other. No factory builds it, so no factory's size limit applies: it costs the seed, the rock and the time.
- **The king's engines** (Henry: king design is an open question in general, and should be cheap to change later). Left as it is: the king flies at full speed whatever state its engines are in.
- **Seeds of one side on one rock block each other** (Henry: yes), and you may do it on purpose. A seed can be sent to any rock, one with a factory of yours or the enemy's on it included; it comes down on the nearest free rock pixel and grows until it meets something. The planting bar now says when the rock already has something planted on it.

Checked like the rest of the hardening. The audit has one more line: nothing planted shares a pixel with another planted thing or with any enemy ship on the map. The random matches count how often a ship at least as big as a planted enemy was pressed up against it (33 ticks) and fail if it never happened. Five more rules were broken on purpose (planted things not solid; the way out from under a bigger ship leading through a factory; moves not walked; a seed's growth not marked planted; a turn swinging into a factory), which makes 14, all caught. 99 tests.

What moved: 12 of the proving ground's 26 matchups by about a second, since ships now stop flush against what they run into. No winner changed; 85 fighters against 3 destroyers went from 5 red wins and 7 draws to 9 and 3. AI matches over 36 seeds: blue 20, red 16 (it was 15 and 21, and 25 seeds kept their winner), lasting 7:56 to 16:05, about 10¾ minutes in the middle. None ran out the clock. Three of the 36 reached the 15–20 minute target.

Two things this opens, to watch in the playtest:
- **Walls.** A seeded design that's a line of plate with a core is now a wall no enemy ship can cross. Its pixels needn't sit on rock, only be paid for with rock, so a wall can run up to 99 px long from any rock big enough. That could be the terrain-making this game wants (a chokepoint, a castle round the king), or a cheap way to box things in.
- **A big ship's turn isn't swept.** A titan's quarter turn swings its whole hull round in one tick and can leave a lancer 12 px inside it, which then takes up to 9 ticks to get out. Planted things are safe from this, because a turn into one is refused. Small ships aren't.

**Left as they are, for Henry to rule on:**
- **Rocks stop nothing.** Shots and ships pass straight through rock, as they always have in the proving ground. It stands out more now that rocks are the terrain: a factory inside its rock gets no cover, and shots cross the giant rock in the middle. And more again now that the factory on a rock is solid to enemy ships while the rock isn't (Henry noticed, and accepts it for now).
- **Carriers can print carriers.** A built design with factory pixels can have itself on its build order, which compounds factories without the king or a rock. It costs pixels, unlike Toy 1's free outposts, but it sidesteps "only the king prints seeds".
- **Printing outruns mining.** From about four minutes in, both banks sit at zero: the factories could print about 50 px a second between them and the miners bring in 10 to 20. With turns, everything prints slowly at once, so half-printed ships sit beside their factories for minutes (12 for over two minutes in four matches).
- **Equal-sized enemies can overlap,** two kings included. Only a bigger enemy, or a planted one, is solid. They can't shoot each other from inside any more.

**Later rounds, already agreed in principle:**
- **Hunt sub-behaviors** (Henry, kept from the proving ground's parking lot): orbit, kite and attack runs at least, picked per design. Each one has to work for any ship drawn, not be tuned per design.
- **Switching a design's orders mid-match** (Henry, "quite possibly"): flip your designs between behaviors, so all your ships of one or more types can sit defensive for a while, then go together on a timing attack. That's strategy, not micro: one click moves a whole type. It changes orders, not the drawing, so it doesn't break "no editing designs mid-match".
- **Node pixels:** a part that extends link range, so relay ships are designs too and the network runs through ship design.
- **Designing your own king.**
- **A pause switch per factory**, if a factory building the wrong thing turns out to be a real problem.
- **Shields.**
- **Smarter pixel targeting.**
- **PvP.**

---

## Where it could go wrong

- **Free land again.** The rock pays for the factory. If seeds are cheap and the king prints them fast, it's Toy 1's outpost spam. Levers: the seed's price, the king's print rate, the eating rate.
- **An early rush to eat every rock.** The same throttle: the king's pace, and growing seeds being exposed.
- **Swingy cuts.** If one fighter can cut your only link and strand half your bank, that's a reflex check, not strategy. Links have to be sturdy enough that cutting one is a commitment, and a mesh of links should survive one cut.
- **Readability.** Toy 1's rules weren't readable ("how does rock claiming work?"). Links, banks, splits, a factory's next build and a seed's target all have to be visible at a glance or on hover.
- **Performance.** A bigger map, dozens of factories printing, ships up to 99 px, and spilled stock as loose pixels. Spills may need capping.
- **The AI.** Expanding organically and attacking sensibly is more than Toy 1's script. Expect a round of AI work.

---

## Settled (Henry, 2026-11-09)

1. **Deck size:** 7 slots, the yard's, for now.
2. **The king:** a fixed design for now. Designing it comes later.
3. **Where printed ships appear:** just outside the factory, on the side facing the enemy king. They print pixel by pixel.
4. **A pause switch per factory:** not for now. Kept as a possible addition.
5. **Starting position:** just a king and a bank.
