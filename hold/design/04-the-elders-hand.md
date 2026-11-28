# Naibihpma Hold, document 04: the Elder's hand

Draft 1, 2026-11-27. The fourth of five design documents. It covers everything the player does: the rewards, the other levers and how they're worked, the Ancient Magic, and the screen they're all worked from.

The Elder can't move anyone (document 01). Everything here is a way of wanting something and letting the Hold respond.

**Controls are Majesty's, with the demo's keyboard play.** Click things on the map to select them, click the panel's buttons, and right-click to back out. Every button also has a key, shown as a yellow letter in its name (usually the first), and Tab steps through everything on the map *(demo)*.

## The levers, in one place

| Lever | Where you work it | Costs | Document |
|---|---|---|---|
| Rewards | any target on the map | dew, set aside | below |
| Stock levels | any building's panel | nothing directly | 01, 02, below |
| Building | the building it's built from | materials, from that building's stock | 02, below |
| Hatching | a building that houses the kind | dew or orbs | 02, below |
| Transport routes | a building's panel | nothing | 01, below |
| Puddles | the map | 5 dew each | below |
| The Ancient Magic | the spell bar | dew, then a wait | below |

## Rewards

**New:** four kinds of reward. The demo had one, Attack.

| Kind | Placed on | Done when | Paid to |
|---|---|---|---|
| **Attack** | an enemy creature or lair | it's destroyed | everyone who hit it, split by damage *(demo)* |
| **Explore** | any square, including unexplored ground | a hero has stood there | the first to arrive |
| **Guard** | one of your buildings | the next raid on it is beaten, or dawn comes | everyone standing guard when it ends |
| **Fetch** | a husk, an orb, or anything lying on the ground | it's carried to where it belongs | the carrier |

**How a reward works:**
- **Setting aside.** Posting a reward sets its dew aside from the Hall *(demo)*. It starts at 20 and moves in steps of 10 with More and Less (M and L) *(demo)*. Cancel (C) gives the dew back *(demo)*. Majesty never gave it back; the Hold does, so you can change your mind.
- **The asking price.** Each hero names its price for each reward, from:
  - the danger
  - the dry ground on its way
  - its temperament
  - how many others have taken it *(demo)*
  - **New:** a grudge or a dream lowers it, as in document 01
- **The candidates.** A selected reward lists who is weighing it and why each declines: "3 TOO DRY, 2 WANT MORE, 1 TOO WEAK" (document 01). This is the main thing you read to decide whether to raise it.
- **The tithe.** A tenth of each payment goes to the hero's house (document 02).
- **At most six rewards at once.** The demo allowed four.

**Notes on each kind:**
- **Attack on a lair** is how nests, Gorf's Stones, leech pools, the crow roost and Gorf's Hollow fall. Heron nests heal fast (document 03), so a small reward on a nest tends to bring a lone hero who can't finish it.
- **Explore** uncovers the map (see "The screen"), and it's how far nests are found before a goliath frog finds them for you.
- **Guard** is the answer to a dream of the next raid. Post it on the building the raid will come to, and heroes gather there.
- **Fetch** is for the husk you want regrown, the orb lying among the roosting herons, or the crows' spilled hoard. Folk already carry things that are wanted. A fetch reward sends a hero into danger to do it.

## Stock levels

A building's Stock button (S) opens its stock window over the map: a bevelled box 200 pixels wide, with one line for everything the building keeps:

```
FUNGIWOOD   2/2   + -
CAP INGOT   0/2   + -    WAITING: IRONCAP CAVE RIPENING
SLINGSHOT   1/3   + -
```

- **Reading a line.** Each line shows what the building has, what it keeps, and two buttons. Selecting a line and pressing + or − does the same.
- **The limit.** A building can't keep more than its limit (document 02). A reservation for something being built or hatched may go over it.
- **Why it's short.** The window's bottom line says why the selected line is short, in plain words (document 02): "NONE SPARE", "NOBODY SKILLED ENOUGH", "IRONCAP CAVE RIPENING".
- **Defaults** are document 02's. Defaults (D on the panel) puts a building back to them.

## Building

1. Select the building you're building from (the Hall, the Burrow, the Chorus Log and so on: document 02 says which). Press Build (B).
2. Choose what to build. Anything you can't build yet is listed grey, with what it's waiting for: "NEEDS A SUN ORB".
3. Click where it goes. The site shows the building's outline and, while you place it, its spots and how much of the ground around it is damp.
4. The building you built from now asks for the cost, and its panel shows what's still to come. When everything has arrived, the site appears at a tenth of its health, and folk dig it up the rest of the way (up to four at once, document 02).

Cancel (C) on a site that hasn't appeared yet gives the materials back to the building's stock.

## Hatching

Select a building that houses a kind, and press Hatch (H). The cost is taken from that building's stock as soon as it's there (document 02). The eggs show in the building's pool, and a tadpole hops out when the hatching is done (document 01).

