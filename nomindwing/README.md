# No Mind Wing

A DOS-style arcade RTS: arcade on top, RTS underneath. You fly your king (mothership) yourself. It is the root of a tree of factories, and you spread that tree across the map with seeds. Everything is made of pixels: rocks, ships and the money that prints them.

**The fun thesis (Henry's words):** the mothership is the king and the root node of a tree of factories. You spawn off little self-propagating wings strategically, like a resource-collection dispatch or a forward production base. It's "the creativity of designing a TCG deck combined with the strategic thrill of outmaneuvering someone in chess."

There are two toys so far:

- **Proving ground** (the default): draw ships, set them loose, and see whether they fly and fight the way their drawings suggest. Iterated in rounds, logged in [ROUNDS.md](ROUNDS.md). From its yard, **MATCH** plays the match (below), planned in [MATCH.md](MATCH.md).
- **Toy 1** (`--toy1`): the first playable match, testing the tree of factories.

```
cargo run --release
cargo run --release -- --windowed
cargo run --release -- --toy1
```

# Proving ground

Toy 1's playtest found good feel and no depth. The planned fix is that strategy flows through the army you design, so every ship has to be drawn, not picked. The hard part, and where the 2020 version stalled ("sketchy enemy mind stuff"), is ship AI that works for any drawing. This toy tests one answer: a three-layer brain where none of the layers knows what kind of ship it is.

The proving ground is iterated in rounds. **[ROUNDS.md](ROUNDS.md)** has the goals, every round's additions, the verdicts and the parking lot. This is round 5.

1. **The order says where to go.** Each design has one: Hunt, Guard, Escort, Harvest or Hold. Guard and Escort are match orders (wait at a post in the rings round what built it; form up on your king) and the order a design is drawn with is the one it starts a match on. The arena has no factory to guard and no king to escort, so a design drawn on either hunts there.
2. **The parts fire on reflex.** Flak turrets fire at the nearest enemy in range whatever the order, so an armed miner shoots back while it mines. A hunter with no guns left stays beside the nearest friend that still has some; guards and miners with no guns run home when hit.
3. **Mass and thrust decide how to fly.** Integer inertia: velocity changes by whole quarter-pixels per tick, as fast as thrust over weight allows. A ship with at least 1 thrust per weight makes attack runs (dive, overshoot, loop back). A lance fires only straight ahead, like a rook, so a ship with one slides onto the target's row or column first and then dives along it ("LINING UP" on hover). Anything heavier holds at weapon range and lines its lances up the same way. Without engines it stays put.

**Ship rules (since round 3):** a ship needs at least one core, and all its pixels must join up into one piece, edge to edge. Corners don't count. Every engine needs a straight 3-wide way out of the ship for its exhaust, like a rook's move: in one of the four directions, the engine's own line and the lines either side of it must be empty right out past the edge of the ship. So there's always a straight line of fire to every engine; plate can't fold round one (round 4; in round 3 the way out could bend). Loose pixels and buried engines show red in the yard, and the ship can't fly until they're fixed. Extra cores cost more each: the second +8, the third +16, the fourth +32. A ship's cost is its pixels plus its extra cores. In a fight, a ship dies when its last core goes. If it's cut apart, every piece that still holds a core flies on as a ship of its own, and pieces without one break away.

**Power (round 5):** hull makes it and engines and guns use it, from one pool per ship (each piece of a split ship runs on its own). A ship short of power runs everything at the fraction it has: at 75%, guns reload a third slower and engines push a quarter less. As hull is shot off, it weakens gradually. Short is allowed, just weaker; the yard shows "UNDERPOWERED", and the arena's hover shows the percentage.

**Parts**, in the yard's six categories (keys 1-6; Tab, or the same key again, steps to the category's next part). The Power category is still empty: hull is the power.

