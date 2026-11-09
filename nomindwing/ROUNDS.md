# No Mind Wing: design rounds

This file keeps the iteration pointed in one direction. Each round names the goal it serves, lists every addition with what it's meant to prove, and records Henry's verdict on each one: **keep**, **cut** or **rework**. Anything considered but not built goes in the parking lot, so it isn't lost or added by accident.

## North star

> The mothership is the king and the root node of a tree of factories. You spawn off little self-propagating wings strategically. It's "the creativity of designing a TCG deck combined with the strategic thrill of outmaneuvering someone in chess."

For that to work, the ships you design have to be the strategy. The proving ground exists to make **ship design** interesting before it's wired into a full match.

## What "interesting ship design" means (the test for every addition)

1. **No single winner.** At equal pixel cost, every strong design has a counter that is also just a drawing.
2. **Shape matters.** Moving a part, not just adding more of it, changes the fight.
3. **Big and small are different, not more or less.** Each size can do things the other can't. The vision spans 3 px fighters to 100 px capitals.
4. **You can see why.** Watching a fight tells you why it went that way.
5. **Any drawing flies.** Ship AI comes from parts and orders, never from a per-design script.

---

## Round 1: the three-layer brain (2026-11-08)

**Built:**
- A 15x15 part grid (hull, armor, flak, lance, engine, scoop, core), with every part costing 1 pixel.
- Orders: Hunt, Guard, Harvest and Hold.
- Part reflexes, and a flight style that follows from thrust per weight.

**Verdict:** the feel and AI layering are good. The parts system fails goals 1–3.
- Spread-out flak plus extra cores beats everything, whatever its shape. The strongest design was "a very ugly carrier" scattering flak and cores over a lattice.
- A "chaff" fighter wrapped in loose hull pixels also worked.
- Guns over-commit to one small target.
- Lance pulses are fine.

## Round 2: size and shape (2026-11-08)

**Goal:** make big and small ships *different* (goal 3) and make *shape* matter (goal 2), which should break the guns-per-pixel monoculture (goal 1).

| # | Addition | Serves | What to look for | Verdict |
|---|---|---|---|---|
| 2.1 | **Connected ships, one core.** Every pixel must touch the core through other pixels (edges, not corners). Pixels cut off in combat break away. | 2, kills the lattice and chaff ships | Does severing a thin spine feel like a real weakness? Do compact ships feel sturdier? | **Keep** connectivity ("definitely a good fix"; spindly ships are "appropriately weak"). **Rework** the one core: extra cores at a steeply rising cost, and split-brain ships (round 3) |
| 2.2 | **Plate** replaces armor. Flak bounces off it completely, shown as a spark. It weighs 2. Lances and cannon shells break it, and plate soaks twice as much of a lance's punch. A staircase of plate (pixels touching only at corners) stops shots too. | 1, 3 | Do fighters stop being good at everything? Do they find weak spots (bare engines, exposed guns)? | **Keep** as it is. Henry was tempted to let small guns chip heavy armor a little, but that needs an HP system (parking lot). |
| 2.3 | **Cannon**, the first big gun. A 3x3 block with a slow-turning turret whose barrel you can see. Its shell blasts a 3x3 hole through plate. It prefers the biggest target in range. | 3 | Can you see the turret fail to track fast ships? Is the cannon worth 9 pixels on a big ship? | **Keep**, but it's too easy to put on an extremely small hull. Henry expects power to fix that; fitting and power is still in the parking lot. |
| 2.4 | **Fire spreading.** Flak turrets avoid targets that already have shots on the way. | 4 (over-commit) | Do many-gun ships stop wasting fire on one fighter? | **Keep** ("looks good"). |
| 2.5 | **Bigger grid (41x41) and a scrolling arena** (480x360, with a minimap). | 3 | Does a 37 px capital beside 3 px fighters start to give the awe? | **Keep** the arena. **Rework** the yard: a step in the right direction, but Henry wants even bigger ships, and big ships need a different way of designing than small ones. Clicking pixels doesn't scale, and more shipbuilding features are coming, so the UI mustn't become a mess. Talk this through before building it. |
| 2.6 | **Bigger enemies are solid.** A ship can't fly into an enemy with more pixels than itself. It skims along the hull, and an attack run turns away when it bumps. A big ship can still plough through small ones. Added after the first headless runs: fighters were flying over the capital and shooting it from inside its plate. | 2, 4 | Does skimming look right, or should small ships steer around big ones earlier? | **Rework**, done this round: attack runs peel off before the hull, and nothing stays inside a bigger ship (see "Fixed during play"). Then **keep** ("no more clipping"). |

