# Many Worlds Line (working title)

A design for one game that plays all 43 tracks of *Many Worlds*. Status: phase 0 is done (the rules run as a simulation, four bots have played them, and a second round changed the rules to add strategy; see "Phase 0 results"). The phase 1 gray-box of leg 1 is built and runs in the browser; nobody has playtested it yet. Nothing here is proven fun until a person has played it.

## The game in one paragraph

You run the train from the *Many Worlds* cover across seven worlds. **The train never stops, and you draw the rail in front of it.** Each track is one leg of the journey and lasts exactly as long as the song. Passengers wait at stations and want other stations; you carry as many as you can before the song fades out and the last stop opens. Rail is scarce, the train cannot reverse, and only the rear car can get off: whoever boarded last leaves first, so where you stop decides who you can deliver next.

## What the music allows

I measured all 43 remastered tracks (3 h 35 min in total). Three facts drive the design:

| Finding | Consequence |
|---|---|
| Tracks run 3:08 to 7:17, median about 4:50 | One track is a natural level length. Seven albums are seven sittings of 24 to 44 minutes. |
| Loudness is steady: the body of a typical track varies by about 4 dB. The intro is about 2 dB quieter and the last eighth about 8 dB quieter. | Loudness cannot drive difficulty. Only the intro and the fade-out are reliable landmarks, so the fade-out becomes the "last stop" in every leg. |
| Only about 6 tracks (1, 4, 13, 29, 37, 43) have a pulse strong enough to detect automatically, by a rough measure | No beat-matching mechanic. The game is in flowing time, not on a beat grid. |

So the music supplies **structure** (length, opening, ending, and section changes marked by hand), and the player supplies the moment-to-moment action. The song is never interrupted: no failure restarts it, and you always hear it to the end.

Each track gets a small cue sheet that Henry fills in, since the composer knows this better than any analysis: optional BPM, key, and 3 to 6 section timestamps. Events in a leg (a new station, a new hazard) fire on those timestamps. Without a cue sheet a leg falls back to evenly spaced events.

## Rules

The playfield is a single screen of 25 × 17 tiles (16 px tiles on a 400 × 300 framebuffer, with a 28 px control strip).

1. **Draw.** Drag across tiles to lay rail. Each tile of rail costs one piece from a fixed stock for the leg (40 on world 1 maps). Plain rail the train has passed is picked up behind it and its piece comes back, unless the train's route comes over it again; junctions stay. Erasing a tile by hand also returns its piece.
2. **The train follows the rail and never stops by choice.** It cannot reverse. If it reaches the end of the rail it waits there, which is a *stall*.
3. **Stations.** A station is a tile you draw through. Driving through it boards waiting passengers, first come first served, up to the train's capacity of six. Each passenger is a car showing the shape they want, added at the back of the train.
4. **Last on, first off.** Driving through a station whose shape matches the *rear* car uncouples it and scores it, then the next rear car if that matches too. A passenger further forward waits until everyone behind them is off. The rear car carries a red lamp.
5. **Tail.** If the engine runs into its own cars, that is a *crash*: all cars are lost and the train stands still for two seconds. This happens on a crossing, or where you draw rail into one of your own cars (the way out if you ever box yourself in).
6. **Switches.** A tile with three rails is a switch. Two of its arms are lit, and the lit pair is the route; the third arm is closed, and a train arriving on it waits. A branch drawn off the line the train is following leaves the train's way lit. Dragging through a switch lights the way you dragged, and clicking it cycles the lit pair. Points are locked while the train is on them. A four-way tile is a crossing and only goes straight.
7. **Express.** Holding express roughly doubles the train's speed, and an express train runs through stations without stopping: no one boards or gets off. That makes express the way to pass a station whose passengers would bury your rear car.
8. **Last stop.** When the fade-out begins, the terminus opens. Driving into it ends the leg and scores 10 × your multiplier; the song plays on to its end.

**Score.** Each delivery scores your multiplier. Consecutive deliveries build a streak multiplier up to ×4 (a step every four deliveries), shown as lanterns in the control strip. A stall or a crash ends the streak. Deliveries fill a lamp-post meter with three lamps. Lamps are the only currency: a world opens when you hold about 60% of the lamps before it, so no single leg can block you.

