---
name: game-design
description: Design games for Henry and show them to him, first as one-line pitches, then as playable demos. Use this whenever Henry asks for a game, a game idea, a game for his music or one of his albums, a new concept, a prototype or demo, a pivot, or a review of a game design or plan, or when he reacts to a demo he played (keep, change, drop, notes), even if he never says "design". Use it before building any new game in this repository too. It covers what to read first, generating and checking concepts, an outside review, pitching, building a demo he can judge in five minutes, a fresh-player test, and acting on his verdict.
---

# Game design for Henry

Henry is a musician. His collection *Many Worlds* is seven sci-fi albums, 43 tracks, and he wants games that use them. Games in this repository look like early-1990s DOS games (see `AGENTS.md`). He isn't a game designer or an artist, and he judges by playing.

Two things make this hard for you. You can't hear the music, and you can't feel how a game plays. A design can read well on paper and fail in its first minute, and the reviews you write yourself share your blind spots. So the process below does four things:
- It starts from what Henry loves, and takes each game's world and mood from his albums.
- It spreads wide before committing to anything.
- It checks every concept against the known ways games go wrong, then gets an outside view.
- It puts something finished-looking in his hands fast and learns from what he says.

Because you can't hear, keep the music out of the mechanics. Nothing the player does should depend on hearing it: no beats, pitches, instruments or section timings driving play, and no events scheduled to the song. Henry has rarely seen that work, and you'd have no way to check you got it right. The music is the soundtrack, and each album supplies a game's world, look and mood. The game has to be good on its own.

## How Henry wants to be shown things

These come from his own answers to a questionnaire. If project memory or `~/.claude/CLAUDE.md` says something newer, the newer one wins.

- **First contact:** two or three one-line pitches, a mix of ideas near his favourites and far from them. No design document unless he asks.
- **Then quiet:** after he picks, don't check in again until there's something to play.
- **Polish bar 4 out of 5:** placeholder gray boxes don't let him judge. A demo has the real music, real pixel art in the repo's style, and real sound.
- **Teach in the game:** a guided first minute inside the demo, with nothing written beside it.
- **Five minutes:** he decides in about five minutes, so the core has to show up in the first minute.
- **Neutral:** describe, don't sell. No "you'll love this".
- **Feedback:** keep / change / drop controls in the demo, notes while he plays, then talking it through.
- **Input:** ask each session, and don't carry an old answer over as if he'd just said it. On 2026-11-26 he wanted keyboard-only.

## Step 1: Read before asking

Most of what you need is already written down. Gather it first, so you only ask him what's missing.

- **His taste:** the list of games he loves and dislikes is in `~/.claude/CLAUDE.md` (usually already in your context) and in project memory. Name which of his favourites each idea is close to; it keeps you honest about fit.
- **The albums:** read `references/music.md`. Each album's world and mood come from Henry first. After him, use the thematic reads in `music_listening_notes.md` at the repo root, which covers five albums. It's AI-written and gets some track names wrong. The tracks are in the zip in `~/Downloads`.
- **The request:** what is the game for (all 43 tracks, one album, one track, something unrelated to music)? How long is a sitting? What input mode?

Then ask what's missing in one short message, at most three questions. If he can't be asked (you're running unattended or in a test), write each question down with the assumption you made instead, and carry on.

## Step 2: Spread wide, then narrow

Write about eight one-line seeds from different starting points. `references/pitches.md` has the recipe:
- from an action he loves (commanding, building, steering, discovering, collecting)
- from a form he loves (a city builder, a roguelike, a pinball table, a fleet battle)
- from a world, the setting and mood of one album
- at least one far from anything on his list

Then turn the most promising four or five into concept pages and check them (step 3). Keep two or three that pass, mixing near and far.

The spread matters because the first idea that survives scrutiny is rarely the best one. A single concept also invites patching instead of replacing.

## Step 3: Check every concept

For each concept, write a short page for yourself. Henry doesn't see it unless he asks. It holds:
- the player's sentence ("I'm a … trying to … while …")
- the main action and the core trade-off
- the pace, and where pressure comes from
- which album dresses it, and how that album's world and mood show
- the first 60 seconds, second by second
- how many rules come before the first fun moment
- which of his favourites it's closest to

Then run the checks in `references/checks.md`:
1. pace fit
2. theme fit with the music
3. the player's sentence
4. the rule budget and explain-back
5. taste fit
6. demo scope

Give each check one line: pass or fail, and why. Fix or drop a concept that fails. Don't pitch it with a caveat, because Henry reads pitches, not caveats.

## Step 4: Get an outside view

Start a fresh subagent with the brief in Henry's words, his taste list, and the concept pages. Don't give it your check results or your reasoning; its value is that it doesn't share them. Use the review brief in `references/agent-briefs.md`. It asks for two things:
- the strongest objection to each concept
- one alternative that would beat all of them

If the alternative is better, check it and swap it in.

Reviewers handed a single design tend to patch it. Reviewers asked to beat a set will say "rethink" when they should. If you can't start a subagent, do a separate pass yourself that reads only those files, and say in your notes that you did.

## Step 5: Pitch

Send Henry the pitches and nothing else. `references/pitches.md` has the format:
- two or three numbered lines
- each one sentence, in the player's words, saying what you do and in which album's world
- a last line asking which one he wants, if any

Then stop and wait. Don't build anything before he picks; his pick is the cheapest feedback you'll ever get.

## Step 6: Build the demo

Build to the bar in `references/demo.md`:
- his real music as the soundtrack, DOS-style pixel art, and quiet sound effects
- a guided first minute, with the core in that minute
- about five minutes of play
- keep / change / drop and note keys that send his feedback to a file

Reuse the arcade's existing browser stack rather than starting over.

Don't send progress reports while building. If you hit a decision only he can make, ask it in one line, then go quiet again.

## Step 7: Test it on a fresh player first

Before Henry sees it, start a subagent with no context to play the demo, using the playtest brief in `references/agent-briefs.md`. It plays for a few minutes in the browser, then writes:
- what it thinks the goal and the rules are
- where it waited
- where it felt rushed
- what it would do next

Compare that with what you intended. Fix the guided minute where they differ, and run it once more. You can't judge your own tutorial, because you already know the rules.

## Step 8: Hand it over, then listen

Give Henry one line: how to start it and the keys. Then wait.

When he answers, read the feedback file and his words. Record the verdict and the reason in project memory, so the next design starts from it. Then act on it:

- **Keep:** deepen it. Now is the time for balance bots, more levels and more art. Bots tell you whether one strategy always wins; they never tell you whether it's fun.
  - For genres that live on detail, he may want design documents first, as he did for Naibihpma Hold. Write them as a short series in the demo's `design/` folder. Send one at a time and wait for his review before the next. End each with the questions only he can answer.
  - Check each document against the game he names as the bar, using its real systems rather than your memory of them, and get an outside review of it before he sees it.
  - That game's spirit is in its small details: what each unit does by itself, how it grows, and how units and buildings feed each other (in Majesty, rogues poison their crossbows with plants the cultists grow). An inventory of its systems misses this, and he'll notice. Research it deeply, from the manual and fan guides, without a tight word limit, and design your own details at the same grain. Match its density of detail, not its list: a detail that works there because of its genre's tropes needn't be copied.
  - Read his own prototypes if he points to them, and use what they had. Their systems are a main source of those details.
- **Change:** make the change he asked for, one at a time, and hand it back. After three rounds of changes on one concept, ask whether to keep going or drop it.
- **Drop:** go back to step 2 with what you learned. Don't re-pitch near-copies of the dropped idea.
