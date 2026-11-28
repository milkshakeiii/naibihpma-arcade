# Majesty: The Fantasy Kingdom Sim (Cyberlore, 2000) and The Northern Expansion: the heroes

Research notes for a designer using Majesty as a minimum bar.

## Source key and caveats

- **(manual)** the Majesty Gold HD manual (Steam PDF, text extracted locally). It is thin on behaviour: one short paragraph per hero. Its building reference is useful.
- **(prima)** the 2000 Prima Official eGuide for Majesty, read as OCR text on archive.org. It is a publisher guide, not a fan page, but I read it through an automated summariser in 100k-character chunks, so treat exact numbers as "as reported". It contains the behaviour percentages, special abilities and building skills that make up most of the detail below.
- **(cyberlore)** the original Cyberlore web pages ("World of Ardania", chapters 1 to 5).
- **(fan)** Steam Community threads and guides, a walkthrough site, and snippets of the Majesty fandom wiki. The fandom wiki itself returned HTTP 402 to every fetch, and GameFAQs returned 403, so wiki facts are second-hand from search-result summaries. Marked (fan) and often (unsure).
- **(unsure)** I could not confirm it in a source, or sources disagree.

Where the numbers are Majesty Gold (the base game plus the Northern Expansion), I say so only when I know.

**How to read the "behaviour percentages."** The Prima guide gives each hero a ranked list of things it might do, each with a percentage. The guide says the number is the *chance of considering that task*, not of completing it. Heroes work down their ordered list until one task is actually chosen, and they re-think periodically or when something happens nearby. This is the real skeleton of the game's "mind of their own" feel, so I quote the percentages for each hero. (prima)

---

## 1. Cross-cutting rules (read these first)

### 1.1 What a hero is
- Heroes are recruited, not commanded. You click Recruit on a guild or temple, pay gold, and after a timer a named hero with its own randomised stats walks out and gets on with its own life. Each building supports a fixed number (usually four; Dwarven Settlement three; Elven Bungalow two; Gnome Hovel three; Temple to Krolm six per the manual). (manual)
- New heroes vary: Artifice, Intelligence, Strength, Vitality and Willpower each roll from 2 below to 3 above the class value. Every hero starts with 50 to 99 gold of its own. (prima)
- The manual's own example of autonomy: no bounty you place on a Skeleton will make a Ranger attack it, because Rangers know they will barely hit something that dodges so well. (manual) The Prima guide agrees: a big enough reward "may still tempt him". (prima)

### 1.2 The stats that drive behaviour
- **Greed**: how much a hero values gold. Very greedy heroes may attack buildings or friendly units for a reward. Distance and danger have to be outweighed by the flag. (manual, prima)
- **Willpower** ("piety"): decides whether the hero indulges in "sinful" activities, namely the Elven Lounge and Gambling Hall. (manual, prima)
- **Loyalty**: low loyalty lets a hero attack *your own* building if you (or an accident) put an Attack Flag on it. High loyalty keeps a hero waiting at an Inn after its guild is destroyed, rather than leaving. Statues (+5 Loyalty each) and Royal Gardens (+15 each) raise it. In multiplayer, loyalty decides whether a hero answers an opponent's flags or spends money in their buildings. (manual, prima)
- **Artifice**: stealth and craftiness; used for stealing and gambling. **Luck**: helps gambling and chance events. (prima)
- **Intelligence**: spell damage bonus, and whether a hero can learn generic spells at the Library. (prima)
- **Sight Range**: reveals map and lets the hero react to nearby dangers; for casters it is also casting range. (prima)

### 1.3 Reward flags
- Two flags: **Explore** (anywhere, stays put) and **Attack** (only on a visible monster, building, lair or hero; follows moving targets). The reward is paid from your treasury up front. You can raise a placed flag's bounty by 100 per click, never lower it. Removing a flag destroys its gold. (manual)
- A flag shows a count of heroes currently responding. There is a short lag between placing a flag and heroes noticing it. (manual)
- When the objective is met, everyone nearby is assumed to have taken part and the reward is split. (manual)
- Heroes do not always respond. "Each hero's thirst for gold is different," and the reward must exceed the hardship of distance and danger. (manual)
- Fan rules of thumb: rogues and elves bite on about 100 to 200 gold for nearby targets; Solarii and dwarves want about 200+; paladins respond to ~1000 on a wight; explore flags cost 100 to 500 near home and 1000+ far away. (fan) Heroes rarely walk far for a bounty but will happily take one on a building they are already standing next to. (fan)
- A flag on a monster the hero cannot realistically beat is ignored, not suicidally pursued, except by "berserk" types. (manual example, prima)

### 1.4 Gold: earning, carrying, paying tax, spending
- Heroes earn gold from flags, from killing monsters, from looting lair treasure and from stealing. They carry it. When they go home to the guild, a share is dropped into the guild's coffers, and your Tax Collector later collects it. Buildings in the Prima guide carry an "Income" percentage, i.e. the share of a hero's visit gold the building passes to the coffers (Rogues Guild 30 percent, Warriors Guild 40, Temple to Agrela 50, Temple to Dauros 45, etc.). (prima)
- The Rogues Guild "returns a smaller percentage of its income to the Palace than others". (manual)
- Elf, Dwarf and Gnome homes have no coffers and are **untaxable**: everything those races earn is lost to you. (prima, fan)
- Fan economics (from a Steam guide that Steam later removed, so treat as (unsure)): heroes keep "unreported" gold that is untaxed until they go home; on entering the home guild about 60 percent is "reported" and 40 percent goes to the coffer; shopping returns about 40 percent of the price as tax; heroes spend taxed money first. (fan, unsure)
- Gold also leaks to Ratmen and to Rogues-on-corpses: a hero who dies carrying thousands of gold loses it (fan: one player saw heroes carrying 10,000+). Gold from a dead Tax Collector vanishes unless a hero is nearby to scoop it up. (manual, fan)