**No failure state inside a leg.** Stations never overflow and there is no game over. The pressure is the song running out.

### Why this is not trivial

The first version of these rules let any passenger whose shape matched get off. The bots showed that was shallow: heading for the nearest useful station scored 80% of careful play ("Phase 0 results", round 1). Last on, first off is what adds the decisions:

| Source of difficulty | The decision it forces |
|---|---|
| Last on, first off | Which stations to stop at, and which to pass on express, so the rear car is always someone you can drop off soon |
| First come, first served boarding | Reading a station's queue before stopping: its passengers board in that order, and the last one becomes your rear car |
| Rail stock is too small to connect everything | Drawing just ahead of the train instead of building a network |
| No reversing | Turning round costs a U-shaped detour |
| The train is moving while you draw | Plan a few tiles ahead or stall and lose the streak |

What the bots found about the ways the game could collapse:

- **One big loop** through every station. With 40 pieces a loop can't cover a world 1 map, and under last on, first off a loop delivers poorly anyway: the big-loop bot scores 21% of the planner at 40 pieces and only 48% even with 120. Keep stock at about 40 on maps this size.
- **Lay ahead, erase behind.** Not a collapse: it is how the game is played. Every strong bot draws a short line just ahead of the train. Picking up rail behind it was busywork that stalled the bots until they learned to batch it, so the rules now do it automatically; the bots score the same either way. Switches and loops rarely pay off for them, so the switch lesson in leg 4 still lacks a strategic purpose.
- **Shuttling between two stations.** Not tested. First come, first served boarding plus last on, first off should bury it under passengers for the third shape.

## Controls

| Action | Mouse | Key | On-screen |
|---|---|---|---|
| Lay rail | Left-drag | | |
| Erase rail | Right-drag, or left-drag in erase mode | `E` toggles erase mode | Eraser button |
| Flip switch | Click it | | |
| Express (faster while held; runs through stations) | | `Space` | Lever |
| Pause / restart leg | | `P` / `R` | Buttons |
| Scanlines | | `F2` | Button |
| Sound effects on/off | | `M` | Button |
| Pick up rail behind the train on/off (playtest only) | | `L` | Button |

Express is the one way to gain time. It leaves less time to draw, and you must let go before each station you want to stop at.

## Responsiveness

- The pointer is a hardware cursor and tracks the hand at full rate. Every input event is latched as it arrives (in the browser, including coalesced pointer samples), so rail appears on the very next 15 FPS frame, with every tile under a fast stroke filled.
- Every action has one immediate visible result: rail drops in, a switch arm lights, a passenger hops aboard, a lamp lights.
- Everything that can hurt you is visible before it happens. The next tile the engine will enter is always highlighted, the engine's tile is outlined in orange when the line ahead has run out, and a tail collision three tiles ahead tints that tile.
- Passengers who have just boarded wait behind the train in boarding order until it pulls them onto the line, so you always see who is at the rear.
- Feedback stays calm: steady highlights, no flashing text or borders.

## Teaching the rules

No rules screen. Leg 1 ("The Corner of Maple St. and the Future") is the tutorial and cannot be lost. Each step shows a ghost hand sliding along a dithered ghost path on the real board, outlines the station to aim for, then waits for the player to do it. The train is held at a signal until rail reaches the first station, and passengers are scripted until step 3 is done.

| Step | Caption (the only text) | Player does |
|---|---|---|
| 1 | DRAW RAIL | Drags from the engine to the ● station; the signal clears and the train boards one passenger for ◆ |
| 2 | MATCH SHAPES | Draws to the ◆ station; the passenger gets off; two board there, ■ then ● |
| 3 | REAR CAR FIRST | The ● car is at the rear, so draws to ● first, then ■. Passengers start arriving everywhere |
| 4 | HOLD SPACE | Shown once the line ahead is long and clear of stations, with the lever button outlined; holds express for four tiles |
| 5 | LAST STOP | Fade-out begins; draws to the terminus |

Two hints can appear at any time: **CLICK THE SWITCH** (with the switch outlined) when a closed switch arm is what stops the train, and **RELEASE AT STATIONS** the first time express runs through a station.

