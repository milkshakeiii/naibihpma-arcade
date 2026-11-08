# Mise en Space: the kitchen redesign

*Working title. Replaces the paint-and-switch halls of the Chuteworks prototype (see `../../chuteworks/docs/design.md`) after playtest 1, which found that building, watching and the goal all felt dry. The DOS look, feel and UI responsiveness carry over unchanged. Draft 1, 2026-11-08.*

> **Status after review (2026-11-08):** the literal music mapping in §1–§4 (tempo as the kitchen clock, harmony as the menu) is **dropped**. Note-reading isn't reliable enough to match the music, and the music wasn't written to admit extra chords. The music stays background only. Places and signature events may still serve as invisible inspiration for restaurants. The prototype in this folder also changed three rules while it was being built: stations pass through what they never use, counters plate only against open order tickets, and full belt loops keep turning. See the [README](../README.md).

---

## 0. The game in one paragraph

You run the automated kitchen of a small restaurant at each stop of a deep-space survey. The **left board is the kitchen**: pantry hatches pulse out ingredients, and you lay belts, grabber arms and machines to cook them into dishes, the way you would in Factorio. The **right board is the dining room**: alien guests come in, sit at tables, order, wait, eat, pay and leave. The two rooms are joined by **chutes on the same square**: a chute you build on kitchen square **E3** opens through the floor at dining-room square **E3**. Service runs live for the length of one song. You can pause and build any time, at 15 FPS with 1×/2×/4× speed. Every object, rhythm, menu and event in a restaurant is derived from its track by the rules in §2.

---

## 1. Core rules

### Kitchen (left board, 8×8)
- **Pantry hatches** (fixed) put out one ingredient every N beats.
- **Belts** carry items one tile per beat and back up when blocked.
- **Grabber arms** lift one item from the tile behind them and set it on the tile in front, like Factorio's inserters.
- **Machines** take inputs, work for a fixed number of beats, then put out a product: slicers, ovens, and more per restaurant. Each machine's cycle is shown as pips.
- **The pass** (plating station) assembles a dish when its recipe's parts are all present. Recipes show parts as icons, so "2 frostberries + 1 ice-rye" reads at a glance.
- **Chutes** send anything that enters them up through the floor at the same square.

### Dining room (right board, 8×8)
- Each level fixes the **door**, **tables** and **seats**. Some restaurants add fixed service belts, or a few placeable pieces (see §4).
- **A chute must surface on a table to serve it.** The dish appears on the table, and the guest at that table who ordered it takes it. If nobody there wants it, it sits on the table and blocks that chute, and the kitchen line behind it backs up.
- Guests walk in on the beat, take the nearest free seat, and show their order in a bubble. They wait, eat (with a small animation), pay, and leave. A guest who waits past their patience leaves hungry and pays nothing.
- **Score is stars for the night:** guests fed, and tips for speed. There's no game over; you can replay a service to improve it.

### Why the same-square chute matters
Every chute takes the same square in both rooms. Tables are where the guests are, and that's rarely where the kitchen wants its output: those squares may be walls, pantry lanes or your best machine spot. **The dining room's floor plan constrains the kitchen's layout through the floor.** That is the tension between the two boards, and it's built into the geometry.

### Ratios, made obvious
Two sources set every ratio, and both can be seen and heard:
- **Recipes** show repeated parts: two frostberry icons means twice the frostberry line.
- **Machine cycles** are note values (whole, half, quarter, eighth beats; thirds in triplet tracks). A machine with two pips takes twice as long as one with one pip, so it obviously needs two of the faster one feeding it, or half the supply.

Each machine plays its sound when it finishes a batch, so a balanced kitchen plays along with the song and a starved one drops beats. A mismatched ratio is something you hear as well as see.

---

## 2. The method: deriving a restaurant from its track

The other games in this repo took *subjects* from the albums: creatures, habitats, taverns. This one takes the **musical structure** (pulse, parts, harmony, form) and turns each into a system. **The music proposes and the designer disposes:** the track supplies more material than a restaurant can hold, and the designer cuts it to a budget by musical salience.