### 1.5 Where heroes spend (and what each costs)
| Place | What heroes buy | Notes |
|---|---|---|
| Blacksmith | +1/+2/+3 weapons and armour | Heroes pay about 100/200/300 each tier, after you research the tier (manual, prima) |
| Marketplace | Healing Potions (25 gold, heal 30 HP, carry five), Rings of Protection (level 2), Amulets of Teleportation (level 3) | Potions need research; Market Day shuts the market and stops sales (manual, prima) |
| Wizards Guild | Weapon and armour enchantments +1/+2/+3 (about 200/400/800) | Only to the level of the guild (manual, prima) |
| Rogues Guild (level 2) | Poisoned weapon: free, 1 HP/sec for 10 seconds, resets on each hit, blocks healing | Chance depends on Artifice; poison is permanent on the weapon (prima) |
| Library | Magic resistance (300 per 5 points up to 50), Wizard-only Intelligence training, generic spells Power Shock and Flame Shield (300 each, need Intelligence 11) | (prima) |
| Fairgrounds | Combat-stat boosts (about 400 gold), tournaments | Level 3 Palace needed (manual, prima) |
| Magic Bazaar (Northern) | Tonic of Speed, Fire Balm, Dirgo Strength, Elixir of Regeneration, Invisibility Brew, Shapeshift Tincture | (manual) |
| Inn | 12 gold for a full heal | Gazebo (appears with Royal Gardens) is the free version (prima) |
| Elven Lounge | about 200 gold per visit, untaxable | Low-willpower heroes only (manual, prima) |
| Gambling Hall | Low-willpower heroes bet 1 to 10 gold per go | Needs Elves plus a Rogues Guild (manual, prima) |

### 1.6 Levelling
- XP is divided by the hero's current level, so each level takes longer. On level-up, HP rises by Vitality/4 plus a random roll; even levels raise the primary attribute by 1; Dodge and Parry rise by 1 per level; melee heroes gain Hand to Hand, ranged heroes gain Ranged Attack. (prima)
- XP sources (prima): attacking a lair = 75 x lair difficulty; killing a monster = its value divided by (hero level / 2); killing an enemy hero = 150 x its level; building/repairing = 75 per hammer swing; casting a spell = 100 x caster level; healer casting = 300 per healer level; charmed monster exploring = 200, charmed monster swinging in combat = 150; exploring new area = 250; planting or harvesting = 50; opening a chest = 50; stealing = 200 per attempt; garrisoning a Guard House = 250; entertaining = 175; Ranger journey = 3000; Library visit = 50 (100 for Wizards).
- Fairgrounds tournaments: entry fee 10 gold, winner gets 10 x participants and 1000/rank XP per combatant; with more than four combatants the winner gains a level outright. Up to eight heroes at once. You choose melee, archery, magic or combo tournaments to attract particular classes, or close them to push heroes out into the field. (manual, prima) A fan reports Wizards at level 50+ after a half-hour game on the Fairgrounds. (fan)
- Naming: heroes can be renamed by the player from level 10. (manual)
- The Palace needs heroes to upgrade (four for level 2, twelve for level 3); Gnomes do not count. This is why cheap fast Rogues are trained for Palace upgrades. (prima, fan)

### 1.7 Resting, healing, fleeing, dying
- Heroes heal in the field only via spells and potions. They rest at **Inns** (12 gold, full heal), the free **Gazebo**, their own guild, or a **Royal Gardens**. Inns on the frontier extend how far heroes wander; Fairgrounds, Lounges and Gambling Halls keep them near home. (manual, prima)
- Heroes flee when hurt or outmatched. The game lists "fleeing in terror" and "fleeing from injuries" as distinct behaviours, and a fan notes low-level heroes tend to run, and slow ones (Warriors) die while fleeing. The Temple to Krolm makes new heroes somewhat less likely to retreat. (prima, fan)
- Death leaves a **Gravestone**. Higher-level heroes' stones last longer (about a sixth of a day per three levels, plus a sixth of a day). Resurrection (Agrela, level 3, 1500 gold) or Re-Animate (Krypta, level 3, 2000 gold) can revive a hero from a stone; once the stone is gone, the hero is gone. Gravestones count against the guild's slots; recruiting a replacement dismisses the lowest-level stone. (manual, prima)
- Graveyards: a Graveyard appears in your town after a number of hero deaths (about 15) and spawns undead. The Northern Expansion's **Mausoleum** stores dead heroes, can bring them back later at a cost scaling with level, and does not spawn undead. Agrela and Dauros heroes never go to the Mausoleum. (manual, prima)
- Heroes whose guild is destroyed walk to the Palace and leave unless a new building is built first, or they sulk at an Inn if loyal. The Northern Embassy and Outpost can house homeless heroes (two and six). (manual, prima)
- The Embassy recruits veteran heroes at random intervals into empty slots, for gold. (manual)

