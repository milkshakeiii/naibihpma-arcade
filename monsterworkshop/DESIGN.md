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

### The world's spirit
They're monsters, but their society is perfectly cooperative and peaceful. Nobody is paid, taxed, traded with or fought. Renown is the commune's shared standing, not money. Goals are shared projects that a town needs, such as a sail for its ferry, and everyone pitches in. Player-facing text follows this: no wages, prices, taxes, buying or selling.

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
- **Pinewater, the weaving town (west)**, about 2×2 screens. It has the spinnery, the loom house, the gathering hall and a public workbench, with the cotton fields right outside. M1 can be played entirely here.
- **Wilderness (middle)**, 2–3 screens of road between the towns: forest (hemlock near the road, oak deeper in), a bamboo grove, scrub and meadow (sticks, ramie), and dye plants toward the east. This is where frontier workshops go. Rarer varieties sit farther from the road.
- **Tanglemere, the dyeing town (east)**, about 2×2 screens. It has the dye works, project boards, a public workbench and the great work. Fabric travels east to be dyed, which is what makes wagons matter.
- **Later regions** feel farther away and weightier: for example, hills beyond the dyeing town with copper and tin for the Temple Bell chain.
- Its two-town shape follows the book: weaving at one town, sizing and dyeing at another.
- **Naming:** place names quietly translate the meaning of the real places behind them. Songjiang ("pine river") becomes Pinewater, and Wuhu ("overgrown lake") becomes Tanglemere. Name the wilderness and later regions the same way.

### Music
Henry composes the soundtrack. The arcade's Star Pond already streams WAV music, so that code can be reused.

**v1.5:** equipment as physical crafted wearables; charity and great-works drop-offs; and an in-world quality sorter (see below).

## Features

### Routines that scale (accepted 2026-10-31)
Literally replaying a recorded routine works for two monsters and turns into traffic management at ten: two loops meeting in a one-tile lane wait for each other forever, and wagons and the ferry make it worse. The macro game, planning the economy, should be the focus, not keeping recorded routes from colliding.
- **Teaching stays the same:** you show a monster the job once by doing it. What changes is what gets recorded: the job's goals, not its keystrokes. "Take slivers from the bench's output mat, deliver them to the spinnery's pad, use the spinnery, take the thread to the loom house." On replay the monster pursues those goals however it can: it walks around others, plans its own pushes (and the occasional slow pull), and rides the ferry as a timed step. When something it needs isn't there, it waits and says so: "I'M WAITING FOR THREAD."
- **Finishing and sharing a recording (built):** R a second time, away from the flag, walks the monster home by itself and saves (while leading a wagon, let go first). Steering it on the way takes over. A loop starts from its flag with P, or carries on from the goal it was at if it's resumed where it was stopped. Since loops became pure goals (phase 4) none of this is needed for replay: the walk home isn't recorded and a loop could start anywhere. It stays for the feel of it (decided 2026-11-01): placing the flag, finishing back at it and starting from it make a loop feel organized and owned. P while standing on another monster's flag takes up that monster's loop, flag and all, and starts it, replacing the copier's own. Handy for testing; two monsters on one loop mostly just take turns, since the second waits for goods the first already took. Flags sharing a tile stand side by side.
- **Planning is deterministic** (fixed tie-breaking), so saves and a persistent world still replay exactly. A monster only ever plans the one item it's moving, which is easy; pulling means no item is ever truly stuck.
- **Not tied to skills.** Every monster plans equally well.
- **Sokoban stays a small element:** in your own hands-on play, and in how efficiently a layout lets loops run. That's enough. A separate puzzle-heavy side zone is an option for later.
- **No extra tie-in visualization for now.** Monsters waiting and saying what for, and goods visibly piling up, may make it plain enough how loops connect.
- **Phases:**
  1. Walking only (built, M2 stage 3): the stretches of a recording where a monster just walks become "go to this tile, then face this way"; pushes, pulls, using a spot and ferry steps still replay as recorded. A looping monster finds the shortest way there around items, by breadth-first search in a fixed direction order. Of two monsters walking straight at each other, the one later in line makes way (see Getting past each other, below); one blocked by a monster that isn't moving says "EXCUSE ME!", waits a moment, then looks for a way around. A monster you steer is never moved aside.
  2. Goal-based pushes (built after M2 stage 3): a recorded run of pushes and pulls of one good is saved as one goal, "move the good lying here to there" (there may be a bin, board or wagon that takes it). Moving the same good on from where its last move left it extends that goal, and the walks in between, and the walk up to the first push, are dropped (left in, they would send a loop to stand on a pad that may hold the next good by then); a good pushed off and back again, a slip put right, records nothing. On replay the monster plans its own pushes from wherever it stands: A* over where the good is and which side of it the monster is on, costing walking, pushes and very slow pulls in ticks, over the whole map (a few milliseconds even when there is no way at all), ignoring other monsters at first; one standing where a push goes or is made from ("EXCUSE ME!") is waited on about as long as going around it would cost extra, then planned around, so a monster that just passes by never sends a loop on a long detour, and waiting never costs more than twice the best choice in hindsight. A wagon being led counts as a monster; a parked one is a wall. Pulls happen only when nothing else works, such as getting a good out of a corner. A goal already met needs nothing ("the slivers are already on the pad"); a good not there yet is waited for ("WAITING FOR THREAD"), and a good on the goal that a workshop is busy using (the loom weaving the last thread) counts as about to be gone: the next is brought right up beside it and waits there ("READY. WAITING FOR THE THREAD TO BE USED.", an hourglass in the short bubble), going in the moment the goal is free. Anything else sitting on the goal is waited out from where the good is ("THE FABRIC IS IN THE WAY"). A failed plan waits a second before trying again. A loop partway through moving a good keeps track of it as that good, not a tile: if anyone else pushes or pulls it, the loop follows it and plans again from there (and is done if they put it where it belongs), and pausing and resuming the loop keeps it too. A good pushed off its source before the loop starts on it is no longer the loop's: it waits for the next one on its own tile (see phase 3). Since a planned move ends wherever its last push does (not where a recorded pull did), every recorded Space comes after a "go to" the tile it was pressed on, facing the way the monster faced, and so does a step replayed as recorded right after a move. So a Space that finds nothing to use on replay means what it used has moved, and only wagons move: the loop waits there for the wagon ("WAITING FOR THE WAGON.") instead of giving up. Pushes while leading a wagon or using the ferry stay as recorded.
  3. **No general sources (decided 2026-11-01).** A loop takes each good only from the tile you showed it and delivers it only where you showed it, never from wherever one happens to be waiting. Those specific input and output tiles are how loops link up, and what you lay out. Loops that want the same thing take turns (see Turns below).
  4. **Goals, not keypresses (built 2026-11-01).** Phases 1 and 2 still replayed some of a recording key by key (Spaces where they were pressed, the ferry, leading a wagon), and every "go to" and recorded step was something movement had to work around. Now a loop is nothing but goals, and walking is never recorded: every goal brings its own walk.
     - **Activate a thing.** Space records what it used, not where the monster stood: this cotton plant (and the side it faced, where the harvest is set out), this workshop or bench, this wagon's tailgate (open, or shut). Each thing says where it's activated from: standing on the plant, on the workshop's work spot, behind the wagon facing it.
     - **Move a good from one tile to another,** as in phase 2. A pull while recording is just one way of moving it.
     - **Lead a wagon somewhere.** Taking the pumpkin's lead records nothing; letting go records where the wagon was left: its bed here and its pumpkin there, which is also which way its tailgate faces. On replay the monster goes to the pumpkin, takes the lead, finds a way the three of them fit (a train can't back up, so turning round takes room, and wagons don't fit the footbridge), and lets go once it's parked. Leading, Space lets go and goods can't be pushed (let go first), so leading stays a goal of its own. A loop that leads a wagon goes through its goals in order, since it's a journey; others pick the first wanted, possible goal as before.
     - **The ferry is a footbridge.** The rope project ("A ROPE FOR THE FOOTBRIDGE", 3 rope) strings a footbridge across the river where the ferry ran. Monsters cross it and push goods over it like ground; wagons don't fit and cross at the log bridge. That removed the only timed step.
     - The scheduler (what to do next), short reservations and turns are unchanged; old saves don't load (version 9).
