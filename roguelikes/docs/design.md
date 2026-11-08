# Two roguelikes, one question

This folder holds two contrasting roguelike prototypes behind one launcher. Each run ends with a short in-game feedback form. The answers, plus a log of what happened in the run, go to `playtest_log.txt`. The aim is to find which direction is worth building into a full game, through play alone. The player never reads this file. Each game teaches itself, and every redesign has to teach itself again (the save stores which lesson version the player has seen).

Both games follow [`../../AGENTS.md`](../../AGENTS.md): a 400x300 indexed 64-colour VGA palette, exact integer scaling, F2 scanlines, 15 FPS ticks with 250 Hz input latching, a hardware mouse pointer, and no alpha (dims and danger marks use checkerboard dither).

They differ on the axis that most separates roguelikes: **where the decisions live**.

| | A: CHANGELING | B: TURNCOAT |
| :-- | :-- | :-- |
| You control | one piece, every move | a band of people you place, then watch |
| Information | complete and exact (danger squares, hover previews) | complete before battle, then it plays out |
| Decision rhythm | every turn, about 10 s each | about once a minute (route, who stays, where to stand) |
| Tension comes from | getting surrounded; a strong shape can turn weak | snowballing conversions; whom to recruit |
| Run | 3 lesson burrows on the first run, then 10 burrows (3 areas + Grof) | 1 lesson battle, then 13 battles over 4 ages + a finale |
| Music | *The Enchanted Forest of Naibihpma* (s3) | *Virtual History* (s5) |

Both draw on threads from the player's own designs: chess shapes as a movement vocabulary, converting instead of killing, and (in B) build-then-watch under a size budget.

---

## A: CHANGELING

*You are a changeling in the forest of Naibihpma. You move like the last creature you caught.*

### Rules
- An 8x8 board with checkerboard ground (so diagonals and leaps read at a glance), mossy rocks, dew drops and a **burrow** (the way down).
- **Shapes.** Every creature moves and strikes with one shape: a step, a diagonal step, a leap, or a line. Lines stop at rocks and at anything standing in them. Leaps jump over everything.
- **You.** Your own shape is one step in any direction. You can move to any lit square: empty ground, dew, the burrow, or a creature.
- **Catch.** Moving onto a creature catches it. It's gone, you get its dew, and **you take its shape** until you catch something else or rest.
- **Rest.** Click yourself: you become yourself again, and the creatures get a turn.
- **Creatures' turn.** After every move or rest, each creature that can reach you **strikes** (you lose a heart). The others move toward a square they could strike you from. Red squares show where you would be struck. Pointing at a lit square shows exactly what would happen if you moved there.
- **Pressure.** Every few turns (6, 5, then 4 by area) something rustles at the edge of the board, and two turns later a new creature arrives there.
- **Sealed burrows.** Roots seal each burrow until you've caught 2 (Underbrush) or 3 (later) creatures in it. This came from bot testing: when the burrow was always open, a greedy one-move bot won 9 of 12 runs by slipping past everything.
- **Glade.** Between burrows, spend dew on charms (rule changes) or on mending a heart.
- **Grof's Hollow** (burrow 10). Grof shrugs off the first two catches: each costs him a ward, and he flees across the board and calls two creatures. The third catch wins.

### Creatures
| Creature | Shape | Dew | From |
| :-- | :-- | :-: | :-- |
| Newt | 1 step, straight | 1 | Underbrush |
| Beetle | 1 step, diagonal | 1 | Underbrush |
| Frog | knight leap (2+1) | 2 | Underbrush |
| Cricket | hop exactly 2 in any of 8 directions | 2 | Dewfields |
| Heron | straight lines up to 3; *wary* (won't stop where you could catch it) | 3 | Dewfields |
| Snake | diagonal lines up to 3 | 3 | Star Pond |
| Salamander | long leap (3+1) | 3 | Star Pond |
| Glowtoad | lines up to 2 in all 8 directions; catching it costs you a heart | 4 | Star Pond |
| Grof | lines up to 3 in all 8 directions; 3 wards | - | Hollow |

### Charms
Bought with dew (4 to 10 each; Mend costs 4). The glade offers 3 charms you don't own plus Mend. Thick Skin (+1 heart), Mend (heal 1), Pounce (catches can chain within one turn), Two Skins (move as either of your last two shapes), Dew Sense (+1 dew per catch), Shell (the first strike on each burrow is blocked), Long Shadow (your lines reach 2 farther), Startle (catching pushes neighbours away and dazes the ones that can't move), Second Wind (heal on leaving a burrow unhurt).

### Balance at v1
Creatures per burrow: 4, 5, 5 / 5, 6, 6 / 7, 7, 8 / Grof + 3. Over 30 runs, a cautious greedy bot (one-move lookahead, buys everything it can) won 14. Most of its deaths came in the Dewfields, where the wary herons arrive. A person thinks further ahead than the bot but will make mistakes on a first run, so expect the first runs to end in areas 2-3.

### What it tests
Do the turn-by-turn tactics of a single piece hold up? Does "your power changes with what you take" create real choices (take the newt now and lose your leap, or dodge it), or is it a chore? Is complete information exciting or dry?