### 1.8 Parties and "friendships"
There is **no party system**. What there is, per hero, is a "follow and support" task with a percentage. The Prima guide lists these:
- Healers follow and heal Warriors (75), Tax Collectors (50), Wizards (35).
- Rangers follow and support Barbarians (90).
- Wizards follow Barbarians (30) and Monks (20).
- Monks guard Healers (95) and Caravans (75).
- Priestesses are trailed by their own Skeletons; Cultists by charmed animals.
- Warriors defend Agrela's temples and Healers (prima flavour text); Paladins guard the Dauros temple.
- Hero dislikes are expressed at the building level (see 1.9) and in flavour text: Dwarves dislike Elves and Gnomes, Elves dislike Dwarves and Gnomes, other races shun towns that host Gnomes, Adepts and Solarii don't get along.

### 1.9 Building exclusions (who can live together)
- Temples to Agrela, Dauros, Krypta, Fervus and Krolm are mutually exclusive in pairs: Agrela and Dauros cannot coexist with Fervus, Krypta or Krolm; Fervus cannot with Agrela, Dauros or Krolm; Krypta cannot with Agrela, Dauros or Krolm; Krolm cannot with any other god's temple; Helia cannot with Lunord or Krolm. (manual)
- Dwarves, Elves and Gnomes are mutually exclusive. (manual)
- Paladins need a Temple to Dauros plus a Warriors Guild; Warriors of Discord need a Temple to Fervus plus a Warriors Guild. A Warriors Guild shares four slots among all three. (manual)

### 1.10 Library, in one place
- Library requires a Wizards Guild; level 2 needs level 1's research done. Just having one raises Intelligence of heroes recruited afterward and cheapens research. (manual)
- Level 1: Fire Blast (Wizards, free; the manual calls it "researched" by you), Magic Resistance training (any hero), Intelligence training (Wizards). Level 2: Meteor Storm (Wizards, free), Power Shock and Flame Shield (heroes with Intelligence 11+, 300 gold each). (manual, prima)
- Fan view: non-Wizard Library spells are usually wasted money; the AI buys them anyway. Monks with Flame Shield and Power Shock are praised. (fan)

---

## 2. Warrior (Warriors Guild)

- **Where / cost:** Warriors Guild, no prerequisite, 450 gold per Warrior; the guild holds four across Warriors, Paladins and Warriors of Discord. (manual, prima)
- **Personality:** Chivalrous all-rounder. Simple fighters; high HP, big HP gains per level, low Intelligence, mediocre Artifice. Described as "sturdy, brave, and often sleepy". (manual, cyberlore, prima)
- **Unprompted:** Raids lairs (95), fights wandering monsters (95) and enemy heroes (95), raids enemy buildings (90), visits Inns (50), explores (45), collects items (20), Royal Gardens (20), goes home (90). (prima) In other words: Warriors are the cheerful lair-bashers who nap at the inn between fights.
- **Spells / growth:** None. Plate mail and a sword, Dodge 35, Parry 45. They are smart enough to buy gear. (prima, fan)
- **Spending:** Blacksmith armour and weapons, potions at the Marketplace, Inn. (fan)
- **Guild ability: Call to Arms** (500 gold): teleports all Warriors, Paladins and Warriors of Discord back to the guild and makes them berserk. Players use it to defend a building next to the guild or pull paladins back before a Gate spell; it costs gold so cannot be spammed. (prima, fan)
- **Interactions:** Healers follow them (75). They guard Agrela temples and Healers. Warrior plus Healer is the standard Vampire counter. A Warriors Guild itself has 700-800 HP, so players use it as a sturdy perimeter building. (prima, fan)
- **Weaknesses:** Slow; they die when fleeing. Warriors are poor against Minotaurs and Hooligans in quests where they die before closing, and against magic-heavy monsters. (fan)
- **Player view:** Early-game workhorse; later heroes eclipse them unless they survive to high level. A level 15 Warrior can solo a golem or dragon (fan). Fans list them low on tier lists but note healer synergy. (prima, fan)

## 3. Ranger (Rangers Guild)

- **Where / cost:** Rangers Guild (700 gold, no prerequisite; Palace level 1). 350 gold per Ranger, four per guild. (manual, prima)
- **Personality:** "Confident loners" who have sworn (Treaty of Ravenswood) to support Barbarians. The manual says they explore, fight wandering monsters, and after the realm is mapped "sometimes journey to distant lands, disappearing off the edge of the map and appearing again later, if they have survived". (manual, prima)
- **Unprompted:** Collects healing herbs (95), explores (95), follows and supports Barbarians (90), fights wandering monsters (75) and enemy heroes (55), journeys to distant lands (30), Royal Gardens (30), goes home (15). They rarely go home. (prima)
- **Special abilities:**
  - *Harvesting*: five healing plants become one Healing Potion; Rangers can carry more than the usual five potions. The plants are the ones Healers plant. (prima)
  - *Journeying*: leaves the map, returns after one and a half days with 3000 XP and 1 to 700 gold. Fans measured about 500 gold and around one level at low levels. (prima, fan)
  - *Growth*: every level adds 10 to both Sight and Range Attack, up to 550; no other hero scales like this. Loot sharing reaches twice as far as other heroes'. (prima, fan)
- **Spending:** Little; they are the least greedy human class (Greed 3). Cheap to keep, spend on bow upgrades. (prima)
- **Interactions:** Support Barbarians (and so are distracted from exploring if barbarians are around; walkthrough notes it reduces exploration). Skeletons are their nemesis: a Skeleton's high Dodge makes Rangers rarely hit, so they ignore them. Poison from a level-2 Rogues Guild is a good match for missile weapons. (prima, fan)
- **Guild ability: Move Camp** (800 gold): instantly relocates the whole Rangers Guild, even under attack, to an explored location. Occupants leave first. Used to follow unexplored land or flee. (manual, prima)
- **Weaknesses:** 14 HP at start; fragile when they stop to fight at close range. A fan notes Rangers defend their guild and die with it if it falls. (prima, fan)
- **Player view:** Among the best early heroes: they explore for free, expand the kingdom and unlock Palace upgrades. Less useful after early missions, but because exploration gives XP some reach Minotaur-fighting strength, and fans call them more reliable than elves or rogues. Best advice: build four early, flag only what you must. (fan, prima)