| Category | Part | Weight | Power | Does |
|---|---|---|---|---|
| Hull & armor | Hull | 1 | makes 1 | Holds the ship together. Any hit breaks it. |
| | Plate | 2 | | Flak bounces off it. Lances and cannon shells break it. |
| Weapons | Flak | 1 | uses 1 | Turret, 48 px, 1 damage. Leads its aim and spreads its fire over several targets. One shot per 1.3 s. |
| | Lance | 2 | uses 3 | Fires straight ahead only, 100 px, 2 damage. Punches through 6 pixels, or 3 of plate. One shot per 3 s. |
| | Cannon | 18 (3x3) | uses 8 | A big turret, 130 px, turning about 42° a second. Its shell blasts a 3x3 hole, plate and all. Goes for the big ships in range (anything at least 2/3 the size of the biggest), and of those the one its barrel reaches soonest, so it stays on its target. One shot per 3 s. |
| Drive | Engine | 1 | uses 1 | Thrust. Needs a straight 3-wide way out. |
| Industry | Scoop | 2 | uses 1 | Mines and carries 3. |
| Command | Core | 1 | makes 4 | At least one per ship; extras cost +8, +16, +32. The ship dies when its last core is gone. |

**Shipyard (round 4):** designs go up to 99x99, odd so the mirror has a centre column. The canvas zooms in whole steps with Q/E, the mouse wheel or the -/+ buttons: 1x is the ship at its real size, as in the arena; 2x shows the whole grid; 4x and 8x are for fine work, and at 8x each cell shows its part's shape. WASD or the arrows scroll. A design opens framed at the biggest zoom that shows all of it. Enter, or the ARENA and YARD buttons, swaps the yard and the arena (Tab did until 2026-11-10, when it became the next-part key in the yard; it still leaves the arena).

| Tool | Key | Does |
|---|---|---|
| Pencil | P | Left paints, right erases, drags leave no gaps. A cannon stamps a 3x3 block. |
| Rectangle | R | Drag a box to fill it; right-drag empties one. Cannons tile it in 3x3 blocks. |
| Fill | F | Fills a hole inside the ship, or swaps a patch of one part for another; right-click empties a patch. Open space that reaches the edge won't fill. |
| Copy / stamp | C, T | Drag a box to pick it up, or right-click a design's tab to pick up the whole design. Click to place it as ordinary pixels (nothing stays linked); T turns it, right-click puts it down. |
| Undo | U | Steps back through changes to the design (64 deep). |
| Mirror | M | Every tool also works on the other side of the centre column. |

The right-hand column is icons and numbers, with the details on hover: cost, flight style, gauges for top speed, turning and thrust per weight (the gold mark is 1 thrust per weight, where attack runs start), a tally of the parts the ship has, and the status line.

Stats change as pixels are shot off: lose your engines and you slow down; lose hull and you may run short of power. Enemies bigger than you are solid, so you can't fly into them and shoot them from inside their plate. Ships move a pixel at a time, so they stop flush against whatever is solid to them and can't skip across something thin. An attack run peels off sideways before it would hit one, so small ships make strafing passes along a big ship's hull.

**Arena:** 480x360, seen through a 304x300 window. Scroll it with WASD, the arrows or the 1/6-scale minimap. Nine ready-made tests use the seven design slots and are named from what's in them, so editing or renaming a design changes the tests. They're made for the starting library (the starter deck): the counter triangle, miners and a capital under attack, and three raids on a factory. A seeded design placed in the arena stands planted, as it would on its rock, so it's solid to every enemy ship. Keys 1-7 pick the design to place. The arena is deterministic: R replays exactly the same fight. A fight ends when one side is wiped out, after 3 minutes, or as a stalemate (a draw) once 20 seconds pass with nobody firing, nothing shot off and no armed ship closing in. The result names any side with no working guns left. Hover a ship to see its order, what it's doing, its target and where it's heading. Faint trails show recent flight paths; gold barrels show where cannons point; white sparks are shots bouncing off plate.