- **Loops that never jam forever (the aim; the scheduler is built 2026-11-01, movement started over the same day).** Loops share workshops, plants, pads, doorways and each other's goods, in any tangle, copied loops included. The guarantee wanted is that nothing waits forever on another monster; not the best route every time. Built from the ground up, replacing the claims tried first (which held goods for later steps, and so held tiles in everyone's way):
  - **A scheduler, not a step counter.** A loop's steps are linked by what they make and take: a Space at a plant or workshop puts goods on a tile, a move takes a good from a tile, a workshop's Space takes its pads. Whenever a monster is free it does the first step of its loop, in the order you recorded it, that is both wanted and possible right now. A step is wanted unless where it puts things is already full of what it would put there (the rule since 2026-11-01; before, one good lying on a workshop's mat stopped it). A tile holds one good, so a harvest isn't wanted while its last harvest lies on its drop tile, nor a move while one of its good lies where it goes (one being worked there doesn't count, so the next waits ready beside it). A workshop's output holds its mat and nine more waiting inside, so a loop keeps working it until all of that is taken up with what it would make; full of something else (made from what was on the pads before), the loop waits for it to be cleared. Deliveries always are. So nothing piles up, a short harvest is made up by harvesting again, work further up the chain carries on while a workshop is busy (the spinnery spins the next thread while the loom weaves), and a loop never waits on one of its own later steps: while one step can't be done, it does another. (Pulling everything from the deliveries was tried first: it made one bolt at a time end to end, with monsters standing about "WAITING FOR FABRIC" while the loom wove.) Two consequences, noted 2026-11-01: **the order is round the loop from where the monster is**: after finishing a goal it looks first at the goals after it, in order, then round to the start. So it follows a good forward along the chain, never does the same goal again while any other goal of the loop can be done, and only gets back to the start once it passes the end or nothing further along can be done (it isn't last-to-first: at the start of a lap the first goal comes first). Goals that can't be done are passed over, though, so while a loop's early goals wait, its later ones run lap after lap: harvest, two bolls into the bin, a boll from the bin's chute onto the bench's pad, work the bench, and while the plant regrows each lap is just chute, pad and bench, so the bench works through the bin's bolls one after another between harvests. And **listing a goal twice in a row mostly just gives it a second turn**: what gets made depends on what's wanted and possible, not on how many times a goal is listed. Listed once, the second boll of a harvest still goes into the bin, on the next pass round the loop after the rest has had its turn; listed twice, it goes in straight away. A Space comes with the walk to where it was pressed; a run of steps played as recorded (the ferry, leading a wagon) stays one step.
  - **Short reservations.** A step reserves the good it's going for and the tiles it's about to use (where it's delivering, the work spot it's walking to) from the moment it commits until it ends, so two monsters never go for the same boll or the same pad. Goods get stable ids for this, so a good someone else pushes is followed wherever it goes. Nothing is set aside beyond the step at hand: a good another loop takes is simply made again. Examining a reserved good says who's seeing to it.
  - **Turns.** Monsters wanting the same plant, work spot or good take turns: one that wants something it can't have yet joins the line for it, and when it frees up, whoever has waited longest and can take it goes first ("AFTER YOU, WREN."). A place in line lasts until that monster's turn comes or it leaves its loop, even while it does other work meanwhile. It only waits for one that can take it right then, so a turn never holds anyone up. Four harvest loops on one cotton plant now split the harvests evenly; before, the two nearest took them all.
  - **Strays get shoved aside.** A good nobody holds (left by a stopped loop, or by hand) lying where a step delivers, on the way its good would go, or in the way of a walk, gets shoved to nearby free floor off that way first.
  - **Nothing to do says why at the source.** With no step possible, a monster reports the wait at the start of its chain of needs ("WAITING FOR COTTON SLIVERS", not the loom's empty pad), and a loop with nothing wanted at all says "NOTHING TO DO." A lap counts each time the loop passes the end of its recorded order.
  - **Movement starts over from greedy (2026-11-01).** First two loops traded tiles when they met (passing through each other); then a stack of mechanisms replaced that: giving way in a circle of waits, making way by who goes first, a search over real moves in a small window, "WE NEED MORE ROOM HERE." Soak tests found 15% of tight random maps (one-tile corridors, dead ends, pockets) still jammed for good, half of them silently, and a review found the search window too small for long dead ends, hopeless searches taking a whole frame, goods able to move two tiles in a tick, and waits that went stale after a load. Each mechanism covered some cases and the gaps between them were the jams, so they were all removed, with the keypress replay underneath them (phase 4 above).
    - **The baseline:** each monster takes the next step of its own shortest way to where its goal takes it, around walls, goods and parked wagons, and waits for whoever is on that tile ("EXCUSE ME!"). Every move is onto a tile free at that moment, so nobody passes through anybody. All of it lives in `motion.rs`, the one place monsters' moves meet (a good's pushes are still planned by `carry.rs`, waiting the same way).
    - **How the baseline did:** greedy jams whenever two meet head-on in a lane, and waits for ever behind a monster that isn't moving, even one you've left standing. 170 of 200 random grounds and 88 of 100 random tangles on the starting world jam. The tangle tests that need monsters to work around each other are marked "needs coordination" and ignored until they pass: they are the yardstick.
    - **Principle 1, go around anyone standing still (built 2026-11-01).** A monster standing still (one you've left standing, one working at a spot, a loop that's waiting, or a wagon whose leader is any of those) is planned around like a wall when there's a way around, for walks, pushes and wagon trips alike; one on the move is waited for, as it moves on. While waiting behind one standing still, a monster looks again for a way around every second. Guarantee: nobody waits for ever behind someone who isn't going anywhere when there's another way. Measured: 6 of the 18 yardstick tests now pass (going round a monster standing on the way, a push going round one, the doorway blocked by a parked wagon, two loops on one harvest, three copies of the whole chain, strays in every doorway); random grounds that jam went from 170 to 141 of 200, random tangles from 88 to 60 of 100. What's left is mostly two monsters each in the other's only way: head-on in a lane or doorway, or in and out of a dead end.
    - **Next:** principle 2, for two that are each in the other's only way: whoever has waited longest goes first, and one in its way that comes later in line makes way, passing that on down a line of others if it has to (PIBT), backing into a junction or pocket to let it by (push-and-rotate). Then an exact search to fall back on when those can't, widening until it has covered everything reachable, and only then "WE NEED MORE ROOM HERE." Some tangles have no way out without passing through (a one-tile tunnel packed from both ends), so the promise to aim for is: if a way exists, it's found; if none does, the monsters say so. Idle monsters parking at their flags out of the way is a possible later step.
  - **Measured by tangle tests (`tangle_tests.rs`):** copies of the whole chain (three, five, starting at different goals, with strays in every doorway, alongside harvest loops), forty seeded random tangles, and walkers (monsters walking between stops by themselves, with no loop) through one-tile tunnels, dead ends, doorways and round a monster standing still, plus a closed tunnel that can't be done and should be reported. Every tick of every one is checked for two things on one tile and for anyone passing through anyone. Ignored soaks run hundreds more: `many_random_tangles`, and `random_grounds` (random maps, printed when someone's stuck); `SEEDS=from..to` picks the seeds.
