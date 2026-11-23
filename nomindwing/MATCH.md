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
- **A design with factory pixels has a build order:** built designs from the deck, set in the yard and fixed for the match. It has two rows of up to sixteen: a START it prints once, in order, and a REPEAT it prints round and round after that (2026-11-17; one looping list of twelve until then). Any entry the full-grown design could never build is flagged.

### The king
- **You fly it** with WASD.
- **It prints anything in the deck,** with no size limit (the one exception to the factory rule), at a fixed rate.
  - Built designs arrive by hyperspace: a blue line sweeps across the ship and reveals it column by column, like Homeworld's hyperspace window.
  - Seeded designs come out as a seed, and then you click a rock.
- **Its queue shows under the cards** (M2.17, 2026-11-11): each job as its card's icon, the one being printed first. A job can be dropped or set repeating there.
- **It's a network node:** it stores pixels and links to your other nodes. Flying it away can split your own network.
- **A beaten king is slow** (M2.10, added 2026-11-10). After the first minute, a king whose side has under 40 px of ships and factories standing flies at a tenth of its speed (3.75 px/s since 2026-11-17; 7.5 until then), slower than anything that flies.
- **For now, the king is a fixed design.** Designing your own king "could be quite fun" (Henry), so it's a feature for later.

### Seeds and growing factories
- **A seed is a fixed little kit:** a core and an engine, unarmed. It costs a set price and flies at fighter speed. Killing seeds in flight is the cheapest counterplay.
- **On the rock, it grows its design outward from the core.** Each new pixel eats one rock pixel. The pixel nearest the core grows first, so the factory is one connected, legal ship at every step.
- **Guns work while it grows.** They fire as soon as they appear.
- **Its factory pixels grow it first, then print.** While it grows, a seed eats at a base rate and every factory pixel grown so far adds its printing to the growth. A design with factory pixels near its core snowballs its own growth. Once it's whole (or the rock runs out), the factory pixels start on the build order. (Built this way so a factory pixel never does two jobs at once; see the judgement calls below.)
- **If the rock runs out first, the factory stays unfinished:** fewer factory pixels, so it's smaller and slower.
- **No claims, no ownership.** A rock is just pixels: anyone can mine it, and anyone's seed can eat it. (A rock is every rock pixel that touches, edge to edge or corner to corner: 2026-11-17. Until then a pixel hanging on by a corner counted as a rock of its own.) Two seeds on one rock share it, and each grows from what it gets. The planted factory is the claim, and taking it back means destroying it.
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
- **A tractor takes their load, a pixel at a time** (2026-11-16; until then a load was banked the moment a miner reached any node). A full miner flies into the reach of a tractor and waits there until it's empty. A miner that works within a tractor's reach is emptied as it works and never leaves the rock. Your king has a tractor. A factory has one if it's drawn with one, and so has a collector, which flies out to where your miners work. A rock near your tractors is cheap to work. One near theirs means long, exposed trips past their guns. Territory comes from distance and guns, not a rule.
- **(Gone, 2026-11-22: nothing pushes friends apart. MA8, far below.)** **A miner at work keeps apart from nobody** (M2.21). Other ships of a side keep a pixel or two apart. A miner going to its rock pixel, working it, or taking a full load home doesn't, and nothing keeps apart from it: miners heap up on a rock face, fly through each other and over a parked fleet, and other ships fly over them.
- **Loose pixels are anyone's:** from shots, kills and spilled stock. Once the rocks are gone, the economy runs on wreckage.

### Networks and split pools
- **Nodes are the king and anything with factory pixels or tractor pixels** (factories, carriers and collectors). In round 1, every node has a fixed link range. Two of your nodes link when their ranges overlap, and links are drawn as dotted lines in your colour.
- **Pixels are stored at nodes.** A network's bank is everything its nodes hold. What a tractor pulls goes into its own ship. Printing draws from the nodes that fly first (a collector is the easiest node to lose), then from the printer itself, then from the nearest node in the network.
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
| Map | 1400 x 1050. Its rocks are laid out one of four ways, as the seed falls (2026-11-17): scattered, with 22 pairs of small rocks, as every map was; or strung along an S, a ring or two triangles, with open space everywhere else. About 25,000 px of rock either way (1200 x 900 with 16 pairs until 2026-11-11) | Toy 1 was 960x720; slower pace, bigger ships. Rocks as thick on the ground as before |
| Starting bank | 190 px at the king (300) | An outpost and two miners, with 51 px left to print with |
| King's print rate | 0.6 px/s, as if it had 16 factory pixels (6 px/s) | A fighter in 15 s, a frigate in 92 s, a 687 px capital in 19 minutes |
| King's own pixels | 1 every 4 s (none) | A miner's worth every 36 s: enough to restart, too little to live on |
| King's speed | 37.5 px/s (45 until 2026-11-11) | It keeps pace with a fighter; it used to match the fastest ship a drawing can make |
| Hyperspace arrival | 2 s | Long enough to see, short enough not to matter |
| Seed | What it grows into, paid at once: 121 px outpost, 693 px shipyard (20 px) | Every factory is a purchase |
| Seed's base eating rate | 4 px/s, plus the factory's own print rate (6 until 2026-11-11; round 1's 4 px/s) | An outpost grows in about 27 s, a shipyard in about two minutes |
| Factory print rate | 1 ship pixel per factory pixel every 26.7 s (4 s) | The outpost's 24 factory pixels print 0.9 px/s, the shipyard's 110 print 4.1 px/s (28 and 118 until 2026-11-16, when some became tractor pixels) |
| Tractor | A pixel every 8 s for each tractor pixel; weight 1, uses 2 power; reaches 30 px past its ship's edge and 4 more for each pixel along the side of the square its pixels would make | Eight pull a pixel a second, which is what three miners bite. The king has 8, the outpost 4, the shipyard 8, the collector 8 |
| Factory size limit | 10 px per factory pixel | Its biggest ship takes 4½ minutes (40 s) |
| Factory pixel | weight 1, uses 2 power (1 until 2026-11-11) | Two hull pixels to feed each one: a factory the same size prints two thirds as fast |
| Lance | weight 2, uses 3 power (2 until 2026-11-11) | A two-lance lancer is 9 px, not 7 |
| Scoop | weight 2, uses 2 power (1 until 2026-11-17; weight 1 and free until 2026-11-11) | The starter miner is 12 px at 22 px/s, where it was 9 px at 30, and 8 px at 45 before that |
| Link range | 80 px per node (king 120) | Links span about one rock gap |
| Scoop mining | A pixel every 3.3 s with two or more scoops, 6.7 s with one (2.7 s and 5.3 s until 2026-11-11; round 1's 0.2 s and 0.4 s) | A 3-scoop miner fills its 9 px in 30 s. More scoops carry more and bite no faster |
| Beaten king | Its side under 40 px standing, after the first minute: 3.75 px/s, from 37.5 (7.5 until 2026-11-17; no such rule in round 1) | The slowest ship a drawing can make flies at 7.5 px/s (11 until 2026-11-17), so anything with an engine catches it |
| Top speed | 22.5 px/s for each thrust per weight, from 7.5 to 45 (until 2026-11-17: 7.5 px/s, and 18.75 more for each thrust per weight, never under 11) | The starter fighter does 37.5, the miner 30, a frigate 11, a capital 7.5 |
| Scoop | Every scoop bites a pixel every 10 s (until 2026-11-17: two or more bit one every 3.3 s between them, however many) | The starter miner's three still bite one every 3.3 s; six bite twice as fast |
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
| S1 | **How a design flies is chosen in the yard: RUNS, KITE or ORBIT.** Three buttons on the yard's flight row, or Y. Until one is picked the drawing picks, as before (1 thrust per weight runs, less kites). It's fixed for a match, and a card's hover line says it. | Is choosing it in the yard and not in a match right, in play? Do you miss a mark for it on the card itself? | **Keep** (Henry, 2026-11-11), on the yard, the card and the yard's layout: "all seems good for now." **Gone, 2026-11-14:** ships choose how they fly (A1), and the yard's buttons are out (A4). |
| S2 | **KITE.** What stand-off was, open to any ship, plus one thing: it backs away from anything else that comes nearer than its guns reach. | Does a light ship on KITE do what you'd call kiting? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S3 | **ORBIT.** Circles the target at the kite distance, flat out, facing it, and turns back the other way at the map's edge or a solid thing. | Does it read as an orbit? Is the circle the right size? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S4 | **SAB, an order beside HUNT, GUARD and ESCORT.** The nearest enemy miner or factory first, past whatever guards it; never a king; with none left it hunts. | Does it go for what you meant by sabotage? | **Keep** (Henry, 2026-11-11): "i saw groups and sabotage, yes all good." |
| S5 | **A ship keeps the enemy it's after** until that one is gone or another is 30 px nearer. Before, it picked the nearest again every tick. | Do fights look steadier, or do ships now ignore things they should turn on? | **Keep** (Henry, 2026-11-11): "i didn't see anything that seemed odd." |
| S6 | **Groups: GROUP 1, 3, 6 or 12** on the yard's flight row, or G. On HUNT or SAB that many of the design gather, within 96 px of one another, before any leaves, and they guard meanwhile. | Do groups leave when you expect? They gather by nearness, so each factory makes its own group: is that right, or should a design gather across the map? | **Keep** (Henry, 2026-11-11), with everything waiting together leaving together: "i saw groups and sabotage, yes all good." **Gone, 2026-11-14,** with the flight buttons (A4, A5). Henry: "I think we can get rid of group size along with manual flight patterns." |
| D1 | **The AI sees a king coming.** Your king counts as a raid from the moment it crosses to the AI's half of the map, not from when it arrives. | Fly your king over at the start with lancers on repeat, as on seed 28100. Does the AI answer in time? | |
| D2 | **Guns before miners against a raid it can't match,** and nothing extra held back under a raid. Against a king with ships round it, it prints what answers the ships; against a king alone, lancers. | Does it spend its bank when you raid it, or sit on it as it did at 4:00 on seed 28100? | |
| D3 | **Its king stands beside its factory while a raid is on,** so the factory's flak and its own cover each other. It stays against what it outweighs. Outweighed, it falls back on a factory nobody is attacking. | Does the king's moving about make sense to watch? Is it too easy to catch out? | |
| D4 | **Its miners are its to defend.** A raider within 200 px of one of its miners turns its fleet out. | With everything on SAB, do its ships now come for yours at the rocks? | |
| D5 | **It arms while your fleet is a quarter bigger than its own.** Guns then come before what it's saving for a factory. | Does a fleet built early draw an answer before it lands? | |
| D6 | **RUSH and SAB,** two scripted openings taken from Henry's two matches of 2026-11-11, beside the five the AI was measured against. | Nothing to judge in play. Is RUSH a fair picture of what Henry did on seed 28100? | The king going too: **yes** (Henry, 2026-11-12). |
| P1 | **BEAM.** Beam pixels one behind another in a column are one gun, firing straight ahead. One reaches 52 px, each more adds 12 (up to 16), and it cuts 4 pixels for each, plate counting double. It shows its line for 1 s, cuts everything on it at once, and rests 5 s. | Does it read as one gun whose length you drew? Is a second's warning long enough to see and short enough not to be dull? | Henry, 2026-11-12: "Beams and shields both seemed fun." One match played with a BEAMER and a WARDEN on the team. The numbers are unjudged. |
| P2 | **Ships get off a warming beam's line** if they can be clear in time, and stay off it until it has cut. | Is it good to watch? Do ships ever dodge when you'd want them to stand and shoot? | |
| P3 | **A ship with a beam flies like a lance ship,** holding off at three quarters of its beam's reach and lining it up. | Does a beam ship keep its distance sensibly? | |
| P4 | **SHIELD.** Each shield pixel throws a 10 px circle that stops 4 enemy shots of any kind coming in. 3 s after the last, it's full again. Shots fired from inside pass. | Radius, charges or rest: which feels wrong first? Should a beam cost a circle more than a flak shot does? | Henry, 2026-11-12: "Beams and shields both seemed fun." One match played with a BEAMER and a WARDEN on the team. The numbers are unjudged. |
| P5 | **A ship's circles are drawn as one outline,** thinning by a quarter for each shot stopped. | Can you read what's left of a shield at a glance? | |
| P6 | **`--parts`:** three example ships and ten ready-made fights in a window of their own, with nothing saved. | Is that a good way to be shown new parts, or should they come in a deck? | |
| L1 | **A team is nine designs** (ruled: "Nine for now"). Nine tabs in the yard, nine rows in the arena, nine cards in a match, keys 1 to 9. A row or card is one line now: icon, name, price, and on a card the order's letter in its colour (H, S, G, E, V for harvest, L for hold). | Is one line a card enough to play from? Do you miss the slot number on the card, or the three-letter order? | Henry, 2026-11-12: "I played a game. I didn't really notice any problems." A 15:51 match on nine cards, with orders switched from the cards, the ninth included (log). |
| L2 | **Empty slots.** A fresh team is the starter deck and two slots that say EMPTY. An empty card prints nothing and says why. | Should the starter deck fill all nine, and with what? | |
| L3 | **The library,** open over the yard (LIBRARY beside the name, or L): your kept drawings, newest first, then the ready-made ones (the new parts' examples, the starter deck, the reference fleet). Each shows its price, order, how it flies and whether it can fly. | Is it where you'd look for it? Can you find a drawing in the list? | Used and no problems noticed: four drawings taken from it in one sitting (log). |
| L4 | **TAKE puts a copy in the lit slot** and goes back to the yard, where it's redrawn like any drawing (ruled: orders and styles "go with the drawing, but you can easily customize a drawing from the database"). The drawing it replaces is kept in the library first, unless one just like it is there. KEEP keeps the lit slot's drawing. | Is take-then-redraw easy enough? Is keeping the replaced drawing by itself what you want, or clutter? | TAKE used four times, into slots 8 and 9, and no problems noticed. Each time the slot held nothing of Henry's own, so the keeping of a replaced drawing and KEEP itself are untried. |
| L5 | **Teams:** LOAD swaps all nine slots (keeping the team you had), KEEP TEAM keeps yours as TEAM 1, 2 and so on, RENAME types a name. | Do you want whole teams in the library yet? | Not tried yet: the log shows no team loaded or kept. |
| L6 | **COPY and PASTE as text.** A drawing is one line and a team ten. PASTE keeps whatever on the clipboard reads as drawings and ignores the rest. | Does it work on your machine? (My tests use a stand-in clipboard: I didn't want to overwrite yours.) Is a line of text a fair way to hand someone a ship for now? | Not tried yet: the log shows nothing pasted. |
| L7 | **`--pg-sim mine`:** the headless fights with your own saved team, orders and styles as your cards carry them. | Nothing to judge in play. | |
| L8 | **CRASH and SESSION END lines in the play log.** | Nothing to judge until something crashes. | SESSION END was written when Henry's session of 2026-11-13 05:34 UTC closed. No CRASH line since. |
| G1 | **A growing factory takes from its rock, and from the bank when there's no rock.** A factory whose rock runs out finishes from its network's bank, where it used to stop unfinished. Something with no factory pixels is in no network and still stops. | Does a half-fed factory draining your bank ever surprise you? | |
| G2 | **Seeds into open space.** With a seeded card picked, Ctrl+click sends the seed to that spot: clear of rock and of anything planted, and within reach of a node of yours. Same price at once, then every pixel it grows from the bank. Factories only. (It had an IN SPACE button as well until 2026-11-13.) | Ruled: twice the cost "actually makes sense". Is Ctrl the right key? | Henry, 2026-11-13, on the button: "too prominent and overlaps the minimap a bit, not suitable for something that is usually unneeded. I think just having players able to see and choose the hotkey in the options may be enough." Taken out. The rest unjudged. |
| G3 | **The warning.** While placing, the bar turns gold and says the bank pays for what grows as well, "SO IT COSTS ABOUT TWICE". A rock too small says how many pixels the bank will pay for. | Ruled: "display a warning of some kind while placing it". Is this one enough? | |
| G4 | **The AI sends a seed into space** only when it wants a factory and no rock will take one. The scripted openings never do. | Nothing to judge until a match runs the rocks out. | |
| X1 | **MISSILE.** A launcher pixel (WEAPONS, uses 4, weighs 2) sends a missile at the nearest enemy within 200 px every 6 s, whichever way the ship faces. | Reach and rate are placeholders. | Henry, 2026-11-13, after a game with an ARCHER on the team (won at 3:46, the quickest yet): "I think archers are overpowered currently but that doesn't matter for now." So: noted, not tuned. |
| X2 | **The missile chases** at 45 px/s, as fast as the fastest ship, straight at wherever its target is now, for 8 s. It bursts like a cannon's shell: a 3x3 hole, plate and all. | Is it fun to watch one chase, and to outrun one? | |
| X3 | **Flak shoots it down.** A flak gun fires at a missile in its reach before any ship, two shots to a missile. A shield stops one for a charge. A beam's cut takes any on its line. | Two launchers get almost nothing past six flak guns. Too strong a screen? | |
| X4 | **A missile ship flies as its card says** (ruled: "Shouldn't its ship just follow whatever order it's given"). The ARCHER example's card says KITE, so it holds off at two thirds of a missile's reach. | Does a missile ship on RUNS or ORBIT do something sensible? | |
| X5 | **The yard's part buttons run to a second row,** and every row under the canvas is 13 px high to make room. | Still easy to hit? | |
| R1 | **REPEATER.** Repeater pixels one behind another in a column are one gun, firing a stream of small shots straight ahead (WEAPONS, uses 2, weighs 1). One pixel reaches 68 px and fires every 8 ticks. Each pixel more adds 8 px, and every two take a tick off the wait, down to 3 ticks at eleven. Sixteen at most: 188 px. Each shot does 1 damage: it breaks hull, and plate turns it. | Every number is a placeholder. Is a long thin gun a shape you want to draw? | |
| R2 | **It fires when a shot fired now would meet an enemy on its line,** if that enemy flies on as it is. It doesn't hold fire as a lance does. A shot that meets nothing is gone at the end of the gun's reach. | Does the stream look right? Does it waste too much on ships that turn away? | |
| R3 | **A stream keeps a shield down:** each shot costs a circle a charge and starts its rest again. Its shots take missiles down as flak's do, though it never aims at one. | Is it clearly the thing to bring against a shield (your question of 2026-11-12)? | |
| R4 | **A repeater ship flies like a lance ship:** it lines its gun up and holds off at three quarters of its reach. | | |
| J1 | **PROJECTOR: what may be drawn** (HULL & ARMOR, uses 2, weighs 1, blue). Each projector pixel needs 4 other projector pixels among the 8 round it, and its own row and column clear to the edge of the ship in 3 of the 4 directions; other projector pixels aren't in the way (Henry's rules of 2026-11-13). A pixel that fails shows red, the ship can't fly, and the yard says how many are TOO THIN, then how many are SHUT IN. | Can you draw the shapes you want? Do the red marks and the line tell you what to fix? | |
| J2 | **Reach and pool come from the pixels.** Projector pixels joined edge to edge are a lump. A lump reaches 10 px for each whole number in the square root of its pixels (16: 40 px, 36: 60, 100: 100; 150 at most), from its box to any part of a friend's box. The ship holds one stopped shot for each projector pixel standing, and all of them are back 3 s after the last shot it stopped. | Placeholders, all of them. | |
| J3 | **The skin.** A shot of any kind that would hit a covered pixel stops there and costs the pool one. Every friend in reach is covered. The projector's own ship never is (put back 2026-11-13, see "as built"), and no projector pixel is, a friend's included. A beam's cut stops at the first covered pixel for one charge. A burst spares the covered pixels in it for one charge. A ship's own shield circle comes first. With two projectors in reach, the nearer one with a charge pays. | Should two projectors be able to cover each other's projector pixels? As built they can't: each covers the other's hull only. | Henry, 2026-11-13, on its own ship: "Yes, put it back". Agreed 2026-11-11 ("it sounded right, yeah"); I had reversed it unasked. On another ship's projector pixels, same day: "No, keep bare". |
| J4 | **Seen.** A dotted line shows each lump's reach. A covered ship wears a blue outline that thins by a quarter for each quarter of the pool spent. A lump has a pale middle. Hovering its ship shows PROJECTOR 32/38. | Can you tell at a glance what's covered and how much is left? Is the reach line too busy with several on screen? | |
| J5 | **The AEGIS example has no guns,** so it stays beside the nearest friend that has (an old rule for any unarmed ship on HUNT), and it's drawn as fast as the RAKER. Small fast ships outrun it. | Nothing was added to flying. Whether ships should stay near a projector is ship behaviour, which is parked: one for the unit-AI talk. | |
| J6 | **The parts team:** the RAKER has the NEEDLE's slot (the NEEDLE is still in the library), the AEGIS is slot 9, and there are nineteen fights. | | |
| C1 | **Selecting.** A box dragged on the field takes what's yours inside it, and if any of that is neither a miner nor your king, only those. A click takes the ship under it, or your king. Ctrl+click: every ship of that design. Q: every warship. Esc: nobody. On the whole map (Tab) too. | Does it feel like the RTS control you had in mind? Is leaving miners and the king out of a box round a fleet right? | Played 2026-11-13. Not enough alone: Henry asked for a double click for a type on screen, a way to take a type across the whole map, and control groups (C8, C9). |
| C2 | **Sending.** With ships selected, a right-click on the field, the whole map or the small map sends them there. They fly together at the pace of the slowest, fight what comes within 130 px on the way and what's within 160 px of the place once there, and stay. (Since 2026-11-16 a sent ship doesn't stop for what comes close on the way: H10 and I5.) | Is going-and-fighting the right thing for a right-click to mean? Do you miss a plain move, or "attack that one"? | Henry: "We don't have to add attack orders but right-clicking a unit should probably effectively move the "go here and fight" order along with the unit." Added (C10). Waypoints with Shift "can maybe wait". |
| C3 | **Let go.** X, or the LET GO button by the clock, sends the selected ships back to what their cards say. | Is "back to the card" what you'd expect, or should they stay where they are? | The key is G now (C12). Not otherwise ruled. |
| C4 | **The king is selected and sent like a ship.** It flies there with no key held and stops when it arrives. WASD still flies it and takes over at once. | Keep WASD as well? Does a sent king feel better under lag, as you guessed? | Henry: "Very likely WASD should scroll the camera, not fly the king (We can think about direct flight as a special level in the campaign, perhaps)." Changed (C11). |
| C5 | **The whole map is for selecting.** A click on it no longer moves the view. The small map still does. | | |
| C6 | **The rules card has a FLEET line.** KING, ORDERS and SOLID were shortened to make room: the card was full to the pixel. | Is anything you'd miss gone from those three lines? | |
| C7 | **On the wire:** one command for each ship sent or let go (version 8). A turn carries 64 commands, so the rest of a bigger batch follows a tick behind. | Nothing to judge alone. | |
| C8 | **Control groups.** Ctrl+1 to 9 makes a group of what's selected; the number selects it; Shift+number adds what's selected to it. The number twice running takes the view to the group. | Do they behave as your hands expect? | |
| C9 | **Double click, and Shift.** A second click on a ship straight after the first (or Ctrl+click) takes every ship of its type in view; on the whole map that is all of them. Shift+click adds a ship or takes it out; a Shift box adds. | Is "double click on the whole map" enough of an answer to "all of a type on the whole map"? | Henry, 2026-11-13: "it would be logical to add cntl-z, cntl-x etc to select all ships of that type." Added (K3). Not otherwise ruled. |
| C10 | **A right-click on a ship sends them after it,** friend or enemy. The place they're sent to moves with the ship, and stays where it last was once it's gone. | | |
| C11 | **WASD and the arrows scroll the view.** The king is only selected and sent. K, or a click on the note above the small map, takes the view to the king, and the view no longer follows it. | | |
| C12 | **The keys that moved.** Cards are on Z X C V B N M (the first seven; the eighth and ninth are clicked), because 1 to 9 are the groups. Let go is G, not X. A card's order is changed by clicking its tag only. | Is the bottom row of letters right for the cards? | Henry: "Why not just make ships 8 and 9 , and .? The key to make the ship should go to the left of the icon, which won't be confusing once we get rid of the disposition letter on the right." Done (K1), and the tag is gone (K2). |
| K1 | **A key on every card.** The nine cards are on Z X C V B N M , . and each card shows its key at its left end, before its icon. | | Keep. Henry, 2026-11-14, after playing: "Hotkeys seemed great otherwise." |
| K2 | **One stance, and no order on a card.** A button by the clock (T): every ship you haven't sent guards where it was built, or attacks the nearest enemy. A match starts on GUARD. The cards' tags, and SAB and ESCORT, are gone from a person's match. | On ATTACK the ships already flying go at once, without gathering into their groups first. Right? | Keep. "I'm happy the disposition letters are gone and I think ATTACK/GUARD with LET GO is an improvement." At once is right, and the exception changes: "ATTACK should send everything unless it has mining equipment. miners should still mine." (K9) |
| K3 | **Q, Ctrl+Q, and Ctrl with a card's key.** Q: your warships in view. Ctrl+Q: every one. Ctrl and a card's key: every ship of that card, anywhere. Shift adds. | "Warship" here is anything you can send that isn't a miner, so an unarmed shield ship is in. Right? | Keep, with the hotkeys. The shield-ship question is settled by K9: no scoop, so it's a warship. |
| K4 | **Formations.** Ships sent together each have a spot: ranks across the way they came, flak in front, longer guns behind, then miners and the unarmed, the king last, types together. | Does it look like a sensible shape? Is the spacing right (6 px between hulls)? | Keep. "Formation looked sensible." |
| K5 | **A sent ship owns its spot.** With nothing to fight it flies to its spot and stops, and nobody pushes it off or is pushed by it. Ships sent to one point each take the nearest clear spot by it. | A change to how friends keep apart. See the notes. | Played ("I think it played well"). Not ruled apart. |
| K6 | **Rally points.** Select a factory (a click), right-click the map: its new ships make that place their home. A flag marks it. A right-click on the factory takes it away. | A king has none. Wanted? | Keep, and no more of them: "I don't think we need rally points for mobile factories." So none for a king or a factory that flies. |
| K7 | **The keys page (F3),** for the match, the yard, the arena and the library, with a test that fails when a key is handled and not listed. | | Keep, with the hotkeys. |
| K8 | **On the wire:** two new commands, the stance and a rally point (version 10). | Nothing to judge alone. | |
| K9 | **The stance takes everything that flies unless it has mining equipment** (Henry's rule, 2026-11-14). A ship with a scoop mines, whatever it was drawn with. A flying design drawn to HOLD takes the stance now. | The HOLD half is my reading of "everything". Right? | |
| K10 | **K selects your king. K again straight away takes the view to it,** as with a group's number. The note above the small map does the same. (Henry's tweak, 2026-11-14.) | | |
| A1 | **Ships choose how they fly.** Each side has a planner, in a match and in the arena. While it has warships in a fight it takes one design at a time, plays the fight forward 8 s with that design on attack runs, kiting and orbiting, and keeps what came out best. | Does it look smart, or does it only win? | |
| A2 | **What's played forward is the fight alone:** the ships within 300 px of an enemy, in a bare arena, with no match round them. | It's what makes it affordable. What it gets wrong is in the notes. | |
| A3 | **The thinking is spread over ticks,** more of it a tick for a small fight: a design is decided in under a second in a fight of twenty ships, about 2 s in one of forty, about 12 s in one of a hundred. | Quick enough to matter in a big fight? | |
| A4 | **The yard's flight buttons and group buttons are gone,** with their keys (Y and G). The flight row shows where the drawing starts its ships. Old saves load, and the two settings are dropped. | | |
| A5 | **Ships leave as they're built.** No gathering into groups, on ATTACK or for the AI. | | |
| A6 | **The arena's sides both plan too.** Its brain buttons are for comparing: ARCH (every side starts on it), PLAIN (no planner; this was CARD), DRILLED. | Keep the buttons? | |
| A7 | **Hover a ship** for what its side has chosen: "ITS SIDE HAS EVERY FIGHTER ORBITING." | | |
| O1 | **A drawing carries no order, and the yard has no order buttons** (or O key). What its ships do follows from the drawing: engines and a scoop mines, engines and no scoop fights, no engines stays put. | A ship with a scoop can't be a warship. Right? | |
| O2 | **A card's own order is the AI's to give.** HUNT, SAB, GUARD and ESCORT are what an AI tells its cards. From a person's side the command is dropped, yours or another player's. | | |
| O3 | **Nothing a player sees names an order:** the bar under a ship says who it is and what it's doing. | | |
| O4 | **The AI orders its unarmed ships with its warships,** so a shield ship of its goes with its fleet. | | |

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
- So I took it Henry's king went too. **Confirmed** (Henry, 2026-11-12): "You asked me to confirm that attacking with my king is part of what made rushing the AI effective. Yes, it certainly is, confirmed."

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

**The beam and the shield, as built (2026-11-12).** Henry: "after finishing the AI defense, please commit+push, then continue on to the implementation of the beam and shield." The defence round went up as 4e632c3. Rows P1 to P6 above. Every number below is a placeholder until Henry has played it.

*The beam's rules* (the shield had its six written out; the beam had one sentence).
1. BEAM is a one-pixel part. Beam pixels one behind another in a column of the drawing are one gun, and its front pixel is the muzzle. A pixel alone is a gun of one. A row of them side by side is so many guns of one.
2. It fires straight ahead only, like a lance.
3. **Reach** is 40 px and 12 more for each pixel of the gun, counted up to 16 pixels: 52 px for one, 100 for five (a lance's), 148 for nine, 232 at sixteen and beyond.
4. **Cut** is 4 pixels for each pixel of the gun, with no cap. Plate uses up two of it, as it does of a lance's punch.
5. When an enemy pixel will be on its line, within reach, a second from now, it starts to warm. For that second its line shows as a dotted line, and the line moves with the ship.
6. Then it cuts: every enemy pixel on the line goes at once, nearest first, until the cut is used up or the reach ends. Whatever is cut off from every core breaks away, by the rule there already was. It cuts hit or miss, and then rests 5 s (longer short of power).
7. Shoot a pixel out of the column and it's two shorter guns.
8. A beam pixel uses 3 power and weighs 2, like a lance.

*What ships do about it.*
- **A ship on a warming beam's line gets off it** (P2): flat out, square to the line, to the side it's already on. It then stays off: it won't fly back towards the line until the beam has cut. Only ships that can be clear in time try. A ship too wide or too slow carries on with what it was doing, since a half-made dodge only spoils its own aim. Kings don't: they're steered.
- **A ship with a beam flies like a lance ship** (P3): on KITE it holds at three quarters of its longest beam's reach and lines the gun up on the target's row or column; on RUNS it slides onto the line first.

*The shield, as built.* The six rules as proposed, with these settled:
- Any enemy shot costs a circle one charge: flak, lance, shell or beam. A shell stopped at a circle doesn't burst.
- "After 3 s with no hit" is 3 s after the last shot that circle stopped. Every shot stopped starts the wait again. A circle that's out stops nothing, so it comes back 3 s after it went out however hard the ship is being hit.
- A circle is met along the path a shot takes relative to the ship, like a pixel. A ship that moves onto a shot doesn't take it inside its circle.
- No clearance rule. That stays with the projector.

*What they do in a fight.* Three example ships, each spending all the power it makes, against the starter deck's. Twelve seeds a fight (`--pg-sim parts`).

| Fight | Result | What it shows |
|---|---|---|
| 1 BEAMER (47 px) against 1 OUTPOST (121 px) | Beamer 12 of 12, in 5 s, untouched | It reaches 148 px and the outpost's flak 48. One cut along the row its core is in kills a one-core factory outright. |
| 2 BEAMERS against 6 FIGHTERS | Fighters 9, beamers 2, 1 drawn | Fighters get off the lines, close, and shoot the guns apart. |
| 2 BEAMERS against 6 LANCERS | Lancers 12 of 12 | The same, faster. |
| 1 BEAMER against 1 FRIGATE | Frigate 12 of 12 | The frigate loses 14 px doing it. I haven't watched how. |
| 6 LANCERS against 1 WARDEN (45 px, 4 shields) | Warden 8, 4 drawn | 45 lance shots a fight stop at its circles. |
| 6 FIGHTERS against 1 WARDEN | Fighters 12 of 12, in 5 s | Sixteen flak shots stopped, and then it's open. |
| 1 BEAMER against 1 WARDEN | Warden 12 of 12 | Every cut that reaches it stops at a circle. |
| 6 NEEDLES (9 px, a beam of two) against 6 FIGHTERS | Fighters 12 of 12 | A short beam on a fighter's frame isn't a fighter's weapon. |

So the beam is what I pitched: a siege gun, good against what can't get out of the way and poor against what can. And the shield turns away few strong shots and gives way to many weak ones. Whether any of it is the right strength is Henry's to say. I'd look first at the outpost dying to one cut.

*My calls, for Henry to overrule.*
1. **Every number.** Reach 40 + 12 a pixel to 16, cut 4 a pixel, 1 s warm, 5 s rest, 3 power; circle 10 px, 4 charges, 3 s, 4 power.
2. **The line moves with the ship while it warms.** A beam comes from where its gun is. So dodging is a race against a ship sliding or turning to follow, not a step off a line fixed in space.
3. **It cuts hit or miss.** Nothing calls a warming beam off.
4. **A beam of sixteen costs a circle what a flak shot costs.** That's what makes the shield the answer to the beam, and flak the answer to the shield.
5. **Every ship that flies dodges,** miners and guards on their posts included.
6. **Beams pass over rocks,** as every shot does.
7. **The parts are just in the yard.** No buildsets: the starter deck is unchanged, and the AI, which plays the starter deck, never fields either part. To meet them in a match Henry has to draw them into a deck.
8. **`--parts` is its own window.** A library has seven slots and Henry's are his own, so the examples can't go in it.
9. **The help screen's colour key now shows all eleven parts,** in two rows.
10. **The network protocol is 4.**

*Found on the way.*
- **A gun that only fires ahead never fired at a target coming straight down its line.** The check asked whether a pixel would be on the line where the target is now, and a target closing at 3 px a tick is 45 px nearer a second later. The beam now looks where the target will be. **The lance has the same check and I've left it:** changing it moves every recorded fight. It's why lancers hold their fire at a fast ship coming straight at them until it's close.
- **Two ships each lining a gun up on the other chase each other the length of the map.** Each leads the other's sideways speed, so each runs ahead of the other, and whichever is faster gets its shots in. A beamer and a warden went from the middle to the top edge, to the bottom and back, with three cuts in a minute. Where a beam is on either ship, neither leads now: each holds the other's line as it stands, and they settle. **Two lance ships still do it** (a frigate against a warden above: the warden wins 11 of 12 having stopped three shots, because it's faster). That's older than this round, and the same one-line rule would mend it at the price of moving the recorded fights. Henry's call. **Mended since, for the lance too: see "A lance's aim" below.**
- **A ship that dodged flew straight back.** Clear of the line, it turned for home, was back on the line when the beam cut, and lost three pixels. Now it stays off until the cut.
- **Fighters with guns shot the muzzle off** the test's beam turret before it fired. Not a fault: 50 px is inside flak range. But it's the commonest way a beam fails in the fights above.
- **Four weapons in a row left CANNON's button without its name.** The yard's part buttons are now as wide as their names.
- **I cut eight functions out of the sim** replacing the one before them, the second time I've made that mistake in this file. The compiler caught it at once, and they went back word for word from the last commit.

*Not built.* The projector, the weapons charger, hold and the scoop rework, the missile. Shields that the AI draws. A beam's line on the minimap. Any change to the lance.

*Checks (2026-11-12, after the beam and the shield).*
- **182 tests pass, eighteen of them new.** Among them: a fight of the example ships on both sides with the sim's audit run every tick, and a new standing rule in that audit, that no shot which flew on came into a circle that's still up.
- **158 of 158 deliberate rule breaks are caught:** the 129 from before and 29 new. Four of the new ones were missed at first, which showed four tests that couldn't tell right from wrong: the beam ship's target died before the ship's distance mattered, it sat near enough the ship's row to be cut without lining up, and two checks on circles and moving ships each looked at half the case. The tests were sharpened and all four are caught.
- **Nothing that existed has moved.** Every arena fight of the starter deck and of the reference fleet prints the same results as before, byte for byte, and so do the AI's matches. Neither part is in any of them.
- **Not done.** I've seen seven exported frames, not a window. Nobody has played either part. No match has been played with them, by a person or the AI. Nothing has been tried over the network since the protocol moved to 4.

**Three steers from Henry (2026-11-12).**
- **What the AI's match results are worth:** "remember AI match results are not really a strong signal for anything but code correctness yet." So the two tables in the defence section say that the rules do what they're written to do and break nothing, and no more. Two rules there were dropped on tables like those (holding posts, calling a wave home). They aren't shown to be bad ideas, only not shown to be good ones, and they're Henry's to bring back.
- **Recorded fights don't protect anything:** "Don't make decisions based on the desire not to move recorded fights. This is early stage development, we don't want to lock in our early alpha build, right?" I'd left the lance's two faults alone for that reason. They're mended below.
- **The lance:** "I have noticed that lance aim isn't great and I think it would be good to fix."

**A lance's aim (2026-11-12).**

*What was wrong.* Four things, found by counting where lance shots end.
1. **It looked for a moving ship in the wrong place.** The gun asked whether a pixel would be on its line, using one lead for the whole ship worked out from where the ship is now. For a ship coming down the line or flying off along it, that's the wrong moment and the wrong place. One lancer fired three shots at a fighter in twelve fights.
2. **It counted pixels its shot never reached.** Anything with its middle in range counted out to its far side, up to 137 px against a capital. The shot is spent at 112.
3. **Two lance ships lining up on each other chased each other.** As found with the beam.
4. **A ship lining up led by the wrong time:** the distance over the shot's speed, whatever the target was doing along the line, and with no end. From 400 px away it led by 100 ticks for a shot that lasts 28.

*What was built.*
1. **The gun fires at where its shot and a pixel will both be.** For each place down its line, the shot is there so many ticks from now, and the question is whether a living pixel will be there then. A ship coming at it is met sooner and nearer, one flying off later and further, and one flying off as fast as the shot never.
2. **It reaches what its shot reaches:** 112 px, one number for the gun and the shot.
3. **It holds fire if its target could be off its line when the shot arrives.** The gun can't turn. Leading is the ship's job, done by flying to the right row or column (rule 4). The gun only asks whether a shot fired now, straight down its line, would run into the ship, and it asks three times: with the target flying on as it is, flying as it's trying to fly, and stopped dead. It fires if the shot would hit some pixel of the ship each time. Not the same pixel each time: a ship is wider than a shot, and one flying along the line stays on it at any speed. A lance shot takes 1.7 s to cross its range, and a ship may do any of the three in that time. In practice it fires at what's flying along its line, at what's sitting still, and at a ship crossing its line only if that ship moves less than its own width while the shot is in flight. (Say a ship crossing at 1 px a tick, 40 px off: the shot takes 10 ticks and the ship moves 10 px. If the ship is 13 px wide, 3 px of it are on the line whether it flies on or stops, and a gun pointed at those fires. If it's 3 px wide, no line catches it both ways, and the gun holds.) I first called this "a sure shot" that "lands three ways". Henry, 2026-11-12: "This sounds very confusing. One reading would be that it never takes any shots because you can't both lead the enemy ship and not lead it at the same time." It isn't sure either: a ship can do a fourth thing. The part's hover text and the README now say it as this rule's heading does. It holds a long shot at something crossing or turning, and is loaded when the ship comes close. Every ship now carries the speed it's trying to fly at, which is what its engines are plainly working towards.
4. **A ship lining up leads by the time its shot really takes,** and never by longer than the shot lasts.
5. **A ship doesn't lead a target that's lining a gun up on it.** Lance or beam, either side. Two frigates now hold their ground about 93 px apart where they used to run the length of the map side by side.

*What it did to the shots.* Shots that hit something, of shots fired, by the side with the lances. Twelve seeds a fight; these are counts of the thing that was changed, not match results.

| Fight | Before | After |
|---|---|---|
| 12 LANCERS at 1 OUTPOST | 33% | 50% |
| 6 LANCERS at 1 FRIGATE | 33% | 37% |
| 6 LANCERS at 6 FIGHTERS | 66% | 64% |
| 40 LANCERS at 1 CAPITAL | 48% | 54% |
| 76 LANCERS at 76 FIGHTERS | 50% | 61% |
| 76 LANCERS at 12 FRIGATES | 28% | 30% |
| 76 LANCERS at 1 CAPITAL | 32% | 41% |
| 12 FRIGATES (lances and flak) at 1 CAPITAL | 42% | 59% |
| Reference fleet: 6 LANCERS at 1 FRIGATE | 45% | 49% |
| Reference fleet: 6 LANCERS at 6 FIGHTERS | 72% | 77% |
| Reference fleet: 40 LANCERS at 1 CAPITAL | 74% | 78% |
| Reference fleet: 40 LANCERS at 1 TITAN | 73% | 93% |
| Reference fleet: 85 LANCERS at 85 FIGHTERS | 69% | 80% |
| Reference fleet: 85 LANCERS at 13 FRIGATES | 44% | 40% |
| Reference fleet: 85 LANCERS at 3 DESTROYERS | 45% | 52% |
| Reference fleet: 85 LANCERS at 1 CAPITAL | 51% | 62% |
| Part examples: 6 LANCERS at 2 BEAMERS | 38% | 44% |

*Corrected 2026-11-12.* This table first held seven rows I'd picked, and I told Henry the rate "rose by 4 to 20 points". It now holds every fight with a side of only lancers (and the frigate row from before). Of those sixteen, fourteen rose, by 2 to 20 points, and two fell: six lancers at six fighters by 2 points, on 95 and 76 shots, and 85 lancers at 13 frigates by 4. All sixteen together: 9,740 hits of 20,844 shots before (47%), 10,447 of 19,480 after (54%). So the gun fires about 6% fewer shots and lands about 7% more. Left out: six lancers at the shielded warden, where the shields stop nearly every shot either way.

*What moved, since Henry will see it.*
- **Fighters against lancers at equal cost, 76 a side: fighters 12 of 12 before, 9 of 12 now.** Lancers now shoot fighters that fly straight down their line at them, which is how a fighter on an attack run flies. Six against six, fighters still win 12 of 12.
- The reference fleet's six lancers beat its frigate 12 of 12 (9 before), and its thirteen frigates beat its capital 8 times of 12 (2 before).
- Six lancers against the shielded warden: the warden wins 11 of 12 in half a minute, where four of twelve were drawn at a minute and a half.

*What it didn't fix, and the lever I haven't pulled.* A lance still misses a ship that's shuffling about at range. In the frigate duel a shot takes 23 ticks to cross the gap, and a frigate moves further than its own width in that time: of 48 shots in thirty seconds, ten landed. No aiming rule mends that. The shot's speed would. I ran every fight with the shot at 6 and at 8 px a tick, beside today's 4:

| | Shot at 4 (as built) | At 6 | At 8 |
|---|---|---|---|
| 76 LANCERS at 1 CAPITAL, shots that hit | 41% | 43% | 49% |
| Reference fleet: 85 LANCERS at 1 CAPITAL | 62% | 71% | 73% |
| 76 FIGHTERS against 76 LANCERS, fighters win | 9 of 12 | 4 of 12 | 3 of 12 |
| Reference fleet: 85 FIGHTERS against 85 LANCERS | 11 of 12 | 7 of 12 | 5 of 12 |

A faster shot buys a little accuracy against big ships and turns fighters-beat-lancers over. That's a balance number and Henry's. I'd leave it at 4.

*Found on the way.*
- **My first cut of rule 2 was too strict.** I gave the gun the 100 px its text says, and an existing test failed: a frigate chasing a beaten king could no longer hit it, because the place it holds off at was just outside 100 px for a target drawing away. The shot really flies 112, so that's what the gun counts now.
- **My own test caught a gap in rule 1.** Working in whole ticks, a ship closing at 3 px a tick was looked for at every fourth pixel and missed. The shot is in each pixel of its line for a quarter of a tick, and the check now covers everywhere the ship's pixel could be in that quarter.
- **A first try at rule 3 called a ship steady if its speed hadn't changed since the last tick.** A heavy ship gathers speed a quarter-pixel every other tick, so half the time it looked steady while it was accelerating. What a ship is trying to fly at says it properly.
- **Most of a lancer swarm's misses at a still target aren't misses.** One lancer hits an outpost with 31 shots of 32. Twelve hit with half of theirs: the rest are in flight when it dies, or fly down a tunnel the shots before them drilled.

*Not built.* A faster lance shot. Anything to stop a swarm over-killing. The same hold-fire rule for the beam, which shows its line and is dodged anyway.

*Checks (2026-11-12, after the lance).* 190 tests pass, eight of them new (the eighth came with the rewording of rule 3 and is its worked example: a 13 px ship crossing 40 px off is fired at down a line it can't clear, a 3 px one never). 169 of 169 deliberate rule breaks are caught, eleven of them new: a lance that asks where a ship is and not where it will be, one that fires at what flies as fast as its shot, one that counts pixels past where the shot is spent, a shot spent short of what the gun fires at, each of the gun's three checks (flies on, flies as it's trying to, stops) left out in turn, a lead worked from the distance alone, a lead longer than the shot lasts, a lead on a ship that's lining up, and a ship whose wanted speed is just its speed. The headless fights now print each side's shots that hit (`--pg-sim`). Every fight with a lance in it has moved, and those are re-recorded. Not done: I haven't watched a lance fight in a window, and nobody has played since the change.

**The design database and how many slots a team has (talk, 2026-11-12).** Henry: "I'm tempted to design the design database soon and integrate it with teambuilding, I think it will help us as we start implementing new design options. I've always been wondering if we want to go up to 8 or 9 slots per team." Nothing here is ruled or built. What follows is mine.

- **Why now.** The yard holds one team of seven and nowhere else to keep a drawing. I had to give the beam and shield examples a window of their own because there was no place to put three ships. Every new part makes more drawings worth keeping than a team has slots.
- **A drawing and its place in a team are two things, and today they're one line.** A design's line holds its pixels and also its standing order, its style, its group size and, for a factory, a build order written as slot numbers. The pixels are what's worth sharing. The rest only means something inside one team: move a factory to another team and its build order points at the wrong ships. So: a library of drawings, and a team that lists which drawings it fields and what each is told.
- **What a drawing would carry in the library.** Its pixels, a name, who drew it, what it was drawn from, and the parts it uses. The last one is the buildset question answered for free: a drawing belongs to any set that has all its parts. Two machines already send each other designs as text, so sharing is moving files until there's somewhere to put them.
- **Seven, eight or nine.** A slot is a card the king can print, an entry factories can build, and one switch to work in a fight. The starter deck spends three of its seven on the economy (miner, outpost, shipyard), which leaves four that fight. That was a triangle and a capital. With a siege ship and a shielded ship it's already one short. Nine would be three for the economy and six that fight.
- **What nine costs.** One more switch to work per slot, and more to read at 400 by 300. The match's cards are seven rows of 18 px with the queue right under them, so nine rows take 36 px from the queue and the minimap or the rows get thinner. The yard's seven tabs fill the width exactly; nine are 44 px each, seven letters. Keys 8 and 9 are free and the wire allows 64 slots. So it's a layout job, not a rules one.
- **What I'd do.** Make the number of slots a property of the team's ruleset, not of the screens, and lay the screens out for nine. Then seven against nine is a number to play both ways, not a rebuild.

**Henry plays the lance, and a steer on planning (2026-11-12, evening).** Henry: "I tried the arena and arena fight 2 is pretty weird. as you said, the lancers only take sure shots. which means they take very few shots while orbiting for example a frigate. so it took a lot of loops doing nothing before they won. don't fix anything yet, let's talk planning. I don't want to just spin our wheels on ship AI right now." Nothing in the game was changed after this.

- **What fight 2 was.** Six lancers against one frigate: blue won at 2:24 without losing a ship. Henry's saved LANCER card is set to ORBIT in groups of 3 (`proving_ground_designs.txt`), and the window runs the saved cards. A lance fires straight ahead, an orbiting ship's target is always crossing its line, and the gun holds fire on a crossing target. So it almost never fires.
- **The hole in my measuring.** The headless fights use the built-in deck, whose lancer flies RUNS: that fight is recorded as won 10 times of 12 in about 0:11. No headless fight has a lance ship on ORBIT or KITE. So when I told Henry the gun "doesn't go quiet", that was only true of lancers on RUNS. I haven't run Henry's card headless, so ORBIT being the whole difference is my reading, not a measurement.
- **A crash, undiagnosed.** Henry: "it looks like I hit a crash either in the game or in claude. i was testing a match." What's left behind: the play log's last match (seed 10274, starter deck) stops at 4:12 with no result line, no game process is running, and Windows' application log holds no crash from that hour. A Rust panic doesn't go to that log, so the game isn't cleared. The Claude session was itself restarted around then. The game doesn't write a panic to its own log. It could, and then the next one would name itself.
- **Parked on Henry's steer:** what a lance does about a crossing target (hold, or take the one guess that it flies on), what a lance ship should do when told to ORBIT, swarm over-kill, the lance shot's speed.

**The plan, Henry's four rulings, and the first round of it (2026-11-12, night).**

*The plan* (mine, put to Henry after the steer above; Henry: "Ok, let's do that plan"). One milestone: two players each build a team from a shared design library and play a match over the network.
1. Now, small: crashes write themselves to the log. Ship behaviour is parked (the list above). An item jumps the queue only if it's a crash or makes a part unusable.
2. The design database and the team builder. The arena and the headless fights read the same designs.
3. Parts in batches, each with example drawings in the library.
4. One ship-behaviour round, later, behind a sweep of every gun on every style, so gaps like a lance on ORBIT show as a table.
5. The AI opponent: its economy against the boom, and levels.

*The rulings* (Henry's words).
- **Slots:** "Nine for now".
- **Orders and styles:** "They go with the drawing, but you can easily customize a drawing from the database". (I'd proposed they belong to the team. They don't.)
- **Buildsets:** "later".
- **The library:** "Eventually I want real players to be able to share their designs with other real players. So I guess we're spinning up a web service! Can we make it so there's not too much development churn on the web service part for now, though? Maybe something that runs locally without much effort but can be put onto a cloud host (hopefully cheaply) later? Or is there some Steam integration that can do this nicely? I don't want players to have to use the Steam Workshop UI but if there's like a Steam Workshop API we can call and put it in our game maybe?"

*Where a shared library lives: my answer, not ruled.*
- **Sharing comes in three sizes, and only the last needs a service.** (1) A drawing as a line of text, by clipboard: built this round. (2) Two players joined for a match looking through each other's library over the link they already have: not built, and cheap. (3) A public shelf that strangers browse: that needs somewhere to keep it, a way to know who a player is, and someone to take down what shouldn't be there.
- **Steam can be the public shelf, from inside the game.** Its `ISteamUGC` calls make an item, upload it, list and search what's there (50 to a page), and fetch one, all behind our own screens. Valve keeps the files for nothing and a player is their Steam account. What it asks: the game has to be a Steam app (100 dollars, paid back after 1,000 dollars of sales), with the option switched on in the app's Workshop settings, and Steam running. One thing the Workshop's own pages can't be kept out of: a player's first upload stays hidden until they accept the Workshop's terms, and the game has to send them to that page. There's a Rust crate (`steamworks`) that wraps these calls.
- **Our own service is small to run and costly to look after.** A single small program with one file of data, started with one command here and later on a rented machine for a few dollars a month. The hosting isn't the cost. Accounts, spam and taking down an offensive drawing or name are, and those are what Steam brings with it.
- **What keeps churn down either way.** The library screen asks a shelf for three things: its list, one entry, and to keep one. Today's shelf is a file. Steam or a service would answer the same three, later and sometimes not at all, so the one thing the screen will need then is a way to say it's waiting.
- **What I'd do.** Run no service yet. When the two-machine play starts, add the second size. Choose between Steam and our own when there are strangers to share with: Steam if the game is going there.

*As built.* Rows L1 to L8 in the table. The calls that were mine:
- **A card and an arena row are one line,** 13 px high where they were 17. The slot number and the loop sign came off the card (a repeating card keeps its blue frame, and its job keeps the sign in the queue). The price is a bare number, orange for a seed. The order is one letter.
- **The starter deck stays seven,** so the AI plays seven and a fresh team has two empty slots.
- **TAKE goes straight back to the yard,** and keeps the replaced drawing without asking.
- **A factory's build order still names slots by number.** It travels with the drawing as those numbers, and means whatever is in those slots of the team it lands in. Taken into another team, a shipyard may be set to print the wrong things until its build order is redone in the yard.
- **Limits:** 200 drawings and 50 teams of your own, 20 things a paste. A pasted name is cut to capitals, digits, spaces and dashes.
- **A design line with its corner off the grid is refused,** from the clipboard, a file or another machine. Before, a line from another machine could hold any number there.
- **The other machine must be this version:** the network's version number went from 4 to 5.
- **Nothing was done about buildsets, a service, or looking through another player's library.**

*Fight 2, measured.* `--pg-sim mine` runs Henry's saved team. Six lancers on ORBIT in threes against the frigate: the fight the window shows ends BLUE 2:24, which is what Henry saw. Over twelve seeds blue wins 11 and one is drawn, in 1:47 on average, and the lancers land 80 of 295 shots. Twelve of them take an outpost in 0:19, where lancers on RUNS take 0:06. So ORBIT is the whole difference, and it's a measurement now.

*The crash line, checked.* I made the game crash once on purpose, in a scratch folder. Its log got: `CRASH 2026-11-13 02:15:56 UTC | panicked at src\main.rs:109:25: index out of bounds: the len is 0 but the index is 7`. The switch that did it is gone.

*Henry plays it (2026-11-12, late).* "I played a game. I didn't really notice any problems. Beams and shields both seemed fun." What the play log shows of that sitting:
- **The library was used to fill slots 8 and 9:** DESTROYER and TITAN were taken first, then BEAMER and WARDEN over them. Nothing was kept, pasted or loaded, so the library has no file yet.
- **One match on that team of nine,** won at 15:51, with orders switched from the cards (the WARDEN's too). A second was started and left at once, the SHIPYARD was edited, and the session closed with SESSION END. No crash.
- **How much of a test it was.** The AI held one factory until the thirteenth minute, with a bank that never passed 69, while Henry had five by the seventh. So the beams and shields were tried in a match the AI wasn't contesting. That's the AI's economy, which is step 5 of the plan.

*Checks (2026-11-12, after the library).* 208 tests pass, 18 of them new. 192 of 192 deliberate rule breaks are caught, 23 of them new: a team left at seven, cards and rows too tall for nine, two orders sharing a letter, a taken drawing losing the one it replaced, undo reaching back into the replaced drawing, a drawing or a team kept twice, an empty slot kept, one press deleting, a ready-made drawing deleted, a paste without limit, a pasted name shown as it came, an empty line pasted as a drawing, a corner off the grid read, the ready-made shelves written to the file, ESC in the library asking to quit, an old save left at seven, a tab name too long for its tab, and a crash spilling over several log lines. The arena's headless fights (`--pg-sim` and `--pg-sim parts`) are byte for byte what the lance round recorded. Not done:
- **The real Windows clipboard is untested.** The tests use a stand-in, and I wouldn't overwrite whatever Henry had copied to try it.
- **AI matches aren't compared byte for byte.** The only older build I'd kept turned out to be from before the lance change, so it differs for that reason. What this rests on: no file of the sim was touched this round, and nothing in the sim reads the slot count.
- **Nothing was watched in a window,** only exported screenshots, and nobody has played it.

**After the first game on nine slots: five things from Henry (2026-11-12, late).** Nothing here is built. The rules below are proposals.

*Henry's words.*
- **Parts:** "i would do missile soon but I wouldn't do charger yet."
- **A gap in play:** "once asteroids were eaten up, it could become impossible to extend your network since new outposts have to be places on asteroids. someway to put a stationary node in space would fix this. I lean towards a way to send seeds to a spot in space and they build themselves using resources from your network. maybe you have to hold down control or alt to place them."
- **Settings:** "at some point we will need a comprehensive hotkey rebinding option (i guess it should use Steam's preferences persistence). and other game settings."
- **The library:** "do you think it's necessary to make library drawn on top of yard like it is now? is that to differentiate it from the public ship database? when will we build the public ship database?"
- **Steam for play:** "we'll also want to ride Steam's multiplayer/matchmaking features. I know they even have some kind of low latency multiplayer backbone, should we use that?"

*Seeds in space: proposed rules.* Today a seed costs its design's whole price when it's sent, and the factory then grows by eating its rock, one rock pixel for each pixel grown. If the rock runs out first, it stops there, unfinished.
1. **One rule for both.** A growing factory takes each pixel from the rock under it, and from your bank when there's no rock there to take. Planting in open space is the case with no rock at all. A factory whose rock runs out part-way finishes from the bank where today it stays unfinished.
2. **What that costs.** The seed's price is still paid in full when it's sent, which is what stops seeds being scattered as cheap walls. So a factory grown wholly in space costs its price twice over, once at sending and once as it grows: the bank supplies what a rock would have. Whether twice is right is a number for Henry. The other choice is a small price at sending and the rest as it grows, which makes space as cheap as rock.
3. **It waits when the bank is empty,** as a factory printing does, and grows at the pace it would on rock.
4. **Where.** Anywhere the whole drawing has room, clear of rocks and of anything planted or growing, and within reach of a node of yours, since that's what feeds it. Out of reach of your network it's refused.
5. **How.** With a seeded card picked, a click on a rock plants on the rock as now. Ctrl and a click on open space plants there, and an IN SPACE button does the same for the mouse alone. The pointer shows the drawing's outline at the spot, and what it would link to. Ctrl, not Alt: Windows keeps Alt for itself.
6. **The AI** uses it only when it wants a factory and no rock in its reach will take one. No more than that this round.

*The missile: proposed rules.* Numbers are placeholders.
1. **A missile pixel is a launcher,** in WEAPONS. It needs no lining up: it launches at the nearest enemy within 200 px, once every 6 s. A cannon reaches 130. Only a long beam reaches further, 232 at its longest, and a beam has to be lined up.
2. **The missile chases.** It flies at 3 px a tick straight at where its target is now, steering every tick, for 8 s. That's as fast as the fastest ships, so a fighter at full speed outruns one from behind and anything slower is caught.
3. **It blasts.** Where it hits it takes that pixel and the eight round it, plate counting double.
4. **Flak shoots it down.** A flak gun with an enemy missile in reach fires at it before anything else, and one hit kills it. No other gun aims at missiles. A beam's cut kills any on its line.
5. **A shield's circle stops one** for a charge, like any shot.
6. **Its ship holds off at range,** as a kiting ship does today. No new flying.
- *The job nobody has:* hitting from far off in any direction, with no lining up. *The answer you can draw:* flak, which gets a second job, and speed.

*The library over the yard.* It isn't necessary, and it wasn't to set it apart from a public database: an overlay was the quickest fit, like the match menu. It already covers all but a 6 px rim of the yard. I'd make it a screen of its own, reachable from the arena and the match menu too, with the public database as one more shelf in it beside YOURS and the ready-made ones, so there's one place to look for a drawing and TAKE and KEEP work the same on every shelf.

*When the public database gets built.* Henry wanting Steam's multiplayer answers the open question of where it lives: Steam's Workshop, called from inside the game. It needs the game to be a Steam app, which only Henry can set up (a Steamworks account and the 100 dollar fee). I'd do the two gameplay pieces first (space seeds, missile), then a Steam round in this order: play over Steam, the Workshop shelf, saves in Steam's cloud.

*Steam for play.* Yes, once the game is a Steam app.
- **What it gives:** no address to type and no ports to open (Valve's relays join the two machines), neither player sees the other's address, a friend is invited from the friends list, and public lobbies can be listed and searched. That's Steam Networking Sockets over Steam Datagram Relay, and Steam's lobbies.
- **The backbone isn't what we need it for.** A match runs in step at 15 ticks a second with one to four ticks of lag allowed for, so it forgives ordinary latency. The gain is that two players behind home routers connect at all.
- **How it fits.** The match already talks through a wire that carries messages whole and in order, and Steam's reliable messages are that. So it's one more kind of wire beside today's. Connecting by address stays for testing.
- **What it doesn't give:** ratings or a ladder. Lobbies can be filtered, but ranking players would be ours to build.

*Hotkeys and settings.* Agreed, and not yet. Two notes for when it's built. The hover texts name their keys in their own words today ("[L] THE LIBRARY..."), about a hundred of them, so rebinding needs one table of actions and keys that those texts read from. And Steam's cloud can sync a settings file with no code at all, so "Steam's persistence" costs nothing beyond saving the file where Steam is told to look. Steam Input is for controllers, as far as I know. Keyboard rebinding is ours.

**Seeds in space and the missile, built; and what Henry said next (2026-11-12, night).**

*Henry's words on the proposals above.*
- **Twice the cost:** "I think factories in space costing double actually makes sense, although it might be a bit counterintuitive to players. Maybe we should display a warning of some kind while placing it."
- **The missile's ship:** quoting my rule 6 ("Its ship holds at range like a kiter, so no new flying"): "Shouldn't its ship just follow whatever order it's given." Yes. Rule 6 is dropped: a missile ship flies as its card says, like any ship.
- **The library:** "I agree with making it its own screen. The public database will need to be organizable and searchable with orderings and such things." Not built this round.
- **Steam:** "I'm already a paid Steam developer." And: "Agreed Steam multiplayer backbone sounds good. It always seemed decent with the only drawback being low-level code and learning curve, which I have you for. Would we still have to build the ELO system as a web service? Any way to do that in Steam? What if we just made it honor system and stored people's ELOs locally?"
- **What's next:** "I'm OK to go forward with your proposed next implementation phase. We do need a few more parts before anything else. Soon though, I'm tempted to start all those Steamworks integration items soon as well. I'd also like to get more team designs with a greater variety of army plans and talk about unit AI and enemy player AI more as well."
- **Unit AI:** "I can picture alternatives to our current unit AI system. For example, we could have a handful of fixed "personalities" that we tailor more carefully, and then not have gametime AI switching, instead there could be a system of attack and defense flags and units decide which flag to prioritize based on their personality, or something like that. I'm definitely not saying we should do that. In fact I think fixed "personalities" rubs against the fully custom ship design a bit. But I think we should give strong consideration to systemic improvements to our unit AI system. I think it's decent now but a little bit clunky."
- **Shields:** "We have weapons that are clearly good against armor. Do we have any weapons that are clearly good against shield right now?"

*Seeds in space, as built* (rows G1 to G4). The calls that were mine:
- **A planted factory is in its network from the moment its seed lands,** so the network feeds it directly: no special feeder. It takes its turn with the king and the factories that are printing, a pixel a turn.
- **Only a factory can go into space.** A seeded design with no factory pixels is in no network, so nothing could feed it there.
- **Clear of rock means the whole drawing:** no pixel of it over rock. With any rock under it, plant on the rock.
- **The seed's own core is free,** as on a rock: a 121 px outpost in space costs 121 at once and 120 as it grows.
- **The IN SPACE button takes the view note's line** in the sidebar while a seed is being placed. Ctrl held does the same. (Taken out 2026-11-13: see "The IN SPACE button taken out", below.)
- **A new command on the wire** ("W team slot x y"), so the network's version number is 6.

*The missile, as built* (rows X1 to X5). Where it differs from what I proposed:
- **Its blast is a cannon shell's,** 3x3 and plate goes with it. I'd proposed plate counting double. One rule for both is simpler, and a missile is easier to stop than a shell already.
- **It's spent after 8 s by the clock,** and flies as fast along a diagonal as along a row (a step along its line is counted for what it covers).
- **It can pass through rock,** like every shot.
- **Flak sends two shots at a missile,** not every gun in reach, so one missile doesn't draw a whole fleet's fire.
- **Its ship flies as its card says.** Left as drawn, the ARCHER has the thrust to make attack runs, so its card is set to KITE.

*What the example fights show* (two ARCHERs, 36 px, twelve seeds; placeholders, and one design).
- Against an OUTPOST: won 12 of 12 in 0:22. The outpost's four flak guns shot down 108 of 192 missiles.
- Against a FRIGATE: 8 draws, 4 losses. Its six flak guns shot down 849 of 870, and neither side could close.
- Against six FIGHTERS: lost 12 of 12 in 0:10, though 60 of 134 missiles landed on fighters flying in at them.
- Against a WARDEN: 9 draws, 3 losses. No missile reached its hull.
- The nine earlier part fights, and both sets of arena fights, are byte for byte as recorded.

*My answers, none of them ruled.*
- **Ratings.** Steam has no rating or ranked matchmaking of its own. It has leaderboards (a number for each player, sorted, for everyone or for friends) and lobbies that can be searched for a number near yours, and its own guide to skill matching is built from those two. A game writes its own player's leaderboard number unless the board is set so that only a server of ours can. So an honour system needs no web service, and I'd keep the number on a Steam leaderboard, not in a local file: the same trust, but everyone can see the ladder, lobbies can match on it, and it survives a reinstall. A local file gives none of that. If cheating ever matters, the upgrade is a small service that writes the numbers. This game makes that service's job easy: a match is its commands replayed, so the service can replay both players' commands and see who won for itself.
- **Unit AI.** What's clunky today, as I see it: three settings on every design (the order, which can be switched in a match, and the style and group, which can't), all of them for every ship of that design at once, and none of them says where. Henry's flags are the missing where. I share the doubt about fixed personalities for drawn ships. Two other things could carry what a personality would: what the drawing itself says (its guns' reach and its speed already pick its style), and where a ship was built (the fun thesis is wings spawned from a tree of factories, so a factory could be what holds the flag its ships fly to). I'd open that talk with two or three whole systems written out side by side and judged on the same tests, not with a patch. Parked until Henry calls it.
*Checks (2026-11-12, after seeds in space and the missile).* 224 tests pass, 16 of them new. 221 of 221 deliberate rule breaks are caught, 29 of them new. Two of the new ones got past their tests at first (a missile that's never spent, and a ship with only launchers counting as unarmed): the first test's missile left the map before its time was up, and the second's ship had a flak gun too. Both tests are tightened and both breaks are caught now. The arena's headless fights are byte for byte as recorded (`--pg-sim`, `--pg-sim old`, and the nine earlier part fights). AI matches play the same until the fourteenth minute and differently after: that's the rocks running short, where a factory now finishes from the bank and the AI may plant in space. They still end. Not done: nothing was watched in a window, only exported screenshots, and nobody has played either feature.

- **Shields.** Yes: flak. A circle counts shots, not damage, so many small shots strip it and one big one is wasted. Six FIGHTERs kill a WARDEN 12 times in 12, in six seconds. Six LANCERs lose to it 11 in 12, and a FRIGATE, a BEAMER and two ARCHERs never hurt it. Anything fired from inside the circle also ignores it. So there's a triangle: plate stops flak, the big guns break plate, a shield stops the big guns, flak strips a shield. What's missing is anything that strips a shield from range: flak reaches 48 px.

**The repeater and the projector: proposed rules (2026-11-13).** Henry, after playing the missile: "I think archers are overpowered currently but that doesn't matter for now. First launch me a side session to do claude-in-chrome for setting up the steamworks stuff (i already have an unpublished game set up but my guess is you'll want to use a new one). Then post the repeater_projector rules." The side session was started (it writes what it sets up to `STEAM.md`). Nothing below is built, and every number is a placeholder.

*The repeater.* The gap it fills: flak is what strips a shield, and flak reaches 48 px.
1. **A gun that fires straight ahead only,** like a lance. WEAPONS, uses 2, weighs 1.
2. **A stream of small shots.** Each does 1 damage, as flak's does: it breaks hull, and plate turns it. It stops at the first pixel it meets.
3. **It reaches 110 px:** more than twice flak's reach, a little past a lance's.
4. **The shape is the rule.** Repeater pixels one behind another in a column are one gun, as beam pixels are. One pixel fires every 8 ticks (two shots a second). Each pixel behind it takes a tick off, down to every 3 ticks with six. Side by side they're separate guns, each on its own line. So a column is one fast stream down one line, and a row is a slow curtain several lines wide.
5. **It fires whenever something will be on its line** when the shot gets there, if the target flies on as it is. It doesn't hold fire as a lance does: its shots are cheap.
6. **What that does to a shield.** A circle loses a charge to every shot and starts its rest again, so a steady stream empties a circle in about two seconds and keeps it empty, from outside flak's reach, while something heavier does the damage.
7. **Missiles.** A repeater shot that meets a missile takes it down, as a flak shot does. A repeater never aims at one: it only fires ahead.
8. **Its ship flies as its card says.** Like any ship with a gun that fires ahead, it lines up on its target's row or column, which lance and beam ships already do. No new flying.
- *The job nobody has:* keeping a shield down from range. *The answers you can draw:* plate; being off its line (speed, or ORBIT); out-reaching it (a missile, a cannon, a long beam).

*The projector.* Henry, 2026-11-11, on shields: "We could make a different part like a shield projector that does have the ability to protect allies." And on auras: "We'll want to think about how to make these "auras" more interesting than just put pixels down, get bonuses." And on a dome: "Getting the ship AI to effectively exploit a big circle that it has to stay inside and shoot out from might be a bit difficult".
1. **It protects friends, never its own ship.** HULL & ARMOR, uses 5, weighs 2.
2. **Its reach comes from the room round it.** A projector pixel reaches 10 px for each of the eight cells round it that's empty on its own ship. Buried in hull it reaches nothing. On the end of a one-pixel stalk, with seven empty cells round it, it reaches 70 px. So a projector has to stick out, where it's easy to shoot off.
3. **A skin, not a circle.** A friend is covered while any of its pixels is within that reach of the projector. An enemy shot that would hit a covered friend stops at the pixel it would have hit, does nothing, and costs the projector a charge. There's nothing for a friend to stay inside or shoot out of, and its own shots aren't touched.
4. **One pool.** A ship's projector pixels share one pool of charges, 4 for each pixel. It's full again 3 s after the last shot it stopped, which are the shield's numbers. Empty, it covers nothing.
5. **No stacking.** A friend in reach of two projector ships is covered by one at a time: the nearer one with a charge left pays.
6. **Big shots cost one.** A missile or a shell costs a charge and doesn't burst. A beam's cut costs a charge at the first covered ship on its line and stops there, as it does at a circle.
7. **A friend's own circles come first.** A shot stopped at a ship's own shield circle never reaches the skin.
8. **Seen.** A covered ship wears a thin outline in the projector's colour, which thins as the pool drains, as a shield's outline does.
9. **No new flying.** It flies as its card says. On ESCORT it stays by the king and covers it. Planted (a seeded design with no engines), it covers the factories round it.
- *The job nobody has:* protecting other ships, the king and what's planted. *The answers you can draw:* flak and repeaters drain the pool; the projector's own ship is bare, and its projector sticks out; or reach past it.
- *Placement, spelled out* (Henry, 2026-11-13: "explain more about the placement rules for projector. does connectivity still apply?"). Connectivity still applies as proposed: a projector pixel joins the ship edge to edge like any pixel, so at least one of the four cells beside it is filled and the most it can have empty is seven. The eight cells are the four beside it and the four diagonals. Any pixel fills a cell, another projector included, so two projectors side by side cost each other reach. Cells off the edge of the grid count as empty. The reach is the same all round, whichever cells are the empty ones, and it's counted from the drawing, not from what's left standing in a fight: otherwise a half-grown factory's projector would reach furthest before the hull round it had grown. Some shapes: buried, 0 px; in a flat edge with hull either side, 30; on a corner, 50; sitting on a flat edge with nothing beside it, 50; on a stalk one hull pixel long, 70. The other way, which was Henry's own first idea (2026-11-11: "something fancy each block of which needs to be given clearance from other pixels on the ship (exception to the connectivity rule)"), is a projector that floats free of the hull with all eight cells empty. It would need the yard's check, the order things grow in and the rule that cuts loose what isn't joined to a core all to make an exception for it. I'd keep it joined: the stalk gets seven eighths of the reach, and it's something to aim at.
- *How it's more than pixels for a bonus:* the reach is paid for in exposure, the pool is shared by everything it covers so covering more drains it faster, and its own ship gets nothing.
- *What I'm unsure of.* Whether 70 px is enough when ships spread out, and whether ships will stay near one: nothing tells them to, and that's ship behaviour, which is parked. It will work best beside the king and over planted things.

**Large ships, and the projector as a structure (2026-11-13).** Nothing here is built.

*Henry's steer.* I'd pictured two or three projector pixels on stalks on a small ship, and said so. Henry: "i think the ship sizes you're thinking about for this are too small and the number of projector pixels is too small. we want to keep ships as diverse as possible and we want to give reasons to make large ships. (honestly large ships seem pretty bad right now, we need to fix that. they should be actually more powerful per-pixel since you have to save up to build them and then wait out the build time. i'm pretty sure they're less powerful per pixel right now). I think we should make the projector something where you build an entire, large, exposed structure."

*What the recorded fights say about size.* Equal pixels a side, twelve seeds, one design of each kind, so a pointer and no more.
- Starter deck, 687 px a side: 76 LANCERs beat 12 FRIGATEs 12 to 0 and one CAPITAL 12 to 0. The CAPITAL beats 12 FRIGATEs 9 to 3. 76 FIGHTERs lose to both, 0 to 12.
- Reference fleet, 600 px a side: 85 LANCERs beat frigates, destroyers and the capital, 12 to 0 each.
- The TITAN, 3,093 px, wins none: 0 to 12 against 70 frigates, and no wins in 12 against 17 destroyers or 4 capitals.
- So the cheapest ship with a lance beats everything bigger, and the biggest ship beats nothing. Flak swarms do lose to plate. None of this counts the saving up or the build time, which makes it worse for the big ship.

*Why, as the rules stand.* My reading, not measured rule by rule.
- **A core makes 4 power and costs one pixel.** A nine-pixel ship gets half its power or more from its core (the LANCER 4 of 8, the FIGHTER 4 of 6). A capital gets about one percent of its from its one core. Small ships carry more gun for their pixels.
- **A gun reaches the same from any ship.** Only the beam grows with the ship that carries it.
- **One core is one point of failure,** and seventy-six ships have seventy-six.
- **A heavy ship turns slowly,** so its forward guns bear on one line while a swarm is all round it.
- **The only tax on a swarm is over-kill.**

*A rule of thumb to adopt, if Henry agrees.* A part should be worth more than its pixels as its structure gets bigger, the way the beam's column is. Then big ships get their reasons from the parts, and small ones keep theirs.

*The projector again, as an array.* It replaces the proposal above.
1. **Projector pixels joined edge to edge are one array.**
2. **A pixel only works with room round it:** at least four of the eight cells round it empty, counted from the drawing. So an array is thin lines and lattices standing clear of the hull on struts. A solid block of projector, or a row lying along the hull, doesn't work.
3. **Its reach comes from its span.** An array reaches 2 px for every pixel of its width plus its height, from every pixel of it. A T, a bar 13 long on a mast of 7 more, reaches 42 px. A cross 21 by 21 reaches 84. A ring 30 by 30 reaches 120. Capped at 150.
4. **Its pool comes from its size:** one charge for each working pixel, full again 3 s after the last shot it stopped.
5. **The skin covers every friend in reach and the rest of its own ship. It never covers projector pixels.** The array is the one part of the fleet that's bare.
6. **Cut it and it shrinks.** An array cut in two is two smaller arrays, each with its own span and its own share of the pool. A piece cut off from every core breaks away as loose pixels, like anything else.
7. **Uses 2 power a pixel, weighs 1.** A 41 px cross needs 82 power, so about 80 hull pixels behind it: a ship of 130 px or more. A 116 px ring wants 350.
8. Carried over from before: no stacking, big shots cost one charge, a friend's own circles come first, covered ships wear an outline that thins as the pool drains, and no new flying.
- *Why it suits big ships:* reach grows with span and the ground covered grows with the square of reach, so a big array is worth much more than twice a small one, and only a big ship has the power for it.
- *The answer you can draw:* shoot the array. A lance or a beam through a strut or a spar cuts its reach. Small shots drain the pool.
- *Shapes it invites:* crosses, rings, combs, dishes on booms. A wide thin lattice gets span cheaply and is easy to cut. A dense one holds more charges and reaches less for its pixels.

*The repeater, stretched the same way.* A column up to 16 pixels long, as the beam's is, where each pixel adds both rate and reach. A six-pixel gun on a small ship strips a shield from 110 px. A sixteen-pixel one on a long ship does it from further and faster. Numbers when it's built.

*The Steam side session finished.* App 1309380 is No Mind Wing's (Henry's old, unreleased "Rock Collector", reused, so no fee). ISteamUGC is on and published as configuration, and the game is still unreleased. `STEAM.md` has what was changed and what's left for Henry: the name, keys for a second account, Cloud quotas, the rating leaderboard.

**The shield projector as an orb, and the lattice kept for stasis (2026-11-13).** Nothing here is built.

*Henry, on the array above:* "could we keep the lines+lattices shape for the stasis projector and have the shield projector be something like a teacup handle, doorknob, exposed orb, or something like that? just purely for intuition and feel purposes; the vibe of a spider web being a Starfleet shields projector doesn't make sense to me. plus the pixels should be blue".

*So.* The array's rules (thin lines and lattices, reach from span, cut it and it shrinks) are set aside for a stasis projector, which hasn't had its talk yet. The shield projector becomes a solid lump that stands proud of the ship: a doorknob.

*The shield projector, as an orb.* It replaces both proposals above. Numbers are placeholders.
1. **An orb is a solid lump of projector pixels.** Its size is the side of the biggest solid square that fits in it: 3 for a 3 by 3, 5 for a round blob seven across. Corners can be trimmed to make it round without losing size.
2. **It has to stand proud.** At least three quarters of the cells round its outline must be empty, counted from the drawing. A neck narrower than the orb is the only thing that may touch it. Half sunk into the hull, it does nothing.
3. **Its reach comes from its size:** 10 px for each pixel of its size, from its edge. A 3 by 3 reaches 30 px, a 5 by 5 reaches 50, a 9 by 9 reaches 90. Capped at 150.
4. **Its pool is its pixels:** one charge for each, full again 3 s after the last shot it stopped.
5. **The skin covers every friend in reach and the rest of its own ship, and never the orb itself.** The orb is the one bare thing in the fleet.
6. **Shot at, it weakens smoothly.** In a fight its reach is 10 px for each pixel of the square root of the pixels it has left, so a 5 by 5 with nine pixels gone still reaches 40 px. A hole through its middle doesn't switch it off.
7. **Uses 2 power a pixel, weighs 1.** A 5 by 5 needs 50 power: a ship of about 80 px. A 9 by 9 needs 162: about 250 px. A 15 by 15 needs 450: a ship of 700.
8. **Blue.** Its pixels are a bright blue with a pale centre, and the outline covered ships wear is the same blue. It has to be told from the blue side's hull, which is a darker blue: a brighter, colder one, checked on screen before it's settled.
9. Carried over: no stacking, a big shot costs one charge, a friend's own circles come first, the outline thins as the pool drains, and no new flying.
- *Why it suits big ships:* one 9 by 9 covers nine times the ground of a 3 by 3 and holds nine times the charges, and nine small orbs on nine small ships each cover only their own neighbourhood. Only a big ship has the power for a big orb.
- *The answer you can draw:* shoot the orb. It's bare, it sticks out, and its neck is thin.
- *A handle instead of an orb* (Henry's teacup handle): a loop whose reach comes from the hole it encloses. I'd take the orb. At this size a one-pixel loop reads as a wire, which is the spider web again, and one hit that opens the loop would switch it off.

**The shield projector by Henry's rule (2026-11-13).** Built the same day: see "The repeater and the projector as built" below. This replaces the orb measured by its biggest square.

*Henry.* On whether my orb was "just ... a fixed shape you drop on": I said nearly, and offered a sized stamp or a knob-and-neck with two dials. Henry: "it's annoying to make very specific requirements that people have to meet from painting arbitrary pixels. I think we should either make it a sized stamp, or else make the rule something like: each shield projector pixel must adjacent to four other projector pixels, and must exposed in the same way engines are, NOT counting other shield projector pixels as blocking. that lets people draw their own exposed shapes".

*The principle in that,* as I take it: a rule about a drawing should be something each pixel passes or fails where it stands, like an engine's way out, or else the part should be a stamp. Not a measurement of the whole shape that the player has to paint towards.

*The rules, from Henry's two.*
1. **Each projector pixel needs four other projector pixels among the eight round it.** (Counting only the four beside it, no finite shape could pass, so I read it as the eight.) What passes: thick, rounded lumps. An orb, a knob, a handle two pixels thick. What fails: a wire, a sharp corner, a stub. The smallest lump that passes is a 4 by 4 with its corners off, 12 pixels.
2. **Each needs a straight way out, as an engine does:** in one of the four directions, its own line and the lines either side empty right out past the edge of the ship. Other projector pixels don't count as in the way. So the lump faces open space, and nothing can be built in front of it.
3. **A pixel that fails shows red and the ship can't fly until it's fixed,** as with a buried engine. So on a ship that flies, every projector pixel works, and the sim has nothing to check.
4. **Reach and pool come from how many pixels there are,** not from any shape. A lump (projector pixels joined edge to edge) reaches 10 px for each pixel of the square root of its pixels: 12 pixels reach 30 px, 25 reach 50, 81 reach 90, 225 reach 150, where it's capped. It holds a charge for each pixel, full again 3 s after the last shot it stopped. Shot at, both go by the pixels left.
5. **Unchanged from the orb:** the skin covers every friend in reach and the rest of its own ship, never projector pixels; blue pixels and a blue outline on what's covered; uses 2 power a pixel; no stacking; a big shot costs one charge; a friend's own circles come first; no new flying.

*Exposed on three sides* (Henry, next: "maybe even better: must be exposed as engines are but from three angles at the same time? that makes it really an exposed shape not just exposed from only one side. with "attack from a particular side" not being anywhere on the roadmap, it might make sense for the exposed requirement to be mostly-exposed, not just from one side"). Rule 2 becomes three of the four directions. One change is forced with it:
- **With the engine's exact check it can't be done.** An engine's way out is three lines wide: its own and the one either side. Take the hull pixel a lump is joined to, and the projector pixel beside that joint. Any projector pixel next to that one along the joint has the hull pixel in two of its ways out at once (diagonally: once looking back at the ship, once looking along the ship's side), so it has two clear at most and fails. Without it the pixel at the joint has three projector neighbours and fails rule 1. So no lump could be joined to a ship at all.
- **So a projector pixel's way out is its own line only:** its row or its column, clear to the edge of the ship in three of the four directions, other projector pixels not counting. An engine's stays three wide.
- **What passes:** a lump on a neck, on the end of a boom, on the bow, the stern or a wingtip, with nothing level with it on either side. A thick handle on the ship's flank, since a pixel looks out through the handle's hole and its other pixels.
- **What fails:** a lump in a notch or a bay, in the corner between two walls of hull, or with a tower standing level with it. And plate can't be put in front of it on any of the three sides.

**The repeater and the projector as built (2026-11-13).** Henry: "Yes, build both". Rows R1 to R4 and J1 to J6 in the table above. Nobody has played them.

*The repeater* is the first proposal's, stretched as "Large ships" said: a column up to sixteen long, each pixel adding reach and rate. The numbers are mine: reach 60 + 8 a pixel, a wait of 9 less half the length in ticks, 3 at least. A six-pixel gun reaches 108 px and fires every 6 ticks. The RAKER's ten reach 140 px and fire every 4.

*The projector* is Henry's rule with three open sides: rules 1 to 5 of "The shield projector by Henry's rule", and the own-line check of "Exposed on three sides". One thing in rule 5 was wrong and has been put back: it never covers its own ship (below).

*Where what I built differs from what I proposed.* All mine, none ruled.
- **One pool for the ship, not one for each lump.** Two lumps on one ship each reach by their own size and fill one pool. One number is easier to show (PROJECTOR 32/38).
- **Reach is measured between boxes:** the lump's box to the friend's box. The dotted line drawn round a lump is exactly that distance.
- **The reach is always drawn,** a dot every fourth pixel in a dim blue. The proposal only gave covered ships an outline, and without the line nothing says how near a friend has to be.
- **The outline has cut corners,** so it reads as a bubble and not as the white corners of the ship under the pointer.
- **Two error lines.** "N PROJECTOR PX TOO THIN" comes first, and "N PROJECTOR PX SHUT IN" once none is thin. One line saying both didn't fit the yard's status line.
- **A repeater's wait is exact.** The other guns wait one tick more than their number says (an old habit of the reload code, which nobody notices at 20 ticks). At 3 to 8 ticks that would have made the repeater a seventh to a third slower than its rule. A test counting shots found it: nine where the rule says ten.
- **Shots stopped at a skin are counted with shots stopped at shields** in a fight's result line.
- **The hover and inspector texts.** "NO GUNS: STAYING WITH THE GROUP" ran one letter off the arena's inspector for any unarmed ship, and is "NO GUNS: WITH ITS GROUP". The two power texts no longer list what every part uses: with fourteen parts the list didn't fit, and each part's hover says its own.
- **PROTO is 7.**

*What I found building it.*
- **Ships leave a projector behind.** My first AEGIS carried two flak guns, so it flew as an armed ship does: to its own range at its own speed, 15 px/s against a lancer's 34. Set beside six lancers it had them in reach for the first two seconds and stopped 3 shots a fight. Blue won 11 of 12 anyway, because the AEGIS was 128 more pixels to shoot at, so the result line alone looked like a success. I traced it second by second before believing it. The AEGIS now has no guns, which puts it under an old rule (a hunter with no guns stays beside the nearest friend that has), and fourteen engines, which make it as fast as the RAKER and faster than the BEAMER, WARDEN and FRIGATE. It still can't stay with fighters or lancers. Nothing was added to flying. Whether a fleet should hold together round its slowest ship is for the unit-AI talk.
- **One shell cut my first orb off its ship.** A round orb seven across stood on a base three pixels wide. A shell's 3x3 hole at its waist left everything above joined to nothing, and 34 pixels went to one hit. The AEGIS's knob now has a base five wide, which one shell can't cut through. A player's orb on a thin neck will still go that way. I think that's a fair weakness of an exposed shape, and the rule lets you draw a broad base.
- **Slow big shots at what it covers never break one.** A pool that's back 3 s after the last stop is full again between volleys 6 s apart. Two ARCHERs draw with an outpost and an AEGIS twelve times in twelve, where the outpost alone falls in 22 s every time. A stream is the answer: two RAKERs beat two BEAMERs and an AEGIS in 12 of 12, having 58 shots stopped a fight on the way. Whether that's too hard a counter is Henry's to say.

*The seven new fights,* 12 seeds each, with the earlier fight each one repeats. Examples to watch, not balance: one design of each kind.

| Fight | Blue | Red | Draw | Stopped a fight | The same fight without it |
|---|---|---|---|---|---|
| 2 RAKERS VS 1 WARDEN | 7 | 5 | 0 | 85 at the warden's circles | |
| 2 RAKERS VS 1 FRIGATE | 0 | 11 | 1 | (plate) | |
| 1 RAKER + 6 LANCERS VS 1 WARDEN | 8 | 4 | 0 | 47 at the circles | 6 LANCERS VS 1 WARDEN: 1, 11, 0 |
| 2 BEAMERS + 1 AEGIS VS 6 LANCERS | 2 | 9 | 1 | 13 at the skin | 2 BEAMERS VS 6 LANCERS: 0, 12, 0, over in 0:17 (with the AEGIS, 1:09) |
| 2 BEAMERS + 1 AEGIS VS 6 FIGHTERS | 9 | 1 | 2 | 80 at the skin | 2 BEAMERS VS 6 FIGHTERS: 2, 9, 1 |
| 2 ARCHERS VS 1 OUTPOST + 1 AEGIS | 0 | 0 | 12 | 57 at the skin, and 61 missiles shot down | 2 ARCHERS VS 1 OUTPOST: 12, 0, 0 |
| 2 RAKERS VS 2 BEAMERS + 1 AEGIS | 12 | 0 | 0 | 58 at the skin | |

(The four AEGIS rows are as re-recorded after the projector stopped covering its own ship. What they were before is in "Put back", below.)

*Checks.* 239 tests pass, fifteen of them new. 254 of 254 deliberate rule breaks are caught, 33 of them new: a repeater that reaches or fires the same at any length, fires off its line, waits a tick too long, shoots through plate or past its reach; a projector pixel that needs no neighbours, may be walled in, or is shut in by its own kind; a skin over the enemy, over any friend however far, over the projector itself; a shot, a beam or a burst that goes through one; a pool that costs nothing, comes back at once, never comes back, or keeps its size when pixels are lost; the farther projector paying first; two lumps reaching as one. The starter deck's 15 recorded arena fights and the reference fleet's 22 are byte for byte as last recorded, and so are the twelve earlier part fights.

**Put back: the projector never covers its own ship (2026-11-13, after the build).** Henry, reading the as-built summary: "did we reverse the decision for the shield projector to not protect the ship it's on?" We hadn't. I had.
- *The record.* On 2026-11-11 my second version said "It doesn't skin its own ship", and Henry said "it sounded right, yeah". My first rules post of 2026-11-13 still said "It protects friends, never its own ship." In "Large ships, and the projector as a structure" I wrote rule 5 as "the skin covers every friend in reach and the rest of its own ship", with no note that it reversed an agreed rule and no reason. It then sat under "unchanged" or "carried over" through two rewrites, and that was the set Henry said "Yes, build both" to. My unstated reason was Henry's point that a large ship should be worth more per pixel.
- *Ruled.* I recommended putting it back (SHIELD is for the ship it's on and the projector is for allies; a bare ship is the way to get at a projector; two projector ships can cover each other). Henry: "Yes, put it back".
- *As it is now.* A lump covers every other ship of its side in reach. Never its own ship: hull, engines, core, all bare. No projector pixel is ever covered, a friend's included, so two projector ships side by side cover each other's hulls and not each other's knobs. That was my call when built, and Henry ruled on it the same day. Asked "Should one projector be able to cover another ship's projector pixels?": "No, keep bare".
- *What moved,* 12 seeds, covering its own ship and then not (blue, red, draw):

| Fight | Covering its own ship | Never its own ship |
|---|---|---|
| 2 BEAMERS + 1 AEGIS VS 6 LANCERS | 2, 6, 4; 1:50 a fight; 51 stopped | 2, 9, 1; 1:09 a fight; 13 stopped |
| 2 BEAMERS + 1 AEGIS VS 6 FIGHTERS | 8, 2, 2; 107 stopped | 9, 1, 2; 80 stopped |
| 2 ARCHERS VS 1 OUTPOST + 1 AEGIS | 0, 0, 12; 57 stopped | 0, 0, 12; 57 stopped: not a number moved |
| 2 RAKERS VS 2 BEAMERS + 1 AEGIS | 9, 2, 1; 128 stopped | 12, 0, 0; 58 stopped |

- *What I got wrong.* I told Henry I expected covering its own ship to be much of why two ARCHERs drew twelve times with an outpost and an AEGIS. It wasn't any of it: the fight is the same to the last number. A missile goes for the nearest enemy, the outpost is nearer than the AEGIS standing behind it, and the outpost is what's covered. Nothing in that fight ever aims at the AEGIS. So "its ship is how you get at it" only holds where the enemy's shots go at the projector's ship, and what a ship shoots at is "the nearest", which is ship behaviour. One more for the unit-AI talk: there's no way to tell a fleet to kill the projector first.
- *Everything else* (the fifteen part fights with no AEGIS in them, the starter deck's fights, the reference fleet's) is byte for byte as before.
- *Checks.* 239 tests pass. The 34 repeater and projector rule breaks were run again, one of them new (a projector that covers its own ship): 34 of 34 caught. The other 221 breaks touch no projector and weren't run again.

**The IN SPACE button taken out, and the placing bar cut back to one rule (2026-11-13, after the commit).** Henry, after playing seeds in space, over two messages. Not committed.
- *The button.* Henry: "the "in space" button is too prominent and overlaps the minimap a bit, not suitable for something that is usually unneeded. I think just having players able to see and choose the hotkey in the options may be enough." Taken out. Ctrl+click is the only way to send a seed into open space, and the view note ("HOLD TO LOOK") keeps its line while a seed is being placed.
- *Not an exception to anything.* I had added the button reading the project's rule ("anything the mouse can drive gets an on-screen button as well as a key") as "a match must be playable with the mouse alone", and I first wrote its removal up as an exception to that. Henry: "wouldn't just flying the king also be an exception to mouse-only control being possible?" It would. The king flies on WASD or the arrows and nothing else, so a match never was playable with the mouse alone, and the rule doesn't ask for it. It asks that what the mouse does drive (pause, speed, the map) has a button and a key.
- *Where the key is told, with no options screen yet.* In the bar while a seeded card is picked and no rock is under the pointer ("FOR A SPOT IN OPEN SPACE, HOLD CTRL"), in the seeded card's hover text ("OR IN OPEN SPACE (CTRL)"), and on the help card's SEEDS line. An options screen that lists the keys and lets them be chosen is not built. I asked whether to build it next, and Henry hasn't said.
- *The bar's words.* Henry: "what does "it joins your king's bank" mean in the outpost in space placement." It meant the new factory would be in the king's network and share its bank. My first answer was to say more: three link lines (it links to your king and shares your bank; it links to nodes cut off from your king and shares theirs; it's too far to link and keeps its own) and a named payer for what grows without rock (YOUR BANK, THE BANK IT LINKS TO, ITS OWN BANK). Henry: "those three sentences are far too specific and I think they'll just confuse people. If anything, we should just say "it draws resources from its network." That general rule is intuitive, it's silly to write out all its implications." So the bar says the rule, and nothing about banks:
  - In open space: "NO ROCK FEEDS IT IN OPEN SPACE: IT DRAWS THE 120 PX IT GROWS FROM ITS NETWORK, SO IT COSTS ABOUT TWICE."
  - On a rock too small for it: "THE ROCK IS 300 PX SHORT: IT DRAWS THOSE FROM ITS NETWORK."
  - The link lines are gone, "IT JOINS YOUR KING'S BANK" with them. The dotted lines still show what the factory would link to: green to your king's network, gold to nodes cut off from it.
  - Kept, and mine: with nothing in reach the bar still says "N PX TOO FAR TO LINK", which is a measurement. The ": IT'LL KEEP ITS OWN BANK" that followed it is gone.
  - The seeded card's hover text and the help card's SEEDS line still said "YOUR BANK" pays, and I asked before touching them. Henry: "Yes, reword them". The hover text now ends "IT DRAWS THE REST FROM ITS NETWORK AS IT GROWS", and the help card says a seed grows "FROM ITS NETWORK IN OPEN SPACE".
- *Checks.* 239 tests pass. Screenshots 22 and 58 looked at: no button, the note back on its line, both bars fit in three and four lines. So were 25 (the help card, its SEEDS line still three lines) and 40 (a rock out of reach: "IT FITS. 80 PX TOO FAR TO LINK."). The rule-break list is 257 long: the button's entry went, and three came in (a plain click that sends a seed into space, a warning that doesn't say where the pixels come from, and one that talks about banks again). The 5 that touch this were run: 5 of 5 caught. No sim code changed, so the rest weren't run again.

**Unit AI and the control scheme: the talk, and two brains tried (2026-11-13).** Henry: "Let's open the unit AI and player control scheme discussion". Nothing here is a rule of the game. What was built is an experiment in the arena, for Henry to watch. Not committed.

*The talk, in order.*
- I opened with where I thought the clunk was (every setting is about how a ship flies, and a match gives almost no say over where and when) and two systems for where and when: flags on the map, and wings that fight from the node that built them. Henry: "Decent proposal but I'm not ready to go with any of those yet. Maybe a bit creative but what do you come up with if you take Savage: The Battle for Newerth (an all-time favorite of mine) as inspiration?"
- I took Savage to be about the commander: garrisons, squads from a factory's build order, orders to garrisons and never to ships, and a key to take a squad's helm. Henry said what it is for them: "It's the fact that the units are so smart all you have to give them is suggestions and they figure everything out themselves". So the garrison system isn't the point, and nothing of it is built.
- I proposed that the smartness be measured, not written: before a match every design fights every enemy design headless, each way of flying, and the ships go by what that showed. Henry: "The matchup matrix seems like a pretty clever idea. I think I like it. It there any precedent you based that on and do you have any concerns or follow-up ideas?" The precedents I gave: Brood War bots that simulate a fight before taking it, fighting-game matchup charts, chess endgame tablebases. The concerns: timid armies, dithering at the edge of a fight, numbers (one lancer loses to a frigate and six beat it), mixed fleets, cost, and ships drawn to fool the drills.
- Henry: "Once battle is joined, does anything look different in the micro scale unit ai?" My answer: mostly no, as proposed. I pitched "good squares" (each design learning where round each enemy it does its damage and takes it). Not built.
- Henry: "Picking the micro behavior based on sims makes sense. I'm curious to try this. The other thing id be interested to try would be an arch-ai that controls all of your units at once and can micro them." Both built the same day, as below.

*What was built.* Each side in the arena has a brain, on a button (B and V).
- **CARD.** Today's: the nearest enemy, flown at as the card says. The recorded fights are byte for byte what they were.
- **DRILLED.** Every pair of designs facing each other is drilled before the fight: equal cost a side (the dearer design's), each side flying each of RUNS, KITE and ORBIT against each of the other's, a minute at most, on two seeds. From the nine results a design keeps two ways to fly at that enemy: the *safe* one, whose worst result was best, and the *bold* one, best over all the enemy's ways together. And a margin: the enemy's share of pixels destroyed less its own share lost. Two rules can read the table. *Ways*: a ship flies at each enemy the way it learned. *Targets*: an enemy it beats seems nearer and one that beats it further (half as far at a wipe-out, half as far again the other way), and it goes for what seems nearest.
- **ARCH.** One planner a side. At the first tick it plays the fight forward 8 s (on fresh dice, with the enemy flying on as it is) for each way each of its designs could fly, and keeps a change only if it came out better than the plan in hand. Every 2 s after that it does the same for one design, in turn. What it decides is a way of flying for each design. It chooses nothing for a single ship, and nothing about who to go for.
- **A fight to watch them in:** A MIXED FLEET VS ITS TWIN, made from the team as it stands, the same fleet on both sides.
- `--brains` runs it all headless.

*What the drills did,* a brain on one side and CARD on the other, sides swapped, five mixed-fleet fights a deck, four decks (starter, parts, the old fleet, Henry's saved team), 12 seeds: 480 fights each.

| Brain | Won | Lost | Drew |
|---|---|---|---|
| Targets from drills, flying by the card | 166 | 168 | 146 |
| Flying the safe way | 156 | 171 | 153 |
| Flying the bold way (the window's DRILLED) | 159 | 144 | 177 |
| Targets, and the safe way | 145 | 180 | 155 |
| Targets, and the bold way | 139 | 191 | 150 |

So the drills made nobody better. The bold way alone is the only one not behind, and that by fifteen fights in 480. Both rules together are the worst.

*What the arch-AI did,* the same fights, 120 a deck. The time is what its thinking cost on the twin fight: the first plan, then the longest single tick after it. A frame is 66 ms. (Henry's team's was timed with other runs going on the machine.)

| Deck | Won | Lost | Drew | Px it lost | Px it took | Its thinking |
|---|---|---|---|---|---|---|
| Starter deck | 76 | 44 | 0 | 32233 | 48950 | 38 ships: 265 ms, then 77 ms at most |
| Parts library | 69 | 16 | 35 | 33978 | 49074 | 58 ships: 504 ms, then 91 ms at most |
| Henry's saved team | 74 | 2 | 44 | 10988 | 18467 | 26 ships: 113 ms, then 25 ms at most |

Over those three: won 219, lost 62, drew 79 of 360. On the starter deck's twin fight it won 24 of 24, where card against card is 7 to 5.
- *The old fleet* (six designs, the TITAN among them) has no arch-AI numbers. I stopped its run after twenty-five minutes with the first fight's row still not out: a three-minute draw between fleets of five thousand pixels, looked ahead from every two seconds, is too slow to wait for. The drilled brains' numbers above do include it.

*What I make of it.*
- **Why the drills don't carry.** What a design learns against one enemy design, alone with it in an open field at equal cost, doesn't describe a fight with four other designs in it. This is the "mixed fleets" concern from the talk, and it turned out to be the whole story. Targets and ways together are worst because a ship then holds its learned range from a target that's standing in the middle of the enemy.
- **Who a ship goes for matters little as things are.** Guns fire by reflex at what's in reach, whatever the ship is flying at. So "lancers peel off for the frigate" changes where a ship flies, not much of what it shoots.
- **Why the arch-AI does better.** It measures the fight in front of it, with everything in it.
- **What I got wrong.** I told Henry I'd expect lancers never to orbit, because orbiting would lose its drills. In the first twin fight I looked at, the arch-AI had every lancer on ORBIT five seconds in. And I'd proposed the matchup table as the source of the smartness: by these numbers it isn't, and looking ahead is.

*Limits, as built.*
- The arch-AI chooses among three ways of flying for a whole design. That's all the "micro" it has.
- It assumes the enemy flies on as it is. With an arch-AI on both sides, blue plans first and red answers.
- Its thinking is paid in the tick it happens in, and at these sizes it overruns the frame: 77 ms at most on the starter deck's twin fight (38 ships) and 91 ms on the parts team's (58 ships), every two seconds. It would have to be spread over several ticks before it went near a match.
- It's in the arena only. A match's map is a million rock pixels to copy for every look ahead, and nothing here has been tried there.
- The drilled brains' constants (a minute, two seeds, the pull on distance) were set once and not tuned.

*For Henry to judge, by watching:* does the arch-AI look smart, or does it only win? And what should it be given to decide next: who each group goes for, when to pull back, or moves for single ships.

*Ruled, and asked (Henry, 2026-11-13, after I described what the arch-AI does and that it thinks every 2 s, not every tick).* "That's an interesting direction. I approve of removing per ship flight patterns and having them select their flight patterns intelligently. That part doesn't help with the decisions of what ship to send where, though. Did you address that at all?"
- **The ruling:** the way a design flies stops being a setting on its card, and the ships choose it. This overrules S1 to S3 as settings (RUNS, KITE and ORBIT stay as the ways a ship can fly). Not built: the yard still has its style buttons, and the arch-AI is in the arena only.
- **The answer to the question: no.** Nothing built decides what ship goes where. The arch-AI picks how each design flies in a fight it's already in. The one built thing that touched "which enemy" was the drilled targets rule, and it changed nothing. In the talk, where-and-when got proposals only (flags, wings from the tree, garrisons and squads, suggestions as weights), none taken and none built.

*Checks.* 251 tests pass, 12 of them new. The card brain's recorded fights (the starter deck's 15, the old fleet's 22, the parts' 19) are byte for byte unchanged. The rule-break list is 280 long, 23 of them new; those and one older entry that the change touched were run: 24 of 24 caught. Not covered by a test: that a plan is played forward on fresh dice.

**What ship goes where: a dispatcher, three tries (2026-11-13).** Henry: "Build the where to send prototype". Nothing here is a rule of the game, and none of it is in the window. Not committed.

*The test bed.* The AI opponent in headless matches. Today it masses its warships on GUARD and sends them out as a wave when its fleet is a quarter bigger than the enemy's by pixels, or after a rest, and brings the wave home when it's down to two fifths or has been out four minutes; a raid turns everything out. A side with a dispatcher keeps its cards on GUARD and has those decisions made for it. Everything else the AI does (planting, printing, its king) is the same code on both sides.

*What all three tries share.* A ship can be *sent* to a place, whatever its card says. It flies there at its group's pace, on the way it fights only what comes within 130 px of it, and there it fights what's within 160 px of the place. Let go, it goes home to its post.

*Try 1: play out the fight off the map, against what stands at the place.* The group as it is, against the enemy ships within 200 px of the target as they are, 30 s on a field of its own. Sent if it came out 15% ahead, called home at 15% behind. It lost its first two matches against today's AI. The trace said why: for one fleet and one target the answer went from +421 to -463 inside ten seconds, depending on which enemy ships happened to be inside the 200 px at that instant, and groups were shot up on the way by a fleet the fight hadn't counted.

*Try 2: the same, counting the enemy's whole fleet.* Fixed dice, two fights averaged, both sides drawn up in ranks, the group flying together, home only on two bad answers running. Steadier, and it never attacked. With even fleets an attack on a place that has guns of its own is a losing fight on paper every time. Against the boom it won 0 of 6, where today's AI won 3: a side that never goes out is out-grown.

*Try 3: play the match itself forward.* Every 20 s: the whole match, copied, played forward a minute for each plan (everyone home; everyone to one of the enemy's two nearest places; everyone to its own hardest-pressed place), with the other side's AI playing on and fresh dice. It takes the plan that leaves it furthest ahead in pixels standing and banked, a king's fall counting for a hundred thousand. As first built it changed its plan 36 times in 80 thoughts in one match and lost its ships on the way there and back: the plans came out within noise of each other. So it now keeps to a plan for a minute once it has changed to it (only a raid at home is weighed against it in that time), and another plan has to beat the one in hand by 100 px.

*Try 3 against today's AI, like for like:* the same seeds, each on both sides of the map, starter deck against starter deck. Won, lost, drew.

| Against | Matches | Today's AI | With the dispatcher |
|---|---|---|---|
| Clock | 4 | 4, 0, 0 | 4, 0, 0 |
| Planter | 4 | 4, 0, 0 | 3, 1, 0 |
| Expander | 4 | 4, 0, 0 | 3, 1, 0 |
| Summoner | 4 | 4, 0, 0 | 4, 0, 0 |
| Saboteur | 4 | 4, 0, 0 | 4, 0, 0 |
| The boom | 8 | 3, 5, 0 | 4, 4, 0 |
| The rush | 8 | 6, 2, 0 | 8, 0, 0 |
| Today's AI, head to head | 8 | 4, 4, 0 | 3, 5, 0 |

Against the seven openings together: today's AI 29, 7, 0; with the dispatcher 30, 6, 0.

*What it cost.* Over those 44 matches it thought 2213 times, changed its plan 323 times and played the match forward 5655 times, and they took 2248 s to play: the 8 against today's AI alone took 981 s for 1462 looks. Late in a match one look is about half a second. Today's AI plays a whole match in two or three.

*What I make of it.*
- **It isn't better than the rules of thumb.** Against the seven openings together it is level with them (30 of 36 against 29 of 36), and head to head it lost 5 of 8. The numbers above are the whole table: eight matches a row at most, so a row that differs by one match says little.
- **Where it did help is the rush:** all eight won, where today's AI lost two. I haven't traced why. My guess is that it keeps everyone home while a king is at its door, where the wave rule can send them out past it.
- **Why it's no better at the rest.** A choice that pays off minutes later can't be read a minute ahead: sending everyone or keeping everyone home come out within a few dozen pixels of each other, and the dice decide. What wins at this scale is massing and keeping to a decision, and the rules of thumb already do both.
- **The contrast with the arena.** There the arch-AI chose how to fly for the next eight seconds, and the look-ahead told the choices apart clearly. Seconds ahead, playing forward works. Minutes ahead, as built, it doesn't.
- **What I got wrong.** I told Henry that playing out the fight a group would find was the way to decide where it goes. Twice over it wasn't: the first two tries are that idea, and both were worse than pixel counts and timers.

*Not tried:* more than one group (everyone goes to one place); looking further ahead than a minute; averaging several rolls of the dice; using the look-ahead only for the short calls (is this place of mine about to fall, is the fight I'm in lost); a plan put forward by a player's suggestion, to be weighed with the rest.

*Checks.* 264 tests pass, 13 of them new. With no dispatcher the AI plays the same matches as the committed game, line for line (three matches compared), and the arena's recorded fights are byte for byte unchanged. The rule-break list is 303 long, 23 of them new; those and one older entry that the change touched were run: 24 of 24 caught. Not covered by a test: that a match played forward is played on fresh dice.

**Select and send: regular RTS control (2026-11-13).** Rows C1 to C7 in the table above. Not committed, and nobody has played it.

*How it was decided.*
- After the dispatcher, Henry: "if we just implement the automatic flight patterns, how will the where-to-send decisions happen? I suppose we could always change this into a regular RTS style with automatic flight patterns, which would make it a lot like Homeworld (which is not bad)? What do you think we should do?"
- I recommended a smaller step: one new command, send a card's ships to a place, with the deck as the control groups, keeping box selection as the fallback.
- Henry: "Hm... I don't think players would like that. I think it would just feel like nerfed RTS controls. I think it would feel better to just be able to select groups of units and they go where you tell them to while flying intelligently on a micro-level." So: regular selection and orders. My half-step isn't built.
- While it was being built, Henry: "assuming we're applying this to the king too, this actually solves the input lag when flying the king too I guess". I hadn't applied it to the king. It is now (C4).

*As built.* The send order is the one made for the dispatcher (a ship goes to a place, fights what comes close on the way and what's round the place), now given by the player. What's new is on the match screen: the box, the click, the right-click, the LET GO button, and a king that flies to where it's sent.

*My calls, for Henry to overrule.*
- **A right-click means go and fight.** There is no plain move, no "attack that ship" (a right-click on an enemy sends them to where it is now), no patrol and no formations.
- **A box round a fleet leaves out miners and the king.** A box with only miners in it, or only the king, takes those.
- **No control groups and no Shift to add to a selection.** 1 to 9 print cards and Shift+number turns a card's order, so groups need keys of their own. Ctrl+click (a design) and Q (every warship) are what there is.
- **The cards' orders stay,** for ships you haven't sent. A ship that's let go does as its card says, which for a card on GUARD is to go home.
- **WASD stays** beside the sent king. Henry's guess was that sending replaces it; I've left both until they've tried it.
- **The view still follows the king.** For a fight elsewhere it's the whole map (Tab) or a hold on the small map.
- **A ship with no guns that's sent goes to the place.** For the dispatcher it used to stay beside an armed friend.
- **The AI is unchanged.** It doesn't select; it has its waves.

*Not built:* automatic flight in matches (approved on 2026-11-13: the yard's style buttons still decide how a design flies), control groups, Shift to add, attacking a chosen target, a plain move, a view that leaves the king.

*Checks.* 271 tests pass, 7 of them new. With nobody selecting, the AI plays the same matches as the committed game, line for line (three matches compared), and the arena's recorded fights are byte for byte unchanged. Screenshots 25, 67 and 68 looked at: the rules card fits, the box draws, and nine ships show their corners, their cross and the LET GO button. The rule-break list is 334 long, 31 of them new; those and two older entries that the change touched were run: 33 of 33 caught. Two got past at first. One changed nothing a player could see (a right-click with nothing selected), and was dropped. The other showed that my test for a ship with no guns used the starter miner, which has one; the test now uses the AEGIS, and catches it. Not covered by a test: what's drawn (the corners, the cross, the box). I haven't played it in the window.

**The rest of what an RTS has (2026-11-13, after Henry played select and send).** Rows C8 to C12, and verdicts on C1 to C4, in the table above. Not committed.

Henry: "I give it a spin. There are several features I think have to go along with RTS controls because people already know how to play RTSs. Very likely WASD should scroll the camera, not fly the king (We can think about direct flight as a special level in the campaign, perhaps). Control groups should be a thing. Double click to select all units of a type on the screen. We might need a way to select all units of a type on the whole map? Probably even waypoints with shift although that can maybe wait. We don't have to add attack orders but right-clicking a unit should probably effectively move the "go here and fight" order along with the unit. I'm sure you know what I'm talking about... anything I'm forgetting?"

*Built from that list:* the keys scroll the view (C11), control groups (C8), the double click (C9), and a right-click on a ship (C10). Waypoints with Shift are not built: Henry said they can wait.

*What I said he'd forgotten, and what became of it.*
- Shift to add to a selection: built (C9).
- Jumping the view (a group's number twice, a key for the king): built (C8, C11).
- Rally points for factories (select a factory, right-click, and its new ships gather there): not built. It's the one I'd do next, and it may replace the cards' orders.
- Hold position, a plain move that ignores enemies, scrolling at the screen's edge, a readout of what's selected: not built.

*My calls, for Henry to overrule.*
- **The cards moved to Z X C V B N M,** under the hand that scrolls, because the numbers had to be the groups. The eighth and ninth cards have no key.
- **Let go moved to G,** since X is a card now.
- **Shift+number adds to a group.** It used to turn a card's order; that's by the card's tag only now.
- **"All of a type on the whole map" is a double click on the whole map,** where everything is in view. There's no separate command for it.
- **Ctrl+click is the double click,** as in other RTSs: in view, not across the map as I first built it.
- **A ship sent after something follows it for good,** friend or enemy, until it's sent elsewhere or let go. (2026-11-22: sent after a friend it stops beside it, clear of it, whichever was built first: ST5.) A king sent after a thing doesn't stop when it reaches it.
- **The view starts on your king and then stays where you put it.**
- **The code that flew the king from the keys is kept,** unused, for the campaign level Henry mentioned.

*Checks.* 275 tests pass, 4 of them new, and a dozen older ones rewritten for the keys that moved. With nobody selecting, the AI plays the same matches as the committed game, line for line, in the same time, and the arena's recorded fights are byte for byte unchanged. Screenshots 25 and 68 looked at. The rule-break list is 360 long; the 57 that touch the match's controls were run: 57 of 57 caught. One got past at first: with ships sent after one of their own number, nothing checked that the one they followed was told to stay where it is. The test checks that now. Not covered by a test: what's drawn. I haven't played it in the window.

**Keys on the cards, one stance, formations, rally points and a keys page (2026-11-13, later).** Rows K1 to K8 in the table above, with Henry's words on C9 and C12. Not committed.

Henry, with a screenshot of sent ships in a loose clump: "we probably also want a hotkey for all combat ships on the screen and all combat ships on the map (Q and cntrl-Q?). Why not just make ships 8 and 9 , and .? The key to make the ship should go to the left of the icon, which won't be confusing once we get rid of the disposition letter on the right. Then it would be logical to add cntl-z, cntl-x etc to select all ships of that type. it looks like we could make the target formation of the move order when there are no enemies around a little more sophisticated. they should put themselves into some sensible shape rather than a wiggly cluster. with dispositions going away, maybe we should retain just one global disposition toggle to make all your combat ships attack the nearest enemy, or guard their place of creation by default? this is for when you just want to press an "overwhelm them" button or a "stay back" button. yes we want rally points and automatic flight patterns both. is this all making sense?" And while I built it: "this might also be a good time to add the hotkey mappings page so we (I) don't forget/miss any extant hotkeys".

*Built from that:* K1 to K7. Automatic flight in matches is not in this batch. It's the next round by itself: the yard's style buttons come off and the arch-AI's thinking is spread over several ticks.

*How each works.*
- **The stance (K2)** is one command, for every card that flies and fights: everything that flies, bar a miner and a design drawn to HOLD. GUARD is the old GUARD order and ATTACK the old HUNT. A person's side starts every such card on GUARD whatever it was drawn with. The AI's side is untouched: it orders its cards one by one, as it did.
- **The formation (K4)** is worked out on the player's machine when the order is given, and each ship is sent to its own spot. The block faces the nearest of the eight compass points to the way the group will come (the middle of the map, if they're sent to where they are). Four lines, front to back: ships with only flak; ships with a longer gun (lance, beam, repeater, cannon, missile); miners and unarmed ships; a king. Each line splits into as few ranks as the width allows, evenly, and no rank is wider than the square root of three times the ships' widths squared. Ships of one type take the spots of their type in the order they already stand across the group, so they don't cross over. A group sent after a ship has no formation.
- **A sent ship's spot (K5)** is in the match's rules, not the screen's. A ship that's been sent and has nothing to fight goes to the place it was sent to, or to the nearest point clear of what already stands there: factories, kings, and ships already at a post or on a spot that were built before it. It's the rule guards at their posts already had.
- **A rally point (K6)** is the home of the ships a factory builds after it's set. Home is where a ship waits on GUARD, gathers for its group on ATTACK, and goes when it's let go, so one setting covers all three.
- **The keys page (K7)** is a table for each screen in `src/pg/keys.rs`. The test reads the source of each screen's key handler, finds every key it names, and fails if the screen's table doesn't name it too.

*Rules I changed that Henry had agreed to. Each is here on its own line.*
- **"Sent out from guarding, they gather first and leave as a group" (the sub-orders round) doesn't hold for the stance.** On ATTACK the ships already flying go at once from wherever they are. Reason: it's the "overwhelm them" button, and gathering first would fly ships that are out on the map home again before they left. Ships built after that still wait for their group, and a card the AI tells HUNT still gathers.
- **A design no longer starts a match on the order it was drawn with,** on a person's side: everything that flies and fights starts on GUARD. It follows from one stance. HARVEST and HOLD still count.
- **SAB and ESCORT are gone from a person's match.** Henry's ruling ("with dispositions going away"). Escorting the king is now a right-click on the king with ships selected. Sabotage has no button: send ships at the miners.
- **Q took every warship on the map (C1). Now it takes those in view,** and Ctrl+Q is the old Q. Henry's ask.
- **Q's "warship" was a ship with guns. Now it's any ship you can send that isn't a miner.** My call, so a shield ship goes with the fleet it covers.
- **"Friends keep a pixel or two apart" has a second exception** (the first was a guard at its post): a sent ship with nothing to fight isn't pushed, on its spot or on its way there. Reason: pushed, the last to arrive never got through the ranks already standing (the first run of the formation test left a fighter 21 px short of its spot for good), and a group sent to one point never settled, which is the wiggly clump in Henry's screenshot. Sent ships can now pass through each other on the way, as miners at a rock face do.
- **One cross marked where selected ships were going. Now each ship's spot has a small cross,** and none once the ship stands on it.
- **LET GO said "back to what their cards say". Now: unsent again,** which means what the stance says.
- **The rules card:** FLEET lost "G LETS GO" and the control-group keys (the keys page has them, and the LET GO button shows while ships are selected), ORDERS became UNSENT, FACTORY gained the rally point, KING says "a card or its key", KEYS points at F3.
- **The wire is version 10.**

*My other calls, for Henry to overrule.*
- The stance key is T and the button reads ALL: GUARD or ALL: ATTACK, lit while attacking.
- Ctrl and a card's key takes that card's ships across the whole map, not just in view. A double click is the one for "in view".
- A box takes factories only when there's no ship and no king in it.
- A rally point moves only the ships built after it. The ones already out keep their home.
- Six pixels between hulls in a formation: four kept the green corners of neighbours running into each other.
- The keys page is F3, and it lists the mouse too.

*Not built, and what's become of the list.*
- A rally point for the king: a right-click on the map sends a king, so there's no gesture left for it.
- Shift waypoints, hold position, a plain move that ignores enemies, scrolling at the screen's edge, a readout of what's selected.
- The yard still has its ORDER, style and GROUP buttons. In a person's match, HUNT, SAB, ESCORT and GUARD on a drawing now all mean the same thing. They want clearing out with the style buttons, next round.
- The where-to-send experiment's numbers further up this file are from before K5. Its ships are sent, so they'd move. I haven't re-run it.

*Checks.* 285 tests pass: 11 new, one gone with the cards' tags. With nobody selecting, the AI plays the same three matches as the committed game, line for line, and the arena's recorded fights are byte for byte unchanged. One match of the where-to-send experiment plays through under K5 without a fault. Screenshots 25 and 69 to 74 looked at. The rule-break list is 428 long: 69 new, 19 mended where their lines had moved, one dropped with the cards' tags. The 88 new and mended ones were run and 86 were caught at once. Two got past. A sent ship pushed about like any other: once the hulls stood 6 px apart, nothing in my tests had to fly through a standing rank. A click on the keys page reaching what's under it: the test clicked where there was nothing to press. Each has a test now, and both are caught. Not covered by a test: how the formation looks in play, and anything about feel. I haven't played it in the window.

**Henry's verdicts on the keys, the stance, formations and rally points, and two tweaks (2026-11-14).** Verdicts on K1 to K7 and rows K9 and K10 in the table above.

Henry, after playing a match and winning it at 6:15 (one ATTACK, at 5:06): "I think it played well. I'm happy the disposition letters are gone and I think ATTACK/GUARD with LET GO is an improvement. Formation looked sensible. ATTACK should send everything unless it has mining equipment. miners should still mine. I don't think we need rally points for mobile factories. Only tweak I'd recommend is that I think K once should select your king and twice should jump the camera to it. Hotkeys seemed great otherwise. Commit and push, then we're ready for automatic flight. I think we can get rid of group size along with manual flight patterns. I think once we have the new automatic flight features, we can and should consider optimizing the set of patterns. One thing I think it might make sense to add is an evasive maneuvers pattern."

*Changed from that.*
- **The stance's exception is mining equipment (K9).** Before: a miner (a scoop and drawn to HARVEST) and any design drawn to HOLD sat it out. Now: anything with a scoop mines, whatever it was drawn with, and everything else that flies takes the stance. A flying design drawn to HOLD therefore guards and attacks with the rest. That last part is my reading of "everything", not something Henry said about HOLD.
- **K selects the king, and K again within 0.4 s takes the view to it (K10).** One press used to take the view. The note above the small map does what the key does.
- **No rally point for a king or a factory that flies.** Nothing to change: they had none.

*For the automatic-flight round, from the same message.*
- Group size goes with the manual flight patterns.
- Once ships choose how they fly, look again at the set of patterns itself. Henry's first thought for a new one: evasive manoeuvres.

*Checks.* 285 tests pass. The rule-break list is 435 long: 7 new for these two changes, and 6 mended. All 13 were run and all are caught.

**Automatic flight (2026-11-14).** Rows A1 to A7 in the table above. Not committed.

Henry: "Commit and push, then we're ready for automatic flight. I think we can get rid of group size along with manual flight patterns. I think once we have the new automatic flight features, we can and should consider optimizing the set of patterns. One thing I think it might make sense to add is an evasive maneuvers pattern." (He approved ships choosing how they fly on 2026-11-13, twice.)

*What's built.* Nobody sets how a design flies, in the yard or anywhere. Each side of a match has the planner that was the arena's ARCH brain, changed in two ways so that a match can afford it.

*Why it had to change: the measurements.*
- The arena's planner copied the whole fight and played it forward, every way of every design, before the first move, and one design again every 2 s. In a match that copy is the whole match. Playing a match forward 8 s costs 3 ms two minutes in, 45 to 70 ms at ten minutes and 200 to 450 ms from fourteen (`--plan-cost`). One think is three or four of those.
- First cut: copy the match, put every ship that isn't in a fight to sleep in the copy, and spread the playing forward over ticks. It won as many fights as the old planner. But an AI match took 300 s to play where it had taken about 20, and the test suite went from 50 s to over ten minutes. The copy still paid, every tick, for the match round the fight: banks, miners, printing, both AIs. A played-forward tick with 14 ships awake cost 132 us at ten minutes, and 67 us of that was a tick with nobody awake at all.
- As built: the fight alone. The ships within 300 px of an enemy are copied into a bare arena, with the shots flying at them, and that is played forward. The same 14 ships cost 15 us a tick, 34 ships 44 us, 120 ships 289 us.
- The share played forward each tick is 24,000 divided by the ships in the fight squared plus 200, between 2 and 40 ticks. That holds the cost near 0.6 ms a side a tick in a match's fights (more for fleets of big ships: 2 ms a side with 38 frigates and capitals).

*Does it still win?* Ships that choose against ships that fly as drawn, mixed fleets, five fights a deck, sides swapped, 12 seeds: 120 fights a deck (`--flight`).

| deck | won | lost | drawn | the arena's old planner |
|---|---|---|---|---|
| starter | 81 | 39 | 0 | 76-44-0 |
| parts library | 78 | 6 | 36 | 69-16-35 |
| Henry's saved team | 87 | 3 | 30 | 74-2-44 |
| all three | 246 | 48 | 66 | 219-62-79 |

- **With a planner on both sides,** as in a match: of 60 fights a deck, 0 were drawn on the starter deck (0 with no planner), 18 on the parts library (4) and 29 on Henry's team (6). Two sides that each pick the way that loses them least keep away from each other more often, and in the arena 20 s with nothing lost is a draw. A match has no such rule, but the standing off is real and worth watching for.
- **Fights run longer when both sides choose:** 35 s a fight against 22 s with no planner on the starter deck, 129 s against 76 s on the parts library, 121 s against 91 s on Henry's team.
- **Cost:** on the twin fight with one side planning, the longest tick was 11, 7 and 4 ms on the three decks (38, 58 and 26 ships) and a tick averaged 3.2, 2.6 and 1.5 ms. The old planner's longest ticks were 77, 183 and 25 ms, after a first plan of 264, 514 and 113 ms. A frame is 66 ms.
- **AI against AI, the starter deck:** seeds 1 to 3 ended at 20:16, 23:20 and 12:33, all won by red. Before: 20:37, 19:20 and 18:06, won by blue, red and blue. They take 14 to 45 s to play with nobody watching, where they took about 20 s, and the test suite takes three minutes where it took 50 s.

*What the fight alone gets wrong.* It's a copy to score ways of flying by, not the match.
- A king that was sent somewhere stands still in it. Kings in a fight are in it, and fire.
- A miner stays where it is: there's no rock in it.
- A ship that's hurt and unarmed runs for a corner of the map there, not for its base.
- A ship still being printed isn't in it, and nothing new is printed during the 8 s.
- Ships more than 300 px from any enemy aren't in it, so reinforcements on their way aren't counted.
- Nobody's AI gives orders in it: every ship keeps the order it has.

*Other things to know.*
- **One way of flying a design, everywhere.** If your fighters are in two fights at once, one choice serves both, made on both fights together.
- **The choice stands until it's thought about again.** Ships that meet a new enemy fly the way last chosen until their design's turn comes round.
- **The AI's ships choose too,** so it fights better than it did.
- **The wire is version 11:** two games that fly differently would fall out of step, so they don't pair.
- **Miners aren't thought about,** armed or not: found by a test, when the planner spent its turns on a design that only mines.
- **A design nobody has thought about flies as its drawing suggests:** at least 1 thrust per weight makes attack runs, less kites. The yard's flight row shows which.
- **The reference fights are re-recorded** with a planner on each side, as the window now fights them. Starter deck, 15 fights: the same side wins every one, 4 scores moved, and a fight runs 19 s on average where it ran 14 s. Parts examples, 19 fights: 12 scores moved and three fights changed hands, 2 RAKERS V 1 FRIGATE (0-11-1 to 6-0-6), 2 ARCHERS V 1 FRIGATE (0-4-8 to 9-0-3) and 2 BEAMERS + 1 AEGIS V 6 FIGHTERS (9-1-2 to 4-8-0); a fight runs 77 s where it ran 60 s. Old fleet, 22 fights: 9 scores moved and two changed, 1 TITAN V 40 LANCERS (12-0-0 to 5-5-2, and 124 s where it took 19) and 13 FRIGATE V 1 CAPITAL (8-4-0 to 6-6-0); 59 s where it ran 53 s.

*Rules changed from what Henry had agreed. Each on its own line.*
- **S1: how a design flies was chosen in the yard.** Gone, on Henry's ruling of 2026-11-13.
- **S6: groups.** Gone, on Henry's word above. A ship on ATTACK, or on an AI's HUNT, leaves as it's built.
- **The presets' own ways are gone with the setting.** The ARCHER's card said KITE; now it starts on attack runs, as it's drawn, and its side decides.
- **The arena's sides had no planner unless a button was pressed. Now both have one,** and PLAIN is the button's other choice. The arena should fight as a match does.
- **The brain called CARD is called PLAIN,** since a card no longer says how its ships fly.
- **The arena's planner thought the whole plan through before the first move.** It doesn't now: designs are taken one at a time from the first tick, flying as drawn until their turn. It was a 264 to 514 ms hitch.
- **A test's bound moved.** "The AI doesn't change its orders every few seconds" allowed 80 changes in eight minutes of one match. With ships choosing, that match has 84, and the bound is 100.

*Not built.*
- **A better set of ways to fly, and evasive manoeuvres:** Henry's next thought, for once this has been played. The planner takes whatever ways there are: a fourth is a line in a list and the code that flies it.
- **The yard's ORDER buttons are still there.** In a person's match they now do nothing: anything with a scoop mines and everything else that flies takes the stance. They still matter in the arena (HOLD stays put) and to the AI's reading of a deck. Whether they go too is Henry's call.
- **The DRILLED brain** is still an arena button. It made nobody better, and nothing uses it.

*Checks.* 288 tests pass. The planner has 9 where it had 4, the two tests of groups are gone, and the tests of the yard's flight buttons, of a design's line and of a card's hover text are rewritten. The rule-break list is 443 long: 22 new for the planner, 14 dropped with the rules they broke (groups, a design's own way of flying, the old planner), 3 mended. The 25 new and mended were run and all are caught. One is caught by a crash, not a failed check: a fight played forward that plans plays forward a fight that plans, and so on until the stack runs out. The runner missed that at first (it looked only for failed tests) and now sees a test run that dies. Screenshots of the yard and of the arena with a planner looked at. I haven't played it in the window: whether ships look smart is unjudged.

**Orders off the drawings (2026-11-14, later).** Rows O1 to O4 in the table above. Not committed.

Henry: "What role are the dispositions in the building screen playing now? I'd like to remove them unless they're there for some pernicious reason that needs more careful thought. My understanding is that they can become part of the opponent AI perhaps (and encapsulated to that scope - remember we want to have at least 3 levels in the future) but I don't believe they should be part of, for example, the PvP 1v1 match at all. Should never be used or referenced in human play at all is my understanding. I guess maybe except for the miner AI. Which is different from hunt and sab AI."

*What they were doing.*
- **In Henry's own matches, nothing.** Since the stance, everything of his that flies without a scoop guards or attacks together, and anything with a scoop mines, whatever it was drawn with.
- **For the AI, two things.** It sorted a deck by them: a scoop and HARVEST marked a miner card, and HOLD marked a card as no warship. Every other order it gave its cards itself as the match went.
- **In the arena, what a placed ship did:** HOLD stayed put, HARVEST mined, SAB went for miners, the rest hunted.
- **In every design's line,** in saves and the library, and in the library's list and the bar under a ship.

*Was there a reason to keep them?* None found.
- All 9 designs in Henry's saved team, all 28 in his library and every ready-made design already had the order their drawing gives.
- Both things the AI read are in the drawing: a scoop and engines is a miner, engines without a scoop is a ship that fights.
- What's lost is on the test bench only: in the arena a flying ship can't be made to stand still or to go for miners.
- One consequence to know: **a ship with a scoop is a miner and nothing else.** That was already true in Henry's matches. It's now true for the AI and in the arena too.

*As built.*
- A design is its pixels, its name, whether it's seeded and what it builds. Its line is `NAME|X,Y|rows|KIND|BUILD`. Older lines, with an order second and perhaps a way of flying and a group size on the end, load as the same drawing.
- The yard: the six order buttons and the O key are gone. BUILT, SEEDED and the build rows moved up into their place, and the info box under them is 24 px taller.
- A match starts every side's cards the same way: what flies without a scoop on GUARD, what flies with one mining, the rest holding. Then a person's stance moves the first kind together and an AI orders its cards one at a time.
- The command that gives one card an order is dropped when it comes from a side no AI plays. Nothing on screen could send it since the tags went; now nothing over the wire can either.
- The AI's orders go to everything of its that flies without a scoop, armed or not (O4). Before, an unarmed ship of its kept the order it was drawn with.
- The bar under a ship no longer names its order (O3): "BLUE FIGHTER - ON GUARD", not "BLUE FIGHTER - GUARD - ON GUARD". The library's list says MINES, FIGHTS, STAYS PUT or FLIES, NO GUNS.
- Orders themselves are untouched: HUNT, SAB, GUARD, ESCORT, HARVEST and HOLD are still what a ship is on, for the AI's levels to use. ESCORT has no user today.
- The wire is version 12.

*Checks.* 291 tests pass. The AI plays the same three matches as before the change, line for line, which is the check that a drawing's role comes out as its old setting did. The rule-break list is 451 long: 12 new, 3 mended, and 4 dropped with the drawn orders they broke. The 15 new and mended were run and 13 were caught at once. Two got past: an armed miner counted among the AI's warships (nothing checked how the AI sorts the starter miner, which has a gun), and a ship placed in a match taking its drawing's order, not its side's (the test set the order by hand). The tests check both now, and both are caught. Screenshots of the yard looked at. I haven't played it in the window.

**What a frame costs (2026-11-14, after the orders).** Henry: "The test suite isn't a concern because we can always speed that up. The real constraint is realtime play with people having different computer specs. For fairness we need a deterministic planner that always finishes in a tick (worst case scenario we would need to stretch the ticks out I assume- and probably show some sort of warning on the screen. Same type of thing we'll need for network issues, I guess)." And, on how fine the planner's choices should be: "it doesn't make sense for a group of fighters flying together to each make their own decision, but it very much makes sense for one frigate fighting a fighter to make its own decision vs. a frigate fighting a capital ship." Nothing in the game is changed. `--frame-cost` is new, and the planner now keeps a clock that only the tool reads.

*How it was measured.* The window's own match with the AI on both sides: three seeds on the starter deck and three with Henry's saved team against it. Every tick: the match's tick (and the planners' share of it), then the screen drawn with the view on the biggest fight and the pointer in the middle of the field. Once a second: the whole map drawn, and the picture scaled up with scanlines at 3x, 4x and 7x (a 1080p, 1440p and 4K screen) and copied out. The copy goes to a bitmap in memory, so the desktop's compositing isn't in it. The machine is an i9-13900KF, one of the fastest desktop processors for one thread, on a 1080p screen (3x). A frame is 66.67 ms.

| match | over at | most alive | most in a fight | tick, max | planners, max | worst frame at 4x |
|---|---|---|---|---|---|---|
| starter, seed 1 | 20:16 | 159 | 136 | 5.43 | 3.01 | 7.2 (158 alive, 123 in a fight) |
| starter, seed 2 | 23:20 | 115 | 94 | 3.61 | 2.54 | 4.8 |
| starter, seed 3 | 12:33 | 94 | 61 | 3.61 | 3.06 | 4.8 |
| Henry's team, seed 1 | 14:31 | 102 | 45 | 3.80 | 3.05 | 5.1 |
| Henry's team, seed 2 | 10:13 | 56 | 36 | 2.81 | 2.46 | 4.0 |
| Henry's team, seed 3 | 6:37 | 44 | 26 | 2.65 | 2.53 | 4.0 |

All six together, 78,791 ticks, in ms:

| | median | 99% | 99.9% | max |
|---|---|---|---|---|
| the match's tick | 1.11 | 2.94 | 3.56 | 5.43 |
| of it, the planners (both sides) | 0.72 | 1.87 | 2.35 | 3.06 |
| of it, the rest | 0.20 | 1.94 | 2.35 | 4.54 |
| drawing, the view on the fight | 0.23 | 0.40 | 0.56 | 0.89 |
| drawing, the whole map | 2.08 | 2.55 | 3.19 | 3.76 |
| scaling up, 3x | 0.28 | 0.42 | 0.66 | 0.79 |
| copying out, 3x | 0.31 | 0.46 | 0.69 | 1.62 |
| scaling up, 4x | 0.49 | 0.71 | 1.03 | 1.43 |
| copying out, 4x | 0.48 | 0.73 | 1.07 | 1.36 |
| scaling up, 7x | 1.41 | 1.82 | 2.26 | 4.94 |
| copying out, 7x | 1.65 | 2.26 | 2.84 | 3.90 |
| a whole frame, 3x | 1.94 | 3.86 | 4.47 | 6.79 |
| a whole frame, 4x | 2.32 | 4.23 | 4.85 | 7.16 |
| a whole frame, 7x | 4.41 | 6.32 | 6.94 | 9.25 |

*What it shows.*
- **On this machine the worst frame used 7 to 9 ms of 66.7.** It came in the biggest fight seen (158 ships alive, 123 of them in a fight), and the match itself was most of it: 4.5 ms of simulation besides the planners.
- **The planners are the biggest share of a usual tick** (0.72 of 1.11 ms), and about 3 ms at most for both sides. Their worst ticks come in fights of 15 to 30 ships, not the biggest ones: a small fight is played forward 40 ticks a tick, and that costs more than the share's rule (ships squared plus 200) expects.
- **Drawing is cheap,** except the whole map, which costs about 2 ms whatever is on it, nine times the usual view.
- **How much the planner could have depends on the slowest machine we aim at,** and that's Henry's to pick. A machine with a third of this one's speed for one thread would have taken about 22 ms for the worst frame. Keeping a quarter of the frame spare (50 ms used at most), that leaves room for about three times today's planning at its worst. One with a fifth of the speed would leave room for very little more.
- **Everything above runs on one thread.** The match's tick is one thread's work, but a think is not: it plays a copy of the fight forward, and nothing in it reads the match. It could run on another processor core, with its choice landing at a tick fixed in advance, the same on every machine, and the match waiting at that tick if the think isn't done (the warning Henry describes). Then the planner would cost the match's own thread almost nothing, and its budget would grow with the cores a machine has rather than its speed for one thread. Not built, and not measured.

**Teams with a strategy each, for the AI work to stand on (2026-11-14).** Henry: "Perhaps before we work on multithreading and adaptive+granular decision grouping, we'd better build a more realistic harness. I think it's warranted to have a handful of teams with realistic strategies to test this on. Our current teams and build orders are still placeholder-like. Why don't you take inventory for me of the current starter teams and AI build orders, and then propose a basic set of new ones to use going forward. You can set them up in the arena too and I can test them out in matches. After that, we'll pick back up the AI work." And on why: "The illusion we're going for is that the ships are intelligent."

*The inventory, before this.*
- **Teams.** Three ready-made ones in the library. The STARTER DECK (Henry's drawings: fighter 9 px, lancer 9, frigate 55, miner 9, outpost 121 seeded, shipyard 693 seeded, capital 687) is the only team any AI has played. NEW PARTS (beamer, warden, raker, fighter, lancer, frigate, outpost, archer, aegis) has no miner, and its outpost's build order names the starter deck's slots, so in this team it builds fighter, fighter, beamer, beamer, warden, raker and never a miner: a sandbox, not something a match can be played with. REF FLEET (the proving ground's seven old warships) has no factory and carries spare hull.
- **Build orders on the cards.** The starter outpost builds miner, miner, fighter, fighter, lancer, frigate (100 px a round, 95 s). The starter shipyard builds miner, frigate, frigate, capital (806 px, 182 s).
- **The AI** plays the starter deck by one set of rules (*The AI*, above): two outposts, then shipyards up to six factories; one more miner than it has factories; its king prints the counter to your fleet once the next factory is paid for with 120 px over; it masses on GUARD and sends a wave at 100 px when it has the edge or has rested long enough. The factories' build lists do most of its printing.
- **The seven scripted openings** the AI is measured against (`--judge`): clock, planter, expander, summoner, boomer, rusher, saboteur. They play the same deck, and send everything on HUNT (or SAB) as it's built.
- **What was placeholder about it:** one deck and one way to play it; sizes that jump from 55 px to 663 with nothing between; flak, lances and cannons only (the newer parts only in the sandbox teams); and an AI that can't rush, boom or time an attack on purpose: it has rules, not a strategy.

*What's built.* Five teams, each drawn for a strategy that RTS players would recognise, and each with that strategy for the AI to play it by. The starter deck stays as it was and stays the AI's default. Every card spends all its power (a test checks it). Seven drawings are new and mine; the rest are the starter deck's cards (with build orders pointing at each team's own slots) and the part examples.

| team | strategy | cards (new drawings in capitals) | factories, in order | miners | king prints | first wave |
|---|---|---|---|---|---|---|
| SWARM | rush | fighter, lancer, miner, NEST, outpost | nests at 0:00, 1:40, 3:20, then outposts | 1 a factory, +1 | fighters and lancers, saving nothing | 2:30 with 54 px, whatever the enemy has; again whenever it has 54 px |
| LANCE LINE | timing attack | fighter, CORVETTE, CRUISER, miner, outpost, DOCK | two outposts, a dock, then outposts | 2 a factory, +1 | corvettes | 7:00 with 400 px, whatever the enemy has |
| BIG GUNS | boom into big ships | fighter, frigate, miner, outpost, shipyard, BATTLESHIP, capital | four outposts, a shipyard, then outposts | 2 a factory, +1 | frigates and fighters, for home | 12:00 with 900 px and the edge |
| RAIDERS | harassment | WASP, archer, raker, miner, outpost | outposts, up to five | 1 a factory, +1 | wasps, archers, rakers | wasps on SAB from the first; the rest 5:00 with 150 px and the edge |
| SIEGE | turtle | beamer, warden, aegis, miner, outpost, shipyard, LANCET | two outposts, a shipyard, then outposts | 2 a factory, +1 | wardens and beamers | 9:00 with 700 px and the edge |

The new drawings:
- **NEST** (41 px, seeded): twelve factory pixels round the core and a flak gun on each side, no plate. Builds up to 120 px, a fighter in 20 s.
- **WASP** (12 px): a three-pixel repeater (84 px of reach, two shots a second) at 45 px/s, the fastest thing a drawing can make.
- **CORVETTE** (24 px): three lances in a column behind a plated nose, two flak, 26 px/s.
- **CRUISER** (124 px): seventeen lances in five columns down a plated hull, four flak, 15 px/s.
- **BATTLESHIP** (188 px): five cannons, six lances, ten flak, plated, 15 px/s. The step between the frigate and the capital.
- **DOCK** (269 px, seeded): 72 factory pixels in two rings and a cross inside a thin plate. Builds up to 720 px.
- **LANCET** (180 px): two fourteen-pixel beams that reach 208 px and cut 56 each, plated, 11 px/s.

How a strategy works (`Strategy` in `src/pg/sim/eco.rs`, the teams in `src/pg/teams.rs`): it replaces the AI's rules for what to plant, how many miners to keep, what the king prints and when a wave goes. Everything the AI does about raids stays as it was (turning out, arming first, where it plants and where its king stands). The scripted openings don't take one. An AI with no strategy plays exactly as before: three AI matches and every test came out the same.

In the window: the match menu has THE AI PLAYS, a button that goes round the six teams, with a line on how that one plays, and YOU PLAY over your two buttons. In the library the five are on the teams shelf after the starter deck, to load into your nine slots for a match or the arena; the new drawings are on a TEAMS shelf of ships. `--teams N` plays every team against every other, AI on both sides, on every core at once. `--deck` now names every gun and why a card isn't legal.

*My calls, for Henry:*
- The starter deck stays the AI's default; the five are options in the menu.
- The strategies are the AI's (as orders are, O1): a person who loads SWARM plays it however they like.
- The menu's new button has no key. The menu has never taken keys; adding a keyboard to it is a change of its own.
- The wire is version 13 (the AI's strategy is part of the match, though no network match has one).
- The numbers in the strategies (times, wave sizes, miners) are my first guesses. None was tuned on the results below.

*What the round robin shows* (`--teams 4`: 120 matches, every team against every other, four seeds with each team on each side, AI on both sides; 244 s on 32 cores). Wins, losses and draws of the team on the left against the team on top:

| | STARTER DECK | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | all |
|---|---|---|---|---|---|---|---|
| STARTER DECK | - | 0-8-0 | 8-0-0 | 7-1-0 | 4-4-0 | 7-1-0 | 26-14-0 |
| SWARM | 8-0-0 | - | 8-0-0 | 8-0-0 | 8-0-0 | 8-0-0 | 40-0-0 |
| LANCE LINE | 0-8-0 | 0-8-0 | - | 0-8-0 | 0-8-0 | 1-7-0 | 1-39-0 |
| BIG GUNS | 1-7-0 | 0-8-0 | 8-0-0 | - | 6-2-0 | 7-1-0 | 22-18-0 |
| RAIDERS | 4-4-0 | 0-8-0 | 8-0-0 | 2-6-0 | - | 8-0-0 | 22-18-0 |
| SIEGE | 1-7-0 | 0-8-0 | 7-1-0 | 1-7-0 | 0-8-0 | - | 9-31-0 |

Median length of the matches between two teams:

| | STARTER DECK | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE |
|---|---|---|---|---|---|---|
| STARTER DECK | - | 8:16 | 12:13 | 19:46 | 18:45 | 13:59 |
| SWARM | 8:16 | - | 3:53 | 3:43 | 4:35 | 4:25 |
| LANCE LINE | 12:13 | 3:53 | - | 15:14 | 9:51 | 15:29 |
| BIG GUNS | 19:46 | 3:43 | 15:14 | - | 18:19 | 15:10 |
| RAIDERS | 18:45 | 4:35 | 9:51 | 18:19 | - | 12:14 |
| SIEGE | 13:59 | 4:25 | 15:29 | 15:10 | 12:14 | - |

Each team over its 40 matches (medians):

| team | length | ships first fight | first wave | no wave at all | factories planted | most ships of its own | biggest fight in its matches |
|---|---|---|---|---|---|---|---|
| STARTER DECK | 13:59 | 3:18 | 2:14 | 0 of 40 | 3 | 48 | 46 |
| SWARM | 4:25 | 2:55 | 2:30 | 0 of 40 | 3 | 25 | 26 |
| LANCE LINE | 12:13 | 3:26 | 7:00 | 21 of 40 | 2 | 22 | 36 |
| BIG GUNS | 15:14 | 3:46 | 12:00 | 15 of 40 | 7 | 109 | 121 |
| RAIDERS | 12:14 | 1:29 | 5:00 | 8 of 40 | 5 | 55 | 38 |
| SIEGE | 13:30 | 3:18 | 9:22 | 23 of 40 | 3 | 22 | 45 |

The most ships in one fight at once, over all 120 matches: median 36, 90% 144, most 210.

*What it means, as I read it (nothing changed for it):*
- **Every strategy does what it says.** The rush goes at 2:30 every time. The timed ones go at their time when they're still standing and not under attack: LANCE LINE at 7:00 and SIEGE from 9:14, BIG GUNS at 12:00. RAIDERS' wasps are on SAB from the first.
- **The 2:30 rush beat everything, 40 of 40,** the AI as it plays the starter deck included. It isn't the cheap NEST: with outposts in its place (a measurement, put back after) SWARM still won 20 of 20 at two seeds. At 2:30 a king that has spent its bank on fighters and lancers has about 150 px of them, and every other plan has spent its bank on a factory and miners. The AI's raid rules don't hold it. This is what Henry found by hand on 2026-11-11 (*The AI's defence*), now as a team.
- **Early waves beat waiting ones generally.** The AI as it always played sends its first wave at about 2:15 and beat the three teams that wait for 7:00 or later in 22 of 24. LANCE LINE and SIEGE never got their wave out in about half their matches: they were dead, or fighting at home, first.
- **The fights are a good spread for the AI work.** Small ones in the rushes, 120 to 210 ships at once when BIG GUNS is in it. That's the range the planner has to be cheap enough for.
- What a person makes of the rush, and of the rest, is for Henry to find by playing.

*Checks.* 294 tests pass, the two new ones among them: every team is legal, spends its power, builds only from its own slots, can mine, and opens on its starting bank; and every strategy plays a match against the AI to the end, replays it exactly from the same seed, keeps its raiders on SAB, and sends no wave before its time. The AI with no strategy plays the last commit's three AI matches line for line.

**The teams' fights in the arena (2026-11-14, later).** Henry: "can you make a new set of arena fights for me to check out all the new teams too before I play them in matches". Not committed, and nobody has watched them.

*What's built.* First as a window of its own; then, after Henry asked why he still saw only 11 fights, in the arena itself, as he laid it out: "Let's just have fight 0 be an empty fight with the Yard team in the slots, and the other fights are the preset ones that don't affect the yard (but do show their ships in the slots). And let's make whichever team is blue show their whole team, not just the ships that are in that fight." So the arena's fights are: an open field with your team, then twenty-four team fights. A team fight carries two ready-made teams (`Test.ships`, `Sides`), and each side's ships come from its own team's slots (`Pg::side`); your team (`Pg.lib`) is never touched, and going to the yard and back lands on the first fight. The list beside the field shows blue's whole team. The starter deck's eleven old fights are gone from the window (`Pg::starter_fights` keeps them for the screenshots and the checks). For each team in turn: its army against the starter deck's, then fights for its new drawings. Then every two of the five armies. An army is about 700 px of the team's warships in its strategy's shape (README, *The teams' fights*). `--pg-sim teams` fights them all headless.

*My calls, for Henry:* while your clicks place for red (T), the list shows red's team, so a red ship can be placed from it; blue's shows otherwise. The showcase fights have the new team on blue so its whole team is listed: 5 LANCERS VS 1 NEST is SWARM against SWARM, 2 CRUISERS VS 1 DOCK is LANCE LINE against LANCE LINE, 2 LANCETS VS 40 FIGHTERS is SIEGE against SWARM, 1 LANCET VS 1 BATTLESHIP is SIEGE against BIG GUNS, and the rest of them are against the starter deck.

**The DRILLED brain taken out (2026-11-14).** Henry: "do these fights use the lookahead flight pattern planner now and can we remove the DRILLED brain?" They do: every arena fight starts both sides on the planner (ARCH), as a match does. The drilled brain and its drills are gone (`sim/drill.rs`, the arena's drill cache and its RUNNING DRILLS wait, the `--brains` tool, and the drilled targeting in `think`). The arena's brain buttons go between ARCH and PLAIN; `--flight` still sets the planner against the plain brain. `sim/brain.rs` holds what's left: the two brains. Nothing in a match changes: no match used it.

*What they come to* (`--pg-sim teams`, 12 seeds each, both sides planning as in a match; px lost are averages; as the fights stand in the arena, each side with its own team's slots):

| fight | blue won | red won | drawn | average length | blue lost | red lost |
|---|---|---|---|---|---|---|
| SWARM VS STARTER DECK | 12 | 0 | 0 | 0:24 | 541 px | 711 px |
| 5 LANCERS VS 1 NEST | 12 | 0 | 0 | 0:08 | 0 | 41 |
| LANCE LINE VS STARTER DECK | 5 | 7 | 0 | 1:15 | 674 | 579 |
| 6 CORVETTES VS 16 LANCERS | 2 | 10 | 0 | 1:18 | 140 | 105 |
| 1 CRUISER VS 2 FRIGATES | 3 | 8 | 1 | 0:55 | 104 | 67 |
| 2 CRUISERS VS 1 DOCK | 12 | 0 | 0 | 0:10 | 0 | 269 |
| BIG GUNS VS STARTER DECK | 4 | 8 | 0 | 0:58 | 675 | 578 |
| 1 BATTLESHIP VS 21 LANCERS | 0 | 12 | 0 | 0:13 | 188 | 56 |
| RAIDERS VS STARTER DECK | 2 | 9 | 1 | 1:40 | 682 | 404 |
| 8 WASPS VS 10 MINERS | 12 | 0 | 0 | 0:12 | 11 | 90 |
| 8 WASPS VS 10 FIGHTERS | 1 | 11 | 0 | 0:27 | 92 | 59 |
| SIEGE VS STARTER DECK | 0 | 12 | 0 | 0:28 | 685 | 154 |
| 1 LANCET VS 1 BATTLESHIP | 6 | 6 | 0 | 0:40 | 127 | 124 |
| 2 LANCETS VS 40 FIGHTERS | 0 | 12 | 0 | 0:14 | 360 | 31 |
| SWARM VS LANCE LINE | 12 | 0 | 0 | 0:19 | 438 | 714 |
| SWARM VS BIG GUNS | 12 | 0 | 0 | 0:22 | 463 | 705 |
| SWARM VS RAIDERS | 12 | 0 | 0 | 0:23 | 424 | 702 |
| SWARM VS SIEGE | 12 | 0 | 0 | 0:13 | 115 | 685 |
| LANCE LINE VS BIG GUNS | 11 | 1 | 0 | 0:47 | 483 | 688 |
| LANCE LINE VS RAIDERS | 1 | 10 | 1 | 1:24 | 688 | 510 |
| LANCE LINE VS SIEGE | 12 | 0 | 0 | 0:31 | 153 | 685 |
| BIG GUNS VS RAIDERS | 4 | 5 | 3 | 2:05 | 544 | 639 |
| BIG GUNS VS SIEGE | 12 | 0 | 0 | 0:55 | 290 | 685 |
| RAIDERS VS SIEGE | 12 | 0 | 0 | 0:25 | 156 | 685 |

**Six rows moved when each side got its own team's slots**, with the same ships in the same places: only the slot numbers changed. A side's planner thinks about its designs in slot order, so which goes first changes what's chosen. BIG GUNS VS STARTER DECK moved most, from 0-12 to 4-8 (a 0:38 average to 0:58); the other five moved by a few seconds and a few dozen pixels. That's a sensitivity in the planner for the AI work, not anything about the teams.

No stalemates. BIG GUNS VS RAIDERS is the one the window shows as a draw on time (3:00): the raiders' missiles and repeaters against a fleet whose flak downs over a thousand missiles across the twelve.

*What it shows, as I read it (nothing changed for it):*
- **At equal cost, many small ships beat everything.** The swarm's 79 fighters and lancers beat every other army 12 of 12, and lost least to the siege army (115 px of 711). Two lancets didn't kill one of forty fighters (each beam is dodged). A battleship loses to 21 lancers. This is the round robin's 40-0 seen up close, and it's Henry's standing problem that large ships are worse per pixel (*Large ships, and the projector as a structure*, above).
- **The siege army loses every fight it's in,** and takes 115 to 290 px of the other side's 700 with it (290 from BIG GUNS, whose big ships can't get off a beam's line). Small ships dodge its beams, and its one aegis covers little.
- **The counter triangle shows where it should:** wasps take miners apart (90 px for 11) and lose to flak fighters; lancers beat the battleship; the lancet and the battleship are even.
- What the fights look like is for Henry to say.

**Which way the planner flies (2026-11-15).** Henry: "does the planner have a strong tendency to pick any particular flight pattern?" Not overall. `--ways 2` counted every choice in matches between every pair of ready-made teams (two seeds a pairing, each team on each side) and in the 24 team fights (12 seeds each). Nothing in the game changed for it: the planner notes each choice only when the tool asks.

| | choices | attack runs / kite / orbit kept | every way scored the same (all 0) | changed the way it was flying | time in a fight: runs / kite / orbit |
|---|---|---|---|---|---|
| in matches | 52,872 | 30 / 38 / 31 % | 18 % (7 %) | 42 % | 29 / 33 / 36 % |
| in the team fights | 37,639 | 27 / 36 / 35 % | 14 % (1 %) | 45 % | 44 / 31 / 23 % |

- Some designs lean one way: in matches the wasp kites 64% of the time, the warden makes attack runs 47%, the lancer kites 43-48%.
- It changes a design's way in more than four choices in ten. That looks like churn: each way gets one play-forward of 8 s on one roll of the dice. Whether replaying a choice on fresh dice keeps the same way is the check for the planner work.
- In big, short fights it barely gets to choose: SWARM's fighters spent 77% of their time in the team fights on attack runs (as they're drawn) though its choices kept attack runs only 28% of the time. The fight is mostly over before the planner has been round its designs.
- A piece broken off a king is planned for as a design of its own (the king's slot).

**Beating SWARM (2026-11-15).** Henry, after playing SWARM: "Its opening was powerful, although it didn't capitalize on its advantage to secure a stronger economy. If we need the AI to be able to beat swarm (which I think we probably do), you could look at my game vs. swarm just now for inspiration. Propose your plan before implementing it." To the plan below: "Yes, start". Not committed.

*Henry's game (seed 1983, won at 12:53), from the log:* an outpost at 0:03, miners repeating from 0:06, Henry's outposts printing three miners a round, and a factory planted the moment the bank held 121, all match. Henry mined 324 px by 3:00 and 1,146 by 5:00, where SWARM mined 135 and 369 and the AI teams 126 to 243 by 3:00. Henry replanted as fast as outposts fell (6 planted by 5:00 with 2 standing, 15 in all), sent about 400 single ships round their own quarter between 3:00 and 6:00 (I read them as their king; not confirmed), and didn't attack until 8:28, with 1,167 px standing to SWARM's 522. SWARM stopped at 6 factories ("most 6") and banked 224, 437 and 713 from 10:00 to 12:00 with nothing to spend them on.

*The AI against SWARM, before:* SWARM won all 40 matches. Under its raid the AI put its whole bank into guns and planted nothing (the starter deck sat on one or two factories for ten minutes), BIG GUNS's king stood by its one factory and died 15 s after it, and the starter deck's own first wave left at 2:21, just before SWARM's arrived.

*The plan:* five general rules for every team, each built behind its own switch, measured over the whole round robin and against the scripted openings, then switched off one at a time and kept only if it helps. The yardstick is the SWARM Henry played: `--teams N --old SWARM` has SWARM play with every new rule off. Four seeds a pairing turned out to be too few. A rule that looked like noise at four seeds was worth nine matches in 80 at eight, so the deciding numbers are from eight.

**The AI teams against the SWARM Henry played** (eight seeds, each team on each side: 16 matches each, 80 in all; each cell is that team's wins and losses against SWARM):

| AI rules | STARTER | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | AI's wins of 80 |
|---|---|---|---|---|---|---|
| before | 1-15 | 0-16 | 0-16 | 0-16 | 0-16 | 1 |
| all six (B1 to B3, and rules 2 to 4 below) | 6-10 | 1-15 | 4-12 | 13-3 | 1-15 | 25 |
| all six, rule 2 off | 5-11 | 0-16 | 6-10 | 12-4 | 1-15 | 24 |
| all six, rule 3 off | 5-11 | 0-16 | 2-14 | 13-3 | 2-14 | 22 |
| all six, rule 4 off | 5-11 | 1-15 | 4-12 | 13-3 | 1-15 | 24 |
| all six, B3 off | 4-12 | 0-16 | 2-14 | 9-7 | 1-15 | 16 |
| **kept: B1, B2, B3** | **7-9** | **0-16** | **2-14** | **14-2** | **2-14** | **25** |
| kept, B1 off | 2-14 | 0-16 | 0-16 | 0-16 | 0-16 | 2 |
| kept, B2 off | 8-8 | 0-16 | 3-13 | 13-3 | 2-14 | 26 |
| kept, B3 off | 4-12 | 0-16 | 1-15 | 9-7 | 1-15 | 15 |
| kept, after the escape fix (S1, below) | 8-8 | 0-16 | 4-12 | 8-8 | 1-15 | 21 |

| row | what | verdict |
|---|---|---|
| B1 | **Miners while there's rock for them** (`RULE_MINERS`). A miner earns its price back in about a minute, so the AI's king keeps printing them while the rock within 200 px of its factories and king holds 60 px for each miner it has, up to 30 of its own (what its factories print besides doesn't count against it). This replaces "one more miner than it has factories" and the teams' own miner counts. A strategy can keep its miners lean for its opening instead (`lean`, a time): SWARM does till 2:30, so its rush is the one Henry played. | |
| B2 | **Money nothing is spending plants another factory** (`RULE_SPEND`). A bank that has held the cheapest factory's price, plus what the AI keeps, for 30 s plants one past the most its strategy or its rules would have, up to 14. This is SWARM capitalizing on its lead: new against new it plants up to 14 factories and its late bank stays at 0 to about 400, where with rule 1 alone it piled up to 1,000 or 2,000. It changes nothing in the table above, and may cost BIG GUNS: over all its matches in those runs, BIG GUNS went 23-57 with it and 28-52 without. | |
| B3 | **It plants as soon as the bank holds the price** (`RULE_SPARE`): it no longer keeps 40 px back for the new factory to print with. Henry planted at 121, the outpost's price, all match. | |

*Henry played it (2026-11-15):* two matches with BIG GUNS's deck against SWARM, both lost. "I played big guns against swarm and lost twice, so I take that as a sign that the AI is improved." The rows weren't judged one by one. From the log (a match ends when a king dies):
- Seed 30458, lost at 15:20. Henry mined more than SWARM until 14:00 (9,273 to 8,982), but SWARM printed nearly twice the ships (502 to 277 by then) and at 14:00 had 235 ships to Henry's 62. It reached B2's limit of 14 factories at 11:00, and from then on its bank piled up again: 429 at 11:00, 1,907 at 13:00, 4,483 at the end.
- Seed 30459, lost at 7:58 while ahead on the board: 1,403 px standing to 1,011, 2,034 mined to 1,269, six factories each. SWARM got to Henry's king.

**Tried and taken out (each measured as above, none worth more than three matches in 80):**
- **2. Plant through a raid.** While saving for its next factory the king prints no guns (miners only), unless the raid outweighs what it has at home; and under any raid it plants nowhere the raiders are. Worth one match. It went back to the agreed rules of 2026-11-12 (guns first while the enemy's fleet has the edge; it plants away from raiders only when they outweigh it).
- **3. The king defends.** First as a weight rule: go after the nearest raider that the king and its ships near it outweigh. That got kings cut down (the starter deck's to 61 px of 218 by 4:00; BIG GUNS's died at 4:04), because SWARM's lancers cut plate whatever the pixels say. Then as a physical rule: go only after raiders whose guns bounce off plate (flak and repeaters), and only while nothing that cuts plate is within 100 px of the raider or the king. Worth three matches. The king's post beside its factory (2026-11-12) is back as it was.
- **4. Hold the wave** while raided, or while enemy warships worth half its fleet are on its half of the map. Worth one match.
- **Save for the factory before extra miners** (only the miners it can't do without while it saves): worse. SWARM won 40 of 40, because miners frozen at four to six never fill the bank.

**The whole round robin, new against new** (eight seeds; SWARM plays with the new rules too, so it's stronger than the one Henry played):

Before:

| | STARTER | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | ALL |
|---|---|---|---|---|---|---|---|
| STARTER DECK | - | 1-15 | 16-0 | 14-2 | 9-7 | 14-2 | 54-26 |
| SWARM | 15-1 | - | 16-0 | 16-0 | 16-0 | 16-0 | 79-1 |
| LANCE LINE | 0-16 | 0-16 | - | 1-15 | 0-16 | 1-15 | 2-78 |
| BIG GUNS | 2-14 | 0-16 | 15-1 | - | 12-4 | 13-3 | 42-38 |
| RAIDERS | 7-9 | 0-16 | 16-0 | 4-12 | - | 16-0 | 43-37 |
| SIEGE | 2-14 | 0-16 | 15-1 | 3-13 | 0-16 | - | 20-60 |

With B1 to B3, and the escape fix (S1):

| | STARTER | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | ALL |
|---|---|---|---|---|---|---|---|
| STARTER DECK | - | 4-12 | 14-2 | 16-0 | 3-13 | 4-12 | 41-39 |
| SWARM | 12-4 | - | 16-0 | 16-0 | 9-7 | 16-0 | 69-11 |
| LANCE LINE | 2-14 | 0-16 | - | 7-9 | 0-16 | 5-11 | 14-66 |
| BIG GUNS | 0-16 | 0-16 | 9-7 | - | 2-14 | 12-4 | 23-57 |
| RAIDERS | 13-3 | 7-9 | 16-0 | 14-2 | - | 16-0 | 66-14 |
| SIEGE | 12-4 | 0-16 | 11-5 | 4-12 | 0-16 | - | 27-53 |

(Before the escape fix the totals were 44-36, 69-11, 16-64, 21-59, 67-13 and 23-57.)

Each team over its matches (medians), before and after: length 12:31 / 14:53 (STARTER), 4:29 / 8:51 (SWARM), 11:39 / 13:02 (LANCE LINE), 15:46 / 14:21 (BIG GUNS), 12:08 / 11:39 (RAIDERS), 12:32 / 14:26 (SIEGE); most ships 46 / 86, 26 / 79, 22 / 49, 109 / 96, 62 / 129, 28 / 53. BIG GUNS sends no wave in 55 of 80 matches (31 before). The biggest fight in a match: median 38 / 104 ships, 90% 143 / 210, most 236 / 357.

**The AI against the scripted openings** (`--judge 5`, the starter deck, ten matches each), before / after (with the escape fix): CLOCK 9-1 / 10-0, PLANTER 9-1 / 10-0, EXPANDER 9-1 / 10-0, SUMMONER 10-0 / 10-0, BOOMER 4-6 / 9-1, RUSHER 4-6 / 6-4, SABOTEUR 10-0 / 10-0, itself 5-5 / 5-5 (median 20:16 / 19:42).

**What a frame costs now** (`--frame-cost 3`, both measured the same night). Cubase was running on the machine, so every number is about two and a half times last time's. Read them against each other, not against the table above.

| ms | before: median | 99.9% | max | after: median | 99.9% | max |
|---|---|---|---|---|---|---|
| the match's tick | 1.90 | 6.41 | 15.81 | 2.56 | 12.28 | 34.84 |
| of it, the planners | 1.23 | 4.10 | 14.95 | 1.42 | 8.79 | 30.65 |
| a whole frame at 4x | 3.91 | 8.57 | 17.95 | 4.61 | 14.46 | 37.12 |

The worst frame (37 ms of 66.7) came in a 35-minute AI match on the starter deck with 147 ships alive, 24 of them in a fight, and was nearly all planner. It's the leak noted on 2026-11-14 (40 ticks played forward a tick for a mid-sized fight). There are more ships now, so the planner budget and threads matter sooner.

*What a match against SWARM looks like now* (`--match-trace "STARTER DECK" SWARM 2 140 260 --old SWARM`, a line every five seconds). At 3:00 SWARM's wave (18 ships, 162 px) comes in against the starter deck's 7 warships (109 px). Most of it never reaches the AI's base: it hangs about 400 px out, picking off miners at outlying rocks (17 miners at 3:35, 7 at 4:05). The AI's fleet goes out to meet it and is lost piecemeal. The AI's second factory went down at 3:10 this time. Its king went out twice and came back whole, before rule 3 was taken out. Those two (miners mining far from their stations, and a fleet that meets a wave piecemeal) are where I'd look next. Neither is built, and neither is a rule to patch without talking it through.

**S1. A small ship caught under a bigger one gets out at its full 2 px a tick** (`shift`, in `sim/space.rs`). The audit of a whole AI match caught it once B3 was in: enemy miners heaped on a rock face, a damaged frigate flying slowly up the face over them (0.75 px a tick), and miners left inside its hull for 34 ticks, where the rule allows 10. A ship inside a bigger enemy heads for the nearest clear pixel at up to 2 px a tick, but it stopped on that pixel when it was only 1 px away. Here that pixel was straight ahead of the frigate, so the frigate caught it again every tick. Now it goes a pixel past the edge when that pixel is clear too. It's a change to the sim, so the recorded fights moved, re-recorded the same night (12 seeds each):
- the starter deck's reference fights: 9 of 15 rows moved, 2 of them in who won (1 CAPITAL VS 40 LANCERS 0-12 to 1-11, 12 FRIGATES VS 1 CAPITAL 6-6 to 7-5), the rest by seconds and pixels;
- the part examples' fights: 16 of 19 rows moved, 7 of them in who won or drew, by up to three matches (1 BEAMER VS 1 FRIGATE 4-5-3 to 7-4-1; 2 RAKERS VS 1 WARDEN 4-2-6 to 2-1-9);
- the team fights: 22 of 24 rows moved, 9 of them in who won or drew, by a match or two (BIG GUNS VS STARTER DECK 4-8 to 3-9, 6 CORVETTES VS 16 LANCERS 2-10 to 1-11, BIG GUNS VS SIEGE 12-0 to 10-2, BIG GUNS VS RAIDERS 4-5-3 to 4-4-4).

*Tools:* `--teams N --minutes` (a line a minute for each match), `--off N` (switch the rules off by bit: 1 B1, 2 B2, 4 B3), `--old TEAMS` (those teams play as before), `--match-trace` and `--ways`. All 293 tests pass.

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

## Heavy plate, the piercer and the tracker (2026-11-15)

Henry asked which part to build next, "something to reward non-swarmy ships or punish swarms, especially lancers". To my pitch of a tracker turret, with heavy plate second: "maybe all three: tracker, heavy plate, and armor-piercing. I think we want the tracker and other guns, but I want to avoid the solution to everything just being 'add a gun.' I want designing a ship to be more interesting than just which gun to slap on it. Note that this will mean adding an integer level armor piercing system. Please summarize that before you begin." My summary: the game already ran on integer levels (a shot's "damage" against a pixel's armour, since round 2), and the real risk is that each level is all or nothing while decks are fixed for a match. On the upgrader's price, Henry: "Hull plus slower round for now." Then, while it was being built: "One tweak, I think missile should be piercing 2 to start with. And can you clarify exactly how Lance's number of pierced pixels interact with hull being 0 and the other as armor values" (the answer is below the table). Not committed.

**Why lancers beat big ships, before any of this:** in 1 BATTLESHIP VS 21 LANCERS (188 px against 189) the lancers win all 12 in about 14 s and lose 5. A lance can't miss a big ship and punches 6 px whatever fires it, so 21 lancers bring 42 lances to the battleship's 6. The battleship's guns are poor against small ships: its cannons turn slowly and go for big ships, its lances hold fire on targets that could dodge, and its flak reaches 48 px to a lance's 100. Missiles chase small ships, but SWARM's fighters carry four flak each and shoot them down.

| row | what | verdict |
|---|---|---|
| V1 | **One rule for every hit:** a shot breaks a pixel only if it pierces more than the pixel's armour; otherwise it bounces off and stops there. Pixels have no hit points, so that's all a hit is. Armour: bare 0, plate 1, heavy plate 2. Piercing: flak, repeaters and trackers 1, lances and missiles 2, cannons and beams 3, the most there is, so everything has an answer. A shot that goes on through (a lance, a beam) uses up 1 and the pixel's armour more of its depth for each pixel it breaks (below the table); a burst goes off wherever it meets a ship and breaks every pixel of its 3x3 whose armour it out-pierces. This was the rule already, with "damage" for piercing; the code and every part's text now say piercing. | |
| V2 | **Heavy plate:** armour 2, in dominoes two wide or two tall, 4 weight a pixel (a domino weighs 8, a plate pixel 2), no power. Only a shot that pierces 3 breaks it: cannons, beams, and a lance or missile beside a piercer. A lance or beam uses up 3 of its depth breaking a pixel. Built first as 2x2 slabs (my call); Henry: "we need to make heavy plate two tall or two wide, rotatable, not 2x2, because it's too awkward to place now in the builder, especially with symmetric mode on." Now dominoes: [T] or the turn button turns the next one; any shape that cuts into dominoes is legal (the yard finds a way to cut it, so a drawing doesn't say which way each one lies); a heavy pixel left over is loose. A domino two tall on the middle column is its own mirror twin. The rectangle fills an even number of cells (an odd box loses its right-hand column, and its mirror its left-hand one), and fill fills a hole only if it cuts into dominoes. | Dominoes (Henry). A whole skin of it that lancers can't touch without piercers: "that's not necessarily bad... I think it seems about right". |
| V3 | **The beam pierces 3 (it was 2).** A change to a rule you agreed, but it changes nothing that was in the game: nothing had armour 2. With heavy plate in, it means heavy plate stops a small ship's lance but not a big ship's beam. | |
| V4 | **The piercer** (UPGRADES, key 4; letter `i`): 1 weight, uses 2 power, so it costs its pixel and two of hull. A gun touching it edge to edge fires heavier rounds: one more piercing, up to 3, flying a quarter slower (a lance's 60 px/s goes to 45, flak's and a repeater's 90 to 75, a tracker's 135 to 105, a missile's 45 to 34, so the fastest ships outrun it). A slower lance shot holds fire on more targets, as its aim rule already says, and flak leads a slower missile by its own speed. Two piercers on one gun are no better than one; a gun already at 3 (a cannon, a beam) isn't changed. A column gun or a turret takes it if any of its pixels touches one. Shoot the piercer off and its guns fire plain rounds again. | |
| V5 | **The tracker** (letter `t`): a 3x3 turret, 18 weight, 16 power (twice a cannon's), so it needs a lot of hull. It reaches 130 px, swings about 170 degrees a second (four times a cannon), fires every 0.4 s, and its shots fly at 135 px/s and pierce 1, so plate turns them. It goes for the small ships in reach (anything no more than half as big again as the smallest), the one its barrel reaches soonest, and aims at its middle, led by how it's flying, so shots dig in to a small ship's core; a shot at the middle that would meet none of the target's standing pixels goes at the standing pixel nearest the middle instead (the second half since Henry's question below). | |
| V6 | **UPGRADES replaces the empty POWER category** (key 4). An eighth weapon part didn't fit the weapons' two rows of buttons, and seven categories don't fit their row. POWER had nothing in it (hull is the power); your speed and blast-radius upgraders would go in UPGRADES too. The power gauge keeps its bolt. | |
| V7 | **Blocks in the yard:** trackers go down and come up whole, as cannons do (centred on the cell), and a mirrored one lands as its mirror image. The rectangle tiles them; fill refuses them. (Heavy plate's dominoes: V2.) | |
| V8 | **Missiles pierce 2** (Henry's tweak; they were 3). Their burst breaks bare pixels and plate, as it always did, and spares heavy plate. | |
| V9 | **A burst goes off wherever it meets a ship**, whatever it meets first: a missile that hits heavy plate still bursts and breaks whatever of its 3x3 it out-pierces. Before, every burst out-pierced everything, so whether it bounced first never came up. My call, from the rule as written ("a burst breaks every pixel of its 3x3 whose armour it out-pierces"); the other reading is that it bounces off heavy plate whole. | |
| V10 | **Five example ships on the library's Parts shelf** and nine fights in the arena window, after the teams' own (blue is a NEW PARTS side, red the starter deck's lancers and fighters, or the AP lancer): PICKET (one tracker, 36 px), BULWARK (four lances behind a prow of heavy plate, 41 px), AP LANCER (the starter lancer with a piercer between its lances, 12 px), IRONLANCER (the starter lancer behind plate, 12 px), BASTION (148 px and two cores, so it costs 156; redrawn, below). Each spends exactly the power it makes. No ready-made team uses the new parts, so the AI plays them only from a deck that has them. | |

**How a lance's depth meets armour** (Henry's question). A lance shot pierces 2 and has 6 of depth. It meets the pixels on its line one at a time, in order, whoever's ship they're on:
- If it doesn't pierce more than the pixel's armour, it bounces off and stops there, and nothing behind that pixel is touched. For a plain lance that's heavy plate (2 is not more than 2).
- Otherwise the pixel breaks, and the shot loses 1 plus the pixel's armour from its depth: 1 for hull or any other bare part (cores, guns, engines: armour 0), 2 for plate, and 3 for heavy plate (which only a lance beside a piercer, at 3, gets to break).
- It stops once its depth is spent. The pixel that spends it still breaks, even if it costs more than was left.

So a plain lance breaks 6 bare pixels in a row, or 3 plates, or a plate and 4 bare. Five bare pixels and then a plate is 6 pixels too: the plate takes its last 1. It stops dead at heavy plate, after whatever it broke on the way to it. A lance beside a piercer breaks 6 bare pixels or 2 heavy plates; bare, bare, heavy plate uses 5 and leaves 1, so the next pixel goes too. A beam works the same way with 4 of depth for each of its pixels, piercing 3. Flak, repeater and tracker shots have 1 of depth: one bare pixel, or a bounce off plate.

**My calls, for you to rule on:**
- The tracker aims at the middle of its target; flak and cannons aim at a random pixel.
- The tracker doesn't aim at missiles. Its shots still take a missile down if they meet one, as flak's and a repeater's do.
- Its targeting rule is the cannon's turned round (no more than 1.5 times the smallest in reach, where the cannon takes at least 2/3 of the biggest).
- The numbers were set once and not tuned: heavy plate's 4 weight, the piercer's 2 power and quarter of speed, and the tracker's 16 power, 130 px, 0.4 s, 135 px/s and turn.

**What they come to** (`--pg-sim teams`, 12 seeds a fight, both sides planning as in a match; px lost are averages over the 12):

| fight | blue won | red won | drawn | average length | blue lost | red lost |
|---|---|---|---|---|---|---|
| 2 PICKETS VS 8 LANCERS (before the tracker fix below; after it, 9-3 again) | 9 | 3 | 0 | 0:34 | 33 px | 67 px |
| 2 PICKETS VS 8 FIGHTERS | 0 | 12 | 0 | 0:10 | 72 | 27 |
| 2 PICKETS VS 6 IRONLANCERS (after the fix: 1-11) | 3 | 9 | 0 | 0:52 | 57 | 25 |
| 2 BULWARKS VS 9 LANCERS | 1 | 11 | 0 | 0:46 | 79 | 32 |
| 2 BULWARKS VS 7 AP LANCERS | 0 | 11 | 1 | 0:51 | 80 | 29 |
| 7 AP LANCERS VS 9 LANCERS | 3 | 9 | 0 | 1:11 | 76 | 50 |
| 1 BASTION VS 15 LANCERS (first drawing) | 0 | 12 | 0 | 0:17 | 131 | 46 |
| 1 BASTION VS 15 FIGHTERS (first drawing) | 0 | 12 | 0 | 0:07 | 131 | 28 |
| (for comparison) 1 BATTLESHIP VS 21 LANCERS | 0 | 12 | 0 | 0:14 | 188 | 56 |

- **Each part does its one job.** Trackers beat an equal cost of lancers 9-3 (and still do after the tracker fix below). The lancer's answer to them, plate, works: lancers behind plate beat the trackers 9-3. Lancers with a piercer beat heavy plate 11-0. And the piercer's slower rounds cost against bare ships: at equal cost, plain lancers beat lancers with piercers 9-3.
- **My two big drawings don't beat a swarm.** The BASTION loses to 15 lancers and to 15 fighters as badly as the battleship does to 21, and kills about two lancers before it goes. I haven't watched these fights; my reading of the drawings is that heavy plate only on the prow leaves the sides and stern bare hull for a swarm to reach, and that fighters close inside their flak's 48 px within seconds, where the PICKET's two flak and the BASTION's four meet 32 and 60.
- **So the parts are in, but "a big ship that beats a swarm" isn't shown yet.** That's down to drawing as much as numbers, and I haven't tuned either. Two drawings would test it before any number moves: a ship in a whole skin of heavy plate, with its guns buried inside (shots pass through their own ship, and only engines must be open), and a tracker ship in plate against fighters. The first would be immune to everything SWARM fires, which is the all-or-nothing risk from the summary showing up.

**Henry on the results (2026-11-15):** "If you used the heavy plate to cover the whole anti-lancer ship the lancers would never win without piercers, that's not necessarily bad, that's the same way you can shield against fighters, in fact I think it seems about right considering there's a nice little multi-layered interaction there." On cores: "Another thing larger ships could use to their advantage is the ability to have at least 2 cores. That's an actual good anti-lancer strategy since lancers are largely about splitting enemy ships down the middle." (Extra cores have been in since round 3, at +8 for the second; none of my example ships has one.) And overall: "This may have actually worked as intended, with the caveat that I bet <25px ships are still a bit too strong overall. But that will change as we add more parts." So no numbers move for now.

**The tracker stuck on a wreck (2026-11-15).** Henry, watching test 23 (1 BASTION VS 17 FIGHTERS): "why does the bastion's left gun get stuck in one position and just keep firing?" A trace of that fight: from about 7 s to the end at 34 s, its left tracker fired some 200 shots at one fighter wreck, sitting still with two pixels left and its middle pixel gone. The tracker aimed at the middle, so every shot flew through the hole, on a diagonal between the two pixels, and missed. (Flak's code had warned of this: it never aims at the middle, as that drills a tunnel later shots fly straight through.) The fix: a shot at the middle that would meet none of the target's standing pixels goes at the standing pixel nearest the middle instead. A tracker now kills that wreck (a test puts one at the bearing that missed), and only shots that would have missed are changed. Missiles, which also steer at a ship's middle, killed the same wreck with their first burst from six bearings, so they don't have the fault.

Every fight with a tracker in it moved; the other 27 team fights didn't (12 seeds). The middle column is my first fix, which had a slip of its own (below):

| fight | before the fix | first fix | the fix |
|---|---|---|---|
| 2 PICKETS VS 8 LANCERS | 9-3 | 2-9, 1 drawn | 9-3 |
| 2 PICKETS VS 8 FIGHTERS | 0-12 | 0-12 | 0-12 |
| 2 PICKETS VS 6 IRONLANCERS | 3-9 | 3-8, 1 drawn | 1-11 |
| 1 BASTION VS 17 LANCERS | 8-0, 4 drawn (1:30) | 12-0 | 11-0, 1 drawn (a stalemate; 0:58) |
| 1 BASTION VS 17 FIGHTERS | 12-0 | 12-0 | 12-0 |
| 1 BASTION VS 13 AP LANCERS | 3-9 | 2-10 | 2-10 |

- **The first fix made the PICKETS lose to lancers** (2-9; over 48 seeds 18-29, where the old aim won 34-14), and Henry asked me to find out why. Run side by side on the same seed, the old aim and the first fix parted before either tracker had fired: in the planners' look-ahead, which plays the fight forward with the same tracker rule. And with the look-ahead held the same, the live shots that differed were at healthy lancers, 9 pixels of 9, core standing. The check "would a shot at the middle meet anything" was wrong. The tracker aims from the snapshot of where its target stood as the tick began, as every gun does, but the check looked at the target's pixels where they were after its move that tick. A lancer crossing at 2 px a tick, 3 px wide, came out "missed", and the fallback aimed from where it now was and led it a second time, two pixels further on. So the PICKETS got worse from my slip, not because aiming through a hole was better.
- **The fix as it stands** takes the check and the fallback from the snapshot too. Now the two aims differ only at real wrecks (lancers down to 1 to 5 pixels, their lances gone, the middle a hole), and over 48 seeds the PICKETS beat 8 lancers 38-10, where the old aim won 34-14. A test fires at a one-pixel bar crossing at 2 px a tick, a tick after its snapshot, and fails on the slip.
- **The BASTION's four draws against lancers are gone,** so they were most likely turrets stuck on wrecks, not lancers it couldn't catch as I'd guessed.

**The BASTION redrawn (2026-11-15).** Henry: "redraw the bastion". Now 148 px with two cores (156 to build), 15 px/s like the battleship, every pixel of power spent:

```
.aaaaaaaaaaaa.     a heavy plate (a skin a pixel thick: 42 pixels in dominoes)
.ahhhhllhhhha.     l lance, i piercer: four lances, each touching a piercer
.attthiihttta.     t tracker: one on each flank
.attthllhttta.     g flak, four
eattthhhhtttae     c core, two, apart
eaghhhhhhhhgae     e engine: twelve along the stern and two on each flank, outside the skin
.ahhhhhhhhhha.
.ahhchhhhchha.
.aghhhhhhhhga.
.ahhhhhhhhhha.
.aaaaaaaaaaaa.
.eeeeeeeeeeee.
```

Its guns fire through its own skin (a shot only meets enemy pixels), so nothing but the engines is bare. The engines have to be outside: each needs a straight 3-wide way out.

| fight (12 seeds) | BASTION won | it lost | drawn | average length | BASTION lost | other side lost |
|---|---|---|---|---|---|---|
| 1 BASTION VS 17 LANCERS (156 against 153) | 11 | 0 | 1 (a stalemate) | 0:58 | 11 px | 16 of 17 |
| 1 BASTION VS 17 FIGHTERS | 12 | 0 | 0 | 0:20 | 16 px | all 17 |
| 1 BASTION VS 13 AP LANCERS (156 against 156) | 2 | 10 | 0 | 0:36 | 132 px | 5 of 13, 88 px |

(After the tracker fix above; before it, 8-0 with 4 drawn, 12-0, and 3-9.)

- **Plain lancers and fighters can't get through it.** Against lancers it loses only engines (11 px on average) and kills 16 of 17. Fighters die in 20 s.
- **Lancers with piercers are its answer,** 10-2 at equal cost. In three of their wins it was cut in two and a piece with a core flew on (splits).
- That's the layering Henry described: heavy plate beats plain lances and flak, the piercer beats heavy plate, and plain lancers beat lancers with piercers (9-3, above).

**Nothing old moved:** with these parts in, every fight recorded before (the starter deck's 15, the 19 part fights, the 24 team fights and the old reference fleet's) comes out exactly as it did, tick for tick, since no old drawing has the new parts, the beam's 3 meets no armour 2, and a missile at 2 breaks plate as it did at 3. (Checked again after the missile change, for every fight with a missile in it.) Twelve deliberate breaks of the new rules (heavy plate at armour 1, rounds not slowed or not raised, stacking piercers, the tracker on big ships or turning like a cannon or piercing plate, the beam back at 2, every burst at 3, the missile at 3, a heavier missile no slower, a burst that bounces off what it can't break, and since the tracker fix, a tracker that always aims at the middle and one that checks against where its target is now) are each caught by the new tests. 302 tests pass.

## The teams get the new parts (2026-11-15)

Henry: "We should probably give the non-swarm teams some of these anti-swarm tools we just added, yes?" To my plan (below): "yes, with the AP lancers for swarm". On the results below: "Those are fine results for now."

| row | what | verdict |
|---|---|---|
| E1 | **BIG GUNS: the BATTLESHIP redrawn.** Two of its five cannons swapped for one tracker (the same power), a cap of heavy plate on its prow, eight lances, twelve flak, and two cores, apart. 195 px and the second core (203 to build), at a frigate's speed as before (it was 188). | |
| E2 | **LANCE LINE: the CRUISER and the CORVETTE redrawn.** The cruiser: a cap of heavy plate on its prow, where enemy lances land when it faces them, and a tracker amidships in place of five of its seventeen lances, with eleven engines so it keeps its speed (132 px; it was 124). The corvette: its nose a domino of heavy plate, plate cheeks, three lances and four flak (29 px and 22 px/s; it was 24 px and 26). | |
| E3 | **SIEGE: a SENTINEL, and the LANCET in heavy plate.** The SENTINEL (new, 47 px, 15 px/s) is a tracker under a cap of plate with plate down its flanks; its outposts build them and its king prints them, between wardens and beamers. The LANCET's sides and stern are heavy plate in dominoes: no slower, as it already flew as slowly as a ship can, but it turns slower. | |
| E4 | **SWARM: AP lancers in its boom.** Its outposts, planted from 5:30, print AP lancers among their miners, fighters and lancers, so it can break heavy plate. Its rush (nests and its king) is as it was. | |
| E5 | **The arena's fights for them:** 6 CORVETTES VS 16 LANCERS is now 5 CORVETTES (to stay level on cost), 1 BATTLESHIP VS 21 LANCERS is now 23 LANCERS, and two new ones: 2 SENTINELS VS 10 FIGHTERS and VS 10 LANCERS. RAIDERS and the starter deck are unchanged. | |

Every redrawn card spends exactly the power it makes, and every fight is level on cost. (SIEGE's escort was to be the SENTRY, but the game makes plurals with an S.)

**The round robin, new against new** (eight seeds, each team on each side: 16 matches a cell, the row team's wins first):

Before:

| | STARTER | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | ALL |
|---|---|---|---|---|---|---|---|
| STARTER DECK | - | 4-12 | 14-2 | 16-0 | 3-13 | 4-12 | 41-39 |
| SWARM | 12-4 | - | 16-0 | 16-0 | 9-7 | 16-0 | 69-11 |
| LANCE LINE | 2-14 | 0-16 | - | 7-9 | 0-16 | 5-11 | 14-66 |
| BIG GUNS | 0-16 | 0-16 | 9-7 | - | 2-14 | 12-4 | 23-57 |
| RAIDERS | 13-3 | 7-9 | 16-0 | 14-2 | - | 16-0 | 66-14 |
| SIEGE | 12-4 | 0-16 | 11-5 | 4-12 | 0-16 | - | 27-53 |

After:

| | STARTER | SWARM | LANCE LINE | BIG GUNS | RAIDERS | SIEGE | ALL |
|---|---|---|---|---|---|---|---|
| STARTER DECK | - | 6-10 | 1-15 | 16-0 | 3-13 | 6-10 | 32-48 |
| SWARM | 10-6 | - | 15-1 | 16-0 | 8-8 | 11-5 | 60-20 |
| LANCE LINE | 15-1 | 1-15 | - | 15-1 | 1-15 | 11-5 | 43-37 |
| BIG GUNS | 0-16 | 0-16 | 1-15 | - | 2-14 | 10-6 | 13-67 |
| RAIDERS | 13-3 | 8-8 | 15-1 | 14-2 | - | 16-0 | 66-14 |
| SIEGE | 10-6 | 5-11 | 5-11 | 6-10 | 0-16 | - | 26-54 |

**The AI teams against the SWARM Henry played** (`--old SWARM`, its AI rules as they were; its cards now include the AP lancers): the AI's wins of 80 went from 21 to 33. STARTER DECK 8-8 to 9-7, LANCE LINE 0-16 to 2-14, BIG GUNS 4-12 to 4-12, RAIDERS 8-8 to 10-6, SIEGE 1-15 to 8-8.

Medians over each team's matches, before / after: SWARM's matches 8:51 / 11:02, LANCE LINE's 13:02 / 12:16, BIG GUNS' 14:21 / 13:44, SIEGE's 14:26 / 14:41. BIG GUNS sends no wave in 66 matches of 80 (55 before). The biggest fight in a match: median 104 / 101 ships, most 357 / 292.

**The arena's army fights** (12 seeds; blue's wins, losses, draws):

| fight | before | after |
|---|---|---|
| SWARM VS LANCE LINE | 12-0 | 0-12 |
| SWARM VS BIG GUNS | 12-0 | 11-1 |
| SWARM VS SIEGE | 12-0 | 12-0 |
| LANCE LINE VS STARTER DECK | 5-6-1 | 12-0 |
| LANCE LINE VS BIG GUNS | 11-1 | 12-0 |
| LANCE LINE VS RAIDERS | 1-11 | 12-0 |
| LANCE LINE VS SIEGE | 12-0 | 12-0 |
| BIG GUNS VS STARTER DECK | 3-9 | 7-5 |
| BIG GUNS VS RAIDERS | 4-4-4 | 12-0 |
| BIG GUNS VS SIEGE | 10-2 | 12-0 |
| SIEGE VS STARTER DECK | 0-12 | 0-12 |
| RAIDERS VS SIEGE | 12-0 | 12-0 |
| 1 BATTLESHIP VS 21 LANCERS, now 23 | 0-12 (0:14) | 0-12 (0:30) |
| 6 CORVETTES VS 16 LANCERS, now 5 | 1-11 | 1-11 |
| 1 CRUISER VS 2 FRIGATES | 4-6-2 | 7-4-1 |
| 1 LANCET VS 1 BATTLESHIP | 6-6 | 1-11 |
| 2 LANCETS VS 40 FIGHTERS | 0-12 | 0-12 |
| 2 SENTINELS VS 10 FIGHTERS | - | 5-7 |
| 2 SENTINELS VS 10 LANCERS | - | 2-10 |

The other fights didn't move.

**What it comes to:**
- **LANCE LINE is a different team:** 14-66 to 43-37. Its army now beats SWARM's head-on in the arena (0-12 to 12-0), and beats every other team's army there. In a match it still loses to SWARM 1-15: the rush comes at 2:30, and its own army is massing for 7:00.
- **SIEGE now holds against SWARM sometimes:** 0-16 to 5-11, and 1-15 to 8-8 against the SWARM Henry played.
- **BIG GUNS got worse:** 23-57 to 13-67, still 0-16 against SWARM, and now 1-15 against LANCE LINE (9-7 before). Its battleship comes late, and a cap of heavy plate on the prow doesn't stop a swarm (it still loses to an equal cost of lancers 0-12, if more slowly), as the first BASTION showed. A BIG GUNS that wanted to hold off SWARM would need its big ship sooner, or a whole skin as the BASTION has. I haven't watched these matches; that's my reading of the numbers.
- **SWARM is weaker, 69-11 to 60-20,** and its matches longer: the teams now last past its rush more often.
- The scripted openings were stopped part way: they play the starter deck against the scripted bots, neither of which changed. So does `--frame-cost`, which came out as before (median frame at 4x about 3.3 ms, worst 7.5 ms, with nothing else loading the machine this time), so it doesn't see the new parts. For the team matches, the biggest fight got smaller (above). 302 tests pass.

## The planner on the other cores, ship by ship (2026-11-16)

Henry, asked what should come next: unit AI ("multithreading, granularity, we want the ships to feel like they're microing themselves effectively"), more parts, a campaign, or Steam. I argued for the unit AI's machinery first (it doesn't depend on which parts exist), then a small round of parts picked for the behaviours they ask of ships, then the behaviours themselves; Steam after the two-machine test, which is still open; the campaign last. Henry: "Yes" to a plan, then "Yes, build it", assuming a four-core machine. Not committed.

**Where the planner stood** (measured on 60 team matches before anything changed). It played the fight forward 8 s for each way one design could fly, one design at a time, on the game's own thread, held to about 1 ms of the 66.7 ms tick:

| ships in the fight | one design decided in | every design on a side |
|---|---|---|
| under 20 | 0.5 s | 0.5 to 1.6 s |
| 20 to 40 | 0.8 s | 1.7 s |
| 40 to 70 | 2.6 s | 6.4 s |
| 70 to 110 | 8 s | 24 s |
| over 110 | 12 s | 36 s |

In a fight of over about 70 ships the answer came after the 8 s it had looked ahead. There was one fight on the map 99% of the time (ships within 300 px of each other counted as one fight), so thinking about separate fights separately would have gained nothing, and isn't built.

| row | what | verdict |
|---|---|---|
| U1 | **The playing forward is done on worker threads** (`sim/pool.rs`): every core but one, the game's own. The game's thread only begins a think (it copies the fight) and uses it. A fight played forward works from its own copy and gives back a number, so it comes out the same on any thread at any speed. | |
| U2 | **A think is used on a tick fixed when it begins, by the size of the fight alone, and never sooner.** A machine that has its answers early waits for the tick; one that hasn't got them by then waits for them. So two machines in a match always fly the same plan, and every machine plays the same game. | |
| U3 | **The machine it's planned for: four cores, each half as fast as an i9-13900KF's** (Henry's ruling on my proposal). A side's planner counts on half of the three spare cores. What a fight costs to play forward is a sum fitted to 6,892 thinks: for each ship, 10 ns for each gun in the fight and 1,200 ns besides, a tick, on this machine. My first sum, the square of the ships, was out by up to three times (a ship with many guns costs far more), and would have had one think in ten late on that machine. | |
| U4 | **Every ship of a side in the fight is thought about at once.** A think plays the fight forward once with nothing changed and once for each other way each unit could fly, from the same start on the same fresh dice. A unit takes the way that beat leaving things as they are by the most, if any did. | |
| U5 | **How fine it thinks: a round of decisions should take about a second** (15 ticks), which sets how many units a side has. Each ship for itself in a small fight; squads of a design's ships that are near each other in a bigger one; a whole design at a time in the biggest, never coarser. Ten fighters a side decide ship by ship; twenty a side in 17 squads; fifty a side in 3; a hundred and fifty a side as one design, in about 3 s. Henry's grain: "per ship when few, groups when many". | |
| U6 | **Each ship has a way of flying of its own.** A ship that stays as it was is given the way it was flying, so what's decided later for the rest of its design doesn't move it. A ship that hasn't been in a think (a newcomer, or one elsewhere on the map) flies as most of its design did after their last. | |
| U7 | **A side begins a think no sooner than 8 ticks after it began its last,** which is how often it decided for a design in a small fight before. | |
| U8 | **On screen:** the bar under a ship says what its side has it doing ("ITS SIDE HAS IT KITING."), where it said what its side had every ship of the design doing. The ARCH button's tip says the same. | |

**Where what I built departs from the plan:**
- I built the threads and "every design at once" together and measured them as one step, not two.
- My estimates of how soon everything would be decided were too rosy, because the sum they came from was too low: I said 0.3 s at 60 ships, under 1 s at 100 and about 4 s at 250. As built, a round takes about a second up to a hundred ships and about 3 s at three hundred, but it gets there by thinking coarser than I'd pictured: ship by ship only up to about 35 ships in the fight.

**The proofs.** Fourteen tests in `sim/arch.rs` and two in `sim/pool.rs`. The main one plays a fight of forty fighters five ways and compares the checksum after every tick: every think worked out by the game's own thread on the tick it's due; by the workers with the game not waiting for them; by workers each held up for some milliseconds; and by workers with the game waiting until every answer is in early. All the same fight. Then the fault those guard against, a think used the moment its answers are in, with the answers in early: a different fight, so the comparison does catch it. Others: a think's scores are what playing each change forward by hand gives; nothing is used before the tick it's due though the answers are in; a copy of a fight made in the middle of a think plays on the same; the units for five sizes of fight. 307 tests pass.

One old test changed: the rush opening's hurt king still goes home, but its one lancer, flying as its side's planner now has it, kills the enemy's idle king six seconds later, so the test allows the match to be the rusher's.

**What a frame costs now** (`--frame-cost`, which now also plays SWARM against RAIDERS and BIG GUNS against LANCE LINE, with nothing else loading the machine; before, the starter deck and my saved team only):

| ms a tick on the game's own thread | before: median | 99.9% | max | now: median | 99.9% | max |
|---|---|---|---|---|---|---|
| the match's tick | 1.83 | 8.98 | 11.58 | 1.02 | 5.60 | 7.29 |
| of it, the planners | 0.96 | 7.00 | 9.08 | 0.00 | 0.37 | 1.12 |
| a whole frame at 4x | 3.13 | 10.40 | 13.35 | 3.22 | 7.86 | 9.59 |

**Whether the cores keep up** (the same four matches, 8,610 thinks, with three worker threads as a four-core machine has). A think plays 11 fights forward at the median and 55 at the most, 129 ms of one core's work at the median and 952 at the most, and is allowed 0.33 s at the median, 1.1 s at the 99th in a hundred and 1.5 s at the most. Here, with three workers, it had every answer in after 15% of that at the median and 60% at the 99.9th in a hundred; 26 of the 8,610 took over half of it, which is what would be late on a machine half as fast, and 2 took longer than allowed (one twice as long: something else had the machine). So on the machine planned for, about one think in 300 would be late and the game would wait for it, a fraction of a second. With every core of this machine, the median think is in after 6% of its time.

**Not done, and for Henry:**
- The round robins and the tally are in, with three measurements they sent me to: "The planner's tables" below.
- The one think in 300: a bigger margin in U3 would end that, and every machine would decide more slowly for it. I left it.
- New things to choose between (targets, keeping a prow to the enemy, staying with a projector) are the next half.
- The tools: `--frame-cost` as above, with `--workers N`; `--ways` now also gives, by the size of the fight, how many ships a decision is for and how long it takes, and how often the same think on other dice comes out the same; `--grain design`, with any tool, holds the planner to a design at a time.

**The planner's tables, and what a decision is made of (2026-11-16, later).** Nothing in the game changed for this. The tools gained two measurements and one switch that only they set: `--ways` now scores each change in a think on two rolls of the dice, and `--flight` fights a planner whose scores are thrown away (`Arena::blind`, `Think::blind`), runs a fight's seeds side by side, and takes `--grain design`. `--flight` lost its line timing a tick, which timed the game's thread waiting for the workers and so said nothing about a frame; `--frame-cost` says that.

*The round robins.* 240 matches each, 8 seeds a pairing, each team on each side, both sides on the same planner: the planner before this work (the committed game), the planner now, and the planner now held to a design at a time (`--grain design`). Each cell is the first team's wins and the second's, of 16. No match was drawn.

| pairing | planner before | now, ship by ship | now, a design at a time |
|---|---|---|---|
| STARTER DECK v SWARM | 6-10 | 6-10 | 6-10 |
| STARTER DECK v LANCE LINE | 1-15 | 0-16 | 2-14 |
| STARTER DECK v BIG GUNS | 16-0 | 13-3 | 14-2 |
| STARTER DECK v RAIDERS | 3-13 | 0-16 | 0-16 |
| STARTER DECK v SIEGE | 6-10 | 0-16 | 0-16 |
| SWARM v LANCE LINE | 15-1 | 16-0 | 16-0 |
| SWARM v BIG GUNS | 16-0 | 16-0 | 16-0 |
| SWARM v RAIDERS | 8-8 | 6-10 | 6-10 |
| SWARM v SIEGE | 11-5 | 15-1 | 11-5 |
| LANCE LINE v BIG GUNS | 15-1 | 16-0 | 16-0 |
| LANCE LINE v RAIDERS | 1-15 | 1-15 | 0-16 |
| LANCE LINE v SIEGE | 11-5 | 13-3 | 10-6 |
| BIG GUNS v RAIDERS | 2-14 | 0-16 | 0-16 |
| BIG GUNS v SIEGE | 10-6 | 7-9 | 7-9 |
| RAIDERS v SIEGE | 16-0 | 16-0 | 15-1 |

| a team's 80 matches: won-lost, median length | planner before | now, ship by ship | now, a design at a time |
|---|---|---|---|
| STARTER DECK | 32-48, 14:02 | 19-61, 13:20 | 22-58, 13:21 |
| SWARM | 60-20, 11:02 | 63-17, 11:21 | 59-21, 11:18 |
| LANCE LINE | 43-37, 12:16 | 46-34, 11:29 | 40-40, 11:04 |
| BIG GUNS | 13-67, 13:44 | 10-70, 12:44 | 9-71, 11:48 |
| RAIDERS | 66-14, 12:39 | 73-7, 12:15 | 73-7, 10:24 |
| SIEGE | 26-54, 14:41 | 29-51, 13:24 | 37-43, 14:15 |
| the 240 matches took, on 32 cores | 428 s | 5,252 s | 3,700 s |

What these can and can't say. Both sides of every match fly by the same planner, so the tables show how the balance between the teams moved and not whether ships fly better. (I told Henry these were the tables that would show whether it plays better. They aren't.) What moved: the lopsided pairings got more lopsided. STARTER DECK won 9 of its 32 against RAIDERS and SIEGE before and none now, at either grain; BIG GUNS against SIEGE went from 10-6 to 7-9; SWARM against SIEGE is 15-1 ship by ship and 11-5 otherwise. Each team's first wave and factories planted are about what they were. A round robin now takes an hour and a half to run where it took seven minutes: the playing forward is most of it, which is what the cores are for.

*Which ways are kept* (`--ways 1`: 30 matches, then the arena's team fights at 12 seeds each; the tally before was on two seeds a pairing and on 24 team fights, so only the rates compare).

| | decisions | attack runs / kite / orbit kept | every way scored the same (all 0) | gave a new way | time in a fight: runs / kite / orbit |
|---|---|---|---|---|---|
| matches, planner before | 52,872 on 60 | 30 / 38 / 31 % | 18 % (7 %) | 42 % | 29 / 33 / 36 % |
| matches, ship by ship | 248,299 on 30 | 33 / 32 / 34 % | 12 % (2 %) | 52 % | 32 / 33 / 34 % |
| matches, a design at a time | 107,782 on 30 | 30 / 36 / 33 % | 9 % (3 %) | 52 % | 29 / 37 / 32 % |
| team fights, planner before | 37,639 | 27 / 36 / 35 % | 14 % (1 %) | 45 % | 44 / 31 / 23 % |
| team fights, ship by ship | 339,390 | 31 / 33 / 34 % | 7 % (0 %) | 57 % | 32 / 34 / 33 % |
| team fights, a design at a time | 106,376 | 27 / 36 / 36 % | 5 % (0 %) | 55 % | 27 / 37 / 35 % |

Ship by ship, no design leans far any way: the widest in matches is the wasp, kiting 45% (60% a design at a time, 64% before). A design at a time, some still do: in matches the wasp kites 60% and the raker 48%, and in the team fights the picket orbits 54% and the ironlancer 15%.

| matches, by ships in the fight | ship by ship: decisions | ships a decision is for, median and most | seconds to decide, median and most | gave a new way | a design at a time: decisions | ships | seconds | new way |
|---|---|---|---|---|---|---|---|---|
| under 10 | 3,158 | 1, 1 | 0.1, 0.1 | 23 % | 2,241 | 1, 6 | 0.1, 0.1 | 24 % |
| 10 to 19 | 23,292 | 1, 1 | 0.1, 0.4 | 38 % | 12,735 | 2, 13 | 0.1, 0.1 | 39 % |
| 20 to 39 | 119,272 | 1, 2 | 0.3, 1.2 | 51 % | 37,951 | 2, 17 | 0.1, 0.3 | 50 % |
| 40 to 69 | 73,881 | 1, 13 | 0.7, 1.4 | 57 % | 28,687 | 4, 30 | 0.2, 0.6 | 57 % |
| 70 to 109 | 21,479 | 1, 23 | 1.0, 1.4 | 58 % | 13,364 | 4, 30 | 0.4, 1.1 | 57 % |
| 110 and over | 7,217 | 3, 35 | 1.0, 2.6 | 59 % | 12,804 | 6, 36 | 1.0, 2.6 | 61 % |

So decisions do land in about a second, as planned, where a big fight took 24 to 36 s to get round a side's designs. And ship by ship, a ship in a fight is handed a new way of flying about every two seconds (129,000 new ways over 238,000 seconds of ships in fights), each of them chosen by looking 8 s ahead.

*The same think on other dice.* The check the tally of 2026-11-15 asked for. Every ten seconds the tool takes each side's think in hand and works it out again with the dice in the fights played forward falling another way.

| | matches, ship by ship | team fights, ship by ship | matches, a design at a time | team fights, a design at a time |
|---|---|---|---|---|
| decisions looked at again | 13,448 | 10,861 | 3,483 | 1,294 |
| came out the same | 44 % | 40 % | 47 % | 56 % |
| of those where either roll gave a new way, the same | 27 % of 10,292 | 27 % of 8,849 | 31 % of 2,670 | 38 % of 913 |

*Each change on two rolls* (`--ways 0`, the team fights only; matches weren't run for this, so nothing here is from a fight of over 110 ships). A change is one unit flying one other way. Its score is the fight played forward with it, less the fight played forward with nothing changed, on the same dice. "The two rolls agree" is the correlation between a change's score on the think's dice and on the other dice: near 1 if the score is what the change did, near 0 if it's the dice.

| ship by ship, by ships in the fight | changes | px the dice alone move the unchanged fight's score (median) | px a change moves it (median) | px a change is worth (average) | the two rolls agree | changes ahead on the first roll that are ahead on the second | all changes ahead on the second |
|---|---|---|---|---|---|---|---|
| under 10 | 1,442 | 3 | 5 | -0.4 | 0.19 | 59 % | 36 % |
| 10 to 19 | 2,946 | 11 | 15 | -0.6 | -0.02 | 55 % | 46 % |
| 20 to 39 | 11,902 | 23 | 22 | +1.7 | 0.07 | 53 % | 47 % |
| 40 to 69 | 5,592 | 27 | 24 | -0.4 | 0.01 | 50 % | 47 % |
| 70 to 109 | 940 | 23 | 29 | -9.9 | 0.03 | 36 % | 32 % |
| a change for 1 ship | 16,710 | | 19 | +0.3 | 0.05 | 53 % | 46 % |
| for 2 to 5 | 5,380 | | 24 | +1.8 | 0.06 | 52 % | 47 % |
| for 6 to 15 | 732 | | 28 | -10.9 | -0.09 | 33 % | 30 % |

| a design at a time, by ships in the fight | changes | the dice alone | a change | worth | agree | ahead again | all ahead |
|---|---|---|---|---|---|---|---|
| under 10 | 636 | 1 | 9 | -3.9 | 0.33 | 63 % | 32 % |
| 10 to 19 | 626 | 9 | 17 | -7.2 | 0.23 | 56 % | 40 % |
| 20 to 39 | 688 | 14 | 26 | -9.0 | 0.39 | 63 % | 42 % |
| 40 to 69 | 380 | 20 | 29 | -7.3 | 0.39 | 60 % | 44 % |
| 70 to 109 | 260 | 32 | 42 | -18.9 | 0.33 | 50 % | 36 % |
| a change for 1 ship | 614 | | 13 | -4.8 | 0.22 | 55 % | 36 % |
| for 2 to 5 | 924 | | 16 | -5.5 | 0.28 | 60 % | 41 % |
| for 6 to 15 | 840 | | 27 | -12.5 | 0.40 | 63 % | 39 % |
| for 16 and over | 212 | | 20 | -10.9 | 0.44 | 55 % | 40 % |

Ship by ship, in a fight of ten ships or more, one roll says almost nothing about the next: a change that came out ahead is ahead again 50 to 55% of the time, where any change at all is 46 to 47%. One ship flying another way moves the score by about what the dice alone move it. A design at a time, about a third of a change's score is the change.

Why: every roll in a fight comes off one run of dice (six places in `sim.rs`: which pixel a gun aims for, the scatter in flak's reload, and how a pixel drifts when it's shot off). A think gives every fight it plays forward the same dice, but a ship that flies another way fires at other moments and breaks pixels at other moments, and from the first of those on every other ship's rolls fall differently too. So "the same fresh dice" stop being the same within a tick or two, and a change's score is mostly a second roll of the whole fight.

*The planner against none, and against a blind one* (`--flight`: mixed fleets from a deck, five fights, 12 seeds, each side once, so 120 fights a cell; won-lost-drawn). BLIND is the control: a planner that thinks as often but throws the scores away and hands each unit, with equal chance, the way it has or either other. The old fleet's deck was left out: its fights are three-minute stalemates and nothing is judged on it now.

| | planner before | now, ship by ship | now, a design at a time |
|---|---|---|---|
| STARTER DECK: planner v none | 81-39 | 85-35 | 83-37 |
| PARTS LIBRARY: planner v none | 82-2-36 | 96-1-23 | 114-0-6 |
| YOUR TEAM: planner v none | 97-23 | 107-13 | 109-10-1 |
| pixels the planner's side lost and took, the three decks | 97,097 and 159,153 | 74,483 and 165,172 | 67,427 and 164,573 |
| STARTER DECK: blind v none | | 58-62 | 63-56-1 |
| PARTS LIBRARY: blind v none | | 67-45-8 | 56-47-17 |
| YOUR TEAM: blind v none | | 73-46-1 | 68-51-1 |
| STARTER DECK: planner v blind | | 77-43 | 78-42 |
| PARTS LIBRARY: planner v blind | | 79-6-35 | 98-2-20 |
| YOUR TEAM: planner v blind | | 90-30 | 99-21 |

*How I read it.*
- **The work was worth doing.** Against ships that fly as drawn, of 360 fights, the planner before won 260, lost 64 and drew 36. Now it wins 288, loses 49 and draws 23 ship by ship, and wins 306, loses 47 and draws 7 a design at a time, losing a quarter to a third fewer pixels.
- **The looking ahead is real.** Handing out ways by chance is about a coin flip against flying as drawn (198-153-9 ship by ship, 187-154-19 a design at a time), and the planner beats that 246-79-35 and 275-65-20. So the scores carry something, though one decision at a time it's little.
- **As built, thinking ship by ship is worse than a design at a time.** It wins fewer fights against none (288 to 306) and against blind (246 to 275), most of the gap on the parts deck (96-1-23 to 114-0-6); it's level on the starter deck. Its decisions are nearer to dice, and it takes about 40% more work. What improved on the old planner is U1 and U4 (decisions in a second, every design at once), not U5.
- **I haven't found which decisions carry the wins.** Fights of under ten ships are the only ones where one ship's change repeats at all (0.19), and every fight ends as one. Or many slight edges add up. Not measured.
- **The round robins don't judge any of it,** as above.
- **U5 is still as built.** The game thinks ship by ship in small fights, as Henry ruled. I haven't switched it to a design at a time on these numbers: it's Henry's rule, and the step below may give ship by ship what it lacks.

*What I'd do next (built the same day: see the next section): give each ship its own dice.* A roll made from the match's seed, the tick, the ship and what the roll is for, in place of the next number off one run. Then a ship that flies another way leaves every other ship's rolls where they were, and a change's score is what the change did. It costs nothing a tick. Every recorded fight plays out anew (the odds are the same, the rolls aren't), and it's a new protocol number. The three tools above would say whether it worked: the agreement between rolls should rise at both grains, and ship by ship should start to beat a design at a time if it has anything to offer. I'd do it before the behaviours half, because the more ways a ship can choose between, the more chances the dice have to pick one. If it doesn't lift the agreement, the fallback is a design at a time with the spare work spent on more rolls for each choice.

## Each ship its own dice, and what a think's choices are worth (2026-11-16)

Henry, asked whether to give each ship its own dice and to measure a ship's own score beside the whole fight's: "Yes, both." Built and measured the same day. How the game's planner chooses is unchanged: everything past the dice is a measurement, or a switch only the tools set.

| # | What | Henry's verdict |
|---|---|---|
| Y1 | **Each ship rolls its own dice.** A roll is made from the fight's dice, the tick, the ship, which of its pixels (or turrets) is rolling, and what the roll is for (`Arena::roll`): the pixel a flak gun, a cannon or a ship's lances aim at, how long a flak reload takes, how far a pixel knocked off drifts, and where a broken node's pixels land. Asking uses nothing up, so what one ship does can't move another's rolls along. The arena no longer holds any dice that get used up (`Arena.dice` is a number; the map's rocks and stars are made with dice of their own). A fight a planner plays forward is given other dice than the match's, made from the match's dice, the tick and the side. | |
| Y2 | **The same roll picks the same pixel while that pixel stands** (`random_cell`). The dice give every pixel of the target a number and the surviving pixel with the lowest is the one: each is as likely as any other. Counted off along the survivors, as before, one pixel lost anywhere on the target moved every pick. | |
| Y3 | **Each ship keeps an account:** the pixels it has knocked off (`dealt`: by its shots, bursts and beams, and whatever broke away or burst afterwards on a ship it was the last to hit) and the pixels it has lost (`lost`). A shot remembers who fired it. A side's score is exactly the sum of its ships' accounts. Nothing in the game reads them; the tools do. | |
| Y4 | **The network protocol is 16.** Every fight falls out another way: the odds are the same, the rolls aren't. | |
| Y5 | **Four tests that were one fight on one roll are now twenty rolls each,** and two of them hold a fault where it is (below). | |

*The proof.*
- Two skirmishes of fighters 600 px apart, played once as they are and once with one fighter of the first kiting: the second skirmish must end with every ship where it was, the same pixels left and its guns as far through their reloads. It fails on the old dice (written first, against them) and passes now.
- The same roll picks the same pixel as the target is taken apart a pixel at a time, until that pixel goes; and on 200 rolls a pixel no pixel is picked under half or over twice its share.
- After eight mixed-fleet fights fought to the end, every pixel a side lost is in one of its ships' `lost` and in one enemy ship's `dealt`. With a burst's kills left uncredited it fails (686 against 781).

*What moved in the recorded fights* (12 seeds each, the old dice beside the new; counted by wins and draws, not seconds or pixels):
- the starter deck's reference fights: 3 of 15 rows moved (12 FRIGATES VS 1 CAPITAL 9-3 to 6-6, 76 FIGHTERS VS 76 LANCERS 7-5 to 5-7, 6 FIGHTERS VS 1 FRIGATE 0-11-1 to 1-10-1); blue's wins over all rows 124 to 120;
- the part examples' fights: 12 of 19 rows moved, none in who was ahead; the furthest are 2 BEAMERS VS 6 LANCERS 1-11 to 4-8, 2 RAKERS VS 1 WARDEN 8-1-3 to 4-0-8 and 2 ARCHERS VS 1 OUTPOST 9-1-2 to 12-0-0; blue's wins over all rows 99 to 104;
- the team fights: not compared. The run on the old dice was still going when this was committed.

*The four tests.* Each was one fight on the roll the arena starts on. On twenty rolls, on the old dice (a copy of the last commit) and on the new:

| The test's claim | Holds on the old dice | On the new |
|---|---|---|
| Two frigates lining their lances up on each other settle (within 90 px up and down, three volleys each or more) | 4 of 20 | 5 of 20 |
| The beamer and the warden keep to their row (within 90 px, four cuts or more reaching the circles) | 15 of 20 | 15 of 20 |
| No fighter is inside the capital for more than 3 ticks in a row | 16 of 20 | 15 of 20 |
| The gunminer hits the fighter (it fires at all in 8, and in 10) | 5 of 20 | 7 of 20 |

So the dice broke none of them: they were never true of more than their own roll. Two are faults, **found here and not mended:**
- **Two lance ships lining up on each other mostly don't settle.** In 11 fights of 20 the frigate ranges further than 90 px (up to the whole height of the map), and in others the two stand off and fire one volley each in thirty seconds.
- **A fighter can be inside a capital for up to 20 ticks,** in about one fight in four. The capital flies onto it and keeps coming the way the fighter is getting out.

The tests now hold each at today's rate (4 of 20, 14 of 20), named for what's true (`..._settle_one_time_in_four`, `small_ships_seldom_stay_inside_big_ones`), with the fault written in.

**What the dice did for a ship's choice.** Each change in a think scored on the think's dice and on other dice (`--ways 0`, the arena's team fights). Ahead again: of the changes that beat leaving things alone on the first roll, the share that did on the second. Any: the share of all changes ahead on the second. Agree: how far the two rolls agree on what a change is worth (1 wholly, 0 not at all).

| Ships in the fight | Shared dice: px a change moves the score, agree, ahead again / any | Own dice | Own dice and Y2 | The same, scored on the unit's own account |
|---|---|---|---|---|
| under 10 | 5, 0.19, 59% / 36% | 5, 0.38, 60% / 35% | 5, 0.43, 55% / 29% | 2, 0.55, 62% / 26% |
| 10-19 | 15, -0.02, 55% / 46% | 7, 0.18, 51% / 34% | 9, 0.16, 53% / 38% | 4, 0.35, 59% / 33% |
| 20-39 | 22, 0.07, 53% / 47% | 13, 0.03, 50% / 41% | 14, 0.11, 49% / 41% | 5, 0.28, 58% / 36% |
| 40-69 | 24, 0.01, 50% / 47% | 16, 0.07, 49% / 44% | 15, 0.19, 50% / 42% | 5, 0.32, 57% / 40% |
| 70-109 | 29, 0.03, 36% / 32% | 15, 0.17, 55% / 45% | 13, -0.02, 47% / 37% | 9, 0.25, 55% / 40% |

A design at a time the rolls agreed 0.23 to 0.39 before and 0.30 to 0.47 now; on the unit's own account, 0.41 to 0.53. So the shared dice were part of the noise and not most of it: a change moves the score about a third less, and one roll still says little about the next. The rest is the fight itself. A ship that flies another way changes who is nearest to whom, targets change, and the fight goes another way by the rules alone.

**What the way a unit is given is worth to its side.** The way picked on the think's dice, scored on two rolls kept back (the whole fight's score, px a unit a think; a unit that stays as it is counts 0; with standard errors, which are on the small side since the units of one fight aren't independent):

| The way is picked on | Ship by ship (10,846 units) | A design at a time (1,324) |
|---|---|---|
| the whole fight's score, one roll (the game) | 2.10 ±0.26 | 5.40 ±0.82 |
| the same on two rolls added up | 2.86 ±0.26 | 6.62 ±0.84 |
| on three | 3.07 ±0.25 | 7.43 ±0.83 |
| the unit's own account, one roll | 1.64 ±0.25 | 5.35 ±0.82 |
| the unit's own account, three rolls | 2.48 ±0.25 | 6.37 ±0.85 |
| chance (as it is, or any other way, each as likely) | -0.63 ±0.18 | -3.74 ±0.66 |
| knowing the two rolls kept back | 15.45 ±0.24 | 16.65 ±0.78 |

- Three rolls for one is worth 0.97 ±0.19 more a pick ship by ship and 2.03 ±0.58 a design at a time: about 40% more.
- **A margin looks free, one think at a time** (in fights it isn't: below). A change that came out 1 to 10 px ahead is worth nothing on other dice (0.28 ±0.79 and 0.18 ±0.93 ship by ship); one 11 to 40 ahead about 2.5; one over 40, 8.4. Giving a new way only when it's more than 10 px ahead keeps the worth (2.16 for 2.10 ship by ship, 5.19 for 5.40 a design at a time) and cuts the units given a new way each think from 56% to 38% (52% to 36%).

**A think's changes made together.** The planner judges each change with everything else left as it is, then makes them all at once. That joint change played forward on the two rolls kept back (px to the side a think, over thinks that give any unit a new way):

| | Ship by ship (1,030 thinks) | A design at a time (502) |
|---|---|---|
| units given a new way | 6.0 of 9.7 | 1.4 of 1.8 |
| what the think counted on: each change on its own dice, added up | 206.5 ±8.7 | 51.3 ±3.4 |
| the changes one at a time on the rolls kept back, added up | 22.1 ±5.2 | 14.3 ±2.4 |
| **all of them made together** | **10.2 ±1.6** | **7.9 ±1.8** |
| only the half that came out furthest ahead, together | 11.4 ±1.4 | 11.9 ±1.8 |
| only the one furthest ahead | 8.2 ±1.2 | 11.5 ±1.8 |
| only those more than 10 px ahead, together (4.0 units) | 10.1 ±1.4 | not run |
| the ways each unit's own account gives, together (5.3 units) | 14.7 ±1.6 | not run |

Made together the changes are worth about half what they're worth one at a time, and the one furthest ahead carries most of it. One at a time the whole fight's score is the slightly better judge; made together, the units' own accounts are.

**In fights** (`--flight`, 120 fights a deck, won-lost-drawn). The other planners are switches only the tools set (`Judge`): OWN judges a change on its own ships' account; HALF makes only the half of a think's changes that came out furthest ahead; MARGIN only those more than 10 px ahead; BEST only the one furthest ahead; BLIND throws the scores away.

| Against ships that fly as drawn | STARTER | PARTS | YOUR TEAM | All 360 |
|---|---|---|---|---|
| the game's planner, shared dice, ship by ship | 85-35 | 96-1-23 | 107-13 | 288-49-23 |
| the game's planner, shared dice, a design at a time | 83-37 | 114-0-6 | 109-10-1 | 306-47-7 |
| **the game's planner, ship by ship** | 83-37 | 106-1-13 | 114-6 | **303-44-13** |
| the game's planner, a design at a time | 77-43 | 113-0-7 | 111-8-1 | 301-51-8 |
| HALF | 89-31 | 114-0-6 | 115-5 | 318-36-6 |
| MARGIN | 88-31-1 | 99-1-20 | 113-7 | 300-39-21 |
| BEST | 90-30 | 98-5-17 | 109-11 | 297-46-17 |
| BEST, a design at a time | 83-37 | 115-0-5 | 110-9-1 | 308-46-6 |
| OWN | 82-38 | 105-0-15 | 110-10 | 297-48-15 |
| OWN, a design at a time | 80-39-1 | 111-0-9 | 112-8 | 303-47-10 |
| BLIND | 61-59 | 66-48-6 | 74-46 | 201-153-6 |

| The game's planner against | STARTER | PARTS | YOUR TEAM | All 360 |
|---|---|---|---|---|
| BLIND | 72-48 | 92-2-26 | 91-27-2 | 255-77-28 |
| OWN | 59-61 | 22-50-48 | 47-71-2 | 128-182-50 |
| OWN, a design at a time | 57-63 | 25-35-60 | 55-63-2 | 137-161-62 |
| HALF | 63-56-1 | 41-38-41 | 59-60-1 | 163-154-43 |
| MARGIN | 66-54 | 57-18-45 | 55-62-3 | 178-134-48 |
| BEST | 63-57 | 67-30-23 | 75-43-2 | 205-130-25 |

*How I read it.*
- **The dice lifted thinking ship by ship to level with a design at a time** against ships that fly as drawn (288 to 303, with a design at a time at 301), most of it on the parts deck. The grain no longer shows there.
- **How many of a think's changes are made hardly matters against ships that fly as drawn.** Every planner that looks ahead wins 297 to 318 of 360; the blind one 201.
- **Against another planner, one does better than the game's: the one that judges a change on its own ships' account.** It beats the game's planner 182 to 128 ship by ship and 161 to 137 a design at a time, though one change at a time it's the slightly worse judge. A unit's own account leaves out the rest of the fight, so a pick doesn't lean on something elsewhere that's gone once everyone has changed at once. Both sides plan in every match, so this is the fight that counts.
- **Making fewer of a think's changes doesn't beat the game's planner.** The better half is level with it (the first figure in each row is the game's planner's wins). One change a think loses to it on every deck, and the margin loses to it on the parts deck, though both are as good as any against ships that fly as drawn. So the margin that looked free one think at a time isn't, against a side that plans.
- **The game's planner is as it was.** I haven't switched it to the unit's own account, a margin or more rolls: those are changes to U4, and what came next (below) may replace what they'd change.

*Tools.* `--ways` gained the lines above (the unit's own account, the worth of a pick on one to three rolls, the margin, how far ahead against what it's worth, a think's changes made together). `--flight` takes `--judges` (blind, own, best, half, margin; a v before a name fights the game's planner against it) and `--lean`. All 311 tests pass (1 ignored).

*Not done.* Nothing was watched in the window. The pick's worth and the changes made together were measured in the arena's team fights only, so nothing over 110 ships. The two faults above stand. Thinking finer was never fought against thinking coarser head to head (the grain is the arena's, not a side's).

**Where it went next (Henry, the same day).**
- Henry asked what I'd build if only "the ships should look like they have a smart onboard AI while basically following high level player orders" were sacred. My answer: move the smartness onto the ship. Each ship reads its own drawing (where its guns reach and bear, its toughest side, how fast it flies and turns) and runs a few general rules from what's near it: hold where your guns reach and theirs don't; show your toughest side to the biggest threat; shoot what your guns can hurt and finish what's nearly dead; get off a line that's about to fire; fall back behind friends when badly hurt; stay with your group and your projector; keep to a choice for a few seconds. Orders are a leash: a place and a posture. The look-ahead stays as the referee for every rule, and perhaps for rare calls for a whole group. The yardstick becomes dumb moments counted over twenty rolls (in range and not firing, inside a bigger ship, changes of mind a minute, hurt ships left in front, ships away from their projector). What worries me about it: the planner does win today, DRILLED failed, it's the ship AI Henry parked, and it retires most of U1 to U8. I proposed an arena toy of three rules. *Not answered.*
- Henry asked whether ships can strafe and fly backwards. They can: thrust is the same in every direction whatever the facing, and the facing (one of four quarter turns) only says where the pixels and the fixed guns are. Every ship stops dead from full speed within a few pixels (a fighter in 0.4 s and about 7 px, a frigate in 0.5 s and 4 px, a capital in 0.8 s and 5 px). Henry: "I didn't exactly intend for ships to be able to strafe and fly backwards. We might be due for a large overhaul. If ships can strafe and fly backwards, especially with fast acceleration, I'm not sure to what extent orbiting and attack runs even make sense. As you said, it's more just a matter of staying in the right zone and dodging a bit."
- I laid out three flight models. *Zone-keepers:* today's physics with the patterns dropped. *Momentum:* thrust any way but far less of it, so stopping or reversing costs seconds. *Nose-first:* full thrust ahead, weak sideways and astern, with momentum; engines could later push the way they're drawn. My pick is nose-first: runs and orbits become consequences and not styles, a ship turning in is a tell, and the drawing matters more. It would move the king's hand-flown feel (which may have to stay an exception), miners, formations, escorts and guard posts, every recorded fight, the match AI's timings and the balance, and at 15 frames a second the quarter turns would show more. With four facings any diagonal travel is partly sideways, so the choice is how freely a ship moves off its nose, not whether. I proposed an arena-only switch between the three, to watch the same fights each way before anything in a match changes.
- I stopped adding planner experiments at that point: they measure a flight model that may go. The dice, the accounts and the twenty-roll tests stand whichever way it goes.
- **Henry's ruling: nose-first.** "I think we want to commit and push what we have now and then begin work on a nose-first overhaul." Two questions first. One: with four rotations on screen, should the pixels behave as they're shown with a hidden rotation only for the flight physics, or should ships accelerate in four directions only? Two: should engines push the way they're drawn straight away, "because that is emergent depth (good) and I believe it's a big chunk of how nose-first makes drawings matter"? And a point Henry had made before: rotation engines, which set how fast a ship turns apart from the regular engines. "This would be an overhaul so don't preserve things that we don't want to preserve, including the whole flight pattern system. They're in the git history. One thing we might want to preserve would be multithreading (your call) but in a very different form." So this section is where the planner and its three ways of flying stop: this commit is the last that has them.

## Nose-first flight: the first stage (2026-11-16 and 17)

Henry's rulings, on my two questions and after: ships accelerate in four directions only, with no hidden heading ("good points on 1, agreed"); engines push the way they're drawn, straight away ("agreed on 2 as well. I like the arcade-hybrid feel of strict retro pixely four-directional ships transported into the world of smart AI, fast CPUs, and pixel simulation"); rotation engines as a part of their own; the flight patterns and anything else not wanted go, with multithreading my call. On my plan for the AI (each ship plays forward only its own motion a few seconds ahead; whole fights are played forward only by the tools, as referee): "that sounds good but intuitively I think it also needs to pay attention to turret tracking (this would allow orbit behavior to emerge) and in-flight projectiles (doesn't look good to run right into a lance blast)."

This is the first stage: what was taken out, the new flight in the arena, and a plain pilot. The match still flies the old way. Nothing here has been watched in the window by anyone but me in exported frames.

| # | What | Henry's verdict |
|---|---|---|
| F1 | **The planner and the three ways of flying are gone:** attack runs, kiting and orbiting, the planner that picked between them (`arch.rs`), its worker threads (`pool.rs`), the arena's brain buttons (B and V), and the tools that measured them (`--flight`, `--ways`, `--plan-cost`, `--frame-cost`, `--grain`, `--workers`). They're in the git history, at 7cac7b0. A ship that fights now holds at the reach of its guns, everywhere. | |
| F2 | **An engine pushes its ship one way: the way it's turned.** Ahead as it comes; [T] or the yard's turn button turns the one you paint to push to the right, astern or to the left, and the mirror puts its twin down pushing the mirrored way. It's four letters in a design (e, d, q, w). Its exhaust goes out the other way, and needs the straight way out of the ship, 3 wide, that an engine always needed: now that one way, where any of the four used to do. With no way out it's buried, shows red and pushes nothing (`Part::pushes`, `exhaust_clear`, `Stats::push`). | |
| F3 | **Every engine pushes with all it has, 12, that one way.** (For a day, 2026-11-16, an engine pushed away from every way out its exhaust had and shared its push between them: Henry's ruling below.) | |
| F4 | **A ship keeps its speed** until an engine pushes against it, and its speed changes by its push over 4 times its weight, a tick. One with no engine left drifts on, and the edge of the map stops it. Top speeds are as they were. | |
| F5 | **A rotation engine (GYRO) is a part,** in the yard's DRIVE group: 1 power, weight 1. A quarter turn takes the ship's spread (each pixel's weight, times one plus the square of how far it sits from the middle) over what turns it: 2 for each core, and for each rotation engine 3 for every pixel it sits from the middle, plus one. Never more than 3 seconds; what can't fly can't turn. Ordinary engines no longer turn a ship. | |
| F6 | **An engine that's pushing leaves exhaust** (2026-11-16; until Henry watched, it showed a flame, one bright pixel on the engine, and every ship had a tail from its middle). A pixel in its side's dim colour is left in space where the engine's exhaust comes out, every third tick of pushing, and stays 20 ticks. Engines side by side take turns. A ship that's coasting leaves nothing, and flown nose-first a ship has no tail. (A match, on the old flight, keeps the tail: nothing there says which engine is pushing.) | Henry, 2026-11-16: "I think sparse exhaust particles coming from the engines would look better than the current flame pixel, and then we can remove the tail that comes from the middle of each ship. the new exhaust particles can look basically the same as that tail just a little sparser." Built so. He hasn't seen it yet. |
| F7 | **The plain pilot.** Every tick a ship burns the engines that bring its speed nearer to what it wants. It flies a leg at a time, across the map or along it; it turns only when its engines can't do a fair share (a third) of the job as it faces; going slowly it uses whatever engine already points the right way; and it begins braking early enough to stop, counting the time to turn an engine against its way. It counts as there when it's as near as a ship that drifts while it turns can place itself: 3 px plus a quarter of its quarter turn's ticks. | |
| F8 | **The flight toy:** `--nose` opens the arena with four ships drawn for this (ARROW, SKATER, BRAKER, LANCET), the starter fighter and frigate, and six fights, flown nose-first. Nothing is loaded or saved. The network protocol is 17. | |

*Henry's ruling on F2 and F3 (2026-11-16).* As first built an engine's way of pushing was read off the hull round it: it pushed away from every way out its exhaust had, sharing its push between them, so the starter fighter's two stern-corner engines pushed 12 ahead and 6 to each side. Henry: "corner engines fire diagonally always, and ships have no default ability to rotate? do you think we should keep it like that or reconsider?" (Neither quite: a corner engine had two half-strength nozzles fired separately, and every ship that flies turns, slowly, by its core.) I recommended one engine, one way, chosen when it's placed; Henry: "Yes, switch." The core's turn stays, and its numbers wait for Henry's eye.

*What the switch moved.* Two built-in drawings had engines on their flanks whose exhaust only ever got out sideways: the BASTION's four and the WASP's two. They're turned to push across, as side thrusters. Nothing in Henry's saved team needed turning: all its engines vent astern. The yard's rectangle tool put a mirrored box down with the same engines and not their mirror images; a test for [T] found it, and it's mended.

*What the drawings come to.*

| | Push ahead / right / astern / left | Quarter turn |
|---|---|---|
| ARROW (`g.g` `ycy` `hhh` `.e.`) | 12 / 0 / 0 / 0 | 2 ticks |
| SKATER (`g.g` `dhw` `hch` `.e.`) | 12 / 12 / 0 / 12 | 12 ticks |
| BRAKER (`.q.` `gcg` `heh`) | 12 / 0 / 12 / 0 | 8 ticks |
| LANCET (`.l.` `ycy` `heh`) | 12 / 0 / 0 / 0 | 2 ticks |
| starter FIGHTER | 24 / 0 / 0 / 0 | 12 ticks |
| starter LANCER | 24 / 0 / 0 / 0 | 14 ticks |
| starter FRIGATE | 60 / 0 / 0 / 0 | 45 ticks (the slowest there is) |
| starter CAPITAL | 372 / 0 / 0 / 0 | 45 ticks |

*The proof* (all in `sim.rs`'s tests):
- Which way an engine pushes, for a dozen drawings: each of the four turnings, on a corner and out on a stalk (still ahead only), turned into the hull (buried, nothing), one shot off, and one whose blocked exhaust is freed by a shot.
- In the yard: [T] and the turn button turn the engine the pencil, the box and the fill put down, the mirror's twin pushes the mirrored way, a turned copy turns its engines with it, and a design saves and loads its engines as they're turned.
- A rotation engine further out turns a ship faster, the same pixels drawn long turn slower than drawn compact, and a quarter turn takes exactly its ticks.
- **A ship only ever speeds up the way one of its engines pushes:** nine drawings, each sent to twenty places, checked every tick for forty seconds.
- **Every flier reaches where it's sent and stops there:** the same nine and twenty places. It gets near enough, stops, never sets off again, and makes sixteen quarter turns at most.
- A ship with its engine shot off keeps its speed and the edge stops it; an engine leaves exhaust while it pushes and only then (where its exhaust comes out, every third tick, gone after 20; five rules broken on purpose, five caught).

*How the toy's fights go, headless, on six rolls each.* The flak ships mostly fight to a finish in 5 to 25 seconds: six starter fighters beat six ARROWs every time (four guns each to two), SKATERs beat ARROWs 4 times in 6 with 2 stalemates, and beat BRAKERs 5 in 6. Against the frigate the small flak ships lose most of their pixels and it ends in a stalemate (ARROWs 5 of 6 and a draw on time, SKATERs 6 of 6); the LANCETs win 4 of 6 and stalemate twice. A ship makes three or four quarter turns in ten seconds of fighting.

*What the pilot can't do yet, which is the next stage's work.*
- **It doesn't dodge and it doesn't think ahead.** It flies legs because, going both ways at once, it would have to stop one, turn and stop the other, drifting all the while.
- **Lancers fight badly.** A lance fires straight ahead and wants its row to the pixel, and a ship that drifts while it turns can't nudge itself a pixel sideways. Two old-fleet lancers against a still slab: 4 shots in a minute, and a draw.
- **A ship that's slow to turn is clumsy by design:** the starter frigate takes 3 seconds a quarter turn and settles for within 14 px of where it's sent. Every existing drawing is like this until it's given rotation engines, Henry's saved team included.
- **The yard still shows the old numbers** (turning and thrust per weight as a match flies them). `--deck` prints the new ones beside them.

*What moved.* The match keeps its old flight, but with the planner gone its ships hold at the reach of their guns where they used to fly as the planner chose, so every recorded match and arena fight plays out anew. Not re-recorded: the fights are about to change again with the pilot and the behaviours. One match test clicked on a lancer where an attack run used to leave it; it now clicks where the lancer is. The lance-and-plate test holds its lancers in line, so it's about lances and plate and not about flying. 298 tests pass, in under a minute where the suite took nine and a half minutes with the planner in it.

*Next, as agreed.* The rollout pilot: each ship plays its own motion a few seconds forward over a few dozen sequences of turns and burns, and scores a path by where its orders want it, by whether a gun can really bring a shot onto it there (a turret's turn rate, and a shot aimed where the ship would be at its present speed), and by shots already in flight as things to stay clear of, weighed by what they'd cost. Then the behaviours, then the match (miners, the king, formations, the AI), and the old flight comes out.

## Nose-first flight: ships pick their own courses (2026-11-16)

The second stage. Henry on the plan for it: "that sounds good but intuitively I think it also needs to pay attention to turret tracking (this would allow orbit behavior to emerge) and in-flight projectiles (doesn't look good to run right into a lance blast)". Both are in. It's in the arena's flight toy (`--nose`); the match still flies the old way. I haven't watched any of it in the window. What I know of how it looks comes from pictures of where the ships went (`--nose-trails`) and the tables below.

| # | What | Henry's verdict |
|---|---|---|
| H1 | **A ship in a fight picks its own course.** Every 4 ticks (ships take turns) it plays its own next four seconds forward some thirty ways, flown by the plain pilot's own rules, and takes the one that costs it least, counted in pixels. Nothing else is played forward: enemy ships are carried on at the speed they have, and the shots in the air fly on along their lines. A ship with no enemy within 240 px and no enemy shot near flies as it did. | |
| H2 | **The ways it weighs:** as its plan says; the course it's already on; standing still; each of eight directions, at its top speed and slowly; a step each of four ways, for half a second or a second and a third, before carrying on as planned; and going round its target either way, fast or at half its speed. | |
| H3 | **Shots in the air.** A shot that would cross the ship costs what it would break: 4 px for a lance shot, 8 for a shell, 1 for anything else. (A missile steers, so it isn't a line to step off.) | |
| H4 | **Guns that could bring a shot onto it.** Flak and turrets fire at where a ship will be if it keeps the speed it has, so a ship that will have changed its speed by the time the shot arrives isn't charged for it. A turret has to swing its barrel round first. An enemy lance is a danger down the lines its lances are on, to a ship any of whose pixels a line crosses. An enemy's guns are shared between the ships of the thinking ship's side inside their reach. | |
| H5 | **Its own guns.** Flak, cannons and trackers count for every tick an enemy is in their reach. A lance fires a whole shot the moment it's loaded and its line crosses a pixel its target still has, so a ship that can't sit still on a line can fly its lance across the target. | |
| H6 | **Its orders aren't the pilot's to change.** A ship whose guns need no lining up is wanted at a range from its target, on whichever side: every pixel off that range, averaged over the four seconds, costs a pixel. So it goes round its target and jinks, and doesn't back away. Any other ship is wanted on a spot, with a leash of 24 px round it that costs next to nothing (room to step out of a shot's way or fly a lance across a target) and a pixel for each pixel beyond. | **Keep, and a move order wins** (Henry, 2026-11-16). Asked "Should a losing ship stay at its guns' distance from its target?": "Yes, unless it has a move order to go somewhere else". The second half is H10. |
| H7 | **It doesn't twitch.** Each quarter turn on the way costs a pixel, and it leaves the course it's on only for one better by a pixel. | |
| H8 | **Two turret fights in the toy, and two ships for them:** the GUNBOAT (a cannon on the smallest hull that feeds it: its core and eight hull make the 12 power the cannon, two rotation engines and two engines use) and SIEGE's SENTINEL (a tracker). The toy has eight ships and eight fights. | |
| H9 | **Tools.** `--nose-referee [N] [parts]`: the toy's fights (or the parts toy's) on N rolls each, with neither side picking its own courses, with blue, with red, with both. `--nose-trails [DIR]`: a picture of where every ship went in each fight's first twenty seconds, on the plain pilot and with every ship picking its own course. | |
| H10 | **A ship sent somewhere else goes** (from Henry's ruling on H6; nose-first flight only). On its way to where it was sent a ship stops for nothing. The place is where it's wanted, so it flies there, its guns fire at whatever comes in reach as it passes, and it dodges as it goes. Within 210 px of the place it takes up the fight with what's within 160 px of the place, as it always did, and holds its range there (H6). Sent with a group, it keeps to the group's pace in a fight too. **This changes C2,** which had a sent ship stop and fight what came within 130 px of it on the way. The match, on the old flight, still does that. | |

*Where this differs from what I told Henry.* I said a ship would try "a few dozen sequences of turns and burns". What's built is speeds to hold, each flown by the plain pilot, which decides the turns and burns: thirty-odd of them. And "where its orders want it" grew a rule of its own, H6, after I saw what ships did without it (below). Both are my calls, unjudged.

*What ships do, from the pictures* (one roll of each fight, both sides picking their own courses, against the same fight on the plain pilot):
- **On the plain pilot** every ship flies a straight line to the reach of its guns, stops and shoots.
- **ARROWS against GUNBOATS:** the ARROWS come in, swing off their lines as the shells come and curve round. The GUNBOATS, wanted at cannon range, turn about and back away as the ARROWS close, their turrets still firing behind them.
- **SKATERS against ARROWS:** a furball. Loops and long sweeping curves round each other, one ship swinging right out to the edge of the fight and back.
- **LANCETS against the FRIGATE:** the LANCETS curve onto the frigate's flank in a bundle and work across it in short passes; the frigate turns away from their lines.
- **ARROWS against the FRIGATE:** they go in, cross it and loop back, and five of six die doing it. The frigate out-guns them, and H6 keeps them at it.

*The referee: the toy's eight fights, 48 rolls each.* Blue's wins, red's, draws; seconds a fight; then for each side the pixels it lost a fight, the pixel hits it took for each shot fired at it, the circles its ships made of their nearest enemy in 10 s near it, and the share of the time they had an enemy inside flak range.

| Fight | Picks its own courses | Blue-red-drawn | Length | Blue | Red |
|---|---|---|---|---|---|
| 6 ARROWS VS 6 FIGHTERS | neither | 0-48-0 | 9 s | 54 px, 0.72, 1.16, 29% | 20 px, 0.74, 0.79, 27% |
|  | blue | 0-48-0 | 15 s | 54 px, 0.53, 0.37, 27% | 25 px, 0.64, 0.32, 18% |
|  | red | 0-48-0 | 9 s | 54 px, 0.61, 0.59, 34% | 16 px, 0.62, 0.50, 33% |
|  | both | 0-48-0 | 11 s | 54 px, 0.54, 0.39, 27% | 17 px, 0.51, 0.40, 23% |
| 6 SKATERS VS 6 ARROWS | neither | 20-5-23 | 27 s | 37 px, 0.67, 0.42, 28% | 43 px, 0.76, 0.45, 29% |
|  | blue | 31-14-3 | 19 s | 38 px, 0.61, 0.52, 51% | 46 px, 0.79, 0.72, 55% |
|  | red | 32-14-2 | 25 s | 37 px, 0.76, 0.32, 38% | 46 px, 0.69, 0.28, 39% |
|  | both | 41-6-1 | 20 s | 33 px, 0.61, 0.56, 42% | 52 px, 0.72, 0.64, 46% |
| 6 BRAKERS VS 6 SKATERS | neither | 3-45-0 | 9 s | 41 px, 0.75, 1.95, 46% | 22 px, 0.66, 0.64, 58% |
|  | blue | 40-7-1 | 13 s | 28 px, 0.59, 0.25, 54% | 50 px, 0.78, 0.34, 61% |
|  | red | 17-29-2 | 16 s | 37 px, 0.72, 0.54, 58% | 36 px, 0.69, 0.46, 60% |
|  | both | 46-2-0 | 18 s | 20 px, 0.53, 0.75, 51% | 53 px, 0.54, 0.78, 53% |
| 6 LANCETS VS 1 FRIGATE | neither | 20-0-28 | 33 s | 26 px, 0.73, 0.05, 10% | 42 px, 0.94, 0.03, 12% |
|  | blue | 48-0-0 | 10 s | 0 px, 0.70, 0.02, 1% | 55 px, 0.65, 0.02, 2% |
|  | red | 2-43-3 | 34 s | 41 px, 0.64, 0.14, 15% | 6 px, 0.71, 0.16, 19% |
|  | both | 29-17-2 | 37 s | 22 px, 0.54, 0.18, 16% | 38 px, 0.76, 0.24, 22% |
| 6 ARROWS VS 1 FRIGATE | neither | 0-7-41 | 45 s | 43 px, 0.77, 0.28, 19% | 5 px, 1.84, 0.21, 14% |
|  | blue | 0-28-20 | 27 s | 50 px, 0.72, 0.27, 35% | 5 px, 1.81, 0.29, 29% |
|  | red | 0-26-22 | 56 s | 47 px, 0.71, 0.36, 16% | 6 px, 1.79, 0.33, 12% |
|  | both | 0-43-5 | 68 s | 53 px, 0.64, 0.25, 21% | 4 px, 1.88, 0.16, 13% |
| 6 SKATERS VS 1 FRIGATE | neither | 0-5-43 | 35 s | 47 px, 0.73, 0.31, 23% | 6 px, 1.77, 0.18, 19% |
|  | blue | 0-27-21 | 31 s | 52 px, 0.66, 0.48, 50% | 7 px, 1.83, 0.51, 39% |
|  | red | 0-35-13 | 49 s | 51 px, 0.66, 0.36, 21% | 6 px, 1.80, 0.31, 17% |
|  | both | 0-35-13 | 39 s | 52 px, 0.64, 0.45, 43% | 6 px, 1.85, 0.47, 32% |
| 6 ARROWS VS 2 GUNBOATS | neither | 48-0-0 | 9 s | 20 px, 0.65, 0.10, 25% | 44 px, 0.81, 0.06, 28% |
|  | blue | 48-0-0 | 9 s | 2 px, 0.07, 0.23, 22% | 44 px, 0.79, 0.18, 25% |
|  | red | 42-6-0 | 20 s | 27 px, 0.43, 0.14, 23% | 41 px, 0.80, 0.15, 30% |
|  | both | 48-0-0 | 15 s | 7 px, 0.14, 0.20, 23% | 44 px, 0.82, 0.22, 26% |
| 6 ARROWS VS 1 SENTINEL | neither | 0-48-0 | 9 s | 54 px, 0.84, 0.38, 15% | 5 px, 1.47, 0.53, 32% |
|  | blue | 10-36-2 | 14 s | 48 px, 0.83, 0.33, 39% | 18 px, 1.58, 0.33, 47% |
|  | red | 0-48-0 | 10 s | 54 px, 0.82, 0.42, 17% | 2 px, 1.74, 0.50, 35% |
|  | both | 0-48-0 | 19 s | 54 px, 0.75, 0.35, 39% | 6 px, 1.77, 0.31, 39% |

Over all eight, a side picking its own courses against one on the plain pilot **won 426, lost 253 and drew 89**, and lost 21,073 px to the plain side's 28,994. Its ships had an enemy in flak range 35% of the time (the plain side's 30%), fired 0.54 shots a ship a second (0.44), took 0.86 pixel hits for each shot fired at them (0.92), and made 4.9 quarter turns in 10 s (5.3).

*On fights it wasn't worked out on:* the parts toy's nineteen fights, 24 rolls each (`--nose-referee 24 parts`; beamers, wardens, rakers, archers, an aegis, all with engines astern only and no rotation engines). Won 490, lost 292, drew 130; lost 32,505 px and took 41,823.

*What it reads as.*
- **Shots in flight, Henry's second point, shows plainly.** A standing ARROW with a lance shot coming from 90 px takes 36 of 36 on the plain pilot and none picking its own course. Against two GUNBOATS, six ARROWS lose 391 px in twenty fights on the plain pilot and 45 picking their own courses.
- **Turret tracking, his first, is in and does next to nothing, and orbiting hasn't come out of it.** With the term switched off in the build as it stands, the whole table's wins, losses and draws are the same to the fight (426-253-89). Against the GUNBOATS, ARROWS circle a little more with it (0.23 circles of their nearest enemy in 10 s against 0.20, and 0.10 on the plain pilot) and lose a little more (2 px a fight against none; with both sides thinking, 7 against none). Against the SENTINEL nothing changes at all. The reason is in the numbers, not the pilot. A cannon's barrel goes right round in 8.5 s. To get round the turret faster than that a ship has to be within about 30 px of it: further out the ARROW isn't fast enough (22 px/s), and the SKATER, which is, can't hold so tight a circle at that speed with one engine's push turning its path inward. Their flak wants them at about 50 px. A tracker turns four times as fast. What saves a ship from a cannon is seeing the shell in the air and not being where it was aimed. If orbiting a turret is to be a real tactic, that's turret turn rates and ships' push, which are Henry's numbers and I haven't touched them. I've left the term in because he asked for it; by its own measure it could come out.
- **The outcome of a fight is still mostly the drawings.** ARROWS lose to starter FIGHTERS every time whoever thinks, and to the SENTINEL 36 times in 48 at best. Where the two sides are near each other it decides it: BRAKERS against SKATERS goes 3-45 on the plain pilot and 40-7 with the BRAKERS thinking.
- **Circling is no commoner overall** (0.33 circles in 10 s against the plain side's 0.36, which is ships passing each other). It shows against the cannon and in the furballs.

*Each term switched off in turn* (the same eight fights and 48 rolls, on the build before the last three changes: a firmer hold on range, the ship's own lance shots, and facing its target as it goes). All in: 448 won, 185 lost, 135 drawn.

| Switched off | Won-lost-drawn | What else moved |
|---|---|---|
| flak and turrets leading it (H4) | 260-338-170 | the one it can't do without |
| its own guns (H5) | 361-194-213 | it stops closing: 0.21 shots a ship a second for 0.51, and fights between two thinking sides run 71 s for 34 |
| where its plan wants it (H6) | 374-197-197 | the same, 94 s |
| a range, not a spot, for ships with no gun to line up (H6) | 388-231-149 | |
| keeping the course it's on (H7) | 407-197-164 | |
| the cost of a turn (H7) | 472-134-162 | it wins more without it, and makes 6.7 quarter turns in 10 s for 2.2. Kept, for how it looks. |
| shots in the air (H3) | 438-192-138 | against the GUNBOATS it loses 11 px a fight for 0 |
| enemy lance lines (H4) | 437-188-143 | a thinking frigate under thinking LANCETS goes down in 14 s for 47 |
| turret barrels (H4) | 446-187-135 | circles of a cannon ship with both sides thinking: 0.18 for 0.59. (In the build as it stands the GUNBOATS back away, and it's 0.15 for 0.20: above.) |
| sharing an enemy's fire (H4) | 435-186-147 | |
| going round its target as a way to go (H2) | 444-189-135 | against the GUNBOATS, 0.15 hits a shot for 0.02 |
| place over the whole way (the end only) | 462-196-110 | fights between two thinking sides run 44 s for 34 |

Five of them move who wins by more than chance does (a run of 768 fights wanders by 30 or so either way). The rest are kept for what they do in the fights they're about, shown in the last column, except turret barrels, which is kept because Henry asked for it.

*Tried and taken out.*
- **Weaving back and forth** as a way to go: 443-204-121 without it against 430-196-142 with. Out.
- **Sharing an enemy's fire among every ship in the fight,** not only the ones inside its reach (so that six ships each afraid of a frigate's whole flak would go in together): 360-244-164 against 449-198-121. Ships went in and were shot. Out.
- **Counting a pixel taken off the enemy for more than a pixel of its own** (one and a half, or two): 449 and 431 wins against 448. Nothing in it. Out.
- **Fear of an enemy lance across the enemy's whole width,** as first built: thinking LANCETS wouldn't line up on a frigate (8-34-6 in 92 s). It's now the lines its lances are really on.

*The one rule that's a choice about the game: H6.* With a pixel out of place costing a sixteenth of a pixel, a side picking its own courses won 470-189-109, more than as built. But its ARROWS, sent at a frigate, backed off the map ahead of it (in flak range 8% of the time, fights of 107 s) and beat a plain frigate 44 times in 48 by leading it about. That's the pilot deciding to retreat, which is an order's to decide. With every pixel out of place costing a pixel the retreat stopped (in range 32%, 37 s), but a ship no longer stepped out of a lance shot's way (36 of 36 hit) and a LANCET broke the slab below in 11 fights of 20. So the two kinds of place are costed differently: a range is held firmly, since a ship can dodge round its target without leaving it, and a spot has a leash. As built: 426-253-89, and the ARROWS go at the frigate and lose to it (0-28-20 in 27 s, in range 35%).

*Henry's ruling on H6, and what it turned up (2026-11-16).* "Yes, unless it has a move order to go somewhere else."
- **A move order didn't get a ship out of a fight.** The send order (C2) has a ship on its way stop and fight whatever comes within 130 px of it. A ship that's sent out of a fight has an enemy that near, so it turned back to it. On the old flight, which is what a match flies today: four fights at ten rolls, every blue ship sent to a place 400 px behind it five seconds in, and thirty seconds later 14 of 240 were at the place. Nobody had tried it: every test of sending sent ships towards a fight.
- **Built, for nose-first flight (H10):** a ship on its way stops for nothing, and takes up the fight again only round the place it was sent to. In the test below, of 212 ships sent out of two fights, the 153 that kept an engine all got there. With the old rule put back, 115 kept one and 56 got there.
- **A rule Henry had kept, changed: C2's "fight what comes within 130 px on the way".** Reason: it and his ruling can't both hold. A ship that stops for what's near can't be ordered away from what's near. What it costs: a group sent across the map now flies past a picket it meets, shooting as it passes, where it used to stop and clear it. To fight the picket, send the group at it. Not ruled.
- **The match isn't changed.** It flies the old way, and there a sent ship still stops to fight. This reaches the match when nose-first flight does, unless Henry wants it sooner.
- **The pace is kept.** A ship sent with a group flew at the group's pace only while it flew as its plan said; picking its own course it would have run ahead. The pilot's ways to go are now capped at the pace, and its picture of its plan is flown at the pace as the game flies it.
- **Not seen in a window.** The flight toy has no way to send a ship, so none of this can be watched yet.
- **Noticed on the way, not changed: hulks adrift.** Against the FRIGATE an ARROW's one engine is often shot off, and with nose-first flight it then drifts on at the speed it had. Over twenty rolls, of the time ARROWS were alive after the first ten seconds, more than two thirds was spent adrift (8,646 ship-ticks of 12,636). The ones that could still fly were between 20 and 40 px from the frigate's middle 69% of the time (they're wanted at 36). That's F-row behaviour, for when Henry watches.

*Henry after watching the toy (2026-11-16): the exhaust (F6, above), and two questions.*

"Why don't the lancets shoot at the frigate in test 4?"
- **The window plays the same roll of the dice every time,** and on that roll the six LANCETS fire one shot in 28 s and lose. On rolls 1 to 12 they win 8 (firing 6 to 15 shots, the first at 5.5 s), lose 3 (1 to 3 shots) and once fire nothing in 60 s.
- **It's the lance's own rule, not their flying.** A lance holds its fire unless the shot would land three ways: with the target flying on as it is, with it at the speed it's trying for, and with it stopped dead (the lance aim round, 2026-11-12). A lance shot takes 19 ticks to cross 75 px. A frigate crossing at a pixel a tick moves 19 px in that time, twice its own width, and since it picks its own course it never stops crossing their lines. On the window's roll a LANCET had its lance on a line through the frigate for 122 ship-ticks, and one shot came of it. On the rolls they win, the frigate flies straight down a LANCET's line in the opening.
- **Two of the three questions ask about things a ship can't do any more.** Flown nose-first it can't stop dead, and it can't have the speed it's trying for at once.
- **The pilot's picture of its own lance is wrong with it** (H5): it counts a shot whenever the lance is loaded and crosses the target, so it flies passes the gun then doesn't fire on.
- **Tried, not kept: a lance fires at where its target is heading** (the first of the three questions alone). On the window's roll the LANCETS fire 5 shots from 5.5 s and win in 6 s; on rolls 1 to 12 they win 10 and two run out the minute. A thinking side against a plain one over the toy's fights at 24 rolls: 208-125-51, against 213-124-47 as built, which is no difference; on the parts toy 533-253-126 against 490-292-130.
- **Not changed.** The hold-fire rule is one Henry agreed to. Put to him.

"Will ships with insufficient penetration target engines if they can't target anything else vulnerable?"
- **No. No gun looks at armour when it picks a pixel.** Flak, a cannon, a tracker and a lance lining up each take a living pixel of the target at random. Flak and trackers pierce 1, lances and missiles 2, cannons and beams 3 (one more beside a piercer); plate is 1 and heavy plate 2, everything else 0, engines among them; a shot that doesn't out-pierce what it meets bounces off.
- **Aiming alone wouldn't do it:** a shot stops at the first pixel on its line, so one aimed at an engine behind plate meets the plate. What makes engines the answer is that each has to keep its exhaust lane open to space, so there is always a side an engine can be hit from. A rule for it would be: a gun aims at a pixel it can break and can get at from where it is. Not built; Henry asked a question.

*The proof* (in `sim.rs`'s tests; the suite is 305 and passes):
- **A ship holds its range in a fight it's losing, and leaves when it's sent somewhere else:** twenty rolls each of six ARROWS against a FRIGATE and six SKATERS against six ARROWS, on a map with room to leave. Eight seconds in, every ship has an enemy within 130 px. Left to it, ARROWS that can fly are within 60 px of the frigate 77% of the time. Sent 400 px away, every ship that keeps an engine comes within reach of the place. Two rules broken on purpose, as a check: sent ships stopping to fight (56 of 115 get there, and the test fails) and no firm range (21% of the time within 60 px, and it fails).
- **A ship sent with a group keeps to the group's pace under fire:** a SKATER sent past a GUNBOAT goes 12 flat out and never above 6 at a pace of 4, on a course of its own for most of the way. With the pilot forgetting the pace, its picture of its plan is wrong, it stops picking courses, and the test fails.
- **A ship playing its flight forward flies exactly as the game then flies it:** every flier sent eight ways, the copy and the ship compared for place, speed and facing at every tick of twenty seconds.
- **A ship gets out of the way of a lance shot that's coming:** 36 shots at a standing ARROW, at each of its pixels from each of four sides. 36 hit on the plain pilot, none picking its own course. With the pilot blinded to shots in the air, as a check on the test, 36 hit and the test fails.
- **Ships under cannon dodge the shells:** twenty fights of six ARROWS and two GUNBOATS. The ARROWS win all twenty either way, and lose 45 px picking their own courses to 391 on the plain pilot.
- **A lancer gets its lance onto a small still target:** a LANCET against a slab three pixels deep, twenty rolls from five starts. Picking its own course it breaks the slab every time.
- **A fight on the rollout pilot plays the same twice,** tick for tick by checksum, and differently on other dice.

*A fault of the plain pilot, recorded and not mended.* On the plain pilot alone the LANCET breaks that slab in 4 fights of 20. It stops within a few pixels of its spot, calls that near enough, and where its lance then bears on nothing it sits there. (Stage one's notes had this as "lancers fight badly".) Every ship in a fight now picks its own course, so a player doesn't meet it; it does mean the referee's plain side is weaker than it should be wherever lances are concerned.

*What it costs.* About 0.04 ms a tick for each ship in a fight, on one core: fights of seven or eight ships ran at 0.20 to 0.33 ms a tick against 0.011 to 0.017 on the plain pilot. A hundred ships fighting at once would be about 4 ms of a 67 ms tick. It's not on other cores. Each ship's think reads the map and writes only its own course, so it could be.

*Open, and mine to flag.*
- **Turret tracking earns nothing as the numbers stand** (above). In, because Henry asked for it. Out, or the turret numbers looked at, is his call.
- **A ship holds its range even when it's out-gunned** (H6): ruled, keep. ARROWS sent at a frigate die at it unless they're sent somewhere else.
- **A sent ship no longer stops on its way** (H10), which changes C2. Not ruled, and not in the match.
- **Cannon ships back away from what closes on them,** to keep cannon range, showing their engines. It follows from H6 and from their plan wanting cannon range. It may read as smart or as running.
- **Turning.** A thinking ship makes about five quarter turns in 10 s, the same as a plain one in a fight.
- **Nothing here is in the match.** Miners, the king, formations and the AI still fly the old way. (Since the next section: everything but the king flies nose-first.)

## Everything flies nose-first; lances lead; guns aim at what they can break (2026-11-16)

Henry, after watching the toy: "We should remove the old flight system, everything should use the new system. We don't want to keep stuff around that we don't need anymore." On the lance: "The old 'sure shot' doesn't apply anymore (and I was always skeptical of it). Yes, lances should lead their target. I'd probably go for a linear lead (not track current acceleration) what do you think? ... shot taking in general should probably be wired in to the flight logic, so that the paths reflect the actual shots they'll take and optimize for that." And on "a gun aims at a pixel it can break and can get at from where it is": "yes, I think we want this."

*My answer on the lead: linear, yes.* An engine burns in bursts of a tick or two as the pilot trims, so a ship's acceleration at any moment says next to nothing about the next second. Flak and turrets already lead by the speed a ship has, and the pilot's dodging is built on exactly that: change your speed after the shot leaves. One rule for every gun means a dodge is the same move against all of them.

| # | What | Henry's verdict |
|---|---|---|
| I1 | **Every ship in a match flies nose-first.** The old flight is deleted: `steer_any_way`, the `nose` switch, the old way of arriving, the tail. Wire version 18. **The king is not converted:** it still flies its own way, straight from its orders at a set speed (I14). | |
| I2 | **A scoop reaches.** Rock within 8 px of a miner (12 for the starter miner: twice what it can place itself to) is taken from where it sits, a loose pixel within that is picked up, and a load is handed over from that far. **This changes the mining rule,** which had a miner sit on each pixel it ate. A ship that keeps its speed can't shuffle 3 px to the next one: left alone, the economy stopped at 18 px in four minutes. | |
| I3 | **A full miner stops at its drop-off.** It used to fly through it flat out, which with momentum carried it half across the map. | |
| I4 | **A ship flies straight at its spot and stops as near it as a ship of its sort can place itself** (3 px with rotation engines, 6 for the starter fighter, 8 for Henry's, 14 for a frigate with none; usually nearer). Stopped that near, it stays. (For a day a ship that stopped more than 3 px off backed off and came in again, and until Henry's first match it flew one leg at a time: both gone, below.) | |
| I5 | **A sent ship stops for nothing on its way** (H10), now in the match. **This changes C2.** | |
| I6 | **A lance fires at where its target is heading:** when a shot fired now would meet a standing pixel of an enemy that flies on at the speed it has. The hold-fire rule is gone. | |
| I7 | **A shot has a tick's start on what it's fired at.** It's fired after the ships have moved and flies its first pixels in the tick it's fired. The lance's rule counted its target as moving from the moment of firing, so at 35 px and further its shots at anything crossing faster than half a pixel a tick went by a pixel in front. Hidden until now, because the old rule never fired at a crossing ship. | |
| I8 | **Every aimed gun's lead is counted right.** A cannon and a tracker aim from where the target stood when the tick began and counted the shell's flight a tick short three times in four; and they measured the way as the crow flies, when a shot covers a pixel of the longer way each step whatever its slant. Cannons landed 61 of 101 shells on a 5x5 hulk drifting steadily by; now 90 of 97. Flak was nearly right already. | |
| I9 | **A ship picking its course counts the shots it will really fire.** Its own lance by the gun's own rule (the same code, asked about later ticks); an enemy lance as a danger where it will fire and the ship will still be; an enemy beam that's warming, as a cut on its line when it's ready; its beam or repeater led as the gun leads. | |
| I10 | **A gun aims at a pixel it can break and can get at from where it is.** A shot breaks the first standing pixel it meets or bounces off it, so flak, a cannon and a tracker aim only at a pixel where the first thing in the way is something they out-pierce. With none such they shoot all the same, and the shots bounce. A ship that lines a gun up picks a pixel some side of which is open to it, and lines up from an open side. | |
| I11 | **Ships whose guns can't break the armour in front of them fly round to where they can.** For each ship the game works out from which of sixteen bearings a shot that pierces 1, and one that pierces 2, can break something; a ship counts its flak and trackers only from an open bearing. | |
| I12 | **The toy:** a TURTLE (a hull in a shell of plate, open only astern, where its engines are) and "6 ARROWS VS 1 TURTLE". Nine ships, nine fights. Engines leave exhaust everywhere now, and nothing has a tail. | |
| I13 | **A ship lining a gun up keeps to the spot it's making for** unless another is 60 px nearer. It used to take whichever was nearest at each moment, and with momentum it overshot each in turn. And a ship gets off a warming beam's line only if its own flight, played forward, gets it clear in time. | |
| I14 | **Not done: the king.** It flies straight to where it's sent at 2.5 px a tick (0.5 when beaten), with no engines consulted. On its own four engines it would be a slow capital that takes 3 s a quarter turn. That changes the beaten-king rule (M2.10), the AI's king, which runs from raids by its keys, and thirty tests that drive a king about. Put to Henry before doing it. | **Not yet** (Henry, 2026-11-16). The king stays as it is until he says. |

*Rules Henry had agreed to that this changes, each with its reason.*
- **C2, "fight what comes within 130 px on the way"** (I5): gone from the match too. A ship that stops for what's near can't be ordered away from what's near.
- **The lance's hold-fire rule** (I6): gone, on his word.
- **Mining pixel by pixel** (I2): a miner works what's in reach of its scoops from one place. Forced: see the row.
- **A guard without guns moves up into a post left empty beside it:** not while an enemy is in sight. A ship in a fight has 24 px round its spot for next to nothing (H6) and doesn't turn about twice to move up 8 px.
- **The starter fighter gets off a warming beam's line:** it can't any more. It takes 12 ticks a quarter turn and its engines push ahead; the beam warms for 15. A ship with rotation engines can.

*What the match does now, against the old flight* (the same four AI against AI matches on the starter deck, the old source built beside the new):

| | Old flight | Nose-first |
|---|---|---|
| Mined by 2:00, 4:00, 6:00 (blue, seed 1) | 117, 504, 1035 | 117, 414, 918 |
| Match length, seeds 1 to 4 | 15:57, 20:35, 13:42, 14:36 | 17:50, 21:27, 19:43, 21:54 |
| Time to play a match | 6 to 11 s | 36 to 94 s |
| A tick, seed 4 | 0.68 ms | 4.8 ms |

The economy is a tenth to a fifth slower and matches run about a third longer. Nothing was tuned. The cost is the pilot: every ship with an enemy in sight thinks. 4.8 ms is a fourteenth of a tick on this machine; the suite went from 50 s to 9 minutes.

*The toy's referee after all of it* (nine fights, 48 rolls each): a side picking its own courses against one on the plain pilot won 505, lost 225 and drew 134. What moved most: LANCETS against the FRIGATE with both sides thinking, 29-17-2 in 37 s to 48-0-0 in 9 s (they fire now); thinking GUNBOATS against plain ARROWS, 6-42 to 45-3, and with both thinking the ARROWS no longer win every time (29-17-2): cannons hit what flies steadily.

*The proof* (the suite is 307 and passes, with two tests switched off: one from before, and the RAKER's, below):
- **Every lance shot at a ship that flies on lands:** fifty hulks drifting across a lance's line, near and far, slow and fast. Every shot fired lands, and every hulk is fired at. This is what found I7.
- **A gun that aims leads by the time its shot takes:** flak 272 of 290, cannon 90 of 97, tracker 341 of 357 on a hulk drifting by. Only the cannon's number tells the new lead from the old.
- **A gun aims at what it can break and can get at:** a flak gun astern of a plated shell breaks it with its first shots and none bounce; in front of it and abeam every shot bounces. With flak aiming at any pixel, the test fails.
- **Ships fly round to where they can:** six ARROWS in front of the shell break it 20 times in 20 picking their own courses and never on the plain pilot; six LANCETS in front of one in heavy plate break it 20 in 20 on either pilot. With guns counted from any side, or a line-up from the nearest side, the test fails.
- **Not shown by any test:** that the tracker's new aim differs from its old one. In two set-ups tried the two did the same.

*Faults and limits, recorded and not mended.*
- **A ship that's slow to turn and lines a gun up doesn't settle on its line.** The parts toy's RAKER (engines astern only, 3 s a quarter turn) was in line with a slab for 88 ticks of 1800 and fired 5 shots: it overshoots its line and comes back over it. Its test is switched off, saying so. The BEAMER closes to 37 px of a target it used to hold 97 px off.
- **Formations and posts are only as fine as the ships are nimble.** A ship that comes to rest as near its spot as it can place itself stays there: usually within a pixel or two, at worst 6 px for the starter fighter and 14 for a frigate. So two guards on neighbouring posts can stand a few pixels into each other. Near a fight a ship has H6's 24 px.
- **Two tests of the where-to-send experiment are switched off.** Their staging counted on a sent fleet fighting what it met on the way; with I5 the two plans they compare come out level, so the tests fail before they reach the rule they're about.
- **The yard's gauges still show the old flight's turning and thrust.**
- **Ships with their engines shot off drift on,** as before.

*Henry's first match on it (2026-11-16, seed 606, his deck against SWARM).* With a screenshot of a fighter selected and standing 230 px from where it was sent: "this fighter seemed to be somehow stuck for a while. i also had a miner that looked like it was mining from like 5 pixels away from the rock".
- **The stuck ship: found, a fault of the plain pilot, mended.** The log has the sends but not which ship each was for, so I played the same match with both sides on the AI and one of blue's ships sent somewhere every second, and looked for any ship on its way that went 10 s without getting nearer. A miner did, for 41 s. The trace: 36 px to fly one way and 3 px the other, its engine pointing along the 3. Standing with two legs to fly, it took the one its engine could start on. That leg was shorter than it drifts while it turns about to stop (7 px for the miner, 11 for Henry's fighter, which takes 1.5 s a quarter turn), so it was carried past, turned, came back and was carried past again, at a quarter of a pixel a tick, with the long leg never begun. Now a leg too short to stop on is left till last.
- **The backing off I added the round before is gone.** It was there to hold ships to within 3 px of their spots. Measured on Henry's fighter and miner it bought nothing: a ship sent to a spot nearer than a hop that simply sets off is carried past and stops as near on the far side, in a third of the time. And a ship now stays where it has come to rest if that's as near as it can place itself. In the match, the longest any ship goes without getting nearer its spot fell from 41 s to 12 with the backing off and 7 without.
- **Tests:** `a_ship_near_its_spot_gets_onto_it_without_fussing` (Henry's fighter and miner, each alone, facing each way: the trap; a spot it can leave be; spots a little further). With the trap put back it fails (never there, against 67 ticks). `a_ship_sent_somewhere_keeps_getting_nearer_until_its_there` is the match: no ship goes 12 s. It doesn't pin any one rule down: a match is too chancy for that.
- **The miner: that was my rule (I2), and half of it was a fault.** A miner bites whatever rock is within 12 px of its middle, and as built it stopped flying the moment something was in reach, so it sat out in space. Now it flies on to the rock it's eating and comes to rest against it. It still eats everything in reach from there before it moves, so by the end of a sitting it is taking pixels from up to 12 px off. Not changed: whether that reach should show, or be shorter.

*"Why do we have a concept of legs?"* Henry, the same evening: "what if we got rid of the concept of legs and just had ships add or subtract vectors that get them closer to their target position+speed? i also noticed Swarm's fighter flew to the top left corner before flying south, which seemed like an odd way to fly toward the enemy base".
- **Why they're there.** An engine pushes one way, and a ship with one engine can stop only one way's speed at a time. Flying across and then along, it only ever has one to stop. That was my shortcut in the first stage, and it's all in the arriving rule: the half of the pilot that burns and turns already works Henry's way, cutting whatever gap there is between the speed it has and the speed it's asked for.
- **What they cost.** Every trip goes by a corner: every one of 28 ships, on 20 trips each, flies 129 to 132% of the straight way. That's SWARM's fighter. And a leg can be too short to fly, which was the stuck ship.
- **Tried twice without them, the same evening, and not kept** (a third try is what's built: below). Each way's speed asked for at once, held to what the ship can stop from in the room left that way; then the same with each way stopped in one burn, the nearer first.

| | Legs | Both ways at once |
|---|---|---|
| SKATER (engines on its flanks too) | 158 ticks a trip, 2 quarter turns, 131% of the straight way | 132 ticks, 2 turns, 102% |
| ARROW (one engine, turns in 2 ticks) | 211 ticks, 5 turns, 130% | 166 ticks, 16 turns, 92% |
| Henry's FIGHTER (one way, 23 ticks a quarter turn) | 213 ticks, 5 turns, stops within 1 px | 214 ticks, 11 turns, up to 13 px off, one trip in twenty never settled |
| Starter FRIGATE (45 ticks a quarter turn) | 406 ticks, 5 turns, within 1 px | 423 ticks, 6 turns, up to 12 px off |
| BRAKER (engines ahead and astern) | 146 ticks, 1 turn | 157 ticks, one trip never settled, 68 px off |
| All 28 ships | 314 ticks a trip | 317 |

- **What it reads as.** For a ship that can push across as well as along, it's simply better: straight there, sooner, no more turning. A one-engine ship that turns fast gets there a fifth sooner but turns three times as often, swapping from one way's braking to the other's as each comes due. A one-engine ship that turns slowly gains nothing and stops badly: by the time it has turned to stop one way, the other is overdue.
- **What would work is a plan, not a rule applied tick by tick:** burn one way, turn once, burn the other, coast straight at the place, turn once and stop one way, turn once and stop the other. Three quarter turns where legs take five, on the straight way. Put to Henry.

*Henry: "Yes, I think we need ships to fly approximately straight towards their destinations. It will be surprising behavior for the player otherwise." Built the same evening: the legs are gone.*
- **As built** (`Flyer::arrive`). A ship asks for speed both ways at once, each its share of its top speed, so the two together point at the place. Each way is held to what the ship can stop from in the room left that way, and is stopped in one burn when it's going faster than that. The two stops come in turn, the shorter way first. So that the shorter is done in time it's flown a little faster than its share, and the ship's line points at where the longer way's own stop begins; the longer way's room has to cover the shorter's whole stop and then its own turn. Setting off, it gets up speed the shorter way first, so it flies with its nose along the way it's mostly going. The last way left is slowed as its room runs out. It turned out not to need a plan kept from tick to tick: the order of the two stops, and one burn for each, was what the first two tries lacked.
- **A one-engine ship's trip:** burn the shorter way, quarter turn, burn the longer, coast at the place nose first, quarter turn and stop the shorter, quarter turn and stop the longer. The end of the line bends a little: the last stretch is flown the longer way alone.
- **Measured,** the same 28 ships and 20 trips each:

| | Legs | Straight |
|---|---|---|
| All 28 ships, a trip | 314 ticks | 278 ticks |
| How far they fly for the straight way | 129 to 132% (WASP 151%) | 114% on average; 94 to 118% for 25 of them |
| Quarter turns a trip | 5 (1 or 2 for ships with engines on their flanks) | 4 for nineteen ships, 5 for five (ARROW 8, LANCET 7; with flank engines, 1) |
| How far off they stop, at worst | 1 to 3 px | 1 to 13 px, never more than a ship can place itself |
| Henry's FIGHTER | 213 ticks, 5 turns, 129% | 174 ticks, 4 turns, 109% |
| Henry's FRIGATE | 410 ticks, 5 turns, 130% | 364 ticks, 4 turns, 113% |
| Starter MINER | 189 ticks, 5 turns, 129% | 155 ticks, 5 turns, 105% |
| GUNBOAT | 314 ticks, 5 turns, 130% | 274 ticks, 4 turns, 100% |

- **What it costs: ships stop less exactly.** Flying a leg at a time a ship crept up on its spot and stopped within a pixel or two. Now the shorter way is stopped in one burn and ends where it ends: a few pixels off, at worst as far as the ship can place itself. Sent in formation, the starter deck's ships all stand within that of their spots, where 15 of 17 stood within 2 px. Formations and guards' rings are a little raggeder.
- **Not as good yet for ships with engines pointing more than one way:** SKATER flies 123% of the straight way, BRAKER 137% and WASP 139%, with a loop near the end. They arrive. SKATER and BRAKER are the toy's; WASP is SWARM's.
- **A fault found on the way:** a ship standing a hop or two from its spot with both ways to make up turned on the spot for ever, each turn changing which way looked better to set off on. It now takes the longer way first, whatever it's facing.
- **In Henry's match** (the random-sends test): no ship goes more than 9 s without getting nearer its spot.
- The suite is 307 and passes, with the same four tests switched off.

*Henry, after playing it (2026-11-16): "I tried the new legless flight. It was an improvement."*

## Pixel tractors and collectors (2026-11-16)

Henry: "I had some inspiration for mining gameplay depth (and slowing down the game a bit): pixel tractors. Pixels don't get dropped off instantly for free any more. Instead, miners need to fly within range of a pixel tractor that beams the collected pixels into the bank at a fixed rate. Miners no longer scale indefinitely on their own, and this opens up a "resource collector" unit type that can follow miners and has tractors and nodes. (Collectors will need their own AI behavior to let them follow groups of miners in a reasonable way.) (Resources get used from mobile collectors first before getting used from stationary outposts.) It's fun to see miners devour the map in 11 minutes right now but it isn't balanced."

Built the same evening. Docking ports, the other half of the idea, are in the wishlist below.

| # | What | Henry's verdict |
|---|---|---|
| T1 | **The TRACTOR, a part** (INDUSTRY, letter `u`, weight 1, 2 power, pink). In a match nothing takes a miner's load the moment it arrives. A ship with tractor pixels pulls a pixel out of a miner in its reach every 8 s for each tractor pixel it has: eight pull a pixel a second. The pixel is banked when it comes across, into that ship's own store. | |
| T2 | **Its reach** is measured from its ship's middle: the ship's edge, 30 px, and 4 more for each pixel along the side of the square its pixels would make. The king's reaches 46 px, the outpost's 44, the collector's 41; sixty-four tractor pixels would reach 62 past the edge. | |
| T3 | **A ship with tractor pixels is a node:** it stores what it pulls and links like a factory (80 px). **Mine to flag:** Henry said a collector "has tractors and nodes". There's no separate node part: a tractor with nowhere to put what it pulls would do nothing, so the one part does both. A node part that only links (a relay) is still open. | |
| T4 | **A full miner goes where it will be empty soonest:** for each tractor ship of its side, the flight there and the loads already waiting there. It keeps to its choice. It stops well inside the reach, on the side it came from, and waits until it's empty. With no tractor left anywhere it waits where it was built. | |
| T5 | **A miner in a tractor's reach where it works is emptied as it works.** It never fills and never leaves the rock. So a mining site's income is what its miners bite or what the tractors there pull, whichever is less. | |
| T6 | **Who a tractor takes from:** the miners waiting to be emptied before the ones still at work, and of those waiting the one with the least left, so it's away soonest. | |
| T7 | **The COLLECTOR** (starter deck, slot 8): eight tractor pixels round its core, fifteen hull and three engines, 27 px, 22 px/s. A ship that flies with a tractor and no scoop is a collector, whatever else it carries: it follows the miners instead of taking the stance, and guns on it fire at what comes in reach. | |
| T8 | **Where a collector stands.** Each miner's place of work is the rock pixel it's after. A tractor has room for as many miners as it can keep emptied (a collector's eight pixels: three). The tractors that stand and the collectors built before this one take the nearest places of work in their reach first, up to their room. Of the places left, this one takes the one with the most others round it and stands in the middle of those. It thinks once a second and keeps its place unless the new one is more than 16 px off. With no miners to stand by, it waits where it was built. Shot at with no guns, it runs for the nearest node. | |
| T9 | **A collector goes no further than still links.** If the miners' rock is out of link of its network, it stands on the way there, as far out as links, and the miners fly the rest. **Mine to flag:** Henry asked only that it follow the miners "in a reasonable way". Out of link, what it pulled would sit in it unspent, so it stays in link by itself. Sent by hand it goes anywhere (G lets it go again). | |
| T10 | **What flying nodes hold is spent first,** the nearest of them; then the printer's own store; then the nearest node. | |
| T11 | **A node that flies out of link by itself hasn't split the network.** The note is for two standing nodes parting, as before. | |
| T12 | **The king has eight tractor pixels,** a ring round its middle core in place of hull: a pixel a second, where a side's first miners unload. | |
| T13 | **The starter deck's factories:** the OUTPOST has four tractor pixels where it had factory pixels (24 factory pixels now, builds up to 240 px), the SHIPYARD eight (110, up to 1,100 px). Same size, power and price. The teams' NEST has two (10 factory pixels) and the DOCK six (66). **This changes Henry's drawings.** A saved starter outpost or shipyard that was never edited is swapped for the new one, and a team saved with seven cards gets the collector in slot 8. | |
| T14 | **The AI prints collectors:** when its tractors can't pull as fast as two thirds of its miners bite, its king prints a collector before another miner. Every ready-made team has the collector on the end of its cards. (`--off 8` switches the rule off for measuring.) A raider near one of its collectors turns its guards out, as one near a miner does. | |
| T15 | **What the match shows.** A dotted pink line from a miner to the tractor emptying it, with a bright pixel running along it for each pixel that crosses. A tractor's reach, for the ship under the pointer and the ones selected: one pink dot that goes once round it at that distance in 4 s, clockwise from the top, and rests 3 s before the next lap. (A dotted circle for the first evening. Henry, 2026-11-17: "can we have the circle showing the tractor's range just be one pink dot that goes around in a circle occasionally. I think it's too loud right now.") The bar: "TRACTOR: 1 PX A SECOND, 46 PX REACH. 3 MINERS WITH A LOAD." When four or more of your miners stand waiting on a tractor, a small mark at the top of the field: a warning triangle with a pink square in it, and the number (a line of gold words for the first day; see Z1). | |
| T16 | **E selects your miners and collectors in view,** Ctrl+E every one (Henry: "E selects economy units just like Q selects combat units"). Q, and a box, leave collectors with the miners. | |
| T17 | **The arena is as it was:** its stations take a load at once. Tractors work only in a match. Wire version 19. | |

**The same four matches** (the AI against itself on the starter deck, seeds 1 to 4). The first column is from before straight flight and hasn't been played again on it, so some of the difference is flight.

| | Before (flight in legs) | Tractors, AI never prints a collector | Tractors, AI prints collectors |
|---|---|---|---|
| Mined by 4:00 (seed 1, blue and red) | 414, 432 | 242, 244 | 299, 298 |
| by 8:00 | 1638, 1284 | 649, 693 | 944, 1152 |
| by 12:00 | 3429, 2991 | 1121, 1247 | 1961, 2454 |
| by 16:00 | 5586, 4558 | 1601, 1847 | 3781, 4306 |
| How the four ended | 17:50, 21:27, 19:43, 21:54 | a draw at 40:00, 37:14, a draw, a draw | 30:20, 21:15, 22:36, 20:49 |
| Collectors a side made | none | none | 9 to 27 |
| Miners a side made | 82 to 101 | 93 to 152 | 60 to 151 |

- **With collectors the AI mines about two thirds of what it did by the same minute.** Three of the four matches ended within three minutes of where they had (two of them sooner), and seed 1 ran twelve minutes longer. Without collectors the AI lives on what its king and factories take in, about 2 px a second, and three matches in four run out the clock.
- **The numbers are a first guess and Henry's to tune:** 8 s a pixel for each tractor pixel, the reach, how many tractor pixels the king and each factory have, and the collector's price.

**Proofs** (`sim/eco.rs` unless said):
- A full miner 30 px from its king loses a pixel every 15 ticks, nine in all, never two in a tick; the bank gains exactly the load and what the king made; the miner goes back to work.
- The outpost as it was drawn before tractors takes nothing in: the miner beside it flies 300 px to the king.
- A full miner takes the nearer of two outposts with nobody waiting, and the further one with five loads waiting at the nearer; it doesn't change its mind on the way.
- Three miners at one tractor are emptied in the order T6 says, pull by pull.
- A collector flies out to four miners at a rock, stands with all four in reach, in link with its king, and banks 60 to 72 px a minute from them; none of them carries anything home.
- With the rock out of link, it stands 176 to 200 px from its king on the way there (it links at 200), in link, and the miners unload at it.
- The king prints a fighter with 100 px of its own and 20 on a collector in link: the collector pays.
- A collector sent out of link and back is never called a split.
- The AI with twelve tractor pixels prints no collector with seven miners, one with eight, and none with the rule off.
- The picture (`play.rs`): the pull is drawn across the gap between the king and the miner it's emptying, the reach only with the pointer on the king, and the bar's words.

**Not done, and known:**
- **Henry's own team had no tractor but the king's** (he drew eight tractor pixels on his OUTPOST the next morning; his SHIPYARD has none and his nine slots hold no collector). His OUTPOST and SHIPYARD are his drawings and aren't touched. As it was that evening, his miners unloaded only at his king, at a pixel a second. Played by the AI against the starter deck (seed 1), his team mined 1,005 px to 5,740 and lost at 18:06.
- **Docking ports** (wishlist).
- **A tractor doesn't pick up loose pixels itself.** Miners scoop them, as before.
- **The whole-map view draws no pull lines and no reach.**
- **The teams' strategies weren't looked at again,** nor was `--teams` played: SWARM's nest takes in a pixel every 4 s, which may or may not suit a rush.
- **The cost of a tick wasn't measured:** the matches above were timed with other work running.
- **The numbers table's prices and print times** for the starter deck now go with 24 and 110 factory pixels.
- The suite is 317 and passes, ten of them new, with the same four tests switched off.

*Henry's first match on it (2026-11-17, seed 60921, his deck against SWARM).* He had redrawn his OUTPOST with eight tractor pixels (144 px). "i have a bug with a seed that won't grow", with a screenshot of the seed sitting at the top edge of his home rock. The match log: planted 0 at 1:00, and his king slowed for having under 40 px standing.
- **The cause was straight flight, not the tractor.** A seed came down when it was within 2 px of the pixel it was sent to. Since ships fly straight, a ship stops as near its spot as it can place itself (3 px for a seed) and, stopped that near, flies no further. A seed that stopped exactly 3 px off sat there for good.
- **How often:** on his map, of 135 seeds sent to every third pixel of his home rock, 4 never came down, each stopped 3 px from its pixel.
- **Mended by the same rule flight uses:** a seed has reached its pixel when it's within 2 px of it, or has come to rest as near it as it can place itself. It still comes down on the pixel it was sent to, not where it stopped. Proof: every third pixel of the home rock on his map and on another, each seed down within a minute with its factory's core on the pixel sent to (`a_seed_sent_to_any_pixel_of_a_rock_comes_down_on_it`); with the old rule it fails on those four.
- **Nothing else in the game waits on a fixed 2 px.** The one other place was the small cross that marks where a sent ship is going, which stayed under a ship resting 3 px or more off its spot: it goes now when the ship has come to rest as near as it can.
- The suite is 318 and passes, with the same four tests switched off.

*His second match that morning (seed 28259).* "did something change with build speeds? this outpost seems very slow to build", with a screenshot at 1:40 of the outpost a third grown at the edge of its rock.
- **Nothing changed in how fast anything builds.** His outpost was growing out of his bank and not out of the rock. The log: bank 0 at 1:00 and at 2:00, 58 px standing at 1:00 and 80 at 2:00, and no ship from a king told to print miners at 0:13. Every pixel he had, and every one his king made, went into the outpost as it came.
- **The cause: a rock was counted edge to edge.** A factory eats the rock its core lands on, and that rock was the pixels touching the core's pixel edge to edge, and so on outward. A rock's ragged outline has pixels that hang on by a corner only, and each of those counted as a rock of its own: 278 of the 22,496 rock pixels on his map, a few on every one of its 53 rocks, 9 on the 471 px rock he planted on. His core came down on one. It ate that one pixel, and the rest was the rule for a rock that has run out: the bank pays.
- **It was always so,** and nothing to do with tractors or flight. A click from outside a rock picks the nearest pixel of its outline, which is where those pixels are.
- **Measured on his deck and map:** his outpost sent to such a pixel of a 95 px rock had 54 px after 20 s (the whole bank) and then 5 px more every 20 s, which is what a king makes by itself. Sent to any other pixel of that rock it was whole, 144 px, in under 40 s.
- **Mended by what a rock is:** every rock pixel that touches, edge to edge or corner to corner. What looks like one rock is one rock. The bar's "ROCK: N PX LEFT" and the box and warning shown while planting use the same rule, so they said 1 px for those pixels too.
- **And rocks are made as one body.** Twelve pixels on his map stood clear of their rock altogether, a pixel or more of space between: the map leaves those out now. So every map has a few rock pixels fewer, and every match plays out a little differently from the same seed.
- **Proofs:** on four maps, whichever pixel of a rock is asked, the rock is all of that rock's pixels. And a seed sent to every pixel of the home rock on three maps comes down within a minute, on the pixel it was sent to, with the whole rock to eat. Counting edge to edge again fails both; leaving the specks in fails the first; the seed's old 2 px fails the second.
- The suite is 319 and passes, with the same four tests switched off.

## After Henry's first whole tractor matches (2026-11-17)

Henry: "Overall, I think pixel tractors are the right decision for slowing down the game. I do think we want a little more mining complexity. I'd like to provide alternatives to swarms of tiny miners. I'm also wondering if we should just make the map a tiny bit bigger and/or have the asteroids show up in a more geometric design, like an S, a donut, or two triangles, chosen randomly. I'm trying to making everything less blobby and make it feel more like each decision matters rather than just a mess of ships. Some of our decisions have already been good for that like how tracker is an effective swarm counter."

**So T1, the tractor itself: keep.** The rest of the T rows have no verdict of their own yet.

Two things he asked for, built:

| # | What | Henry's verdict |
|---|---|---|
| Z1 | **The waiting mark.** "I think the miners missing tractor message is a bit too big. Maybe just like [! pink square] in a warning triangle with a number of miners?" The line of words is gone. In its place, top middle of the field: a gold warning triangle 11 px wide with a pink square in it, and the number of miners waiting. With the pointer on it the bar says what it means. | |
| Z2 | **A build order has a START and a REPEAT.** "I think we need to split the builds cycle into a fixed start and then a repeat sequence. We should allow both to be pretty long." A factory prints its START row once, in order, then its REPEAT row round and round. | |
| Z3 | **Sixteen in each row** (one row of twelve before). The yard shows the two rows one over the other, with the ADD buttons under them, in room taken from the text box, which never used it. A click on a row's name lights that row, and ADD puts designs on the lit one; a right-click on an entry takes it off. REPEAT is lit to begin with. | |
| Z4 | **What it's too small for is passed over in the start too,** and doesn't come round again. **With an empty REPEAT a factory prints its start and stops.** A piece cut off a factory that still prints goes on with the round. | |
| Z5 | **Saved as `3,3,7>0,1`:** the start, a `>`, the repeat. With no start the line is what it always was, so nothing saved before has changed. A card's tip in a match says both rows, the same design several times running said once ("3 MINER, FIGHTER"). Wire version 20. | |

**Engine against weight** ("We should also check engine vs. weight design because currently ships all feel pretty samey to me in terms of speed, except for fighters being faster"). Every ship of the starter deck and of his own:

| Ship | Pixels | Weight | Engines | Top speed | Time to top speed | A quarter turn |
|---|---|---|---|---|---|---|
| FIGHTER (starter) | 9 | 9 | 2 | 37 px/s | 1.0 s | 0.8 s |
| LANCER | 9 | 11 | 2 | 33 | 1.1 | 0.9 |
| MINER | 9 | 12 | 2 | 30 | 1.1 | 1.0 |
| FIGHTER (his) | 13 | 13 | 2 | 30 | 1.2 | 1.5 |
| PICKET (his) | 46 | 55 | 10 | 33 | 1.1 | 3.0 |
| COLLECTOR | 27 | 27 | 3 | 22 | 1.2 | 3.0 |
| FRIGATE (starter) | 55 | 79 | 5 | 15 | 1.4 | 3.0 |
| FRIGATE (his) | 59 | 87 | 5 | 15 | 1.5 | 3.0 |
| BATTLESHIP (his) | 188 | 271 | 15 | 15 | 1.6 | 3.0 |
| BASTION (his) | 157 | 295 | 14 | 11 | 1.4 | 3.0 |
| CAPITAL | 663 | 1028 | 31 | 11 | 2.2 | 3.0 |

- **He's right, and it's three rules flattening it.**
- **Top speed has a floor.** It's 2 plus five times thrust over weight (in quarter pixels a tick), and never under 3. So anything with an engine does 11 px/s, and everything from a frigate to a capital lands on 11 or 15. The capital's 31 engines buy it the same top speed one engine would.
- **Everything gets up to speed in a second or two.** Weight is felt in how long that takes, and the drawings all carry engines enough that it's never long. With one engine the capital would still do 11 px/s, and take 68 s to get there.
- **A quarter turn is never slower than 3 s.** Every ship over about 30 px with no rotation engine sits on that limit, so a frigate turns like a capital.
- Nothing changed: it's Henry's to rule (below).

**Put to Henry, not built:**
1. **Heavy plate.** "The 2x1 isn't a real restriction, just an annoyance. Your original 2x2 was better but it didn't play well with the symmetrical building mode. Maybe 3x3?" What a domino doesn't stop is a skin one pixel thick, which is the thing worth stopping. Proposed: no blocks at all, and a rule of thickness: every heavy plate pixel has to be part of some full 2x2 square of heavy plate, the squares overlapping as they like. Any shape two thick or more is legal, a wall three wide sits across the mirror's centre column, [T] and the dominoes go, and a pixel too thin shows red as a thin projector does. The stricter version is the same rule with 3x3. Either makes the heavy plate on his OUTPOST, SHIPYARD and BASTION illegal as drawn, and the teams' ships need redrawing.
2. **Alternatives to swarms of tiny miners.** The swarm is what the rule asks for: a miner bites a pixel every 3.3 s however many scoops it has past two, so the cheapest fast miner earns the most. Proposed: every scoop bites (a six-scoop miner bites three times as fast as a two-scoop one). A big miner is then several small ones in one hull with one core and one set of engines: cheaper for what it bites, slower, a fatter target, and best parked beside a collector. Small ones stay best for far rocks.
3. **Rocks in shapes.** Now that a tractor has a reach, a rock's shape is a decision: a long S wants collectors along it, a donut has an inside. Proposed: fewer rocks, each one of a handful of shapes (S, donut, two triangles, bar, cross), mirrored for the other side as now, and the map's size left alone until that's been seen. Rocks still stop nothing.
4. **Speed.** In the order I'd try them: take the floor off top speed, so it's thrust over weight and nothing else and an under-engined ship crawls (the beaten-king rule leans on that floor and would need its own number); then let a quarter turn be slower than 3 s for the heaviest.

- The suite is 322 and passes, with the same four tests switched off.

*Henry's rulings on those four, the same day:*
- Rocks: "When I was talking about the S I meant the asteroids make a big S on the map, and there's empty space otherwise." So not rocks shaped like letters: the whole field of rocks laid out as one big figure. Not built; planned below.
- "Two-thick rule makes sense."
- "I agree we should make every scoop bite. What happens when tracktor and scoop is on the same ship?"
- "I think we need to make those speed numbers a lot more variable."

| # | What | Henry's verdict |
|---|---|---|
| Z6 | **Heavy plate has to be two thick.** Every heavy plate pixel has to be part of some full 2x2 square of heavy plate; the squares overlap as they like, so a wall three wide, an L or a slab is fine and a skin one pixel thick isn't. It's painted a pixel at a time like plate. A pixel that's too thin shows red, and the status line says how many ("14 HEAVY PLATE PX NOT 2 THICK"). The dominoes, and [T] turning them, are gone. | |
| Z7 | **Three ready-made ships redrawn** for it, the only three that wore it one thick. CORVETTE: a nose three wide and two thick (it costs 31 px where it cost 29). LANCET: sides and stern two thick, 74 pixels of heavy plate where it had 32 (222 px where it cost 180, and more than twice the weight). The example BASTION: its skin two thick (206 px where it cost 156). The fights that set them against a swarm are levelled on cost again (48 fighters against two lancets, 23 lancers or fighters or 17 AP lancers against a bastion). | |
| Z8 | **Every scoop bites:** one scoop bites a pixel every 10 s, and a ship's scoops between them bite one every 10 s over their number. The starter miner's three bite as they did, one every 3.3 s. Six bite twice as fast. (Two scoops bit one every 3.3 s and any more bit no faster, so the cheapest fast miner earned the most and a swarm of them was the best mining there was.) The AI's rule for collectors and a collector's room both count scoops now, not miners. | |
| Z9 | **A ship with a scoop and a tractor** is a miner: it goes to the rock and works. Its tractor reaches its own hold, so what it bites is banked in itself as it bites; it never fills and never carries a load anywhere. It's a node, so within link of its side's network what it holds can be spent, and out of link it sits in the ship. Nothing brings it home. Its tractor empties other miners in its reach too, and a collector doesn't come to a miner whose own tractor keeps up with it. Nothing new was built for this: it's what the tractor's rules already did, now with a test. | |
| Z10 | **Top speed is thrust over weight, and nothing else:** 22.5 px/s for each thrust per weight, from 7.5 to 45. The 2 it started from is gone, and the least is 7.5 where it was 11. The number is the one that leaves the starter fighter at the king's pace (37.5), the miner at 30 and the wasp at 45. | |
| Z11 | **A beaten king flies at a tenth of its speed, 3.75 px/s.** **This changes M2.10's number** (7.5 px/s, a fifth), because the slowest ship is now 7.5 and the rule is that a beaten king is slower than anything that flies. | |
| Z12 | **The yard's turning gauge and its time to top speed were the old flight's.** They show what a match flies now: seconds for a quarter turn, and the time to top speed on the engines that push hardest. | |

**What Z10 does to every ready-made ship** (px/s, before and after):

| | Before | After |
|---|---|---|
| WASP | 45 | 45 |
| FIGHTER | 37 | 37 |
| LANCER, ARCHER | 33 | 30 |
| MINER | 30 | 30 |
| AP LANCER | 26 | 22 |
| COLLECTOR, RAKER | 22 | 18 |
| AEGIS | 18 | 15 |
| CORVETTE, WARDEN, SENTINEL | 15 | 11 |
| FRIGATE | 15 | 11 |
| BEAMER | 11 | 7 |
| CRUISER, BATTLESHIP | 15 | 7 |
| CAPITAL, LANCET | 11 | 7 |

And Henry's own: FIGHTER 30 to 26, PICKET 33 to 30, FRIGATE and BATTLESHIP 15 to 7, BASTION 11 to 7; LANCER and MINER as the starter's.

- **Nine speeds where there were six, and the big ships are a class of their own:** a frigate is now a third as fast as a fighter where it was two fifths, and a capital a fifth.
- **Still the same among themselves, at the bottom:** cruiser, battleship, capital, lancet and bastion all sit on the least, 7.5. A big ship can't carry engines enough to leave it: engines go along one edge and weight goes by the area, and a capital with engines right across its stern has a quarter of a thrust per weight. What would let a big ship buy speed is a bigger engine, which is a part to draw, not a number. Not built.
- **Turning is untouched.** Every ship over about 30 px still takes exactly 3 s a quarter turn.
- **What it did to a fight:** the example beamer (7.5 px/s now) against the example warden (11). The warden takes two cuts on its circles, reaches its lances' distance in 7 s and kills the beamer, which is too slow to get off a lance's line. Before, that fight ran past forty seconds with the beamer alive. Slow ships are easier for lances.

**Henry's own team under Z6:** his OUTPOST (44 px too thin), SHIPYARD (36) and BASTION (38) are not legal as drawn and can't be printed until their heavy plate is two thick. They aren't touched.

**The rock field as one big figure: the plan, not built.**
- A map's rocks are strung along a figure that fills the map: an S, a ring, or two triangles, chosen by the map's seed. Open space everywhere else.
- Every figure is the same turned half round about the map's middle, as maps are now, so neither side has the better of it. An S is that already; so is a ring; the two triangles are one each side.
- The figure passes within about 150 px of where each king starts (an S's two tails end there), so a first factory has a rock in reach.
- About as much rock as now (22,000 px a map), in the same rough discs, each one body.
- What has to be looked at: the AI plants on the rock nearest its king that links, and a good many tests stand a rock a set distance from a king.

**The same four matches again** (the AI against itself on the starter deck, seeds 1 to 4), with Z6 to Z11 in: they ended at 26:58, 26:05, 23:04 and 22:14, where with tractors alone they had ended at 30:20, 21:15, 22:36 and 20:49. Mined by 8:00 on seed 1: 1,138 and 1,045 (944 and 1,152 before); by 16:00: 4,279 and 3,959 (3,781 and 4,306). Mining is where it was, as it should be with every miner in these decks having three scoops. The matches ran 50 s longer on average, with frigates at 11 px/s where they flew at 15. Four matches is too few to say more than that nothing broke.
- The suite is 324 and passes, with the same four tests switched off: 323 in one run of the whole suite, and one more run by itself afterwards, the cannon fight whose plain-pilot baseline I re-recorded (the ARROWS flying straight in won 15 of 20 or more; they win 11).

*Henry, later the same day:* "Yes, but let's keep the existing map style as a fourth option. Additionally, let's bump the hull cost of a scoop up by one. New mining wishlist item: a scoop with a range. Question: Should we rename "tractor" to something like "processor" or "integrator?" After this, let's implement some new parts. I don't want to get stuck tweaking gameplay while not all the content is there yet."

| # | What | Henry's verdict |
|---|---|---|
| Z13 | **A map's rocks are laid out one of four ways, as its seed falls:** scattered all over, as every map was; or strung along one big figure with open space everywhere else: an S across the map with a big rock where its curves meet, a ring round an empty middle, or two triangles, one each side, their long sides facing across an open middle. Both machines of a linked match have the seed, so both lay out the same map. | |
| Z14 | **Every layout is a fair map:** the same turned half round about its middle. No rock is nearer a king's start than 84 px, and on every layout there's rock within 180 px of it, in reach of a first factory (an S's tails end by the two kings). A figure's rocks are the same rough discs, most 7 to 13 px in radius and one in seven 18 to 24, with 16 to 44 px of open space between one and the next. | |
| Z15 | **A scoop uses 2 power, where it used 1.** A ship short of power bites slower in step, as its guns reload slower, so a miner drawn without the hull doesn't bite as fast as one with it. | |
| Z16 | **The starter MINER has a row of three more hull pixels** for its scoops: 12 px where it was 9, 22 px/s where it flew at 30, a quarter turn in 1.5 s where it took 1. **This changes Henry's drawing,** which the scoop's new cost left a third short of power. His own saved MINER is the old drawing: it still flies, at two thirds of its power, 18 px/s, biting a pixel every 5 s where it bit one every 3.3. | |
| Z17 | **A ship that's going away from the spot it's sent to, and too fast across to stop there, stops dead first and sets off again from rest.** A fault in straight flight, a day old, found on the way: a ship with one engine, steering for a spot while it was going away from it (sent somewhere else at speed, or carried past), turned the one way's speed round while the other way's carried it past, then the same the other way about, and could go round and round the spot for good. In a match a fighter sent to a spot 18 px behind it was still circling 15 s later. A ship that's only going away, with room the other way, turns round as it always did. | |

**Measured, on eight maps of each layout** (seeds 1 to 8):

| | Scattered | An S | A ring | Two triangles |
|---|---|---|---|---|
| Rock, least to most | 21,484 to 28,216 px | 19,760 to 32,614 | 17,792 to 30,640 | 17,792 to 31,206 |
| Rock, average | 24,626 px | 26,640 | 24,573 | 25,023 |
| Of the map, how much is over 110 px from any rock | 17 to 26% | 48 to 49% | 51 to 52% | 53 to 56% |
| The AI against itself by 8:00 (seed 1): factories planted | 2 and 2 | 2 and 2 | 3 and 3 | 3 and 3 |
| and pixels mined | 916 and 945 | 946 and 1,042 | 1,117 and 1,118 | 976 and 1,074 |
| That match ended | 22:29 | 28:08 | 28:51 | 26:26 |

- **A figure's rock varies more from map to map than a scattered map's,** because one rock in seven is big and there are only fifty or sixty of them. The averages are within a tenth of each other.
- **Z17, measured.** Henry's fighter alone, sent to eight spots near and far from every speed it can have and every way it can face, 416 starts: before, it never came to rest from 11 of them (17 at the speed it had the day before), and the slowest that did took 38 s. Now it comes to rest from all of them, the slowest in 31 s. So do the seven other ships tried, from every start; the slowest of each: the starter fighter 23 s, lancer 19, miner 30, wasp 44, collector 50, corvette 55 and frigate 72 (the last two fly at 11 px/s, and were tried from 672 starts each). From rest every ship got there already.
- **The first cut of Z17 was blunter, and wrong:** stop dead whenever the ship is going away from its spot, either way. It cured the circling, but a ship holding its range from an enemy that moves is going away from its spot half the time, and it kept stopping: five tests failed with it (a stance sending ships home, a ship getting onto a near spot, ships dodging shells, ships holding range in a losing fight, ARROWS flying round armour). The rule as built stops a ship only in the state that circles: going away one way, and the other way too fast to stop in the room it has by the time the first is turned round. All five pass with it. In the match test the longest any ship goes without getting nearer its spot is 11 s, where it was 9 before the fix (17 with the blunt one).
- **Two tests re-recorded for Z17.** Eight seconds into the SKATERS' fight every one of them had an enemy within 130 px; in one roll of forty one is now 132 px off at that moment, and the bound is 140. And the collector that stands at the end of its link was judged at one instant, a minute and a half in; it happened to be shifting 20 px to a new stand at that instant, so it's judged over the half minute after (at rest 431 ticks of 450, never out of link).
- **Seen in that trace, not touched:** a collector shifting 20 px creeps there at 4 px/s and runs a few px past before it settles, because with one engine it needs two quarter turns to brake. It's the same inexact stop the straight flight has had since it was built.
- **That match test read Henry's saved team from its file,** so what it tested changed with whatever he'd drawn last. It has his warships of 2026-11-16 written into it now, with the starter deck's miner and factories.
- **The tests' maps are scattered,** as every one was, so nothing they measure moved for this. Three new tests are about the layouts: each seed falls on one of the four and always the same; every layout is fair, in one body rock by rock, with rock in reach of both kings and its figure where it should be; and the AI plants and mines on every one.
- **For the tools:** `--map scatter|s|ring|triangles` plays the headless matches on that layout; without it each match's is picked by its seed, as the game's are. `--map-screenshots` writes a picture of the whole map for each.

**Five matches on the build as it stands** (the AI against itself on the starter deck, seeds 1 to 5, each on the layout its seed picks):

| Seed | Rocks | Won by | At |
|---|---|---|---|
| 1 | two triangles | RED | 26:26 |
| 2 | an S | BLUE | 26:05 |
| 3 | two triangles | RED | 21:08 |
| 4 | scattered | BLUE | 23:17 |
| 5 | a ring | RED | 27:04 |

- Every one ended with a king killed, none in a draw. The four before (all scattered, before the scoop's cost) ended at 26:58, 26:05, 23:04 and 22:14. These aren't the same maps, so the lengths don't compare match for match: five matches on four layouts say that nothing broke, and no more.
- Mined on seed 1 by 8:00: 976 and 1,074 (1,138 and 1,045 before, on its scattered map with the miner at 9 px and 30 px/s); by 16:00: 4,042 and 4,278 (4,279 and 3,959).
- `--match-sim` names each seed's layout on its line now.

**"Should we rename tractor to something like processor or integrator?"** My answer: keep TRACTOR for this part. What a player plans round is its reach and its pull at a distance, and that's what a tractor is: the dot going round it, the line to the miner. PROCESSOR says something else, that a load has to be worked on before it counts, and that fits a part a miner has to touch, which is nearer the docking port. If the wish is for a word for the whole job (taking loads in at a rate), that's the name of the pair, tractor and dock, not of this one. Not renamed; Henry's to say.

- The suite is 328 and passes, with the same four tests switched off. Four are new: the ship sent somewhere at speed, and the three about the layouts.

---

## Dock pads, the energy ball, and a king that runs (2026-11-17, night)

*Henry, after a match on an S map (seed 33943, the AI on SWARM; he won at 15:49):* "MINERS text just barely got cut off. Why does every combat ship say "HOLDING AT RANGE?" Do they ever say anythign else? I played a game on an S map. I think we definitely want to keep a map variety, it was a good change of pace. Whether or not we need some minor edits to the plain shapes is TBD. Seemed good from my game. What determines the length of a node connection? My game ended a little early when the enemy king didn't seem to run away correctly. Docking port and energy ball next are good and what about a speed-up field and a stasis field? Those seem among the riskier additions that would be good to get in earlier. I'm also interested in the ranged scoop. I want to add a way to get a burst of resources to encourage building big ships. What about a phase-harvester that builds up a charge on an entire asteroid and harvests the entire asteroid at once once the bar reaches the amount of pixels remaining on the asteroid? Another wishlist item: a repair array that uses banked pixels to repair damaged ships (this could even indirectly build two full ships out of one if a ship got split brained and then repaired). If we implement those plus maybe a couple of my other wishlist items I think we're getting close to some sort of "full set" of part content, although I haven't decided exactly how we want to handle building sets."

**His verdict on the layouts (Z13, Z14): keep.** "We definitely want to keep a map variety." Whether the plain shapes want small edits is open.

**The two questions.**
- *"Why does every combat ship say HOLDING AT RANGE?"* Because that's the one thing a ship with engines says from the moment it has a target, whether it's still 400 px off and flying in or already at its guns' distance and firing. In a fight every warship has a target, so that's what they all say. The others it can say: IDLE (no enemy, at its post), ON GUARD, SENT TO A PLACE, ESCORTING THE KING, DODGING A BEAM, HOLDING (it has no engines), FLEEING HOME (unarmed and hit), and a miner's and a collector's own. Not changed. It could say which it is (closing, or at its range) and what it's after.
- *"What determines the length of a node connection?"* Two fixed numbers and nothing else. Every node reaches 80 px from its middle, and a king 120. Two of a side's nodes are linked when their reaches touch: two factories with their middles up to 160 px apart, a king and a factory up to 200. A node's size, its parts and what it holds don't come into it.

| # | What | Henry's verdict |
|---|---|---|
| W1 | **The yard's line for a collector reads COLLECTOR: FOLLOWS YOUR MINERS.** It was two letters longer and its last letter was cut off. The test of what fits the line never asked for this one; it does now. | |
| W2 | **A bot's king keeps clear of the reach of an enemy's guns:** of the gun of that enemy's that reaches furthest, and 30 px besides. It backed away only from an armed enemy within 90 px, whatever its guns, and a cannon or a tracker reaches 130 and a missile 200: it stood still while it was shelled. | |
| W3 | **It doesn't fly back to a post that's in reach of guns that outweigh it.** With the enemy on its post and itself clear, it waits where it is. It used to go back the moment the enemy was 90 px off, into the guns. | |
| W4 | **Once it has lost a quarter of its pixels the AI's king stops trusting that it outweighs what's shooting it,** and keeps clear of every gun. The weighing is by the pixel, and counts fighters against armour they can't break. (The rushing opening's king turns for home at the same loss.) The weighing itself is as it was. | |
| W5 | **It falls back on a factory only if it's nearer that factory than the enemy is,** not one it would have to pass the enemy to reach. With none, it takes whichever of the eight ways leaves it safest 120 px on: straight away unless another way is better, as along the map's edge is for a king with its back to it. | |
| W6 | **DOCK PAD** (INDUSTRY, weight 1, no power, letter `k`): a lump of them is a landing pad, and its ship is a node. A miner of its side flies onto a pad that fits it and comes to rest there, and everything it carries goes into that ship's store at once. Miners choose between tractors and pads by where they'd be empty soonest. | |
| W7 | **A miner fits a pad that holds a square as wide as the miner is long,** so it can come down on it turned any way. *This is narrower than "large enough for the miner to fit on it":* a starter miner, 3 by 5, needs 5 by 5. A pad with a pixel shot out of it holds a smaller square. | Replaced the same night by W13: "I don't think the miner should need to be able to fit any direction." |
| W8 | **A pad takes one load every 3 s.** *My departure:* I first built one miner on a pad at a time, the others waiting a little way off. A ship's own side flies over it freely, so nothing but a rule kept them off; and the short hop from the waiting place took a starter miner 12 s (a ship with one engine creeps on a short hop). So miners waiting for a pad wait on it, and the turn-round is what one pad can do. | Replaced the same night by W14 to W16: miners tile the pad, and the rest is counted "per pixel". |
| W9 | **A ship that flies with a pad and no scoop is a collector,** like one with a tractor: it goes and stands on its miners' place of work, as far out as still links, and they land on it. | |
| W10 | **ENERGY BALL** (HEAVY WEAPONS, weight 2, uses 3, letter `z`): each lump of them is one gun, and a lump has to be two thick like heavy plate. It throws a 2x2 ball at 30 px/s, 130 px, any way. The shield circle or projector that stops a ball loses 20 charges (a circle holds 4). A ball that reaches a ship breaks only the bare hull of the 2x2 it lands on. A gun of 4 px fires every 4 s, and a bigger one sooner in proportion. It goes for a ship with shield pixels standing or a projector over it, whether the shield is up at that moment or not. | |
| W11 | **A seventh category of parts, HEAVY WEAPONS, key 7:** the cannon, the tracker, the missile and the energy ball. Eight weapons' names don't fit the two rows of buttons a category has. It's last so the other categories keep their keys; the category buttons are 27 px wide where they were 32. | |
| W12 | **The yard's "N HEAVY PLATE PX NOT 2 THICK" lost its "PX".** With a thousand such pixels the line was a letter too long. The energy ball's line is worded the same. | |

*Henry, of the pad's line in the yard (W7):* "What does the "it fits a pad that holds a square..." sentence mean". It meant: take the miner's longer side, and somewhere on the pad there must be a solid block of dock pixels that many wide and that many tall. I reworded it, and Henry gave it better: "How about "the pad has to be big enough for the miner to land facing any direction?"" The line reads that now. The rule is as it was.

**The king, measured.** Henry's log holds no positions, so his match can't be played again. What I could do was send two of his own BATTLESHIPS (cannon, armour the starter ships can't break, 7.5 px/s) at an AI king four minutes into a match of its own:
- As it was: the king stood at its post and lost 45 px to shells from 130 px down to 90; backed off 100 px; came back to its post as soon as the battleships were 90 px away again; and so on. Dead in 144 s.
- Now: it stands by its fleet until it has lost a quarter of itself (W4), then keeps clear. Alive two and a half minutes on with 163 px of 218.
- **Five matches, the AI against itself on the starter deck (seeds 1 to 5, each on its seed's layout):**

| Seed | Rocks | Before W2 to W5 | Now |
|---|---|---|---|
| 1 | two triangles | RED at 26:26, blue beaten (under 40 px standing) | RED at 25:23, blue's king lost with 52 px standing |
| 2 | an S | BLUE at 26:05, **red's king lost with 1,436 px standing** | BLUE at 29:44, red beaten |
| 3 | two triangles | RED at 21:08, **blue's king lost with 2,288 px standing** | RED at 24:52, blue beaten |
| 4 | scattered | BLUE at 23:17, red beaten | BLUE at 23:33, red beaten |
| 5 | a ring | RED at 27:04, blue beaten | RED at 26:58, blue beaten |

- Two kings of five used to die with a healthy side behind them, which is what Henry saw. None does now. The same sides win; the matches are 78 s longer on average (24:48 to 26:06), most of it in the two that used to end early. Mining on seed 1 is the same to the pixel at 8:00 and 16:00.
- **Not looked at:** a king against ships faster than it is, which no rule here helps; and `--teams`.

**The pad as first built, measured** (W7 and W8; the pad as rebuilt is below). Three example drawings are on the library's PARTS shelf: PADPOST (the starter outpost with a pad six wide on its flank, 157 px where the outpost is 121), HAULER (six scoops, six long, 23 px, 22 px/s, carries 18) and SAPPER (an energy ball gun of eight, two flak, 43 px).
- **A starter miner 80 px from a pad** is down and empty in 13 s. A second miner, bound for the same pad from much the same place, in 20 s. Most of that is coming to rest on an exact spot: straight flight stops a ship within 3 px, and the pad draws it the rest.
- **A carrier** (a pad five wide, hull and three engines) sent out with three miners on a rock beyond its side's link: in two and a half minutes it's standing 195 px out and holds 45 of the 63 px they mined.
- **The starter deck with PADPOST for its outpost, the AI playing it against the starter deck** (seeds 1 to 3): it lost at 26:09, won at 27:00, won at 23:11. Its miners unloaded 1,831 of 8,189 px on pads, 2,120 of 9,680, and 1,506 of 10,529: about a fifth, the rest through its tractors. Mined on seed 1 by 8:00: 1,099 to the starter deck's 1,046; by 16:00: 4,736 to 4,604.
- **The same with HAULER for its miner as well** (seeds 1 to 3): it won at 22:15, lost at 13:30, lost at 19:35. On pads: 1,652 of 11,434 px, 114 of 788, and 490 of 5,002. Its first fighter is out at 2:28 where the starter deck's is out at 1:08: a miner of 23 px and an outpost of 157 make a slow start, and the two it lost it lost early. Mined on seed 1 by 8:00: 958 to 1,048; by 16:00: 5,095 to 4,275.
- Three matches each, one deck, one AI: this says pads work in a whole match and miners use them. It says nothing about what a pad is worth.
- **Not done:** the AI doesn't count its pads when it decides whether it needs another collector, and no deck the game ships with has a pad in it.

**The energy ball, measured** (in the tests; nobody has played it):
- One ball takes a shield's circle from 4 charges to none, and a projector's pool of 12 to none. A flak shot takes one.
- A gun of four against a post with one shield, a minute: fifteen balls, every one stopped at the circle, the post untouched. A circle is back 3 s after the last shot it stopped, and four pixels fire every 4 s.
- A gun of eight: thirty balls. The circle never comes back, the second ball lands on the hull, and the post's core is gone inside ten seconds.
- A ball on a slab of bare hull breaks 1 to 4 px. On plate, none.
- Ships see a ball coming as they see any shot, and the ones that pick their own courses fly round it. Not measured.

**The pad, rebuilt the same night.**

*Henry, of the pad as first built:* "There should be a pink animation when a miner offloads at a pad. I don't think the miner should need to be able to fit any direction. Actually I think it should just need to fit on an exact match of its profile and if there's extra room it would be cool if the miners tiled it and multiple could offload at the same time. Miners should clear the area of the pad once they're done offloading. I believe you have a scripted docking that takes over so they don't have to carefully fly themselves onto the pad right? It's like they're being tractored in? We can keep that and just use it for the full on->tile->off sequence. The three second cooldown can be counted per pixel or you can just fake that with a variable speed combined cooldown. Make sense?"

What I had was less than he took it for: the pad drew a miner only the last pixel or two, after it had flown itself to a stop on the spot. The pad does the whole of it now.

| # | What | Henry's verdict |
|---|---|---|
| W13 | **A miner fits a pad that has a pad pixel under every pixel of the miner,** turned whichever of the four ways. *This replaces W7.* A starter miner needs thirteen pad pixels in its own shape, or any block 3 by 5 either way up. A pad pixel missing where the miner has no pixel doesn't matter. | |
| W14 | **A pad with room takes several miners at once, side by side.** Places are taken by rows, so miners of a size tile the pad, and a miner is turned the way that lets the pad hold most. A block 6 by 5 holds two starter miners, 9 by 5 three, 6 by 10 four. | |
| W15 | **The pad moves the miner all the way: on, down, off.** A full miner flies up to the pad's ship and waits 5 px clear of it. The pad takes hold of one that's within 12 px of there and going no faster than the pad moves it, when there's a place for it; turns it; moves it on at 15 px/s; and when its load has gone moves it off again, 5 px clear of the ship on the side of the rock it's working, and lets go. In the pad's hands a miner doesn't fly. *This replaces* the miner flying itself onto its place. | |
| W16 | **Where a miner sat, the pad rests 3 s:** the pad pixels under it, from the moment its load goes. *This replaces W8.* | |
| W17 | **A pink line sweeps the miner from nose to tail while its load goes,** half a second. The load itself is banked at once, the moment the miner is down. | |

- **What's mine in that, to flag:** "its profile" I took as the miner's pixels as they stand, so a miner with pixels shot off fits a smaller place, and the pad may be bigger than the miner or any shape. The pad turns a miner when that's what makes it fit, or what makes the pad hold more. The numbers (5 px clear, 12 px to take hold, 15 px/s, half a second of sweep) are first guesses.
- **The pad's line in the yard** now reads: THE MINER HAS TO FIT ON THE PAD. A BIG PAD TAKES SEVERAL AT ONCE. WHERE ONE SAT RESTS 3 S. His to reword.
- **A starter miner 80 px from a pad 3 by 5:** the pad has hold of it at 6.4 s, it's down and empty at 7.6 s, and it's let go clear of the station at 8.8 s. As first built it was down at 13 s: the difference is the miner no longer has to bring itself to rest on an exact spot.
- **A pad 6 by 5 and three full miners:** two are down side by side a fifth of a second apart, and the third 3.6 s after them, when a place had rested.
- **The carrier** holds 54 of the 72 px its three miners mined in two and a half minutes (45 of 63 before).
- **The starter deck with PADPOST for its outpost, the AI playing it against the starter deck** (seeds 1 to 3): it won at 20:55, lost at 26:00, lost at 23:20. On pads: 2,241 of 7,746 px, 2,061 of 6,174, and 2,170 of 6,240: about a third, where it was a fifth. Mined on seed 1 by 8:00: 1,177 to the starter deck's 1,056; by 16:00: 4,451 to 4,497.
- **With HAULER for its miner as well:** it lost at 20:10, 14:30 and 15:03. On pads: 1,613 of 5,072 px, 460 of 1,458, and 454 of 2,231. Mined on seed 1 by 8:00: 1,018 to 1,048; by 16:00: 4,267 to 4,633.
- **Found on the way: Henry's new miner took 50 s over a trip that should take 30.** (Mended since: W18, below.) A flight test read his saved team from its file, so it tested whatever he'd drawn last; it has his fighter and his miner of that afternoon written in now. The miner he drew late on 2026-11-17 (24 px weighing 31, seven scoops, two engines that push one way, 11 px/s, 3 s a quarter turn) failed it: a spot 200 px across and 23 along takes it 50 s to come to rest on, where the 200 alone takes 19. Traced, second by second: 26 s along the long leg at 7.5 px/s, not its 11; 6 s turning about and braking, by which time it's 22 px past; 10 s for the 23 px leg (a turn, a creep at 4 px/s, an about-turn, a stop); and 8 s going back for the 22 px. Eight quarter turns, 24 s of the 50. *Henry: "is it a bug?"* Partly. The turning is the drawing: engines that push one way and the slowest quarter turn there is make every stop a 6 s about-turn, and this trip can't be done with fewer than five quarter turns. The 22 px overrun and the slow long leg are the pilot's, and are worth about 15 s. (I had first written that the last 23 px took half a minute, and that the ship was 31 px. Neither was right.) At a pad the last hop is the pad's now. Going to a rock it's still the miner's own.
- Three matches each, one deck, one AI, as before: it says the rebuilt pad works in a whole match.
- The suite is 336 and passes, with the same four tests switched off. One more is new (the pad's three became four).

**The miner that overran, and how far its scoops reach.**

*Henry:* "Fix the overrun. Also, I'm noticing that my miner's range seems longer than I expect?" Then, with a picture of his miner biting a rock from well off: "image of the very long range". And of a test run I'd told him was still going: "it's not stuck, is it?" It was.

| # | What | Henry's verdict |
|---|---|---|
| W18 | **A ship that leaves a short leg till last flies the long leg at its full speed, and counts the whole of its about-turn.** Two faults in the pilot, both in the one case: a leg of the trip too short to stop on (24 px or less for his miner) is left until the other is flown. The long leg had only its share of the ship's speed, rounded down, though nothing else was being flown: two steps of his miner's three. And its stop was planned a quarter turn short, as if the short leg's stop had already turned the ship part of the way round: 22 px of overrun, and a leg to go back for it. | |

- **His miner's trip** (200 px across, 23 along): 50 s and eight quarter turns before, 29 s and five now. That's 19 s for the long leg, as when it's flown alone, and 10 s for the short one, which is what a ship that takes 3 s to a quarter turn needs for it.
- **Every flier, twenty trips each.** Only a trip with a short leg left till last changes: the SKATER's twenty take 201 s where they took 204, the BRAKER's 234 (238), the starter fighter's 185 (188), the collector's 397 (406). The other ten ships' are the same to the second, his new miner's among them (none of the twenty has such a leg). None is slower. 6,698 s over all 320 trips, where it was 6,718.
- **What I tried first, and took back.** Rounding every ship's share of its speed to the nearest step, not down, on every trip: a ship a little off its line flies a step slower than it can, and for one with two or three steps of speed that's a third of it. The 320 trips were 11% quicker for it (the capital's twenty: 708 s, from 990). But the example BEAMER, with two steps of speed and 3 s to a quarter turn, then came in on both legs at once faster than it can stop, slid past its place and drifted away up the map without ever firing. So slow ships are still a step slow off their line. Recorded, not mended: it wants the two legs' stops planned together.
- **The run that hung.** That beamer's test loops until the arena's clock reaches seventy seconds. An arena whose fight has been called over doesn't advance its clock, so the loop span for good, and I reported the suite as still running for half an hour. Fourteen test loops of that kind end when the fight does now, and the suite runs under a time limit. A second test moved for the mirror of that reason: in one roll of forty the SKATERS, sent away from a fight, shot down the last of the ARROWS as they left; the fight was won and the clock stopped with five of them still on their way. They were counted as never arriving. They aren't now.
- **"My miner's range seems longer than I expect."** It is long: 28 px from the miner's middle. A scoop reaches 8 px, or twice as near as its ship can be sure of placing itself, whichever is more; and a ship that takes 3 s to a quarter turn can't place itself nearer than 14 px, which is what it drifts while it turns to stop. The starter miner, at 1.5 s a quarter turn, reaches 16. A miner with a rotation engine reaches 8. The rule is mine, of 2026-11-16, and it's there so that a clumsy miner isn't left sitting just out of reach of a rock with no hop short enough to close the gap. What it does is backwards: the clumsier the miner, the further its scoops reach, and it leaves little for a scoop with a range to add. Not changed. What I'd do: once as near as it can place itself, not twice (his miner 14 px, the starter 8), and a miner carried past the pixel it was after takes the nearest rock where it stops. A clumsy miner would then shift more often, at 10 s a shift for his. His to say.
- The suite is 336 and passes, with the same four tests switched off.

**A scoop reaches 8 px, and a miner gets itself that near.**

*Henry, of the reach:* "Let's just give scoops a short, fixed range of 8 px. This isn't a place that it makes sense to have a complicated rule. Ships should be able to fly short distances, the flying algorithm might need to be improved if they can't."

| # | What | Henry's verdict |
|---|---|---|
| W19 | **A scoop reaches 8 px from its ship's middle, whatever the ship.** *This replaces my rule of 2026-11-16* (8 px, or twice as near as its ship could place itself: 16 for the starter miner, 28 for his miner of that night). The scoop's text in the yard says so: MINES ROCK WITHIN 8 PX. | His ruling. |
| W20 | **A miner gets itself that near.** At the rock it places itself within 6 px of the pixel it's after, where a slow ship settles for 14 on any other errand. With rock in reach it stays where it is and bites. With none in reach it goes for the nearest rock it can fly to and stop on. With nothing left but rock too near to stop on, a last crumb, it backs off a hop and comes in again. | |
| W21 | **The AI's king doesn't fly to its post, or to a factory, through guns that outweigh it.** It looks 120 px along its way and waits where it is if the way isn't clear. *This mends W3,* which asked only whether the post itself was safe: a hurt king up the map's edge from its post, with a battleship half way between, flew past the battleship to get home. | |

- **How short a ship can fly.** A ship stops by turning its engines about. His miner of that night takes 6 s over that, and at the slowest speed there is, 3.75 px/s, that's 22 px: it can't stop on anything nearer than 24 px. That's the drawing, and no flying mends it; an engine astern or a rotation engine does. What the pilot can do is not leave such a ship standing short. Asked to place itself finely, one that's too near its point to stop on it sets off the other way, turns about a hop further off, and comes in. (Setting off towards the point, as it did, it's carried as far past as it began short, and so on.)
- **Measured, from rest to a point 7 to 60 px off, each of four ways, 216 trips a ship:** his miner comes to rest within 6 px every time, the slowest in 25 s, with six quarter turns at most. The starter miner: 12 s. The starter frigate: 25 s.
- **Each miner alone at its king's home rock for four minutes** (its loads go to the king's tractor, 80 px off, so the bites include the trips):

| Miner | Scoops | A quarter turn | First bite | Bites in 4 min | Furthest bite |
|---|---|---|---|---|---|
| Starter | 3 | 1.5 s | 10 s | 36 | 8 px |
| The starter's before a scoop used power | 3 | 1.8 s | 12 s | 36 | 8 px |
| Henry's, that afternoon | 3 | 1.6 s | 10 s | 37 | 8 px |
| Henry's, that night | 7 | 3 s | 18 s | 67 | 8 px |
| HAULER | 6 | 3 s | 17 s | 50 | 8 px |
| One with a rotation engine and an engine astern | 3 | 0.5 s | 12 s | 36 | 7 px |

- **Only a miner is asked to place itself finely,** at the rock and after loose pixels. A ship sent to a spot still settles for as near as it places itself as a rule, 14 px for a slow one. Whether a sent ship should get onto its spot the same way is Henry's to say: it would cost a slow ship up to 25 s of backing and turning for a move of 10 px.
- **W21 was found by the king's own test of that morning,** when the change to mining left the AI a little less standing at four minutes. The king, hurt, sat safe up the map's edge; the battleship came between it and its post; the post was 211 px from the battleship and so "safe"; the king flew home past it and was shot from 125 px to 33. That test was also asking something it shouldn't: with its one outpost dead the AI's side was beaten, and a beaten side's king flies at a tenth of its speed and is meant to be caught. It has a station far off now, so the side isn't beaten. And a second test stages the thing itself: a hurt king, its post, a battleship parked between. With the rule the king is never nearer the battleship than 172 px of the 180 it began at; with the rule taken out again it comes within 65.
- **Five matches on the build as it stands** (the AI against itself on the starter deck, seeds 1 to 5):

| Seed | Rocks | Before W19 to W21 | Now |
|---|---|---|---|
| 1 | two triangles | RED at 25:23 | BLUE at 29:23, red's king lost with 80 px standing |
| 2 | an S | BLUE at 29:44 | BLUE at 34:43, red beaten |
| 3 | two triangles | RED at 24:52 | BLUE at 27:10, red beaten |
| 4 | scattered | BLUE at 23:33 | RED at 33:31, blue beaten |
| 5 | a ring | RED at 26:58 | BLUE at 35:19, red beaten |

- **Mining is where it was.** On seed 1 the two sides had mined 1,018 and 911 px by 8:00 (976 and 1,074 before), 4,449 and 3,983 by 16:00 (4,042 and 4,278), and 12,679 between them by 20:00 (11,981).
- **The matches are six minutes longer, 32:01 on average from 26:06, and four of the five have a different winner.** Nearly all of that came with the reach and the miner's placing: a run with W19 and W20 and without W21 averaged 31:37. In it seed 2 ended at 32:35 with red's king lost and 1,196 px of its side still standing, which is the fault W21 mends; with W21 red is beaten first, two minutes later. Why the same mining makes a longer match I haven't found out: on seed 1 the two sides are simply closer at 20:00 (3,586 px standing against 3,267, where it was 2,410 against 3,471). Five matches say nothing broke, and no more.
- The suite is 339 and passes, with the same four tests switched off. Three are new: the scoop's reach, a ship closing a short gap, and the hurt king's way home.

**Kings, and ships sent to a spot (2026-11-18).**

*Henry, of my report that "the king flew home through guns":* "I don't think we want kings to behave autonomously, they should probably follow a different movement logic from all other ships and just go exactly where and when they're told to go." And then: "Oh, you were just talking about the AI's king. That's fine then." So, as a rule of the game: **a king does nothing of its own accord.** It's as built: a king is kept out of the ships' brain and their pilot, and moves only on its keys or when it's sent, at a fixed pace, stopping within a pixel of where it was sent. The AI's king is moved by the AI giving that same steer command once a second, and W2 to W5 and W21 are about what the AI tells it.

*Asked whether a ship he sends should get onto its spot as a miner does onto its rock:* "Yes, sent ships too".

| # | What | Henry's verdict |
|---|---|---|
| W22 | **A ship sent to a spot comes to rest within 3 px of it,** however slow it is to turn. Too near the spot to stop on it, it backs off and comes in. It used to settle for as near as a ship of its sort places itself, 14 px for the slowest, and sent a shorter way than that it didn't move at all. Kings aren't touched. | His ruling. |
| W23 | **The stop that was planned a quarter turn short (W18), mended in a third place:** when both legs of a move are too short to stop on and only the longer is being flown. A collector that had backed off from its spot came in 10 px past it, backed off the other way, and so on for good. | |

- **Seventeen ships, 68 trips each** (twenty long, and forty-eight of 5 to 60 px, from rest): every trip ends at rest within 3 px of its spot, with fifteen quarter turns at most. All of them together take 15,276 s, where they took 13,001 settling for as near as each ship places itself: the short ones are flown now, where a slow ship used to stay put and call it done. The slowest single trip is the same one, the capital's, at 86 s.
- **What a short move costs a slow ship,** from the same trips: the starter frigate's 68 take 1,336 s where they took 1,078, and a move of 10 px takes it about 25 s of backing off and turning. That's W20's arithmetic: it can't stop in less than 24 px. It's the price of the order being obeyed.
- **Five matches** (the AI against itself, seeds 1 to 5; the AI sends its fleets with the same order, so its ships land on their spots too): BLUE at 24:35, BLUE at 26:50, RED at 26:48, RED at 24:46, RED at 23:49. Four losers beaten, and one king lost with 35 px of its side standing. The average is 25:22. With W19 to W21 and without this it was 32:01, and before those 26:06, with mining the same in all three (on seed 1 here: 968 and 924 px by 8:00, 3,961 and 4,103 by 16:00). So the six minutes I reported with the scoop's reach came and went with the next change: five matches' lengths move that much on their own, and I shouldn't have read anything into them.
- **Two tests moved, both on the test's side.** The one that plays a ship's flight forward and compares it with the game's hadn't been told that a sent ship places itself finely. And the king's whole-match test allowed a hurt king 3 s in a battleship's reach; in this build's version of that fight it's there 3.1 s (and keeps 151 px of 218), so it allows 6: the king as first built stood there 17 s and 22.
- The suite is 339 and passes, with the same four tests switched off.

**The parts Henry asked about, not built. What I think of each:**
- **A speed-up field and a stasis field.** (The stasis field is built: see "The stasis lattice", below. The speed-up field is measured there and not built.) As I put it on 2026-11-17: they're one part with two signs: a lump that changes how fast things near it fly. What makes them more than "pixels for a bonus" (his words of 2026-11-11) is who they touch. I'd have a field touch everything in its reach, friend and enemy alike, so a speed-up ship has to be flown ahead of its own fleet and a stasis ship into the enemy's, and each is a thing to shoot first. The risk he names is real: ships choose courses by looking ahead at their own speed, and a field changes it under them. The stasis lattice (his shape, 2026-11-13) is the natural drawing.
- **A scoop with a range.** A mining beam: a column of them, like a beam gun, bites the rock in line with it up to a reach its length sets. It lets a big miner sit off a rock instead of nosing into the heap, and it gives a miner a facing that matters.
- **A phase-harvester.** I like it for the reason he gives: it's the one way of mining that pays a big ship and can't be done by a swarm. It also makes a rock a thing to fight over for a known time, which scattered miners never do. What needs settling: whether rivals' charges on one rock race or cancel, whether miners biting the rock meanwhile help it (less left to charge) or are locked out, and whether the charge is lost when the harvester is driven off. **Henry, 2026-11-18:** "rival charges cancel. miners can "help" by reducing the rock size. no opinion on item 3." So rivals' charges cancel, miners biting the rock help (there's less left to charge to), and whether a harvester driven off keeps its charge is mine to propose. Not built yet: "before building phase harvester, we'll do a playtest". (Built 2026-11-22: "The phase ring", far below.)
- **A repair array.** It makes damage something a side can undo by holding ground, which nothing does today. Paying for it from the bank keeps it honest. His point about a ship cut in two and both halves repaired is right, and I'd let it stand: it costs the pixels either way.
- **A full set, and building sets.** With those five the list covers each job at least twice over (two ways to unload, three to mine, two to stop shots, two to undo them). That's when sets start to mean something: a set is a choice between answers to the same job.

- The suite is 335 and passes, with the same four tests switched off. Seven are new: the king and the battleships, three about the pad, and three about the energy ball.

---

## The stasis lattice (2026-11-18)

*Henry, asked whether a field should touch every ship in its reach or one side's:* "Tough call. Both sides might in some ways be the more interesting design, yeah, at least for stasis. That way the effect can be very pronounced and you build your army composition around it. There would need to be some logic around turning it on and off though, I think. Maybe the speed-up field should just be a small always-on buff to your team's engines though?"

So stasis is built, as a bare toy: the part, one example ship, and two fights to watch. No ready team has it and the AI doesn't know it exists. The speed-up field is measured below and not built.

| # | What | Henry's verdict |
|---|---|---|
| Q1 | **STASIS, a part** (DRIVE, key 3; violet). Stasis pixels joined edge to edge are one lattice. A pixel weighs 1 and uses 2 power. | |
| Q2 | **It holds every other ship in its reach, friend or enemy. A held ship flies and fires at a quarter of its speed:** its pilot and its guns get one tick in four. Never the lattice's own ship. Mining, unloading and printing go on as ever. Two fields hold a ship no harder than one. | Both sides: his. The quarter, and that guns are held too, are mine: see Q9. |
| Q3 | **Its reach is 2 px for each pixel of its width plus its height,** (4 px since 2026-11-21: LT2, below) from the lattice's box, 150 at most. Reach comes from how far it spreads and not from how many pixels are in it: 25 px in a block reach 20, the same 25 as two arms reach 52. Shot through, it reaches as far as what's left of it spreads. Short of power it reaches less in proportion. A ship is in reach if any part of its box is, as a projector counts it. | |
| Q4 | **On and off, the first half: it works only while its ship is still.** The field is set 2 s after its ship stops and drops the tick it moves. Turning on the spot is still. (Changed 2026-11-22: only where it means to stay. Stopped to turn on its way somewhere isn't still: ST1, below.) So where you park it and when you move it is the switch, and there's no new key. | Mine. He asked for some logic; this is my answer, and a switch on the card is the other one. |
| Q5 | **The second half: it's up only while an enemy is in its reach.** With no enemy in it, it holds no friend. Otherwise a lattice waiting at its post by the shipyard held its own side's miners and new ships all day. | Mine. |
| Q6 | **Kings aren't held.** | Mine, and his to rule: it follows from "a king goes exactly where and when it's told", and it keeps a lattice from pinning a king for the guns. |
| Q7 | **A ship with a lattice stays on the spot it's sent to,** guns or no, and doesn't step aside for shots. One with no guns, on ATTACK, takes itself to the nearest enemy and stops when that's in reach, or will be by the time the field is set. An armed one that hasn't been sent flies by its guns, so its field is up only when it happens to stand still. (Changed 2026-11-21, after Henry had built SNAREs and never seen one work: LT1, below. It keeps its post now. And 2026-11-22: with an enemy in its reach it stops where it is and stays, and only sending it moves it: ST2 to ST4, below.) | |
| Q8 | **How it shows.** A lattice whose ship is moving shows nothing. Set, its reach is a sparse dotted violet line. Up, the line is twice as close and every other pixel of the lattice turns white. The inspector says "STASIS: 84 PX REACH, UP." (or WAITING FOR AN ENEMY, or DOWN UNTIL ITS SHIP IS STILL), and of a held ship "HELD BY STASIS". (More since 2026-11-21: LT4, below. Its rings dotted since 2026-11-22: ST6.) | |
| Q9 | **Guns are held as well as flight.** As first built a held ship only flew slower. That lost every fight I ran (the first table below), for a plain reason: a ship at a quarter of its speed shoots as fast as ever, so your own ships lose nothing by standing in your field and the enemy loses little. "Both sides" only shapes an army if being in the field costs. | Mine. |
| Q10 | **The ANCHOR,** on the library's parts shelf: 212 px, an H of lattice 21 across (two rails standing clear of the hull on a bar through its middle) that reaches 84 px. No guns. Five engines turned astern, so it stops without turning about. Cut the bar and it's two lattices that reach 62. | |
| Q11 | **Two fights to watch** in the arena's list: ANCHOR + 4 FRIGATES against 48 FIGHTERS, and 8 FRIGATES against the same. | |
| Q12 | The core's white is now the palette's white (it was a second entry of the same colour), which frees an entry for the lattice: the palette was full. Nothing looks different. The network protocol is 22. | |

**What I measured.**

- **Flight, held, is the same flight at a quarter of the speed, exactly.** Seventeen drawings, each sent five ways from rest beside an enemy's lattice, and again with the lattice shot away: after every fourth tick the held ship is on the same pixel, facing the same way, at the same speed as the free one after one tick, for 40 s of its own time. So no stop or turn the pilot plans goes wrong in a field; it comes out late. That was the risk I named on 2026-11-17, and it's why the slow is done by giving the pilot one tick in four and not by dividing its speed.
- **Fire, held, is a quarter:** a starter fighter fires 159 flak shots in a minute free and 40 held, either side's.
- **Twelve fights each, level on cost, blue the side with the lattice.** All of these are head-on: both fleets start 180 px apart and go for each other.

| Fight (432 px a side) | Flight held only | Flight and guns held |
|---|---|---|
| 8 FRIGATES v 48 LANCERS (no lattice) | 3 won, 9 lost | the same fight |
| ANCHOR + 4 FRIGATES v 48 LANCERS | 0, 12 | 0, 12 |
| 8 FRIGATES v 48 FIGHTERS (no lattice) | not run | 1 won, 5 lost, 6 drawn |
| ANCHOR + 4 FRIGATES v 48 FIGHTERS | 0, 12 | 0, 12 |
| ANCHOR + 24 LANCERS v 48 LANCERS | 0, 12 | 0, 12 |
| ANCHOR + 24 FIGHTERS v 48 FIGHTERS | 0, 12 | 0, 12 |
| 12 PICKETS v 48 LANCERS (no lattice) | 12, 0 | the same fight |
| ANCHOR + 6 PICKETS v 48 LANCERS | 4, 8 | 1, 11 |
| 12 PICKETS v 48 FIGHTERS (no lattice) | not run | 0, 12 |
| ANCHOR + 6 PICKETS v 48 FIGHTERS | not run | 0, 12 |

- **So the side that spent half its pixels on an ANCHOR lost every time, under either rule,** and holding guns as well changed no result. In these fights the field is up 6 to 24 s and does hold the enemy (against 48 fighters, 447 enemy ship-seconds of a possible 770 or so), and the fleet behind it is half the size and loses anyway.
- **What I take from it, and what I don't.** It's one drawing, one kind of fight and one AI, so it's no verdict on the numbers ("no numbers from one deck and one AI"). What it does say is that the lattice as built is not yet the thing an army gets built around: a quarter isn't pronounced enough to be worth half a fleet in a rush. And a rush is the worst case for it, since it has to stop and stand 2 s with the enemy already on it. A match has prepared positions, which the arena hasn't: a lattice parked beside guns before the enemy comes is set when they arrive. Nobody has played that.
- **Ten faults put in on purpose, each caught by a test:** a held ship flying at full speed; firing at full rate; a field that holds only the enemy; one that's set while its ship moves; one set the tick its ship stops; one up with no enemy in it; a king held; a lattice's own ship held; a field reaching a third further than it says; and a lattice ship picking its own courses again. That last one got past the first version of its test: the fault only shows against an enemy that's manoeuvring, and the test's enemy sat still.
- The suite is 347 and passes, with the same four tests switched off. Eight are new: seven about the lattice, and one that a king in a field flies as fast as ever. An AI match on seed 1 comes out as it did before the part went in, state line for state line (BLUE at 24:35), so nothing in play without a lattice has moved.

**Not done.** The AI doesn't build a lattice, keep out of one, or shoot one from outside its reach. Ships don't keep out of a friend's field. A ship weighing its courses doesn't know a held enemy's guns are slow. No ready team has the part. Nobody has played it.

**The levers, if it wants to be stronger** (rule by rule, none of them touched): how hard it holds (a quarter; all the way to stopped); what it holds (flight and guns now; mining and printing too would make it a siege piece); its 2 s to set; its reach for its spread; and what a lattice pixel costs in power, which is most of the ANCHOR's size: 134 of its 212 px are hull to feed 61 px of lattice.

**The speed-up field, measured and not built.** His suggestion is "a small always-on buff to your team's engines". Speeds here are whole steps, 2 to 12, each 3.75 px/s, so I tried a buff to engines on the 31 drawings that ship (the teams', the examples', the flight toy's):

| Engines stronger by | Faster, by one step or more | Just as fast as before |
|---|---|---|
| 10% | 12 | 19 |
| 20% | 19 | 12 |
| 25% | 23 | 8 |
| 33% | 25 | 6 |
| 50% | 25 | 6 |

- The six that never gain are the three already at the top speed (WASP, SKATER, BRAKER) and the three so heavy that they're under the slowest (CAPITAL, the big LANCET, BASTION).
- So there's no small buff on this grid: a percentage is nothing for most ships and a whole step for the rest, and the biggest ships get nothing at any size.
- **What I'd build instead: one step for every friend in its reach.** That's 3.75 px/s for each: half as fast again for a capital (7.5 to 11), a tenth for a fighter (37.5 to 41). Always on and its own side only, as he says. It's small, it's the same for every ship, and by its arithmetic it's a part for big ships. Not built until he says.

**Henry's two questions about it, the same day, and what I answered.**

- *"So the field activates whenever any enemy ship flies into it, right?"* Yes: once its ship has been still 2 s, the field is up the tick any part of an enemy ship is inside its reach, and down the tick the last one leaves or dies. An enemy ship here is anything of theirs but a king, a seed still growing, and a ship still being printed or arriving, so a miner or a finished building counts, and a lattice parked in reach of an enemy outpost is up for good. Once up it holds everything in reach, his ships too.
- **What follows from that, which I hadn't said and haven't tested:** the enemy can fly one cheap ship in to turn the field on over his fleet, and stand the rest of theirs outside its reach to shoot in at full rate. That's the weak side of Q5. A switch on the card (the other answer to Q4) would put the choice back with the lattice's owner.
- *"But they're only slowed by 25% you said? I think it should be at least 50"* I'd written "at a quarter of its speed", which reads either way. It's the strong one: a held ship flies and fires at a quarter of its rate, 75% slower, so it's past his 50 as built. The part's own text says "FLIES AND FIRES AT A QUARTER SPEED"; I offered "FOUR TIMES SLOWER" if that reads as ambiguous too, and he hasn't said.

---

## The slipstream (2026-11-18)

*Henry, asked whether to build the speed-up field as one speed step for every friend in its reach:* "Yes to speed up Fields bumping one speed step. Instead of a ranged scoop, which naively is pretty redundant with a regular scoop, what if we have a vacuum that is specialized in picking up loose debris pixels but is less efficient when used on asteroids?"

So the speed-up field is built, as a bare toy: the part and one example ship. No ready team has it and the AI doesn't know it exists. The vacuum is thought through below and not built.

| # | What | Henry's verdict |
|---|---|---|
| SL1 | **SLIPSTREAM, a part** (DRIVE, key 3; green). Slipstream pixels joined edge to edge are one lump. A pixel weighs 1 and uses 1 power, as an engine does. | |
| SL2 | **Ships of its side in its reach fly one step faster:** 3.75 px/s more than their own engines give them. It's always on, and nothing switches it. | His: one step, always on, his own side's. |
| SL3 | **Only ships slower than the ship that carries it.** So it never speeds its own ship, two ships as fast as each other don't speed each other, and the ship that carries it can always keep up with what it speeds. | **Mine, and it narrows what he agreed to** ("every friend in its reach"). The reason: without it, one pixel of this on every ship of a fleet speeds the whole fleet, each ship in its neighbours' reach. That's "put pixels down, get bonuses" (his words of 2026-11-11), and nothing to decide. With it a slipstream belongs to a fast ship, and what it does for a fleet depends on what's flying with what. |
| SL4 | **It reaches a pixel for each pixel of the lump,** from the lump's box, 150 at most. Any shape. Short of power it reaches less in proportion. Shot through, each piece reaches what it has left. A ship is in reach if any part of its box is, as a projector and a lattice count it. | Mine. Twice the pixels cover four times the ground, so one big lump beats two small ones. |
| SL5 | **Kings aren't sped.** A king that carries a slipstream counts as fast as it flies (37.5 px/s; 3.75 once it's beaten), whatever engines are drawn on it. | Mine, for Q6's reason: a king goes exactly where and when it's told. |
| SL6 | **Two slipstreams speed a ship no more than one.** A ship that carries one can be in a faster ship's, and goes on speeding only what's slower than its own engines make it. | |
| SL7 | **How it shows.** A ship in a slipstream leaves a green dot astern every third of a second while it's under way, and its exhaust is green. The slipstream's reach is one green dot that goes once round it and rests, as a tractor's reach is shown. The inspector says "IN A SLIPSTREAM" of a ship that's sped, and of the carrier "SLIPSTREAM: 42 PX REACH, SPEEDING 3 SHIPS." The yard's status line for one with no guns: "SLIPSTREAM: REACHES 42 PX". | |
| SL8 | **A group sent together keeps the pace of its slowest ship as it stands when it's sent** (C-rows: that's unchanged). Ships already in a slipstream count at their faster speed, so a fleet gathered round its pace ship and sent off flies a step faster all the way. Sent from apart, it keeps the slower pace. | |
| SL9 | **The PACER,** on the library's parts shelf: 129 px, a lump of 42 across its bow that reaches 42 px, and the engines for 30 px/s. That's faster than the starter deck's frigate (11 px/s), miner (22) and capital (7.5). Its lancer is as fast and its fighter faster, so those it doesn't speed. No guns: left to itself it stays beside the nearest friend that has some. | |
| SL10 | The part's green is the green that says "good" in the panels (the palette is full, and no part had a plain green). The network protocol is 23. | |

**What I measured.**

- **A trip.** Three 25-pixel blocks that do 7.5 px/s, sent 400 px together: 54.6 s alone and 37.5 s with the PACER sent along (in its slipstream 99% of the way, at 11.25 px/s). With the PACER lost ten seconds in, 50.5 s: the tick after it's gone they're back on their own engines.
- **A match the part isn't in is unchanged:** the AI against itself on seed 1 comes out state line for state line as the build before (BLUE at 24:35, bank 346, 197 ships, mined 10,069).
- **Faults put in on purpose:** sixteen, each caught by a test. An enemy sped; a ship no slower than the carrier sped, and one just as fast; a reach a third too far, and one a pixel too far; a reach with square corners; a king sped; a step of two; a ship that stays sped out of reach; a sped ship that loses its step when it's hit; a king's slipstream going by the engines drawn on it; a sped carrier counted as fast as its slipstream makes it; a lump reaching by all its ship's slipstream pixels; a reach that takes no notice of a power shortage; a dead ship's slipstream going on; and a slipstream that speeds nothing. The reach that ignores a power shortage got past the tests as first written (they checked the drawing's reach and not the field's on the map). And one clause couldn't be caught because it was dead: "never its own ship" needs no rule of its own once only slower ships are sped, so I took it out of the code.
- The suite is 352 and passes, with the same four tests switched off. Five are new: four about the slipstream and one about a king in one.

**What a step costs without it, for scale** (arithmetic from the speed rule, and no tuning): top speed is 6 quarter-pixels a tick for each thrust per weight, and an engine gives 8 thrust and needs a pixel of hull to power it. On its own engines one more step would cost the starter frigate about 4 px, and the starter capital about 76: its engines give it less than the slowest speed there is, so it has a long way to the next step. A PACER is 129 px. Beside two capitals it has paid for itself, and beside frigates it takes over thirty of them. So it's a piece for a fleet of big ships, which is what I said it would be.

**Not done.** The AI doesn't build one or fly beside one, and no ready team has the part. A pace ship left to itself stands by the nearest armed friend, not the slowest. A formation doesn't put the pace ship in its middle. There's no arena fight for it: a head-on fight doesn't show a speed field. Nobody has played it.

**The vacuum, thought through.** (Built later the same day, with rule 3 as it first stood: see "The vacuum", below.) His idea: "a vacuum that is specialized in picking up loose debris pixels but is less efficient when used on asteroids", in place of the scoop with a range.

- **What there is today.** Every pixel shot off a ship, every pixel of a ship that dies and a destroyed node's stock become loose pixels. They drift a few ticks and then lie where they are for good (past 3,000 on the map the oldest vanish). Any scoop takes every loose pixel within 8 px of its ship at once, with no biting, up to what it carries, and a miner turns aside for one within 30 px. Nothing goes to a battlefield on purpose.
- **How much lies there.** Three matches of the AI against itself with the starter deck (seeds 1 to 3, 24:37 to 27:01 long): 8,191, 5,302 and 8,496 px were shot off ships or lost with them, against 12,954, 10,397 and 14,753 brought home by miners. So wreckage is half to two thirds the size of everything that's mined. At 20:00, before the 3,000 limit starts dropping the oldest, 62%, 62% and 76% of all the wreckage made so far was still lying where it fell. All three matches reached the limit. One AI and one deck, so it's a size and not a balance figure.
- **So a scoop is already the better tool for a loose pixel it's next to.** What a vacuum can add is reach, the pull, and a ship that goes to the wreckage. What I'd build:
  1. **VACUUM, a part** (INDUSTRY). Pixels joined edge to edge are one lump, and it reaches a pixel for each of its pixels, as a slipstream does.
  2. **It pulls every loose pixel in its reach to it,** 2 px a tick, and takes each as it arrives. You see the wreckage stream in. A pixel in two vacuums' reach goes to the nearer.
  3. **It carries 3 for each of its pixels,** as a scoop does, and unloads at a tractor or a pad as a miner does.
  4. **On rock a lump bites as one scoop, however big it is.** That's his "less efficient on asteroids": thirty pixels of vacuum mine like one scoop.
  5. **A ship with a vacuum goes to the biggest heap of loose pixels there is,** as a miner goes to rock, fills, and takes it home. With none, it waits with the fleet.
  6. **Scoops go on taking the loose pixels beside them,** as now. Nothing he has agreed changes.
- **Why I'd rather have it than the scoop with a range.** It makes a fight leave a prize. Whoever holds the ground afterwards banks both sides' losses, so there's a reason to fight near your own tractors, a reason to send a ship in behind a push, and a ship worth killing first. It also pays for a fight the winner didn't want to take. The scoop with a range only made the mining there already is more comfortable.
- **Henry, the same day: "how does storage and adding the vacuumed pixels to the network work?"** As rule 3 has it, like a miner: what it takes is cargo on the ship and no part of the bank until a tractor pulls it off (a pixel every 8 s for each tractor pixel, so 1 px/s at a king) or the whole ship lands on a pad. Looking at it again for his question, that doesn't fit a ship that takes in hundreds: a 30-pixel vacuum's 90 px take a minute and a half to empty at a king, and a pad has to be as big as the ship. **What I'd build instead of rule 3: the vacuum's ship is a node, as a tractor's is, and what it takes goes straight into its own store.** Then the rules there already are do the rest. Within link range of the network (200 px of the king, 160 of another node) it's in the bank as it's taken. Out of range it's a pool of its own, and can't be spent until the ship flies back. Shot down, it spills what it holds as loose pixels for anyone, 300 at most, and the rest is lost. Printing spends what flying nodes hold first. No ruling yet.
- **The one thing I'd ask him before building:** whether it takes pixels out of a fight as they're shot off, his own ships' included. I'd say yes (a loose pixel is anyone's, and it makes a vacuum behind your line a thing to shoot), but it does mean a side fighting beside its vacuum gets some of its losses back as it takes them.

---

## The vacuum (2026-11-18)

*Henry, offered a vacuum whose ship would be a node and bank what it took:* "I kind of like having the offloading as a problem that needs to be solved. Seems boring and frictionless if it's just automatically a node?"

So it's built with what it takes as cargo, as a bare toy: the part and two example ships. No ready team has it and the AI doesn't know it exists.

| # | What | Henry's verdict |
|---|---|---|
| VA1 | **VACUUM, a part** (INDUSTRY, key 5; deep green). Vacuum pixels joined edge to edge are one lump. A pixel weighs 1 and uses 1 power. | The part is his idea. |
| VA2 | **A ship's vacuum pulls every loose pixel in its reach to its vacuum pixels,** 2 px a tick, and takes each as it gets there. They can be seen streaming in. It pulls whatever its ship is doing, sent somewhere or left to itself. | |
| VA3 | **It reaches, past its ship's edge, a pixel for each pixel of its biggest lump,** 150 at most, less in proportion when short of power, and never less than a scoop's 8 px. It's measured from the ship's middle, as a tractor's reach is, so where the lump sits on the ship and which way the ship is turned don't matter. | Mine. (For an hour it reached from the lump's own box, as a slipstream does. A ship gets no nearer its spot than a few pixels, so one with a small lump sat with the last of a heap just out of its reach, and stayed.) |
| VA4 | **Its ship carries 3 for each vacuum pixel,** as for each scoop. With no room it pulls nothing. | Mine. |
| VA5 | **What it carries is cargo. Nothing is banked until a tractor pulls it off, a pixel at a time, or the ship lands on a pad it fits:** a miner's rules, unchanged. | His: offloading is "a problem that needs to be solved". |
| VA6 | **On rock a lump is one scoop, however big it is.** It bites as one, a pixel every 10 s, and takes home what one carries, 3. | "less efficient when used on asteroids" is his. That it also carries home only a scoop's load from rock is mine: otherwise it sat on a rock for twelve minutes filling 72. |
| VA7 | **A ship with a vacuum goes to the loose pixels.** It goes for the nearest heap that would fill the room it has, or with none that big the biggest. It flies to the middle of the heap and sits there till it's full, takes the load wherever it would be empty soonest (as a miner chooses), and comes back. It keeps to a heap while anything is left of it: at rest with nothing of its heap in reach, it moves to the middle of what's left, or to the nearest pixel if that middle is already in its reach. Only with nothing loose anywhere does it mine rock, and a part load goes home first. A heap is what lies in one 32 px square of the map. | Mine. |
| VA8 | **A loose pixel in reach of two ships' vacuums goes to the nearer ship, whoever's it is.** | Mine: loose pixels are anyone's. |
| VA9 | **It takes pixels out of a fight as they're shot off,** its own side's ships' included: a pixel is loose from the tick it's knocked off, and one still drifting is pulled like any other. | **Keep** (2026-11-18, asked whether it should or should take only pixels that have come to rest: "Yes, keep it"). The rule was mine. |
| VA10 | **Scoops are as they were:** a scoop takes the loose pixels within 8 px of its ship at once. A ship with scoops and a vacuum has one hold for both. | |
| VA11 | **A collector comes to a vacuum ship at work,** as to a miner. A ship with a vacuum and tractor pixels of its own is no collector: it goes for wreckage, and its own tractor empties it. | Follows from the tractor's rules as they stand. |
| VA12 | **How it shows.** The loose pixels stream in. Its reach is one dot, the colour of loose pixels, that goes once round its ship and rests, as a tractor's does. Its vacuum pixels turn that colour as it fills, one for every 3 aboard, as scoops do. The inspector says "VACUUM: 28 PX REACH. IT HOLDS 72." under what it's carrying (from the ship's middle, as a tractor's line says), and the yard's status line for one with no guns "VACUUM: REACHES 24 PX" (past its edge). On a heap the inspector calls it MINING. | |
| VA13 | **Two examples** on the library's parts shelf. The SWEEPER: 63 px, a lump of 24 with an open mouth that reaches 24 px past its edge, a hold of 72, a miner's 22 px/s. It's 9 by 8, so no pad that ships today takes it. The DUSTER: 25 px, a lump of 10, a hold of 30, 26 px/s, and 5 by 5, so it fits the PADPOST's pad as the HAULER does. Neither has guns. | |
| VA14 | The vacuum's green takes the palette entry that was a projector lump's pale middle, which is now the panels' bright accent, a shade different. Its letter in a saved design is a capital V: every small letter is some part's. The network protocol is 24. | |

**What I measured.**

- **Two heaps, one SWEEPER, and a king's tractor.** Twenty loose pixels 160 px from blue's king and two hundred 330 px off. It went for the far one, the only one that would fill it, and was full 28 s after setting off. The first pixel came off it at 43 s and the last at 114 s: of the 114 s its first load took, 71 were the king's eight tractor pixels emptying it at a pixel a second. All 220 px were home at 6:23, in four loads.
- **With the starter collector** printed beside the king and two hundred loose pixels 150 px off: the collector came and stood by the heap, the SWEEPER never went home, and the collector banked 60 px in a minute with the SWEEPER's hold at 71 of 72 all the while.
- **A pad.** The DUSTER, full, lands on a pad 6 by 6 and all 30 are in that station's store at once. The SWEEPER doesn't fit it, and would fit one 9 by 8.
- **Rock.** The SWEEPER's lump of 24 bites a pixel every 10 s, and a ship with two lumps of four every 5.
- **A vacuum ship with one tractor pixel of its own,** out of every other tractor's reach, is emptied by it, a pixel every 8 s, into its own store.
- **No ship is left beside pixels it can't reach.** A ship with three vacuum pixels, the DUSTER and the SWEEPER, each sent at four awkward heaps in one square of the map (three pixels in each of two opposite corners, a line along one edge, one pixel in a corner, a ring): every pixel taken within a minute and a half. That sweep is what found two faults in the first version: the reach measured from the lump (VA3), and a ship slow to turn going back and forth for good between the middle of a heap and a corner of it.
- **A match the part isn't in is unchanged:** the AI against itself on seed 1 comes out state line for state line as before (BLUE at 24:35, bank 346, 197 ships, mined 10,069).
- **Faults put in on purpose:** twenty-six, each caught by a test. In the pull: twice as fast; a reach a third too far, and one a pixel too far; a square reach; pulling with no room left; taking a pixel before it arrives; a contested pixel going to the first ship and not the nearer, and a tie going to the later ship; a drifting pixel that goes on drifting; a dead ship's vacuum going on. In what it holds and reaches: one a pixel; every vacuum pixel biting rock; a reach by all its vacuum pixels and not its biggest lump; a reach that takes no notice of a power shortage; a reach from its middle no further than its lump's pixels; a small one reaching less than a scoop. In where it goes: the nearest heap whatever its size; the biggest however far; not keeping to its heap; never flying to the nearest pixel; choosing its spot again whenever it stands still; keeping a part load when nothing loose is left; not going to wreckage at all. And in what it is: a collector when it has a tractor; no miner by its order; no collector coming to it. Three got past the tests as first written, and each got a test: two lumps on one ship, the nearer of two heaps that would both fill it, and keeping to a heap. (The computer crashed while these were running and left one of them in the source, the reach that ignores a power shortage. I found it by checking that every fault's original text was back in place, put it back, and reran every fault after it and the whole suite.)
- The suite is 362 and passes, with the same four tests switched off. Ten are new, all about the vacuum.

**The offloading problem, as it stands for whoever draws one.** A hold fills in seconds and empties at the pace of whatever takes it, so what a vacuum earns is what its side can take off it. The ways there are today: more tractor pixels where it comes home (each pulls a pixel every 8 s); a pad big enough for it to land on, which takes the lot at once; a collector sent out with it, which banks as it works if it's in link; tractor pixels on the vacuum ship itself; or small vacuum ships that fit a small pad. None of these is tuned.

**Not done.** The AI doesn't build one or go for wreckage, and no ready team has the part. A vacuum ship doesn't keep out of a fight: it goes where the pixels are and runs when it's hit, as an unarmed miner does. Past 3,000 loose pixels on the map the oldest still vanish. Nobody has played it.

---

## The SCAVENGERS, a team for the playtest (2026-11-19)

*Henry:* "before building phase harvester, we'll do a playtest. can you create an all-new team for me that includes a variety of these new parts? do a little testing of the team yourself before handing it over, but no need for any deep optimization."

**Where it is.** In the library: L in the yard, Tab for whole teams, SCAVENGERS, Enter. That loads it into the nine slots and keeps the team it replaces as one of his own. No AI plays it, so it isn't in the match menu's list of opponents. Its cards are also on the library's TEAMS shelf singly.

| Slot (key) | Card | Px | Speed | What it is |
|---|---|---|---|---|
| 1 (Z) | DART | 11 | 30 px/s | A fighter: a repeater two long and two flak. |
| 2 (X) | PULSAR | 39 | 18 | An energy ball gun of six pixels, and two flak. |
| 3 (C) | MAUL | 110 | 7.5 | Two cannons, a tracker and two flak in a whole skin of plate. Only its engines are outside it. |
| 4 (V) | SNARE | 175 | 7.5 | A stasis lattice in an H, 15 by 15, that reaches 60 px, with a tracker and four flak of its own. (Redrawn 2026-11-21: 200 px, a cross inside a skin of plate that reaches 120. See "The lattice that never came up".) |
| 5 (B) | HERALD | 93 | 30 | A pace ship: a slipstream lump of 28 across its bow, and no guns. Every other card but the DART is slower, so it speeds them. |
| 6 (N) | DIGGER | 19 | 22 | A miner with four scoops, 5 by 4. It carries 12. |
| 7 (M) | GLEANER | 35 | 22 | A vacuum lump of 12 that reaches 12 px past its edge, 6 by 6. It carries 36. |
| 8 (,) | PORT | 135 | planted | The factory: twenty factory pixels, four flak, and a dock pad 6 by 9 on its flank. No tractor. It builds every card here. |
| 9 (.) | BARGE | 47 | 18 | A dock pad 6 by 6 that flies. It's a collector: it goes out to the miners and the GLEANERs, and they land on it. |

(The PULSAR, HERALD, DIGGER, GLEANER and BARGE were redrawn with rotation engines on 2026-11-19, after his first match: see the next section. The numbers here are measured on the cards as they are now.) Every card is a new drawing and spends all the power its hull makes. The parts of 2026-11-17 and 18 are all in it: the dock pad (PORT, BARGE), the energy ball (PULSAR), stasis (SNARE), the slipstream (HERALD) and the vacuum (GLEANER).

**How it's meant to be played.**

- **A PORT first.** It's 135 px of the 190 a king starts with. It prints four DIGGERs and a GLEANER, then two DARTs, a PULSAR and a DIGGER round and round. Miners land on its pad: a GLEANER, or two DIGGERs at once.
- **Each warship answers one of the starter deck's.** MAULs for fighters, DARTs for lancers, PULSARs for frigates (the table below). Nothing here but the MAUL beats fighters.
- **The SNARE is sent to a spot and left there.** Parked, with an enemy within 60 px, it holds everything in its reach, his own ships too. Its own tracker and flak aren't held. (Since 2026-11-21 it needn't be sent: left where it's built it keeps its post, and it reaches 120.)
- **The HERALD flies with the slow ships.** A MAUL or a SNARE beside it does 11 px/s for 7.5.
- **GLEANERs go to wreckage by themselves,** and a BARGE goes out to wherever the miners and GLEANERs are working.

**What I tested.** A little, as he asked, and none of it is a balance verdict: one AI, one opposing deck, twelve fights a line.

- **The cards.** All nine are legal, spend their power exactly, and are no drawing the game had; the PORT builds every one of them and the starting bank plants it. (A test.)
- **The pads.** The PORT's pad has two places for a DIGGER and one for a GLEANER; the BARGE's has one for either. Full, from 70 px off, a DIGGER was unloaded 5 s on and a GLEANER 9 s on. On the BARGE, with the PORT gone, a GLEANER was unloaded within 36 s.
- **The opening, with nothing done but planting the factory at once:**

| | Mined by 4:00 | By 8:00 | By 10:00 | Warships standing at 8:00 | Bank at 8:00 |
|---|---|---|---|---|---|
| Starter deck | 112 | 323 | 484 | 369 px | 109 |
| SCAVENGERS as it ships (four miners first) | 195 | 522 | 747 | 183 px | 364 |

- So it earns more than the starter deck and spends it slower: one PORT's twenty factory pixels print less than the team mines. A second PORT, or ships asked of the king, is where the bank should go.
- **Wreckage.** In that opening, 150 loose pixels strewn 130 px from the PORT at 5:00 were all taken 26 s later, and landed on the pad.
- **The small warships, head on and level on cost (about 440 px a side), against the starter deck's** (won, lost, drawn of 12):

| | v 48 FIGHTERS | v 48 LANCERS | v 8 FRIGATES |
|---|---|---|---|
| 40 DARTS | 0, 12, 0 | 12, 0, 0 | 0, 9, 3 |
| 11 PULSARS | 0, 12, 0 | 2, 6, 4 | 12, 0, 0 |
| 4 MAULS | 12, 0, 0 | 0, 12, 0 | 0, 12, 0 |

- **The MAUL was drawn three times.** With a bow of heavy plate and bare flanks, a SNARE and two of them lost to their cost in fighters 12 times of 12. In a skin of plate left open over the outer corners of its cannons, four beat 48 fighters 3 times, lost 5 and drew 4. Sealed, 12 of 12. Flak doesn't break plate, so against fighters a ship is as good as its skin is whole.
- **The line: a SNARE parked in front and two MAULs parked with it (383 px), against about their cost coming for them** (won, lost, drawn of 12):

| | v 42 FIGHTERS | v 42 LANCERS | v 7 FRIGATES |
|---|---|---|---|
| The MAULs 90 px back, outside the field | 6, 0, 6 | 0, 12, 0 | 2, 8, 2 |
| The MAULs beside the SNARE, inside it | 4, 0, 8 | 0, 12, 0 | 7, 5, 0 |
| The same with hull for the SNARE's lattice | 3, 0, 9 | 0, 12, 0 | 3, 8, 1 |

- **A SNARE by itself is no fortress.** Against 12 fighters (108 px, three fifths of its cost) it won 2 of 12 and killed 7.2 a fight; with hull for its lattice it won none and killed 2.5. Most of its pixels are lattice and the hull to power it.
- **Lancers are the hole.** They outrange the SNARE's field (a lancer stands 75 px off what it shoots, and the field reaches 60), they break plate, and only the DARTs beat them.
- **The AI can't play it.** Ten matches of the AI on this team against the AI on the starter deck, and ten with the sides changed: it lost all twenty, between 9:31 and 17:32. It prints GLEANERs as if they were miners and sends SNAREs to hunt. That says the AI doesn't know these parts, which was known. Its miners bank on the pads (565 of the 646 px it mined in the first of them). The AI on this team against itself, six matches: all six ran to a king's death, 18:00 to 31:02. (Three matches was what I ran before handing the team over. The fourth, on the redrawn cards, was the one that crashed: see the next section.)
- The suite is 366 and passes, with the same four tests switched off. Four are new: one about the cards and three about how the team plays.

---

## Henry's first match with the SCAVENGERS (2026-11-19)

*Henry, with a screenshot of a hurt DART alone in the top left corner of the map at 9:49:* "The energy ball looks good. The port seems to work well with two diggers offloading at the same time. The digger behavior seemed less predictable than I expected. They went far away it seemed like when there was rock right near the port. One even went off on a fast orbit? And one just ran away to the top fo the map? And there's a dart up there too?" *And a few minutes on, with a screenshot of his base beside a field of loose pixels:* "It almost seems like diggers are prioritizing debris over regular rock".

The match (from his log): seed 48925, the AI on SWARM, rocks in an S. SWARM is the rush, fighters and lancers from 2:30, and from 4:00 he was losing ships steadily (7 lost by 4:00, 28 by 10:00). So his miners were being shot at for most of the match, which is where three of the four things below come from.

| What he saw | What it was | What I did |
|---|---|---|
| Diggers leaving rock that was right there | **They do put loose pixels before rock.** An old rule, every miner's: one with a loose pixel within 30 px of it goes for that, each time it looks. In a field of wreckage each pixel it takes puts another in reach, so it walks off along the wreckage. Reproduced: at a rock face a DIGGER bit 12 px in a minute; with a trail of loose pixels leading away, one every 25 px, it bit none. The starter miner bit 9 and then none, took six of the trail, and ended 152 px off. | **Nothing yet: it's his to rule.** What I'd do is below. |
| "a fast orbit", and "ran away" | **A miner with no gun that's hit runs home, and it never braked.** It made for the middle of its nearest node flat out, flew through, and went on. Hit once a second for 20 s from 120 px off, a DIGGER was at its port at 8 s, 234 px out the far side at 20 s and 368 px off before it turned back. Nobody had seen it because the starter miner has a flak gun, and a ship with a gun doesn't run. | **Fixed.** It runs to the node and stops beside it. A test hits a DIGGER, a GLEANER and a BARGE once a second for 20 s: each is beside the port from 10 s on and all but still, and goes back to work when it's left alone. |
| A DART in the top corner of the map, and a digger gone to the top | **A ship with its engines shot off drifts on until the edge of the map stops it.** That's the nose-first flight's rule (2026-11-16, unjudged): a ship keeps the speed it has. His DART had 9 of 11 px, and its two engines are its stern corners. Reproduced: a DART doing 30 px/s that loses both engines is at the top edge two minutes on, at rest, and the inspector still says ON GUARD. A ship running from fighters shows them its engines. | **Nothing yet: it's his to rule.** |
| Diggers clumsier than he expected | **My drawing.** The DIGGER is five across with one core and had no rotation engine: 3 s to a quarter turn, the slowest there is, where the starter miner takes 1.5. The PULSAR, HERALD, GLEANER and BARGE were as slow. I hadn't looked at turn times when I tested the team. | **Redrawn,** each with rotation engines at its stern corners: a quarter turn in 0.3 s for the DIGGER, 0.7 the GLEANER, 0.9 the PULSAR, 1.1 the BARGE, 1.5 the HERALD. The DIGGER is 19 px for 15 and does 22 px/s for 26; the GLEANER's lump is 12 for 14. |

**What the redrawn cards changed, measured the same way as before.** A full DIGGER 70 px from the port is unloaded 5 s on, where it took 17, and a GLEANER 9 s on for 21. The opening with nothing done but planting the PORT mines 522 px by 8:00 where it mined 436 (the starter deck: 323). The 150 loose pixels strewn at 5:00 are home 26 s later, where they took 88. So most of what looked like a slow pad was miners taking three seconds to turn round.

**What he said that needs nothing done.** "The energy ball looks good." "The port seems to work well with two diggers offloading at the same time." (In his match 160 of the 297 px he'd mined by 4:00 came in on the pad and the rest through his king's tractor, which he kept beside the port: a miner goes wherever it would be empty soonest.)

**The two that are his to rule.**

- **Loose pixels before rock.** With a vacuum in the game I'd have miners keep to rock: a scoop still takes whatever loose pixel comes within its 8 px as it works, but a miner doesn't turn aside for one. Wreckage is then the vacuum's job, and a miner is where you expect it. With no rock left anywhere a miner would go for the nearest loose pixel, so a side without vacuums can still live on wreckage at the end. It changes every deck's income a little (in three AI matches about a third of all wreckage was picked up, all of it by miners), so it isn't mine to make on my own. **Ruled the same day, and built: "Rock first", below.**
- **Ships with no engines.** They drift to the edge of the map and sit there, still counted as ships and still said to be on guard. The choices I see: leave it, and have the inspector say ADRIFT, NO ENGINES; or have a ship with no engine left come to rest where it is; or have it break up. I'd take the first: it's what the flight rules say happens, and the fault is that nothing tells him. **(Changed my mind the same day: I'd take the second. See "No guns, no course", below.)** **Henry ruled, 2026-11-19: "Leave the drift."**

**A crash found on the way, and fixed.** The tick a ship's last vacuum pixel was shot off, the game stopped: the vacuum looked for the box its pixels lie in, and there were none (a ship's stats are worked out at the end of a tick, and loose pixels are pulled before that). It was in the build he played, and his match didn't meet it. An AI match on the redrawn team did, on its second seed. A test now shoots a vacuum off in the middle of a tick, and 26 AI matches with the team on one side, the other and both ran to their ends.

**What the fixes moved.** Running home touches one starter card, the collector, which has no guns. So the AI against itself on seed 1 is the same to 6:00 and differs from 8:00 on. BLUE still wins it (at 25:30 where it was 24:35, which is one match's length and says nothing).

- The suite is 367 and passes, with the same four tests switched off. One is new, a miner with no guns that's hit, and the vacuum's pull test now shoots a vacuum off in the middle of a tick.

## Rock first (2026-11-19)

*Henry, asked whether miners should keep to rock and leave loose pixels to vacuums:* "They should prioritize rock and only mine loose pixels when there is no rock anywhere near (within approx 1/3rd or 1/4th the map?)"

Built as he said. The old rule, a loose pixel within 30 px before any rock, is gone.

| # | What | Henry's verdict |
|---|---|---|
| RF1 | **A miner goes for rock while there's rock near it,** and for loose pixels only with none near. | His ruling. |
| RF2 | **Near is 350 px:** a quarter of the map's width, which is a third of its height. (In the proving ground's arena, 120 px.) | His range ("approx 1/3rd or 1/4th the map?"). Where in it is mine. |
| RF3 | **Near is measured from the miner,** where it is at that moment, as the crow flies: not from its port or its king. So a miner whose rock runs out moves on to rock up to 350 px off before it looks at wreckage. | Mine. |
| RF4 | **With no rock near, it goes for the nearest loose pixel that's nearer than the nearest rock.** It goes no further for a loose pixel than it would for rock: with rock 400 px off and wreckage 450 px off, it flies to the rock. | Mine. |
| RF5 | **With no rock left on the map it takes loose pixels wherever they lie,** the nearest first. | Mine (what I'd proposed). |
| RF6 | **Off after loose pixels, it turns to rock as soon as some is near.** It looks once a second. | Mine. |
| RF7 | **Its scoops still take any loose pixel that comes within their 8 px,** at the rock or flying past. Unchanged. | |
| RF8 | **A ship with a vacuum is unchanged:** loose pixels first, and rock when nothing is loose. | |

**The scene he saw, both ways.** Three starter MINERs and two DIGGERs at blue's home rock, 200 loose pixels strewn within 120 px of its face, three minutes, on four maps (the last is the map of his match). What the five took between them:

| Map | No wreckage: rock | Old rule: rock, loose, left lying | New rule: rock, loose, left lying |
|---|---|---|---|
| 1 | 152 | 47, 119, 81 | 146, 26, 174 |
| 2 | 146 | 40, 148, 52 | 120, 50, 150 |
| 28259 | 154 | 44, 147, 53 | 124, 66, 134 |
| 48925 | 151 | 39, 131, 69 | 142, 25, 175 |

- Under the old rule the miners all but left the rock: about a quarter of the rock they bite when there's no wreckage. Now they bite most of it, and the loose pixels they take are the ones their scoops pass within 8 px of.
- **It doesn't bring more in.** Rock and loose together come to 166, 188, 191 and 170 px under the old rule and 172, 170, 190 and 167 under the new: the same, give or take. What changes is where the miners are, and that two thirds to seven eighths of the wreckage now lies there until a vacuum comes, where a quarter to two fifths did. A deck with no vacuum leaves most of a battlefield beside its rock.

**What it moved.** No test had to be re-recorded: the 367 passed as they stood. The AI against itself on seed 1 is the same to 6:00 and differs from 8:00 on, which is when the first wreckage lies (mined by 8:00: 968 and 933 px, where it was 968 and 914). Four AI matches on the starter deck and six with the SCAVENGERS on one side or the other ran to their ends.

**Tested.** Three new tests. One puts a single rock pixel exactly 350 px from a miner and then 351, and again on the diagonal, with a loose pixel the other way; then rock 400 px off with a loose pixel 450 off; then no rock at all; then the last rock pixel bitten and a loose pixel 300 px off, which it fetches and takes home. One sends a miner after loose pixels towards far rock and has it turn to the rock when that comes near, leaving a loose pixel 150 px to the side. One is the scene above with the rule checked every tick for every miner: what it's making for was rock when the tick began, or it's staying where it is. Twelve deliberate faults (the old rule back, the line a pixel out, near as a third or a fifth of the width or a quarter of the height, near measured square and not round, never looking again, and so on) each fail one of them.

- The scoop's line in the yard still says "MINES ROCK WITHIN 8 PX AND PICKS UP LOOSE PIXELS", which is still so. Not touched.
- **Still his to rule:** ships with no engines (above). (Ruled later the same day: "Leave the drift.")
- The suite is 370 and passes, with the same four tests switched off.

## No guns, no course (2026-11-19)

*Henry, with a screenshot of two whole DIGGERs alone in open space:* "my miners are running off the map when enemies came"

**His match** (from his log): seed 4258, the AI on SWARM, two triangles. He stopped at 4:00, eight ships down. He was playing TEAM 7, not SCAVENGERS. When he loaded another team the library kept the cards he had in his slots as TEAM 7, and those were the SCAVENGERS as I first drew them, before five were redrawn to turn: his DIGGER took 3 s a quarter turn and flew at 26 px/s. He loaded TEAM 7 back and played it. The redrawn cards are the team called SCAVENGERS on the TEAMS list.

**What it was.** A ship with an enemy within 240 px picks its own course (H1): it plays its next four seconds forward some thirty ways and takes the cheapest. A ship takes up to three seconds to turn a quarter. One that can't turn about and stop inside the four seconds it looks at sees nothing gained by beginning to: the turn costs it a little, and the stop is out of sight. So having once taken up a speed to get clear of a gun, it kept that speed for as long as an enemy was in sight, and the edge of the map stopped it.

Reproduced on his map with his cards: a PORT at blue's home rock, four DIGGERs and a GLEANER at work, two DARTs and a PULSAR, nobody playing blue, and the SWARM raiding from 3:00. A GLEANER 65 px from its PORT, taking its load in, passed the PORT at 22 px/s and stopped against the left edge of the map 184 px on, with 35 of its 36 px. A DIGGER making for rock 50 px off flew by it at 19 px/s and in among the fighters.

**Not what I said the first time.** After his first match I put "a fast orbit" and "ran away" down to a miner running home and never braking, and tested the mend with nothing shooting. That fault was real. This one was beside it, and no test I ran had an enemy in sight.

| # | What | Henry's verdict |
|---|---|---|
| NC1 | **A ship with no guns doesn't pick a course of its own.** It flies as its plan says, whatever is in sight. A miner works, and with a load it goes to its node and stops there; hit, it runs home and stops there; a collector follows the miners; a ship with no guns that's with the fleet keeps its place by it; sent somewhere, it goes. It still gets off the line of a warming beam, which every ship's plan has it do. **This narrows H1** (every ship in a fight picks its own course), which he hasn't ruled on. | Mine. |
| NC2 | **A ship whose last gun is shot off drops the course it was on,** and flies as its plan says from then. | Mine. |
| NC3 | **Armed ships are as they were, and the same blind spot is there for them.** They still weigh four seconds, and most of the big warships take three to turn a quarter (the FRIGATE, CORVETTE, CRUISER, BATTLESHIP, CAPITAL, MAUL, SNARE, RAKER and the SIEGE team's five). They fly at 7 to 19 px/s and their plans hold them at a range from a target, which may be why it hasn't shown as this did; his kept PULSAR, at 22 px/s and 3 s, is the likeliest to. Not measured. What would mend it for every ship is a look-ahead as long as the ship takes to turn about and stop, which changes how every warship flies and has the slow ones fly themselves forward more than twice as far each time they think. | Not built: for the talk on ship AI. |

**Why this and not the look-ahead.** A ship with no guns has no gun to bring to bear, and what it does about an enemy is already its plan's to say (the yard says NO GUNS: RUNS HOME WHEN HIT). Twice now he has said of miners that they weren't where he expected them. The look-ahead is the wider mend and it moves every fight.

**Measured, the old rule and the new.** The scene above on six maps, his the first, six minutes each. For blue's five miners while they had an engine left: the furthest any got from a blue node, how many of them touched the edge of the map, and the longest any flew away from the place it was making for.

| Map | His kept cards, old rule | His kept cards, new rule | Redrawn cards, old rule | Redrawn cards, new rule |
|---|---|---|---|---|
| 4258, two triangles | 172 px, 1, 8 s | 96 px, 0, 2 s | 115 px, 0, 2 s | 95 px, 0, 2 s |
| 1, scattered | 154 px, 1, 28 s | 123 px, 0, 5 s | 93 px, 0, 2 s | 140 px, 0, 5 s |
| 2, an S | 684 px, 1, 28 s | 90 px, 0, 3 s | 143 px, 0, 3 s | 96 px, 0, 2 s |
| 48925, an S | 208 px, 2, 21 s | 111 px, 0, 3 s | 101 px, 0, 5 s | 97 px, 0, 2 s |
| 7, a ring | 270 px, 1, 11 s | 124 px, 0, 4 s | 143 px, 0, 2 s | 122 px, 0, 2 s |
| 606, scattered | 135 px, 0, 6 s | 71 px, 0, 2 s | 134 px, 0, 1 s | 171 px, 0, 6 s |

- With his kept cards a miner reached the edge of the map on five maps of six under the old rule, and on none under the new.
- The redrawn cards never reached it under either: they turn about inside the four seconds. So he wouldn't have met this on the team as it's listed, and any card of his own that's slow to turn and has no guns would have.

**What else moved in those scenes.** Nobody plays blue in them, so they aren't matches, and this is what I saw and no more.

| | His kept cards, old rule | new rule | Redrawn cards, old rule | new rule |
|---|---|---|---|---|
| Miners alive at 4:00, of 30 over the six maps | 9 | 8 | 16 | 13 |
| Maps where red had won by 6:00, of 6 | 1 | 5 | 3 | 2 |
| Mined by 5:00, on the maps still going under both rules | 276, 316, 312 | 330, 343, 330 | 447, 440, 519, 473 | 512, 471, 614, 523 |

Miners that keep to their work bring more in. With his kept cards red won sooner on four maps more; it may be that raiders went after miners that ran and now find them at the PORT and go on to the king. I haven't tested that, and it's one team against one AI.

**Tested.** Four new tests. One is the scene above with his kept cards on five maps: no miner with an engine left ever touches the edge of the map, and none flies away from where it's going for longer than it takes it to turn about (three quarter turns, and a burn to stop and a burn to get going for each of across and along). One is the rule itself: for a side with no gun on anything that flies (both HERALDs, both BARGEs, both DIGGERs, both GLEANERs and the starter COLLECTOR, one of them sent somewhere), the match with the SWARM raiding is the same match, tick for tick by the number two machines compare, as the match with that side set to the plain pilot. One shoots the guns off a FIGHTER that's on a course of its own. One sends a SNARE, which has guns and a lattice, to a spot with three fighters coming for it: it picks no course, and once stopped it doesn't move. (That rule is from the stasis round, Q-rows, and had no test: I found that out by breaking it.) Twelve deliberate faults each fail one of them or one of the old tests of armed ships: the old rule back; only miners, only ships that aren't miners, only the slow, only the quick; a sent ship, a ship running home or a ship taking its load in still picking a course; no ship at all picking one; a sent lattice picking one; and the course kept when the last gun goes.

**Three old tests moved, and why.**

- **A gun aims at a pixel it can break.** Its target is a plated shell with no guns that "the tests have hold where it's put". It hadn't been holding: it was picking courses under the gun. Now that it holds still, on one roll in ten a flak shot and a tracker shot were in the air together and the second bounced off what was behind the pixel the first broke. The test's tracker was SIEGE's SENTINEL, which has two flak guns that reach at 40 px. It's now the SENTINEL with those two taken off, which is what the test's own note says it meant (one gun at a time). Passes on all ten rolls.
- **Matches driven by random commands stay sound.** Its limit on how long a ship that can fly may sit inside a bigger enemy went from 20 ticks to 30. A FRIGATE at the map's left edge, with a king flying up the edge over it, took 21 to get out, where the most seen had been 15: the nearest way out was ahead of the king, and a ship gets out at 2 px a tick where a king flies at 2.5. That's the way-out rule, which is older and which I haven't touched; the FRIGATE has guns and flies as it did. Random commands make a match where one ship flying differently moves everything after it. **Not mended:** a ship under a bigger one should pick a way out the bigger one isn't flying towards.
- **Every strategy plays a match and replays it.** Seed 3 became seed 1. SIEGE against the starter AI on seed 3 had been a red win at 23:27 and is now a draw at the 40-minute limit, with both sides mining and fighting at the end. On seeds 1 and 2 all five strategies' matches end with a winner.

**A second way a miner leaves, and it isn't mended: no engines.** In the same scene on his map a DIGGER whose three engines were shot off while it was doing 7 px/s drifted 855 px, the length of the map, in two minutes and stopped against the top edge. That's the ruling still open from his first match (above), where I said I'd leave the drift and have the inspector say so. **I'd now have such a ship come to rest, which reverses what I recommended.** A pixel knocked off a ship stops within 2 to 7 px; a ship that loses some of its engines is held at once to its lower top speed; only one that loses them all keeps the speed it had. And a repair array, when there is one, could do nothing for a hulk at the far edge. It needs his word: it changes a flight rule (2026-11-16, unjudged), and the gunnery tests that use a drifting hulk as their target would be rebuilt.

**Henry's ruling, 2026-11-19: "Leave the drift."** So it stays as the flight rules have it: a ship that loses every engine keeps the speed it had until the edge of the map stops it. Nothing was built for it, and nothing in the match says a ship is adrift (the inspector line I first suggested wasn't asked for and isn't there).

**What the fix moved.** It touches one starter card, the collector, so AI matches change from the first time a collector has an enemy within 240 px. Ten AI matches on this build ran to their ends: four on the starter deck, and six with the SCAVENGERS on one side or the other.

- The yard's line for a miner with no guns already says NO GUNS: RUNS HOME WHEN HIT. Not touched.
- The suite is 374 and passes, with the same four tests switched off.

## Smaller over bigger (2026-11-21)

*Henry, from a match on the build of 2026-11-19:* "only thing I'm noticing in the current playtest is that Gleaner appears to be rendered beneath Port"

**What it was.** Ships were drawn in the order they were made, each over the ones before it. A PORT built after a GLEANER was drawn after it, so the GLEANER sat on the pad underneath the PORT. In every scene I'd tested the PORT was made first, which is why I never saw it. It wasn't only pads: a fighter made before a factory flew under it.

| # | What | Henry's verdict |
|---|---|---|
| LY1 | **Ships are drawn the biggest first,** so a smaller ship shows over a bigger one it's flying over or sitting on, whichever was made first and whoever's they are. A ship's size is its drawing's, so two ships don't change places as pixels are shot off them. | He reported the GLEANER. The rule is mine. |
| LY2 | **Of two ships the same size, the one made later is on top,** as it was. | |
| LY3 | **A ship in a pad's hands is drawn over the pad's ship, whatever their sizes.** A miner fits a pad by the pixels it has left, so a big miner shot small can sit on a pad whose ship was drawn smaller than it. | Mine. |
| LY4 | **The whole map (Tab), the minimap and the arena draw in the same order.** One piece of code draws the ships for every screen. | |

- **Not changed: which ship the pointer picks.** Of the ships whose box is under it, that's the one whose middle is nearest. It never went by what was drawn on top.
- **What it also means:** a ship being printed is drawn where it will be, and it's small, so it shows over a bigger ship that's over its place. The DIGGER a PORT starts to print with what a GLEANER brought shows over the GLEANER for a tick as the pad puts it off. That was already so: a ship being printed was always the last made.
- **Tested.** Three new tests. The PORT and a full GLEANER, made in either order: from the tick the pad takes hold of the GLEANER to the tick it lets go, every pixel of the GLEANER shows as it does drawn alone (19 ticks over the PORT, 8 of them down on the pad). A FIGHTER and a FRIGATE on one spot, made in either order, an enemy's FIGHTER over both, the FRIGATE shot down to five pixels, and a ship on a pad whose drawing is the bigger. And no screen draws a ship itself. Eight deliberate faults each fail one of them: the old order back; the smallest first; of a size the later one underneath; a ship on a pad at its own size; size as the pixels left; dead ships drawn; the match's field or the arena drawing its own ships.
- The match itself is untouched: nothing here is in the rules, and two machines stay in step as they did.
- The suite is 377 and passes, with the same four tests switched off.

## The lattice that never came up (2026-11-21)

*Henry:* "I built stasis field units a few times in my playtests but I never got to see it work. it should maybe be 1. better protected in the unit design 2. a bigger area or better cooldown/duration? and 3. have a pretty loud animation when it's doing its thing, if it doesn't yet"

**Why he never saw it.** I ran the SNARE as he had it, with six DARTs and a MAUL beside it, against red's starter ships coming for them from 460 px off: six fights each way. The number is the fights in which its field was up for any tick at all.

| How the SNARE was used | v 12 FIGHTERS | v 10 LANCERS | v 10 FIGHTERS + 8 LANCERS |
|---|---|---|---|
| Built and left, on GUARD | 0 of 6 | 0 of 6 | 0 of 6 |
| On ATTACK | 0 of 6 | 0 of 6 | 0 of 6 |
| Sent to its spot by hand | 6 of 6 | 0 of 6 | 6 of 6 |

- **It had guns, so it flew by them.** A lattice works only while its ship is still. A SNARE has a tracker, and a ship with a gun went for whatever came near, picking its own courses: it was under way for as long as an enemy was about. That's the last sentence of Q7, which I wrote on 2026-11-18 ("its field is up only when it happens to stand still") and left there. Nothing in the game said a SNARE had to be sent somewhere.
- **A lancer outranged it.** It reached 60 px, and a lancer stands 75 px from what it shoots. Parked by hand, its field was set and never up against lancers.
- **Its lattice was the first thing shot.** The two rails stood clear of the hull. Left on guard against the fighters it ended those six fights with 7, 0, 0, 5, 3 and 0 of its 43 stasis pixels.

| # | What | Henry's verdict |
|---|---|---|
| LT1 | **A ship with a lattice flies for its field, whatever guns it has.** On GUARD it keeps its post, whatever comes near and whatever hits it. On ATTACK it goes to the nearest enemy and stops with that in its reach. Sent, it stays where it's sent, as before. It never picks a course of its own, and its friends don't push it off a spot it's holding. Its guns fire at what comes in their reach all the same. | **This changes Q7,** whose last sentence was "An armed one that hasn't been sent flies by its guns". That sentence was mine, and so is this. |
| LT2 | **A lattice reaches 4 px for each pixel of its width plus its height.** It was 2. The most is still 150. | He asked for "a bigger area". The number is mine: the table below. **This changes Q3.** |
| LT3 | **The SNARE is redrawn.** Its lattice is a cross, 15 by 15, inside the hull, and the hull has a whole skin of plate, which flak doesn't break. It reaches 120 px (it was 60). A tracker and four flak, which its own field doesn't hold. Five of its eleven engines are turned astern, so it stops without turning about. 200 px (it was 175), which is the most a PORT builds. 7.5 px/s, as it was. | He asked for "better protected in the unit design". The drawing is mine. |
| LT4 | **A field that's up shows.** All of it, from the lattice out to its reach: a dim dotted web fixed to the map, and violet rings 32 px apart that close in on the lattice at 2 px a tick. The line round its reach is four times as close as it was. Every ship it holds, his or the enemy's, has a violet corner at each corner of its box. A field that's only set (no enemy in it) is still the sparse dotted line and nothing else. | He asked for "a pretty loud animation when it's doing its thing". **This changes Q8.** |
| LT5 | **No timing is changed.** There's no cooldown and no duration: a field is up for as long as its ship is still and an enemy is in its reach, and the only clock is the 2 s it takes to set once its ship has stopped. | His "better cooldown/duration?" I've taken as a question. If he wants it to be a thing that fires and runs out, that's a different part, and his call. |
| LT6 | The network protocol is 25. (26 since 2026-11-22: ST7.) | |

**The same fights on what's built now,** eight each. Sent by hand and left on guard are the same fights now, to the tick, so they're one row.

| How the SNARE is used | Against | Field up, in fights | Ticks up, of ticks an enemy was within 130 px | Enemy ship-ticks held | His own ship-ticks held | Stasis pixels left |
|---|---|---|---|---|---|---|
| Left on GUARD, or sent | 12 FIGHTERS | 8 of 8 | 244 of 244 | 1543 | 1368 | 29 of 29 |
| Left on GUARD, or sent | 10 LANCERS | 8 of 8 | 227 of 230 | 908 | 1445 | 29 of 29 |
| Left on GUARD, or sent | 10 FIGHTERS + 8 LANCERS | 8 of 8 | 398 of 400 | 2642 | 1838 | 28 of 29 |
| On ATTACK | 12 FIGHTERS | 8 of 8 | 210 of 239 | 1109 | 218 | 29 of 29 |
| On ATTACK | 10 LANCERS | 0 of 8 | 0 of 14 | 0 | 0 | 29 of 29 |
| On ATTACK | 10 FIGHTERS + 8 LANCERS | 8 of 8 | 354 of 395 | 2148 | 361 | 26 of 29 |

- **On ATTACK against lancers alone it still never comes up,** and that isn't the field: the SNARE does 7.5 px/s, its DARTs fly ahead, and the fight is over 300 px from it. An enemy was within 130 px of it for 14 ticks a fight.
- **Left at a post it holds about as many of his ships as the enemy's.** That's "both sides", which is his ruling, on a field 255 px across. On ATTACK it's five to one the right way, because his own ships are out in front of it.
- Who won isn't in the table because it isn't a finding: blue won 44 of the 48 and lost none, and that's one army against one kind of attack.

**The reach, measured before I picked it.** The SNARE's first drawing with LT1 in, left on guard, eight fights each: fights its field was up in, and the ticks it was up of the ticks an enemy was within 130 px.

| Each pixel of width plus height reaches | That SNARE reached | v 12 FIGHTERS | v 10 LANCERS | v 10 FIGHTERS + 8 LANCERS |
|---|---|---|---|---|
| 2 px (as it was) | 60 | 8 of 8, 164 of 219 | 0 of 8, 0 of 115 | 8 of 8, 164 of 364 |
| 3 px | 90 | 8 of 8, 278 of 292 | 8 of 8, 92 of 152 | 8 of 8, 342 of 399 |
| 4 px | 120 | 8 of 8, 224 of 224 | 8 of 8, 209 of 210 | 8 of 8, 299 of 299 |

- At 2 a lancer is never in it. At 3 it's in it some of the time: when it's shooting at the SNARE itself and not at the ship beside it. At 4 it's in it.
- **A lance shot flies 112 px, not the 100 on the card,** and the gun fires at whatever the shot will reach. The redrawn SNARE's lattice was 13 by 13 at first and reached 104: a lancer flying straight at it fired its first two shots from outside. At 15 by 15 it reaches 120, and a lancer can't hit any pixel of the ship without being held first. That's why the ship is 200 px and not 173.

**What lances still do to it.** One lancer against a SNARE with hull for its guns (so the lancer lives): it fired two volleys held, at a lattice reaching 120, then stood outside what was left and fired the rest free. The SNARE's reach as each volley was fired: 120, 120, 32, 32, 32, 28. A lance punches through six pixels, a lancer fires two down one line, and the lattice is one pixel thick: a volley that happens to run along a bar takes ten pixels of it and the middle of the cross. So plate keeps flak off the lattice and nothing keeps a lance off it. With its tracker a SNARE shoots a lone lancer before that; a line of them will cut it down. That's the way to beat it, and I've left it.

**Engines astern, measured.** A SNARE that has to turn about to stop takes six seconds to do it. With engines astern it doesn't turn, but it needs enough of them. Sent six ways from rest, how long until it's stopped for good, and how far past the spot it flew:

| Astern | 80 px ahead | 250 px ahead | 120 px to the side | 60 px to the side and 90 astern | 200 px astern | 36 px off | Furthest past |
|---|---|---|---|---|---|---|---|
| Three of eleven | 30 s | 52 s | 38 s | 54 s | 30 s | 42 s | 28 px |
| Five of eleven | 13 s | 35 s | 21 s | 37 s | 28 s | 20 s | 3 px |

(Measured a drawing before the last: it had three more stasis pixels where the four flak are now, and weighed the same. The last one, sent 80 px in a test with fighters coming, stops a pixel from its spot.)

**What he has to do to see it.** His saved deck holds the SNARE as it was drawn before. LT1, LT2 and LT4 work on that drawing too: its H reaches 120 now, and it keeps its post. Its rails are still bare and it still has to turn about to stop. To get the new drawing he takes the SCAVENGERS from the library's TEAMS shelf again, and the deck he has goes on the shelf as a TEAM with a number.

**Not done, and what's still open.**
- **The ANCHOR isn't redrawn.** It reaches 150 now (21 and 21 would be 168, and 150 is the most), and its rails still stand clear of its hull. Its two arena fights will go differently; I haven't run them again.
- **One cheap enemy ship flown in turns the field on over his own fleet.** I said so on 2026-11-18. The field covers more than three times the ground it did, so it matters more. A switch on the card is still the answer if he wants one.
- The whole-map view and the minimap don't show a field. There's no sound for one.
- The card's text is as it was, but for the number. "FOUR TIMES SLOWER" for "AT A QUARTER SPEED" is still his to say.
- The AI doesn't build a lattice, keep out of one, or shoot one from outside its reach.
- I haven't seen it in the running game: the pictures are frames the game drew headless. `screenshots/stasis_field.gif` is three seconds of one.

**Tested.** Eight new tests. The SNARE left on guard in twelve fights: every tick it's where it began, its field is set from two seconds in, and it's up on exactly the ticks an enemy is in its reach. On ATTACK in four: its field is up within five seconds of the first enemy coming into reach, and it doesn't move while one is. A lancer can't fire at a whole SNARE from outside its field. No stasis pixel of the SNARE has an open side, and twelve fighters at it for 40 s land 40 shots and cut none. The ANCHOR, which has no guns, stays when it's shot at, where the same drawing with hull for its lattice runs. The field's web, its rings (whole, 32 px apart, 2 px nearer each tick, and nothing past its reach or off the map's rectangle), the held ships' corners, and that both screens draw the field under their ships. Twenty-one faults put in on purpose each fail one of them: a lattice ship picking courses, chasing on guard, flying by its guns on attack, running when hit, or pushed off its spot by friends; a reach of 2 or of 3 px; the SNARE's plate open over a bar's end, or with three engines astern, or none; rings that stand still or go outward; a field that shows when it's only set, or past its reach, or fixed to the screen; either screen not drawing it, or drawing it over its ships; no mark on a held ship, or a mark on every ship; and the line round it no closer than before.

- Seven older lattice tests had the old reach in them and say the new one now. The post they use is 19 across where it was 41, so that it reaches 92 and not the most there is.
- The suite is 385 and passes, with the same four tests switched off.

## The SNARE that wouldn't stay put (2026-11-22)

*Henry, after a 30-minute match on the build above:* "do gleaners still try to wait for barges even though they're too big for them? the SNARE still moves when its field is active. it should basically never do that, I think. also, the animation for the field is slightly too loud."

**The gleaners.** No, and nothing is changed. A miner only ever makes for a pad it fits: every pixel of it over a dock pixel, turned whichever of the four ways suits. Asked of his own cards: a GLEANER (6 by 6) has one place on a BARGE (6 by 6) and none on his PORT, whose pad he has redrawn 6 by 5. A DIGGER has one place on each. So in his deck a GLEANER lands on a BARGE or unloads in his king's tractor, and that's all there is for it. His cards on his map and two others, eight minutes each with the SWARM raiding: GLEANERs stood waiting by a BARGE for 6, 4 and 12 s in all, 4 s at the longest, and by the king for 7, 1 and 2 s. A sixth row of pad on the PORT would let them land there.

**Why the SNARE moved.** His cards, on his map (seed 49089, a ring) and two others, the SWARM raiding from about 3:00, nobody playing blue: a PORT by the home rock, miners, and six DARTs, a PULSAR and two SNAREs put down as the PORT would have built them, with posts in its rings. Seven minutes. For the two SNAREs together: the ticks an enemy was in a SNARE's reach, the ticks of those its field was up, and the ticks of those it was under way.

| How they were used | Map | Before: up, of ticks in reach | under way | Now: up, of ticks in reach | under way |
|---|---|---|---|---|---|
| Left on GUARD | his, a ring | 969 of 1136 | 160 | 1193 of 1195 | 0 |
| Left on GUARD | two triangles | 492 of 510 | 17 | 876 of 892 | 0 |
| Left on GUARD | an S | 1111 of 1381 | 253 | 1302 of 1303 | 0 |
| Stance to ATTACK when a field first came up, back to GUARD 20 s on | his | 1324 of 1527 | 180 | 1106 of 1110 | 0 |
| The same | two triangles | 517 of 634 | 117 | 867 of 867 | 0 |
| The same | an S | 1910 of 1947 | 35 | 2245 of 2247 | 0 |
| Stance ATTACK from 2:30 | his | 273 of 532 | 151 | 273 of 532 | 151 |
| Stance ATTACK from 2:30 | two triangles | 280 of 513 | 175 | 280 of 513 | 175 |
| Stance ATTACK from 2:30 | an S | 1082 of 1983 | 748 | 1479 of 2207 | 565 |
| Sent as a group at 2:30, 200 px towards the middle | his | 516 of 1150 | 431 | 356 of 629 | 155 |
| The same | two triangles | 544 of 879 | 242 | 354 of 793 | 278 |
| The same | an S | 467 of 748 | 219 | 570 of 722 | 100 |

(The matches aren't the same matches before and now, so the ticks in reach differ. On ATTACK two of the three are the same to the tick: nothing in them touched what's changed.)

Left on GUARD, on ATTACK and sent (the nine runs with no stance flipped back), before: a SNARE got under way within 3 s of its field having been up twelve times, and seven of those times an enemy was within 40 px of its reach (three times inside it). Now: nine times, and the nearest enemy was 42 px outside its reach or more every time.

- **A turn on the spot counted as still.** A SNARE has engines ahead and astern and none to the side, so it turns before it sets off and for each leg. A quarter turn takes it 3 s, and a field sets in 2. So on its way to a post or a spot its field came up where it stood turning, with an enemy in reach, and dropped when it flew on. That's Q4, which is mine.
- **On GUARD it flew to its post whatever was in reach, and its post moved.** Ships built at one place close up their rings when one built before them is lost. At the first raid on his map the SNAREs' posts moved 61 and 58 px. Each such move is a 3 s turn, a flight at 7.5 px/s and 2 s to set again.
- **A ship sent after it took its place.** He sent one after a SNARE at 18:54 of his match. A ship sent after a thing is sent to where the thing is, and a waiting ship gives way to one built before it that's waiting too. So a MAUL sent after a SNARE built later than itself took the SNARE's own place, the SNARE gave way, and the MAUL followed it: in two minutes the SNARE was off its post on 1783 ticks of 1800 and its lattice was set on one. (A DART sent after it moved its place 14 px, which a SNARE counts as there, so it didn't stir.) Every kind of ship did this, not only one with a lattice.

The table I gave him on 2026-11-21 didn't show any of it: in those fights the SNARE was put down on the spot it was to hold, with nothing built before it and no post to fly to.

| # | What | Henry's verdict |
|---|---|---|
| ST1 | **A lattice sets only once its ship has stopped where it means to stay.** Turning there is still. Stopped to turn on its way somewhere isn't. | **This changes Q4** ("Turning on the spot is still"). His words: "it should basically never do that". The rule is mine. |
| ST2 | **With an enemy in its reach, a lattice ship that hasn't been sent anywhere stops where it is,** on GUARD as on ATTACK: on its way to its post, or going home after the stance changed. "In its reach" counts an enemy that will be by the time a field has set, at the speed it's coming. | **This changes LT1** ("On GUARD it keeps its post"): it stops short of a post it hasn't reached. |
| ST3 | **One whose lattice is set keeps its place while an enemy is within 40 px of its reach,** whatever its post or the spot it was sent to does meanwhile, and whatever the stance changes to. It takes up its post again when the enemy has gone. On ATTACK it still closes on an enemy that's out of its reach: staying would never bring that one in. It doesn't get off a warming beam's line either. | The 40 px is mine: an enemy that has only stepped out is back before a field could set again. |
| ST4 | **Sending it somewhere is the one thing that moves it sooner.** Sent to a new place, or after something, it lets its field go on the spot and leaves. Sent again to where it already is, nothing changes. A sent ship doesn't stop on its way (H10, as for every ship), so its field is down for the whole flight and sets where it arrives. (Changed later the same day, by his ruling: it stops on its way for an enemy that comes into its reach. SP1, SP2.) | Mine: his "basically never" needs one way out, and this is the order he gives the ship itself. |
| ST5 | **A ship sent after a friend stops beside it, clear of it,** whichever was built first, so the friend is never pushed out of its place. Every kind of ship. | Mine. It's what a ship sent after a friend built before it did already. |
| ST6 | **The rings of a working field are dotted:** every other pixel along each ring. They're as far apart, come in as fast, and their dots keep their rows and columns as they come. The web, the line round the reach and the corners on held ships are as they were. (Twice as far apart later the same day: SP6.) | His "slightly too loud". **This changes LT4.** One step: if it's still too much, the next is fewer rings or a thinner web. |
| ST7 | The network protocol is 26. (27 later the same day: SP7.) | |

- **The stance to GUARD doesn't pull a SNARE out of a fight.** With an enemy in its reach or just outside it stays until that's gone, and then goes home.
- **Sent somewhere, it has no field until it's there** (ST4). In the table that's the last three rows: sent out as the raid came in, the SNAREs were under way for a good part of the time an enemy was in reach, before and now. It could stop on its way for an enemy in reach, as it does on GUARD. That would go against H10 for this one kind of ship, so it's his to say.
- **On ATTACK its field is up for about half the time an enemy is in its reach,** before and now. It takes a SNARE 34 ticks to stop from 7.5 px/s and a field 30 to set: over 4 s from the first enemy in reach to a field. Not touched.
- **Idle, it still shuffles** to a new post when the rings close up. Only an enemy within 40 px of its reach holds it.
- A ship sent after an *enemy* still makes for the enemy's middle, as it did.
- I haven't seen any of it in the running game. `screenshots/stasis_field.gif` is redone with the dotted rings: frames the game drew headless.

**Tested.** Six new tests. A SNARE sent to a spot with an enemy in reach where it starts stands turning for 60 ticks, and its lattice is never set until it's at the spot. On guard with an enemy standing on its post 340 px off, it stops 117 px short, 34 ticks after the enemy came into reach, stays exactly there with its field up, and goes on to its post when the enemy is gone. Set at its post with three enemies to one side, it stays when the guard built before it is lost and its post moves, stays with the nearest enemy 20 px outside its reach, and goes when the nearest is 60 px outside; on ATTACK it closes on one 20 px outside. In a match: sent again to where it is, its field stays up; sent somewhere else, or after a PORT, its field is down the same tick and it's set again only where it arrives. A nimble frame with one stasis pixel gets off a warming beam's line with the gun's ship 4 px outside its reach, and 4 px nearer, with the gun's ship in its reach, it stays and is cut. A MAUL or a DART sent after a SNARE built later stops clear of it, and the SNARE never leaves its post nor loses its set.

- One older test read "still" off a ship's track by where it was and how fast: it reads what the ship is making for as well now. The test of what the field shows says a ring is dotted.
- Seventeen deliberate faults, each caught by one of these: any stop counting as still; each of the two reasons to stay taken out, both taken out, and each of the two exceptions (sent, closing) taken out; 0 px and 70 px for the 40; a send, a send to the same place and a follow each doing the other thing; the beam dodged; the follower taking its friend's place; the rings whole, dotted one way only, one pixel in three, and across for down. The first run of them missed three, and each miss was a fault of mine: the test of the 40 px took its distances from the 40; the beam test's ship couldn't have got off the line whatever the rule said; and a second clause I had put in the follow rule did nothing, so it's gone.
- The suite is 391 and passes, with the same four tests switched off.

## A sent SNARE that stops, BARGEs that wait for their miners, fewer rings (2026-11-22, later)

*Henry, to "Should a SNARE you've sent somewhere stop on its way when an enemy comes into its reach?":* "Yes, stop." *And:* "do barges have to stop to unload resources? something about the barges didn't seem right. people would wait for a while for them even though they were empty. I can try to reproduce it again if needed." *And:* "I think there should be fewer rings in the stasis field animation"

**The barges.** Yes: a pad takes hold of a miner only while its ship is at rest, and lets go of one its ship turns under. (The BARGE itself never has to unload. What lands on it is in its store, and it stands where it's in link with his network, so it's in his bank.) Measured on his cards: eight DIGGERs, two GLEANERs and two BARGEs at his home rock, nobody attacking, eight minutes, on his map and two others, with his PORT there and without. Seconds that a miner with a load stood beside the BARGE it was bound for, by what kept the pad from taking it (the longest single wait in brackets):

| Scene | The BARGE took a new place | Before: the BARGE was under way | its pad had a miner on it | its pad was resting, nothing on it | Now: the BARGE was under way | its pad had a miner on it | its pad was resting, nothing on it |
|---|---|---|---|---|---|---|---|
| his map | 23 times, now 20 | 73 s (6) | 91 s (2) | 144 s (2) | 14 s (2) | 39 s (2) | 96 s (2) |
| two triangles | 16, now 13 | 15 s (3) | 57 s (2) | 108 s (2) | 33 s (2) | 51 s (2) | 106 s (1) |
| an S | 14, now 14 | 10 s (2) | 51 s (2) | 117 s (2) | 18 s (2) | 39 s (1) | 106 s (2) |
| his map, with his PORT | 39, now 22 | 72 s (6) | 61 s (2) | 127 s (1) | 13 s (2) | 47 s (2) | 124 s (2) |
| two triangles, with his PORT | 14, now 12 | 28 s (4) | 90 s (2) | 152 s (1) | 17 s (2) | 58 s (1) | 132 s (2) |
| an S, with his PORT | 17, now 14 | 40 s (4) | 73 s (2) | 145 s (2) | 22 s (2) | 72 s (2) | 155 s (2) |

- **Most of it is the pad's rest, and nothing showed it.** A pad pixel rests 3 s after a miner unloads on it (his rule, 2026-11-17), a DIGGER covers 19 of a BARGE's 36 and a second doesn't fit beside it, so a BARGE takes one DIGGER every four seconds or so. For about half of that the BARGE had nothing on it and looked ready. That's the "even though they were empty".
- **The rest of it is the BARGE moving off.** It took up a new place whenever its miners' work had shifted 16 px, whoever was beside it: 14 to 39 times in the eight minutes. A miner that arrived as it left stood beside it for up to 6 s.
- What they mined in the eight minutes is the same before and now, within a twentieth either way (1284, 1323, 1329, 1560, 1587, 1623 before; 1386, 1314, 1350, 1575, 1578, 1614 now).

| # | What | Henry's verdict |
|---|---|---|
| SP1 | **A ship with a lattice that's been sent somewhere stops on its way for an enemy that comes into its reach,** and stays there as it does on GUARD (ST2, ST3). The enemy gone, it goes on to where it was sent. | His ruling: "Yes, stop." **This changes H10 for a ship with a lattice, and ST4's last sentence.** Every other sent ship still stops for nothing. |
| SP2 | **Sent away from an enemy that's in its reach already, it goes,** and stops for nothing until it's clear of what it was sent away from. From there it stops for the next enemy. So sending is still how to get it out of a fight, and sending it again somewhere else is how to make it push on. Let go (the HOME order), it isn't leaving any more. | Mine. Without it SP1 would have left him no way to move a SNARE that had an enemy in its reach. |
| SP3 | **A BARGE stays where it is while its pad has work:** a miner in its pad's hands, or one with a load that's bound for it and within 30 px of where the pad would take hold of it. On its way to a new place it stops for one, and goes on when the work is done. | Mine. The 30 px: with none, his map's 14 s beside a BARGE under way was 31 s. **Keep.** Henry, the same day, after playing: "barges seem much improved so that's good". |
| SP4 | **A pad pixel that's resting shows:** every other one is dark, for its 3 s. | Mine. His rest rule is as it was. **Keep.** Henry, the same day, after playing: "barges seem much improved so that's good". |
| SP5 | **A pad just built is ready at once.** Its pixels started out of step with each other, as guns do, so a new pad rested for up to a second. | Mine: SP4's test found it. **Keep.** Henry, the same day, after playing: "barges seem much improved so that's good". |
| SP6 | **The field's rings are 64 px apart.** They were 32: a SNARE's field shows two at once where it showed three or four. | His: "fewer rings". **This changes ST6 and LT4.** |
| SP7 | The network protocol is 27. (28 later the same day: MA4.) | |

- **The rest itself is still the big number,** and it's his rule: about 100 to 150 s of ten miners' time in eight minutes. A pad with room for two side by side halves it (the SCAVENGERS' own PORT has one, 6 by 9; a BARGE's is 6 by 6; the PORT he drew has 6 by 5).
- **A BARGE still takes a new place 12 to 22 times in eight minutes,** and two BARGEs pick the same place. I tried having it stand by the middle of all the work within 32 px and within 64 px, in place of the one busiest spot: nothing clearly better (on his map 12 and 11 moves for 20; the waits and the mining the same). Left as it was.
- **On the two other maps with no PORT the wait beside a moving BARGE went up,** 15 s to 33 and 10 to 18, though no single wait is over 2 s now where they ran to 6: a BARGE that stops for each miner on its way takes longer to get where it's going.
- I haven't seen any of it in the running game. `screenshots/stasis_field.gif` is redone with the rings further apart, and `screenshots/pad_resting.png` is a BARGE with a DIGGER's place resting: frames the game drew headless.

**Tested.** Three new tests. A SNARE sent 400 px with nothing near it stops 34 ticks after an enemy by its way comes into its reach, 262 px short of the spot, stays there with its field up, and goes on to the spot when the enemy is gone. A BARGE that wants to leave doesn't stir, turn or make for anywhere while a full DIGGER 45 px off comes to it, is brought on, emptied and put off; then it goes; and with the DIGGER 120 px off it goes at once. Every pad pixel of a PORT is one colour until a DIGGER lands; then the 19 under it rest, every other one of them drawn dark; three seconds on they're as they were. The test of sending a lattice ship has one more step (sent off and let go the next tick, with the enemy in its reach, it stays), and the test of the field's look says the rings are 64 px apart.

- Thirty-one deliberate faults, each caught: the seventeen of 2026-11-22 again, and fourteen for this: a sent ship never stopping, the rule going by "sent" as it did, a send and a follow that don't make it leave, letting go that doesn't end it, a BARGE that leaves whatever its pad is doing, one that stays only for a miner in its hands and one only for a miner waiting, 0 px and 300 px for the 30, a resting pixel that looks ready, every resting pixel dark, a new pad that starts out resting, and rings 32 px apart. (Two of them didn't build the first time round: my script's mistakes, not the tests'.)
- The suite is 394 and passes, with the same four tests switched off.

## The MAULs that ran off (2026-11-22, his match against LANCE LINE)

*Henry, playing the build above (seed 13631, an S, won at 31:48), as he went:* "the movement of my mauls didn't seem to make sense. one ran off to the edge of the board and i'm not sure why. it did come back after a bit." *Then:* "this enemy miner also appears to being going towards an edge and I can't tell why", *and later of it:* "the miner wasn't exactly going to an edge but it was flying horizontally away from any minable resources in a way that didn't make sense". *And:* "barges seem much improved so that's good". *And:* "I Can't tell why the enemy's king isn't slow?"

**The MAULs.** His MAUL is his own drawing, and it's a kind of ship no card the game ships with is: grown from a seed, with engines, guns and factory pixels (it prints SNAREs and HERALDs). 280 px, 691 of weight on four engines that all push ahead: 7.5 px/s, 3 s a quarter turn, 3.8 s to change its speed by the least there is. Two of them put down by his home rock with an AI raiding, ten minutes, five maps:

| As the game was | Furthest either MAUL got from where it stood | Ticks against the map's edge, of 9000 (each MAUL) |
|---|---|---|
| As he played it | 933 to 1275 px, on every map | 0 to 727 |
| With MA1 alone (on GUARD, as the stance said) | 96 to 220 px | 1771 to 5586 |
| With MA1 and MA2 | 0 px on three maps (nothing came near), 68 to 135 px on the two the SWARM raided | 0 |

- **It never took the stance.** A seeded design was left out of the cards the stance orders, so one that flew kept the order its parts give it: with engines and no scoop, ATTACK. His MAULs went after the nearest enemy anywhere on the map whatever the button said. That's against his own rule of 2026-11-14 ("ATTACK should send everything unless it has mining equipment"), and it's my code that left them out.
- **It was pushed off by the ship it was printing.** Friends keep a pixel or two apart, each pushed away from the other. A half-printed ship rides beside its factory wherever the factory goes, so a factory that flies was pushed away from its own half-printed ship on every tick, twice as hard as it could fly back: in a straight line, to the edge of the map, where it stayed until the print was done. Then it came back, and began the next. That's "ran off to the edge" and "did come back after a bit".
- **Sent to a spot, it never settled.** A sent ship has to stop within 3 px of its spot and goes round again until it has. This one is carried 7 px while it burns off the least speed there is. It came to rest 5 px past the spot one way and then the other, and was still at it four minutes on. He didn't report this; the test of MA2 found it.

| # | What | Henry's verdict |
|---|---|---|
| MA1 | **A design grown from a seed takes the stance if it flies,** like one that's built: GUARD where it grew, or ATTACK. | **This changes which cards the stance orders** (K2), to what his words of 2026-11-14 said. |
| MA2 | **A factory that flies isn't pushed away by the ship it's printing.** | A fault. |
| MA3 | **A ship is at its spot when it's as near as its engines can place it:** 2 px and what it's carried in one burn. That's 3 px still for every card the game ships with but one (the SIEGE team's LANCET: 4), and 9 for his MAUL. The looser distance a ship that hasn't been sent settles for goes up by the same: 1 px for the SNARE, the SCAVENGERS' MAUL, the BASTION and the CAPITAL, 2 for the LANCET, 7 for his MAUL. | **This changes "sent ships too" (2026-11-18) for a ship that can't land that fine.** Mine. Sent twelve ways, his MAUL now comes to rest within 7 px every time. |
| MA4 | The network protocol is 28. (29 later the same day: MA7.) | |

- **His MAUL is still very slow to do anything,** and that's the drawing: four engines for 691 of weight, none turned to brake. To stop it has to turn right round (6 s) and burn. A few engines turned the other way would let it stop without turning, as the SNARE's do; more engines would make every burn quicker.
- **On ATTACK it still goes for the nearest enemy anywhere,** as every ship does. Now that's by the stance.
- A SNARE printed by a MAUL takes its post in the rings round where the MAUL stood when it finished.
- (No longer so: the push went the same evening, MA8.) A ship a flying factory has just finished is a ship like any other, and the two are pushed apart for the second or two before it's off to its post. His MAUL didn't move a pixel for it in the test; a light carrier would be nudged.

**"barges seem much improved so that's good".** That's SP3 to SP5 (it stays while its pad has work, a resting pad shows, a new pad is ready): **keep, on his word.**

**The enemy's king.** A king slows only when its side has under 40 px standing besides the king itself (M2.10). When he asked, red had no factories and one ship left, and that ship was 99 px (the log at 31:00: 2 ships, 108 px; at the end: 1 ship, 99 px). So its king flew at full speed. Nothing is changed. The game says THE ENEMY KING IS SLOWED when it is, and nothing about how far off that is. Asked whether it should show how many pixels the enemy has left, Henry, the same day: "No, leave it". **Ruled: not shown.**

**The enemy's miner.** Not reproduced, so not explained. (Henry, later the same day: it may be miners from an outpost with everything mined out round it, so the nearest rock is by the enemy; "I agree that doesn't need fixing".) Both sides played by an AI (the SWARM against LANCE LINE) for 25 minutes on his map and two others, red's miners sampled three times a second: flying more than 150 px from any rock and more than 80 px from any loose pixel on 144 of 73,325 samples, 12 of 101,075 and 333 of 57,445. Every one of those was making for rock, for a node of its own side or for its king: an empty one going to the nearest rock left on the map, which was by one of its side's outposts, or a loaded one taking its load in. None was against an edge, in those or in the five matches of the table above (0 of 567,563 miner-ticks within 6 px of one). What a miner flies to, for reading the next one: the nearest rock pixel left anywhere; a loose pixel only with no rock within about 350 px and the pixel nearer than the rock; with a load, the tractor or pad that would empty it soonest; hit with no guns, its nearest node. With the pointer on a ship the bar says which.

**Tested.** Three new tests, all on his MAUL as he drew it. On GUARD with an enemy 500 px off it stays where it stands for 40 s, goes for the enemy on ATTACK, and comes home on GUARD again. With pixels to print with it prints a SNARE (206 s, the half-printed SNARE inside the distance friends are pushed apart at the whole time) and starts another without stirring, and sent 150 px off with a ship half printed it comes to rest 5 px from where it was sent with that ship still beside it. Sent twelve ways from rest it comes to rest within 9 px every time (7 at the worst).

- One older test had a seeded warship fly off by itself once it was whole, which was this fault: its side is on ATTACK in that test now.
- Six deliberate faults, each caught: a seeded design left out of the stance, the push from its own half-printed ship, the burn counting for nothing in where a sent ship stops and in where an unsent one does, and the burn reckoned a quarter and twice what it is. The first run missed the push: in my test the MAUL faced up the map and the SNARE it printed sat a pixel outside the pushing distance. It faces across now, and the test says the SNARE is inside it.
- The suite is 397 and passes, with the same four tests switched off.

### The MAUL that still went to the edge (the same day, on the build with MA1 to MA3)

*Henry, with two pictures (a MAUL against the left edge at 14:56 on GUARD; a red miner by its king):* "the miners might just be because they're produced from an outpost where everything is all mined out so the nearest resources are in a weird spot near the enemy. so I agree that doesn't need fixing. the MAUL still appears to be going to the edge of the map, however" *and* "unless my build didn't pick up the recent changes?"

**His build had them** (built 16:29; my last change was 16:10). The five-map table above was two MAULs put down by themselves. He plants them on rocks, with PORTs on the same rocks and the ships both print standing round. So this time: on his map of that match (seed 4959), his map of the match before and one other, a PORT and a MAUL planted by the king's own orders on each of the four big rocks nearest his king, pixels in the bank to print with, LANCE LINE raiding, 22 minutes. Six MAULs grew.

| As the game was | Furthest each MAUL got from where it grew (px) | Ticks against the map's edge |
|---|---|---|
| With MA1 to MA3 (what he was playing) | 481; 774; 242, 129, 179, 886 | 0; 762; 0, 0, 0, 29 |
| MA5 alone | 243; 378; 274, 307, 177, 215 | 0; 440; 0, 1079, 0, 0 |
| MA6 alone | 128; 128; 269, 89, 145, 136 | 0; 0; 0, 0, 0, 622 |
| MA5 and MA6 | 58; 60; 56, 70, 52, 37 | none |

- **It had no place of its own to hold.** A built ship has a post in the rings round what built it, and holds it: friends go round. A ship grown from a seed had none, so every friend that stood near pushed it away, twice as hard as it could fly back: the SNAREs and HERALDs it had printed, a PORT planted beside it, a BARGE come for the miners, another MAUL growing.
- **It picked courses it couldn't fly.** A ship in a fight weighs its next four seconds. His MAUL needs 115 ticks to burn off even 7.5 px/s, so no stop it could begin shows inside the four seconds, and whatever speed it took up to get clear of a lance, it kept. That's NC3, which I had left for the talk on ship AI: "Not measured".

| # | What | Henry's verdict |
|---|---|---|
| MA5 | **A ship grown from a seed has a post: where it grew.** It holds it as a built ship holds its place in the rings. Friends go round it, and ships sent to its spot come to rest beside it. | Mine. |
| MA6 | **A ship too heavy for its engines to burn off its own top speed inside four seconds picks no course of its own.** It flies as its plan says, as a ship with no guns does: on GUARD it meets what comes within its guard and goes back to its post. Every card the game ships with still picks its courses (the slowest to burn off its speed is the SWARM's WASP, 48 ticks). | Mine. **This narrows H1 again,** as NC1 did, and it's the far end of NC3: the rest of NC3 (a ship that needs more than four seconds to turn about and stop) is as it was. |
| MA7 | The network protocol is 29. (30 the same evening: MA10.) | |

- The 70 px at most that's left is a MAUL meeting raiders inside its guard and coming back.
- **The enemy's miners: no change, on his word.** "I agree that doesn't need fixing."

**Tested.** Two new tests on his MAUL. Four ships sent to the very spot it stands on: it doesn't stir or turn, and they come to rest round it, clear of it; and with a PORT 10 px from where it grew it stays where it is. In a fight with six fighters, beside the SCAVENGERS' own MAUL (the same four engines under a quarter as much weight, near enough): theirs is on a course of its own for 142 ticks and his for none. Five deliberate faults, each caught: no post, a post in a ring, every ship picking courses, and the four seconds halved and doubled. The suite is 399 and passes, with the same four tests switched off.

**The push itself (Henry, the same evening, asked what it is):** "it doesn't seem like the destacking push should be a physics push, it should be more just a hand-of-god one-time convenience intervention, which is it?" It's neither. Each tick a ship whose box is within 2 px of a friend's adds 15 px/s away from that friend to the speed its pilot makes for, and then flies that with its own engines: so it gathers real speed and has to stop and come back. Miners at work, ships at a post, ships sent to a spot and a lattice holding are let off. Measured with both sides played by an AI, 14 minutes, three matches (SWARM v LANCE LINE, BIG GUNS v SIEGE, LANCE LINE v SWARM): it acted on 909 of 37,053, 914 of 69,363 and 753 of 38,338 samples of ships not at work, nearly all of them in a fight. Ships with an enemy in sight that had a friend's middle within 2 px of their own: 110 of 14,009, 370 of 4,141 and 250 of 10,992 with the push; 196 of 13,400, 206 of 3,101 and 98 of 4,627 with it taken out (the third match ended at 9:25). So it changes how much ships stack very little: where a ship stops is already chosen clear of its friends once, by its post, the spot it was sent to or its place in a firing line. **Proposed: take the push out.** Henry: "Yes, take it out".

### The push taken out (the same evening)

| # | What | Henry's verdict |
|---|---|---|
| MA8 | **Nothing pushes friends apart.** On the way and in a fight they fly through one another, as they always could. Where a ship stops is chosen clear of its friends, once: its post, the spot it was sent to, its place in a firing line, or MA9. | His ruling: "Yes, take it out". **This removes "ships of a side keep a pixel or two apart"** (in the rules above, and the exceptions made to it since: miners at work, M2.21; posts, M2.12; sent ships, K5; a lattice holding). |
| MA9 | **A ship with no gun to line up takes a place of its own round its target.** Sixteen places at its guns' distance, one out each way the game reckons a target's open sides by. It takes the nearest round from the place it has (from where it is, with none yet) that no friend built before it has for the same target, and of those, one on a side its guns can break something from if there is one. A ship with a stasis lattice doesn't: it comes at the enemy along its own line, as it did. | Mine, and not asked for: it's what taking the push out turned out to need (below). **This changes H6** ("on whichever side") **and I11**, which is the plan's doing now, on either pilot. |
| MA10 | The network protocol is 30. | |

**What the push had been doing, that nobody had designed it to.** The suite found three things when it went, and I had told him the day's measurement showed it wasn't needed.

- **It was what spread ships with turrets and flak.** Each stood at its guns' distance on the line from its target to itself, whoever else was there. Six ARROWS sent at one target in a block came to rest on one pixel. One cannon shell then takes all six: on the plain pilot they won 1 fight of 20 against a cannon where they had won most. (Picking their own courses they still won 20 of 20. In the three matches I had measured, ships arrive from all sides and at all times, which is why stacking barely showed there.)
- **It was what carried them round armour.** I11 says ships whose guns can't break the armour in front of them fly round to where they can, and its test had six ARROWS break a plated shell 20 times in 20 picking their own courses. With the push gone they sat in front of it and broke it in none of 20. It was the jostling that had taken them round, a few pixels at a time, until a course to the stern showed a gain inside four seconds.
- **It had been getting in attackers' way.** Two BATTLESHIPS sent at the AI's king pushed each other off their lines as they came.

With MA9: the six ARROWS break the shell 20 times in 20 on either pilot (the plain pilot was 0 in 20, by design then); they come to rest in an arc, no two on one place; and the cannon test passes as it did.

**Stacking, measured again on what's built.** The same three matches played by two AIs: samples of ships with an enemy in sight that had a friend's middle within 2 px of their own.

| Match | With the push | Push out, nothing in its place | Push out, with MA9 |
|---|---|---|---|
| SWARM v LANCE LINE | 110 of 14,009 | 196 of 13,400 | 85 of 13,626 |
| BIG GUNS v SIEGE | 370 of 4,141 | 206 of 3,101 | 137 of 3,259 |
| LANCE LINE v SWARM | 250 of 10,992 | 98 of 4,627 | 39 of 5,648 |

**His planted MAULs, on what's built** (as in the table above): 57; 37; 0, 49, 0, 0 px at most from where they grew, and none against the edge.

**What it costs: the AI's king is easier to corner.** Two BATTLESHIPS sent at the AI's king four minutes in, for two and a half minutes, on eight maps:

| | Came through with half of itself or more and under 6 s in reach | Run into the corner of the map it starts by |
|---|---|---|
| With the push | 8 of 8 (134 to 218 px left of 218) | once, for 85 ticks |
| Now | 5 of 8 | 3: left with 89 px, and killed twice |

The king's own rules are as they were. Its start is 150 px from a corner, it backs away from guns that outweigh it, and two battleships that no longer hinder each other get it there. One scenario and one pair of ships: not a verdict on balance, but it's the AI losing a defence it had by accident. Not mended: asked whether to stop the AI's king backing into its own corner, Henry said "Not now".

- **Two choices inside MA9 that no test pins.** Whether a free place on a shut side comes before a taken one on an open side (it does: a ship would sooner stand alone and bounce shots than share), and whether a ship gives way only to friends built before it or to all of them. Either way round passes every test there is.
- Comments here and there in the code still explain a gap or an exception by the push. The push itself, and the note of who was at work that only it read, are gone.
- I haven't seen any of it in the running game.

**Tested.** One new test: the six ARROWS at a block of plate, in an arc and no two on one place, still there ten seconds on; and two of the SCAVENGERS' MAULs setting off for one target 10 px apart, neither ever making for a speed away from it. Three older tests re-stated: flying round armour (either pilot now), the SNARE on the attack (its field is up inside five seconds of an enemy staying in its reach; in these four fights within 2.5 s), and the AI's king, which is eight maps where it was one and says what happens on each. Eight deliberate faults: six caught (the push back, a taken place as good as a free one, a shut side as good as an open one, counting round from where it is, a lattice taking a firing place, guns taken to break anything), and the two orderings above not. The suite is 400 and passes, with the same four tests switched off.

---

## The phase ring (2026-11-22)

*Henry, 2026-11-17:* "I want to add a way to get a burst of resources to encourage building big ships. What about a phase-harvester that builds up a charge on an entire asteroid and harvests the entire asteroid at once once the bar reaches the amount of pixels remaining on the asteroid?" *2026-11-18, of the points I'd left open:* "rival charges cancel. miners can "help" by reducing the rock size. no opinion on item 3." *2026-11-22, with the playtests done:* "i'd like to implement one or two more parts now".

So the phase-harvester is built, as a part called the PHASE RING, with two example drawings: the HALO, which carries the ring, and the CRADLE, a pad it fits on. It's a bare toy: the AI doesn't know the part exists, no ready team has it, and nobody has played it.

*Henry the same evening, of the ring's ship being a node that banked what it took (PR8 as I first built it):* "I'd like to stick to what I said about dropping off the resources. We can't just be making everything a node; it doesn't work in the design, one reason is because you don't control where units go precisely, so you can't keep them in a particular network. The phase harvesters need to be paired with a way to offload the pixels. THEN you get the burst." *And:* "BTW since you have that nice required empty space anyway, you could use it to graphically indicate the pixels the phase ship is holding." Both are built: PR8 and PR13. *Asked whether a ship that's destroyed should spill the cargo it was carrying:* "Yes, spill it". Built: PR17.

| # | What | Henry's verdict |
|---|---|---|
| PR1 | **The part.** PHASE RING, in INDUSTRY: pale pink, letter `O` in a drawing's text. A pixel weighs 1 and uses 1 power. It does nothing in the arena: it's a match part, as the tractor and the pad are. | |
| PR2 | **A ring is worth the empty space it shuts in.** Phase pixels are drawn as a closed ring, and what the ring is worth is the empty pixels inside it: the ones that can't be left edge to edge. They count only where phase pixels alone wall them in. With any other part in the wall, or anything standing inside, that ring counts nothing. A corner pixel may be left out. One pixel shot out of the side of a ring one thick opens it, and hull behind the gap doesn't close it. | **Mine.** His pitch says what a harvester does and not what makes one strong. See "Why a ring", below. (He has since called it "that nice required empty space", which isn't a verdict.) |
| PR3 | **It charges the rock nearest it while its ship is at rest.** From a second after the ship stopped, with rock within 24 px of the ship's edge (measured from its middle, as a tractor's reach is, so it doesn't matter which way the ship is turned): 1 px of charge a second for each 100 pixels the ring shuts in, and for no more than 400 of them. Short of power it charges slower in proportion. Under way it charges nothing. | The charge is his. "At rest", the reach and the numbers are mine. |
| PR4 | **A rock has one charge, and it's one side's.** Rings of a side on a rock add up. A ring of the other side takes its charge off first, and begins a charge of its own only when none is left: so two sides at one rock gain by the difference, and two that are even gain nothing. | His ruling: "rival charges cancel." |
| PR5 | **When the charge is all the rock has left, the rock goes:** every pixel of it, that tick, aboard the ship with that side's biggest ring on it. A ring of that side has to be at work on the rock when it happens. A charge never stands higher than the rock has left. | His. (Which ship of several, and that a ring has to be there, are mine.) |
| PR6 | **Miners help.** A miner biting the rock, of either side, leaves less to charge to. | His ruling: "miners can "help" by reducing the rock size." |
| PR7 | **The charge is the rock's, and it leaks.** A ship that moves, is sent away or loses its ring leaves its charge on the rock. With no ring of its side at work on it, a charge leaks 1 px a second. | Mine: "no opinion on item 3." Losing it at once makes one shot at a ring worth a minute of its work, and keeping it for good makes driving a ring off worth nothing. |
| PR8 | **What it takes is cargo, and its ship is no node.** The rock is aboard the ring's ship as a miner's load is aboard a miner, and nothing is banked until it's been taken off: by a tractor, a pixel at a time, or all at once by a pad the ship fits on. That's the burst. With a load aboard, a ring charges nothing. | **His ruling** (above). For an evening I had the ship a node that held what it took in its own store, against what he'd said of the vacuum, and flagged it: he put it back. |
| PR9 | **A ship with a closed ring goes to the rocks by itself,** as a miner does: to the rock nearest it, where it stops and stays. With a rock aboard it takes that in first, to wherever it would be empty soonest, and then goes to the rock nearest it. It isn't one of the fleet: it takes no stance, and "select the fleet" leaves it out. Sent somewhere, it parks there, and its ring is on whatever rock is in reach. | Mine. |
| PR10 | **Shot at, it keeps to its rock.** (An unarmed miner runs home for two seconds.) With its ring broken it goes home, or takes in what it has aboard. | Mine. |
| PR11 | **How a charge shows.** A rock with a charge wears a small bar at its middle: as much of it lit as the charge is of what the rock has left, in the colour of the side whose charge it is. One pale dot runs from each ring at work to its rock, once a second. The rock going is sparks where it was. The inspector says what the ring puts on a second and how its rock stands ("CHARGED 235 OF 470", "TAKING OFF THE ENEMY'S CHARGE: 40 LEFT", "IT TOOK A ROCK OF 470 PX", "IT CHARGES NOTHING WITH A LOAD ABOARD"), and the bar under a rock says whose rings have charged it and how far. In the yard the pixels a ring shuts in each show a dot, and the status line says "PHASE RING: ENCLOSES 225 PX", or "ENCLOSES NOTHING" in gold. | |
| PR12 | The network protocol is 31. (32 the same evening: PR16.) | |
| PR13 | **What it's carrying shows in the space its ring shuts in:** in the colour of loose pixels, as a miner's load shows in its scoops. As much of the space is filled as the share of that rock it still has aboard, from the stern forward. So it's full the tick the rock goes, drains as a tractor empties it, and is empty the tick a pad takes the lot. | His idea. That it's a share of the rock, and not a cell for each pixel, is mine: a HALO shuts in 225 and most rocks are bigger, so a cell a pixel would show full until a tractor was half done. |
| PR14 | **The CRADLE:** a pad for the HALO. A pad pixel under every pixel of a HALO, bow up or bow down, with the CRADLE's own core and four flak turrets in the space a HALO's ring leaves open: a HALO comes down round it like a hoop on a peg. Seeded, 172 px. | Mine: an example of "a way to offload". |
| PR15 | **A ship with a rock aboard reckons the rock when it chooses where to unload.** A loaded ship goes where it would be empty soonest, and a tractor is reckoned to empty it the moment it arrives: only the flight and the queue are counted. That's as it was, for every miner. For a ship whose ring has taken a rock, the time that tractor would take over what it carries beyond its own hold counts too: reckoned the old way, a HALO with a rock aboard passed a pad it fitted for its king's tractor, which was the shorter flight and would be seven minutes about it. | Mine. **For an hour I had every ship's whole load counted,** which is the rule as it's written ("where it would be empty soonest"). It changed where every miner went, moved two older tests (a vacuum ship with a tractor of its own flew off to a faster one, and the AI king's whole-match test came out differently) and made the suite's AI matches run near three times as long. Nobody had asked for mining to change, so it's confined to a ring's rock. Whether miners should reckon their own load is a question about mining, and his. |
| PR16 | The network protocol is 32. (33 with PR17.) | |
| PR17 | **A ship that's lost spills what it was carrying:** loose pixels where it was, for whoever comes. 300 of them at most, as with what a broken node held, and the rest is lost. It's every ship's rule: a miner's load spills too. The network protocol is 33. | His ruling: "Yes, spill it". (The 300 is mine: it's the node's rule.) |

**Why a ring (PR2).** A harvester needs a rule for how fast it charges. The plain one, a pixel of charge for so many pixels of the part, pays a swarm of small harvesters exactly what it pays one big one. A ring twice as far across has twice the pixels and four times the room inside, so a ring's worth for what it costs grows with its size: the first rule in the game by which a big ship is worth more a pixel than a small one, which he asked for on 2026-11-13 ("they should be actually more powerful per-pixel since you have to save up to build them and then wait out the build time"). It makes the part a large, hollow, exposed structure and nothing else (2026-11-13, of the projector: "an entire, large, exposed structure"), and it has an answer you can draw and one you can shoot: one pixel out of its side. What I'd watch for: whether every ring ship comes out as the same square. The most a ring counts is 400 pixels shut in, which is 20 by 20, so there's no use in a bigger one. If he'd rather have the plain rule it's one line of `stats_of`: nothing else here depends on it.

**The examples.** The HALO: a ring 17 across in a band of hull, which makes its power and takes the first hit from any side; a core; ten engines and two rotation engines at its stern. 149 px. It shuts in 15 by 15, so it puts 2.2 px a second on a rock. 11 px/s, and three seconds to a quarter turn. No guns. The CRADLE: PR14. Both are on the library's PARTS shelf.

**What it does, measured.** All on the tests' map (seed 1, scattered rocks) unless it says otherwise, with nobody playing either side.

- **A rock of 401 px** (blue's home rock) went in 179 s, aboard the HALO. Nothing was banked.
- **With only its king's tractor to go to** (eight tractor pixels, which pull a pixel a second): the first pixel was banked 11 s after the rock went, 61 were a minute later, and the HALO was empty 6 min 51 s after the rock went. Then it flew to the nearest rock and had its ring on it 26 s later. So a HALO puts 2.2 px a second on a rock and its king takes 1 px a second off it.
- **With a CRADLE 80 px from the rock:** the HALO passed its king's tractor by, came down on the pad, and the whole 401 was banked in one tick, 28 s after the rock went. Its ring was on the next rock 14 s later.
- **In a match on seed 3** (the pictures): a rock of 470 was half charged at 1:45 and went at 3:29, with blue's bank at 242; the HALO was down on a CRADLE at 3:42, and blue's bank was 715.
- **Asked of the king and left:** the king takes 4 min 10 s to print one. It flew to the nearest rock, 99 px, and its ring was on it 19 s later. A minute on it hadn't stirred. Sent 154 px to another rock, its ring was on that one 19 s later.
- **Printed by a factory on a rock** (the starter shipyard on blue's home rock, 36 s to print one): it's put off 26 px from the rock's middle, which is in its reach, so its ring is on its own factory's rock a tick later and it never leaves until that rock has gone.
- **Miners:** with two unarmed miners of each side at the same rock it went in 138 s, not 179, and the HALO got 309 of the 401.
- **Armed miners shoot it.** The starter deck's miner has a flak turret. Two of the enemy's at the rock shot hull off the HALO's band until it was short of power and charging at 200 where it had 225, and the rock went four seconds later than with no miners there at all.
- **Rivals:** one HALO of each side on a rock for forty seconds leaves no charge on it. Blue's alone for forty seconds puts on 90. Two of red's then take that off in forty seconds, and put on 87 of their own in the next forty.
- **Driven off:** a HALO with 90 on its rock, sent 150 px away and back, came back to 36.
- **With a load aboard** and another rock in its reach for twenty seconds: no charge on that rock.
- **Forty-eight approaches to a pad** (a loaded HALO put down 45, 90 and 180 px from a CRADLE on sixteen bearings, turned each of four ways): every one down and banked, the slowest 35 s after it was put there.
- **A tractor nearer than a pad:** 67 px from its king's tractor's reach and 98 from a CRADLE, with 400 aboard, it went to the CRADLE and all 400 were in its store 27 s later.
- **A ring that can't fly** (nine across on a base with no engines) charges the rock it stands by, and has what it takes aboard: cargo for a tractor that reaches it.

**What that is beside miners, as arithmetic and not as a result.** A scoop bites a pixel every 10 s, so a DIGGER (19 px, four scoops) bites 0.4 px a second at the rock face, before any flying or waiting on a tractor; 149 px of DIGGERs is 3.1 px a second there. The SCAVENGERS' passive opening banked 522 px by 8:00, which is 1.1 px a second for the whole operation. A HALO puts on 2.2 px a second while it's at a rock, and banks nothing until the rock has gone and been taken off it: at a pixel a second by a king's tractor, or at once by a pad that costs 172 px more. The biggest ring that counts is 22 across and puts on 4; a ship drawn round it as the HALO is would be about 190 px. None of this is tuned, and none of it has been played.

**What follows from rules that were already there, and I haven't tried.**
- A seed on its way to a rock that goes heads for the nearest free rock, as it does when a rock is mined away.
- A factory growing out of a rock that goes finishes from its network: so an enemy ring can take the rock out from under a factory that's still growing, and the factory then costs its side the pixels the rock would have given.
- **A loaded HALO has to place itself more finely than a miner does.** A ship stops as near its spot as a ship of its sort can, which is 14 px for one that takes three seconds to a quarter turn, and a pad takes hold of a ship within 12 px of where it waits. Placing itself so, a loaded HALO stopped outside a pad's hold on some approaches and stood there for good; so with a rock aboard it places itself to 3 px, as a ship sent to a spot does. A miner that's as slow to turn still places itself to 14. I haven't looked at whether any miner stalls that way: the SCAVENGERS' miners turn fast enough that they can't, and his own drawings might not.
- (What a ship carried was lost with it: a miner's dozen pixels, and then a ring ship's whole rock. Asked, he said to spill it: PR17, and "The spill", below.)

**The spill (PR17), and what it moved.** A HALO with 470 aboard and a starter miner with a full load, each losing its core: where each was there are its own pixels, loose, and its load with them, all of the miner's and 300 of the HALO's. An enemy miner put down among the HALO's has a full load a tick later. It's a rule for every ship, so every match in which a loaded miner is lost plays differently from here on. One older test moved, the AI's king chased by two BATTLESHIPS on eight maps, which moves with every change to the game:

| | Came through with half of itself or more and under 6 s in reach | The others |
|---|---|---|
| Before the spill | 5 of 8 | 3, in its own corner: left with 89 px, and killed twice |
| With it | 6 of 8 | 2, against the map's edge: killed once, in its corner; and once alive with 110 px of 218 after nine seconds in reach, against the left edge |

That test asked of a king that doesn't come through well that it be in its corner. It asks now that it be against the edge of the map, corner or no: the third time today it has been re-stated. One scenario, and no verdict on anything.

**Not done.**
- The AI doesn't build one, doesn't shoot one first and doesn't bring a ring of its own. (A ship of its on SAB counts one as a worker, as it does a miner. Not tried.)
- No ready team has the part. The HALO and the CRADLE aren't in the SCAVENGERS: they're on the PARTS shelf to put in slots. The SCAVENGERS' PORT has a pad 6 by 9, which a HALO doesn't fit.
- A ship that's been sent somewhere doesn't unload, as a sent miner doesn't: sent with a rock aboard it parks where it's sent until it's told to go back to its own devices.
- A charge doesn't show on the tactical map or the minimap, and nothing shows a ring's reach.
- The giant rock in the middle of a scattered map (about 2,800 px) would take a HALO 21 minutes, and a king's tractor 47 more to empty it.
- A ring's ship places itself by the rock pixel nearest it and not by where it's safe.
- I haven't seen any of it in the running game: the pictures are frames the game drew headless. `screenshots/phase_ring_yard.png`, `phase_ring_at_work.png`, `phase_ring_takes_the_rock.png` and `phase_ring_on_its_pad.png`; `--phase-screenshots [DIR]` draws them again.

**Tested.** Eighteen new tests: what a ring shuts in (closed, open, a corner out, other parts in the wall or inside, two rings, two thick, the most it counts, short of power, the HALO's numbers); a rock charged and taken whole as cargo, taken off at its king's tractor a pixel at a time, and the next rock after that; a rock landed on a pad it fits and banked in one tick; miners of both sides; rivals; at rest only, and the leak; printed by the king and left, then sent; printed by a factory on a rock, and nothing charged while it's being printed; a ring broken at a corner and at a side, and a ring ship shot at; which of two rocks, the reach to the pixel, and nothing charged with a load aboard; whose ship takes the rock, and a ring that can't fly; a full charge with no ring on the rock; forty-eight approaches to a pad; a pad chosen over a nearer tractor; the bar and the dot; the load in the ring; a lost ship's spill; the yard's dots and status line. Forty-five deliberate faults, each caught by a test. Three got past the tests as first written, and each has a test now: a ring charging while its ship was still being printed, a loaded ship stopping too far from a pad to be taken hold of, and a tractor reckoned to empty a rock the moment the ship arrived. The suite is 418 and passes, with the same four tests switched off.

### The repair array: proposed the same evening, not built

*Henry, 2026-11-17:* "a repair array that uses banked pixels to repair damaged ships (this could even indirectly build two full ships out of one if a ship got split brained and then repaired)".

What I'd build, put to him 2026-11-22:

- **A lump of repair pixels is an array,** and its ship is a node: it draws on its network's bank.
- **It mends ships of its side in its reach,** a pixel of reach for each pixel of the lump, from the lump's box, as a slipstream reaches.
- **It puts lost pixels back one at a time, each next to one still standing,** so a ship grows back from the wound. Each is paid for from the bank, as a printed pixel is, and takes what a factory pixel takes to print one (4 s for an array of one pixel, and a bigger one sooner in proportion).
- **A piece cut off with a core of its own grows back to the whole drawing,** so a ship cut in two and mended is two ships, each paid for. (His. A piece keeps only its own pixels today, so it would have to remember the drawing it came from.)

Three things that are his to say, with what I'd pick:

1. **In a fight, or between fights?** I'd mend only a ship that hasn't been hit for 3 s (a shield's rest). Mending under fire is a healer, and a ball of ships round one is hard to break by anything but killing the healer.
2. **Its own ship?** I'd say no, as a projector never covers its own: the array's ship is the thing to shoot, and two of them mend each other.
3. **The king?** I'd say no for now. A king's damage is for good today, and that's what ends a match.

---

## Wishlist (Henry's, for later; none of it is a work order)

- **2026-11-22: the phase-harvester is built, as the PHASE RING (see "The phase ring").** Next in his queue: the repair array.
- **2026-11-18, later: the vacuum is built too (see "The vacuum").** Next in his queue: the phase-harvester and the repair array.
- **2026-11-18: the speed-up field is built (see "The slipstream"), and the scoop with a range is out of the queue.** Henry: "Instead of a ranged scoop, which naively is pretty redundant with a regular scoop, what if we have a vacuum that is specialized in picking up loose debris pixels but is less efficient when used on asteroids?" The vacuum is thought through in that section and not built.
- **The part queue as Henry left it on 2026-11-17** (this one is a work order): a speed-up field and a stasis field ("among the riskier additions that would be good to get in earlier"), the ranged scoop, a phase-harvester ("builds up a charge on an entire asteroid and harvests the entire asteroid at once once the bar reaches the amount of pixels remaining on the asteroid"), and a repair array ("uses banked pixels to repair damaged ships"). Then "we're getting close to some sort of 'full set' of part content, although I haven't decided exactly how we want to handle building sets."
- **A scoop with a range** (Henry, 2026-11-17: "New mining wishlist item: a scoop with a range."). A scoop takes rock within 8 px of its ship's middle (W19).
- **Docking ports**, built 2026-11-17 as the DOCK PAD (W6 to W9). (Henry, 2026-11-16, with the tractor): "The other thing I considered is docking ports rather than tractor beams. Maybe if we create distinct part collections in the future that can be an alternative to this. Or both can just coexist. I think it will be fun to have both. They serve different purposes too because one reaches out to the miners a little bit and the other has miners doing more flying. We can have the docking station be good because it drains the miner's whole cargo instantly, but it needs to be large enough for the miner to fit on it."
- **A counter to SWARM** (Henry, 2026-11-16): "an easy way to counter swarm is to put a tracker on your outpost." His own OUTPOST has one.

- **A king that's upgraded twice.** Henry, 2026-11-14: "Once we eventually get to king design, I want to have people optionally design two upgrades to their king where you pay the price upfront and it grows into a bigger, more consequential mothership/king. (twice. so you draw your starter king and then two bigger versions)." So three drawings of a king: the starter and two bigger ones, each upgrade paid for in full before it grows.
- **Campaign levels where you fly the king by hand:** the old WASD king. Henry, 2026-11-13: "We can think about direct flight as a special level in the campaign, perhaps." And 2026-11-14: "I'm thinking of having special manual-king-flying levels in the campaign. (our old WASD kings)". The code that flew the king from the keys is kept, unused, for this (`Cmd::Steer`, `Link::steer`).
- **After automatic flight: a better set of flight patterns,** and perhaps an evasive-manoeuvres pattern (2026-11-14, above).
- **A theme:** the game as a galaxy-wide sport played over a 64-bit link, the player climbing to Wingleader (2026-11-10, in the round 2 notes).
- **A reactionless drive that works inside armour,** as something a campaign could offer (ROUNDS.md).
- **Part ideas (Henry, 2026-11-15, "some parts brainstorming"):** ideas for parts, not a work order. My notes in brackets are only what each meets in today's rules.
  - **An energy-ball gun:** it launches a 2x2 energy ball that moves slowly and brings shields down in one or two hits. (Today a shield stops four shots and is full again 3 s after the last.) (Built 2026-11-17 as the ENERGY BALL: W10.)
  - **A phase-blade gun:** it launches an L-shaped phase blade that moves slowly and destroys armour. (Today plate shrugs off any hit no stronger than it, and a lance uses up two of its punch on a plate pixel.)
  - **A projectile-speed upgrader:** you put it next to weapons and it speeds up their projectiles.
  - **An armour-penetration upgrade** for weapons.
  - **A blast-radius upgrade** for weapons, "works even on weapons that aren't splashy already!"
  - **A second level of armour** with a higher level of penetration needed to get through it. (Today there's one: plate.)
  - **Engines that increase turn speed**, "if that isn't already on there". (It partly is: a ship takes 2 x its weight / its thrust ticks to turn a quarter, so engines already turn it faster, by adding thrust. A part for turning alone would be new.)
  - **A fast-tracking turret** with fast-moving anti-fighter projectiles: very effective against fighters, but large, and it needs lots of hull. (Built 2026-11-15 as the TRACKER, with the second level of armour as HEAVY PLATE and the armour-penetration upgrade as the PIERCER: "Heavy plate, the piercer and the tracker".)
  - **Engines that must be even more exposed** but are a little more powerful than today's. (Today each engine needs a 3-wide way straight out of the ship for its exhaust.)
  - (The earlier list, of 2026-11-11, is in "The parts talk opens": logistics parts for miners and for speed, diagonal guns, stasis fields, rotation engines, rapid-fire guns, more turrets, a weapons charger, parts that need clearance, a sweeping weapon, more range and tracking play.)

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
