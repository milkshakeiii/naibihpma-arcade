# Chuteworks: design draft

*Working title. A chill two-board factory puzzle game in this repo's DOS/VGA style ([`../../AGENTS.md`](../../AGENTS.md)). Draft 1, 2026-11-07.*

---

## 0. Pitch

Two 8×8 boards sit side by side under a desk lamp. On the **left** you lay belts and machines to make things. Press Run and the little factory hums and fills a rack of glass **chutes** between the boards. Then the chutes open one at a time and drop each item onto the **right** board, where it slides across the floor and you steer it into **bins**. Each level is one small idea. Speed never matters, nothing times out, and a failed run costs one keypress.

The idea the whole game teaches: **order is something you manufacture.** The left board decides *what* each item is and *in what order* it reaches the floor. The right board decides *where* it goes. Most of the best puzzles come from moving work from one board to the other.

**How I read "arcade-puzzle":** as arcade *feel* (short levels, instant runs, tactile feedback, no menus) wrapped around deterministic, hand-designed puzzles, the way ChuChu Rocket!'s puzzle mode plays. Real-time pressure (Pipe Mania style) works against the "satisfying puzzles, not numerical exercises" goal (see §5.3). It's the first open question in §8.

---

## 1. Research: what makes a puzzle satisfying

| Principle | Source | Rule adopted here |
| :-- | :-- | :-- |
| **Eureka moments are the atoms of puzzles.** A puzzle is anything that hides one; eureka (sudden understanding) is a different pleasure from fiero (beating something hard). | Elyot Grant, *30 Puzzle Design Lessons* (GDC 2021); diplograph notes | Every level states its **point** in one sentence and hides exactly one eureka. Difficulty is not the goal. |
| **The catch.** The obvious approach must visibly fail. The solver needs a reason to doubt their first idea (Grant's "aporia is a promise of surprise"). | Mark Brown, GMTK *What Makes a Good Puzzle?*; Grant | Every seed in §3 lists its catch: what the natural first attempt does wrong. |
| **Make the easiest version that still shows the idea.** Remove solution steps, trim the possibility space, keep it bite-sized. | Patrick Traynor, *System-Centric Puzzle Design in Patrick's Parabox* (GDC 2024) | ≤ 8 items per level, exact trays, and whichever board isn't the point comes pre-built. |
| **Find an interaction, then force it.** Play with the system until something interesting happens, then reverse-engineer a level that requires it. If the point is hard to state, the puzzle is unfocused. | Corey Martin (Bonfire Peaks); Traynor ("enumerate mechanic pairs", "things almost happen") | Levels come from the mechanic-pair matrix (§4). A level whose point I can't say in one sentence gets cut. |
| **Puzzles communicate truths about the system.** A good system gives back things you didn't put into it. | Jonathan Blow & Marc ten Bosch, *Designing to Reveal the Nature of the Universe* (IndieCade 2011); Grant ("interesting truths are the root of surprise") | Each world is built around 2–4 **truths** (§3), and levels exist to reveal them. |
| **Trust and transparency.** Hidden rules, hidden state and unreadable boards make a puzzle feel like an IQ test. | Grant | All state is on screen: switch levers, zipper "next" lamps, chute contents, tower layers, which chute releases next. |
| **Make experimenting cheap.** | Grant; Schell #2 ("easy to get started") | Run, stop, undo, step and scrub are each one key. Re-delivering from the same chute contents is instant. |
| **Sense of progress, sense of solvability.** | Jesse Schell, *The Art of Game Design*, principles #3–#4 | The Make→Deliver split is a checkpoint: you can check the chutes before anything reaches the floor. Bin lamps light as bins fill. Exact trays signal that a solution exists with these parts. |
| **Salience should match pertinence.** Be careful with red herrings: things that stand out should matter, and things that matter should stand out. | Grant | No decoy pieces on the main path. Every tray piece gets used. |
| **The key move should be provably unique.** | Grant | A solver (§7) checks each level: the key move is unique, even if the routine parts have many variations. |
| **Parallelism, pyramids, hints, give the answer.** | Schell #6–#9 | 3–4 levels open at once, a capstone per world, tiered hints that end in showing the solution. |
| **Insight vs search.** Difficulty can come from finding the right problem space (insight) or from the size of the search (brute force). Search-tree size tracks felt difficulty. | Scott Young on problem spaces; Nathan Sturtevant (via *The Saturday Paper*); Grant ("understand the state space") | The "numerical exercise" is search-and-bookkeeping difficulty. §2.7 lists three guarantees that keep difficulty on the insight side. |
| **Depth from a small ruleset.** One measure of a good system is how many puzzles you can make with it. | Traynor; Grant ("seek depth in systems") | Pieces work on both boards (§2.6), and one rule, "everything acts on the top layer", covers W4. Few rules, many interactions. |
| **Open-ended alternatives.** An expressive toolset and an open problem lead players to solutions the designers never imagined. | Zachtronics (Zach Barth; *Open-Ended Puzzle Design at Zachtronics*, GDC 2019) | Kept out of the main path (it's engineering, not eureka) but offered as optional **Commissions** with spare parts and solution histograms. |

**Touchstones:** ChuChu Rocket! puzzle mode (place a few arrows, then watch), Turing Tumble (a flip-flop as a visible bit), Manufactoria (conveyor sorting), Opus Magnum and SpaceChem (build-then-run, histograms), Cosmic Express (topology on a small grid), Mini Metro (calm pacing).

The design also follows threads from your earlier designs: set up a machine and watch it run, a parts budget (the tray), and a solver in the loop.

---

## 2. Core rules

### 2.1 Screen (400×300, 20 px tiles)

```
x:  0   16              176     224             384  400
    +-----------------------------------------------+
y 0 |  [plate: hall · level name, shown at start]   |
    |                                               |
 40 |   +-------------+  ====   +-------------+     |
    |   |  LEFT 8x8   |  ====   |  RIGHT 8x8  |     |   chutes are glass tubes across
    |   |  make       |  ====   |  deliver    |     |   the 48 px gutter, one per hatch row
199 |   +-------------+         +-------------+     |
    |                                               |
220 |   [tray shelf: pieces + count pips]   (lamp)  |
    +-----------------------------------------------+
```

- Both boards are 160×160. The gutter holds up to three chutes. Each chute is a horizontal tube at its row's height and holds 5 items, drawn as 8×8 miniatures.
- The rest of the 400×300 frame is the room: a dark factory wall with a window above, and the tray as a shelf below. There's no HUD text while you play. **F1** shows a key card.

### 2.2 Items

- **Blanks** are grey. **Paint** adds primaries: red, yellow, blue. Two different primaries mix: orange, green, purple. Painting a color the item already has changes nothing. A third primary makes **mud**, and mud is ruined.
- Each primary also carries a 2×2 symbol (stripe / dot / cross) that adds up the same way colors do, so the game reads without color vision.
- W4 adds **towers** (up to 3 items stacked) and W5 adds **tokens** (items stamped into tools). A single item is a tower of height 1.

### 2.3 Make (left board)

- **Crates** are fixed. Each holds a short, visible list of items and emits them in order, one per beat, from its front face.
- **Belts** are free and unlimited. You draw them by dragging, or by holding Space and moving the cursor. A belt is a line. **Lines may not cross or share tiles**, except inside a machine that allows crossing (the booth).
- Items move one tile per beat and queue behind each other.
- **Machines** sit on lines: booths, switches, zippers, sorters, stackers, pegs and dies (introduced world by world).
- **Hatches** are fixed on the right edge. A line that ends at a hatch fills that hatch's chute.
- A **jam** is a line that dead-ends, a chute that overflows, or mud. The run pauses on the culprit with a small marker.

### 2.4 Chutes

- A chute is first-in, first-out: the first item made is the first released.
- During Deliver, chutes **take turns from the top**: chute 1, then 2, then 3, then 1 again, skipping empty ones. A small lamp steps across the chute gates so you can always see which one fires next.
- Only one item or tower is on the floor at a time. The next release waits until everything on the floor has come to rest.

### 2.5 Deliver (right board)

- Items enter at their chute's row, moving east, one tile per beat. They slide straight until a piece turns them.
- **Arrows** turn everything. A **bin** accepts the items shown on its face (e.g. `2 × orange`).
- A run fails if an item enters a wrong or full bin, or hits a wall or the board edge. The item thunks and puffs; there are no buzzers and no lives.
- Paths on the floor **may cross and merge freely**, because only one item moves at a time. On belts they may not. This asymmetry drives many puzzles.
- **Win:** every item lands, every bin is full.

### 2.6 One tray, two boards

There's one shelf of pieces, and **most pieces work on either board**:

| Piece | On a belt (left) | On the floor (right) |
| :-- | :-- | :-- |
| Booth | paints the line; two lines may cross inside it | paints items sliding through |
| Switch | deals items to two lines in turn | flip-flop: turns items to its current side, then flips |
| Sorter | items containing its color go to the side line | colored arrow: turns only items containing its color |
| Peg | lifts a tower's top item onto a side line | throws the top item its way; the rest slides on |
| Zipper, Stacker, Die | left only (they need lines) | — |
| Arrow | — (belts already turn) | right only |

So in every world the player keeps facing the same useful question: **which board should do this job?**

### 2.7 Three guarantees against "numerical exercise"

1. **Speed never matters.** The order of items in a chute is a pure function of the layout: crates, lines, switches, zippers and stackers. Collisions, arrival races and beat parity never decide anything. Zippers wait for their turn and stackers count, so you can never be "off by one beat".
2. **Counts stay small and uniform.** ≤ 8 items per level, ≤ 5 per chute, bins of 1–3. There are no ratio quotas like "5 red and 3 blue from 8 blanks".
3. **Goals are about identity and place, not amounts.** A bin asks for *what*. Getting there is about *which* line, *which* board and *which* order, and each of those is a structural choice you can see.

### 2.8 Feel: responsive, immersive, minimal

- **Input:** keyboard first: arrows/WASD move, Space places or draws, R rotates, X removes, Tab switches board, Enter runs or stops, Backspace undoes, S steps one beat, hold `[` `]` to scrub time, F speeds up, F2 toggles scanlines. Mouse also works (drag belts, right-click rotates). Keys are polled at 250 Hz, and **sound plays on the key edge**, so a 15 FPS frame still feels instant.
- **Run cadence:** belts move 5 px per tick (one tile every 4 ticks). The floor moves 10 px per tick, so it feels like marbles. A typical run lasts about 15 s at 1× and about 5 s fast.
- **Make → Deliver:** chutes fill with glassy clinks, the gates lock with a clunk, a short pause, then the lamp starts stepping. Editing only the right board keeps the chute contents, so you can re-deliver instantly.
- **Chalk trails:** after a run, each item's path stays on the floor as a faint dithered line until you edit. This makes debugging part of the scene.
- **Audio:** each bin is tuned to a pentatonic note, so a correct run plays a small, never-dissonant phrase. Each hall has its own hum. Failure is a soft thunk.
- **Calm UI:** nothing flashes (per `AGENTS.md`). The plate wipes in and out with a dither wipe. The level select is a cutaway of the building: five floors (halls), with rooms that light up when solved.

---

## 3. The five worlds

The arc follows the right board's question: **what → when → where-by-what → in layers → by itself.** Each world adds one left piece, one right behavior, and 2–4 truths. Each world has 12 levels. Shown here: 7–8 seeds per world (point, catch → eureka) and one fully worked level.

Legend for maps: `.` floor, `#` wall, `@` crate (contents listed below the map), lowercase `r y b` booths, uppercase `R O B G Y` bins, `^ v < >` arrows, `==1==>` hatch/chute 1.

---

### W1 · The Paint Shop: *what things are*

**New pieces:** belts, booths (left and floor), arrows, bins. Chutes exist but only one or two are used, and order doesn't matter yet.

**Truths**
- A line paints everything that rides it, and paint adds up.
- Lines can cross only inside a booth, and crossing there costs paint.
- The floor is a specification: read the bins first and plan backwards.

| # | Seed | Point | Catch → eureka |
| :-- | :-- | :-- | :-- |
| 1 | First Coat | A line paints everything that rides it. | Show level: no catch. |
| 2 | Two Coats | Yellow then blue is green. | No green booth → pass through two. |
| 3 | Slide | On the floor things slide until something turns them. | Bin not in line with the chute → one arrow. |
| 4 | Shared Arrow | Floor paths may merge; one arrow can serve two chutes. | Two inlets, one arrow in the tray. |
| 5 | **Through the Booth** | Lines can cross only inside a booth. | Worked below. |
| 6 | Mud | A third coat ruins it, but the same coat twice is harmless. | The obvious crossing booth would mud one line → cross inside the booth whose color that line already has. |
| 7 | Floor Paint | Booths work on the floor too. | No room for the booth on the left → paint after the chute. |
| 8 | Backwards *(capstone)* | Read the bins first: the floor decides which chute must carry which color. | The bin beside chute 1 is walled off from it → chute 1 must carry the *other* color, which forces a crossing on the left. |

#### Worked level: *Through the Booth*

```
        LEFT (make)                  RIGHT (deliver)
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . . . . . .
   1  @ . . . . . . .  ==1==>   1  . . . . . . . O
   2  . . . . . . . .           2  . . . . . . . .
   3  . . . . . . . .           3  . . . . . . . .
   4  . . . r . . . .           4  . . . . . . . .
   5  . . . . . . . .           5  . . . . . . . .
   6  @ . . . . . . .  ==2==>   6  . . . . . . . R
   7  . . . . . . . .           7  . . . . . . . .

@ row 1: 2 grey     @ row 6: 2 yellow     r: red booth (fixed)
O: bin, 2 orange    R: bin, 2 red         Tray: empty
```

- **Point:** lines can cross only inside a booth, and crossing costs paint.
- **Catch:** the bins sit straight across from the chutes and the tray is empty, so the floor fixes the plan: red (grey + red) must go to chute 2 and orange (yellow + red) to chute 1. That means the top crate must reach the bottom hatch and the bottom crate the top hatch. Draw one line and it walls off the other.
- **Eureka:** the booth has a door on all four sides. Both lines need red anyway, so they cross *inside* it:

```
      0 1 2 3 4 5 6 7
   0  . . . . . . . .
   1  @ * * * . + + +  ==1==>   + line: yellow + red = orange
   2  . . . * . + . .
   3  . . . * . + . .
   4  . + + r + + . .
   5  . + . * . . . .
   6  @ + . * * * * *  ==2==>   * line: grey + red = red
   7  . . . . . . . .
```

- **Check:** the two lines' endpoints alternate around the board's edge (top-left→bottom-right, bottom-left→top-right), so they *must* cross somewhere, and the booth is the only legal crossing. The key move is unique; only the routing around it varies. It isn't a counting puzzle: the whole question is "how can two lines cross?"

---

### W2 · The Switchyard: *when things go*

**New pieces:** the **switch** (splitter on belts, flip-flop on the floor) and the **zipper** (merges two lines, alternating from a marked side; when one side runs dry the other drains). The chutes taking turns now matters.

**Truths**
- Alternation can be made and unmade: a zipper weaves, and a switch or flip-flop unweaves.
- The chutes take turns, so two chutes are a free zipper, and they always start at the top.
- You can't sort identical things: split first, then make them different.
- A switch remembers; the first item sets the phase for the rest.

| # | Seed | Point | Catch → eureka |
| :-- | :-- | :-- | :-- |
| 1 | Switch | A switch deals items to two lines in turn. | Show level. |
| 2 | Split Before You Paint | Split first, then make the halves different. | One crate of greys and two bins of different colors invite sorting on the floor, but every item on one line gets the same paint → put the switch *upstream* of the booths. |
| 3 | Flip-Flop | The same switch on the floor deals to two directions. | Show level (dual piece). |
| 4 | Turn Taking | Two chutes are a free zipper. | The fixed flip-flop needs red, blue, red, blue and the tray has no zipper → one color per chute, and floor paths merge on a shared arrow. |
| 5 | **Wrong Foot** | The chutes always start at the top; to start differently, weave it yourself. | Worked below. |
| 6 | Swap Pairs | A switch into a zipper that starts on the *other* side swaps each pair (2,1,4,3). | A one-line crate of red, blue, red, blue against a flip-flop that wants blue first. |
| 7 | Remember | One flip-flop shared by two inlets: the earlier stream leaves it pointing for the later one. | Each stream looks solvable on its own; together they're out of phase until the chute order is swapped. |
| 8 | Where's the Switch? *(capstone)* | One switch, two boards, and only one has room. | The natural spot (floor) is walled → split on the belts instead, which costs a hatch. |

#### Worked level: *Wrong Foot* (follows *Turn Taking*)

```
        LEFT (make)                  RIGHT (deliver)
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . # . R . .
   1  @ . . . . . . .  ==1==>   1  . . . # . . . .
   2  . . . . . . . .           2  . . . # . . . .
   3  . . . . . . . .           3  . . . . . F . .
   4  . . . . . . . .           4  . . . # . . . .
   5  . . . . . . . .           5  . . . # . . . .
   6  @ . . . . . . .  ==2==>   6  . . . # . . . .
   7  . . . . . . . .           7  . . . # . B . .

@ row 1: 2 blue     @ row 6: 2 red     F: fixed flip-flop, first up, then down
R: bin, 2 red       B: bin, 2 blue     #: wall with one gap at row 3
Tray: zipper x1 (left), arrow x2 (right)
```

- **Point:** the chutes always start at the top. If you need a different first item, weave the order yourself.
- **Catch:** the previous level taught "one color per chute, the chutes alternate". Do that here and blue (chute 1) fires first, so the flip-flop throws it into the red bin. Funnelling both inlets into the gap also needs three arrows, and you have two. Lines can't cross, so you can't move red to chute 1.
- **Eureka:** merge both crates with the zipper, red side first, into **one** chute:

```
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . # . R . .
   1  @ * * * * + + +  ==1==>   1  . . v # . . . .
   2  . . . . * + . .           2  . . . # . . . .
   3  . . . . Z + . .           3  . . > . . F . .
   4  . . . . * . . .           4  . . . # . . . .
   5  . . . . * . . .           5  . . . # . . . .
   6  @ * * * * . . .           6  . . . # . . . .
   7  . . . . . . . .           7  . . . # . B . .
Z: zipper, inputs north/south, output east, starts south (red)
```

  Chute 1 then holds red, blue, red, blue. The flip-flop sends them up, down, up, down. ✓
- **Check:** this level deliberately breaks the lesson of the level before (Grant: "break the rules, sometimes"). The catch is structural (which chute fires first), not a count. The two failure modes (wrong phase, one arrow short) both point the same way, so the player isn't bounced around between half-ideas.

---

### W3 · The Sorting Office: *paint is a ticket*

**New piece:** the **sorter** (on belts it sends items containing its color to a side line; on the floor it's a **colored arrow** that turns only items containing its color). From here on, booths on the floor matter.

**Truths**
- What an item is decides where it goes.
- Secondary colors obey both of their primaries' arrows. Grey obeys none.
- The colors of the arrows along a route add up to the color that route accepts.
- Paint adds up the same in any order, but a route only sees what an item has *so far*.

| # | Seed | Point | Catch → eureka |
| :-- | :-- | :-- | :-- |
| 1 | Ticket | A red arrow turns only items containing red. | Show level. |
| 2 | Contains | Orange obeys both red and yellow arrows. | The route has a red turn then a yellow turn → paint it orange. |
| 3 | The Route Spells the Color | Read the arrows to a bin: their colors add up to the bin's. | A purple bin at the end of a red-turn, blue-turn route. |
| 4 | **When It Turns Red** | A route only sees the colors an item has so far. | Worked below. |
| 5 | Sorter on the Belt | The same sorter on a belt splits a line by color. | Show level (dual piece). |
| 6 | Grey Passes | Grey ignores every colored arrow, so paint at the very end. | A colored-arrow maze that only a blank can cross. |
| 7 | Order or Color? *(capstone)* | Separate by order (switch) or by identity (sorter): the layout decides which. | The tray holds one of each; only one assignment of piece to board fits. |

#### Worked level: *When It Turns Red*

```
        LEFT (make)                  RIGHT (deliver)
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . . . . . .
   1  . . . . . . . .           1  . . . . . . . .
   2  . . . . . . . .           2  . . . . . . . .
   3  @ . . y . . . .  ==1==>   3  . . . ^ . . . O
   4  . . . . . . . .           4  . . . . . . . .
   5  . . . . . . . .           5  . . . . . . . .
   6  . . . . . . . .           6  . . . . . . . .
   7  . . . . . . . .           7  . . . . . . . .

@: 2 grey    y: yellow booth (fixed)    ^: RED arrow (turns items containing red north)
O: bin, 2 orange                        Tray: red booth x1
```

- **Point:** paint adds up the same in any order, but a route only sees the colors an item has so far.
- **Catch:** make orange on the left (yellow booth, then your red booth) and every orange item contains red, so the red arrow turns it north into the wall.
- **Eureka:** paint it **yellow on the left and red on the floor, after the arrow**: put the red booth at, say, column 5 of row 3 (`. . . ^ . r . O`). Yellow slides past the red arrow untouched, turns orange two tiles later, and lands. ✓
- **Check:** color mixing commutes and routing doesn't. That truth sits in the system itself, so the puzzle simply reveals it. It uses the dual-piece rule with no extra explanation. One piece, one placement decision, zero arithmetic.

---

### W4 · The Stack Hall: *things in layers*

**New pieces:** the **stacker** (collects up to 3 items into a tower; the first in is the bottom) and the **peg** (on the floor it throws the top item its way and the rest slides on, so a single item just turns; on a belt it lifts the top onto a side line). A tower takes **one chute slot**.

One rule covers the whole world: **everything acts on the top layer.** Booths paint only the top item, pegs take only the top, and colored arrows read only the top.

**Truths**
- Stacks reverse: the first in is the last off.
- Read a stack from the other end and it isn't reversed.
- A tower is one item to a chute.
- Whatever is underneath is hidden from booths and arrows.

| # | Seed | Point | Catch → eureka |
| :-- | :-- | :-- | :-- |
| 1 | Stack | The first item in is the bottom. | Show level. |
| 2 | One Slot | A tower is one item to a chute. | Three items, and the chute holds one → stack them. |
| 3 | Hats Off | A peg takes the top and lets the rest go by. | One throw feeds three bins in passing. |
| 4 | **Read It Backwards** | A stack reads top-first, so read it from the other end. | Worked below. |
| 5 | Mask | Booths paint only the top: hide an item underneath to keep it clean. | One line must pass a booth, but only some items should be painted. |
| 6 | The Hat Decides | A colored arrow reads only the top: the tower goes where its hat says. | Route a whole tower by painting its top item. |
| 7 | Peg on a Belt | On a belt, a peg unstacks onto a side line. | Show level (dual piece). |
| 8 | Twice Reversed *(capstone)* | Stack, peel, stack again: two reversals cancel. | No room to loop around on the floor this time → fix the order on the belts. |

#### Worked level: *Read It Backwards* (follows *One Slot* and *Hats Off*)

```
        LEFT (make)                  RIGHT (deliver)
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . . . . . .
   1  . . . . . . . .           1  . . R . Y . B .
   2  . . . . . . . .           2  . . . . . . . .
   3  @ . . . . . . .  =[1]=>   3  . . P . P . P .
   4  . . . . . . . .           4  . . . . . . . .
   5  . . . . . . . .           5  . . . . . . . .
   6  . . . . . . . .           6  . . . . . . . .
   7  . . . . . . . .           7  . . . . . . . .

@: red, yellow, blue (in that order)    chute 1 holds ONE item
P: fixed pegs, throw the top item north    R Y B: bins, 1 each
Tray: stacker(3) x1 (left), arrow x4 (right)
```

- **Point:** a stack reads top-first, so read it from the other end.
- **Catch:** the one-slot chute forces a tower: red at the bottom, blue on top. Slide it east and the first peg throws **blue** into the **red** bin. Nothing on the left can reorder three items with one stacker.
- **Eureka:** don't reorder the tower; **approach the pegs from the east:**

```
      0 1 2 3 4 5 6 7
   0  . . . . . . . .
   1  . . R . Y . B .
   2  . . . . . . . .
   3  . v P . P . P <
   4  . . . . . . . .
   5  . > . . . . . ^
```

  The tower loops down, across and back up, then slides west: blue goes to B, yellow to Y, and red (now a single item, so the peg simply turns it) to R. ✓
- **Check:** a perceptual shift that turns order reversal into direction reversal, and the system supplies it for free. A loop over the top (row 0) works too. That's fine: the key idea is unique, the routing isn't.

---

### W5 · The Toolroom: *the works builds itself*

**New pieces:** the **die** (stamps a *grey* item into a **token** carrying the die's arrow; painted items pass through untouched) and **sockets** on the floor (a token that enters an empty socket installs there as that piece and stops; everything else passes over empty sockets). Installed pieces steer everything after them, **including the next token**.

Painted tokens install as colored arrows (W3). Switch-shaped dies make flip-flops (W2). Towers of tokens can be thrown into sockets by pegs (W4).

**Truths**
- The first thing you deliver is the road for the second.
- Goods that arrive before their road get lost.
- The road you build also steers the builders that come after.

| # | Seed | Point | Catch → eureka |
| :-- | :-- | :-- | :-- |
| 1 | Die | A die turns a blank into an arrow; a socket installs it. | Show level. |
| 2 | **Lay the Road** | The first token is the road for the second. | Worked below. |
| 3 | Traffic Waits | Goods before their road get lost. | Tokens and goods in two chutes fire alternately → move the goods to a chute that fires later, or weave them. |
| 4 | Red Signal | A painted token installs as a colored arrow. | One socket must turn only the red items. |
| 5 | Switch Token | A flip-flop installed mid-run: what came before passed straight, what comes after alternates. | Two bins and four goods, but the first two must both go straight. |
| 6 | Tool Tower | Stack three tokens; the pegs throw each into its socket in one pass. | Too few chutes for three separate tokens. |
| 7 | Builders Get Steered | An installed arrow also turns the next token, so build the far piece first or route around it. | The near socket, filled first, sends the second token into a wall. |
| 8 | The Works *(finale)* | An empty floor with only sockets and bins: manufacture the whole yard, then the goods. | Combines all five worlds. |

#### Worked level: *Lay the Road*

```
        LEFT (make)                  RIGHT (deliver)
      0 1 2 3 4 5 6 7              0 1 2 3 4 5 6 7
   0  . . . . . . . .           0  . . . . . . . .
   1  . . . . . . . .           1  . . . . . . . .
   2  . . . . . . . .           2  . . . . . . . .
   3  @ . . . . . . .  ==1==>   3  . . . o . . . .
   4  . . . . . . . .           4  . . . . . . . .
   5  . . . . . . . .           5  . . . . . . . .
   6  . . . . . . . .           6  . . . o . . G .
   7  . . . . . . . .           7  . . . . . . . .

@: grey, grey, green, green    o: empty socket    G: bin, 2 green
Tray: switch x1, zipper x1, arrow die x2 (left)
```

- **Point:** the first token you deliver is the road for the second.
- **Catch:** greens slide straight over the empty sockets into the wall. Send the tokens first in the wrong order and the "right" token installs at (3,3) pointing along the row (useless), so the "down" token sails past it into the wall. Put both dies on one line and the first die stamps both greys "down".
- **Eureka (floor):** a "down" token first, so it installs at (3,3). Then a "right" token: it hits the new arrow, turns down, and installs at (3,6). Then the greens follow the road you just built.
- **Eureka (belts):** a switch and a zipper treat alternate items differently while keeping the order (a W2 lemma coming back):

```
      0 1 2 3 4 5 6 7
   0  . . . . . . . .
   1  . . * d * * . .      d: die, stamps "down"
   2  . . * . . * . .
   3  @ * S . . Z * *  ==1==>
   4  . . * . . * . .
   5  . . * e * * . .      e: die, stamps "right"
   6  . . . . . . . .
S: switch, first item north    Z: zipper, north first
```

  Item 1 (grey) goes north and becomes "down". Item 2 (grey) goes south and becomes "right". Items 3 and 4 (green) pass the dies unchanged. The zipper rebuilds the order: down, right, green, green. ✓
- **Check:** the order constraint comes from a spatial cause-and-effect you can watch (a road gets built), not from counting. Three tokens or fewer on the main path keeps the chains short.

---

## 4. How the worlds combine

Traynor's "enumerate mechanic pairs" applied to the five families. Each cell is a level seed for the later halves of worlds 2–5:

|  | **Switch / Zipper** | **Sorter** | **Stack / Peg** | **Token / Socket** |
| :-- | :-- | :-- | :-- | :-- |
| **Booth** | Split before you paint; a booth on one branch paints every other item | Paint en route changes which arrows see you | Booths paint only the hat → stack to mask | Painted tokens install as colored arrows |
| **Switch / Zipper** | | Separate by order or by color: the layout picks | A switch before a stacker deals two towers (odds and evens) | Switch + zipper stamps alternate blanks; switch tokens make flip-flops mid-run |
| **Sorter** | | | Colored arrows read the hat: the tower goes where its top says | A sorter token makes part of the yard color-selective |
| **Stack / Peg** | | | | A tower of tokens, thrown into three sockets in one pass |

Within a world, each level uses the new pieces plus **at most two** older ones, to keep the toolset readable (Grant: depth comes from simple rules, not from piling them up).

---

## 5. Stress test

### 5.1 Scorecard against the principles

| Principle | How the design meets it | Where it could still fail |
| :-- | :-- | :-- |
| One eureka per level | Every seed has a one-sentence point and a named catch. | Mid-world "combine" levels can turn into two half-eurekas. Rule: cut or split any level with two points. |
| The catch | The previous level's lesson is often the trap (*Wrong Foot*, *Read It Backwards*). | Players who skip the previous level miss the setup → cluster levels so prerequisites are obvious. |
| Easiest version | ≤ 8 items, exact trays, the non-point board pre-built. | Pre-built boards can feel like a museum → leave a small tactile job on each board. |
| Truths, not tricks | Every eureka comes from a rule that also governs other levels (crossing in booths, mixing commutes but routing doesn't, stacks reverse, roads steer builders). | The die only stamps grey items. That's a rule invented for the puzzle, so it needs its own show level. |
| Transparency | Levers, zipper lamps, chute contents, tower layers and the release lamp are all visible. | "Contains" (orange obeys red arrows) is invisible logic → the additive symbols must make it legible. |
| Cheap experiments | One-key run, undo, step and scrub; re-deliver without re-making; chalk trails. | Re-drawing long belts after a change could feel like a chore → belts re-route when you move a machine. |
| Progress and solvability | The Make→Deliver checkpoint; bin lamps; exact trays. | Exact trays can also give away too much ("I must use the zipper"). That's acceptable early; later worlds lean on *where* a piece goes, not *whether*. |
| Salience = pertinence | No decoys. | Fixed belts that aren't the point might look like clues → draw fixed belts in a duller ramp. |
| Unique key move | Solver check (§7). | Belt freedom creates many cosmetic variants. The solver must compare key placements, not whole layouts. |
| Insight over search | The three guarantees (§2.7); small boards; structural order. | W2 and W5 could turn into sequencing puzzles if levels get long → cap at one switch per path and three tokens. |
| Depth from few rules | Dual pieces; "everything acts on the top"; about 60 levels from 5 families plus their pairs. | Dual pieces risk confusion → one show level per dual piece, with the same sprite in belt and floor variants. |
| Rest, pyramid, hints | 3–4 levels open at once; a capstone per world; tiered hints. | — |

### 5.2 The numerical-exercise audit

Each place where "counting instead of thinking" could creep in, and what blocks it:

| Trap | Where it would creep in | What blocks it |
| :-- | :-- | :-- |
| Quotas and ratios | "Make 5 red and 3 blue from 8 blanks" | Bins ask for *what* and hold 1–3 of the same thing; uneven splits never appear on the main path. |
| Beat timing and parity | Two streams colliding on a beat, or items needing to arrive together | Lines can't touch; merges are pieces that wait for their turn; floor delivery is single-file. Speed is proven irrelevant. |
| Modular chains | Switch cascades giving 1:1:2 splits; counters; turntables | Main path: at most one switch per path. Cascades only in optional Commissions. |
| Long-sequence bookkeeping | "This chute must hold R B B R Y R" | ≤ 5 per chute. Goal sequences always come from one structural trick (alternate, group, reverse), never from an arbitrary list. |
| Search without an idea | Many legal placements and no reason to prefer one | Exact trays; small open floor; the solver flags levels whose solution count explodes. |
| Optimization grind | Cycle and cost histograms on every level | Histograms only in Commissions. |

### 5.3 What failed the test (rejected or benched)

- **Real-time production with live editing** (the strongest reading of "arcade"). Rejected for the main mode: timing blurs deduction, and it rewards reflexes over insight. Could return as an optional "Night Shift" mode (§8).
- **Collisions as a mechanic** (items crash when they meet on the same beat). Rejected: the puzzles became parity counting. Replaced by "lines can't touch; merges are pieces".
- **Walls turn items right** (ChuChu's rule). Rejected: a hidden auto-behavior that surprises more than it reveals. Walls stop items as a visible failure instead.
- **Ovens or delays measured in beats.** Rejected: arithmetic.
- **Return chutes and loops** (right board back to left). Rejected: counting laps, and runs that never end.
- **Mosaic floor** (items slide and rest; goals are pictures). Benched. In the mental prototype, a 2×2 checkerboard needed blue-then-red in one lane and red-then-blue in the other, and building that took two switches and two zippers of pure bookkeeping. It could come back as a bonus hall if every picture maps to one structural trick.
- **Knight springs** (items hop in chess shapes; a second way to cross lines). Benched. It fits your earlier chess-shape ideas and is spatially fun, but it mostly repeats the booth-crossing idea and landing math is fiddly without a preview. A good candidate for a bonus hall.
- **Rotating items and orientation.** Rejected: chirality on 12 px sprites isn't readable.

---

## 6. Risks and playtest plan

**Risks to watch**
1. *Attention split.* Players may treat one board as scenery. Rule: one board carries the point and the other is pre-built or a short spec. Note where testers look first.
2. *Dual pieces.* "The switch is a flip-flop on the floor" needs a show level and an identical sprite family.
3. *"Contains."* The secondary-color routing rule depends on the symbol system being legible.
4. *Deliver pacing.* Single-file delivery can drag; tune floor speed, fast-forward and scrub.
5. *Keyboard belt drawing.* It has to feel like a pen, not like menu navigation.
6. *W5 chain length.* Bootstrap levels get hard fast; keep to three tokens or fewer on the main path.

**Playtest method** (from Traynor's GDC 2024 slides): narrated screen recordings; fresh non-expert players each round in batches of 1–2; play the whole build; watch each video in full before changing anything; log time per level and *the first thing they tried* (that's the catch working, or not). He ran about 15 full-game sessions and hadn't hit diminishing returns.

---

## 7. Production notes

**Structure:** 5 halls × 12 levels = 60 main levels, plus 2 **Commissions** per hall (open-ended: spare parts, Opus Magnum-style histograms for pieces and beats). In each hall: three linear intro levels, then clusters of 3–4 open at once; the capstone unlocks at 8 of 11.

**Hints, in tiers:**
1. The level name, which is always a gentle nudge ("Wrong Foot").
2. The foreman's note, which is the level's point phrased as a riddle ("Booths have four doors.").
3. Ghost the key piece in its spot.
4. Watch the solution (Schell #9: give the answer).

**Solver** (a level-design tool and a checker):
- The floor is a deterministic simulation. Belts are vertex-disjoint paths on 8×8 with ≤ 3 lines, which backtracking handles easily at this size.
- For each level it reports: solvable; distinct **key placements** (machine, board, tile, rotation) across all solutions, which should be 1; total solution count (a cheese alarm); and the par for Commissions.
- It also checks the guarantees: no chute order depends on timing, and no item revisits a tile in the same state (endless loops).

**Tech, per [`AGENTS.md`](../../AGENTS.md):**
- 400×300 indexed framebuffer, 64-color 6-bit VGA palette, with one palette mood per hall: warm ochre paint shop, blue-teal switchyard, cream-and-green sorting office, violet stone stack hall, brass toolroom.
- 20 px tiles, 12×12 item sprites with a 1 px dark hull.
- Integer motion: 5 px per tick on belts, 10 px per tick on the floor.
- Shadows on glass tubes and the room are dithered; no alpha.
- Exact `N×` scaling, F2 scanlines, 15 FPS ticks, 250 Hz input latching.
- Levels are plain-text files like `starpond/levels/*.lvl`: one 8×8 grid per board plus the crates, chutes and tray.

---

## 8. Open questions

1. **Arcade:** is *feel* enough, or do you want real-time pressure? If the latter, the cleanest add-on is a **Night Shift** mode per hall: one fixed factory gets a stream of new right-board orders, and you make a few edits per order against a gentle clock that only affects a bonus.
2. **Input:** keyboard-first (true to DOS) or mouse-first (faster belt drawing)?
3. **Fiction and sound:** a sleepy night-shift factory is the default. Five halls could also each take a soundtrack from the s1–s5 albums.
4. **Trays:** exact everywhere on the main path (current plan), or a spare piece plus an optional "thrift" seal?
5. **First prototype:** W1 and W2 (belts, booths, chutes, switch and zipper, floor arrows and flip-flops) is the smallest slice that tests the core claim, *order is something you manufacture*.

---

## Sources

- Jesse Schell's ten puzzle principles, with a summary of Elyot Grant's lessons: <https://library-sigma-mauve.vercel.app/details/game-design/puzzles/Puzzle-Design>
- Elyot Grant, *30 Puzzle Design Lessons From the World's Greatest Puzzle Communities* (GDC 2021): <https://gdcvault.com/play/1027306/30-Puzzle-Design-Lessons-From>
- "Eureka is not fiero" (notes on Grant): <https://diplograph.net/notes/games/puzzle-design/eureka-is-not-fiero>
- Patrick Traynor, *System-Centric Puzzle Design in Patrick's Parabox* (GDC 2024 slides): <https://media.gdcvault.com/gdc2024/Slides/GDC+slide+presentations/Traynor_Patrick_SystemCentricPuzzle.pdf>
- Corey Martin on Bonfire Peaks' puzzle process: <https://gamedeveloper.com/design/bonfire-peaks-explores-complex-emotions-through-puzzle-design>
- *Open-Ended Puzzle Design at Zachtronics* (GDC 2019): <https://gdcvault.com/play/1025715/Open-Ended-Puzzle-Design-at>
- Zachtronics on expressive toolsets (Road to the IGF: Opus Magnum): <https://gamedeveloper.com/business/road-to-the-igf-zachtronics-i-opus-magnum-i->
- Jonathan Blow & Marc ten Bosch, *Designing to Reveal the Nature of the Universe*: <https://marctenbosch.com/news/?p=181>
- Mark Brown, GMTK *What Makes a Good Puzzle?*: <https://amara.org/v/C3BEg/>
- Scott Young on problem spaces and insight: <https://www.scotthyoung.com/blog/2022/03/28/human-problem-solving/>
- Nathan Sturtevant's brute-force level analysis, via *The Saturday Paper*: <https://www.gamedeveloper.com/design/the-saturday-paper---exhausting>