### Lessons (first run, and after any redesign)
1. *First Steps*: lit squares and the burrow.
2. *Catching*: take a newt's shape, then rest.
3. *Danger*: red squares, strikes, lines blocked by rocks, dew.

Each lesson teaches a rule, never how to solve the board.

---

## B: TURNCOAT

*History is a contest of persuasion. Whoever runs out of resolve joins the side that won them over.*

### Rules
- A 7x6 field. Your band (blue) stands on the bottom three rows, the rivals (red) on the top three. Before each battle you see the rivals' formation and drag up to N of your people onto your rows.
- Each person has **resolve** (hit points), **voice** (how much resolve they take), **pace** (who acts first) and **reach** (a shape; "ahead" means toward the other side).
- **Battle** (automatic, in rounds). In order of pace, each person persuades the rival in reach with the least resolve left (ties go to the nearest). If no rival is in reach, they walk one step toward the nearest square that has one.
- **Turning.** When someone's resolve hits 0, they **join the side that won them over**, with half their resolve back. They keep their kind and their reach.
- The battle is won when everyone on the field carries your banner. After 30 rounds, whoever has more people wins; a tie is a draw.
- **After a win**, everyone on your side may stay (including people who sat on the bench). Your band holds 7; choose who stays. 4 people fit on the field at first, +1 for each leader beaten. A person who turns three others becomes a *veteran* (+1 voice, +2 resolve).
- **A loss** costs one of 3 chances. Your band is unchanged.
- **Route.** After each battle you pick which of two rival bands to meet next, so you choose whom you could recruit.
- **Ages.** Prehistory, Iron Age, Golden Age, Space Age: two battles, then the age's leader. Beating a leader gives an **idea** (pick 1 of 3) and one more deployment slot. A Myth is the final battle.

### People
| Person | Resolve | Voice | Pace | Reach | Trait |
| :-- | :-: | :-: | :-: | :-- | :-- |
| Gatherer | 4 | 1 | 2 | 1 step, straight | |
| Hunter | 3 | 2 | 3 | 1-2 squares straight ahead | |
| Elder | 3 | 1 | 1 | anything within 2 steps | never walks |
| Shaman | 3 | 1 | 2 | 1 step, diagonal | the people it turns join at full resolve |
| Smith | 6 | 1 | 1 | 1 step, any direction | sturdy: takes 1 less |
| Rider | 3 | 2 | 4 | knight leap | walks 2 steps |
| Herald | 3 | 1 | 3 | the whole column ahead | persuades everyone in that column |
| Scholar | 3 | 3 | 2 | straight lines up to 2 | |
| Priest | 4 | 1 | 1 | 1 step, any direction | neighbours can't be turned (they hold at 1) |
| Merchant | 4 | 1 | 3 | 1 step, straight | turning someone restores its own resolve |
| Broadcaster | 3 | 1 | 2 | straight lines, any length | |
| Astronaut | 4 | 2 | 5 | hop exactly 2, any direction | can stand anywhere on the field |
| Engineer | 5 | 2 | 2 | 1 step, any direction | builder: its neighbours take 1 less |
| Chieftain (leader) | 9 | 2 | 2 | 1 step, any direction | rally: neighbours speak 1 louder |
| Warlord (leader) | 8 | 3 | 4 | knight leap | swift, sturdy |
| Emperor (leader) | 11 | 2 | 2 | anything within 2 steps | faithful |
| Director (leader) | 9 | 2 | 3 | lines any length, 8 directions | rally |
| A Myth (final) | 15 | 3 | 3 | anything within 2 steps | awe (turns join at full), sturdy |

### Ideas
Pick 1 of 3 after each leader. Oral Tradition (Elders reach 3; Heralds and Broadcasters speak 1 louder), Bronze (+2 resolve for everyone at deployment), Roads (+1 pace for everyone), Writing (your converts join at full resolve), Banner (+1 deployment slot), Big Tent (+2 band size), Rhetoric (+1 voice while at full resolve), Walls (your back row takes 1 less), Medicine (each round your most shaken person regains 1).

### Balance at v1
A bot that places its strongest people at random on the front two rows and picks routes at random won 2 of 10 runs, usually losing its third chance in the Iron or Golden Age. Placement against the visible formation should do much better.

### What it tests
Is setting up a fight and watching it play out (your lineage's build-then-watch) more compelling here than direct control? Does recruiting your enemies make the route and the "who stays" choices meaningful? Are the battles readable enough to learn from a loss?

### Lesson (first run, and after any redesign)
One guided battle: drag people onto lit squares, point at someone to see their reach, press START. The battle pauses at the first turning to name the rule. Then it shows who stays and how the route choice works.

---

## Feedback form (both games)
Five-point ratings: fun, clarity ("did you understand why things happened"), interesting decisions, difficulty (too easy to too hard), wanting another run, and one game-specific question. Two optional free-text fields: the best moment, and what bugged you or what you'd change. Everything is appended to `playtest_log.txt` with the run's log.