## 4. Rogue (Rogues Guild)

- **Where / cost:** Rogues Guild (600 gold, then 850 for level 2). 275 gold each, four per guild; cheapest and fastest to recruit, which makes them a quick path to Palace upgrades. (prima)
- **Personality:** "Secretive, ruthless, and the least loyal heroes" (Loyalty 0, Willpower 3, Greed 26, highest Artifice of any class, high luck, good dodge and parry). Cyberlore: "lockpickers, poison-makers, and greedy". Crossbow, leather armour, no chain or plate so they stay quiet and fast. (manual, cyberlore, prima)
- **Unprompted:** Loots gravestones (100), steals (95), collects special items (90), harvests poison plants (90), fights wandering monsters (65) and heroes (55), raids lairs (35), visits Inns (35), Royal Gardens (30), explores (20), goes home (95). (prima)
- **What the small behaviours are:**
  - *Grave robbing*: they hang near heroes who might die and steal gold from the gravestone before it vanishes, for 200 XP. (prima)
  - *Stealing*: from lairs and buildings. Each attempt takes 1 to 100 gold after a check against Artifice (1-30), can also try for a weapon or armour from a lair (second check, 5 percent per lair difficulty for weapons, 4 for armour); a failed theft halves the lair's gold; every attempt gives 200 XP. With very low loyalty they will steal from your own buildings. Cultists and Elves can steal too, but less often. (prima)
  - *Harvesting the Cultists' poison plants*: **Rogues harvest the poisonous plants that Cultists plant**; five plants make one dose of weapon poison. (manual, prima) 50 XP per plant.
  - *Poisoned crossbows*: with a level 2 Rogues Guild, any hero can get a free poison coating (1 HP/sec for 10 seconds, stacks on each hit, blocks healing). The Prima guide says poison "decides tough quests". Skeletons and Rock Golems are not exempt. (prima, fan)
- **Spending:** Mostly nothing; they hoard. Fans say Rogues profit at the Gambling Hall and Warriors don't. Rogues keep more of their gold than other humans (the guild returns a smaller share to you). (prima, fan, unsure)
- **Reward flags:** Extremely greedy, so even 100-gold Explore Flags send them out, and they accept any Attack Flag (including one on your own building if you have no Statues; about three Statues stop this). Small Explore Flags are used to reveal the map near a Trading Post. Rogues swarm Vampires if there is a flag. (prima, fan)
- **Guild ability: Extortion**: you press it and it instantly takes 20 percent of carried hero gold and all gold in buildings (and caravans, which forfeits the Marketplace arrival multiplier). Rogues keep 35 percent. It is recommended every six or seven days to sweep Housing, as an emergency tool. (manual, prima, fan)
- **Interactions:** Rogues will tear down Elven Lounges and Gambling Halls (nobody else will; Wizards at level 2 can with Lightning Bolt). The Gambling Hall only appears if both a Rogues Guild and Elves exist. Rats can steal from the guild's hoard. (fan, manual)
- **Weaknesses:** 11 HP, no armour, low damage (the crossbow does not scale with Strength). Die often. (prima, fan)
- **Player view:** "The Rogues Guild is valuable; the hero is not": poison, Extortion, cheap Palace-upgrade fodder, and looting corpses forces fights that level them. Great scouts for small flags. (fan)

## 5. Wizard (Wizards Guild)

- **Where / cost:** Wizards Guild (Palace level 2; 1500 / 2500 / 3500 gold). 500 gold each, four per guild. Also needs a Library to grow. (manual, prima)
- **Personality:** Frail veterans, "often distracted and detached from daily life", cautious, high Intelligence, low Strength. Spellcasters only, no armour. (manual, cyberlore, prima)
- **HP:** About 4 at level 1. Players say a rat can kill one; it is the fragile glass cannon of the game. (prima, fan)
- **Spells by level (prima, fan):** Energy Blast (level 1, 10 damage), Fire Shield (2), **Fire Blast (3, learned at the Library, 20 damage to up to three enemies)**, Teleport (4, range 1400, an escape), Fire Ball (5), Resist Magic (6), **Meteor Storm (7, learned at the Library, 5 to 25 damage to everything within 175 for a third of a day)**. So **Wizards learn Fire Blast and Meteor Storm at the Library as they level**, free for them, no Intelligence roll. (prima, fan)
- **Unprompted:** Learns new spells at the Library (90), trains Intelligence (55), Magic Resistance (15); follows Barbarians (30) and Monks (20); raids lairs (45), enemy buildings (30), fights monsters (65) and heroes (60), explores (50), goes home (85). (prima)
- **Spending:** Wizards do not buy equipment upgrades; casters in general don't. They pay the Library. (fan)
- **Guild abilities and what the guild gives you:** The guild gives you two Sovereign spells per level: Farseeing and Invisibility (L1), Lightning Bolt and Anti-Magic Shield (L2), Lightning Storm and Supercharge (L3). Spell range is limited and extended by Wizards Towers (500 gold each, within 800 of another tower or guild). A level-2 guild can Enchant towers (400 gold, 1.5 days) to fire bolts. Enchantments for heroes' weapons are sold here. (manual, prima)
- **Interactions:** Healers follow them (35). Monks and Barbarians are favourite escorts. Dangerous foes: Medusae, Dryads, Evil Oculi, Vampires (reflect spells), Magic Mirror monsters. Wizards die to Minotaurs in few blows. (prima, fan)
- **Typical deaths:** Running ahead of escorts and stumbling into something; having a guild raided; reanimated too soon next to the thing that killed them. (fan)
- **Tactics:** Invisibility on them so they level safely; a Guard House or Wizards Tower beside the guild; hire one at a time and keep the camera on it; Fairgrounds to level fast; research Fire Blast first. Don't resurrect a wizard until the killer has left. They sometimes berserk for no clear reason. (fan, prima)
- **Player view:** Weak early, "demigod" by level 7+ if babysat; a perennial argument ("Wizards are garbage" thread) about the fragility. (fan)