**New defaults:** fighter (7 px), lancer (7, a hull pixel shielding its lance), frigate (44, sealed in plate except its engines), destroyer (173, one cannon), capital (620, four cannons, sealed in plate with flak sponsons outside) and gunminer (9).

**What the headless runs showed** (`--pg-sim`, equal cost, about 600 px a side, wins out of 12 seeds; "draw" means time ran out; rerun after the lancer fix below):

| | vs fighters | vs lancers | vs frigates | vs destroyers | vs capital |
|---|---|---|---|---|---|
| **85 fighters** | | win 12 | lose 12 | draw 9, lose 3 (fighters lost 83 of 85) | draw 12 (fighters lost 79 of 85) |
| **85 lancers** | lose 12 | | win 12 | win 12 | win 12, in 7 s |
| **13 frigates** | win 12 | lose 12 | | win 12 | lose 7, win 5 |
| **3 destroyers** | draw 9, win 3 | lose 12 | lose 12 | | lose 10 |
| **1 capital** | draw 12 | lose 12, in 7 s | win 7 | win 10 | |

- **The first version of this table was single fights, and one row was luck.** It said frigates lose to destroyers; over 12 seeds they win 11 (12 since the lancer fix). `--pg-sim` now runs every matchup 12 times.
- **Goal 1 holds for the first time.** Every design has a counter, and there's a real triangle: fighter beats lancer beats plated ship beats fighter.
- **The default destroyer wins nothing.** It loses to frigates, lancers and the capital, and draws with fighters. That's this drawing, not necessarily destroyers.
- **Shape mattered more than expected.** The first default capital had bare hull behind its flak sponsons and engines, and 30 fighters dug through to its core in 7 seconds. Sealed in plate, it shrugs off 85 fighters.
- **Settled: the single core pixel stays.** Lance shots tunnel through plate, so 40 lancers (280 px) kill the 620 px capital in 4 seconds. Henry's call: the lancer is doing exactly what it's supposed to, and an effective counter is a good signal at this scale, not a bad one. Balance can wait for bigger fights (parking lot).
- **Open question: lancers still dive into flak after they fire.** Their drawing says "attack runs", so after a shot from 100 px they carry on into the target's flak. Kiting would be a sub-order (parking lot), and Henry wants lancer flight to be choosable anyway. Why they used to hold their fire is under "Fixed during play".
- **Open question: crippled ships stall the fight.** Every fighters-vs-big-ship draw is the same story. The fighters strip the capital's 8 engines within 12 seconds, because its engine row sticks out behind the plate. The capital strips the fighters down to hull and core. Neither side can reach the other, so nothing happens until the 3-minute limit. Every draw in the 12-seed runs is still this stall (checked after the lancer fix: 1 in test 2, 6 in test 5, 4 in test 7). In each one, a side has lost all its guns, or both sides have guns and no engines. Options:
  - A ship with no guns and no engines is a wreck and counts as dead. (This wouldn't end most stalls: the capital keeps its guns, and the fighters keep their engines.)
  - A fight ends after some seconds with nothing shot off.
  - Leave it: both sides crippled is a real result.
  - **Decided:** this only matters in the arena, which isn't the planned main game. In a real match a disabled ship stays disabled, and the match goes on around it. Henry: disabling a ship by destroying its working pixels, even when you can't get to its core, is a neat feature, and stalemates need "some type of tie breaker ... or just allow draws." So draws stay, but a stalemate ends early: after 20 s with nothing shot off (counted from the first hit, so slow ships still closing in aren't cut off), the fight ends as a draw. The result line names any side that has no working guns left, so a disarmed enemy shows up in the log and the sims without being called a win.

**Fixed during play:**
- **2.3: the cannon kept missing small ships that sat still** (Henry, test 5). Shells flew along the barrel, which has only 64 angles, about 8 px apart at 85 px, so a 3 px hulk between two angles could never be hit. Once the barrel is on target, the shell now flies at the aim point itself. The barrel looks the same, and fast ships still outturn it.
- **2.6: fighters could end up inside a big ship and stay there** (Henry). Fighters got inside when the capital drifted onto one or when one turned in place into the hull. Once inside, a fighter could move anywhere, including deeper, and could die in there. Now a ship that finds itself inside heads for the nearest way out, and a turn can't swing a pixel into a bigger enemy's hull. In test 5's first minute, time spent inside dropped from 54 ticks (25 in a row) to 7 (2 in a row).
- **2.6 rework: attack runs peel off** (Henry: "should we consider having them turn to avoid it?"). A run used to break off by flying straight on, which against a capital meant straight into the hull and then skimming along it. Now a ship on an attack run looks ahead about as far as it needs to stop. If its course would hit a bigger enemy, it turns sideways, away from that ship's centre: a strafing pass. Bump ticks dropped from 87 to 3 against a plate slab, and from 100 to 2 in test 5. Side effect: in test 6, lancers no longer park against the hull drilling one tunnel to the core. They make passes, so the capital dies in 43 s on average instead of 5. Henry: that doesn't matter, because the lancer's flight will be choosable anyway. The lancer fix (next) brought it back to 13 s.
- **Lancers held their fire** (Henry, test 2: "often failing to take shots", worse when they start closer). The firing check worked; the dive didn't. A lance fires only straight ahead, like a rook, but the ship flies freely with only 4 facings. The dive headed for the target's centre from wherever the lancer was, so a lancer 10–20 px off the frigate's row stayed off it until it was nearly touching, then peeled away. Over 12 runs, lancers spent 2,681 ticks with a loaded lance and the frigate in range ahead but off their line, and fired 70 times. In one close start, 4 of 6 dove the whole first pass loaded and never fired. Now any ship with a working lance, on an attack run, slides onto the target's row or column first and then dives along it, facing the target the whole time ("LINING UP" on hover). The slide gets the speed it needs first, and what's left closes the distance. It leads a moving target by the time the shot takes. Stand-off uses the same two helpers (the pixel to aim at, and where the ship must sit for its lance to bear), so there's one piece of lance code for every flight style, and nothing per design. A test runs three lance layouts (centred, offset to one side, wide hull) from three starting points; all of them fire their first shot from 95–100 px, where the old dive fired from 60 px or not at all.
  - Test 2 at three starting distances, wins out of 12: 180 px (the test) went from 3 to 9, 110 px from 12 to 10, and 70 px from 1 to 11.
  - Elsewhere: test 6 now ends in 13 s instead of 43. Frigates (lance ships that stand off) now lead a moving target, so they beat the destroyer more often in test 4 (10 wins instead of 7). Every equal-cost result held.

**Round 1 designs** were saved to `proving_ground_designs_round1.txt`. Most of them are illegal under 2.1.

**Henry's notes:**
- Test 5: the cannon kept missing crippled fighters (fixed above).
- Test 6: lancers killing the capital isn't a bad thing. "The existence of an effective counter like this I see as a good signal not bad."
- 2.1: spare cores scattered everywhere were too cheap, but exactly one core per ship is too strict. Allow more cores at a quickly rising cost. A ship cut in two, with a core in each half, becomes two ships: "super cool".
- 2.6: some fighters still seem to get inside big ships and then disappear. They bump right into the hull, which looks a little weird; should they turn to avoid it? (Measured: in the first minute of test 5, fighters got inside the capital 5 times, 4 of them because the capital drifted onto one. One died inside.)
- Engines can go anywhere; nothing requires them on the outside. Henry: they should have to be exposed. A campaign could later offer a special reactionless drive that works inside armor.
- Spindly ships seem appropriately weak.
- Lancers taking 43 s instead of 5 doesn't mean much, since we want to be able to choose the lancer's AI anyway. A real game might want a button per ship type to switch its AI live.
- 2.2 plate: good as it is. Small guns doing a tiny bit of damage to heavy armor is tempting, but it needs a whole HP system.
- Test 2: are lancers bugged? They often don't take shots, more so when they start closer (diagnosed above).
- 2.3 cannon: too easy to put on an extremely small hull. Power should help.
- 2.4 fire spreading looks good.
- 2.5: the arena yes. The yard is a step in the right direction, but we may need a different way to design large ships than small ones. "I want even bigger." More shipbuilding features are coming, so we need to keep the UI from becoming a mess.
- 2.6: no more clipping.
- Lancers fire like a rook but fly smoothly, so they should use that to line themselves up on attack passes. "Make sure it's a generalizable fix, we don't want to be having to debug every ship-AI combination." (Done: see "Fixed during play".)
- Exposed engines: an engine should need a 3-wide path to the back of the sprite.
- Crippled ships: what's the decision here? "It's a neat feature that you can disable a ship by destroying its functional pixels even if you can't get to its core."
- Exposed engines: the back-only reading is right, but maybe extend it to any 3-wide path out of the square. "We don't want to be too restrictive."
- Crippled ships are really just a question about the arena, which isn't the planned main game mode. Stalemates need some tie breaker, or just allow draws.

**Queued for round 3** (agreed after the round 2 verdicts): split-brain ships, extra cores at a steeply rising cost, exposed engines and stalemates ending early. All four were built in round 3.

---

## Round 3: cores and engines as shape decisions (2026-11-08)

**Goal:** make *where* the cores and engines go a shape decision (goal 2), and let big ships buy redundancy that small ones can't afford (goal 3). Also, an arena-only fix so stalemates don't idle for 3 minutes.

| # | Addition | Serves | What to look for | Verdict |
|---|---|---|---|---|
| 3.1 | **Split-brain ships.** When a ship is cut apart, every piece that still holds a core flies on as a ship of its own, with its own guns and engines. The biggest piece keeps the original. Pieces with no core break away as before. A ship dies when its last core goes. | 2, 3 | Draw a ship with two pods on a thin spine and a core in each, then cut the spine. Is that the "super cool" moment? In the defaults it only happens as scrap: lancers carve a 3–13 px piece around one of the capital's cores, with no engines, and it dies in seconds. | **Keep**: rare, but "pure upside". |
| 3.2 | **Extra cores at a steeply rising cost.** The second core costs +8, the third +16, the fourth +32. A ship's cost is its pixels plus its extra cores; the yard and roster show it, and the equal-cost sims use it. The cores must all be in the one connected piece. | 1, 3 | Is the curve right? A second core costs more than a whole fighter, while the default capital now carries three (fore, middle, aft) for +24 on 620 px. | **Keep**: "approximately correct for now"; tweak later. |
| 3.3 | **Engines must be exposed.** Every engine needs a 3-wide way out of the ship: a 3x3 gap against one face of the engine that can slide out past the ship's edge, in any direction and around corners. Buried engines show red in the yard, and the ship can't fly. | 2 | Does engine placement become a real choice without being fiddly? Does a fast ship's soft stern show up in fights? | **Keep** ("seems fine"). The only complaint: the buried-engine warning ran off the side of the yard (fixed, below). |
| 3.4 | **Stalemates end early** (arena only). Once something has been shot off, 20 s with nobody firing, nothing shot off and no armed ship getting closer to an enemy ends the fight as a draw: a "stalemate". The result names any side with no working guns left. | 4 | Do the stalemates you see look like real dead ends? | **Keep**: no problems, and it only matters in the arena, which isn't planned as a main mode. |

**Default changes:** the destroyer's two rows of engines were one engine behind another, so 2 of them were buried. It now has one row of 8. The capital has two more cores. A saved design that's still exactly an old default (your destroyer was) swaps to the new one when it loads; edited designs are left alone.

**What the headless runs showed** (12 seeds with the default designs, after everything under "Fixed during play", compared with round 2 after the lancer fix):
- **Disarmed ships no longer stall fights.** Since gunless hunters stay with their group (below), they follow their armed friends into the enemy's guns, and the fights that used to idle end. Test 5 (capital vs 30 fighters) is 12 capital wins in 0:27 on average (round 2: 6 wins, 6 draws, 1:51). Fighters vs the capital at equal cost is 12 capital wins in 0:58 (round 2: 12 draws at 3:00). No default matchup ends in a stalemate now; some still run the full 3 minutes (3 of 12 fighters vs destroyers).
- **Fixed while building: the first stalemate rule cut off real wins.** "20 s with nothing shot off" turned 3 of test 5's capital wins into stalemates, including the window's fight, which became a stalemate at 0:43 instead of a win at 0:48. In each one the capital, down to one or two engines, was creeping toward the disarmed fighters out of range for 22–32 s. Counting shots fired didn't help (nobody was in range), so an armed ship closing on an enemy now keeps the fight going too.
- **Splits happen without being designed for:** 7 times over 12 runs of test 6, and 10 at equal cost (8 against lancers, 2 against frigates). All are off the 3-core capital: small scraps that lance tunnels carve around a core. They don't change who wins; lancers still kill the capital every time, in 16 s.
- **The triangle holds** at equal cost: fighters beat lancers 12–0, lancers beat frigates 12–0, frigates beat fighters 12–0. Lancers now win test 2 every time (9 of 12 before), destroyers lose all 12 to the capital (2 wins before), and frigates beat destroyers 8–4 (12–0 before).

**Fixed during play:**
- **Shots could get past plate when a ship moved or turned** (Henry, test 1 with a three-core dumbbell fighter: "some split brains retreat"). Flak never broke plate in any version; the bug was flak reaching what the plate guarded without going through it. Ships move first each tick, then shots fly, and a shot only checked the pixels it stepped into. In the window's fight at 0:07, a pod moved 1 px onto a flak shot resting against its plate, the shot's next step went straight into the gun, and the gunless pod ran home.
  - My first fix (a shot also checks the pixel it sits on) was only a patch. Henry: shots "shouldn't be able to get anywhere that would require teleporting through something", and shot-movement edge cases shouldn't decide what they destroy. An exhaustive check agreed: a gun and core sealed in a ring of plate, flak from every spot within 8 px flying 10 ways, the pod facing each way, moving up to 3 px and turning or not (232,680 cases). The original code let flak reach the inside 998 times (310 moving, 688 turning, never while holding still); the patch still let 900 through.
  - **The fix:** each shot is tested against each ship in that ship's own frame. Its path relative to the ship this tick, from where it was before the ship moved or turned to where it ends up, is walked a pixel at a time, with the diagonal-squeeze rule. A path from outside always meets the outer pixels first, so no move or turn can carry a shot past plate. The exhaustive check is now a test, with 0 leaks, plus a test of the exact case Henry saw.
- **Gunless hunters stay with the group** (Henry's call, replacing "runs home when hit" from round 1, for Hunt ships). A Hunt ship with no guns left keeps beside the nearest friend that still has guns, on the side it's already on, or beside any friend if none do; with no friends left it stays put. Hover shows "NO GUNS: STAYING WITH THE GROUP". Guard and Harvest ships still run home when hurt with no guns.
- **The buried-engine warning ran off the side of the yard** (Henry, verdict on 3.3). "1 BURIED ENGINE: NEEDS A 3-WIDE WAY OUT" didn't fit the status line; it's now "1 BURIED ENGINE: NO 3-WIDE EXIT", and hovering the line still explains the rule. The arena's "CAN'T FLY" notice had the same problem with any long reason, so it now wraps inside the field. A test checks every status line and notice at its worst case (every cell of the grid loose or buried, a 10-letter name).

**Henry's notes:**
- Test 1 with easily split fighter hulls (three cores: two plated pods on a spine, plus a core in the middle): the frigate can't seem to destroy them even though it has a lance. Dodging, or a bug? They don't seem to dodge on purpose.
  - Measured on the window's fight: neither. The frigate's lance hit with 11 of its first 15 shots, often hitting whichever dumbbell was in front of its target. Each hit only takes a few pixels off a 45 px, three-core ship; plate halves the lance's punch, and killing one core doesn't kill the ship. The dumbbells shot off all 3 of its bare engines by 0:44. A ship with no engines can't move or turn, so it sat facing north for over two minutes, firing only at ships that crossed in front of it. Its flak bounces off the pods. 4 of the 15 shots missed, at least 2 because the target turned: a quarter turn swings a 21 px-long ship's pixels up to 10 px at once.
  - Across 12 seeds, every one of these fights is a 3:00 draw, still true after both fixes. Neither side can finish the other, but it isn't called a stalemate, because the dumbbells keep firing flak that bounces off the frigate's plate, and firing counts as still fighting.
- Some split brains retreat; maybe a bug? Partly: the shot bug above disarmed a pod sealed in plate. The rest were pieces with no guns left following the round 1 rule, run home when hit. Henry: they should **stay with the group** (done above).
- Henry: "it should behave intuitively with physical logic: shots shouldn't be able to get anywhere that would require teleporting through something and nothing to do with shot movement edge cases should change whether or not they destroy armor." (My first write-up said ships flying into shots are hit "armored or not", which read as flak breaking plate. It never does: flak bounces off plate.)

---

## Round 4: a yard for big ships (2026-11-08)

**Goal:** make a 100 px ship as quick to build as a fighter, in a yard with room for the 25 or so part types to come (goal 3), before any new parts are added.

**Decided in the talk before building:**
- **Ships go up to 100x100.**
- **Modules are stamps.** Copying a design or a region onto a ship makes plain pixels; nothing stays linked. Henry: 100x100 "is not so many that you would never want to edit it manually", and it would "muddy the waters" if editing a module changed the ships that used it. My first pitch said live modules would serve the deck-building and factory-tree ideas; Henry pointed out those ideas stand however a ship is painted. Launching sub-ships from a big ship can be designed on its own later (parking lot).
- **One editor with tools**, not a second big-ship editor.
- **The yard's text readouts become icons**, freeing space for a bigger canvas.
- **Parts get sorted into categories.** More are coming: factory, guns, power, maybe shields, more armor types, a second collector, network nodes.

| # | Addition | Serves | What to look for | Verdict |
|---|---|---|---|---|
| 4.1 | **A 99x99 grid with whole-step zoom.** 1x is the ship at its real size (drawn as in the arena), 2x is the whole grid, 4x and 8x are for fine work; at 8x each cell shows its part's shape. Q/E, the mouse wheel or the -/+ buttons zoom around the pointer; WASD or the arrows scroll. A design opens at the biggest zoom that shows all of it. | 3 | Does moving between 2x and 8x feel quick enough to draw a big ship's outline and then detail it? Is 1x useful as the preview? | **Keep.** |
| 4.2 | **Rectangle and fill tools.** Drag a box to fill it (right-drag empties); cannons tile it in 3x3 blocks. Fill takes a hole inside the ship, or swaps a patch of one part for another; open space that reaches the edge won't fill. Mirror works with both. | 3 | Can you rough out a hull, plate it and fill it in a minute? Does the fill refusing open space ever get in the way? | **Keep.** |
| 4.3 | **Stamps.** C, then drag a box to pick it up; or right-click any design's tab to pick up the whole design. A dithered ghost follows the pointer, T turns it, a click places it as ordinary pixels, right-click puts it down. With mirror on, a flipped copy lands on the other side. | 3 | Is building a big ship from your own turret pods and engine blocks the shortcut it should be? | **Keep** ("stamps was the right choice"). |
| 4.4 | **Six part categories**, keys 1–6, pressing again for the next part: hull & armor, weapons, drive, power (empty until its parts arrive), industry, command. Each part's swatch has a small shape in it. | 3 | Does it still feel quick to switch parts? Do the shapes help tell parts apart? | **Keep.** Once a category outgrows one row of part buttons, the layout will need some shuffling. |
| 4.5 | **Icon readouts.** Cost and the status line stay as text. Flight style is an icon and a word; speed, turning and thrust per weight are gauges with their numbers (the gold mark is 1 thrust per weight, where attack runs start); the part lines became a tally of swatches with counts. Everything explains itself on hover. | 3 | Can you read a ship at a glance? Is anything you used to read now missing? | **Keep.** |
| 4.6 | **TITAN, a 97 px default** (7th slot): plated two deep, nine cannons, flak on sponsons and behind the plate, three prow lances, three cores, 13 engines. Two new tests: 1 titan vs 5 capitals, and vs 40 lancers. | 3 (the awe) | Does a 97 px ship beside 3 px fighters give the awe? | **Keep** ("impressive"). Henry is tempted to go even bigger, but 99x99 stays for now. |

**Judgement calls while building** (say if any are wrong):
- **99, not 100.** An odd grid keeps a centre column for the mirror. Ships are at most 99x99.
- **Undo (U, and a button), 64 steps deep.** It wasn't in the plan, but a misplaced fill or stamp on a 99x99 grid is too costly without it.
- **A 7th design slot** for the titan, rather than replacing one of the six. The yard's tabs lost their little ship icons to fit seven names; the arena's roster entries got a little shorter.
- **The acceleration line became the thrust-per-weight gauge.** It's the number that decides flight style; the time to top speed is in its hover text.
- **Status lines got shorter** to fit the narrower column: "3 LOOSE PIXELS", "2 ENGINES: NO 3-WIDE EXIT", "NO GUNS: STAYS WITH ITS GROUP". The test from round 3 still checks that every one fits, now for every cell of a 99x99 grid.
- **Save files store each design trimmed to its drawing**, with its place on the grid, instead of the whole grid (a 99x99 grid would be 10,000 characters a line). Old files load centred, so Henry's designs carry over.

**What the headless runs showed** (12 seeds each):
- **Nothing else changed.** Every round 3 matchup gives exactly the same results, so the bigger grid didn't touch the fights.
- **The titan kills lancer swarms and loses to equal-cost big ships.** 1 titan vs 40 lancers: titan 12–0 in 0:21, losing 287 of its 3,069 px. Its cores sit 15–20 px deep, beyond a lance's 6-pixel punch, and its 32 flak shred them (40 lancers kill the capital in 16 s). 1 titan vs 5 capitals: capitals 12–0 in 1:43, losing no ships. At its own cost (3,093 px) it loses to 4 capitals (10 wins, 2 draws), 17 destroyers (8 wins, 4 draws) and 70 frigates (12 wins). Five capitals bring 20 cannons to its 9, and a 97x41 ship is the easiest target in the arena.

**Fixed during play:**
- **Engines need a straight 3-wide way out** (Henry, changing 3.3). His capital wrapped plate round its engines and let the exhaust out through a channel that turned a corner, which the round 3 rule allowed. "The point of making engines be exposed is allowing fighters to have something they can do vs. armored capital ships, and this defeats that." Now it's a rook's move: in one of the four directions, the engine's own line and the lines either side of it must be empty right out past the edge of the ship, so there's always a straight line of fire to every engine. A straight well, however deep in plate, still counts; a bend or a pixel diagonally behind the engine doesn't. Every default still flies. Of Henry's saved designs only that capital is affected: all 6 of its engines show buried.
- **Cannons stay on their target among equals** (Henry, watching the titan against five capitals: "its guns do one volley against one target and then spin all the way over to another"). A cannon took whichever enemy in range had the most pixels left, so every hit made its target smaller than the four untouched capitals and sent the barrel swinging round to one of them. Measured over the first minute: the titan's nine cannons changed target about 180 times while firing about 130 shells. Now a cannon goes for the big ships (anything in range at least 2/3 the size, as built, of the biggest), and of those takes the one its barrel reaches soonest, so it keeps shooting the ship it's aimed at. Barrel movement fell by about 70% (1,150 steps to 365 a minute) and target changes to 20–60; a test checks a cannon between two equal hulks never changes target. This is every cannon, not just the titan's. 12 seeds each:
  - The titan still loses to 5 capitals (0–12), but takes more off them (1,026 px, up from 830). At its own cost it now draws 5 of 12 against 4 capitals (was 2) and kills 7 of 17 destroyers (was 2).
  - Cannon ships mostly do better: 1 destroyer vs 4 frigates 3–9 (was 1–11); 13 frigates vs 1 capital 2–10 (was 5–7).
  - One went the other way: 13 frigates vs 3 destroyers is 12–0 for the frigates (was 8–4). The destroyers kill about as many frigates as before (7 of 13, was 8) but now lose all three.
  - The fighter > lancer > frigate > fighter triangle holds at 12–0 each way.

**Henry's notes:**
- "The new pixel editor looks good to me. There's even extra room in the bottom right that we can use in the future if we need it."

---

**Next:** power, once round 4's verdicts are in. It's the first new category: reactors, and how a gun's real cost becomes space and shape (parking lot: fitting and power).

---

## Parking lot (considered, not built)

- **Beam lasers**: a held beam that carves a groove. Mostly for the cool factor.
- **Fitting and power**: reactors power the guns near them, so a gun's real cost is space and shape. This keeps "pixels are money" honest without price tags. Henry expects it to stop cannons going on tiny hulls (2.3). The first new part category after round 4.
- **Small guns chip heavy armor**: flak doing a tiny bit of damage to plate. Needs an HP system.
- **Shields and counter-shields.**
- **Sub-orders**: Hunt → attack runs, orbit or kite. Also a hunting group size (Toy 1's packs of 3, generalized). In a real game, maybe a button per ship type to switch its AI live.
- **Reactionless drive** (campaign): a special engine that works buried inside armor, as an exception to the exposed-engine rule.
- **Hangars and launching**: a big ship carrying your fighter design, or a piece of it with its own core and engines detaching on command.
- **Torpedoes or bombs** for small ships to threaten plate, if lances aren't enough.
- **Per-part prices or multiple materials**: only if weight and fitting can't carry the costs.
- **Lancers vs big ships, balance**: 40 lancers kill a 620 px capital in 4 s. Good as a counter for now; revisit once fights are bigger (bigger cores, half-pixel death, or cores that resist small guns were the options).
- **Resource networks, split pools, rock claiming by network reach**: the Toy 2 match direction, waiting on ship design.