**Headless tools:**
- `--pg-sim`: the arena's reference fights, fought with the starter deck: each of its nine ready-made tests, then every pair of its four warships at the cost of the dearest (687 px a side), each on 12 seeds (seed 1 is the fight the window replays). Draws are split out into stalemates, and splits count the ships that broke in two. `--pg-sim old` fights the proving ground's own fleet (below) instead, in its own tests, as the five rounds were judged.
- `--pg-trace BLUE_SLOT N RED_SLOT N EVERY UNTIL`: ship-by-ship state. Use blue slot 99 to run test number RED_SLOT.
- `--match-sim N [--blue FILE] [--red FILE] [--bots BLUE,RED]`: N matches with nobody watching (seeds 1 to N), with how each ended, what each side made of each card and when it first had one, how many ships it had at 2:00 and 4:00, and a line every two minutes of the first. Each side plays the starter deck, or the designs in a save file such as `proving_ground_designs.txt`. `--bots` says who plays: `ai` (the default) or one of the scripted openings the AI is measured against, `clock` (round 1's opponent), `planter`, `expander`, `summoner` and `boomer` (a factory whenever the bank allows, and a king that prints miners without a pause).
- `--judge N`: the AI against each scripted opening and against itself, N seeds on each side of the map. One line each: won, lost, drew, how long the matches ran and how many ships each side made.
- `--pace N`: how fast a match runs away, on seeds 1 to N: the boom against the AI, and the AI against itself. For each, who won and how long the matches ran, and for each side the factories it had planted and the pixels it had mined by 3:00, 5:00 and 7:00, when its second factory went down, and the ships it made (medians).
- `--deck [FILE]`: what each card of a deck is (size, cost, power, speed, guns, build order and how long one round of it takes), for the starter deck or a save file.
- `--export-screenshots DIR`: proving-ground and match screenshots.

## The match (Match round 2)

A 1v1 against the AI on a mirrored 1400x1050 map: kill its king. From the yard, **MATCH** asks whether you play your 7 designs as your deck or the starter deck; the AI plays the starter deck. Your seven start out as the starter deck, so there's a working deck to redraw from the first minute. Each side starts with its king and a 190 px bank, nothing else. [MATCH.md](MATCH.md) has the plan, the numbers and what each round is testing.

| Input | Does |
|---|---|
| WASD / arrows | Fly your king (like Toy 1's, straight from the keys): 37.5 px/s, or 7.5 once your side is beaten (below). |
| 1–7 or click a card | Your king prints that ship, a pixel at a time from its network's bank (0.6 px/s). It arrives beside the king by hyperspace. |
| Right-click a card, or R | Your king keeps printing that ship, one after another, until you right-click it again. The card wears a blue frame and a loop sign. R does it for the ship being printed. |
| The queue, under the cards | Your king's jobs as icons, the one it's printing first, with a bar for how much of that one is paid and a loop sign under one that repeats. Click a job to drop it; right-click it to set its design repeating, or stop it. Five show; a longer queue shows four and a count of the rest. |
| Backspace / X | Your king drops what it's printing and gets back what it had paid. |
| A seeded card, then a rock | You pay everything the design will grow into (121 px for the outpost), at once. Its seed flies to the rock and grows the design out of it, one rock pixel per pixel. A seeded card you can't pay for is greyed. Right-click or Esc cancels the choosing. While you choose, a dotted line runs from the rock under the pointer to every node the new factory would link to there: green if that reaches your king's bank, gold if it only reaches nodes cut off from it. Seeds already on their way count as the factories they'll grow, so a chain can be planted one after another. With nothing in reach, a dim line runs to the nearest node and the bar says how many pixels too far it is. The seed comes down on the very pixel it was sent to, if that's still free. |
| A card's order tag, or Shift+1–7 | Switches that design's standing order: HUNT, GUARD or ESCORT (and back to HARVEST for a miner). Every ship of the design obeys at once, and so does every one printed afterwards. |
| The small map | Hold the mouse button on it to look elsewhere; flying your king brings the view back. |
| Tab / MAP | The whole map in the field at a fifth of its size. Your king still flies, and you can plant and give orders from it. |
| Space / PAUSE, F / speed, F1 | Pause, 1x/2x/4x, the rules. |

- **A beaten king is slow.** After the first minute, a king whose side has under 40 px of ships and factories standing (everything of its team but the king itself) flies at a fifth of its speed: 7.5 px/s, not 37.5. That is slower than the slowest ship a drawing can make, so anything with an engine catches it. It's back to full speed the moment its side is back to 40 px: a seed in flight and a factory still growing count, the bank doesn't. The field says so for either king, a slowed king's engines go dark, and the two lines under the small map show SLOW; hover a king for how much its side has standing. At full speed a king is as fast as any ship a drawing can make, and faster on a diagonal (the starter deck's fighters make 37 px/s), so one with nothing left to lose could otherwise run until the 40-minute draw.
- **Standing orders.** HUNT goes after the nearest enemy, wherever it is. GUARD waits at a post round what built the ship (below) and fights what comes within 160 px of its builder's place. ESCORT forms up in rings round your king, flies where it flies and fights what comes within 110 px of it; a king at full speed outruns its escort, which catches up when the king slows. A design starts a match on the order it was drawn with. The starter deck's warships are drawn on GUARD, so they gather round their factory until you send them.
- **Posts.** A ship with nothing to do waits at a post of its own: a place in the rings round what built it, which is its factory, or the spot where your king summoned it. Guards fill the rings in the order they were built. The first ring is 16 px outside the builder, clear of what it prints next, and rings and posts are about 16 px apart, so an outpost's first eight guards stand all round it and the ninth starts a second ring. At its post a ship looks outward and sits still. It gives way only to what can't move aside for it (a factory, a ship being printed, a king) and to a ship already waiting that was built before it: then it waits at the nearest place clear of them all (of two as near, the one further from home), and goes back when the way is free. Whatever flies past goes round it, except a miner at work, which flies over. A guard that leaves to fight comes back to the post it had; the rings close up when one is lost. Ships your king summons within 24 px of posts already kept join those rings.
- **Designs are built or seeded** (yard: BUILT / SEEDED). A seeded design's core stays at the grid's centre, where its seed lands, and it grows outward from there. It needs a rock at least as big as itself, or it stays unfinished. A seeded design may have engines and guns: it grows out of its rock, then flies off when it's whole.
- **Factory pixels** (Industry, 2 power each) print a design's build order (yard: BUILDS / ADD, by deck slot). Each prints 1 pixel about every 27 s, so the outpost's 28 print 1.05 px/s, and each lets the factory build ships up to 10 px bigger. It builds the next thing on its list that it's big enough for, pays as it prints, and waits when the bank is empty; printers sharing a bank take turns, a pixel each. Nothing grows, prints or arrives by hyperspace into an enemy's pixels: it waits or grows round. Carriers (built ships with factory pixels) print the same way.
- **Pixels come from rocks.** A ship with two or more scoops bites a pixel out of a rock every 3.3 s and hauls a full load to the nearest node. A miner at work flies straight to its rock pixel and straight home with a full load: it keeps apart from nobody and nobody keeps apart from it, so miners heap up on a rock face and fly through each other. Each bite sparks on the rock, and a scoop turns the colour of loose pixels for each scoopful aboard, so you can see a miner working and how full it is. Your king also makes one pixel every 4 s by itself, so a side that has lost every miner can still print one.
- **The network:** your king and anything with factory pixels are nodes. Nodes store pixels and link (dotted lines) when within reach of each other, and linked nodes share a bank. Miners haul to the nearest node. Cut a link and the bank splits by where the pixels sit; a destroyed node spills what it held. A note says so when your network breaks in two: two nodes parting, or your king cut off by a node you lost. Your king flying out of reach by itself breaks nothing, and says nothing. BANK is what your king's network holds; a `+N` beside it is what nodes cut off from your king hold. Fly near them, or plant a link, to spend it.
- **Rocks have no owners.** Anyone can mine any rock, and a seed can eat any rock. The planted factory is the claim.
- **Factories stay put, keep apart and are solid.** No factory grows into another, yours or the enemy's: a seed sent where one already stands comes down beside it and grows what it has room for (the planting bar warns you first). Whatever is planted on a rock is solid to every enemy ship, whatever its size: a king flown at an enemy factory stops against its plate. Your own ships fly over your own factories, and your miners work the rock under them and under any ships parked there. A bigger ship does pass over a drifting hulk with no engines, without pushing it. Rocks themselves stop nothing.
- **The AI** plays by the same commands you do. It plants a factory when its bank holds the price (and never lets you have more factories than it has for long), keeps one more miner than it has factories, masses its warships on GUARD, and sends them out as a wave (the slow designs first, so they arrive together) when it's the stronger side or has rested long enough. A raider within 200 px of its king or anything it has planted turns the guards out, and they stay out until every raider is 60 px further off than that. Its king prints what counters your fleet: something big against small flak ships, a flak swarm against small lance ships, lances against big ships.

For anyone changing the sim: every part is one row of `PART_INFO` (`src/pg/design.rs`); every rule about where pixels may be is in `src/pg/sim/space.rs`; all match input is a `Cmd`; the arena and the match draw the world through `src/pg/scene.rs`. `cargo test --release` runs soak tests that audit every tick of every arena test, an AI match and four matches driven by random commands (`src/pg/sim/audit.rs`).

Sessions append to `proving_ground.log` (each run's designs and result, and each match's start, a line a minute and its result). Designs persist in `proving_ground_designs.txt`, each trimmed to its drawing with its place on the grid (round 2 and 3 files, which saved the whole 41x41 grid, load centred); round-1 designs (15x15, old rules) were moved aside to `proving_ground_designs_round1.txt`. A saved design still exactly like an old default is swapped for the current one when it loads. All of these files are gitignored.

**The starting library.** With no save file, your seven designs are the starter deck: FIGHTER, LANCER, FRIGATE, MINER, OUTPOST, SHIPYARD, CAPITAL. They are Henry's drawings of 2026-11-10: every card spends all the power its hull makes, so none carries pixels that do nothing (`--deck` prints the sheet). The miner was redrawn once more that day, after eighteen drawings were played against each other. On 2026-11-11 a lance, a factory pixel and a scoop each came to use more power (and a scoop to weigh 2), and the five cards that left short were given the hull they need, shapes and sizes kept: the lancer is 9 px, the miner 9 px with two engines, the outpost has 28 factory pixels and the shipyard 118. Those five are stand-ins, drawn by rule and not by play. A save still holding an untouched card from before gets today's when it loads. Delete `proving_ground_designs.txt` to get it back. The fleet the rounds were judged on (fighter, lancer, frigate, destroyer, capital, gunminer, titan) lives on in the code with its own nine tests. Its drawings carry hull that feeds nothing, so since 2026-11-11 nothing is measured on it: `--pg-sim` fights the starter deck, and `--pg-sim old` gives the old fleet's numbers, which a change to the arena's rules would still show up against. The checks fly both. The designs drawn during the rounds were moved aside to `proving_ground_designs_before_match_deck.txt` on 2026-11-10. To have one back, copy its line over the line of the slot you want it in (the file is a header, then slots 1 to 7 in order).

What the rounds found so far is in [ROUNDS.md](ROUNDS.md).

# Toy 1

## Controls

| Input | Does |
|---|---|
| WASD / arrows | Fly your king. If it dies, you lose. |
| 1–5 or click a card | Print it. Seed cards then need a rock: click one on the map, minimap or tactical view. |
| Right-click / Esc | Cancel planting |
| Tab / TAC | Tactical view: the whole map at exactly 1/4 scale |
| Backspace / UNDO | Take the last card off the print queue (printed pixels come back) |
| Right-click a queued card | Repeat it every time it finishes (blue frame); seeds can't repeat |
| P / PAUSE, F1 / HELP, F2 | Pause, rules, scanlines |

## Rules in one breath

- Pixels are money. Every design costs its pixel count, paid one pixel per frame as it prints.
- Collectors chew pixels off rocks and wrecks and haul them to the nearest factory.
- A seed flies to a rock and eats it into a base, pixel by pixel.
- A base keeps its pinned build order alive, rebuilding whatever dies:
  - **Outpost:** 2 collectors, 1 escort, and 1 outpost seed. The seed means outposts spread.
  - **Forward base:** 2 escorts and 3 raiders.
- Raiders wait at their factory until there are 3, then the pack hunts.
- Every colour is a part: yellow shoots, blue-white pushes, orange scoops, grey takes two hits, and a ship dies when its white core is gone.
- Shots knock pixels loose for anyone to collect.

## Tools

- `--sim N`: plays N AI-vs-AI matches and N AI-vs-idle-king matches headless, then prints the stats.
- `--trace SEED [--both]`: prints one match's state every 20 seconds.
- `--export-screenshots DIR`: 2x PNGs of the main screens.

Playtests append to `nomindwing_playtest.log` (gitignored): every card played, every base planted or lost, and a snapshot every 30 seconds.

## Tuning notes (2026-11-07)

- **Map:** 960x720. Every rock has a mirror twin through the centre. An earlier centre rock sat 0.35 px closer to one king, and that king won 5 of 6 AI matches.
- **Aiming:** gunners aim at a random surviving pixel. Aiming at the centre drilled a straight tunnel through a king that later shots flew through without touching anything.
- **Pacing:** the AI builds economy first, plants its forward base at about 1:50, prints no raiders before 1:40, and doesn't send its king before 3:20. An idle king falls at about 2:10. AI-vs-AI matches end in 3½–6½ minutes, roughly 50/50.