The rest of leg 1 is free play on the same map. Because the tutorial uses part of the song, lamp thresholds on a first play shrink to the share of the leg left when free play begins. The remaining tools arrive one per leg in world 1, each with one caption and one ghost demonstration. Every later world teaches its single new rule the same way in its first leg.

**Text budget.** Captions of at most three words, plus the track title and album shown for a few seconds at the start of each leg. Shapes, lamps and icons carry everything else.

## The seven worlds

One new rule per world. Each is taught in that world's first leg.

| # | World | New rule | What it changes |
|---|---|---|---|
| 1 | The Marvelous Electrical World of Tomorrow | Base rules, then one tool per leg | Learning the railway |
| 2 | Abstract Objects | **Prisms**: driving through one changes a car's shape to the next in a fixed cycle | You can manufacture the cargo a station wants, at the cost of a detour |
| 3 | Stellar Cartography | **Fog**: tiles are dark until the engine's lamp passes within three tiles; rail can only go on charted tiles | Exploring competes with delivering |
| 4 | Virtual History | **Ruin**: a rail tile crumbles after the train has crossed it three times | Standing loops wear out; routes must keep moving |
| 5 | The Enchanted Forest of Naibihpma | **The rider**: Gorf travels your rails toward you and steals a car if he reaches the train; switches steer him too | Your own network can be used against you, and you can trap him with it |
| 6 | Background Music Radiation | **Half-life**: each car has four pips that drain while aboard, faster inside drifting glowing zones; it scores its remaining pips | Short routes and prepared rail beat large loads |
| 7 | Lost Wing | **Wraparound**: rail leaving one screen edge continues from the opposite edge, combined with earlier rules | The finale remixes everything |

## The 43 legs

| # | Track | Length | Leg |
|---|---|---|---|
| 1 | The Corner of Maple St. and the Future | 4:49 | Tutorial: draw, match, rear car first, express, last stop |
| 2 | The Late Shuttle Home | 5:30 | Tight stock; erasing for refunds |
| 3 | The Lights Above Inventor Museum | 4:48 | Express past stations whose passengers would bury your rear car |
| 4 | Transformer Junction No. 9 | 3:11 | Switches |
| 5 | Starbridge | 3:55 | Bridges: rail may cross water and pass over other rail |
| 6 | The Great Transoceanic Railway | 6:38 | World 1 exam, the largest map |
| 7–12 | Alpha, Beta, Gamma, Delta, Epsilon, Zeta | 5:50, 5:33, 7:09, 4:18, 5:01, 4:40 | Prisms: one, then chains, then prisms behind switches |
| 13–19 | Star 2 Object 25; 'Oumuamua; Star 1 Planet 2; Star 3 Planet 2; Star Ø Object 2; Vesta; Earth | 6:57, 5:41, 6:58, 5:41, 6:14, 6:05, 6:30 | Fog: one star system per leg, ending in the fully charted homecoming to Earth |
| 20–25 | Prehistory, Stone Age, Golden Age, Space Age, A Myth, Posthistory | 5:03, 5:20, 4:33, 6:27, 3:08, 7:17 | Ruin: the tileset advances one era per leg |
| 26–31 | The Underbrush; Daot the Enchanter; Twen, Dew Collector; Star Pond; Gorf, Dark Sorcerer; Ancient Magic | 5:52, 4:30, 4:48, 4:10, 4:51, 4:25 | The rider: slow at first, two riders by the end |
| 32–37 | Ultraviolet, Gravitational, Antimatter, Solar, Nebular, Microwave | 4:01, 3:44, 4:20, 4:22, 4:23, 3:36 | Half-life: zones grow and drift faster |
| 38–43 | Level 1; Level 2; Becoming Lost; The Hours; The Hours; Next Game, Until | 3:59, 3:42, 5:12, 4:00, 4:16, 3:37 | Wraparound alone, then with prisms, fog, ruin, half-life, and finally everything |

Between legs, a route map shows the line with seven stops, a dot per leg and its lamps. Progress saves after every leg.

## Look and sound