- **When plans fail, generalize the planner.** It is an exact search for one good at a time (other items are walls), with pulls so a single good is almost never stuck; it searches the whole map but doesn't move other goods aside. If failed plans start piling up, the fix is a more generally correct search (moving other goods aside), never one-off rules for particular layouts.

### Pushing and pulling
Goods move by being pushed, so laying out a workshop is a small Sokoban puzzle; once loops plan their own pushes (see Routines that scale), it stays a small element of hands-on play and of layout efficiency. Pulling exists to get out of a stuck spot, such as a good pushed into a corner, not to replace pushing. Hold **Shift** and step away from the good in front of you: the monster backs up and drags it along, at **2 seconds a tile** (ten times slower than a push), straining and kicking up dust. Pushing a good that no push could ever move says "STUCK! HOLD SHIFT AND STEP BACK TO PULL IT." Recorded loops can pull too, and pay the same two seconds every cycle, so solving the puzzle stays the better way for automation as well. You can't pull a good that is being worked on, or onto a work spot.

### Recipes
There is no recipe menu. What's on a workshop's pads decides what it makes. When several recipes fit, the one that uses the most of what's on the pads wins (built): lay out a hammer's ingot, fuel, rope and sticks and you get the hammer, not a knife from the ingot and fuel alone. The tech tree has 115 such overlaps. When two recipes still fit equally well (16 pairs take identical ingredients, such as clay and fuel for tiles, bricks or molds), a signboard on the workshop wall shows the current one, and walking into the board switches it; a loop records the switch like any step. The signboard comes with the first workshop that has a real tie. Empty pads show a dithered ghost of what belongs there, as they do in the prototype.

