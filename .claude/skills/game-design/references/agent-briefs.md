# Briefs for fresh agents

Both agents earn their keep by not knowing what you know. Give them only what is listed, and never your reasoning, your check results, or what you expect them to find. If you tell a reviewer the hypothesis, it tends to come back confirmed.

## Concept review (step 4)

Give it these files: the brief in Henry's own words, his taste list (loves and dislikes), and the concept pages. Then:

> You're advising a musician who wants a game built on his albums. Read the brief (his words), his list of favourite and disliked games, and the concept pages. For each concept, give the strongest objection a player would have after five minutes, not a designer's nitpick. Then propose one concept, different from all of these, that you believe would beat them, in one sentence plus three lines on why. He doesn't want rules that depend on hearing the music; his albums supply each game's world and mood, and the tracks are the soundtrack. Be candid: if none of these should be built, say so. Up to 600 words.

What to do with the answer:
- If an objection lands, the concept fails a check you passed too easily. Fix it or drop it.
- If the alternative is better, write its concept page, run the checks, and swap it in.

## Fresh-player test (step 7)

Give it: how to start the demo (URL or command), the keys, and nothing about the rules or the goal. Then:

> Play this game for about three minutes, as a newcomer would. Use the in-game teaching; nothing else will explain it. Take screenshots as you go. Then write:
> (1) What is the goal? How do you know if you're doing well?
> (2) What are the rules, as you understand them?
> (3) Where did you wait with nothing to do?
> (4) Where did you feel rushed or confused?
> (5) What kind of place did this feel like, and did the music fit it?
> (6) Would you keep playing? Why or why not?
> Answer from what you saw, not from guesses about what was intended.

Compare its answers with the intended goal and rules from your concept page.
- Every mismatch is a failure of the guided first minute. Fix the teaching rather than the player.
- Answer (5) checks whether the theme fit survived the build.
- Run the test once more after fixing.

If the built-in browser pane is hidden, the page gets no animation frames. Tell the agent to let time pass with the debug stepper, for example: "run `window.hdStep(15)` to advance one second; after every action advance 15 to 90 frames, then take a screenshot". Also ask it to list anything that looked broken: cut-off text, overlaps, buttons that didn't answer, sprites it couldn't make out. The first run of Naibihpma Hold found most of its interface problems this way.

If the agent can't drive a browser, give it the guided-minute script and a few screenshots instead, and treat the result as weaker evidence.
