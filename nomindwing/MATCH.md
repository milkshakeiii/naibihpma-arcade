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
- **Its queue shows under the cards** (M2.17, 2026-11-11): each job as its card's icon, the one being printed first. A job can be dropped or set repeating there.
- **It's a network node:** it stores pixels and links to your other nodes. Flying it away can split your own network.
- **A beaten king is slow** (M2.10, added 2026-11-10). After the first minute, a king whose side has under 40 px of ships and factories standing flies at a sixth of its speed, slower than anything that flies.
- **For now, the king is a fixed design.** Designing your own king "could be quite fun" (Henry), so it's a feature for later.

### Seeds and growing factories
- **A seed is a fixed little kit:** a core and an engine, unarmed. It costs a set price and flies at fighter speed. Killing seeds in flight is the cheapest counterplay.
- **On the rock, it grows its design outward from the core.** Each new pixel eats one rock pixel. The pixel nearest the core grows first, so the factory is one connected, legal ship at every step.
- **Guns work while it grows.** They fire as soon as they appear.
- **Its factory pixels grow it first, then print.** While it grows, a seed eats at a base rate and every factory pixel grown so far adds its printing to the growth. A design with factory pixels near its core snowballs its own growth. Once it's whole (or the rock runs out), the factory pixels start on the build order. (Built this way so a factory pixel never does two jobs at once; see the judgement calls below.)
- **If the rock runs out first, the factory stays unfinished:** fewer factory pixels, so it's smaller and slower.
- **No claims, no ownership.** A rock is just pixels: anyone can mine it, and anyone's seed can eat it. Two seeds on one rock share it, and each grows from what it gets. The planted factory is the claim, and taking it back means destroying it.
- **A seed comes down on the pixel it was sent to** (M2.15, 2026-11-11), or the nearest free one within 10 px if that's gone. While you pick a rock, the match draws what the factory would link to there, so where it will stand has to be exact.
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
- **A miner at work keeps apart from nobody** (M2.21). Other ships of a side keep a pixel or two apart. A miner going to its rock pixel, working it, or taking a full load home doesn't, and nothing keeps apart from it: miners heap up on a rock face, fly through each other and over a parked fleet, and other ships fly over them.
- **Loose pixels are anyone's:** from shots, kills and spilled stock. Once the rocks are gone, the economy runs on wreckage.

### Networks and split pools
- **Nodes are the king and anything with factory pixels** (factories and carriers). In round 1, every node has a fixed link range. Two of your nodes link when their ranges overlap, and links are drawn as dotted lines in your colour.
- **Pixels are stored at nodes.** A network's bank is everything its nodes hold. A delivery goes into the node it reached, and printing draws from the nearest nodes in the network first.
- **A split is physical:** cut a link and each half keeps the pixels sitting in its nodes. Reconnect and they pool again. You're told when a network breaks in two: two nodes parting, or your king cut off by a node you lost. Your king flying out of reach by itself breaks nothing and says nothing (M2.16, 2026-11-11). Hovering a factory shows its network's bank.
- **A destroyed node spills its stock** as loose pixels anyone can scoop.

### Ships
- **Everything printed flies itself by its design's order** (Hunt, Guard, Harvest, Hold), with the proving ground's three-layer brain. There's no unit control, so there's nothing to click-spam. The clicks are strategy: what the king prints, where seeds go, which factories run.

### The AI
- **A fixed deck and build orders.** It starts like you, with a king and a bank. It seeds rocks nearest-first, mines, and its factories send warships on Hunt.
- **It must expand and attack on a schedule.** Toy 1's passive AI was the pushover that made outpost spam look like a strategy.

### Multiplayer (a spike is built: N1, below)
- **PvP is a core part of the vision.** The sim is whole numbers only and deterministic: the same inputs always replay the same match. That's what lockstep multiplayer needs (each machine runs the same sim from the same inputs). The match must keep that: no floating point, no frame-time effects, and all randomness from the seeded generator.

---

## The numbers (round 2's, as slowed on 2026-11-11; round 1's in brackets)