- **Art.** Top-down pixel art under the rules in `AGENTS.md`: indexed palette, integer positions, no alpha (fog and zones are checkerboard dither or palette remaps), 15 FPS ticks, scanline toggle. Each world takes a 16-colour ramp from its album cover. The three-eyed cartographer drives the engine; passengers and stations use each cover's motifs (lamp posts, crystals, amphibians, the arcade cabinet).
- **Train speed.** Set per track in the cue sheet, about one tile per beat where a track has a pulse, stepped in whole pixels.
- **Sound effects.** Few, short and quiet, so they never compete with the track. Henry makes or approves them; the cue sheet's key lets pitched effects stay in tune.
- **Credit.** Track title and album appear at the start of every leg, and the pause screen names the album.
- **Files.** The masters are 2.2 GB of WAV. The game ships compressed audio, roughly 250 MB.

## Proving it

Build order, with a stop-or-go decision at each gate. Art comes after the fun is confirmed.

| Phase | What is built | Gate |
|---|---|---|
| 0 | Rules as a deterministic simulation with no graphics, plus four bots: random, greedy (nearest delivery), big-loop, and planner | On a test map: random scores no lamps, greedy about one, planner three, and big-loop below planner. Bots are limited to human drawing speed and reaction time. |
| 1 | Gray-box leg 1 with the real track and the tutorial (built; see "Phase 1 build") | Henry plays it. Go only if drawing ahead of the train feels good for five minutes and he wants leg 2. |
| 2 | World 1, six legs, still gray-box | Each map passes the bot gate; Henry finishes the world without boredom. |
| 3 | World 1 art and sound | Looks and reads clearly at integer scale. |
| 4 | Worlds 2 to 7, one at a time | Each new rule passes the bot gate and one play session before the next begins. |
| 5 | Route map, saves, credits | |

If phase 1 fails, the fallback is to change the core (for example, several trains on a network you draw before pressing run) while keeping the one-track-one-leg structure and the bots.

### Phase 0 results

The rules run in [`manyworlds/`](manyworlds/) as a Rust simulation with no graphics. Four bots play a 4:49 leg (the length of "The Corner of Maple St. and the Future") on the hand-made town map and 12 generated maps, 8 seeds each:

- **random** scribbles rail, erases, clicks and presses express at random.
- **greedy** is a newcomer who understands the rules: it always heads for the nearest station where it can deliver or board someone, and never uses express.
- **big loop** builds one loop through as many stations as the stock allows, splices in new stations, and times express between stations.
- **planner** is the best play written so far. It looks three stops ahead (who would get off, who would board, how far), routes around other stations and through tiles its tail will have left, and uses express.

Bots react every 1/3 s, drag at 7.5 tiles a second, take 1/5 s to move the pointer elsewhere, and finish one gesture before starting the next. Lamps are set per map from the planner's mean score there: one lamp at 30%, two at 60%, three at 85%.

#### Round 1: any matching passenger gets off

As first written, greedy scored **80%** of the planner and earned two or three lamps in every run: a newcomer did nearly as well as careful play. Switching one skill on or off showed why. Express alone took greedy to 94%, and looking three stops ahead instead of one added 2%. Choosing where to go barely mattered; "nearest useful station, quickly" was close to the best plan. Eleven rule variants (stock, capacity, erase delays, scoring bonuses, scarcer passengers, one shape per station) never pushed greedy below about 63% without artificial help. A 12-second delivery timer on the streak got the numbers into shape (greedy 55%), but it only magnified the gap in delivery rate and added no decision, so it was dropped.

#### Round 2: last on, first off (current rules)

Only the rear car can get off. Boarding order now decides delivery order, and passing a station's passengers by on express becomes a real choice. Three unloading rules were tried; rear-only and front-only behaved alike, and rear-only was kept because it reads naturally (a slip coach uncoupled from a moving train) and allows quick nested deliveries.

| Bot | Share of planner | Deliveries | Runs with 0 / 1 / 2 / 3 lamps |
|---|---|---|---|
| random | 0% | 0.5 | 104 / 0 / 0 / 0 |
| greedy | 51% | 46 | 2 / 85 / 17 / 0 |
| big loop | 21% | 23 | 66 / 31 / 7 / 0 |
| planner | 100% | 91 | 0 / 0 / 1 / 103 |

The gate passes without any timer: random earns nothing, greedy mostly one lamp, the planner three, and the big loop stays well below.

**Where the planner's lead comes from:**

| Variant | Share of planner |
|---|---|
| greedy | 51% |
| greedy that also uses express | 78% |
| greedy with express that also steers around stations | 79% |
| planner without express | 76% |
| planner that looks one stop ahead instead of three | 97% |
| planner that looks four stops ahead | 100% |

