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

## The numbers (round 2's; round 1's in brackets)

| What | Number | Why |
|---|---|---|
| Map | 1200 x 900 | Toy 1 was 960x720; slower pace, bigger ships |
| Starting bank | 190 px at the king (300) | An outpost and two miners, with 51 px left to print with |
| King's print rate | 0.6 px/s, as if it had 16 factory pixels (6 px/s) | A fighter in 12 s, a frigate in 73 s, a 644 px capital in 18 minutes |
| King's own pixels | 1 every 4 s (none) | A miner's worth every 36 s: enough to restart, too little to live on |
| Hyperspace arrival | 2 s | Long enough to see, short enough not to matter |
| Seed | What it grows into, paid at once: 121 px outpost, 513 px shipyard (20 px) | Every factory is a purchase |
| Seed's base eating rate | 6 px/s, plus the factory's own print rate (4 px/s) | An outpost grows in about 20 s, a shipyard in about 85 s |
| Factory print rate | 1 ship pixel per factory pixel every 26.7 s (4 s) | The outpost's 20 factory pixels print 0.75 px/s, the shipyard's 88 print 3.3 px/s |
| Factory size limit | 10 px per factory pixel | Its biggest ship takes 4½ minutes (40 s) |
| Factory pixel | weight 1, uses 1 power | Like other modules |
| Link range | 80 px per node (king 120) | Links span about one rock gap |
| Scoop mining | A pixel every 2.7 s with two or more scoops, 5.3 s with one (0.2 s and 0.4 s) | A 4-scoop miner fills its 12 px in 32 s and earns about 0.3 px/s near home |
| Guard and escort | Fights within 160 px of where it was built; within 110 px of its king | Guard outreaches every gun, so nothing shells a factory unanswered |
| Toy 1, for comparison | 15 px/s per factory; matches lasted 3½–6½ minutes | Too fast |

---

## Match round 1: what gets built

A bare toy before wrappers: everything the loop needs, nothing it doesn't.

| # | Addition | What to look for | Verdict |
|---|---|---|---|
| M1.1 | **The board:** a mirrored map of rocks, your king flown with WASD, kill the king to win. The proving ground's sim at map scale, with a minimap. | Does it feel like Toy 1 (the feel you liked), just slower? | **Keep.** "Overall I'm pretty happy", no bugs noticed, "the gameplay is still pretty smooth". The camera has to come off the king, and Toy 1's tactical map would be welcome back. |
| M1.2 | **The deck in the yard:** built or seeded per design, the factory part, and build orders edited in the yard. | Is designing a factory and its build order fun in itself? | **Not judged.** All three matches were played with the starter deck. |
| M1.3 | **Seeds grow factories** pixel by pixel out of their rock, their own factory pixels speeding the growth. | Is planting and watching a factory grow the awe? | **Rework the cost.** No word on the awe. "It's much too easy to gobble up the whole map": 10 factories in the first minute of the third match. |
| M1.4 | **Factories print** by the count rules, pay as they print, skip what's too big, wait when broke. Carriers work the same way. | Can you read what a factory will do next? | **Rework the pace.** Ships come out so often "that it will feel spammy to the detriment of tactics". |
| M1.5 | **Mining and the network:** scoop ships haul to the nearest node, linked nodes pool, splits divide by where the pixels sit, destroyed nodes spill. | Does where you build start to matter? Is a split ever legible and fair? | **Not judged.** |
| M1.6 | **The king's hyperspace builds:** anything in the deck, arriving behind a blue line. | Is the king the quick lever it should be, without making factories pointless? | **Not judged directly.** Toy 1's right-click-to-repeat would be welcome back. In the second match the king summoned 106 ships, nearly as many as the factories printed. |
| M1.7 | **An AI** with a fixed deck that expands and attacks. | Does it put up a fight, or is it Toy 1's pushover? | **Rework.** "The AI is too weak." "I can't necessarily tell as much about balance and such if I just build anything and win simply because my opponent is not building stuff." |
| M1.8 | **The pace,** with the numbers above. | Is there time to think? Does a match last 15–20 minutes? | **Rework: too fast.** "It should be a big deal when you get your first frigate, build a new seed, get a capital." The three matches lasted 5:27, 4:02 and 2:22. |

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