## 6. Healer (Temple to Agrela)

- **Where / cost:** Temple to Agrela (Palace level 2; 1000 / 1500 / 2600). 300 gold, four per temple. Cannot coexist with Fervus, Krypta or Krolm temples. (manual, prima)
- **Personality:** "Compassionate and self-sacrificing, and therefore terrible at fighting." Higher than average Willpower, no armour, dagger only. (manual)
- **Unprompted:** Heals others (95), follows and heals Warriors (75), the Tax Collector (50), Wizards (35); plants healing plants (50), Royal Gardens (50), Inns (20), goes home (90). They fight only when attacked. (prima)
- **Special abilities:** *Planting*: healing herbs, 50 XP each, which Rangers harvest. *Rebirth*: once per level, a killed Healer returns home as an ethereal unit and revives itself. Spells: Heal at level 1 (8 HP per caster level), Meditation (4), Aura of Peace (7). Heal casting gives 300 XP per Healer level, so they level fast near fighters. (prima)
- **Spending:** Greed 1. Almost nothing. (prima)
- **Temple spells you cast:** Healing 200 gold (heals 100), Blessing 100, **Resurrection 1500**. (prima)
- **Interactions:** They heal heroes from other guilds and aligned temples. They follow Warriors and Wizards, and Monks guard them. Fan: they only heal when someone passes by ("check_nearby"), do not seek out the wounded, and following the Tax Collector or a Wizard pulls them away from the fight. In multiplayer, low loyalty can make them heal *enemy* heroes. (prima, fan)
- **Weaknesses:** 7 HP. A fan notes that when their guildhall is attacked they charge at the attackers and die, hence Resurrection. (fan)
- **Player view:** Ideal support in theory; reality is luck-dependent. Healer plus Warrior is the Vampire answer. (fan)

## 7. Priestess (Temple to Krypta)

- **Where / cost:** Temple to Krypta (Palace level 2; 1400 / 1800 / 2200). 400 gold, four per temple. Excludes Agrela, Dauros and Krolm. (manual, prima)
- **Personality:** "Sinister necromancers"; poor fighters, lower Willpower, fairly high Intelligence; staff, no armour. Slow. (manual, prima)
- **Special abilities:** Spells Drain Life and Animate Skeleton (level 1), **Control Undead (level 7)**. **A Priestess of Krypta can take control of enemy undead**: charms Skeletons, Zombies, Vampires and (reportedly) Shadow Beasts. Fan: vampires are charmed at about level 10 (unsure). Skeleton count is one per two levels, capped at three. She can only charm if she has fewer than three followers, so the sanctioned trick is to kill one of your own skeletons with an Attack Flag to open a slot for a Vampire. (prima, fan)
- **Unprompted:** Creates Skeleton followers (95), fights monsters (75) and enemy heroes (70), learns spells (70), explores (50), raids lairs (40), enemy buildings (30), goes home (80). Skeletons earn her XP as they fight and explore. (prima)
- **Interactions:** Skeletons guard her guild, so put it next to a Marketplace for cheap defence. Elven Lounges "keep priestesses at home" (fan walkthrough). They tend to wander ahead of their own skeletons (annoyance). Skeletons dodge very well so they are good against dragons, and Wither (Sovereign, 300) slows dragons. (prima, fan)
- **Temple spells:** Wither (300), Animate Bones (800, creates a Skeleton follower), Re-Animate (2000, revives a gravestone's unit, cost higher than Agrela's). (prima)
- **Weaknesses:** Weak alone; skeletons are flattened by Yetis and Rock Golems; slow walkers. (fan)
- **Player view:** "A Priestess with enslaved vampires is the best hero in the game." Level 10 priestess kills a dragon alone (anecdote). Essential on fire-dragon maps. (fan)

## 8. Cultist (Temple to Fervus)