The skill ladder now has three rungs: learn express (+27 points), then learn which stops are worth it given the order of the cars, such as not boarding passengers who would bury the rear car (+18), and a little more from planning further ahead (+3). The middle rung is a real decision rather than dexterity, and it is the one round 1 lacked.

**Rule variants on the current rules:**

| Variant | greedy | big loop |
|---|---|---|
| Current rules | 51% | 21% |
| Any matching passenger gets off (round 1) | 80% | 5% |
| Front car only | 47% | 19% |
| Rail behind not picked up automatically | 52% | 5% |
| 12-second delivery timer | 29% | 12% |
| Stock 25 | 56% | 0% |
| Stock 60 | 44% | 38% |
| Stock 120 | 41% | 48% |
| Express still stops at stations | 47% | 19% |
| Capacity 4 | 52% | 21% |
| Capacity 10 | 51% | 22% |
| Every station its own shape | 58% | 8% |
| Half as many passengers | 69% | 23% |

**Verdict.**

- **There is now a decision worth learning.** Under last on, first off, judging which stops to make is worth about 18 points over a newcomer who already uses express, and the planner delivers twice what greedy does.
- **Looking far ahead still adds little.** This is a game of reading the next stop well, not of long plans. That suits a reactive game, but phase 1 should check whether it feels thoughtful enough.
- **Execution is unmeasured.** Drawing just ahead of a moving train and timing express are things bots do perfectly and people do not. Phase 1 tests that.

Run it again with `cargo run --release --bin gate` (current rules), `-- sweep` (all variants), `-- diag <variant>` (the one-skill variants), `-- lamps` (lamp thresholds per map), or `-- show <bot> <map> <seed> [ticks…|stalls]` (text pictures of the board).

### Phase 1 build

Leg 1 is playable in the browser, from the same Rust rules the bots play, compiled to WebAssembly with no other dependencies ([`manyworlds/web/`](manyworlds/web/)):

- **Look.** The 400 × 300 indexed framebuffer is drawn in Rust and expanded to exact N × N blocks of physical pixels at the largest integer scale that fits, centred on black, with scanlines on the bottom row of each block (F2 or the button turns them off). Colors are at 6-bit VGA DAC precision; dim overlays are checkerboard dither. Checked on the live canvas: every sampled logical pixel is an exact block.
- **Pointer.** An OS cursor made from the game's pointer sprite at scale N, fully opaque or fully clear, never drawn into the frame.
- **Clock.** The song's playback position drives the leg at 15 FPS ticks, so the leg ends with the track. Pausing pauses both, and a hidden page pauses itself rather than falling behind the song.
- **Gray-box art.** Ground, ponds and buildings; track with ties and rails, dimmed and barred on a closed switch arm; stations as colored shape plates with their queues in boarding order above; the engine with a headlamp, cars showing their passenger's shape, and the rear car's red lamp; the terminus turns purple when it opens.
- **Sound effects.** Placeholder blips made in the browser (delivery, boarding, stall, crash, lamp, rail laid), very quiet, off with M.

To try it:

```
cd manyworlds/web
./build.sh ~/Downloads/"Henry Solberg - Many Worlds - 2026 Car-Friendly Remasters.zip"
python3 -m http.server 8765 --directory www
```

then open http://localhost:8765. The zip argument only makes `www/assets/music/maple.m4a`, which git ignores. Without it the leg runs on a silent clock.

**What to watch for in the playtest:**

1. Does drawing just ahead of the train feel good for five minutes, or like chasing it?
2. Does "rear car first" make you look at a station's queue before stopping, or just feel like an obstacle?
3. Does express feel like a choice (skip that crowd) or a chore?
4. With rail picked up behind the train (default) versus by hand (L): which do you prefer?
5. Are the lamps reachable? Thresholds come from the planner bot, which never misses a beat.

## Open decisions for Henry

1. **Cue sheets.** Whether to fill in BPM, key and section times for each track, or start with evenly spaced events.
2. **Sound effects.** Made by Henry, generated in code, or none at first.
3. **Title.**
4. **Labelling.** The packs are sold as "no AI"; the game needs wording that keeps that claim true for the music.