**The starter deck is the starting library (2026-11-10).** Henry opened the yard to playtest and found the proving ground's seven warships and no deck: "redesign the starting ships into a match deck for me so I have something to work off of".
- **Your seven designs now start out as the starter deck,** the one the AI plays, drawing for drawing: FIGHTER, LANCER, FRIGATE, MINER, OUTPOST, SHIPYARD, CAPITAL. Seven slots hold a miner, two factories and four warships, so the destroyer and the titan made way. The deck itself is unchanged, so the first match is a mirror: whatever Henry redraws is the only difference between the sides.
- **Henry's own designs from the rounds are kept** in `proving_ground_designs_before_match_deck.txt` (a cannon gunboat, a seven-lance frigate, a 200-flak cross, a six-core capital, and the untouched lancer, gunminer and titan).
- **The arena's ready-made tests are rewritten for the deck** and named from whatever is in their slots: the triangle (three tests), fighters against miners, a capital against fighters and against lancers, and three raids on a factory. A seeded design placed in the arena stands planted, as on its rock.
- **The old fleet and its nine tests are the reference** for `--pg-sim` and the checks. All 26 of its matchups, six AI matches and all 25 earlier screenshots came out exactly as before. 100 tests.
- **Two headless tools came with it:** `--deck [FILE]` describes a deck card by card, and `--match-sim N --blue FILE --red FILE` plays the AI against itself with a saved deck on either side. So a deck Henry draws can be measured against the starter deck.

What the new tests and counts say about the deck as it stands:
- **Its factories are soft.** 12 lancers (84 px) kill an outpost (121 px) in 7 seconds and lose nobody. 4 frigates do it in 26 seconds, and a capital kills a shipyard in 36. A factory's flak reaches 48 px and bounces off plate; lances reach 100 and cannons 130, so a factory can't answer anything that outranges it. Left alone for the playtest, since the AI's factories are exactly as soft. A factory that survives a raid is the obvious first thing to draw.
- **It fields mostly frigates.** Over 8 AI matches a side made about 45 fighters, 23 lancers, 95 frigates and 88 miners, with a few capitals: the shipyards print frigates while they wait for the pixels a capital needs. Lancers beat frigates 12–0 at equal cost, so a factory that prints lancers is the obvious counter to try against the AI.

**Left as they are, for Henry to rule on:**
- **Rocks stop nothing.** Shots and ships pass straight through rock, as they always have in the proving ground. It stands out more now that rocks are the terrain: a factory inside its rock gets no cover, and shots cross the giant rock in the middle. And more again now that the factory on a rock is solid to enemy ships while the rock isn't (Henry noticed, and accepts it for now).
- **Carriers can print carriers.** A built design with factory pixels can have itself on its build order, which compounds factories without the king or a rock. It costs pixels, unlike Toy 1's free outposts, but it sidesteps "only the king prints seeds".
- **Printing outruns mining.** From about four minutes in, both banks sit at zero: the factories could print about 50 px a second between them and the miners bring in 10 to 20. With turns, everything prints slowly at once, so half-printed ships sit beside their factories for minutes (12 for over two minutes in four matches).
- **Equal-sized enemies can overlap,** two kings included. Only a bigger enemy, or a planted one, is solid. They can't shoot each other from inside any more.