**Auto-hatch** (A on the building's panel) keeps the building full: it hatches again whenever one of its kind dies or moves away. It's on by default for every kind, so the Hold fills without being taught; turn it off to save dew or orbs.

## Transport routes

Select a building, press Route (O), and click the building it should feed. While either one is selected, the route is drawn as a dotted line of single pixels between them. Unroute (N), on either building's panel, removes it.

## Puddles

Press Puddle (U) on the Hall's panel, then click a square of dry ground within 20 squares of a building. A newt with a shovel digs it.

- **What it does.** A puddle is a square of water. It clears Dry (document 01), and damp ground spreads one square around it.
- **Cost.** 5 dew each, to keep the map from filling with them.
- **Where it can't go.** Not on cursed ground.
- **How long it lasts.** It lasts until the curse spreads over it, or a dry spell shrinks it (document 03).
- **Pools.** Goliath frogs widen puddles into pools unasked (document 01).

## The Ancient Magic

The Elder's own spells, named after the album's last track. The Elder is too old to move, but old enough to remember the marsh's first magic.

**New.** Each spell comes from a building. It costs dew from the Hall, and then has to recharge.

| Spell | From | What it does | Cost | Recharge |
|---|---|---|---|---|
| **Rain-call** | The Hall | Rain over a 15-square circle for 900 ticks (document 03's rain, but in one place) | 40 | a day |
| **The old song** | The Chorus Log, once the Chorus Toad is level 5 | Bolsters every hero within 7 squares of the Chorus Log at once | 30 | half a day |
| **Starlight** | The Star Pond | Uncovers a 12-square circle for a day, and shows every orb carrier inside it, as a dream does | 25 | half a day |
| **Ley-light** | Daot's Grotto | Lifts Gorf's curse from a 7 by 7 patch for a day | 40 | a day |
| **The deep calling** | The Olm's Cave | Wakes the Olm at once. What it does then is its own choice. | 50 | a day |

**Casting.**
1. Press F4 or click the spell bar, then choose a spell with its number key (1 to 5) or a click.
2. Click where it goes. The spell stays on the pointer, so you can cast again if you can pay, until you right-click (Esc) to put it away.

**Why these five.** Each one works through a system documents 01 to 03 already have: Dry and the ferns, Bolster, the dreams' orb sight, the curse, and the Olm's sleep. None of them is a fireball. The heroes do the fighting.

## The screen

Everything fits a 400 by 300 framebuffer, at an exact integer scale with black borders, and the F2 scanlines (AGENTS.md).

**Layout:**
- **The map** fills the top 400 by 240: 25 by 15 squares of 16 pixels *(demo)*.
- **The panel** fills the bottom 60 *(demo)*. From left to right:
  - **The selection box,** pixels 2 to 134. It shows name, kind, level, temperament, a health bar, the purse, gear icons, and the sentence of what it's doing (document 01), which may take two lines.
  - **The buttons,** pixels 137 to 281: up to six, in two rows of three, each with its yellow letter.
  - **New: four small icon buttons** in a column, pixels 284 to 296: Pause, Fast, Magic and Screens.
  - **New: the minimap,** pixels 299 to 398: 100 by 45, one pixel a square, with the view drawn as a box.
- **Lines of text** are one line of the demo's font, at most about 60 characters. Every line in documents 01 to 05 is written to fit.
- **The spell bar** opens as a row of five icons just above the panel.

**Scrolling (new):**
- **Ways to scroll.** The map is several screens across (document 05). You can scroll with:
  - the pointer at a screen edge
  - the arrow keys when no map cursor is out
  - a click or drag on the minimap
  - Home, which centres the Hall
- **Pixel-exact.** The view scrolls in whole pixels, 8 a frame, so nothing shimmers and the whole map takes about 13 seconds to cross.

**Unexplored ground (new).**
- At the start only the Hold and its surroundings are seen. The rest is black.
- Any creature of yours uncovers what it can see, and uncovered ground stays uncovered.
- Enemies on uncovered ground you can't currently see are shown where they were last seen, in the dark checkerboard dither, with no alpha blending (AGENTS.md).

**Day and night** remap only the background ramps (document 01), so the panel and every sprite keep their colours.

**Calm by rule:**
- Nothing flashes or strobes.
- Palette cycling is kept to water glints, spores and fireflies.
- Events arrive as one line of text above the panel, held for a few seconds: "A HERON TOOK WREN BY THE STREAM". Sound effects stay quiet, under the music.

**The pointer** is the OS hardware cursor, built from the game's own pointer sprite at the current scale (AGENTS.md).

## Information screens

| Screen | Key | What it shows |
|---|---|---|
| **Heroes** | F5 | Every hero, one line each: name, kind, level, temperament, purse, what it's doing. Click a line to select that hero and centre the view on it. |
| **Folk** | F6 | The same for folk, grouped by house. |
| **Rewards** | F7 | Every reward, its dew, its candidates and who has taken it |
| **Log** | F8 | The last 40 event lines |

These replace the map while they're open and pause nothing. Esc closes them.

## Keys

Letters belong to the panel's buttons, so they mean different things on different panels. The one exception is P, which always pauses. These keys work anywhere:

| Key | Does |
|---|---|
| P or Space | Pause *(demo)*. Everything above works while paused. |
| F3 | Fast: two ticks a frame. The screen still updates at 15 frames a second. |
| Tab, Shift+Tab | Step through everything on the screen *(demo)* |
| Arrows, then Enter | Move a map cursor and pick *(demo)* |
| Home | Centre the Hall |
| Esc or right-click | Back out *(demo)* |
| F2 | Scanlines *(demo)* |
| F4 | The Ancient Magic's spell bar |
| F5 to F8 | The information screens |
| F9 | Playtest feedback: keep, change, drop, or a note *(demo, moved from F so letters stay free for buttons)* |

Pause, Fast, the spell bar and the information screens also have the icon buttons in the panel. The pause screen has Restart (R) and Quit (Q).

## What a first minute uses

Document 05 builds the guided start from these, in this order:
1. Select a nest and post an Attack reward.
2. Watch a toad weigh it and take it.
3. Raise a reward with More until a doubting toad takes it.
4. Build the Ironcap Cave from the Chorus Log, and see on the Chorus Log's panel what it's waiting for.

Stock levels, building and the Ancient Magic come after the first minute, as the game asks for them.

## What comes next

Document 05: the first scenario.
