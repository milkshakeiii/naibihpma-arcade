# Concept checks

Run all six on every concept page before you pitch it. Write one line per check: pass or fail, and why. A concept that fails gets fixed or dropped, not pitched with a caveat.

These checks aim at the ways a design goes wrong when the designer can't play it. Each one asks a question that is easy to skip on paper and obvious in the first minute of play.

## 1. Pace fit

**Question:** when the player is thinking hardest, what is the game doing to them?

Sort the main action:
- **Thinking actions:** planning, building, arranging, routing, choosing, composing, reading a situation. These need the player to own the clock. The game pauses, or is turn-based, or waits for them, or runs a simulation they can pause at any moment (Factorio, Roller Coaster Tycoon and Dwarf Fortress all run in real time but pause).
- **Reflex actions:** steering, dodging, aiming, timing, catching. Real-time pressure suits these, and the pressure should come from things in the world (a wall, a swarm, a gap).
- **Both:** separate them into phases (plan, then run), or make the real-time part direct control of one thing while the thinking happens between runs.

**Fails when:** a thinking action runs against a clock the player can't stop. That includes the song itself used as a countdown. Henry's favourites are either pure direct control (Race the Sun, Tyrian, pinball) or self-paced (Factorio, Timberborn, chess). The middle ground, careful planning against a clock that can't be stopped, is missing from his list.

**Also check:** where does any pressure come from? It has to come from the world (a storm, a rival, a hungry colony). It must never come from an arbitrary timer, and never from the song's length.

## 2. Theme fit with the music

**Question:** does this game's world, look and mood belong to the album it's dressed in, and does it play well with no help from the music?

- **Name the album** and say what of its world shows on screen: its setting, its creatures or machines, its era, its palette. Henry's description of the album comes first, the catalog's thematic read second (`music.md`).
- **Mood:** the pace and feel of the activity match the album's. A frantic game over a contemplative, drumless album reads as wrong, and so does a sleepy one over a driving album.
- **Swap the album:** if the game would fit any of the seven equally well, the theme is paint. Put more of this album's world into it, or dress it in the album it does fit.
- **No literal music mechanics:** nothing the player does depends on hearing the track. That rules out beats to hit, pitches or keys to read, instruments as things in the world, and events scheduled to the song or its sections. The game plays the same muted. You can't hear the music to check such mechanics, and Henry dislikes them.

**Fails when:**
- the setting has nothing of the album in it
- the mood fights the music
- it would fit any album equally well
- any rule needs the player to hear the music

## 3. The player's sentence

**Question:** what would a player say they are doing?

Write it as "I'm a ___ trying to ___ while ___." It passes when:
- a player would actually say it; a designer's description ("matching shapes to stations") doesn't count
- it fits the mood of the music as Henry describes it, or as the catalog does
- it's something a person would want to be or do
- the first ten seconds of play show it: what the player sees and hears places them there

**Fails when:** the honest sentence is abstract ("I'm drawing lines so shapes match") for a game that is supposed to carry an album's world. Abstract can be fine for a pure puzzle, but then say so, and check the theme fit still passes.

## 4. Rule budget and explain-back

**Question:** could a new player say what they're trying to do, and how they're doing, after one minute?

- At most three rules before the first fun moment. Later rules arrive one at a time, each taught where it first matters.
- The goal is visible on screen within ten seconds: what am I trying to do, and how do I know I'm doing well?
- Write the guided first minute as a script, second by second: what appears, what the player does, how the game shows it worked. Every rule in it is taught by doing, and the game waits for the player while it teaches.
- Every rule creates a decision. A rule that adds something to remember without a choice is complexity, not depth; cut it.

**Fails when:**
- the goal is never stated
- more than three rules come before the fun
- a rule is taught only in text
- the tutorial runs on a timer

"Minimal text" means teaching by doing, not teaching less. The final judge is the fresh-player test (step 7 of SKILL.md), because you can't tell whether your own tutorial teaches.

## 5. Taste fit

**Question:** which of Henry's favourites is this closest to, and what exactly does it share?

Name the specific quality, not just the game: "indirect control, like Majesty", "build your engine from parts, like Cogmind", "you are the ship, like Race the Sun".

Then check it against his limits:
- no competitive twitch play
- no bolted-on minigames that break the main loop
- no optimisation homework past the point of insight
- nothing that plays like GTA, Counter-Strike, Roblox or DotA

**Fails when:** you can't name a close favourite and the concept isn't one of the deliberate far pitches, or it hits a limit.

## 6. Demo scope

**Question:** can a five-minute demo at polish 4 out of 5 be built in about a day?

Find the smallest slice that still shows the core in the first minute: one screen, one track, three rules. If even that is too big, the concept is too big for a first demo. Find its kernel, or drop it.

**Fails when:** the core only appears after systems that take days to build.