**Later rounds, already agreed in principle:**
- **Hunt sub-behaviors** (Henry, kept from the proving ground's parking lot): orbit, kite and attack runs at least, picked per design. Each one has to work for any ship drawn, not be tuned per design.
- **Switching a design's orders mid-match** (now in Match round 2, below; Henry, "quite possibly"): flip your designs between behaviors, so all your ships of one or more types can sit defensive for a while, then go together on a timing attack. That's strategy, not micro: one click moves a whole type. It changes orders, not the drawing, so it doesn't break "no editing designs mid-match".
- **Node pixels:** a part that extends link range, so relay ships are designs too and the network runs through ship design.
- **Designing your own king.**
- **A pause switch per factory**, if a factory building the wrong thing turns out to be a real problem.
- **Shields,** and more besides. After the first playtest (2026-11-10) Henry expects "big rewards for adding more variety in weapons, defenses, even special abilities, etc. in the future".
- **Smarter pixel targeting.**
- **PvP.**
- **AI levels** (Henry, 2026-11-10): easy, medium and hard at least; "hopefully without practicing I shouldn't be able to beat the hard AI. Even medium should be a challenge"; and "a superhuman AI eventually if possible".
- **A public database of ship designs that players share, and plenty of preset designs** (Henry, 2026-11-10, "eventually").
- **Wins by timing attack** (Henry, 2026-11-10): the escalating late game stays for now, watched, but "it should also be possible to win with well-executed timing attacks in the early-middle to middle game".

---

## Match round 2 (built and played 2026-11-10)

**What the first playtest said.** Three matches against the AI with the starter deck, all won. Henry is "pretty happy" overall, noticed no bugs, and found the play "still pretty smooth". The rest:
- **Too fast.** "It's much too easy to gobble up the whole map and I think ships are currently coming out with such frequency that it will feel spammy to the detriment of tactics. It should be a big deal when you get your first frigate, build a new seed, get a capital."
- **The AI is too weak, and that spoils the data.** "Adding better AI will be key to get better playtest data from me."
- **Always-attack ships make it spammier.** "I think the realtime swappable AIs for individual ship types will be an important thing to test. I don't want it to become something you have to swap constantly, but I think the always-attack AI makes it more spammy naturally, rather than having brilliant tactical strokes."
- **The view.** "We need a way to detach the camera from the king." The tactical map and right-click-to-repeat-build would be welcome back from Toy 1.

**What the log shows.**

| Match | Length | Henry | The AI |
|---|---|---|---|
| 1 | 5:27 | 10 factories by 3:00; 657 ships printed; 15,864 px mined | 4 factories; 163 ships; 2,569 px mined |
| 2 | 4:02 | 3 factories; 113 ships printed and 106 summoned by the king | 3 factories; 89 ships |
| 3 | 2:22 | 10 factories in the first minute, 21 by 2:00; 394 ships | 2 factories; 41 ships |

A seed costs 20 px whatever it grows into and the starting bank is 300, so the opening buys 15 factories. The AI plants one every 75 seconds by the clock. This is Toy 1's outpost spam again. "Where it could go wrong" below named it first ("Free land again"), and the throttle the plan relied on, the king's pace, was no throttle: the king prints a seed in 3 seconds.

**Three trials of slower numbers** (AI against AI, starter deck, throwaway builds):

| Trial | Ships a side at 2:00 | At 4:00 | Printed a side per match | Matches |
|---|---|---|---|---|
| As played | about 30 | about 70 | 160 to 430 | 8:44 to 15:22 |
| Seeds cost their size; mining halved; factories print at a third | 8 | 9 to 13 | 20 to 105 | 6:35 to 12:18 |
| The same, factories at half | 8 | 10 to 12 | 40 to 175 | 9:32 to 17:58 |
| Seeds cost their size; mining and factories at a quarter | 3 to 5 | 6 | 6 to 37 | one of four deadlocked for 40 minutes |

What they say: the numbers can make every ship count, but slower isn't longer. With fewer ships the first lost fight decided the match, and the loser sat at a bank of 0. At a quarter speed one match stopped dead: both sides lost their miners with empty banks, and nothing could ever be built again. So the round needs a floor under the economy and has to be tuned with the new AI, not before it.

**Goals:**
1. Milestones are a big deal: the first frigate, every new seed, a capital.
2. The opponent builds, so winning or losing says something about the decks.
3. Fights come as strokes you choose, not a stream, with no unit control and no constant switching.
4. You can see and run the whole map.

| # | Addition | What to look for | Verdict |
|---|---|---|---|
| M2.1 | **A seed costs what it grows into, paid in full:** 121 px for the outpost, 513 for the shipyard (20 for either in round 1). The bank has to hold the price when you click the rock. It's taken at once and the seed sets off by hyperspace, without waiting in the king's queue. A seeded card you can't pay for is greyed, with its price in red. The rock is still eaten and still limits the size. | Is every new factory a decision? Can the map still be gobbled? Is losing a seed in flight a fair swing? | **No word.** The log: 23 factories in 19 minutes (20 outposts, 3 shipyards), the second at 4:38. Round 1's third match had 21 in two minutes. |
| M2.2 | **A slower economy:** each factory pixel prints a pixel every 26.7 s (4 s before), a miner bites a rock pixel every 2.7 s (0.2 s before), and the starting bank is 190 px (300 before). Measured with the starter deck, AI against AI: the first outpost stands at 0:24 and the first frigate flies at 2:14 (planned: 0:45 and 2:30); a side has 7 ships at 2:00 and 17 at 4:00 (planned: under 10 at 2:00); the second factory is planted at 2:53 (planned: 4:00); the first shipyard stands at about 10:00 (planned: planted at 7:00); the first capital flies between 12:50 and 15:25 (planned: 10:00 to 12:00); matches run 18 to 24 minutes (planned: 15 to 20). | Is the first frigate a big deal? A capital? Is there still enough happening in the first three minutes? | **Keep.** "The slower pace too. Was way too spammy before." The late game: "getting spammy late-game is more like an exciting escalation, we might want to keep it just like that, although we should also keep an eye on it. Every game shouldn't be like that". |
| M2.3 | **The king prints ships no faster than an outpost:** 0.6 px/s (6 px/s before), with no size limit. A fighter takes 12 s, a frigate 73 s. | Is the king the lever for the one ship you need now, and no longer a second, better factory? | **No word.** The log: the king made 73 of his 436 ships. In round 1's second match it made 106 of 219. |
| M2.4 | **A floor under the economy:** the king makes a pixel every 4 s by itself. | Can you come back from losing every miner? Is it too little to matter otherwise? | **No word.** |
| M2.5 | **Standing orders you can switch,** per design: every ship of that design, flying or still to be printed, takes the new order at once. **HUNT** goes after the nearest enemy anywhere. **GUARD** holds where the ship was built and fights what comes within 160 px. **ESCORT** forms up in rings round your king, goes where you fly it and fights what comes within 110 px of it. A design starts the match on the order it was drawn with; the starter deck's warships are drawn on GUARD. Each card shows its order as a tag (HNT, GRD, ESC, and HRV for a miner): click the tag, or Shift+1–7. | Do attacks become strokes: massing, then releasing or leading? Do you switch a few times a match, not all the time? Does ESCORT make the king a flagship, or an unkillable blob? | **Keep.** "Switching orders felt good, although I'm pretty sure it's what let me beat the AI, because I don't think the AI used them effectively." The log: six bursts of switching in 19 minutes. |
| M2.6 | **An AI that plays the economy,** by the same commands as you. It plants when its bank holds the price (two outposts, then shipyards, up to six factories, each where it links to its king's network; and more outposts for as long as you have more factories than it has); keeps one more miner than it has factories; masses its warships on GUARD and sends them out as a wave when it's the stronger side or has rested two minutes, the slow designs first so they arrive together; calls them home when the wave is spent; turns everything on a raid near its king or factories; has its king print the counter to your fleet; and flies its king over to nodes that have been cut off. It plays the starter deck. | Does it build as fast as you do? Do its waves hurt? Do you lose sometimes? | **Keep, and make it stronger.** "I'm loving the changes so far. Mostly the better AI." But: "I don't think the AI can beat me yet." |
| M2.7 | **The view, said on screen.** It already worked the way Henry asked for: hold the mouse button on the small map to look elsewhere, and steering brings the view back. Nothing said so. Now a line above the small map reads HOLD TO LOOK, or STEER TO RETURN while the view is away, and the rules panel has a VIEW entry. | Do you find it without being told? Can you watch a fight or a rock while flying somewhere else? | **No word.** |
| M2.8 | **The whole map** (Tab, or the MAP button): the map at quarter size in the field. A ship is a block as big as it is, factories are outlined, links and seed courses are dotted. The king still flies, you can plant on it and give orders, and a click or drag moves the view as on the small map. | Is it where you make decisions, as in Toy 1? | **No word.** Opened five times. |
| M2.9 | **Repeat builds:** right-click a card and your king keeps printing that ship (blue frame) until you right-click it again. Backspace, or the X beside the progress bar, drops what the king is printing and gives back what it had paid. | Is it the convenience it was in Toy 1 without being a hose, now the king is slow? Is cancelling needed? | **No word.** Used four times, for miners and lancers. |

**Henry's rulings on the plan (2026-11-10):**
1. **The seed's price is its size, paid in full up front:** yes.
2. **The timeline:** "yes although it implies that capitals cost substantially more than factories. Maybe building an early capital from your king is just wasteful but it's possible earlier than 10min. Probably builds focusing on getting a capital out early can do it faster than 10min. But also there may need to be some other sorts of cost adjustments eventually."
3. **ESCORT:** "include it".
4. **Starter warships start on GUARD:** yes. "GUARD just means guard where you were built whereas ESCORT means guard the king, right?" (Yes.)
5. **The king's trickle:** "1 px/s". (Built slower: see the first judgement call.)
6. **A fair AI on the starter deck:** yes.
7. **The camera:** "I didn't realize there was a way to move the camera. Maybe make it like you can hold mouse button down on the minimap to move the camera, and then snap back whenever you steer." That's what it did already, so the arrow keys and FOLLOW button were dropped and the screen now says so.

**Judgement calls while building** (say if any are wrong):
- **The king makes 1 px every 4 s, not 1 px/s.** At the agreed timeline an outpost spends 0.75 px/s and a miner earns about 0.3. At 1 px/s the king would out-earn three miners and pay for an outpost every two minutes by itself, and raiding miners would hardly matter. A quarter of that is a miner's worth every 36 s: a way back from nothing, and little else. It's one constant (`KING_MAKES_EVERY`).
- **An early capital comes from a shipyard, not the king.** At 0.6 px/s the king takes 18 minutes over a capital. To make one possible from the king before 10:00, the king would have to print twice as fast as an outpost, which is the second, better factory M2.3 removes. The quickest capital in the runs came at 12:50, from a shipyard planted at 8:05.
- **Ships made in a match came out above the 60 to 100 planned:** the AI made 116 to 196 a match against the scripted openings and about 180 a side against itself, miners included. The first eight minutes are on the plan; the late game is not. Income keeps climbing because every outpost round adds two miners, and nothing yet makes miners worth less as they multiply.
- **In the arena, a design drawn on GUARD or ESCORT hunts.** A design's drawn order is now the order it starts a match on, so the starter deck's warships are drawn on GUARD, and the arena's ready-made fights would otherwise be two sides staring at each other. Round 1's arena Guard (hold a spot, fight within 100 px) is gone. The alternative is to keep it and add a separate setting for the match's starting order.
- **An escort trails a king at full speed.** The king flies at 45 px/s (64 on a diagonal), as fast as a fighter and four times as fast as a frigate. Escorts aim a second ahead of it, fall behind while it sprints and form up within a few seconds of it slowing. So a king that wants its escort has to fly like a flagship. Whether that's texture or a chore is for the playtest.
- **A miner can be switched too:** its tag steps HARVEST, HUNT, GUARD, ESCORT. ESCORT calls your miners in to the king.
- **Seeds skip the king's queue.** With the king this slow, a seed waiting behind a frigate would arrive a minute late.
- **Cancelling a print** wasn't in the plan. A wrong click on the capital would otherwise block the king for 18 minutes.
- **The AI's waves leave in order of speed.** Without that, its fighters reached you a minute before its frigates and died alone.

**How the AI was judged.** Four scripted openings stand in for the ways a player can try to out-build it. Each plays the starter deck by the same commands, on both sides of the map (`--judge 5`: five seeds, ten matches each).

| Against | Won | Lost | Drew | Matches ran | Ships the AI made | Ships the other made |
|---|---|---|---|---|---|---|
| Round 1's opponent: one more factory every 75 s by the clock | 10 | 0 | 0 | 9:37 to 19:50, median 15:14 | 116 | 56 |
| Plants an outpost every time the bank holds 121 px | 10 | 0 | 0 | 9:44 to 21:25, median 16:18 | 196 | 107 |
| The AI's own economy, with every ship on HUNT from the start | 10 | 0 | 0 | 12:31 to 23:14, median 19:30 | 166 | 114 |
| One outpost, and a king that prints warships without a pause | 9 | 1 | 0 | 12:20 to 20:42, median 17:57 | 139 | 74 |
| Itself | 5 | 5 | 0 | 17:56 to 21:28, median 18:57 | 178 | 178 |

The bar set in the plan was nine in ten against round 1's opponent and holding its own against the rest. It won all ten against round 1's opponent, all ten against the planter and the expander, and nine of ten against the summoner. One step on the way matters. Before the AI was taught to match the other side's factory count, it only broke even with the planter (five each), which out-built it almost two to one. So planting an outpost every time you can pay for one is still a strong way to play; the AI now answers it by doing the same. What this can't show is how it does against Henry: every script is simpler than a person, and none of them raids miners on purpose or leads an escort.

**Three bugs the slower pace uncovered,** all fixed by the rule and not by a patch, each with a test that fails without the fix:
- **Miners stopped mining.** The rule that keeps friends a pixel or two apart pushed a miner off any rock pixel under its own factory, and off any under a ship parked on GUARD. At round 1's speed a miner got its bite in on the way past; at 2.7 s a bite it never did, and mining stopped at 72 px a side by 4:00. Now a miner goes to its rock pixel whoever is parked over it.
- **One factory took every pixel.** Printers sharing a short bank take turns, and who went first moved round with the clock. The king's own pixel arrives on a steady beat, so it always found the same factory first in line: of two yards fed only by the king, one printed 21 fighters and the other nothing. Who goes first now moves round with every pixel handed out.
- **The AI stranded its own bank.** It planted outposts just out of link range, miners hauled to them, and thousands of pixels sat where its king couldn't spend them while its shipyard starved. It now picks rocks where a factory will link, measured from what's planted and not from wherever its king happens to be, and its king flies over to link whatever does get cut off.

**Checks.** 109 tests pass, one of which drives the match screen through its own keys and buttons (order tags, Shift+number, repeat, cancel, the whole map, planting from it, the view coming back). Nobody has played it in the window yet. 22 rules broken on purpose, one at a time (the 14 from the hardening and 8 for this round's rules), each fail the tests; a 23rd showed a rule nothing depended on, and that rule was taken out. The proving ground's reference results are the same in 25 of 26 matchups. The one with miners in it moved (4 fighters against 4 gunminers, 9–3 to 11–1), because mining is slower.

**First word from the playtest (2026-11-10, while still playing; not yet the verdicts):** "I'm loving the changes so far. Mostly the better AI. And the slower pace too. Was way too spammy before."

**During the playtest (2026-11-10): "are these miners mining?"** Henry sent a picture of his miners sitting on a rock beside his outpost, then: "maybe they're working fine, they're just more static than before, intentionally I assume". They were working: his log had 60 px mined by 2:00, and a replay of his opening showed every miner's load climbing. But the stillness wasn't designed. It's what a 2.7 s bite looks like: a miner now sits at its rock for half a minute, where in round 1 it was gone in two seconds. Nothing on screen told a working miner from a stuck one, and this round did have a bug where they were stuck. Two things now show the work, both drawing only:
- **Each bite sparks** on the rock pixel that was taken.
- **Scoops fill:** one scoop turns the colour of loose pixels for each scoopful aboard, so a 4-scoop miner goes pale a scoop at a time and flies home full.

Not changed, for Henry to rule on: miners after the same rock pixel now sit on top of one another, because a miner at work no longer keeps apart from its friends (that rule was what pushed them off the rock). Giving each miner its own patch of rock would spread them along the rock's edge. It would also limit how many can work one rock at a time, which is an economy rule and may be the brake the late game needs.

**Henry's playtest (2026-11-10): one game of 19 minutes, starter deck, won.**
- **Overall:** "I played one 19 minute game and it was pretty awesome."
- **The late game:** "Getting spammy late-game is more like an exciting escalation, we might want to keep it just like that, although we should also keep an eye on it. Every game shouldn't be like that - it should also be possible to win with well-executed timing attacks in the early-middle to middle game."
- **Orders:** "Switching orders felt good, although I'm pretty sure it's what let me beat the AI, because I don't think the AI used them effectively."
- **The AI:** "I don't think the AI can beat me yet. Ideally we should have at least easy-medium-hard AIs in the end and hopefully without practicing I shouldn't be able to beat the hard AI. Even medium should be a challenge. I'd also like to add a superhuman AI eventually if possible."
- **Ships to come:** "I'm really excited to see how wider ship variety works. We should definitely have a public database of ship designs that players share, and we should provide plenty of preset designs, eventually."
- **The yard:** "I haven't designed any ships yet, so I'm not sure if that system can be broken. Quite possible it can."

**What the log shows** (seed 7425):

| When | Henry | The AI |
|---|---|---|
| 6:00 | 4 factories, 22 ships, 921 px mined | 4 factories, 21 ships, 732 px mined |
| 12:00 | 9 factories, 62 ships, 4,300 px mined | 9 factories, 58 ships, 3,384 px mined |
| 15:00 | 14 factories, 80 ships, 7,198 px mined | 9 factories, 37 ships, 4,761 px mined, bank 0 |
| Whole game | 23 factories planted, 436 ships made, 11,593 px mined, 328 ships lost | 12 factories planted, 250 ships made, 5,028 px mined, 280 ships lost |

- **Level for twelve minutes, then the economy ran away.** Up to 12:00 the two sides had the same factories and the same fleets. From 13:00 Henry planted two shipyards and seven outposts in under two minutes; the AI's bank sat at 0 from 14:00 and it never planted again. Its rule for keeping pace (match the other side's factory count) does nothing once it can't pay.
- **Orders, six times in 19 minutes:** lancers to ESCORT at 1:36; frigates and fighters to ESCORT at 6:08; everything to HUNT at 10:03 and back to GUARD 47 seconds later; everything to ESCORT at 11:43; everything to HUNT at 16:36, which ended the game at 19:01. That's the "few times a match" M2.5 hoped for, and it's massing, probing and releasing, not fiddling.
- **Pixels cut off from his king:** 599 of 1,102 at 13:00, and 522 at the end. Whether he noticed the +N beside BANK isn't known.

**For the next round, to talk through first** (my reading, not rulings):
1. **Timing attacks have to be able to win.** Henry wants a well-executed attack in the early-middle or middle game to decide some matches. This one was decided by economy at minute 13 and ended at 19. Before changing anything, measure it: can a side that commits everything at 5 to 8 minutes kill a king, or even a factory line, against an opponent of the same size?
2. **The AI and orders.** It masses on GUARD and sends waves on HUNT, but it never escorts, never recalls a wave to defend, and attacks on a clock and a head-count, not at a weakness. Henry's six switches each had a purpose.
3. **The AI's economy late on.** It needs to see the run-away coming (Henry's income was half as much again by 13:00) and answer with raids on miners or an attack, not with outposts it can't afford.
4. **Levels.** Easy, medium and hard at least, with hard beyond an unpractised Henry and medium a challenge; a superhuman one eventually. Today's AI is the first rung.

**Where round 2 could still go wrong:**
- **Slower but not longer.** If the first lost fight still decides it, the match is a coin flip settled at minute 6. The floor income is one answer. Factories that survive a raid are another: today 12 lancers kill an outpost in 7 seconds, and that outpost is about to cost 121 px.
- **Two sides on GUARD is no game.** The AI has to attack on its own.
- **The ESCORT blob.** A king inside its whole fleet may be unkillable, or may be exactly the bait a sharper player wants. Slow ships fall behind a king at full speed, so a mixed fleet has to be led at its slowest ship's pace: that may be texture or a chore.
- **A fair AI may still be too easy for the person who designed the game.** If it is, the next step is a handicap dial (extra income for the AI), kept out of this round so the first read is honest.
- **The late game still floods.** Miners multiply and nothing makes the tenth worth less than the first. If matches past ten minutes feel like round 1 again, the fix is in the rules (rock that runs out sooner, or miners that get in each other's way), not in slower numbers.
- **Too slow now.** Seven ships at 2:00 may be the big deal Henry asked for, or two minutes of waiting. The 2X and 4X buttons are the stopgap.
- **Cut-off pixels.** With pixels scarce, what a cut-off outpost holds matters, and the only sign is a +N beside BANK. It caught the AI out; it may catch Henry out.

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
