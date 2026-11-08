# Two roguelikes, one question

This folder holds two contrasting roguelike prototypes behind one launcher. Each run ends with a short in-game feedback form, and the answers go to `playtest_log.txt` together with a turn-by-turn log. The aim is to find the direction worth building into a full game through play alone. The player never reads this file: each game teaches itself, and every redesign has to teach itself again (the save stores which lesson version each game last taught).

Both games follow [`../../AGENTS.md`](../../AGENTS.md): 400x300 indexed 64-colour VGA palettes, exact integer scaling, F2 scanlines, 15 FPS ticks with 250 Hz input latching, a hardware pixel-art pointer, no alpha. All music comes from the player's albums.

Earlier rounds are in git (round 1: commit f870cdc) and in `archive/round2/` (not compiled).

---

## History

| Round | A | B |
| :-- | :-- | :-- |
| 1 | **Changeling** v1: one piece; you take the shape of what you catch. Fun 3, decisions 3, difficulty 2, again 1. Liked: chaining catches, chess inspiration. "Couldn't see how the enemies' moves were decided." | **Turncoat** v1: place a band, watch; the defeated change sides. Fun 2, decisions 1, again 1. "Where are the interesting choices?" It pointed to the player's own *curses* and *Creature Builder*. |
| 2 | **Changeling** v2: planned moves shown as arrows, chains as a rule. No ratings: "an improvement but I haven't found the fun... less fun than chess puzzles... think outside the chess box... a full redesign... keep the music and amphibian theme." | **Conduit** v1: gear monsters into held/worn/inner slots, tick battles over conduits. Fun 2, decisions 1, gearing 1, again 1. "Too simplistic... no ways to grow my creatures... didn't feel proud of my creatures because I didn't design them or level them up... the equipment wasn't interesting... the music didn't fit... the conduit-hit sound was annoying." |
| 3 | **GULP** (below) | **HATCHERY** (below) |

Round 3 also replaced every sound effect with softer, pond-like ones, and the same effect can no longer restart within 70 ms. Both games use *The Enchanted Forest of Naibihpma*.

---

## A: GULP

*A frog in the forest of Naibihpma. Hop, tongue, spit, croak; eat to grow; keep your frogspawn alive.*

