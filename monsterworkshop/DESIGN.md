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

**v1.5:** equipment, as physical crafted wearables (see below).

## Features

### Recipes
There is no recipe menu. What's on a workshop's pads decides what it makes. When two recipes take the same inputs, a signboard on the workshop wall shows the current one, and walking into the board switches it. Empty pads show a dithered ghost of what belongs there, as they do in the prototype.

### Tools
Workshops have tool pegs on the wall, which you fill by pushing tools onto them, the same way pads work. Each tool shows a small durability bar. When a tool breaks, a looping monster stops with a "? + hammer" bubble.

### Hiring
There is no character-creation screen. A **hiring hall** in town holds applicants. Each one is already rolled with a monster type and 3 transferable skills. Walk up to an applicant and their bubble shows type, skills, stats and renown cost. Space hires them. You choose which applicant to hire, not which boxes to tick, and the roster refreshes over time.

### Renown and goals
Renown is a HUD counter. You spend it in the world: hiring at the hall, and contributing to **town project boards**. A project board shows what goods it wants and a progress bar, and finished projects unlock things like the road to the second town or new workshops. A final "great work", for example a temple bell, ends 1.0.

### Delivery and shares
Delivering a good is a small celebration. "+12 RENOWN" rises from the drop-off, then smaller credits fly to each monster that contributed ("MOGG 6, BRUNA 4"). This is how the shares system becomes visible. All renown still goes to the one commune.

### Roster
Tab cycles monsters. A monster that's off-screen gets an **edge marker**: an arrow at the screen edge with its state icon (looping, waiting, idle), so a stuck monster is noticeable from anywhere. Holding Tab shows a strip of portraits with states in the HUD.

### Skills
Skill growth is shown as it happens. A small "+SPINNING" floats up when a skill improves, and a skilled monster visibly works faster: the progress bar fills quicker. Exact numbers are on the monster's inspect card.

### Items and quality
Quality is shown on the sprite: brighter is better, and exceptional items sparkle. Tools carry a durability bar. The inspect card gives the details.

### The inspect card
Hold the inspect key (to be chosen by feel, e.g. Shift) to show a card anchored next to the tile you're facing. If that tile is empty, the card describes the monster you're controlling.
- **Item:** name, quality, durability (for tools), carried-over tags, and who contributed to it.
- **Monster:** name, type, ability scores, the 3 transferable skills, and the top applied skills as bars.
- **Workshop:** current recipe, what's still missing, spin time.

The card is drawn in the same beveled style as the help overlay. It never pauses the world.

### Wagons and roads
Wagons are fully physical. Press Space on a wagon's handle to hitch it, and it follows behind you. Push items onto its bed, where they stack visibly, and push them off at the destination. Roads leave the map edge at signposts naming the next town. Recorded loops can include wagon trips.

### Equipment (v1.5)
Gloves, glasses and the like are crafted goods from the tech tree, not bought with renown. Push one into a monster to equip it. It then shows on the sprite (a goblin in glasses), and its bonus appears on the inspect card.

## Milestones

- **M1, economy:** port the rules from sokobanmonstermakers, using its 92 tests as the spec. Complete the tech-tree data for the first chain. Add quality on sprites, skill popups, the inspect card, delivery credits, the renown counter and the hiring hall.
- **M2, second town:** roads and signposts, wagons, town project boards, edge markers and the Tab roster strip.
- **M3, content and art:** chains 2–3 and the great work. Replace placeholder sprites with final pixel art under the AGENTS.md palette rules.
- **M4, ship:** itch.io build and outside playtesters.
- **v1.5:** equipment.

## Open questions

- **Saving:** a local-first game needs a save file (world, monsters, routines). Format and autosave cadence are still to decide.
- **Offline progress:** either monsters only work while the game runs, or the game catches up a capped amount of time on launch. The current lean is only-while-running for 1.0.
- **Applicants:** how many are offered at once, how often they refresh, and how cost scales with type and skills.
- **Inspect key:** which key feels best. Try it in play.