| What | Number | Why |
|---|---|---|
| Map | 1400 x 1050, with 22 pairs of small rocks (1200 x 900 with 16 until 2026-11-11) | Toy 1 was 960x720; slower pace, bigger ships. Rocks as thick on the ground as before |
| Starting bank | 190 px at the king (300) | An outpost and two miners, with 51 px left to print with |
| King's print rate | 0.6 px/s, as if it had 16 factory pixels (6 px/s) | A fighter in 15 s, a frigate in 92 s, a 687 px capital in 19 minutes |
| King's own pixels | 1 every 4 s (none) | A miner's worth every 36 s: enough to restart, too little to live on |
| King's speed | 37.5 px/s (45 until 2026-11-11) | It keeps pace with a fighter; it used to match the fastest ship a drawing can make |
| Hyperspace arrival | 2 s | Long enough to see, short enough not to matter |
| Seed | What it grows into, paid at once: 121 px outpost, 693 px shipyard (20 px) | Every factory is a purchase |
| Seed's base eating rate | 4 px/s, plus the factory's own print rate (6 until 2026-11-11; round 1's 4 px/s) | An outpost grows in about 27 s, a shipyard in about two minutes |
| Factory print rate | 1 ship pixel per factory pixel every 26.7 s (4 s) | The outpost's 28 factory pixels print 1.05 px/s, the shipyard's 118 print 4.4 px/s |
| Factory size limit | 10 px per factory pixel | Its biggest ship takes 4½ minutes (40 s) |
| Factory pixel | weight 1, uses 2 power (1 until 2026-11-11) | Two hull pixels to feed each one: a factory the same size prints two thirds as fast |
| Lance | weight 2, uses 3 power (2 until 2026-11-11) | A two-lance lancer is 9 px, not 7 |
| Scoop | weight 2, uses 1 power (weight 1 and free until 2026-11-11) | The starter miner is 9 px at 30 px/s, where it was 8 px at 45 |
| Link range | 80 px per node (king 120) | Links span about one rock gap |
| Scoop mining | A pixel every 3.3 s with two or more scoops, 6.7 s with one (2.7 s and 5.3 s until 2026-11-11; round 1's 0.2 s and 0.4 s) | A 3-scoop miner fills its 9 px in 30 s. More scoops carry more and bite no faster |
| Beaten king | Its side under 40 px standing, after the first minute: 7.5 px/s, from 37.5 (no such rule) | The slowest ship a drawing can make flies at 11 px/s, so anything with an engine catches it |
| Guard and escort | Fights within 160 px of where it was built; within 110 px of its king | Guard outreaches every gun, so nothing shells a factory unanswered |
| Posts | First ring 16 px outside the builder; rings and posts about 16 px apart; a summoned ship joins rings within 24 px; giving way, it looks up to 64 px from its post for the nearest clear place (none: every guard flew to the one spot it was printed on) | Eight guards stand all round an outpost before a second ring starts, clear of what it prints next |
| Toy 1, for comparison | 15 px/s per factory; matches lasted 3½–6½ minutes | Too fast |

The cells that follow from the drawings (prices, print times, what the starting bank buys) are for the starter deck as Henry redrew it on 2026-11-10, with the five cards that M2.20 gave more hull. With round 2's own cards they read: 51 px left, a fighter in 12 s, a 513 px shipyard, 20 and 88 factory pixels (0.75 and 3.3 px/s), a 4-scoop miner.

---

## Match round 1: what gets built

A bare toy before wrappers: everything the loop needs, nothing it doesn't.

| # | Addition | What to look for | Verdict |
|---|---|---|---|
| M1.1 | **The board:** a mirrored map of rocks, your king flown with WASD, kill the king to win. The proving ground's sim at map scale, with a minimap. | Does it feel like Toy 1 (the feel you liked), just slower? | **Keep.** "Overall I'm pretty happy", no bugs noticed, "the gameplay is still pretty smooth". The camera has to come off the king, and Toy 1's tactical map would be welcome back. |
| M1.2 | **The deck in the yard:** built or seeded per design, the factory part, and build orders edited in the yard. | Is designing a factory and its build order fun in itself? | **Keep: on the right track** (Henry, 2026-11-10, after redrawing the deck). "Yes it seems to be fun/on the right track and should get even better once we start adding more features." |
| M1.3 | **Seeds grow factories** pixel by pixel out of their rock, their own factory pixels speeding the growth. | Is planting and watching a factory grow the awe? | **Rework the cost.** No word on the awe. "It's much too easy to gobble up the whole map": 10 factories in the first minute of the third match. |
| M1.4 | **Factories print** by the count rules, pay as they print, skip what's too big, wait when broke. Carriers work the same way. | Can you read what a factory will do next? | **Rework the pace.** Ships come out so often "that it will feel spammy to the detriment of tactics". |
| M1.5 | **Mining and the network:** scoop ships haul to the nearest node, linked nodes pool, splits divide by where the pixels sit, destroyed nodes spill. | Does where you build start to matter? Is a split ever legible and fair? | **Keep, with two fixes** (Henry, 2026-11-10). "It seems to be working." Wanted: a preview, when placing a seed, of which nodes it will connect to ("otherwise it's a not-fun guessing game sometimes"), and no split message "when just your king flies out of the network". |
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
| M2.1 | **A seed costs what it grows into, paid in full:** 121 px for the outpost, 513 for the shipyard (20 for either in round 1). The bank has to hold the price when you click the rock. It's taken at once and the seed sets off by hyperspace, without waiting in the king's queue. A seeded card you can't pay for is greyed, with its price in red. The rock is still eaten and still limits the size. | Is every new factory a decision? Can the map still be gobbled? Is losing a seed in flight a fair swing? | **Keep, and expansion gets dearer** (Henry, 2026-11-10). "i think every factory is a decision, but the game is currently a bit too fast again and making expansion a bit more expensive is probably on the cards among other things." |
| M2.2 | **A slower economy:** each factory pixel prints a pixel every 26.7 s (4 s before), a miner bites a rock pixel every 2.7 s (0.2 s before), and the starting bank is 190 px (300 before). Measured with the starter deck, AI against AI: the first outpost stands at 0:24 and the first frigate flies at 2:14 (planned: 0:45 and 2:30); a side has 7 ships at 2:00 and 17 at 4:00 (planned: under 10 at 2:00); the second factory is planted at 2:53 (planned: 4:00); the first shipyard stands at about 10:00 (planned: planted at 7:00); the first capital flies between 12:50 and 15:25 (planned: 10:00 to 12:00); matches run 18 to 24 minutes (planned: 15 to 20). | Is the first frigate a big deal? A capital? Is there still enough happening in the first three minutes? | **Keep.** "The slower pace too. Was way too spammy before." The late game: "getting spammy late-game is more like an exciting escalation, we might want to keep it just like that, although we should also keep an eye on it. Every game shouldn't be like that". |
| M2.3 | **The king prints ships no faster than an outpost:** 0.6 px/s (6 px/s before), with no size limit. A fighter takes 12 s, a frigate 73 s. | Is the king the lever for the one ship you need now, and no longer a second, better factory? | **Keep, and no slower** (Henry, 2026-11-10; said of the king's build speed before asking what M2.3 was). "the one thing I think we shouldn't slow down any more is the speed at which the king builds stuff- that already feels about right. maybe a little too slow if it's trying to build big things." |
| M2.4 | **A floor under the economy:** the king makes a pixel every 4 s by itself. | Can you come back from losing every miner? Is it too little to matter otherwise? | **Keep** (Henry, 2026-11-10). "I doubt you can come back from losing every miner against an equipped opponent but I don't see any need to change the king's pixel trickle currently." |
| M2.5 | **Standing orders you can switch,** per design: every ship of that design, flying or still to be printed, takes the new order at once. **HUNT** goes after the nearest enemy anywhere. **GUARD** holds where the ship was built and fights what comes within 160 px. **ESCORT** forms up in rings round your king, goes where you fly it and fights what comes within 110 px of it. A design starts the match on the order it was drawn with; the starter deck's warships are drawn on GUARD. Each card shows its order as a tag (HNT, GRD, ESC, and HRV for a miner): click the tag, or Shift+1–7. | Do attacks become strokes: massing, then releasing or leading? Do you switch a few times a match, not all the time? Does ESCORT make the king a flagship, or an unkillable blob? | **Keep.** "Switching orders felt good, although I'm pretty sure it's what let me beat the AI, because I don't think the AI used them effectively." The log: six bursts of switching in 19 minutes. |
| M2.6 | **An AI that plays the economy,** by the same commands as you. It plants when its bank holds the price (two outposts, then shipyards, up to six factories, each where it links to its king's network; and more outposts for as long as you have more factories than it has); keeps one more miner than it has factories; masses its warships on GUARD and sends them out as a wave when it's the stronger side or has rested two minutes, the slow designs first so they arrive together; calls them home when the wave is spent; turns everything on a raid near its king or factories; has its king print the counter to your fleet; and flies its king over to nodes that have been cut off. It plays the starter deck. | Does it build as fast as you do? Do its waves hurt? Do you lose sometimes? | **Keep, and make it stronger.** "I'm loving the changes so far. Mostly the better AI." But: "I don't think the AI can beat me yet." |
| M2.7 | **The view, said on screen.** It already worked the way Henry asked for: hold the mouse button on the small map to look elsewhere, and steering brings the view back. Nothing said so. Now a line above the small map reads HOLD TO LOOK, or STEER TO RETURN while the view is away, and the rules panel has a VIEW entry. | Do you find it without being told? Can you watch a fight or a rock while flying somewhere else? | **Keep** (Henry, 2026-11-10). "Working fine for now". |
| M2.8 | **The whole map** (Tab, or the MAP button): the map at quarter size in the field. A ship is a block as big as it is, factories are outlined, links and seed courses are dotted. The king still flies, you can plant on it and give orders, and a click or drag moves the view as on the small map. | Is it where you make decisions, as in Toy 1? | **Keep** (Henry, 2026-11-10). "Working fine for now". |
| M2.9 | **Repeat builds:** right-click a card and your king keeps printing that ship (blue frame) until you right-click it again. Backspace, or the X beside the progress bar, drops what the king is printing and gives back what it had paid. | Is it the convenience it was in Toy 1 without being a hose, now the king is slow? Is cancelling needed? | **Keep; show the queue better** (Henry, 2026-11-10). "Repeat builds seem fine, maybe we could tweak the indicators in the under-construction panel, something like an xinfinity sign or right click that zone to repeat/cancel repeat rather than the unit button. The visual indication of what exactly was queued up I think can be improved." |
| M2.10 | **A beaten king is slow** (Henry's idea, from the second playtest; built the same day). After the first minute, a king whose side has under 40 px of ships and factories standing, everything of its team but the king itself, flies at half a pixel a tick: 7.5 px/s, a sixth of its 45. Back to 40 px, back to full speed. The field says so for either king, a slowed king's engines go dark, the side panel shows SLOW beside that side's count, and hovering a king says how much its side has standing. | Does a won game end when it's won? Is a king ever slowed in a game that wasn't over? Does it read as a rule, or as the controls breaking? | **Keep** (Henry, 2026-11-10). The calls: "your calls look good." In play: "the beaten king has been fine". |
| M2.11 | **The starter deck is Henry's drawings** (2026-11-10). All seven cards redrawn so each spends the power its hull makes: a fighter with 4 flak, a lancer with 2 lances, a frigate with 6 flak and 4 lances, a miner with 5 scoops and 5 flak, an outpost with 42 factory pixels, a shipyard with 184 and guns on every side, a capital with 27 cannons. It's the deck the AI plays and the one a new save starts with. | Does a match on it show the rules, with no card you'd redraw on sight? Is an outpost that prints twice as fast the pace you want?  | **Keep for now** (Henry, 2026-11-10): "my deck the default for now is fine". |
| M2.12 | **Posts** (from Henry's playtest: guards "too grouped up", with a "weird wiggle"). A ship with nothing to do waits at a post of its own in the rings round what built it. It looks outward and sits still. It gives way only to a factory, a ship being printed, a king, or a waiting ship built before it; whatever flies past goes round it. A guard back from a fight returns to the post it had. | Do guards read as a formation? Is 16 px the right spread? Does anything still fidget with no enemy near?  | **Keep: much improved** (Henry, 2026-11-10): "guard posts were much improved". |
| M2.13 | **Tab in the yard** steps to the next part of the open category (Henry's request). Enter now swaps the yard and the arena, which Tab did. | Is that what you meant by Tab? Is Enter where you'd look for the swap? | **Keep** (Henry, 2026-11-10). The reading: "Yes, that's it." Enter: "enter in the yard is fine". |
| M2.14 | **A new miner,** picked by playing eighteen drawings against each other (Henry: "you can fix my miner then. just make sure it doesn't have any wasted pixels"). 8 px: three scoops, a flak turret on the nose and three engines round the core. 45 px/s, 4 of 4 power used, no hull. | Is the economy it gives the pace you want (a second factory by 4:00, matches about five minutes shorter)? Do raids on miners still matter?  | **Too fast again** (Henry, 2026-11-10, two games on it): "the game is currently a bit too fast again". The miner itself (2026-11-11): "New miner hull is fine for now". |
| M2.15 | **Planting shows its links** (Henry: "otherwise it's a not-fun guessing game sometimes"). While you pick a rock for a seed, a dotted line runs from it to every node the new factory would link to: green if that reaches your king's bank, gold if it only reaches nodes cut off from it. Seeds already on their way count as the factories they'll grow. With nothing in reach, a dim line runs to the nearest node and the bar says how many pixels too far it is. A seed now comes down on the very pixel it was sent to, so the picture is exact. | Does it end the guessing? Is "N px too far" the right help, or would you rather see each node's reach drawn? | **Keep for now** (Henry, 2026-11-11): "it's good for now". |
| M2.16 | **Your king flying off isn't a split** (Henry: no split message "when just your king flies out of the network"). The note comes when two nodes part, or when your king is cut off by a node you lost. | Is the note there when you want it, and only then? | **Fine so far** (Henry, 2026-11-11): "didn't bother me, yet". |
| M2.17 | **The king's queue, as icons** under the cards (Henry: "something like an xinfinity sign or right click that zone"; "the visual indication of what exactly was queued up I think can be improved"). The job being printed comes first, with a bar for how much of it is paid; a loop sign sits under a job that repeats, and on its card. Click a job to drop it; right-click it to repeat or stop; R repeats what's being printed. | Can you read the queue at a glance? Should the card's right-click go, now the queue takes it? | **Seemed good** (Henry, 2026-11-11, one game): "new build window seemed good". I read the build window as this queue. The card's right-click stays, and means something else: "I actually thought right-clicking the card should be the infinite-that-unit queue entry on the end of the queue." To build. |
| M2.18 | **A waiting ship's place is the nearest one clear of everything it gives way to.** A fault in M2.12, found while building M2.15: a guard pushed out from under one thing could come to rest inside the next. | Nothing new to judge: a guard should never sit inside a king or a ship being printed. | Nothing to judge. |
| M2.19 | **The reference fights are fought with the starter deck** (Henry, on redrawing the reference fleet: "probably yes"). `--pg-sim` now measures Henry's drawings: the nine arena tests made for them, and every pair of the four warships at equal cost. The proving ground's own fleet, with its spare hull, is behind `--pg-sim old`. | Is that the redraw you meant, or do you want a destroyer and a titan drawn without spare hull as well? | **Fine, probably** (Henry, 2026-11-11): "I'm not sure but I think it's fine. We will want more preset ship options soon but I think making more pixel/part variety is higher priority (and the ships that use those.)" |
| M2.20 | **Slower, by the rules** (Henry's list, 2026-11-11). A bite every 3.3 s; a map 1400 x 1050; a factory pixel uses 2 power, a lance 3, a scoop 1 and weighs 2; a seed eats 4 px/s; the king flies at 37.5 px/s. Five starter cards were given the hull the new costs need. | Is the pace nearer? Which lever should move further, and which back? Are the re-hulled cards fair stand-ins until you redraw them? | **Improved** (Henry, 2026-11-11, one game): "speed was improved but the AI was more vulnerable early due to the speed slowing. that's not a problem with the speed that's a problem with the AI." The stand-in cards: "Looked good to me". No lever to move. |
| M2.21 | **A miner at work keeps apart from nobody, and nobody from it** (Henry: "got a little wiggle happening here with this blue miner"). A full miner flies straight home, and miners at a rock push no one. Before, the miners still working pushed a full one back like any friend: with a few of them between it and its drop-off it hung there, a few pixels back and forth, sometimes for over a minute. | Is the wiggle gone? Do miners flying straight through each other, and over your guards, look right? | **Keep** (Henry, 2026-11-11): "you basically just made miners not collide? that seems fine to me, yeah." |
| M2.22 | **A right-clicked card puts a repeating job on the end of the queue** (Henry: "I actually thought right-clicking the card should be the infinite-that-unit queue entry on the end of the queue"). It's a new job, whatever of that design is queued already. A right-click on a job in the queue sets that one job repeating, and R the last job asked for. | Is that the entry you meant? Should a ship you ask for once go ahead of the repeating jobs, even ones that haven't had their first turn? | **Keep as it is** (Henry, 2026-11-11): "I was thinking that clicking the cards, right or left, would always putting something on the end of the queue. I knew that that would mean you could put stuff after a repeating job which doesn't make sense unless you cancel the job- but of course every repeating job is bound to be cancelled eventually. I don't want to fiddle with it too much more though it's fine how it is." So: no one-off jumps ahead; the card's second right-click, which stops the repeat and adds nothing, is the one place it differs from what they pictured, and it stays. |
| M2.23 | **The map's edge never moves what can't fly.** A fault the sim's own audit found while the spike was being built: a piece shot off a ship right beside the edge was shoved inwards on the next tick. | Nothing to judge. | |
| N1 | **The network spike** (Henry: "I think we're ready for the network spike"). A lag dial against the AI; HOST and JOIN on the match menu; two games playing one match in step by sending only commands; a hash of the whole match compared once a second. | With the dial at 2, 3 and 4 against the AI: does your king still feel right, and where does it stop? In two windows, does a whole match stay IN STEP? Does it on two machines? | **The dial** (Henry, 2026-11-11): "honestly it was playable up to 4 lag." In step over a whole match, in two windows and on two machines: not yet tried. |
| S1 | **How a design flies is chosen in the yard: RUNS, KITE or ORBIT.** Three buttons on the yard's flight row, or Y. Until one is picked the drawing picks, as before (1 thrust per weight runs, less kites). It's fixed for a match, and a card's hover line says it. | Is choosing it in the yard and not in a match right, in play? Do you miss a mark for it on the card itself? | **Keep** (Henry, 2026-11-11), on the yard, the card and the yard's layout: "all seems good for now." |
| S2 | **KITE.** What stand-off was, open to any ship, plus one thing: it backs away from anything else that comes nearer than its guns reach. | Does a light ship on KITE do what you'd call kiting? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S3 | **ORBIT.** Circles the target at the kite distance, flat out, facing it, and turns back the other way at the map's edge or a solid thing. | Does it read as an orbit? Is the circle the right size? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S4 | **SAB, an order beside HUNT, GUARD and ESCORT.** The nearest enemy miner or factory first, past whatever guards it; never a king; with none left it hunts. | Does it go for what you meant by sabotage? | **Keep** (Henry, 2026-11-11): "i saw groups and sabotage, yes all good." |
| S5 | **A ship keeps the enemy it's after** until that one is gone or another is 30 px nearer. Before, it picked the nearest again every tick. | Do fights look steadier, or do ships now ignore things they should turn on? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S6 | **Groups: GROUP 1, 3, 6 or 12** on the yard's flight row, or G. On HUNT or SAB that many of the design gather, within 96 px of one another, before any leaves, and they guard meanwhile. | Do groups leave when you expect? They gather by nearness, so each factory makes its own group: is that right, or should a design gather across the map? | **Keep** (Henry, 2026-11-11), with everything waiting together leaving together: "i saw groups and sabotage, yes all good." |
| D1 | **The AI sees a king coming.** Your king counts as a raid from the moment it crosses to the AI's half of the map, not from when it arrives. | Fly your king over at the start with lancers on repeat, as on seed 28100. Does the AI answer in time? | |
| D2 | **Guns before miners against a raid it can't match,** and nothing extra held back under a raid. Against a king with ships round it, it prints what answers the ships; against a king alone, lancers. | Does it spend its bank when you raid it, or sit on it as it did at 4:00 on seed 28100? | |
| D3 | **Its king stands beside its factory while a raid is on,** so the factory's flak and its own cover each other. It stays against what it outweighs. Outweighed, it falls back on a factory nobody is attacking. | Does the king's moving about make sense to watch? Is it too easy to catch out? | |
| D4 | **Its miners are its to defend.** A raider within 200 px of one of its miners turns its fleet out. | With everything on SAB, do its ships now come for yours at the rocks? | |
| D5 | **It arms while your fleet is a quarter bigger than its own.** Guns then come before what it's saving for a factory. | Does a fleet built early draw an answer before it lands? | |
| D6 | **RUSH and SAB,** two scripted openings taken from Henry's two matches of 2026-11-11, beside the five the AI was measured against. | Nothing to judge in play. Is RUSH a fair picture of what Henry did on seed 28100? | |

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

**During the playtest (2026-11-10): "are these miners mining?"** Henry sent a picture of their miners sitting on a rock beside their outpost, then: "maybe they're working fine, they're just more static than before, intentionally I assume". The miners were working: Henry's log had 60 px mined by 2:00, and a replay of that opening showed every miner's load climbing. But the stillness wasn't designed. It's what a 2.7 s bite looks like: a miner now sits at its rock for half a minute, where in round 1 it was gone in two seconds. Nothing on screen told a working miner from a stuck one, and this round did have a bug where they were stuck. Two things now show the work, both drawing only:
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
- **Pixels cut off from Henry's king:** 599 of 1,102 at 13:00, and 522 at the end. Whether Henry noticed the +N beside BANK isn't known.

**Henry's second playtest (2026-11-10): five more games, starter deck, four won and one lost.**
- **Timing attacks:** "Played a few games. I think the current timing attack situation is well within balance range."
- **A beaten king:** "I did feel like we should add a feature where the king slows down when all or most of its army and factories have been destroyed." And, while it was being built: "it's not only that the AI beaten kings ran a good bit, it's also that I think real players could run for significantly longer".

**What the log shows.**

| Seed | Result | Henry's factories | What happened |
|---|---|---|---|
| 39453 | Won at 10:53 | 7 | Lancers and fighters on ESCORT by 1:57, frigates sent on HUNT at 5:56. The AI had nothing standing at 9:00; its king lasted nearly two minutes more. |
| 39454 | Won at 9:07 | 8 | Everything on ESCORT or HUNT by 1:10. The AI had no factory at 8:00 and one ship at 9:00. |
| 39455 | Won at 10:21 | 5 | One outpost until 6:29. Escorts from 0:53, released on HUNT at 9:45; the AI had nothing at 10:00. |
| 39456 | **Lost at 11:10** | 1 | One outpost all game against the AI's four, and no order switched: everything stayed on GUARD. The king died with 670 px sitting in a network it had flown out of. The AI's first win against Henry, **and not a real one:** "btw on the loss: i was just AFK, so don't take that as the AI being strong" (Henry, 2026-11-10). |
| 39457 | Won at 6:13 | 1 | One outpost, escorts from 2:51, HUNT at 3:43. The AI had nothing at 5:00; its king lasted over a minute more. |

- **Attacks before the tenth minute won four games of five,** the earliest at 6:13 off a single outpost. That is the timing-attack win the first playtest asked for, and Henry's ruling is that it sits within balance range. Nothing changes there.
- **The chase.** In every win the AI's king outlived everything else it had: by at least 21 s, 67 s, 73 s and 113 s. The log has a line a minute, so each is the least it can have been.

**M2.10, the beaten king: why, and what was built.**
- **Why a king could run.** A king flies at 45 px/s, and 64 on a diagonal. The starter deck's fastest ships make 45 and no drawing can make more. So nothing catches a king that runs, and a side with nothing left to lose could run until the 40-minute draw. The AI's king only backs away from what comes near. A person would do it on purpose.
- **The rule** is in the table above. In the sim it is `KING_NEEDS` (40 px), `KING_SLOW` (2 quarter-pixels a tick) and `KING_GRACE` (the first minute), in `src/pg/sim/eco.rs`.
- **Measured:** the AI against each scripted opening and itself, 10 matches each, the same seeds three ways. The figures are how long a king lasted once its side was under 40 px for good: the median, and the longest in brackets.

| Against | Rule off | A third of full speed (15 px/s) | As built, a sixth (7.5 px/s) |
|---|---|---|---|
| Clock | 206 s (460) | 70 s (120) | 63 s (80) |
| Planter | 127 s (301) | 51 s (93) | 40 s (58) |
| Expander | 180 s (310) | 100 s (237) | 48 s (148) |
| Summoner | 66 s (600) | 57 s (187) | 49 s (123) |
| Itself | 69 s (95) | 54 s (110) | 34 s (52) |

  Who won didn't change in any column: 10-0, 10-0, 10-0, 9-1 and 5-5. The median match got shorter by 20 seconds to 2½ minutes (against Expander, 19:30 to 16:51). What's left of the wait is the winner's fleet getting there and getting through the king's 218 pixels.

**Judgement calls on M2.10** (mine; say if any are wrong):
- **"All or most" became "under 40 px standing":** about six fighters, or a third of an outpost. One factory standing keeps a king at full speed whatever else it has lost. The other reading is a share of the most the side has ever had, say under a quarter. That would also slow a king that still has three outposts and twenty ships late in a long game, which is a side that's behind, not one that's beaten. It would end won games sooner still, and it would reach into games that are still live, which makes it a balance lever. Not built.
- **A sixth, not a third.** A third (15 px/s) was built first. Frigates and capitals fly at 11 px/s, so a beaten king still outran half the starter deck's warships: in the AI's games they trailed it at 70 to 100 px and never closed. At half a pixel a tick it is slower than the slowest ship a drawing can make, on a diagonal as well.
- **Not in the first minute.** Both sides start with nothing standing. By 1:00 an outpost has been up for half a minute. A side that has built nothing by then is slowed, and the field says why.
- **Everything counts but the king:** miners, a seed in flight, a factory still growing, a ship half printed (the pixels it has so far). The bank doesn't count. A king makes a pixel every 4 s, so a runner could otherwise wait its way back to full speed.
- **The AI plays no differently.** It already sends everything when the other side has no fleet. Its own king, beaten, still backs away from what comes near, at a crawl.

**What M2.10 changes that wasn't asked for.** A side that loses its only factory and most of its ships is now very likely to lose its king as well; before, it could run and replant. With an outpost's price in the bank it is back at full speed in 10 to 20 seconds, because the seed counts and a growing outpost passes 40 px about 7 seconds after it lands. So 121 px held back is insurance, and an early attack that wipes a base is close to decisive. That bears on timing attacks, which Henry has just ruled are in range: worth watching in the next games.

**A bug the soak caught while building it.** A slowed king moves by the same quarter-pixel remainders as every other ship, and a piece shot off a king takes its remainders with it. Kept negative, they made a piece with no engine drift one pixel, and the audit's "has no thrust but moved" caught it at tick 15,702 of an AI match. They're now kept the way every other ship keeps them (0 to 3), and the break is on the list of rules broken on purpose.

**Checks after M2.10.** 111 tests pass. Two are new: the rule to the pixel and the tick (39 px is short, 40 is enough, the king's own pixels and the other side's don't count, the first minute is exempt), and the chase (a fighter and a frigate each catch a beaten king; a fighter can't catch a king with a fleet). 32 rules broken on purpose, one at a time (the 22 from the checkpoint and ten for this rule), and the tests catch every one. The arena's 26 reference matchups are exactly as they were at the checkpoint.

**Henry's reply to M2.10 (2026-11-10).**
- **The calls:** "your calls look good."
- **The lost game:** "btw on the loss: i was just AFK, so don't take that as the AI being strong".
- **The yard:** "minor yard feature request: tab tabs through pixels of the selected type."
- **Guards:** "regarding guard orders: i think they're too grouped up and they do this weird wiggle when there aren't any enemies around".
- **The deck:** "i've done some simple optimization on the default team. I don't think we should present people with ships that just have extra hull that's unused, so take a look. i'm pretty sure the optimized team should crush the unoptimized one most of the time. this will also be important going forward while we're testing. we don't want to take silly ship designs and make gameplay decisions when the silly designs are affecting gameplay!"

**M2.11, Henry's deck: what changed, card by card.**

| Card | Round 2's drawing | Henry's |
|---|---|---|
| FIGHTER | 7 px, 2 flak, 4 of 6 power used, 45 px/s | 9 px, 4 flak, 6 of 6, 37 px/s |
| LANCER | 7 px, 1 lance, 4 of 7, 45 px/s | 7 px, 2 lances, 6 of 6, 37 px/s |
| FRIGATE | 44 px, 4 flak and 1 lance, 9 of 15, 11 px/s | 55 px, 6 flak and 4 lances, 19 of 19, 15 px/s |
| MINER | 9 px, 4 scoops and 1 flak, 2 of 6, 22 px/s | 14 px, 5 scoops and 5 flak, 6 of 6, 15 px/s |
| OUTPOST | 121 px, 20 factory pixels, 24 of 68; a round of its build order in 110 s | 121 px, 42 factory pixels, 46 of 46; a round in 68 s |
| SHIPYARD | 513 px, 88 factory pixels, 8 flak, 96 of 352 | 693 px, 184 factory pixels, 22 flak and 12 lances, 230 of 230 |
| CAPITAL | 620 px, 4 cannons and 12 flak, 52 of 471, 11 px/s | 663 px, 27 cannons and 12 flak, 259 of 264, 11 px/s |

- **It crushes the old one, as Henry expected.** AI against AI, 20 seeds each way round: 40 wins to 0, in a median 10:38 and 12:15.
- **Against itself** the AI splits the games, as it did on the old deck. Twenty mirror matches ran from 6:50 to 27:17 with a median of 21:18; on round 2's cards they ran from 17:21 to 31:18 with a median of 20:44. The middle of a match is about as long as it was, and the short end is much shorter: a side can now be finished inside seven minutes.
- **It's now the starter deck** (`starter_deck()` in `src/pg/design.rs`, drawn at the same grid places as Henry's save, so that file and the built-in deck are the same lines). A save still holding an untouched round 2 card gets the new one when it loads.

**What the new deck changes about a match** (measured, AI against itself, the same seed on each deck; none of it is a fault, all of it is worth knowing before reading the next games):
- **The outpost spends the bank as it comes in.** It prints 2.1 times as fast for the same 121 px. At 2:00 a side has 9 ships and 19 px in the bank, where the old outpost had 7 ships and 117 px. In twenty mirror matches the second factory went down at 5:07 at the earliest and mostly between 5 and 8 minutes, where it used to at 2:52. Nobody can tell a factory to wait, and a seed is paid at once, so while there's income to print with there's never 121 px to plant with. A person is in the same squeeze as the AI. **Corrected in M2.14:** most of this was the miner, not the outpost. With a miner that pays its way the same outpost leaves room for a second factory by 4:00.
- **The miner costs more and hauls slower:** 14 px (9), 15 px/s (22), 15 px a load (12). A side has mined 270 px by 4:00 where it had 324, and 600 by 6:00 where it had 696. What that buys is five flak on every miner. **M2.14 measured what it cost:** the same deck with almost any cheaper miner beat it nearly every game.
- **Nothing in the deck catches a king at full speed** (fighters make 37 px/s, a king 45). That was a tie before. M2.10 covers the beaten king; a king with a side still standing can leave any fight.
- **The AI against the scripted openings,** the same 10 matches each. On round 2's cards it won 10, 10, 10 and 9 (Clock, Planter, Expander, Summoner). On Henry's, before anything else changed: 7, 8, 6 and 10. After M2.12 and the alarm change below: 10, 9, 8 and 10. Against itself it is still 5 wins each of 10, in a median 19:48 (18:37 on round 2's cards).
- **Not redrawn:** the king (still mostly spare hull: it is the match's health bar, and a fixed design for now) and the arena's reference fleet, which stays as the rounds judged it so `--pg-sim` still means what it meant. Say if either should follow.

**M2.12, posts: what was wrong, and what was built.**
- **Measured before.** An outpost's 11 guards all had the same home: the one spot beside the factory where it prints. They sat 1 to 3 px apart, and each moved on about two ticks in three (92 to 108 of 150), over 23 to 48 different pixels in ten seconds. Every guard flew for the same point, the rule that keeps friends from stacking pushed it off, and it flew back.
- **The rule.** A built ship's home is what built it: the middle of its factory, or of the king it arrived beside. With nothing to do it waits at a post of its own in the rings round that. Guards fill the rings in the order they were built; the first is 16 px outside the builder and they are 16 px apart. At its post a ship sits still and isn't pushed: whatever flies past goes round it. It gives way only to what can't move aside (a factory, a ship being printed, a king) and to a waiting ship built before it, by stopping at the nearest clear point, and goes back when the way is free.
- **Measured after.** The same outpost, five minutes in: 16 guards (it prints faster now), none of which moved in ten seconds, 10 to 18 px apart, the first eight all the way round the factory, each facing outward.

**Judgement calls on M2.12** (mine; say if any are wrong):
- **16 px.** Eight guards make a ring round an outpost and thirty fill three rings, 54 px out, all well inside the 160 px they guard. Wider is one number (`POST_GAP`, `POST_SPACE` in `src/pg/sim.rs`).
- **Home is the builder, not the spot printed on.** So the 160 px a guard covers is measured from the middle of its factory, about 10 px from where it was.
- **Anything waiting gets a post,** not just guards: a hunter with no enemy left, a miner with its scoops shot off or no rock to mine, a ship a factory built on HOLD. They come after the guards in the rings. They had the same one spot to fight over.
- **At its post a guard looks outward.** A lance only fires ahead, so the ring points its lances at what's coming.
- **Ships your king summons within 24 px of posts already kept join those rings,** so six fighters summoned while the king drifts a little make one ring, not six.
- **ESCORT is as it was.** Its rings move with the king. It wasn't measured for the same fidget when the king parks beside a factory; if it shows, the same rule fits.

**What M2.12 changes that wasn't asked for.**
- **The AI's alarm.** A raider within 200 px of anything the AI has planted turns its guards out on HUNT, and they went back on GUARD the moment it stepped outside. With posts in, a raider hovering at that edge had the AI's whole fleet turn out and home every 2 to 8 seconds: 156 switches in eight minutes against the Planter, where the test allows 80. The flicker was there before; the new deck and the spread-out guards made more of it. Now a fleet turned out stays out until every raider is 60 px further off, and that match is back under the limit. Only the fleet's orders follow the slower alarm: what the AI keeps back in the bank under attack, and what its king prints against a raid, still go by the 200 px line, so the change reaches no further than the flicker it's there for. This is how the AI plays, so it shows in its results above, and those can't be split between this and the posts.
- **Guards start a fight spread out,** not as one clump, so one cannon shell or one burst of flak catches fewer of them.

**M2.13, Tab in the yard: how the request was read.** "Tab tabs through pixels of the selected type" was taken as: Tab steps to the next part (pixel) of the open category (type), as pressing the category's number again already did, which nothing said unless you hovered the category. With one part in the category it says so. Tab used to swap the yard and the arena, so Enter does that now, from either; Tab still leaves the arena. The category's hover text, the F1 help and the two buttons' tips say so. If the request meant stepping through the pixels of that part on the ship itself, it isn't built. **Confirmed (Henry, 2026-11-10):** the parts of the open category is what was meant.

**Checks after M2.11 to M2.13.** 118 tests pass. Seven are new: guards wait in rings and sit still; a waiting guard gives way to a block and to the king, then goes back; a guard that leaves to fight comes back to the post it had, and one that stays doesn't move up into the empty posts; two sets of rings that cross give every guard a place, the older guard keeping its own; summoned ships share rings; Tab and Enter in the yard; and a slowed king flying up or left. Six older tests had the old drawings' numbers in them (a 7 px fighter, a frigate as the slowest ship, an 88-pixel shipyard, a second factory by 6:00) and now read them from the deck. 46 rules broken on purpose, one at a time (the 32 from M2.10 and fourteen for posts and the alarm), and the tests catch every one. The arena's 26 reference matchups are exactly as they were.

**Four breaks got past the tests on the first run of that list,** and each was a test leaning on luck:
- **The AI and the greedy planter.** The test was a ten-minute match, and on the new deck nobody out-plants anybody in ten minutes, so it passed with the rule switched off. It's now a test of the decision itself: two factories down and an enemy with three, the AI plants the outpost it can pay for; level on factories, it keeps saving. (Played on round 2's cards with this turn's changes, the old match has the AI two factories behind the planter at minute 9 on one of its two seeds, where it had stayed within one. Those cards are retired, so nothing was done about it.)
- **A slowed king's remainders** (M2.10's bug). It was only caught when a soak match happened to shoot a piece off a slowed king. The audit now checks every ship's quarter-pixel remainders every tick, and a test flies a slowed king up and left.
- **A guard giving way to one built after it,** and **a guard away fighting losing its post:** neither showed while every guard left and came back together. The tests now keep one gunless guard at home through the fight, and send two crossing sets of rings off to fight and back.

**Henry's reply to M2.11 to M2.13 (2026-11-10).**
- **Tab:** "Yes, that's it."
- **The miner:** "my miner is worse: oh well you can fix my miner then. just make sure it doesn't have any wasted pixels. you can even do a tiny bit of automated testing to find the best miner if you want."
- **The AI:** "explain a bit more about what the ai doing worse against the scripted openings means?"
- "i'll provide more feedback soon. let's just fix the miner first"

**M2.14, the miner: what the rules say, and what was tried.**
- **The rules.** A miner bites one rock pixel every 2.7 s once it has two scoops, and no faster with more: extra scoops only carry more (3 px each). Scoops use no power. A core makes 4 and each hull pixel 1; an engine and a flak turret each use 1. So a miner with no wasted pixels has a core, as many engines and flak as its power feeds, and scoops to taste. Speed comes from engines per pixel, up to 45 px/s.
- **Eighteen drawings,** all 3 pixels wide, every one legal and using exactly the power it makes (bar round 2's, kept in for comparison): 2 to 6 scoops, 1 to 4 engines, 0 to 5 flak, 7 to 14 px.
- **First, each against Henry's deck,** which differs only in the miner: AI against AI, 10 seeds each way round.

| Miner | Size | Scoops, engines, flak | Speed | Won of 20 | Second factory (median) |
|---|---|---|---|---|---|
| The one chosen | 8 px | 3, 3, 1 | 45 px/s | 20 | 3:39 |
| Five others | 7 to 11 px | 2 to 6 scoops, 2 to 4 engines, 0 to 2 flak | 45 px/s | 20 | 3:25 to 4:10 |
| Six more | 7 to 11 px | 2 to 6 scoops, 1 to 4 engines, 0 to 3 flak | 22 to 45 px/s | 18 or 19 | 3:27 to 4:35 |
| Round 2's miner (4 of its 6 power unused) | 9 px | 4, 1, 1 | 22 px/s | 18 | 4:10 |
| Unarmed: 2 scoops and 4 engines; 6 and 4 | 7, 11 px | no flak | 45 px/s | 17, 16 | 3:25, 4:16 |
| Henry's with a second engine | 14 px | 5, 2, 4 | 26 px/s | 14 | 5:05 |
| Henry's with 2 scoops | 11 px | 2, 1, 5 | 18 px/s | 13 | 5:28 |
| Henry's first miner, against itself | 14 px | 5, 1, 5 | 15 px/s | 10 | 6:20 |

- **Then the best eight against each other,** every pair 10 seeds each way round, 140 matches apiece:

| Miner (scoops, engines, flak) | Size | Won of 140 |
|---|---|---|
| 3, 3, 1 | 8 px | 86 |
| 2, 3, 1 | 7 px | 79 |
| 2, 2, 2 | 7 px | 77 |
| 3, 2, 2 | 8 px | 75 |
| 4, 3, 1 | 9 px | 72 |
| 2, 4, 0 | 7 px | 65 |
| 6, 3, 1 | 11 px | 54 |
| 3, 4, 0 | 8 px | 52 |

- **What it shows.** Small beats big: cargo past 9 px costs more to print than it saves in trips. Fast beats slow. One or two flak beats none. The top four are too close to call on this many games (61% to 54%, and they split their games with each other); the one that came first was taken.
- **The miner:** `.g.` / `sss` / `ece` / `.e.`, up being the front: a flak turret on the nose, three scoops, the core with an engine either side and one behind. It's in `starter_deck()`, and Henry's first miner is with the retired cards, so a save still holding it gets the new one when it loads. The old line, to have it back: `MINER|HARVEST|48,45|.g./ggg/sss/sgs/hch/.e.|BUILT|`.

**What M2.14 changes about a match** (AI against itself, 20 seeds, the same build but for the miner):
- **The economy runs about half as fast again.** At 4:00 a side has mined 441 px (270 with the first miner; 324 on round 2's cards), at 6:00 933 (600; 696), at 10:00 about 2,800 (1,500; 1,550).
- **The second factory goes down between 3:28 and 5:05,** mostly before 4:00 (5:07 to 9:31 before; 2:52 on round 2's cards). The first shipyard can stand at 10:00.
- **Matches are shorter and closer together:** 6:57 to 17:02 with a median of 15:53 (6:50 to 27:17, median 21:18). Round 2, which Henry called a better pace than round 1, ran a median 20:44 on its own cards. This is the thing to judge in play.
- **The AI against the scripted openings:** 9, 10, 9 and 10 of 10 (Clock, Planter, Expander, Summoner), 38 in all. It was 37 with the first miner and 39 on round 2's cards.
- **A finding of mine was wrong.** M2.11 put the late second factory down to the outpost spending the bank as it came in. Most of it was the miner: with this one the same outpost has 64 px banked at 2:00 and a second factory before 4:00. The question it raised for the next round (a lever for saving) is withdrawn below.

**What the AI's losses to the scripted openings were** (Henry asked what "doing worse" meant; these are the three it lost on Henry's deck with the first miner).
- **The openings are the yardstick for the AI's play.** Each is one simple plan played by the same commands on the same deck: a factory every 75 s by the clock; an outpost every time the bank holds one; the AI's own economy with every ship sent out as it's made; one factory and a king that never stops printing warships. Both sides hold the same cards, so who wins is down to decisions. An AI that plays well should beat each of them nearly every time.
- **39, then 37, then 38 of 40** is a difference of one or two games, which on its own is noise. What it's worth is in the lost games.
- **Planter, lost at 10:30.** The AI never planted a second factory. Under raid it holds 150 px back before it will plant, and its bank never passed 73 px. The Planter stood at three outposts and had mined 1,365 px to its 475.
- **Expander, lost at 11:39.** Two factories to three, 958 px mined to 2,130. The Expander's steady trickle of hunters kept killing its miners (14 made, to 33), while its own fleet was turned out and called home 68 times.
- **Expander, lost at 21:24.** The AI out-mined it and planted six outposts, and never a shipyard. The Expander had a shipyard at 12:48 and four capitals from 17:40. The AI lost 306 ships to 142.
- **So what it means:** nothing about the deck. The AI's saving habits (hold 150 px back when raided, save for a shipyard after two outposts) assume a bank that fills, and when it doesn't the AI sits on one or two factories. A person who raids its miners early should find the same weakness. It is AI work for the next round, not a deck fault.

**Checks after M2.14.** 120 tests pass. Two are new: loading a save that still holds a retired card beside one edited by hand; and a miner biting rock under a friend parked on it. All 46 rules broken on purpose are still caught. One needed a new test first: the faster economy cleared the old test's bar (200 px mined by 4:00) even with friends pushing miners off their rock, so that rule now has the direct test above. The arena's 26 reference matchups are exactly as they were.

**Henry's third playtest (2026-11-10): three games, all won, and a round of verdicts.** The verdicts are in the two tables above (M1.2, M1.5, M2.1, M2.3, M2.4, M2.7 to M2.9, M2.14). The beaten king in play (M2.10), the deck (M2.11), posts (M2.12) and Enter in the yard (M2.13) were ruled later the same evening: all four keep.

**What the log shows.**

| Seed | Deck | Result | What happened |
|---|---|---|---|
| 59113 | Henry's own, with the first miner | Won at 6:27 | One outpost. Escorts at 1:26, HUNT at 4:50. The AI's king was slowed at 5:14, free again at 5:51 with 58 px standing, slowed for good at 5:55. |
| 72667 | Starter, new miner | Won at 5:33 | The king on repeat miners from 0:04. Outposts at 0:02, 2:41 and 4:20. 999 px mined by 5:00. The AI's king was slowed at 4:55 and dead 38 s later. |
| 72668 | Starter, new miner | Won at 8:53 | The king on repeat miners all game. Ten outposts by 7:12 and a shipyard at 8:51. 5,059 px mined to the AI's 1,812; 133 ships at the end. The AI's king was slowed at 8:38 and dead 15 s later. |

- **The economy runs away from the AI.** By 4:00 Henry had mined 975 px in the last game; the AI against itself mines 441. Miners printed by the king from the first second, and an outpost every half minute from 3:00, is the map-gobbling of round 1 again, at a price.
- **The beaten king** ended both games on the new miner inside 40 seconds. In the first game the AI's king was free again for four seconds, at 58 px.

**Rulings on the open questions (Henry, 2026-11-10).**
- **The king and the reference fleet:** "The king is a special case and we're going to visit the possibility of manual king designs later. The reference fleet probably yes."
- **Bigger miners:** "We should definitely reward bigger miners at some point. I was thinking about maybe adding more possible types of miner parts/pixels. I don't think we want to scale scoops while they still cost 0 hull though."
- **Round 2's five departures from the plan** (the king's income and print speed among them): "I think those are probably all approved, I don't remember exactly what they were but I'm pretty sure they are fine."

**Where the game is going (Henry, 2026-11-10).**
- **Capture the simple game.** "I do think we should capture an early, simple form of the game like how it is now (but probably not literally how it is now) in case we want to make it into a pure, simple or training type game mode."
- **Buildsets.** "Adding more parts and part interactions will be a big determiner of how the game eventually plays, and it makes sense to provide different buildsets like there are different cardsets in MTG. It will be a question whether we want to allow asymmetrical cardset matches. Of course, I won't rule out at this point that we might want to just give the golden options and have that be the game too."
- **The next phase.** "I predict we'll soon be in a design phase where we're mostly trying out new pixel/part types and deciding if they make the matches and building more interesting or not. We'll need to decide though how much of the other parts we want to build before that. Better AI, the network layer, campaign features. I think the network layer might be worth derisking. What do you think?"
- **A theme,** offered as a side note the same day: the game as a galaxy-wide sport played over a 64-bit link, the player climbing to Wingleader. Kept as an idea; nothing built.

**Asked for in this playtest, and where each stands:**
1. When placing a seed, show which nodes it will link to. **Built: M2.15.**
2. No split message when it's only the king that has flown out of its network. **Built: M2.16.**
3. The king's build panel shows what is queued and what repeats, and takes the right-click that sets or clears a repeat. **Built: M2.17.**
4. The reference fleet redrawn with no spare hull, and its matchups recorded afresh. **Done another way: M2.19.** No ship was redrawn; the reference is Henry's deck.
5. The pace brought back down, with dearer expansion among the levers and the king's build speed left alone. **Measured, not changed:** the numbers and the options are below, for Henry to pick.

**M2.15 to M2.19: what was built (2026-11-11, after Henry's "Fixes and pace quickly first").**

*M2.15, the links while planting.*
- **What the match draws.** The rock under the pointer is asked where a factory would stand on it and what stands within reach of that: your nodes, and the places your seeds in flight will come down. A line to each. Green means the links lead to your king as things stand; gold means they don't.
- **Out of reach,** the nearest node is the one whose reach falls short by the least, and the bar gives that shortfall in pixels, rounded up: a rock "1 PX TOO FAR" links if the factory stands one pixel nearer.
- **A seed lands where it was sent.** It used to come down on the rock pixel nearest to where it happened to stop, up to 2 px from the pixel clicked. A reach is a hard edge, so a preview that could be 2 px out would lie at exactly the rocks where it matters. The seed and the preview now ask the same question of the same pixel.
- **Judgement calls:** a link through your king counts as joining its bank, though the king can fly off; the preview shows on the field and on the whole map, not on the small map; a seeded design with no factory pixels links nothing and shows nothing.

*M2.16, the split note.* Two nodes that shared a network and no longer do are a split, unless one of them is the king and the side has lost no node this tick. So: the king flying off alone, nothing; the king flying off when it was the only link between two outposts, a note; an outpost destroyed between the king and another, a note.

*M2.17, the queue.*
- **The strip** under the cards holds up to five jobs as 14 px icons (a longer queue shows four and "+N"), the bar, and the X.
- **Dropping a job from the middle** needed the command to say which. `Cancel` now names the job's place and its design; if the queue has moved up by the time it's applied, the job dropped is the last one of that design at or before that place. That's for the network play to come, where a command arrives a few ticks after it was given.
- **Kept, though Henry floated moving it:** right-click on the card still starts a repeat. It's the one click that says "keep making miners" before any job exists. Say the word and it goes.
- **Added, not asked for:** R, because the rules here are that anything the mouse drives has a key too, and repeat had none.

*M2.18, a guard inside its neighbour.* Landing seeds on their exact pixel moved the test outpost a pixel or two, and two of M2.12's tests failed. One was a real fault. A guard gave way to one thing at a time, stepping out through its nearer side; pushed out from under the king, it stepped into the space kept clear round a ship being printed, was pushed back, and after twelve rounds of that came to rest 8 px from the king where 13 is the rule. A waiting ship now takes the nearest point that is clear of everything it gives way to, and of two as near, the one further from home. For a ship with one thing in its way that is the same place as before.
- **The other failure was the test's.** A guard standing where ships come out of the factory steps clear each time one begins. That's the rule working, so those two tests now stop the outpost printing first.
- **A third test changed seed.** The AI-against-itself match on seed 1 now ends at 7:25 with one side holding a single factory, and the test needs both to plant twice. It runs on seed 2.

*M2.19, the reference fights.* The fleet the rounds were judged on carries hull that feeds nothing (its fighter makes 6 power and uses 4; its capital and titan are mostly hull). Rather than draw seven new ships to stand in for it, the reference is now the deck Henry drew: its nine arena tests and the six pairings of its warships at 687 px a side, twelve seeds each.

| Fight | Blue | Red | Average | What's left |
|---|---|---|---|---|
| 6 fighters v 1 frigate | 0 | 11, 1 draw | 0:16 | The frigate loses 9 px |
| 6 lancers v 1 frigate | 10 | 1, 1 draw | 0:17 | One lancer lost |
| 6 fighters v 6 lancers | 12 | 0 | 0:09 | Two fighters lost |
| 4 fighters v 4 miners | 12 | 0 | 0:04 | No fighter lost |
| 1 capital v 30 fighters | 12 | 0 | 0:11 | The capital loses 10 px |
| 1 capital v 40 lancers | 1 | 11 | 0:16 | 23 lancers lost |
| 12 lancers v 1 outpost | 12 | 0 | 0:04 | No lancer lost |
| 4 frigates v 1 outpost | 12 | 0 | 0:12 | Nothing lost |
| 1 capital v 1 shipyard | 12 | 0 | 0:14 | The capital loses 34 px |
| 76 fighters v 76 lancers | 11 | 1 | 0:12 | 44 fighters and 74 lancers lost |
| 76 fighters v 12 frigates | 0 | 12 | 0:12 | The frigates lose 92 px between them |
| 76 fighters v 1 capital | 0 | 12 | 0:24 | The capital loses 37 px |
| 76 lancers v 12 frigates | 12 | 0 | 0:26 | 33 lancers lost |
| 76 lancers v 1 capital | 12 | 0 | 0:10 | 21 lancers lost |
| 12 frigates v 1 capital | 4 | 8 | 0:28 | The capital loses 381 px |

- **The table is as fought under M2.20's costs** (a lancer is 9 px now, so 76 of them make 687 px, not 98). The first recording, a day earlier, read much the same. M2.21 took a second off the fight with miners in it and left the rest exactly as they were.
- **What it shows is the parts doing what they say, on one set of drawings:** flak can't hurt plate and lances can. Henry (2026-11-11): "fighters not being able to scratch capitals and frigates is obvious because fighters only have flak and those ships are covered in plate." It is a check that a rule change hasn't broken that, not a finding about balance.
- **The old fleet's 26 fights came out exactly as before** (`--pg-sim old`) after M2.15 to M2.19, so those changed nothing in the arena's rules. M2.20 moved three of them: the ones with the gunminer, whose scoops now weigh 2.
- **Not done, and flagged:** no destroyer or titan was redrawn, so those two sizes have no reference. Nothing in a match uses them. The old drawings stay in the code as fixtures for the sim's tests.

**The pace: what was measured (2026-11-11). Nothing is changed.**

*The yardstick.* Henry's winning play is now a scripted opening, the boom: plant the cheapest factory every time the bank allows, with the king printing miners without a pause. Against the AI on today's rules it plants nine factories and mines 3,141 px by 7:00, and wins six matches in ten, the median one at 8:45; Henry's own game had ten outposts by 7:12 and ended at 8:53. `--pace N` plays it N times, and the AI against itself N times.

*The levers,* ten matches each. The king's build speed is the same in every row.

| Rules | Boom: factories planted by 3:00 / 5:00 / 7:00 | Boom: mined by then | AI v AI: median match | AI v AI: second factory |
|---|---|---|---|---|
| Today | 2 / 5 / 9 | 450 / 1,395 / 3,141 | 16:21 | 3:35 |
| A seed 42 px dearer | 2 / 4 / 7 | 441 / 1,332 / 2,907 | 18:04 | 4:03 |
| A seed 84 px dearer | 1 / 4 / 4 | 441 / 1,325 / 2,904 | 16:27 | 4:45 |
| A seed 126 px dearer | 1 / 3 / 4 | 441 / 1,296 / 2,853 | 18:33 | 5:03 |
| A bite every 3.3 s (2.7 today) | 1 / 4 / 7 | 333 / 1,098 / 2,332 | 17:01 | 4:36 |
| A bite every 4 s | 1 / 3 / 5 | 261 / 846 / 1,827 | 19:03 | 5:46 |
| A bite every 5.3 s | 1 / 2 / 3 | 189 / 540 / 1,161 | 20:39 | 8:05 |
| Bite 3.3 s, seed 42 px dearer | 1 / 3 / 5 | 333 / 1,089 / 2,239 | 16:40 | 4:56 |
| Bite 3.3 s, seed 84 px dearer | 1 / 3 / 4 | 333 / 1,053 / 2,240 | 18:35 | 5:30 |
| Bite 4 s, seed 42 px dearer | 1 / 2 / 4 | 261 / 846 / 1,748 | 21:18 | 6:17 |
| Bite 4 s, seed 84 px dearer | 1 / 2 / 4 | 261 / 828 / 1,701 | 19:50 | 6:31 |
| Factory pixels print a third slower | 2 / 6 / 11 | 414 / 1,386 / 3,191 | 18:08 | 3:04 |
| Factory pixels print half as fast | 3 / 7 / 13 | 396 / 1,327 / 3,013 | 17:43 | 2:49 |
| Scoops use 1 power; the miner given hull to pay for it (9 px, 37 px/s) | 2 / 4 / 7 | 405 / 1,260 / 2,736 | 15:24 | 3:58 |
| Scoops use 1 power; the miner that rule favours (5 px, two scoops) | 3 / 8 / 9 | 684 / 2,112 / 3,744 | 14:03 | 3:09 |

- **How "dearer" was measured:** every factory pixel costs one more pixel (two more, three more), and the starting bank is raised by what that adds to the first outpost, so the opening is untouched and only the expansion costs more. For the outpost that is 42, 84 or 126 px on its 121. A flat fee per seed would do the same to an outpost and less to a shipyard.
- **Without the bigger starting bank** the opening stalls instead: the AI holds 40 px back when it plants, can't afford its first outpost at 0:04, and has mined 171 px by 3:00 where it has 261 today.
- **Round 2's cards, for scale:** the AI against itself ran a median 20:44 and mined 324 px by 4:00; today it mines 441 by then.

*What the numbers say.*
1. **The miner is the engine, not the factory.** An 8 px miner brings home 9 px every half minute, and the king prints one every 13 seconds. Dearer seeds halve the boom's factories by 7:00 and leave what it mines almost untouched (2,904 px against 3,141): the pixels wait in the bank.
2. **A slower bite cuts both.** At 4 s the boom has five factories by 7:00 and has mined 42% less.
3. **Slower factories backfire.** They drain the bank less, so the boom plants more of them: 11 and 13 by 7:00.
4. **Scoop power isn't a pace lever.** With the bite as it is (one per ship, two scoops or six), a power cost makes the best miner a 5 px drawing with two scoops and no gun, and that one mines a fifth more than today's. It belongs with the question of bigger miners, as Henry said.
5. **Match length is the noisy column.** At ten matches the medians wander by minutes: the boom against the AI ends anywhere from 7:07 to 17:13 with no pattern, so that pairing's lengths are left out of the table, and the AI-against-itself medians shouldn't be read closer than a couple of minutes. Planted and mined are steady.
6. **The boom beats the AI in every row,** six to nine times in ten. That's the AI, and a separate job.

*Options I put to Henry. None was taken: see Henry's answer below.*
- **A bite every 4 s, and seeds 42 px dearer.** Both of Henry's levers, each by a little. The boom has four factories by 7:00, not nine, and 56% of the pixels. The AI against itself runs 21:18, where round 2's cards ran 20:44.
- **Milder: a bite every 3.3 s, and seeds 42 px dearer.** Five factories by 7:00 and 71% of the pixels.
- **The bite alone, at 4 s.** One number and no new rule: five factories, 58% of the pixels.
- **Seeds alone, 84 px dearer.** Four factories by 7:00, and the boom as rich as today.

**Checks (2026-11-11, after M2.15 to M2.19):** 123 tests pass. 59 of 59 deliberate rule breaks are caught, thirteen of them new: a king flying off noted as a split, a king cut off by a loss not noted, two nodes parting not noted, a seed landing by where it is, the preview forgetting seeds in flight, linking a rock one pixel out of reach, saying a pixel too few, calling any link the king's, a guard stepping clear of one thing at a time, a guard taking the nearer-home of two places, a drop that takes the wrong job, a drop that misses a job the queue has moved up, and an AI alarm that never clears. The old fleet's 26 arena fights are unchanged.
- **One old break had gone unseen.** "The AI's alarm drops the moment a raider steps back" was caught only by counting the AI's order changes in one match, and after today's changes that match stays under the count with the rule switched off. The alarm now has a test of its own: a raider parked in range, a step outside it, and well clear.
- **The pace table was measured on the build before M2.15 to M2.18.** Those change where a seed lands by a pixel or two and where a displaced guard waits, not the economy.

**Henry's answer to the pace table (2026-11-11), and M2.20.**

*What Henry said.*
- **On the method:** "correct me if I'm wrong but a lot of your reasoning appears to be based on just the one army we designed, which doesn't seem like the way to approach this. for example, where is "42" coming from? is that the number of factory pixels in one of our factories? if we're going to be using AI vs AI data like this, we should have a variety of AIs, mostly stronger ones, using a variety of armies. We don't want to over index on exactly what is in the game now. It's an early development build."
- **On the levers:** "my proposal is make bites a little slower, make the map slightly larger, make factories cost more hull, make other things cost more hull (lances?), make outposts build themselves more slowly, make scoops cost 1 hull and be a little heavier, make the king a little slower, and so forth? there are a lot of other levers than the ones you mentioned."
- **On the reference table:** "fighters not being able to scratch capitals and frigates is obvious because fighters only have flak and those ships are covered in plate."

*Henry was right on each.* 42 is the number of factory pixels in the one outpost there is; "a seed 42 px dearer" hid that. Every row of the pace table came from one deck, one AI and one scripted opening. Until there are several AIs, stronger than this one, flying several armies, those runs are good for one thing: seeing that the game still plays. They aren't a way to pick a number.

*M2.20: what was changed.* Each of Henry's levers, by the smallest step that could be felt. They are rules, so they hold for any drawing.

| Lever | Was | Is |
|---|---|---|
| A bite, with two scoops or more | every 2.7 s | every 3.3 s |
| The map | 1200 x 900, 16 pairs of small rocks | 1400 x 1050, 22 pairs: a sixth longer each way, rocks as thick on the ground |
| A factory pixel | uses 1 power | uses 2 |
| A lance | uses 2 power | uses 3 |
| A scoop | free, weighs 1 | uses 1 power, weighs 2 |
| A seed eating its rock | 6 px/s, plus its factory pixels | 4 px/s, plus its factory pixels |
| The king | 45 px/s | 37.5 px/s |

- **Power comes in whole numbers,** so the smallest step for a factory pixel is from 1 to 2. That one isn't "a little": a factory of the same size and price prints two thirds as fast.
- **The king's build speed is untouched,** as ruled.
- **The whole map now shows at a fifth of its size** (a quarter no longer fits the field), and the small map at a sixteenth.
- **The king moves by quarter-pixels at full speed too,** as it already did when slowed: 2 px one tick and 3 the next.

*The starter deck, re-hulled.* The new costs left five of Henry's seven cards short of power. Each keeps its shape and size and gets the hull it now needs; the fighter and the capital needed nothing.

| Card | Was | Is |
|---|---|---|
| Lancer | 7 px, 37 px/s | 9 px, 33 px/s: hull either side of the rear lance |
| Frigate | 55 px | 55 px: four of its plates are hull |
| Miner | 8 px, 45 px/s: three scoops, three engines, a flak | 9 px, 30 px/s: three scoops, two engines, a flak, two hull |
| Outpost | 121 px, 42 factory pixels, 1.6 px/s | 121 px, 28 factory pixels, 1.05 px/s; builds up to 280 px |
| Shipyard | 693 px, 184 factory pixels, 6.9 px/s | 693 px, 118 factory pixels, 4.4 px/s; builds up to 1,180 px |

- **These are stand-ins, drawn by rule and not by play.** A save still holding one of the five untouched gets the new one when it loads; a card that was edited is kept, and the yard's power gauge will show it short.
- **The miner is a guess.** Nobody has played drawings against each other under these costs, and with the bite as it is a two-scoop 5 px miner is still legal and still flies at top speed. That question stays with bigger miners.

*Does it still play?* Yes: 123 tests pass, matches run to an end, and the AI still plants and attacks. For what one deck and one AI are worth, the boom has seven factories by 7:00 where it had nine, and has mined a little over half as much (six matches).

*Not pulled, and there if this isn't enough:* the starting bank, the king's own trickle, a factory pixel's print rate, how big a ship a factory pixel lets it build, a seed's own speed, how much rock there is, how far a node reaches.

*Three tests moved with the numbers.* The floor under an AI economy at 4:00 went from 300 px to 150 (it mines about 250 now). A ship that finds itself inside a bigger enemy may now be there for up to 20 ticks, not 10: a king at 2.5 px a tick takes longer to pass over a ship leaving at 2. And the shipyard's factory count.

**An order for later: SAB** (Henry, 2026-11-11, "another side note for the future"): "possibly add a SAB sabotage order that has the ship seek out and attack things like miners and poorly protected factories". Kept as an idea; nothing built. It would be the first order that picks its target by what it is, and it gives raids on an economy a name.

**Checks (2026-11-11, after M2.20):** 123 tests pass, and 60 of 60 deliberate rule breaks are caught on the new numbers.

**Henry's wiggling miner, and M2.21 (2026-11-11).** "got a little wiggle happening here with this blue miner. not too bad." (With a screenshot: a miner at the edge of a group of them on a rock, an outpost beyond the group.)

- **What it was.** Ships of one side keep a pixel or two apart: each friend close by pushes the other away. A miner going to its rock pixel was already let off (pushed off, it never got its bite). A full miner going home was not, so the miners it had been working beside pushed it like any friend. Miners at one rock face sit on top of each other and give way to nothing, and a few pushes together beat a miner's engines. With the heap between it and its drop-off, a full miner stopped short, was pushed back out and came on again.
- **It was worse than it looked.** Counted over nine nine-minute matches on the old rule: 50 of 3,536 trips home took over 4 s longer than the straight flight, and the worst took 83 s longer. On the new rule none of 3,206 does, and the worst is 2 s over. That's one deck and three pairs of scripts, so it counts the fault and says nothing about balance. A boom left alone mined the same either way in its first four minutes (about 500 px on each of three seeds), so the fix costs nothing early. What the hold-ups cost a big economy I haven't measured.
- **The rule (M2.21).** A miner at work takes no part in keeping apart, either way. Going to a rock pixel, working it, or taking a full load home, it isn't pushed, and it pushes nobody.
- **Why both halves.** Either half alone cures the full miner. The second is the root: things that push and never give way add up to a wall. A light, fast ship got through a heap of miners on its momentum. A slow, heavy one didn't, and neither did a miner starting from rest beside it. The first half is there so that nothing else heaped on a miner's way home (parked hulks, say) can hold it either.
- **What changes on screen.** Full miners fly a straight line home: through the heap, through each other, and over guards at their posts. Other ships no longer steer round miners at work. Miners heaped on a few pixels of rock were there before and still are.
- **Not changed.** A ship at its post still pushes what flies past it. Posts are 16 px apart, so those pushes never add up to a wall. A gunless miner running home hurt isn't at work and is pushed like any ship, but no longer by the heap.
- **Who sees it.** Whether a ship was at work is taken with the tick's snapshot, like its position, so every ship gets the same answer whatever order they think in.
- **Tests.** Three new. A full miner behind a heap of working miners and three parked hulks flies home in the time the distance takes, and a picket it flies over doesn't stir. A heavy tug sent to a spot beyond a heap of miners gets there. And in a six-minute match of a booming side against the AI, every trip home to a drop-off that stayed put takes the straight flight and no more than 3 s on top (22 s at the worst on the old rule; 1.2 s now).
- **One older test had gone blind, and is fixed.** "Guards sit still while miners fly past" proved that a guard at its post isn't pushed. Miners no longer push anyone, so it proved nothing: a fighter is now flown straight over a guard as well.
- **The arena.** The two reference fights with miners in them moved by a second (4 fighters v 4 miners averages 0:04, not 0:05; the old fleet's gunminers 0:10, not 0:09), with the same results. The other 39 are exactly as they were.

**Checks (2026-11-11, after M2.21):** 126 tests pass. 64 of 64 deliberate rule breaks are caught, four of them new: a full miner pushed like any other ship, miners at work pushing their own side, a full miner pushing what it flies over, and the old rule whole.

**Henry's game on M2.20 and M2.21 (2026-11-11).** "you basically just made miners not collide? that seems fine to me, yeah." And: "finished a test game. speed was improved but the AI was more vulnerable early due to the speed slowing. that's not a problem with the speed that's a problem with the AI. new build window seemed good."

- **What it settles.** M2.21 keeps. M2.20's pace is better, and no lever moves on the strength of this game. The king's queue (M2.17) seemed good, if the build window is the queue.
- **What it opens: the AI's early game.** As written, the AI's king prints miners and saves for factories. It prints guns only once a raider is within 200 px of something it owns, or once its bank runs 120 px past the next factory's price. So its first standing guards come out of its first factory. M2.20 made that later (a seed eats its rock more slowly, and a factory of the same size prints two thirds as fast) while a king prints as fast as ever. Nothing is built for this yet: what the AI should do instead is to be agreed first.
- **Still without a verdict:** the links shown while planting (M2.15), the split note (M2.16), whether the card's right-click should go (M2.17), the reference fights on the starter deck and whether a destroyer and titan are wanted (M2.19), the five stand-in cards (M2.20), and the miner itself (M2.14). M2.18 is a fault fixed, with nothing to judge.

**The rest of the verdicts (Henry, 2026-11-11, the same day).**

- **M2.15, the links while planting:** "it's good for now".
- **M2.16, the split note:** "didn't bother me, yet".
- **M2.17, the card's right-click:** "I actually thought right-clicking the card should be the infinite-that-unit queue entry on the end of the queue." So it stays, as a way to put a repeating job of that card at the end of the queue. Today it marks the card itself as repeating, wherever its jobs are. To build.
- **M2.19, the reference fights:** "I'm not sure but I think it's fine. We will want more preset ship options soon but I think making more pixel/part variety is higher priority (and the ships that use those.)" No destroyer or titan for now. New parts come before new presets.
- **M2.20, the stand-in cards:** "Looked good to me".
- **M2.14, the miner:** "New miner hull is fine for now".
- **The AI.** "To me the main issue with the AI was that it didn't use its king's gun, which can make a huge difference in an early battle. I don't think that's something to fix right now." So the rule I had proposed (guns before the next factory while it's outgunned) isn't taken, and wasn't the main fault: the AI's king backs away from every armed enemy and never fights. Noted for when the AI is next worked on.
- **Next:** "I think we're ready for the network spike."

**M2.22: the card's right-click (2026-11-11).** Henry: "I actually thought right-clicking the card should be the infinite-that-unit queue entry on the end of the queue."

- **What it did before.** It marked the card as repeating. With none of it queued that added a repeating job, which is what Henry expected. With one queued already it marked the first of those, and added nothing.
- **What it does now.** Always a new job on the end of the queue, on repeat. Two fighters asked for and then the fighter card right-clicked is three jobs: two fighters, then fighters for ever. A repeating job that's finished goes to the back again, as before.
- **Right-click the card again** and every job of it stops repeating. The jobs stay; drop them with a click. My call: it's the same click undoing itself.
- **Right-click a job in the queue** and that one job repeats or stops. Before, it switched the card's repeat, which could mark a different job from the one clicked.
- **R** is the last job asked for, where it was the one being printed. So a number and then R is a right-click on the card, from the keys. It names the job by its design and finds it when the command is applied, because with lag the job isn't in the queue yet when R is pressed. (The first try at this lost the R under two ticks of lag.)
- **Not changed, and asked above:** a ship asked for once after a repeating job still queues behind that job's first turn. After that first turn the repeating job is at the back, so it's one job's wait and only once.

**M2.23: the map's edge (2026-11-11).** A ship is kept half its longer side from the edge, so it has room to turn. That was done to everything, every tick, flying or not. A piece shot off a king right beside the edge lay with all its pixels on the map but nearer the edge than that, and on the next tick it was moved three pixels inwards: a hulk with no engines, moved. The audit's rule "nothing without thrust changes position" caught it, on a new roll of its random matches. Now the edge stops what flies into it and moves nothing that can't fly.

**N1: the network spike (2026-11-11).** Henry: "I think we're ready for the network spike." It answers two questions, and both answers are Henry's to give by playing.

*The two questions.*
1. **Does the king still feel right when its keys land late?** Two machines can only play one match if each has the other's commands before the tick they're for, so every command waits a few ticks. If that feels fine at the lag a real connection needs, plain lockstep is enough and no new part has to obey any network rule. If it doesn't, the next step is to show your own king's moves at once and correct them when the tick arrives, and that has a cost every later part pays.
2. **Do two machines stay in step for a whole match?** They do in the tests. A real match on two real machines is the proof.

*What's built.*
- **The lag dial.** L, or LAG at the top of the field. Against the AI it runs from 0 to 8 ticks (0, 1, 2, 3, 4, 6, 8), 67 ms a tick. Everything you do waits that long: steering, cards, orders, planting.
- **HOST A MATCH and JOIN A MATCH** on the match menu. The joiner types the host's address; it starts as 127.0.0.1, this machine. The host plays blue, the joiner red, each with their own seven designs. Also `--host [PORT]`, `--host local` and `--join ADDR[:PORT]`.
- **In step.** Each machine runs the whole match and sends only its commands, a turn for every tick. It plays a tick when it holds both sides' turns for it: blue's commands first, then red's, on both machines. Short of the other's turn it waits, and after half a second says so. A machine that falls behind (a window being dragged) plays up to two extra ticks a frame until it's level.
- **The check.** Once a second each machine hashes everything in the match and sends the number. The top of the field shows IN STEP and the last time they agreed, or OUT OF STEP and since when. Hashing costs about a thousandth of a second.
- **The match screen plays either side.** It took the player for blue in about fifty places. Now red's window has red's cards, bank, queue and king, and YOU WIN means you.

*How the hash can't miss a field.* The arena's hash names every one of its fields, so adding a field without deciding about it doesn't compile. Everything inside (ships, shots, the queues, the banks, the dice) derives its hash, so a new field there is in it with nothing to remember. A new part's state can't be left out by forgetting.

*What isn't trusted.* Everything off the wire is read as text and checked: a side that isn't 0 or 1, a slot off the deck, a turn out of order or sent twice, a line that isn't the protocol. A bad line ends the match; nothing is ever indexed with a number from the other machine. A command counts only for the side whose turn it came in, on both machines alike, so a game that sends "blue's king prints a fighter" in red's turn changes nothing.

*What a network match can't do, on purpose for now.* Pause, run at 2x or 4x, or stop under the rules panel or the quit question (the other clock is running). Play again: both players would have to want it. Rejoin after a drop. Repair a match that's out of step: it only says so. Get through a home router by itself: the host's port 7777 has to be reachable. None of that is needed to answer the two questions.

*What turned up on the way.*
- **Keys in pairs.** "4 then R" was lost under lag (above). Anything that reads the queue to decide what a second key means has to allow for commands still on their way. Backspace has the same weakness and I left it: a job asked for and dropped inside the same fifth of a second is not dropped.
- **Panels froze the match.** F1 and the quit question stopped a match while they were up. Against the AI that's right; in a network match it would have frozen the other player too.
- **M2.23,** above.

*Tested.* Alone, a command is applied exactly the dial's ticks late and no later. Two machines joined by a wire with no lag play, tick for tick, the very match a script of 690 commands made on one machine, and hash the same throughout. With messages taking one to three frames, each machine missing frames and hanging for two seconds, and both dials turning, every tick both reach still hashes the same, and a machine that fell behind comes level. One pixel changed in one bank on one machine is caught within a second, on both, at the right tick. A machine plays no tick without the other's turn. Over real sockets on this machine: host, join, hellos, four seconds of match, a goodbye. The match screen, driven by its keys and buttons from red's side. And two real windows, run against each other for fifteen seconds: they met, started the same match, agreed on their first check, and the host was told when the other closed.

*Not tested, and can't be from here.* Two machines. A connection with real lag and loss. A whole match by two people.

**Henry on the dial, and lag made up for on the king alone (2026-11-11).**

- **The verdict:** "honestly it was playable up to 4 lag." Four ticks is 267 ms. A home connection's one-way trip is mostly under one tick, so a real match would want the dial at 1 or 2.
- **Henry's idea:** "do you think we could do some sort of minimal lag comp just for the king's movement and nothing else? I suspect that might save 80% of the control feel."
- **How it would be done.** No second copy of the king's movement rule. Each frame, copy the match, apply your own commands still waiting out the lag, run the copy forward that many ticks, and draw your king where the copy has it, with the view following. The copy runs the real rules, so a new part that pushes or slows a king is covered with nothing to remember. This takes back what I wrote above about a cost every later part pays: that's only so for a hand-written prediction.
- **What it costs.** A tick averages 0.7 to 0.9 ms over whole bot matches (22,620 ticks in 16.5 s; 20,235 in 17.5 s), so four ticks ahead is about 4 ms of a 67 ms frame. The worst tick of a big late match isn't measured.
- **King only, or the whole screen.** King only: your king is drawn ahead of where the match has it by 2.5 px a tick of lag while it moves (5 px at lag 2, 10 at lag 4), and it's drawn in a later moment than the shots coming at it, so a dodge can't be read off the screen. The whole screen drawn from the copy: one moment again, and your cards and queue answer at once; but what the other player did that hasn't arrived isn't in the picture, so their king jumps a few pixels when it changes course.
- **Not built.** Henry: "I can't decide if the complexity cost is worth implementing lag comp now, or if we should wait to see if the input lag approach bothers us in the future." My answer is to wait. It changes only what one machine draws: not the match, not the wire, not a rule. So it can be added any day, and nothing built before then makes it harder, so long as the match stays deterministic (the check proves that every second) and the screen is drawn from the match's own state. What would tell us it's needed is a real match on two machines at the dial that connection needs. **Ruled** (Henry, 2026-11-11): "Yes, wait."
- **What the spike is, since Henry asked:** nobody is the server and no pixels are sent. Each machine runs the whole match and draws its own screen; only commands cross. Hosting means waiting for the other game to connect, choosing the map's seed and playing blue. Each player's commands wait that player's own dial, so neither is ahead of the other.

**The parts talk opens (2026-11-11).** A talk, not a round: nothing below is built, and only what's marked as Henry's is ruled.

- **What today's nine parts leave out (mine).** Reach: weapons fire from 48 to 130 px and a guard answers anything within 160. Protection: nothing shields anything. Big-ship parts: only the cannon needs a big drawing. Miner choices: one part.
- **Four tests for a new part (mine, not ruled).** The shape is the rule. A job nobody has, and an answer you can draw. Paid for in pixels, weight and power. The part flies itself.
- **Four candidates (mine).** BEAM: beam pixels in a straight row from stern to bow make one gun; reach and cut grow with the row; it shows its line, then cuts everything on it. SHIELD: a bubble that stops shots from outside until its charge is gone. HOLD: carries pixels, can't bite. MISSILE: a slow following shot that flak can shoot down. Henry: "i like the candidates." And: "i do like the way your beam and missile proposals integrating with AI. ships can intentionally dodge beams and intentionally shoot down missiles. those are details that will be fun for players to discover."
- **No converting (Henry).** "don't worry about converting. a hacking part or something like the salvage corvette from homeworld would be cool but this is an abstract strategy game with custom-designed pieces, spaceship combat being the main theme is intentional."
- **Henry's own list.** "other things I think might be cool: logistics parts for miners that make them mine faster in a radius. logistic parts that make nearby ships fly faster. diagonal guns. stasis fields. rotation engines. rapid-fire guns. more turret-based guns. a weapons charger that is expensive but upgrades nearby guns. something fancy each block of which needs to be given clearance from other pixels on the ship (exception to the connectivity rule). a weapon that does a sweep around the ship. more play with long and short range as you said. more play with tracking speed."
- **Sub-orders (Henry).** "that reminds me, we need to implement the subcommands soon." From the proving ground's first playtest: Hunt into kiting, orbiting or attack runs, with a group size.
- **Henry asked: "how is the strength vs. radius of the shield determined?"** My answer: by the drawing. Each shield pixel covers a fixed disc round itself and holds its own charge; the bubble is the discs together. Spread them and the bubble is wide and thin; knot them and it's small and deep. A drained pixel's disc drops out until it refills, so fire on one spot opens a hole. Refill slows when the ship is short of power, by the rule guns already follow. Henry's clearance idea fits as the emitter's price: it floats in a gap, so it can't be plated over.
- **Henry asked: "does hold imply we will rework scoop?"** Yes, and my pitch had it wrong. A miner with two or more scoops bites one pixel every 3.3 s however many it has; scoops past the second only carry, 3 each. So the scoop is the hold already. For the hold to be a choice the scoop has to be the biter (bite rate growing with scoops, no stop at two) and the carrying has to move to holds. That's the change Henry said should wait "while they still cost 0 hull"; since M2.20 a scoop uses 1 power and weighs 2. It stops punishing big miners without rewarding them; Henry's mining booster is what would.
- **The list sorted (mine).** Lines: lance, diagonal gun, beam, sweep. Turrets: rapid-fire, more turrets, range, tracking. Fields: shield, mining booster, speed booster, stasis. Neighbours on the drawing: weapons charger, rotation engines, clearance. Turrets and fields lean on sub-orders: tracking is beaten by orbiting, range by kiting, and a field needs ships that stay in it.
- **Order I proposed (not ruled).** Sub-orders on today's parts first; then the lines (beam, diagonal gun, rotation engines); then the shield; then mining (scoops that scale, hold, booster); then the missile with a rapid-fire gun. Stasis wants its own talk.

**The shield, proposed again alone (2026-11-11).** Henry: "I couldn't quite parse your shield idea. Can you give a shield proposal alone?" Numbers are placeholders.

1. SHIELD is a one-pixel part. Each one throws a circle 10 px in radius, centred on itself.
2. An enemy shot that reaches a live circle from outside stops at its edge, and that pixel loses 1 of its 4 charges.
3. At 0 charges its circle goes out. After 3 s with no hit it comes back, full. Short of power, it takes longer.
4. Ships fly through circles. A shot fired from inside a circle isn't stopped by it.
5. A circle covers whatever is inside it, friends included.
6. A shield pixel uses a lot of power (4, say).

Strength against radius is then where the pixels are drawn. Four together: one small bubble, 16 hits before it's open. The same four spread 15 px apart along a hull: a bubble 65 px long that opens a hole after 4 hits on one circle. Optional, from Henry's clearance idea: a shield pixel must have empty cells all round it.

**Sub-orders: what ships do today (read from the code, 2026-11-11).** Henry: "those three make sense to me but we should probably talk about the details a bit more. For example I'm guessing that current behavior makes some decisions on what attack runs and kite means exactly and we should make those explicit. Also there's the question of target selection."

*Which enemy a ship flies at.*
- HUNT: the nearest enemy of any kind, anywhere, picked again every tick. It doesn't hold on to a target, and doesn't ask whether its guns can hurt it. A hunter with no guns stays beside its nearest armed friend.
- GUARD: the nearest enemy within 160 px of home, while the ship is within 210 px of home.
- ESCORT: the nearest enemy within 110 px of the king, while the ship is within 160 px of the king.
- Armed ships never break off or run. Unarmed ships on GUARD or HARVEST run home for 2 s after a hit.

*How it flies at it: the drawing decides.* One engine or more for every 8 of weight flies attack runs; less stands off; none stays put.
- **Attack runs.** It aims where the target will be and dives at full speed through the target's centre, whatever its guns' range. A lance ship first slides onto the target's row or column. It turns outbound within 6 px of the target's edge, or on a bump, or by peeling off sideways if a bigger ship is in the way. Outbound it flies straight on until it's 48 px past the target (at least half a second, at most 2 s), then comes about.
- **Stand-off.** It holds a distance from the target and faces it: 75 px with a lance, else 87 px with a cannon, else 32 px with flak only (three quarters or two thirds of the gun's range). Lance ships pick a spot on the target's row or column and avoid spots a friend holds. If the target closes, the ship backs away at its own speed. It takes no account of the enemy's range.

*Which enemy each gun shoots, whatever the ship is flying at.*
- Flak: the nearest enemy within 48 px of the gun, with 12 px added for each shot already on its way there, so fire spreads. It aims at a random standing pixel, plate included.
- Lance: no choice. It fires when an enemy pixel is on its line ahead within 100 px.
- Cannon: of the enemies within 130 px, those at least two thirds the size of the biggest; of those, the one its barrel reaches soonest. It turns about 42 degrees a second.

*Groups.* None. A ship on HUNT leaves alone the moment it's built. Massing is done by hand: hold a design on GUARD, then switch it to HUNT.

*What sub-orders would have to settle (mine, not ruled).* Whether the player picks the style or the drawing still does. What KITE adds to stand-off (the enemy's range, and being open to light ships). ORBIT's radius. Whether a ship keeps its target. What a ship may be told to go for (nearest, armed ships, workers, the biggest, the king). Whether a ship ever breaks off. Group size.

*My first proposal, and Henry's answer.* I proposed three switches per design: style (RUNS, KITE, ORBIT), prey (ANY, ARMED, WORKERS, KING) and group size, with a ship keeping its target until it dies or something is clearly nearer. Henry (2026-11-11): "I think that's too many parameters to control all of them at runtime: base order, flight pattern, target selection, and group size."

*The smaller plan (mine, not ruled).* In a match there is still one switch per card, the order, as now. Style and group size go on the card in the yard, fixed for the match and shown on the card. Prey is not a switch: SAB joins the order list beside HUNT, GUARD and ESCORT, as Henry first described it, and goes for miners and factories. ARMED and KING are dropped. The same hull with two styles is two cards, out of seven. **Agreed** (Henry, 2026-11-11): "Agreed on the updated sub-orders proposal." Not yet built.

**Two shield parts (2026-11-11).** Henry: "For the shield, it's a reasonable design for a personal shield. My idea for personal shield was to have it automatically be the shape of the ship. I'm pretty sure yours is better. It does drop one thing which had been mentioned though, which was shields being able to protect allies. We could make a different part like a shield projector that does have the ability to protect allies."

- **SHIELD** stays as proposed above: the personal one. Its circle stops any enemy shot that reaches it, so a friend tucked inside is covered, but at 10 px that's incidental.
- **PROJECTOR (mine, not ruled).** The same circle rule, built on Henry's clearance idea. A projector floats in a gap in the ship with nothing touching it, and the wider the gap, the bigger its circle (say 4 px of radius for each empty cell of clearance). A wide dome needs a ship drawn as a ring round a hole. It holds more charges than a shield pixel and uses more power. Shots from inside pass, so ships on attack runs get under it and ships that stand off don't. Today's flying already uses it: on ESCORT it domes the king, and a gunless ship on HUNT stays beside its nearest armed friend.
- **Henry on the dome (2026-11-11):** "I think I like the PROJECTOR proposal. How does it protect allies exactly? Does it give them individual shields? Getting the ship AI to effectively exploit a big circle that it has to stay inside and shoot out from might be a bit difficult."
- **My answer: the dome doesn't, and Henry's worry is right.** The dome only stops shots at its own edge, so it covers what happens to be inside. Nothing makes a friend stay inside: a ship on attack runs flies through its target and 48 px past, and a ship standing off sits 75 or 87 px from its target wherever the dome is. Keeping friends inside would be formation flying written for one part, which breaks the test that a part flies itself.
- **PROJECTOR, second version (mine, not ruled): a skin for each friend in reach.** It still floats in a gap, and its reach grows with the gap (say 12 px for each empty cell). Every friendly ship whose centre is within reach wears a skin in its own shape, which is Henry's shape-of-the-ship shield. An enemy shot that would hit a skinned ship is stopped and costs the projector 1 charge from one pool (say 12). At 0 every skin goes out, and after 3 quiet seconds they come back. It doesn't skin its own ship. Friends fly as they always do; only the projector's ship has to stay near them, and a gunless ship on HUNT, a ship on ESCORT and a ship at its post all do that today. Its answers: shoot the projector's ship; empty the pool with many small hits, since a flak shot and a cannon shell cost it the same; or fight where the friends have flown out of reach. Lost against the dome: getting under it, and where you stand mattering.

**Sub-orders, as built (2026-11-11).** Henry: "Yes, build it." Built on today's nine parts, as agreed: one switch per card in a match (the order), with style and group size set in the yard. Rows S1 to S6 above.

*What each does.*
- **Style (S1).** RUNS, KITE and ORBIT are buttons on the yard's flight row, where the words ATTACK RUNS or STAND-OFF were. A design with no engines has no buttons and says STAYS PUT.
- **KITE (S2).** The ship holds at the range of its guns from what it's after. If another enemy is nearer than that range, the nearest of all is the one it keeps its distance from. For the heavy ships that always stood off, this is what they did already; what's new is that a light ship can be told to.
- **ORBIT (S3).** It makes for the place on its circle 12 px round from where it is, so far off it flies straight for the circle and on it it goes round. It goes the way it was already going. Where the circle leaves the map, or something solid stops it, it turns back the other way.
- **SAB (S4).** A miner or factory is anything with a scoop or a factory pixel still on it, or anything planted. Never a king, whatever is drawn on it. A seed in flight doesn't count.
- **Kept targets (S5).** On every order. On SAB a miner or factory always comes before what it was hunting.
- **Groups (S6).** Waiting ships guard where they were built and show WAITING FOR ITS GROUP. When as many as the group size are waiting within 96 px of one of them, all of those go. Sent out from GUARD or ESCORT they gather first, and a group already gathered leaves on that tick. Switched between HUNT and SAB, the ones already out stay out.

*My calls, for Henry to overrule.*
1. **Style and group size aren't drawn on the card.** I said they would be. A card is 90 px wide and its two lines are full, so they're in the hover line of the card and of its order tag: "THESE ORBIT, AND LEAVE IN GROUPS OF 3."
2. **Picking a style pins it.** Until a style is picked, the drawing picks, and a redraw can change it. Once picked it stays however the ship is redrawn. There's no button back to "as drawn".
3. **Groups gather by nearness, not by builder.** The king builds wherever it happens to be, so "built by the same thing" would have split its ships up.
4. **Everything waiting together leaves together,** even when that's more than the group size. A group that loses ships doesn't come back for more.
5. **The yard's six orders are in two rows of three,** and the lines under them moved down a few pixels. O steps through the orders, Y the styles, G the group sizes.
6. **The AI doesn't use any of it.** Its ships keep their targets like everyone's; it never picks SAB, a style or a group.
7. **The network protocol is 2.** A design's line can now carry a style and a group, and SAB crosses the wire. Two games on different versions refuse each other, as before.

*Not built.* Breaking off when hurt. Going for armed ships only, or for the king only. Anything on the card itself.

*What moved.* The arena's reference fights: 11 of 37 changed their score by a win or two out of twelve, none changed winner, none moved by three. The cause is kept targets; nothing else touches a ship with no style picked. AI against AI over four seeds: 3 to 1 as before, 17:11 to 24:56 with a median of 23:52 (it was 22:29 to 26:12, median 25:08).

*Found on the way.* My first orbit steered by a speed in and a speed round. Where its circle left the map it pushed into the edge and stayed there. I found that planning its test, before it ever ran, and rewrote it to make for a place on the circle, which the map's edge can cut short.

*Checks (2026-11-11, after sub-orders).* 153 tests pass, twelve of them new. 103 of 103 deliberate rule breaks are caught: the 84 from before, run again because every ship's choice of target changed, and 19 new ones (a target picked afresh every tick, a target never let go, a chosen style ignored, a kiter that lets things close, an orbit that doesn't go round, one 20 px too wide, one that never turns back at the edge, SAB hunting like HUNT, SAB taking a king, SAB lost on the wire, groups leaving alone, a waiting ship hunting anyway, a group counted from all over the map, HUNT to SAB calling ships back, a gathered group waiting a tick more, style and group lost from a design's line, an engineless design told to orbit, the yard's style buttons working on one, a card silent about how its ships fly). Not done: I haven't watched any of it in a window, beyond one exported frame of thirty fighters orbiting a capital.

**Henry's verdicts on sub-orders, and what came with them (2026-11-11).** Fourteen questions, asked in the chat.

- **KITE, ORBIT and kept targets (S2, S3, S5):** "i didn't see anything that seemed odd."
- **SAB, groups, and a waiting crowd bigger than its group leaving together (S4, S6):** "i saw groups and sabotage, yes all good."
- **Style picked in the yard, nothing drawn on the card, the yard's layout and keys (S1):** "all seems good for now."
- **Not answered:** whether a picked style wants a button back to "as drawn". Left as built.
- **The projector's second version:** "it sounded right, yeah, I'm curious to try some of these new parts out."
- **The weapons charger:** "I meant own ship. I suppose both would be interesting. We'll want to think about how to make these "auras" more interesting than just put pixels down, get bonuses."
- **The order of the part rounds:** "i liked your original proposal of beam and shield first." So the first part round is BEAM and SHIELD. The diagonal gun and rotation engines come after.
- **The AI:** "AI seems very weak still. specifically its lack of defense capabilities have been killing it so far."

*What the log shows.* The two matches played on this build, both on Henry's own deck.
- **Seed 28100, won at 4:42.** An outpost at 0:03, LANCER on repeat from 0:16, lancers on HUNT at 0:25. The AI's king was beaten down to slow at 1:53 and its only factory was gone by 2:00. After that the AI had the bigger income (369 mined to Henry's 99 by 4:00) and lost everything it made with it. At 4:00 it had one ship of 9 px standing and 183 px unspent.
- **Seed 5635, won at 9:10.** Fighters and lancers on SAB from 1:36, frigates from 2:56. The AI mined more for eight minutes (772 to 756 at 8:00) and lost its two factories between 6:00 and 8:00.

*What the AI does about a raid (read from `bot_think` and `bot_steer`).*
- **Guns come last.** Miners first, one more than it has factories, raid or no raid. With no raid it prints a warship only when the bank holds 120 px more than it's saving for its next factory.
- **Under raid it saves more, not less.** It holds 150 px back on top of a factory's price before it replants. With 183 px and no factory it was waiting for 271.
- **Under raid it feeds ships in one at a time.** Its king prints what counters the raiders for 100 px or less, and every warship is put on HUNT, so each leaves for the raiders alone the moment it's built.
- **A wave that's out isn't called home.**
- **Its king only backs away** from anything armed within 90 px. It never uses ESCORT.

*What I'd propose (mine, not ruled).* Measure first, change second. Two scripted openings taken from these two matches, RUSH and SAB, played against today's AI over several seeds on both decks, beside the five openings it's measured against now. Then the changes, in the order the log blames them: under raid, guns before miners and nothing held back; a garrison kept in step with the enemy's warships before the second factory; ships under raid gathering on the king or under a factory's flak and fighting together, which is what groups are for; a wave called home when home is losing. Every number waits for the measuring.

*Auras (mine, not ruled).* Henry's worry is the first of the four tests: with "pixels down, bonus up" the drawing doesn't matter. Four ways to make it matter.
1. **Touching, for a part that helps its own ship.** A charger feeds the guns it touches, one gun each. It competes with plate and hull for the cells beside a gun, and shooting it off takes the charge away.
2. **Reach is paid for in the drawing.** A part that helps other ships floats in a gap (Henry's clearance idea) and the gap sets its reach. A wide aura is a ship with a hole in it and a soft part in the hole.
3. **One pool, shared.** What it gives is split among those in reach, like the projector's charges. More pixels and more friends don't simply add up, and where ships stand becomes a choice.
4. **It takes as well as gives.** A boosted engine turns wider. A charged gun waits longer between shots. Then adding one isn't always right.

**The AI's defence (2026-11-12).** Henry: "AI defence first." Rows D1 to D6 above. Measured first, changed second, and every change measured again on its own.

*What Henry's rush is.* The log doesn't keep the king's keys, so I scripted the opening from what it does keep (an outpost at 0:03, LANCER on repeat, lancers on HUNT) and played it against the AI as committed.
- **With the rushing king at home it loses.** 0 of 4 on Henry's deck and 1 of 4 on the starter deck. That's the SUMMONER opening over again, which the AI has always beaten.
- **With the king flying over it wins every time.** 8 of 8 on both decks, the median match 2:55 on the starter deck and 2:21 on Henry's. The king crosses the map in 25 s, stops 40 px from the AI's first factory while it's still growing, and prints its lancers there. In the match I watched tick by tick (seed 1, starter deck) the factory was dead 25 s later.
- **What the AI did meanwhile.** Its king stayed at its start, 125 px from the factory, where its flak reaches nothing that's attacking the factory. It finished its second miner first. Its first gun was delivered 20 s after the enemy king arrived, with the factory down to its last pixels. Then it backed away from the nearest gun until it was caught.
- So I take it Henry's king went too. **That's a guess from the log, for Henry to confirm.**

*The two new openings.*
- **RUSH** (`rusher`): one factory at once, a king that prints its cheapest lance or cannon ship without a pause, everything on HUNT. Its king flies to within 40 px of the nearest thing the enemy has planted (or the enemy king, when nothing is planted) and prints there. Once it has lost a quarter of its pixels it goes home for good and prints from there. Without that last rule the AI beat it by killing a king that sat still under its lancers, which says little about defence.
- **SAB** (`saboteur`): the AI's economy with every warship on SAB.
- The scripted openings see and save as they always did. Nothing the AI learned below is theirs, so the yardstick doesn't move when the AI does.

*Before: the AI as committed, against all seven.* Matches won by the AI. Six seeds on each side of the map before, eight after.

| Opening | Starter deck, before | Starter deck, after | Henry's deck, before | Henry's deck, after |
|---|---|---|---|---|
| CLOCK | 12 of 12 | 16 of 16 | 12 of 12 | 16 of 16 |
| PLANTER | 12 of 12 | 16 of 16 | 11 of 12 | 15 of 16 |
| EXPANDER | 10 of 12 | 15 of 16 | 10 of 12 | 16 of 16 |
| SUMMONER | 12 of 12 | 16 of 16 | 12 of 12 | 16 of 16 |
| BOOM | 4 of 12 | 3 of 16 | 2 of 12 | 5 of 16 |
| RUSH | 1 of 12 | 11 of 16 | 0 of 12 | 16 of 16 |
| SAB | 11 of 12 | 16 of 16 | 10 of 12 | 16 of 16 |

Against RUSH the AI's king was slowed (its side beaten down to under 40 px) in 23 of the 24 matches before, between 1:00 and 1:47. After, in 13 of 32.

*What was built.*
1. **A king on the AI's half is a raid** (D1), wherever on that half it is. Its half is everything nearer its own king's start than the enemy's.
2. **Guns before miners against a raid that outweighs it** (D2). The raid's pixels against the pixels of every warship the AI has (or, while a wave is out, of those near home). Outweighed, its king prints the counter it can get for 100 px or less before anything else. Against a raid it outweighs, miners still come first, as before.
3. **Nothing extra held back under a raid** (D2). It used to keep 150 px on top of a factory's price. Now it keeps the 40 px it always keeps.
4. **Where it plants under a raid** (D2). Against a raid it outweighs, as on any day. Against one that outweighs it, never on a rock the raiders are within 200 px of, and only where the new factory joins its network; with no factory left, on any rock the raiders aren't at.
5. **What it prints against a king** (D2). A king raiding with ships round it is answered through the ships, by the old triangle. A king raiding alone counts as a big ship, so lancers.
6. **The king's post** (D3). While a raid is on, the king stands beside the factory nearest its start, 4 px clear of it on the side it came from. It doesn't take the post if that would put a factory out of its reach. With no raid on it's at its start.
7. **Standing and falling back** (D3). The king used to back away from any armed enemy within 90 px. Now it stays while it and the armed friends within 200 px of it outweigh the armed enemies within 200 px. Outweighed, it makes for the nearest of its factories with no enemy gun within 200 px. With none it backs straight away, as before.
8. **Miners count** (D4). A raid was an enemy warship within 200 px of the king or something planted. Now a miner counts too.
9. **Arming while outarmed** (D5). While the enemy's warships outweigh its own by a quarter (the edge it attacks with itself), its king prints a counter before what it's saving for a factory. It used to print one only with 120 px to spare.

*Each rule on its own.* The AI as built, and with one rule switched off, in matches won of 16. The columns that moved most are shown.

| Switched off | RUSH, starter | RUSH, Henry's | EXPANDER, starter | EXPANDER, Henry's | BOOM, starter | BOOM, Henry's |
|---|---|---|---|---|---|---|
| Nothing (as built) | 11 | 16 | 15 | 16 | 3 | 5 |
| 1. A king on its half is a raid | 5 | 0 | 15 | 16 | 3 | 5 |
| 2. Guns before miners | 5 | 7 | 15 | 15 | 4 | 4 |
| 6. The king's post | 8 | 13 | 13 | 13 | 4 | 4 |
| 7. Standing when not outweighed | 8 | 15 | 15 | 14 | 1 | 6 |
| 8. Miners count | 12 | 16 | 11 | 12 | 6 | 3 |
| 9. Arming while outarmed | 11 | 16 | 14 | 11 | 7 | 5 |
| 7. Falling back on a quiet factory | 11 | 16 | 15 | 16 | 3 | 5 |
| 4. Where it plants under a raid | 12 | 16 | 15 | 16 | 3 | 5 |
| 3. Nothing extra held back | 12 | 16 | 15 | 15 | 3 | 6 |

- **Rules 1 and 2 are what beat the rush.** Without either, the AI is back to losing it.
- **Rules 6 and 7 help against the rush** and a little elsewhere.
- **Rules 8 and 9 are what improved EXPANDER.** Rule 9 may cost something against BOOM on the starter deck (3 against 7), where guns bought early are pixels not mined. At 16 matches a cell, a difference of two or three is noise, and I wouldn't read that one as settled.
- **Three rules show nothing in these matches:** falling back on a quiet factory, where it plants under a raid, and the 150 px. They're kept on reasoning (the 150 px is what left the AI sitting on 183 px on seed 28100), and they're Henry's to drop.

*Tried and dropped.*
- **Holding posts against a raid that outweighs what's at home.** My proposal was that defenders should gather and fight together. Built as "stay on GUARD until those at home match the raid", it lost matches to the openings that stream ships: on the starter deck PLANTER went from 12 of 12 to 10, and EXPANDER from 10 to 7. With that one rule out they were back at 12 and 11. A ship on GUARD answers only what comes within 160 px of its post, so the raiders killed miners and far factories unopposed. Holding a post isn't fighting together.
- **Calling a wave home.** It only did anything through the holding rule, and without it a recalled ship goes straight back to hunting where it is. A real recall needs its own rule, and nothing measured says it would pay. Not built.
- **The king beside its factory all the time.** With the king 125 px from its start, its second home rock can be out of its network's reach. An existing test caught a factory cut off from the bank. So the post is for raids only, and not even then if it would cut a factory off.

*What this doesn't fix.* **The boom still beats the AI,** 24 times in 32. That was known (the pace section, above: "the boom beats the AI in every row"), and it isn't a defence problem. The boom out-builds it: the AI keeps one more miner than it has factories and saves for shipyards, while the boom prints miners without a pause and plants an outpost whenever it can pay. The AI neither punishes that early nor matches it. It's the next thing wrong with the AI, and it wants its own round.

*Found on the way.*
- **My first RUSH kept its king at home and lost.** Had I changed the AI against that, I'd have been fixing the wrong thing.
- **Half of what I proposed was dropped, and what mattered most wasn't on my list.** I proposed guns before miners, a garrison, gathering defenders and a recalled wave. The first is rule 2. The garrison is rule 9, which helps against EXPANDER and not against the rush. The other two are under "Tried and dropped". Seeing the king coming, rule 1, came from watching a match tick by tick.
- **A rule break found a clause that could never matter.** The post check let a factory count as kept if it would link through the first factory. Nothing that links through the first factory is ever out of the king's reach from a post 18 px away from it, so no test could tell the clause from its absence. It's gone.
- **The network protocol is 3.** A bot remembers one more thing, which is in the state the two machines compare. Two games a version apart now refuse each other at the start; they'd otherwise have fallen out of step at the first check.

*My calls, for Henry to overrule.*
1. **RUSH is my reading of seed 28100,** and of what a careful player does once the rush is answered (go home at three quarters).
2. **The AI still doesn't use SAB, a style or a group.** Henry's answer to that question was that its defence is what's killing it, so that's where this round went.
3. **The king's post is 4 px from its factory.** Near enough that its flak reaches what attacks the factory from close by. It also puts the king and the factory on one lancer's line.
4. **Three rules are kept without a measured effect** (above).

*What else moved.*
- **A bigger sample against RUSH:** 20 seeds on each side of the map. Before, the AI won 4 of 40 on the starter deck (2 drawn at the 40-minute limit, its king still running) and 1 of 40 on Henry's (1 drawn). After, 30 of 40 and 37 of 40.
- **AI against AI,** starter deck, four seeds: 2 to 2, from 15:37 to 24:21 with a median of 22:11. It was 3 to 1, from 17:11 to 24:56 with a median of 23:52.
- **The arena's reference fights** aren't touched: nothing in a ship changed.

*Checks (2026-11-12, after the AI's defence).* 164 tests pass, eleven of them new. 129 of 129 deliberate rule breaks are caught: the 103 from before (two of them re-pointed at lines this round rewrote) and 26 new ones. Among the new: no king taken for a raid, a king at home taken for one, miners first against a raid that outweighs the AI, a king answered before its ships, a post never taken, a post taken in peace, a post that cuts a factory off, a king that runs from everything, a king that runs from nothing, no falling back, falling back on a factory under attack, the 150 px back, a seed sent to the raiders, a factory planted out on its own, no arming while outarmed, arming against no fleet, miners not counted, a scripted opening given the AI's eyes, a rush that stays home, a rush that never goes home, a saboteur that hunts. Not done: I haven't watched any of it in a window, and nobody has played it. Every number here is the AI against scripts.

**Checks (2026-11-11, after N1):** 141 tests pass. 84 of 84 deliberate rule breaks are caught, twenty of them new. Among them: a command that skips the lag, a machine that plays on without the other's turn, each machine applying its own commands first, a check that compares nothing, a hash with the ships left out, a command counted whoever's name is on it, a turn let through out of order, the joiner's cards printing for blue, a network match that stops under a panel, a right-clicked card marking a job already queued, and the edge shoving a hulk. One was missed at first: the king's keys sent every frame change nothing in the match (the sim drops the repeats), only what crosses the wire, and no test had looked at the wire. One does now. The arena's 41 reference fights are exactly as last recorded.

**What to build before the part phase: my answer.** The order is ruled (Henry, 2026-11-10): "Fixes and pace quickly first before Network spike works for me." The rest is mine.
- **Agree on the network, as a spike and soon.** The match already runs on commands alone, the same on both sides, so the sockets are the small part. The two things worth finding out early are whether the king still feels right when its keys arrive a few ticks late, and whether two machines stay in step for a whole match. The first decides between plain lockstep and rollback, and rollback is a rule every new part would have to keep (all match state in the sim, cheap to copy and replay). Finding that out after twenty parts costs more than finding it out now.
- **A lag dial first.** Delaying the player's own commands by one to four ticks in a single-player match answers the feel question in an hour, with no network at all.
- **Then two windows:** connect by address, swap decks, play in step, compare a hash of the match every second. No matchmaking, ratings, servers or design database yet.
- **The AI splits in two.** A pilot that flies any deck sensibly, and a tournament tool round it, are what the part phase runs on: the miner tournament found a bad card in twenty minutes and corrected a wrong finding. Levels and a superhuman AI can wait.
- **Buildsets before the first new part.** A buildset is a list of parts and their numbers. With that in place, today's parts are the first set, a new part goes in a new one, and the simple game stays playable for good.
- **Campaign last.** It's the wrapper.

**For the next round, to talk through first** (my reading, not rulings):
1. **Timing attacks have to be able to win.** Henry wants a well-executed attack in the early-middle or middle game to decide some matches. This one was decided by economy at minute 13 and ended at 19. Before changing anything, measure it: can a side that commits everything at 5 to 8 minutes kill a king, or even a factory line, against an opponent of the same size? **On hold (Henry, 2026-11-10):** "before you make any changes around the timing attack balance, let me test out a timing attack. Furthermore, it may end up being a subtle balance question, in which case it's better left for late-stage tuning." **Ruled (Henry, 2026-11-10, after five more games):** "I think the current timing attack situation is well within balance range." Nothing to change, and nothing to measure.
2. **The AI and orders.** It masses on GUARD and sends waves on HUNT, but it never escorts, never recalls a wave to defend, and attacks on a clock and a head-count, not at a weakness. Henry's six switches each had a purpose.
3. **The AI's economy late on.** It needs to see the run-away coming (Henry's income was half as much again by 13:00) and answer with raids on miners or an attack, not with outposts it can't afford.
4. **Levels.** Easy, medium and hard at least, with hard beyond an unpractised Henry and medium a challenge; a superhuman one eventually. Today's AI is the first rung.
5. **Saving up with a factory that never stops:** withdrawn. It came from Henry's first miner, not from the rules (M2.14). With a miner that pays its way there's a second factory before 4:00.
5a. **A miner is nearly a solved drawing.** Two scoops bite as fast as six, so the best miner is the cheapest fast one and there's little to design. If mining should reward bigger miners, the rule to look at is the bite: one per scoop, say, instead of one per ship. A question, not a proposal: it would speed the economy again.
6. **The AI on the new deck.** It was tuned on round 2's cards. It beats the scripted openings 38 times in 40 (M2.14), and the three games it lost on the deck before show the habits that cost it: holding 150 px back under raid, and never reaching a shipyard. It has never been measured against a person on this deck.

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
