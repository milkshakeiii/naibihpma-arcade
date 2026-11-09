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

**Queued for round 3** (agreed, built after the round 2 verdicts):
- **Split-brain ships** (2.1 rework). When a ship is cut apart, every piece that still holds a core flies on as its own ship. Pieces without a core break away as now.
- **Extra cores at a steeply rising cost.** For example, the second core costs +8, the third +16, the fourth +32. On a fighter, a second core costs more than the whole ship. On a capital, two or three are cheap insurance. This needs a cost separate from pixel count, shown in the yard and used by the equal-cost sims.
- **Engines must be exposed.** Henry's rule: an engine needs a 3-wide path out of the ship's square. It started as "a 3-wide path to the back of the sprite"; Henry then opened it to any 3-wide path, so the rule isn't too restrictive. Read as: from one side of the engine, a 3-wide channel of empty space leads out of the ship's bounding box. The channel can point any way and can bend. Technically, a 3x3 empty window sitting against one face of the engine can slide cell by cell through empty space until it's outside the ship. Otherwise the engine shows red in the yard and the ship can't fly, like a loose pixel. Engines packed inside plate or stacked tightly are buried; for example, the default destroyer's middle engines would be. Plate can't hug an exhaust, so a fast ship still has soft spots.
- **End stalemates early** (decided above, arena only: 20 s with nothing shot off ends the fight as a draw, and the result names any disarmed side). Split-brain makes the question more pressing, because a lone core pixel shot loose would become a "ship" that keeps the fight alive.

**To talk through before building** (after round 3):
- **How to design big ships.** Henry wants even bigger ships, and clicking pixels on a 41 grid doesn't scale. Big ships probably need their own way of being designed, and the yard has to stay tidy as features pile up. One direction: design small modules (a turret pod, an engine block, an armor panel) pixel by pixel, then lay a big ship out from your own modules, so the big-ship editor only places them. Fitting and power could live there too, as a reactor module powering the modules beside it. That would also make a cannon on a tiny hull expensive.

---

## Parking lot (considered, not built)

- **Beam lasers**: a held beam that carves a groove. Mostly for the cool factor.
- **Fitting and power**: reactors power the guns near them, so a gun's real cost is space and shape. This keeps "pixels are money" honest without price tags. Henry expects it to stop cannons going on tiny hulls (2.3); see the big-ship talk above.
- **Small guns chip heavy armor**: flak doing a tiny bit of damage to plate. Needs an HP system.
- **Shields and counter-shields.**
- **Sub-orders**: Hunt → attack runs, orbit or kite. Also a hunting group size (Toy 1's packs of 3, generalized). In a real game, maybe a button per ship type to switch its AI live.
- **Reactionless drive** (campaign): a special engine that works buried inside armor, as an exception to the exposed-engine rule.
- **100 px capitals**: these need ships built from your own smaller designs (stamped turret modules, hangars holding your fighter design).
- **Torpedoes or bombs** for small ships to threaten plate, if lances aren't enough.
- **Per-part prices or multiple materials**: only if weight and fitting can't carry the costs.
- **Lancers vs big ships, balance**: 40 lancers kill a 620 px capital in 4 s. Good as a counter for now; revisit once fights are bigger (bigger cores, half-pixel death, or cores that resist small guns were the options).
- **Resource networks, split pools, rock claiming by network reach**: the Toy 2 match direction, waiting on ship design.