| Music | Becomes | What it means in play |
| :-- | :-- | :-- |
| **Pulse** (exact BPM) | The kitchen clock. Belts step and machines cycle on the beat. | Slow songs make slow, contemplative kitchens; fast ones make busy kitchens. |
| **Note values** of the rhythmic parts | Machine cycle lengths (1/2, 1, 2, 4 beats; 3 in triplet tracks) | Ratios between machines are musical (2:1, 4:1, 3:2) and audible. |
| **Signature rhythmic instruments** | The restaurant's new machine types; each sounds like its instrument | A balanced line grooves; a starved machine goes quiet. |
| **Common tones** across a section's chords | **Staple** ingredients that every dish uses | The staple line is the kitchen's backbone, like Factorio's iron plates. |
| **Each chord** of a section | A **dish** | Guests order the dish of the chord sounding as they sit, so how long each chord lasts sets the order mix. |
| **The note that moves** between chords | That dish's **own ingredient** | Dishes share lines and differ by a part or two. |
| **A doubled note** in the voicing | A **double portion** | The obvious "twice the hot sauce" ratio. |
| **Modal mixture** (major↔minor of one chord) | A **sibling dish**, one ingredient swapped | One branch off an existing line. |
| **Transposition** | The whole menu keeps its shape but **new pantry hatches open in new places** and the old ones close | Re-plumb the inputs; keep the machinery. |
| **Enharmonic pivot** (D♭ = C♯) | Same ingredient, new name | A joke that looks like a crisis. |
| **Form** (section map) | The **service script** | Intro = prep, themes = seatings, beatless passages = lulls to rebuild, climaxes = rushes, coda = last orders. Guests arrive on 4-bar phrase lines. |
| **Lead soloists** | **VIP guests** who arrive when the solo starts with an unusual order and leave when it ends | A moment to react to, not a constant. |
| **Place** (the body the track depicts) | What the ingredients physically are, who eats there, and **one physical rule** | Gives each restaurant its own material behavior. |
| **The track's signature event** | The restaurant's **twist** | The thing you remember the level by. |

**Budget per restaurant:** about 5 ingredients, 2–4 dishes, 1–2 new machines, 1 new rule and 1 twist. Cuts keep, in order: the tonic, the most repeated chords, the signature event, the first soloist.

**Players never see note names.** They see ice-rye and frostberries, and *hear* that the menu changed when the key did.

---

## 3. The route: *Stellar Cartography*, seven restaurants

**Why this album:** it's literally intergalactic, no other game in the repo uses it, and its arc is a survey voyage inward from rogue objects to Earth, which reads as a run of restaurants. Every track is in 4/4 with an exact project BPM in the notes, which suits a beat-clocked kitchen.

| # | Track (length, BPM) | Restaurant | New machine (from its parts) | Rule and twist (from place and signature event) | Guests; VIP |
| :-- | :-- | :-- | :-- | :-- | :-- |
| 1 | Star 2 Object 25 (6:57, 93.75) | **Object 25 Canteen**, a survey outpost on an ice planetoid | **Slicer**: half-beat cycle (the 16th pulse arpeggiator) | **Eclipse** (beatless 3:40–4:24): solar machines stop, a window to rebuild. **Signal Lock II** (+2) opens new hatches for the transposed menu. | Survey crews; theremin surveyor (3:05) |
| 2 | Star Ø Object 1, 'Oumuamua (5:41, 98.68) | **The Tumbler**, a diner bolted to a tumbling rogue rock | **Tumble drum**: 3-beat cycle (the eighth-note-triplet ostinato), so 3:2 against everything else | **Outbound acceleration** (2:55): the menu shifts +1 semitone for good. Every hatch changes, but the shapes stay. | Cryo-sleepers waking one by one; the reed-horn elder (1:03) |
| 3 | Star 1 Planet 2 (6:58, 75, drumless) | **The Brook Garden**, farm-to-table on a verdant super-Earth | **Brook**: fixed water channels that carry items for free (the square-wave brook arpeggio). **Reed oven**: a slow drone. | **Ripening**: living ingredients ripen as they travel and spoil if they wait too long. Parallel-minor surges (3:16, 6:16) briefly darken the menu. | Fauna-folk of the canopy; shawm dancers (5:45) |
| 4 | Star 3 Planet 2 (5:41, 75/150) | **The Lattice Bar**, a crystal-world cocktail counter | **Prism**: splits one item into two (kalimba ping-pong bells), a built-in 1:2 | **Double time** from 1:48: the kitchen clock doubles. The IDM breakdown (2:56–3:56) makes belts stutter. | Crystal beings; portamento singer (chorus) |
| 5 | Star Ø Object 2 (6:14, 83.33) | **The Periastron Refectory** in a relic on a hyperbolic flyby | **Organ oven**: huge, 8-beat cycle (the cathedral monolith). **Music box**: plates parts in a fixed order. | **Periastron** (2:57): every belt reverses. The outbound menu is redshifted −2. Symmetric kitchens win. | Pilgrims; the organist (1:52) |
| 6 | Star 4 Planet 4.5, Vesta (6:05, 83.33/166.67) | **Core Sample Mess**, an asteroid miners' canteen | **Crusher** for heavy ingredients (the iron-core riff). **Pebble sorter** (the L/C/R clicks). | **The 360° orbit**: the menu modulates through five keys and comes home. Heavy items ride belts at half speed. | Miners; solo violinist (2:18, 5:05) |
| 7 | Star 4 Planet 3, Earth (6:30, 66.96) | **Pale Blue Dot**, a seaside restaurant at home | **Rain barrel**: free water during rain (the rain and surf). **Hand-drum line** (the kalimba groove). | **The Gaia breath**: every 4 bars (14.34 s) the tide washes the dining room's lowest row. The D♭ sunbreak renames the menu (enharmonic). | Everyone comes home; **the whale**, who eats a huge order on each breath |

