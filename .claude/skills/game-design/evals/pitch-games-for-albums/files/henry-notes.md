
Henry is the musician behind henryscifimusic.com (seven albums, the "Many Worlds" collection; the tracks are in ~/Downloads as a zip, and stems don't exist). He wants AI to do the game design and art he says he isn't skilled at, in this repo's DOS/VGA pixel style (AGENTS.md). He is interested in improving Claude's automated game design ability itself, for example with a skill and evals. See [[game-design-lessons-many-worlds-line]].


Henry's own list, which he "spent some time thinking about":

- **Loves:** Dwarf Fortress, EVE Online, Homeworld, Warcraft III, Savage: The Battle for Newerth, Pathfinder: WotR, Timberborn, Race the Sun, Factorio, Intergalactic Fishing (a calm sci-fi fishing RPG with a mystery plot), The Witness, Noita, Siralim Ultimate, Dune: Awakening, Cogmind, Rift Wizard, Roller Coaster Tycoon, Dave the Diver "minus the minigames", Majesty, Space Pirates and Zombies, Guns of Icarus, Dual Universe, Islanders, Zeus + Poseidon, Zachtronics "but only to a certain extent", Portal, SimCity 2000 and other old Maxis games, Gauntlet Legends, Dig Dug, pinball, Smash Bros, StarCraft 2, chess, Tyrian 2000.
- **Dislikes:** GTA, Counter-Strike, Roblox, DotA.

Patterns I read in it (check with him before leaning hard on them):
- Systems that make their own stories: DF, Factorio, Timberborn, RCT, SimCity, Zeus, Islanders, Majesty (indirect control).
- Commanding at scale: Homeworld, WC3, SC2, Savage, EVE, chess.
- Building your own engine from parts and synergies: Rift Wizard, Siralim, Cogmind, Noita.
- Discovery and insight: The Witness, Portal, Noita's secrets.
- Direct-control arcade with physical feel: Race the Sun, Tyrian 2000, Dig Dug, pinball, Gauntlet, Smash.
- Sci-fi atmosphere: EVE, Homeworld, Dual Universe, Dune, SPAZ, Intergalactic Fishing.
- Limits: optimisation homework wears thin (Zachtronics), and bolted-on minigames that break the main loop are unwelcome (Dave the Diver).
- Real time is fine either as pure direct control (you are the ship) or as a self-paced sim he can pause and think in. Careful planning under an unpausable clock, the Many Worlds Line combination, is missing from the list.

See [[henry-musician-game-maker]], [[demo-presentation-prefs]], [[game-design-lessons-many-worlds-line]].


Henry's answers on how to show him game design work:
- Start with two or three one-line pitches, a mix of near and far ideas. No design doc unless he asks.
- After he picks, go quiet until there is something to play. Don't check in at every step.
- Polish bar is 4 out of 5: gray boxes don't let him judge. A demo needs the real music, real pixel art in the repo's DOS style, and real sound.
- Teach the rules with a guided first minute inside the game, with nothing written next to the demo.
- He decides in about five minutes, so the core has to show up in the first minute.
- Present neutrally: no selling, no spin.
- His feedback channels: keep / change / drop controls in the demo, notes while he plays, then talking it through.
- On 2026-11-26 he wanted keyboard-only controls "today", so ask about input each session.

**Why:** The Many Worlds Line gray-box failed partly on immersion and unexplained rules, and he chose this style himself.

**How to apply:** fewer, more finished demos rather than many gray boxes. Build the in-game guided minute and the keep / change / drop and note controls into the shared demo kit.

See [[henry-game-taste]], [[game-design-lessons-many-worlds-line]].


Henry played leg 1 of Many Worlds Line (draw rail ahead of a train that never stops; the song is the clock) and rejected the concept: the time limit made no sense for what felt like a deliberate track-drawing puzzle, the theme did not match the music, there was no immersion, and the rules were never explained.

**Why:** I validated what bots can measure (strategic depth, a skill ladder) and treated that as fun. The music was only a timer and background: swapping in any other 5-minute file would change nothing. The tutorial was two-word captions, and I checked it by playing it myself while already knowing the rules. About 5,400 lines were built before anyone had a concept in their hands.

**How to apply:** for any game design for Henry:
- Match the music through theme, world and mood, and keep the mechanics a real game on their own (he liked "pinball tables, one per album"). He dislikes literal music mechanics: instruments as creatures, playing by ear, rhythm play, events timed to the song. He didn't like Crypt of the NecroDancer. He asked for all literal music to be left out, because a model that can't hear can't get it right. The music is the soundtrack, and the albums supply world and mood. The music shouldn't override the player's own strategy (in pinball, lighting shots belongs to the player). Corrected 2026-11-26: I had first read his "theme doesn't match the music" complaint as "the music must drive the mechanics", which was wrong.
- Pressure must come from the fiction or the music, never an arbitrary clock.
- State the goal in the first seconds, and have a fresh subagent with no context explain the rules back from the build.
- Offer two or three rough concepts he can play before investing in one; bots are for balance after he says the core is fun.

Blind test, 2026-11-26: four fresh agents reviewed the old DESIGN.md (two with the saschb2b/skills game-design skill, two without) and a blind grader scored them against Henry's four complaints (out of 8): with the skill 5 and 4, without 4 and 4. That's no real difference. All four caught "the music is only a clock"; none caught that real-time pressure was wrong, none doubted the tutorial, and all said "continue with changes". So run a cheap fresh-agent review before building (it would have caught the music problem), but text reviews can't catch pace or clarity: that needs someone playing the build.

Henry wants to improve Claude's automated game design ability, not just this game. See [[henry-musician-game-maker]].