### Tools
Workshops have tool pegs on the wall, which you fill by pushing tools onto them, the same way pads work. Each tool shows a small durability bar. When a tool breaks, a looping monster stops with a "? + hammer" bubble.

### Workshops: town and frontier
There are two kinds of workshop, with a clear trade-off between convenience and being close to raw materials.
- **Town workshops** are permanent: they never wear out, have a fixed quality of around 0.8, and are far from raw materials. Their pegs hold basic permanent tools of modest quality, around 0.5, so a new player can craft straight away; bringing better tools improves the results. Nobody builds inside towns, which keeps them fixed and easy to read.
- **Frontier workshops** go anywhere outside town limits except on roads, for example next to a resource. Each kind has a fixed, pre-designed layout (its pads, pegs, work spot and output mat), like the town workshops; players choose where, not how.
- **They come from kits** (decided 2026-10-31). A workshop kit is an ordinary good crafted at a workbench, so it gets a quality roll, skill learning, shares and its varieties like anything else. You push or cart it to where you want the workshop (kits are heavy: a job for wagons). Facing a kit shows its footprint as a ghost on the ground, red where something's in the way; Space unpacks it, turned to face the way it was last pushed. The kits' recipes are new goods, for approval.
- **What the kit decides:**
  - its **quality** becomes the workshop's quality, which counts as a weight-8 tool in the quality roll of everything made there;
  - its **material worth** (the base values of the materials in it, unaffected by quality, so a well-made kit doesn't count twice) makes the workshop work faster, gently: say a hemlock shed at town speed, an oak one about 20% faster, with diminishing returns. Speed only;
  - its **main material** gives the workshop its color and name ("OAK SPINNING SHED"), as tools take their wood's; until per-region palettes, each wood borrows an existing wood-toned ramp;
  - its **weight** is what makes hauling the kit hard;
  - its **makers** (crafter, haulers, toolmakers) hold the workshop's 8 shares in everything made there.
- **Wear counts time worked, not jobs done,** so a fast workshop never wears out faster than a slow one; it just gets more done, and one that often waits for inputs wears less per hour. Per-job wear would also have worn quick-recipe workshops out in minutes.
- **No repairs at first: a worn-out workshop needs another kit.** It stops working; pushing a new kit of the same kind into it renews it in place, with the new kit's quality, worth, color and makers, and the same layout, so loops around it keep working. Left worn out long enough, it falls apart and frees the ground.
- **Reasons to build:**
  - shorter physical loops, because distance is time;
  - possibly higher quality;
  - workshop shares: builders earn weight-8 shares whenever *anyone* uses their workshop.

  Wear-out also clears abandoned workshops out of a persistent world.
- **Workbenches.** Recipes that don't need a workshop use a small, cheap bench placed anywhere. Benches wear out too, and towns have a few public ones.

### Joining the commune
There is no character-creation screen. A **gathering hall** in town holds monsters hoping to join. Each one is already rolled with a monster type and 3 transferable skills. Walk up to one and their bubble shows type, skills, stats and the renown they join at. Space welcomes them. You choose who joins, not which boxes to tick, and the hall's visitors change over time. Nobody is hired or paid: renown is the commune's standing, and a well-known commune draws newcomers. (Internally the code still calls them applicants.)

### Renown and goals
Renown is a HUD counter. You spend it in the world: hiring at the hall, and contributing to **town project boards**. A project board shows what goods it wants and a progress bar. In 1.0 boards count goods of any quality, so a low-quality good always has somewhere useful to go. Finished projects unlock things like the road to the second town or new workshops. A final "great work", for example a temple bell, ends 1.0.

### Delivery and shares
Delivering a good is a small celebration. "+12 RENOWN" rises from the drop-off, then smaller credits fly to each monster that contributed ("MOGG 6, BRUNA 4"). This is how the shares system becomes visible. All renown still goes to the one commune.

### Roster
Tab cycles monsters. A monster that's off-screen gets an **edge marker**: an arrow at the screen edge with its state icon (looping, waiting, idle), so a stuck monster is noticeable from anywhere. Holding Tab shows a strip of portraits with states in the HUD.

### Skills
Skill growth is shown as it happens. A small "+SPINNING" floats up when a skill improves, and a skilled monster visibly works faster: the progress bar fills quicker. Exact numbers are on the monster's inspect card.

Skills are shown in **points out of 1000**, like a batting average: a skill of 0.312 reads "SPINNING 312", and a craft that teaches 0.019 reads "+19". Underneath they stay 0..1. Gains under one point keep a decimal ("+0.4") so a small gain doesn't read as none. Quality keeps its decimal scale, since 1.0 means "standard" and it can go above it.

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
  - what is holding quality back most, e.g. "SECONDARY SKILLS (520): A TALENT FOR HANDCRAFTS OR MUSIC SKIPS THE WEAKEST".

  It is computed from the items actually on the pads and pegs, so putting in a better tool updates it straight away. While a monster stands on a work spot, the HUD status line also previews its expected quality.

**Two pages, led by icons** (since M2 stage 1). The first page holds the few things that decide what to do next; while E is held, Tab turns to the details (Tab doesn't switch monsters then), and every new press of E opens page 1. Short cards (boards, the ferry, signposts, bins) are one page. Each line can lead with a 7×7 pixel icon that means the same thing on every card: a star for quality, an hourglass for time, the good's own icon for what comes out, an amber "!" for what holds it back, a green up arrow for learning, a grey down arrow for forgetting, a sparkle for renown. Under a quality sits a meter on the 0–2 scale: a band for the usual range, a bright tick for the expected roll, and marks at standard (1.0), exceptional and masterwork. Page 1 of a workshop forecast is quality and meter, time and output, what holds it back, and the main skill gained; ranges, odds, other skills, forgetting and levelling off are on page 2. An item's card only says what it's worth in renown if a drop-off or project actually takes it.

The card is drawn in the same beveled style as the help overlay. It never pauses the world.

### Wagons and roads
Wagons are fully physical, and roads leave the map edge at signposts naming the next town.

Built in M2 stage 3: the town wagon and its pumpkin wait in Pinewater's southeast corner, by the cloth store. Space facing the pumpkin takes its lead (Space facing it again, or with nothing else to use, lets go). The pumpkin follows a tile behind its leader and the wagon a tile behind the pumpkin, snake-like; they block the way like anyone else, and a monster leading them can't pull goods or board the ferry. Pushing a good into the wagon loads it (one kind at a time, up to its room); Space at its back opens or shuts the tailgate, and a parked wagon with its tailgate open sets its goods out behind it, straight into a bin there if the bin takes them. While leading, recorded steps stay literal, since the route is what moves the wagon, and every loop starts facing the way its recording did. The town wagon holds 6 storage volume and pulls 25 weight per point of its leader's CHA, times (1 + wagon driving), at full speed; heavier loads slow it a tick per tile for every half of that. Goods push slower the same way when heavier than 10 × STR × (1 + hauling). Pushing practices hauling and leading a loaded wagon practices wagon driving, a few game seconds per tile.

Decided 2026-10-31:
- **Something pulls the wagon, and the monster leads it with CHA.** CHA was always meant to drive wagons (the original rules gave wagons a capacity of (1 + wagon driving) × CHA); it needs more uses, and leading a draft creature is one. The puller is a magical walking pumpkin: not a monster, it doesn't join or craft, it just follows a charismatic leader anywhere. Later pumpkins can grow in a patch; at first the town lends one with its wagon.
- **The monster walks ahead, leading the pumpkin,** and the wagon rolls behind it: three tiles in a line. Wagons don't fit the footbridge (a two-tile ferry until 2026-11-01), and that's fine: the bridge is what opens the river to them.
- **Pulling power** is the wagon's power (from its value) times (1 + wagon driving) times the leader's CHA. Load beyond it slows the wagon down; it never refuses goods.
- **A wagon is a dispenser.** Goods pushed into it go into its bed; parked, it sets them out one at a time like a chute. **A dispenser holds one type of good.** Bigger wagons, later, have several dispensers.
- **A wagon's power comes from its value,** so a better-made wagon (quality, oak over hemlock, more steps) pulls more.
- **The town lends a wagon at first,** like its tools. Later, wagons of different qualities and sizes are built.
- **A bridge comes from a project** (built), away from the ferry: a second road leaves Pinewater from its southeast corner, by the cloth store, and crosses the river further south. Monsters deliver about 12 logs to its board on the far bank, where the woods are; once it's built, two tiles wide so wagons can pass, wagons drive straight across. The footbridge keeps carrying monsters on foot.
- **Weight only has to feel right.** A log may weigh as much as 180 cotton bolls; even bolls are pushed around Sokoban-style.
- **Hauling earns shares when a good is used, not each time it moves** (built). A good remembers who has pushed or pulled it since it was made. When a workshop uses it, or a drop-off or board takes it, those haulers get 1 share between them, split by the tiles each pushed, and the share flows on into whatever the good becomes. If it reached its town by wagon, the wagon's leader gets 2 more, as in the original rules. Pushing a good in circles or shuffling it between pads pays nothing extra; a good pays at most 3 hauling shares before it becomes something else. Skill is different (decided 2026-10-31): hauling and wagon driving grow per tile as the work is done, like crafting skill grows with time at the bench. Practice is practice, and pointless circles already cost forgetting.
- **Used-up inputs don't count toward weight** (built). An input filling a fuel, lye, lime or dye slot is marked spent in the good's history: it still counts toward value, not weight. A dyed bolt now weighs what a plain bolt does (its storage volume was also corrected to match).
- **Wagon Wright is back** as an applied skill (it was in the original skill list), and the wagon's production tree (wheels and the rest) is designed later. Its new goods need approval like any others.

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
- **Ungraded goods have no quality anywhere in the game.** 46 goods are marked `has_quality: false`; for them there is no number, no tint and no quality forecast. Mechanically they pass their inputs' quality through unchanged, with raw ungraded goods counting as 1, so they never drag later steps down. *Changed:* the original averaged inputs and tools, which silently lowered downstream quality (slivers at 0.75 cost an expert's thread about 14%). No ungraded good can currently be made from a graded one; if one ever can, that is probably a tech-tree mistake, and pass-through keeps it harmless.
- **The simulation uses its own seeded random numbers,** so a saved world replays identically.

### How the rules play in the world (M1 stage 2)
- **Pace:** fast mode runs 4 game seconds per tick, so a real second is about a game minute. Harvesting cotton takes about 2–3 seconds, spinning 3–5, and weaving 20–40, depending on the monster.
- **Harvested plants work like a chute.** A harvest gathers the whole rolled quantity at once. The plant sets the goods out one at a time on its drop tile (the tile the harvester faced) as that tile clears, shows how many are left, and regrows once it is empty. Space on a plant that still holds goods simply succeeds. Space on a plant is always recorded, ripe or not, so a recorded "Space, push, walk back" waits for regrowth when it has to, and yields one good per cycle however many a harvest rolled. Like a chute's mouth, the drop tile works as an output square, so routes should avoid crossing it.
- **Ungraded goods roll their quantity,** so skill shows in how many come out. *Changed:* 12 ungraded crafted goods had a fixed quantity of 1 (slivers, bast fiber, lye, coke and others); all now roll, like ungraded raw materials always did. A test keeps the rule.
- **One item per tile, no stacks.** Every unit keeps its own quality, history and shares. Where extra units wait, a small number shows how many.
- **Workshops hold finished goods** until an output mat is free, then set them out one at a time. They keep up to 9 waiting inside (`OUTPUT_BUFFER`) and keep working while goods wait, so a loop that takes one per cycle never jams. A full workshop stops and says so. A loop never adds to a full workshop for its own later steps: its scheduler only makes what something downstream needs (see Routines that scale).
- **Built-in tools show on the pegs** as faint outlines. A tool pushed onto a peg is used instead, and wears out.

### The visible layer (M1 stage 3)
- **Quality on sprites:** five brightness steps, made by shifting palette indices within their ramps, with dithering for the half steps. The breakpoints are 0.25, 0.5, 0.9 and 1.25. Exceptional goods (1.25 or better) twinkle every two seconds; masterworks (1.5 or better) twice as often.
- **Skill popups:** "+SPINNING" rises over a monster when a job finishes.
- **Hold E:** a card about what the monster faces: an item, a monster, a plant, a bin, or the town tool on a peg. With nothing ahead, it describes the monster itself. At a work spot it shows the forecast: expected quality and its range, the odds of exceptional and masterwork results, the weakest factor and what would help, time per craft, learning per craft, forgetting, and where the main skill levels off.
- **Plants forecast too.** Examining a plant shows its state plus what a harvest would give the monster you control: how many goods (and their expected quality if they are graded), how long it takes, and what it teaches.
- **HUD preview:** an idle monster at a work spot shows its expected quality in the status line.
- **First craft:** a bubble suggests holding E to examine things.

### Progress (M1 stage 4)
- **The cloth store is a drop-off.** A good pushed in pays renown: its value times `RENOWN_PER_VALUE` (10, a first guess; early cotton fabric pays about 20). "+N RENOWN" rises from it, then each contributor's share flies to that monster as credit. Monsters keep a running total of renown earned, shown on their card.
- **The gathering hall** (then called the hiring hall) sits in Pinewater's southwest screen. Three applicants stand there, each a monster type with three rolled talents. Facing one shows their pitch; Space hires them if you have the renown. A new applicant arrives about 20 seconds after a hire, once the stand is clear, and every 5 minutes one applicant leaves for someone new. Applicants block the way like anyone else. For now they are goblins and cyclopes only, the types with sprites.
- **Saving** works as designed above: a title screen lists your worlds; the game autosaves every minute and saves on quit; writes are crash-safe with two backups; a test saves a world mid-loop, reloads it, and checks the two stay identical for 3,000 ticks. Still to come: the per-world settings (time scale, offline catch-up).

### Learning and forgetting
Unchanged from the original, plus the Python port's two fixes: skills never read below 0, and learning counts per craft rather than per whole multi-batch task. Mastery takes roughly 100–200 crafts, about 30–40 minutes of a looping monster at prototype speeds. Each monster has a single forgetting counter that drains every skill it isn't practicing, so monsters specialize.

## M2: the second town

Built in four playable stages:
1. **The east road:** the wilderness and Tanglemere on the map, the rope ferry and its project board, signposts, edge markers, the Tab roster strip.
2. **Tanglemere's dye works:** the indigo chain, ending in dyed fabric at the cloth hall. At first you push fabric east by hand.
3. **Loops that walk around each other, then wagons:** walking pathfinding for recorded loops (phase 1 of "Routines that scale" below), then hitching, loading, recorded wagon trips, hauling learned by doing. After stage 2 you'll have felt the long walk.
4. **Frontier building:** blueprints, ghost outlines, built quality, wear and repair, workbenches.

### The east road (M2 stage 1)
- **The world is three region maps side by side**, 175×32 tiles (7×2 screens): Pinewater (`data/pinewater.map`), the wilderness (`data/wilderness.map`) and Tanglemere (`data/tanglemere.map`). Each region's entities are listed after the previous region's, so adding a region never reorders what an older save refers to. Buildings never straddle a screen edge.
- **Named areas,** announced by a banner when the view arrives: Pinewater; Tinless (Wuxi, "without tin": a joke for later, when the bell needs tin), with the pine river; Thatch Hills (Maoshan); the Dye Meadows; Tanglemere.
- **The rope ferry** (a rope footbridge since 2026-11-01: see phase 4 of Routines that scale). The road leaves Pinewater's east gate and meets the pine river. A two-tile boat is moored at the west bank, but it has no rope to pull it across. Its project board on the bank, "A ROPE FOR THE FERRY", wants 3 rope; each one earns renown like a delivery, and the last one strings the rope. From then on the ferry shuttles by itself: it waits 2 seconds at each bank, then crosses at 6 ticks a tile. Anything on its deck rides along, monsters and goods alike. It never leaves while someone is still stepping aboard, and it waits up to 10 more seconds for anyone standing on the landing. A recorded loop that crosses waits for the boat like it waits for goods ("WAITING FOR THE FERRY").
- **Rope** (approved 2026-10-31, the first good added beyond the book's cotton chain): 2 thread twisted by hand at the **gate bench**, a public workbench by Pinewater's east gate, on the thread's way from the spinnery to the ferry. A spinning wheel can only ply thread into cord; rope is strands twisted together again the other way, which takes hands, a hook or a stick, not a machine. So the recipe needs no tool. Every bench knows every bench recipe and what's on its pads picks one; an empty bench shows the recipe that fills its pads (the west bench's one pad cards cotton, the gate bench's two twist rope). Rope is graded, keeps its fiber ("cotton rope"), and uses STR, so a strong cyclops twists good rope. It also supplies the rope that the stone hammer, metal hammer, metal axe and flail need, which nothing made before. The book twists boat ropes from hemp, so hemp rope fits once hemp thread exists. No separate Cord good until something needs fine cord (bowstrings, fishing line).
- **Wild plants** grow along the way, each giving its good when you stand on it and press Space: hemlock and elder by the road, ramie in the meadow, rhododendron in Thatch Hills, oak deeper south, arrow bamboo and indigofera toward Tanglemere. Gatherable trees are lighter than the old dark trees, which aren't for gathering (E says so).
- **Tanglemere** stands around its tangled lake: the dye house and ash kiln (idle until stage 2) and the cloth hall, which already takes fabric for renown. An indigofera field grows east of the lake.
- **Edge markers:** a monster on another screen shows as its initial in a small box at the screen edge, pointing its way, colored by what it is doing (looping, waiting, working, recording, idle).
- **Roster strip:** hold Tab to see every monster and what it is doing. A tap still switches monsters.
- **Pulling** (see Features) gets a monster out of a stuck push.
- **Saves are version 3,** with the boards and the ferry. While the game is in development, saves don't migrate: a save from another version doesn't load, and the title screen says so.

## Milestones

- **M1, economy:** port the rules from sokobanmonstermakers, using its 92 tests as the spec, with the changes in Crafting math above. The tech tree now lives in `data/good_types.json`; give every good a stable `id`, and complete the data for the first chain. Add saving and the world list. Add quality on sprites, skill popups, the inspect card with the workshop forecast, delivery credits, the renown counter and the hiring hall. Workshops in M1 are the town's permanent ones.
- **M2, second town:** roads and signposts, wagons, town project boards, building frontier workshops and workbenches, edge markers and the Tab roster strip. Stage plan above.
- **M3, content and art:** chains 2–3 and the great work. Replace placeholder sprites with final pixel art under the AGENTS.md palette rules.
- **M4, ship:** itch.io build and outside playtesters.
- **v1.5:** equipment, charity and great-works drop-offs, and the quality sorter.

### Tanglemere's dye works (M2 stage 2)
- **The indigo chain runs in three town workshops,** each laid out so every pad can be loaded by pushes from the door and every output pushed back out of it:
  - **The kiln** (two pads) burns ramie stalks with fuel into lye, or limestone with fuel into quicklime; what's on the pads decides.
  - **The indigo house** (five pads in a row) mixes indigofera leaves, lye, two quicklime and fuel into about 3 indigo.
  - **The dye house** (two pads and a peg) dyes fabric with indigo, using the town's medium-quality tub or a better one on the peg.
  - **The cloth hall** takes plain and dyed fabric for renown; drop-offs can now take several goods.
- **Materials nearby:** limestone outcrops in the Thatch Hills (heavy grey rocks that regrow like plants), ramie stalks beside the kiln, elder bushes for fuel along its path, and the indigofera field east of the lake. Cotton fabric still comes from Pinewater, across the footbridge.
- **Dyed goods take their dye's color:** indigo-dyed cloth is drawn blue (its cotton-white palette ramp swapped for the water-blue one) and named "INDIGO COTTON DYED FABRIC", the last variety added coming first.
- **Each pad shows its own ingredient** as a faint picture, so the indigo house's five pads say leaves, lye, quicklime, quicklime, fuel.
- **Tutorial:** after reaching Tanglemere, four steps walk through lye, quicklime, indigo, and dyed fabric at the cloth hall.
- **Saves are version 4.**
- A test runs the whole chain through the real workshops: lye, quicklime, three indigo, indigo cotton cloth, renown at the cloth hall.

### Decided 2026-10-31 (from the stage 2 loose ends)
- **Tools travel with monsters.** Eventually a monster carries a tool in an item slot along with its other items, and uses it for gathering in the wild. Until then, gathering ignores tool requirements.
- **Tools nobody can make yet still appear in recipes.** A recipe lists the tools it really needs. Towns lend permanent medium-quality versions on their workshops' pegs, as they already do; once monsters carry tools, medium-quality instances can also be placed in the world, for example at a quarry. When the tool's own chain arrives, better ones simply replace them, with nothing to redo.
- **An earthenware vat** is a future tub, significantly better than the wooden one.
- **Fuel adds its value** to what it helps make: burning oak makes more valuable indigo than burning sticks.
- **Drop-offs can take several goods,** so the cloth hall takes plain and dyed fabric.
- **The indigo pits' 5 pads** get a playtest before anything is changed.
- **Forgetting stays harsh, and a beginner's crafting can lower value.** Both are intended; forgetting may be revisited.
- **Weight you can feel** is an accepted future design (below).
- **A recipe's normal yield shares its inputs' value by the square root** (built). Every unit used to carry the full value of all the inputs, so a recipe that makes 2 doubled value; dividing evenly would make multi-unit recipes worthless. Now a recipe's normal yield carries the inputs' value times the square root of that yield between them: 2 units carry 1.41 times, 3 carry 1.73, 4 carry 2. The share per unit is set by the recipe, not the roll: extra units from a good roll are each worth as much as a normal one, so a high-rolling monster makes more value in total and never less per unit. Per-unit value matters more than the total, because deep goods multiply it. A harvest's units are each their own raw material and are not split.
- **Limestone and Quicklime** (approved and added 2026-10-31). Limestone is gathered at an outcrop in the Thatch Hills: very heavy (density 250, an oak log is 100), cheap, about 2 per gather, the prospecting skill with STR, and a pick that gathering ignores for now. Quicklime is limestone plus fuel burned at a kiln, about 2 per burn, the firing skill with CON, ungraded so it passes quality through. It counts as lime, so it serves indigo, seeth water, the temple bell and the great statue. The book's inland towns burn limestone; Oyster Lime is the coast's lime and stays for a coastal region.
- **Wooden Tub, fuel and indigo yield** (approved and added 2026-10-31):
  - A **Wooden Tub** is 2 logs at a workbench, using carpentry, with a saw and a chisel that town benches lend at medium quality until those tools can be made. It keeps its wood ("oak wooden tub") and is the tub that Dyed Fabric (its tag had been split into letters), Bast Fiber and climbing hempvine stalks need.
  - **Fuel:** elder and rhododendron sticks, hemlock and oak logs, coal and coke count as fuel. 43 recipes needed fuel and nothing provided it.
  - **Indigo** yields about 3 per batch (rolled, base 3), since one vat dyes many cloths; each one carries the batch's inputs divided by the square root of 3.
  - A test checks that dyed fabric, indigo, lye, quicklime and the tub all resolve down to goods you can gather.
- **Stone and hard stone.** Limestone counts as stone, ready for building and carving. The stone hammer and stone anvil now ask for hard stone, since limestone is too soft for them (a knife scratches it). A hard stone, such as river cobbles at the Tinless ford, is a future good to propose.
- **No by-products for now.** Goods that really come from one process, such as cotton slivers and cotton seeds from ginning, stay separate recipes, chosen with the workshop's signboard. A by-product mechanic waits until there's a way to deal with unneeded by-products; the v1.5 charity drop-off could be that way.
- **Ramie stalks make ash on purpose.** Small links between industries, like a fiber crop that also feeds the dye works' lye, are wanted over strict separate chains. Rice straw, the book's lye ash, can join it once there are paddies.

## Open questions

- **Art across biomes,** and whether to hire a pixel artist before publishing. Henry likes the current style.

- **Screen-by-screen layouts** of the two towns and the wilderness, designed in M1 and M2.
- **Weight you can feel (built in M2 stage 3 for pushes and wagons):** heavier items push a little slower, scaled against the monster's STR (for example 3 ticks for a normal push and 4 for a heavy one), and wagons get a weight capacity. Oak versus hemlock then becomes a trade-off between value and hauling effort.
- **Tune in play:** economy numbers (starting renown, hiring costs, monster count, project costs), applicant rules (how many, refresh rate, pricing), and the time-scale presets.
- **Hauling and wagon skills** are learned by doing. Each tile a monster pushes cargo, or pulls a loaded wagon, counts as a few seconds of hauling or wagon-driving task time, for both learning and forgetting.
- **Tech-tree data still to fix:**
  - 62 required tag groups have no provider even after the stage 2 proposal. The biggest gaps: metal ingots and smithing tools (forceps, anvils), sickles and scythes, charcoal, the potter's wheel, mulberry leaves, berberine wood, and the bell's rosin, beeswax, oil and mortar.
  - Tags split into letters: Dyed Fabric (fixed by the tub proposal), climbing hempvine stalks, green vitriol (found).
  - Already fixed in M1 stage 1: stable ids for every good, the second "Saw" renamed "Pick", nested tool weights on Bricks and Ash Glaze, and Indigo listing Chemistry twice.