- **Where / cost:** Temple to Fervus (Palace level 2; 900 / 1300 / 2500). 400 gold, four per temple. Excludes Agrela, Dauros and Krolm. Also raises Luck of heroes recruited afterward and gives Rogues +3 Artifice. (manual, prima)
- **Personality:** "Clever, carefree, spontaneous" followers of the god of chaos. Two daggers in melee and throw knives. Immune to poison. Very low HP (11), low damage. (manual, prima)
- **Special abilities:** **Cultists sow the poisonous plants that Rogues harvest** (planting, 50 XP). Spells: Charm Monster (level 1), Camouflage (4, an escape only), Change Shape (7, becomes a Hell Bear with big Strength/Vitality bonuses). They charm animals (Varg, Roc, Rats, Hell Bears) and can keep up to three; at higher levels Werewolves and Medusae. (manual, prima, fan)
- **Unprompted:** Charms animals (95), swarm attacks with other Cultists (90), fights monsters and heroes (85), plants poison plants (55), Royal Gardens (50), steals (25), raids lairs (20). They also "patrol creature dens" (fan, unsure). (prima, fan)
- **Temple behaviour:** The temple spawns animals for Cultists to charm: Rocs at level 1, Vargs at 2, Hell Bears at 3. (prima)
- **Temple spells:** Healing 400, Illusionary Hero 500 (decoy that loses HP), Vines 1000 (freeze). (prima)
- **Player view:** Liked but weak; their power depends on the map (Medusae, werewolves, wendigos). "Place them near a Medusa lair." Often described as situational. (fan, prima)

## 9. Paladin (Warriors Guild, needs Temple to Dauros)

- **Where / cost:** Warriors Guild plus Temple to Dauros. 1000 gold, very expensive. (manual, prima)
- **Personality:** "Painfully self-righteous", excellent warriors, high willpower (23), plate armour, two-handed sword, one protective spell. (manual, prima)
- **Special ability:** Shield of Light at level 4 (+10 Dodge, +10 Parry, +15 resist, +4 armour for a quarter of a day). With it a Paladin can fight a Vampire or Werewolf alone. Harm Undead (level 8, fan, unsure). (prima, fan)
- **Unprompted:** Fights wandering monsters (90), raids lairs (85) and enemy buildings (85), fights heroes (80), goes home (80), trains Magic Resistance (25), Royal Gardens (10). Paladins attack lairs unprompted, which makes coordinated attacks hard. (prima, fan)
- **Guard duty:** They guard the Dauros temple when attacked. (prima)
- **Weaknesses:** Weak magic resistance (5) and high cost; losing a low-level one hurts. Fan: the heavy armour means the Blacksmith matters little to them, they are slow. (prima, fan)
- **Player view:** Strongest class by many accounts, or "bland and risky if luck is poor"; the core force in many walkthroughs. (fan)

## 10. Monk (Temple to Dauros)

- **Where / cost:** Temple to Dauros (Palace level 2; 1600 / 2200 / 3400). 550 gold, four per temple. (manual, prima)
- **Personality:** Very high willpower, "will almost never go into Gambling Halls or Elven Lounges". Cautious. Open hand, no weapons or armour, vow of silence. Slow. (manual, prima)
- **Spells:** Hands of Steel (1), Stone Skin (4), Iron Will (7; +55 resist, faster healing, 6 percent critical hit). After Iron Will they are nearly invulnerable. They mostly survive on their own after level 4. (prima)
- **Unprompted:** Guards Healers (95), collects items (70), guards Caravans (75), fights monsters (75), raids lairs (50), Inns (50), trains Magic Resistance (45), learns spells (55), goes home (60). Fans say they favour Inns, Royal Gardens and Gazebos when idle. (prima, fan)
- **Temple spells:** Stone Skin 200, Vigilance 800, Petrify 1500. (prima)
- **Interactions:** Escort Healers and Caravans; Wizards like to travel with them. (prima)
- **Player view:** Snowball unit that struggles at levels 1 to 6, then becomes very hard to kill. Fans note they ignore flags, hardly buy items, and sometimes flee fights they would win. Slow, so fleeing often means death. (fan)

## 11. Solarus (Temple to Helia)

- **Where / cost:** Temple to Helia (Palace level 3; 1000 / 2000). 500 gold, four per temple. Excludes Lunord and Krolm. (manual, prima)
- **Personality:** Women warriors of the sun; a huge crushing mace, chain mail, strong, fire spell. Average speed. (manual, prima)
- **Special behaviour:** **Solarii raid lairs by themselves and can sense hidden lairs from afar.** This is the "useful lair-raiding behaviour". Prima gives "finds hidden lairs and raids them" 95. Fans note that they destroy lairs unprompted, which can be a problem when a quest wants a lair left alone. (manual, prima, fan)
- **Unprompted:** Finds hidden lairs and raids them (95), explores (75, with half-hearted interest), checks for garrison duty (50), learns spells (50), Royal Gardens (15), goes home (5). They garrison Guard Houses (250 XP) so if you build Guard Houses near them they may sit in them instead of exploring. (prima, fan)
- **Spells:** Sun Scorch at level 4: hits all enemies within 65 for 15 plus 1 per level. The fan wiki lists a first-level fire spell (Fire Hammer, unsure) and says Sun Scorch also fires when a Solarus dies (unsure). (prima, fan)
- **Temple spells:** Fire Strike 350 (2-15 damage, units only, no range limit), Sun Scorch 1200. (prima)
- **Interactions:** Solarii and Adepts do not get on; the manual puts them in separate, exclusive temples. (prima, manual)
- **Player view:** Top of one Steam tier list: the only melee-and-magic hero, strong at about half a Paladin's cost. They need ~200 gold explore flags. Don't train them early in "Slay the Mighty Dragon" because they die. (fan)

## 12. Adept (Temple to Lunord)