Difficulty rises by budget rather than by track number. Restaurant 1 teaches the core loop; restaurant 7 uses everything.

---

## 4. Restaurant 1 worked through: Object 25 Canteen

**Track:** Star 2 Object 25, 6:57 at 93.75 BPM, so one beat is 640 ms and one bar is 2.56 s. Service is one song.

### Menu, derived from Signal Lock I (0:46–1:06)
The progression is D**sus2** (D–E–A–E) → **Bm9** → **Gmaj7** → **Gm(add4)** (B♭–D–G–C).

- **D is in all four chords**, so it becomes the staple, **ice-rye**, used by every dish.
- **E is doubled** in the Dsus2 voicing, so the first dish takes two **frostberries**.
- **B** (Bm9, Gmaj7) becomes **radio-moss**, and **G** (Gmaj7, Gm) becomes **pulsar beans**.
- **G → Gm swaps B for B♭**, so a sibling dish swaps radio-moss for **shade-moss**.

| Dish | Chord | Recipe |
| :-- | :-- | :-- |
| Signal Loaf | Dsus2 | 1 ice-rye + **2** frostberries |
| Moss Bowl | Bm9 | 1 ice-rye + 1 radio-moss |
| Bean Plate | Gmaj7 | 1 ice-rye + 1 pulsar beans + 1 radio-moss |
| Shade Plate | Gm(add4) | 1 ice-rye + 1 pulsar beans + 1 shade-moss |

### Machines, derived from the instrumentation
- **Slicer** (½-beat cycle): the crystalline 16th/32nd pulse arpeggiator, so it ticks on the arp.
- **Steam oven** (2-beat cycle): the muffled 16th sub-bass of the survey section.
- **The pass**: pings like the sonar blips when it plates.
- **Grabber arms**: click like the L/C/R telemetry clicks.
- **Pantry hatches**: pulse with the pulsar chirps.

### Service script, from the section map

| Time | Music | Service |
| :-- | :-- | :-- |
| before 0:00 | — | **Prep**, paused: build freely. |
| 0:00–0:46 | Pulsar sweep, riser, 4 sonar pings | The pantry starts pulsing. Doors open on the 4th ping. |
| 0:46–1:27 | Signal Lock I | The first guests. Only Signal Loaf and Moss Bowl are ordered (budget). |
| 1:27–3:40 | Orbital survey (beat enters, chords drift) | Steady seatings; Bean Plate and Shade Plate join the menu. The **theremin surveyor** VIP arrives at 3:05 and wants a double Shade Plate. |
| 3:40–4:24 | **Eclipse** (beatless) | Lights out: the steam ovens (solar) stop. Guests wait patiently. **A window to rebuild.** |
| 4:24–4:58 | Signal Lock II, +2 | New hatches open (the transposed pantry) and the old ones close. Same recipe shapes, re-plumbed inputs. |
| 4:58–6:28 | Climax: overdriven bass, starlight arps | The **rush**: the most guests of the night. |
| 6:28–6:57 | Deceleration, pulsar static | Last orders. Guests still seated finish and pay. |