### Rules
- **Turn:** one hop (one square, any way) and one action, in either order. Click yourself or press Space to end the turn.
- **Tongue (T, or right-click a line):** the first thing along one of 8 lines, within 3. Smaller than you: swallowed. Your size: yanked to the square next to you (a non-swimmer yanked into water drowns). Bigger than you, or reeds and stones: you're pulled over to it. The tongue is also movement, at the cost of your action.
- **Spit (S):** shoots what's in your throat along a line (range 4): 1 damage plus its size, and the target is shoved a square.
- **Gulp (G):** digest it. Food grows you: Small, then Medium at 3, then Large at 8. Each size adds a heart, heals 1 and lets you swallow bigger things. A Large frog sinks lily pads.
- **Croak (C):** shoves everything next to you a square (every 3 turns).
- **Shoves:** into water drowns most creatures; into a wall, an egg clutch or another creature costs both 1.
- **Telegraphed strikes:** after their turn, creatures move and declare strikes, shown in red with a marker for direction. Strikes land when your turn ends, relative to where the striker stands then, so shoving or yanking a creature moves its strike, possibly onto another creature.
- **Frogspawn:** each pond has a clutch or two of your eggs. Predators go for whichever is nearer, the eggs or you, and choose the eggs on a tie. A strike on a clutch, or a creature shoved into it, costs you a heart. Each clutch that survives the pond heals you 1.
- **Pond:** clear the predators (flies don't count; reinforcements ripple in at the edges a turn ahead), then hop to the exit. Flies keep drifting in so there's always food.
- **Between ponds:** choose 1 of 3 mutations (Long Tongue, Big Throat, Poison Skin, Spring Legs, Acid Spit, Deep Croak, Gills, Sticky Tongue, Glutton, Camouflage), or Mend when you're hurt.
- **Run:** 10 ponds across the Underbrush, Star Pond and the Deep Pool, with Grof's Hollow last. Creatures: fly (food), beetle (charges in a line), newt (bite), dragonfly (diagonal dive, flies), snake (2-square lunge, swims), crayfish (3-square pinch, armored), heron (stabs 2 away, too big to swallow), Grof (stomps all around, calls help).

### Why it's built this way
- *Outside the chess box:* the verbs are physical (pull, yank, grapple, spit, shove), and you play with water, walls and other creatures. It doesn't ask you to calculate shapes.
- *Predictable:* every strike is shown and committed; hovering any square shows exactly what would hit you there.
- *The tension is the eggs, not dodging.* Without them, a greedy bot won 19 of 20 runs, because a mobile frog can always step out of committed strikes. Eggs can't move, so you have to intervene: Into the Breach's buildings, given a body.
- Healing is scarce: eating doesn't heal (except with Glutton or Gills); saved clutches and growing do.

### Balance at v1
A greedy bot (swallows and spits when it can, croaks threats away from eggs, hops to the safest square) wins 5 of 20 and usually dies around ponds 4-5, nearly all from lost eggs. It barely protects the clutch, so a player who does should do much better.

### Lessons
1. Hopping. 2. Tongue and gulp (grow to Medium). 3. Strikes and the egg clutch; beetles charge; water drowns. 4. Spit and croak; shoved creatures strike from where they end up. Each lesson's exit stays shut until the last step.

---

## B: HATCHERY

*Doat's hatchery at the Star Pond. Hatch familiars, design them gene by gene, raise them, and watch them defend the eggs from Grof's creatures, night after night.* Built on the player's Creature Builder and curses, and on the round 2 feedback: design, growth, pride.

### Rules
- **Eggs are lives and currency.** The clutch starts with 7. A creature that reaches the nest carries one off. Hatching a familiar costs an egg, and a new egg is laid each dawn. No eggs left means the run is over.
- **Hatching:** choose a body, then its first gene. Bodies: Frog (quick, head/legs/skin), Toad (tough, skin/head/legs), Newt (fast, frail, tail/head/skin/legs), Salamander (fighter, head/tail/skin/legs), Axolotl (regrows in water, tail/skin/head). Each body opens its gene places in its own order.
- **Genes (18)** change behaviour, not just numbers. Head: Long Tongue (bites from 3 away and reels prey in), Wide Mouth (swallows the weakened whole), Venom Fangs, Fire Gland (breath down a line), Chorus Sac (nearby familiars act again). Skin: Warts, Slime, Glow Spots (allies bite harder, foes come for it), Bright Colors (foes avoid it), Bark Skin. Legs: Leaping Legs, Sticky Pads, Webbed Feet, Digging Claws (buried until its first bite). Tail: Split Tail, Regrowth, Egg Sac (tadpoles hatch when it falls), Club Tail.
- **Growth:** surviving a night gives 2 experience, and each foe a familiar finishes gives 1 more. At 3, 7 and 12 it levels up (Tadpole, Froglet, Adult, Elder), gains HP, and opens its next gene place. You choose the gene. A familiar's sprite shows its genes.
- **Learning:** each foe carries a gene (shown on hover). After a night, learn one new gene from what you defeated.
- **Night:** place familiars on the four columns by the nest, then watch a deterministic tick battle. Foes come from the right and head for the nest, biting what's next to them (crows fly over everything). Familiars go for the nearest foe. When a gene acts, its name floats up. A familiar that falls is gone for good.
- **Run:** 8 nights. New foes each night (rat, beetle, crow, hedgehog, water snake, leech, heron, toadling), and Grof comes on the last.

### Balance at v1
A naive bot (random bodies, always the first gene offered, everyone in one column) never wins; it reaches night 6-8 about half the time. Designing for synergy and placing with intent should change that.

### Lesson
Hatch two familiars (body, then gene), let night fall, drag them onto the pond, start. The first night pauses when a gene first acts and when an egg is first taken. Growing and learning genes are taught by tips at the first dawn.

---

## Feedback form (both games)
Five-point ratings: fun, clarity ("did outcomes make sense"), interesting choices, difficulty (too easy to too hard), wanting another run, plus one per game (GULP: "tongue, spit and croak felt fiddly to delicious"; Hatchery: "designing your familiars was busywork to the best part"). Two optional free-text answers: the best moment, and what bugged you or what you'd change.