- **Where / cost:** Temple to Lunord (Palace level 3; 1000 / 2000). 500 gold, four per temple. (manual, prima)
- **Personality:** Fleet of foot, spell and martial arts, light armour only, a staff. "Run like the wind". (manual)
- **Spell:** Teleport (Short) at level 4 (up to 500 range). (prima)
- **Unprompted:** Patrols the whole kingdom (95), goes home (95), fights monsters (60) and heroes (55), learns spells (40), trains Magic Resistance (25), Royal Gardens (20). (prima)
- **Role:** The kingdom's constant patrol: "graveyard and sewer incursions are rarely a worry while they're on duty." Adepts do not raid lairs or hunt without a flag. (prima, fan)
- **Temple spells:** Winged Feet 400 (extreme speed), Wind Storm 1000 (damage and scatter, useful against Elves and Strangleweed). (prima)
- **Player view:** Weak early, strong later; slow to level. Pair with Monks and Dwarves via Winged Feet. They can teleport to a burrow bypassing blockers in "Legendary Heroes". (fan)

## 13. Barbarian (Temple to Krolm)

- **Where / cost:** Temple to Krolm (Palace level 2; 900 gold; excludes every other god's temple). 350 gold, six per temple per the manual (prima says four). (manual, prima)
- **Personality:** "Live by the Law of Steel", very high HP, almost always berserk, two weapons (axe and club), good dodge and parry, avoid armour as "fear and weakness". (manual)
- **Unprompted:** Raids enemy buildings and lairs (95), fights monsters and heroes (95 each), explores (45), goes home (45), defends the Rangers Guild. Critical hit begins at 0 and gains 1 percent per level. (prima)
- **Guild ability: Rage of Krolm** (1500 gold): for nearly half a day all your heroes get strong boosts to Strength, melee, dodge, parry, HP and move/attack speed. Fans use it periodically in big fights; it requires a solid economy. (manual, prima, fan)
- **Interactions:** Rangers follow them; Wizards sometimes do. Barbarians are the only temple with no building spells, but other temples' Healing, Resurrection and Stone Skin work on them. (prima)
- **Weaknesses:** Don't buy potions; ranged poison attacks pick them apart; low armour; often die early. (fan)
- **Player view:** Cheap, expendable, mass-produced; strong in hordes against dragons and golems. (fan)

## 14. Dwarf (Dwarven Settlement)

- **Where / cost:** Dwarven Settlement (1250 gold; Palace level 2 and Blacksmith level 3; not with Elves or Gnomes). 500 gold, three per settlement. The settlement fires bolts automatically and lets you build **Ballista Towers**. (manual, prima)
- **Personality:** Stocky, strong, slow, master engineers. Hammer, chain mail, immune to poison, enormous magic resistance (70). (manual, prima)
- **Special ability: Building**: constructs and repairs (75 XP per swing). (prima)
- **Unprompted:** Helps build (85), raids lairs (75) and enemy buildings (80), goes home (80), Inn (20). Dwarves are greedy (Greed 20) and rarely go exploring. They deal extra damage to buildings and lairs. (prima, fan)
- **Player view:** Fans say they waste time building and repairing; slow, poor dodge. Dwarf settlement itself is the main draw: Ballista Towers are strong against dragons. Dwarves dislike Elves and Gnomes. (fan, prima)

## 15. Elf (Elven Bungalow)

- **Where / cost:** Elven Bungalow (750 gold; Palace level 2, Marketplace and Inn; excludes Dwarves and Gnomes). 500 gold, two per bungalow. (manual, prima)
- **Personality:** "Swift, slightly built, weakness for the pleasure-drenched lifestyle". Excellent longbow; chain mail not worn; greed 23; willpower about 6. (manual, fan)
- **Special abilities:**
  - *Entertaining*: plays at Inns (50), Trading Posts (35), Marketplaces (20): 30 gold and a free Inn stay (the inn gets nothing), or 25 gold at a Marketplace, 175 XP each. (manual, prima)
  - *Stealing*: 50 (prima).
  - *Elves gamble*: low willpower means they visit the Elven Lounge (200 gold per visit, untaxable) and the Gambling Hall. This is where their savings go. (manual, prima, fan)
- **Unprompted:** Collects special items (80), fights monsters and heroes (80), steals (50), Royal Gardens (45). (prima)
- **Bungalow effects:** Doubles Marketplace income (does not stack), causes an Elven Lounge to appear half a day later, and with a Rogues Guild causes a Gambling Hall. Both are untaxable and cannot be removed by you. (manual, prima)
- **Interactions:** Elves "bribe easily" with explore or attack flags. Guard Houses with arrows frighten them and Warriors scare them (the fear works both ways in one quest: Warriors flee Elves' fear, and Elves flee Warriors) (prima, fan, unsure). Medusae kill them. (fan)
- **Player view:** The main economic boost (Marketplace income doubled); weak fighters levels 1 to 6; income funds Ballistae. Their guild is untaxable, so you lose their gold. (fan, prima)

## 16. Gnome (Gnome Hovel)

- **Where / cost:** Gnome Hovel (100 gold; excludes Elves and Dwarves). 100 gold each, three per hovel. After the first, more hovels appear on their own up to three; if all burn down the Gnomes leave. (manual, prima)
- **Personality:** Stunted, pacifist, scavengers; gives newly recruited heroes a small luck bonus. Not counted toward Palace requirements. (manual, prima)
- **Unprompted:** Helps build (75), Royal Gardens (50), fights monsters (20), heroes (10), learns spells (15), goes home (90). 3 HP. (prima)
- **Special ability:** Builds and repairs very fast. In the Northern Expansion a Gnome at level 8 becomes a Gnome Champion, a real fighter. (fan)
- **Player view:** Strong early start and repair; hard to get rid of and attract graveyards; other races shun towns with Gnomes. (fan, prima)

## 17. Warrior of Discord (Warriors Guild, needs Temple to Fervus)

- **Where / cost:** Warriors Guild plus Temple to Fervus. 900 gold. Can also be freed from Slave Pits (10 percent) or arrive as quest reinforcements. (prima)
- **Personality:** "Chaos incarnate", insane brutes created by Fervus's priests; huge bladed polearm, a harness of leather, chain and spikes; the least intelligent hero. (manual, prima)
- **Abilities:** Attack Multiple Foes (up to three foes within 30 at level 3), 3 percent instant-kill Critical Hit, immune to poison, 35 HP and Strength. (prima)
- **Unprompted:** Raids enemy buildings and lairs (95), fights heroes and monsters (95), goes home (50). They cannot buy potions or upgrades. A fan reports their howl causes fear. (prima, fan)
- **Interactions:** Elven Lounges tempt them (willpower matters). Call to Arms affects them. (prima)
- **Player view:** Strong but suicidal; high upkeep; works with priestesses and Call to Arms. (fan)

---

## 18. Guild and temple summary

### Warriors Guild
- Level 1 only; 800 gold; 700 HP; income 40 percent; holds four across Warriors, Paladins, Warriors of Discord. Call to Arms 500 gold. (prima)
### Rangers Guild
- 700 gold; 250 HP; Move Camp 800 gold. (prima)
### Rogues Guild
- 600 / 850. Extortion (35 percent cut). Level 2: free poison for all heroes. Causes Gambling Hall with Elves. (manual, prima)
### Wizards Guild
- 1500 / 2500 / 3500. Two Sovereign spells per level, spell range 800 / 1050 / 1300. Allows Libraries and Wizards Towers; enchants heroes' gear. (manual, prima)
### Temples (Sovereign spells)
- **Agrela:** Healing 200, Blessing 100, Resurrection 1500.
- **Dauros:** Stone Skin 200, Vigilance 800, Petrify 1500; unlocks Paladins.
- **Krypta:** Wither 300, Animate Bones 800, Re-Animate 2000.
- **Fervus:** Healing 400, Illusionary Hero 500, Vines 1000; unlocks Warriors of Discord; spawns animals.
- **Krolm:** Rage of Krolm 1500.
- **Helia:** Fire Strike 350, Sun Scorch 1200; no range limit on temple spells.
- **Lunord:** Winged Feet 400, Wind Storm 1000. (prima, manual)
- Temple spells have no range limit; Wizards Guild spells do. (manual)
- Prima strategy flavour: Fervus and Krypta are "the hammer" (armies), Agrela and Dauros "the scalpel" (nurturing). Krolm is mayhem. Helia direct damage; Lunord defence. (prima)

### Northern Expansion additions touching heroes
- **Embassy:** recruits veteran heroes at random into open slots (costs gold per hero). **Outpost:** second command centre, houses six heroes. **Hall of Champions:** place a bounty on a monster type; visiting heroes gain HP and healing. **Magic Bazaar:** six potions. **Mausoleum:** stores and later revives dead heroes. **Sorcerers Abode:** Change of Heart (turns a fleeing hero berserk), Frost Field, Dismiss, Chain Lightning, Gate (teleports heroes), Earthquake. (manual)

---

## 19. Things I could not confirm

- Warrior of Discord "howl" (fan thread only).
- Paladin Harm Undead at level 8; Solarus Fire Hammer and death-explosion (fan wiki snippet only).
- Whether Rogues truly "profit" in the Gambling Hall (lightly tested by one fan).
- The exact 60/40 tax split (from a removed Steam guide).
- Precise flee HP threshold and the exact flag-acceptance formula: not found.
- Fandom wiki pages and GameFAQs FAQs: unreachable; their detail may be richer than what is here.

---

## URLs used

- http://cdn.akamai.steamstatic.com/steam/apps/25990/manuals/Majesty_Manual_HD.pdf?t=1447352469
- https://archive.org/stream/Majesty_Prima_Official_eGuide/Majesty_Prima_Official_eGuide_djvu.txt
- https://www.cyberlore.com/majesty/chapter1.htm
- https://www.cyberlore.com/majesty/chapter2.htm
- https://www.cyberlore.com/majesty/chapter4.htm
- https://www.cyberlore.com/majesty/chapter5.htm
- https://strategygamers.com/walkthrough/majecty-the-fantasy-kingdom-sim
- https://steamcommunity.com/app/73230/discussions/0/2262439317591133046/
- https://steamcommunity.com/app/73230/discussions/0/540732889649219561
- https://steamcommunity.com/app/73230/discussions/0/520519112496590406/
- https://steamcommunity.com/app/73230/discussions/0/1482109512316717434/
- https://steamcommunity.com/app/73230/discussions/0/133261907146207943/
- https://steamcommunity.com/app/73230/discussions/0/405692224227727608/
- https://steamcommunity.com/app/73230/discussions/0/3106893419989470313/
- https://steamcommunity.com/app/73230/discussions/0/1741101364289920969/
- https://steamcommunity.com/app/73230/discussions/0/133261907142158017/
- https://steamcommunity.com/sharedfiles/filedetails/?id=2199487769
- Fandom wiki pages seen only through search summaries (fetch blocked): https://majesty.fandom.com/wiki/Rogue , /Rogue's_Guild , /Cultist , /Elf , /Elven_Bungalow , /Wizard , /Library , /Healer , /Priestess , /Solarus , /Monk , /Paladin , /Dwarf , /Gnome , /Adept , /Ranger , /Barbarian , /Temple_to_Fervus , /Temple_to_Krypta , /Temple_to_Helia , /Attributes , /Economy