### Floor plans (sketch)

```
     KITCHEN (left)                 DINING ROOM (right)
     A B C D E F G H                A B C D E F G H
   1 R . . # . . . .              1 . . . . . s T s
   2 . . . # . . . .              2 . s . . . . s .
   3 F . . . . . . .              3 s T s . . . . .
   4 . . . . # # . .              4 . s . . s . . .
   5 M . . . . . . .              5 . . . s T s . .
   6 . . . . . . . .              6 . . . . s . . .
   7 B . # . . . . .              7 . . . . . . . .
   8 . . . . . . . .              8 D . . . . . . .

R ice-rye  F frostberry  M radio-moss  B pulsar beans (pantry hatches)
#  coolant pipes (walls)   T table   s seat   D door
```

Three tables, at **G1, B3 and E5**, so the kitchen must put chutes at G1, B3 and E5. **B3 sits right next to the frostberry hatch at A3**, which crowds the Signal Loaf line. **E5's kitchen square is boxed in by the pipes at E4/F4.** The far table at **G1** is the long run. A kitchen that serves all three tables with the right dishes, in a compact 8×8, is the puzzle; how you route to each is yours.

---

## 5. Stress test

### Against the playtest's diagnosis

| Was dry | Now |
| :-- | :-- |
| **Nothing wanted anything** | Guests with orders, patience and reactions. Species quirks and VIPs per restaurant. |
| **Watching only confirmed** | Everything runs live with the song: belts, swinging arms, machines sounding on the beat, guests walking and eating. Backups and blocked tables happen naturally rather than as scripted failures. |
| **No authorship** | Open kitchens with many solutions, judged by stars, plus optional footprint and part counts. |
| **The verbs were plumbing** | Still placement, but now in a Factorio register (lines, arms, machines, throughput) with materials that behave: ripening, heavy items, splitting, reversing belts. |
| **The content was symbolic** | Concrete foods, recipes shown as icons, and ratios read as pips and heard as rhythm. The note names stay behind the curtain. |

### Against the earlier principles
- **Each restaurant teaches one truth through its twist.**
  - Restaurant 1: rebuild in the lull.
  - Restaurant 2: shapes survive transposition.
  - Restaurant 5: symmetric kitchens survive reversal.
  - Restaurant 7: the tide rewards high tables.
- **Room text teaches rules only**, never the trick. Playtest 1 showed that stating the trick kills it.
- **Ratios stay legible:** small whole numbers, always shown, always audible. No hidden arithmetic.

### Risks
1. **Real-time stress.** Pause-anytime and no game over should keep it calm; this needs checking in play.
2. **8×8 may be too small for Factorio-style building.** Two 160-pixel boards fill the 400-pixel screen at 20-pixel tiles. If the kitchen feels cramped, the dining room can shrink to 6 columns and give the kitchen 10.
3. **Over-derivation.** If a music-derived rule makes a level worse, the budget rule says cut it. The method should serve the game, not the reverse.
4. **Song-length services** (5–7 minutes) may feel long for a first restaurant. A practice mode could loop just the first section.

---

## 6. Smallest slice to prototype

**Restaurant 1 up to the eclipse (0:00–3:40):**
- **Kitchen:** pantry hatches, belts, grabber arms, slicer, steam oven, the pass, chutes.
- **Dining room:** door, tables, seats, and guests who walk, order, eat and leave.
- **Live service:** synced to the song, with pause/build and speeds. Machine sounds land on the beat.

That's enough to answer the three questions the paint-shop prototype couldn't:
- Is building satisfying?
- Is watching alive?
- Do the guests make the goal matter?

The renderer, palette, input, hardware cursor and UI patterns carry over from the current prototype.

## 7. Open questions
1. **Album:** start with *Stellar Cartography*, as proposed, or another?
2. **Route order:** album order (inward to Earth) or difficulty order?
3. **How literal is the harmony mapping?** Players never see note names. Is "the menu changes when the key does" the right depth, or should menus change less often?
4. **Guests leaving hungry:** a gentle loss, or purely fewer stars?
5. **Dining-room spice:** which placeable pieces are first on the right board? Lazy Susans that rotate one dish around a table's four seats, short service belts, plate racks.
