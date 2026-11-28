# Majesty reference

This is what made *Majesty: The Fantasy Kingdom Sim* (Cyberlore, 2000) and its Northern Expansion feel alive, gathered as the bar for Naibihpma Hold. The first version of this page listed systems and missed the point. This one is organised around behaviours: what heroes do unasked, what they make for each other, and how the kingdom's own success makes trouble.

**The detail behind it** is in three research files in `majesty/`:
- `heroes.md`: every hero class, with its behaviour list
- `buildings.md`: every building, the economy and the sovereign's tools
- `monsters-and-life.md`: monsters, lairs, and what players remember

**Sources.** The research used:
- the Majesty Gold HD manual, read in full
- the 2000 Prima Official eGuide on archive.org, which has each hero's behaviour list with percentages
- Cyberlore's original web pages
- Steven Carter's quest guide
- archived fan wiki pages, Steam threads and reviews

The live fan wiki and GameFAQs refused automated access, so some details may be missing. The numbers come from the Prima guide or fans, and are "as reported".

## Every hero runs a list of maybes

- **Ranked tasks with chances.** Each class has a ranked list of tasks, each with a chance of being *considered* (Prima). A hero works down the list until it reaches a task it considers and can do. It rethinks every so often, and whenever something happens nearby. Some lists:
  - **Rogue:** loot gravestones 100, steal 95, harvest poison plants 90, go home 95.
  - **Solarus:** find hidden lairs and raid them 95, explore 75, go home 5.
  - **Ranger:** collect healing herbs 95, explore 95, follow and support Barbarians 90, go home 15.
  - **Healer:** heal others 95, follow Warriors 75, follow the Tax Collector 50, follow Wizards 35.
- **No two heroes alike.** Each recruit's attributes roll a little above or below its class, and each arrives with 50 to 99 gold of its own. Two Warriors make different choices on the same afternoon.
- **Hopeless rewards are ignored.** No bounty will make a Ranger attack a Skeleton, because a Ranger would barely hit something that dodges so well (manual).
- **Heroes take a moment to notice a new flag.** The flag shows how many are heading for it.

## Heroes make things for each other

- **Herbs.** Healers plant healing herbs, and Rangers harvest them, five herbs to a potion. Rangers can carry more potions than anyone else.
- **Poison.**
  - Cultists sow poisonous plants, and Rogues harvest them, five to a dose of weapon poison.
  - A level 2 Rogues Guild gives poison to any hero who wants it. It deals 1 HP a second for 10 seconds, resets on each hit, and stops healing.
  - It's the answer to Trolls' regeneration, and the Prima guide says poison decides tough quests.
- **Skeletons and undead.**
  - Priestesses of Krypta raise skeletons: one per two levels, up to three, which follow them, soak damage and earn them experience.
  - At level 7 a Priestess can take control of enemy undead: skeletons, zombies and vampires.
  - She can only charm with a free follower slot, so players put an attack flag on one of her own skeletons to make room for a vampire. Fans call a Priestess with enslaved vampires the best hero in the game.
- **Animals.** Cultists charm animals (rats, rocs, vargs, hell bears) and keep up to three. Their temple spawns animals for them to charm. At level 7 a Cultist can become a hell bear.
- **Escorts.** Majesty has no parties, only follow-and-support tasks:
  - Healers follow Warriors, Tax Collectors and Wizards.
  - Rangers have sworn (the Treaty of Ravenswood) to support Barbarians.
  - Monks guard Healers (95) and caravans (75).
  - Wizards tag along with Barbarians and Monks.

## Each class has a signature habit

- **Solarii raid lairs on sight.** They sense hidden lairs from afar and go for them without a flag. That's useful, and a nuisance when a quest wants a lair left alone. They also garrison Guard Houses and heal there.
- **Rangers explore,** and gain 10 sight and 10 range every level. Once the map is known they sometimes walk off its edge. They come back about a day and a half later, if they survive, with 3,000 experience and some gold.
- **Adepts patrol the whole kingdom.** While they're on duty, sewer and graveyard trouble rarely matters. They don't raid without a flag.
- **Paladins and Warriors raid lairs by themselves,** which can spoil a coordinated plan.
- **Builders.** Dwarves and Gnomes rush to build and repair.
  - Gnomes don't look for danger while building, so they don't run from phantoms.
  - Gnome hovels multiply by themselves up to three. Burn them all and the Gnomes leave.
- **Healers come back.** Once per level, a killed Healer drifts home as a spirit and revives.
- **Barbarians** are almost always berserk, think armour is for cowards and rarely shop.
- **Monks** keep a vow of silence and fight with their fists. After level 7's Iron Will they're nearly unkillable. They're slow, though, so running away often kills them.
- **Warriors of Discord** are the least intelligent heroes. They can't buy potions or upgrades, and have a small chance to kill outright.
- **Elves** play music at Inns (30 gold and a free stay), Trading Posts and Marketplaces.
- **Rogues** hang around heroes who might die, to rob the gravestone. They steal from lairs, and from your buildings if they're disloyal enough. They're the only heroes who will tear down an Elven Lounge.

## Heroes grow through buildings

- **The Library.** Wizards learn Fire Blast at level 3 and Meteor Storm at level 7 there, once you've researched them. A Wizard with nothing better to do visits the Library 90% of the time.
  - A Library makes every later recruit more intelligent.
  - Heroes of Intelligence 11 or more can buy Power Shock and Flame Shield there.
- **The Fairgrounds.** You pick the tournament: melee, archery, magic or combined, which picks who comes. Entry is 10 gold, and with more than four entrants the winner gains a level outright. Choosing "no contest" closes it and sends heroes back to the field. One fan levelled Wizards past level 50 in half an hour this way.
- **Every habit earns experience,** so a hero grows by doing its habit. Some values from the Prima guide:
  - attacking a lair: 75 × its difficulty
  - exploring new ground: 250
  - garrisoning: 250
  - stealing: 200 per attempt
  - entertaining: 175
  - a Ranger's journey: 3,000
- **The Palace grows with heroes.** It needs 4 living heroes to reach level 2 and 12 for level 3, and Gnomes don't count. That's why players hire cheap Rogues early.

## Temptation, loyalty and vice

- **Willpower** decides who gives in:
  - The Elven Lounge costs about 200 gold a visit and pays no tax.
  - At the Gambling Hall heroes bet 1 to 10 gold a go, with published odds. Blue pays double half the time, gold pays twenty times one time in twenty, and a skull loses everything one time in four.
  - Monks almost never go. Elves, greedy and weak-willed, gamble away their savings.
- **The Elven bargain.**
  - An Elven Bungalow doubles Marketplace income, but an Elven Lounge appears by itself, plus a Gambling Hall if you have Rogues. You can't remove either one yourself (Rogues can).
  - Fans build the bungalow last, so the Lounge and Hall go up on the outskirts as a buffer.
- **Loyalty.** Statues (+5) and Royal Gardens (+15) raise it.
  - A disloyal hero will answer a flag on your own building, and Rogues (loyalty 0) steal from you unless you have enough Statues.
  - A loyal hero whose guild is destroyed waits at an Inn instead of leaving.
- **Greed.** Very greedy heroes will attack friendly things for a reward. Rogues and Elves take flags cheaply, while Paladins need huge bounties.

## The gold you can see

- **Earning and tithes.** Heroes earn from kills, flags, lair caches, chests and theft. Going home, they tithe to their guild or temple: Warriors 40%, Agrela 50%, Rogues 30%. Elves, Dwarves and Gnomes pay nothing.
- **Tax collectors.**
  - They walk a route you can edit, with a minimum pickup and a minimum return. Guard Houses act as banks along the way.
  - A killed collector drops its gold. A hero nearby scoops it up; otherwise it's gone.
  - Ratmen hunt collectors, and Healers follow them around.
- **What heroes pay for:**
  - a potion: 25 gold, heals 30, and a hero carries five
  - Blacksmith upgrades: about 100 to 300 gold a tier
  - Wizard enchantments: 200, 400 or 800
  - an Inn: 12 gold for a full heal

  A hero who dies carrying thousands loses it all.
- **Marketplaces** are the main income, and must be spaced apart.
  - Market Day gives a pile of gold now, then shuts the market and throws the shoppers out. Fans use that to push heroes back into a fight.
  - Trading Post caravans pay more the farther they travel, multiplied by the destination market's level, and Trolls ambush them.
  - Extortion collects every coffer at once, minus the Rogues' cut.
  - A Fountain appears for every five of certain shops, and adds a collector.
- **Flags.**
  - Paid up front, raised 100 at a time, never lowered.
  - Removing a flag loses its gold.
  - The reward is shared by everyone nearby when the target falls.

## Success makes trouble

- **Sewers** appear as you build, roughly one every four or five buildings (a fan estimate). They can't be destroyed. Rats and Ratmen come out to ambush tax collectors and rob Marketplaces; one fan saw a market lose about 1,000 gold in seconds. Heroes who kill the Ratmen get the gold.
- **Graveyards** appear after many hero deaths (about 15), spawn undead, and can't be destroyed. The expansion's Mausoleum stores the dead instead, and releases undead if it's destroyed.
- **Houses** appear as you recruit. They take land, add a little tax, and add a little to each Marketplace's income.
- **Growth feeds itself.** Gnome hovels multiply, and more buildings mean more sewers.

## Monsters with motives

- **Monsters target the nearest building,** so a Guard House beside a Marketplace is a decoy. The manual says so.
- **Trolls** want treasure: Marketplaces, caravans, gold sitting in Trading Posts. They regenerate, so poison is the counter.
- **Ratmen** want tax gold and market coffers.
- **Minotaurs** raid in parties, hitting the nearest Trading Post or outlying houses on the way to the Palace.
- **Dragons** go for Peasants repairing buildings. The boss dragon picks targets unpredictably.
- **Vampires** reflect Wizards' spells back with Magic Mirror. The counters are Paladins, Anti-Magic Shield and poison.
- **Skeletons** dodge and shrug off missiles, so Rangers won't fight them. They're also why a Priestess's own skeletons make good dragon fodder.
- **Others:**
  - Medusae chase heroes and Elves, and Gorgons turn heroes to stone.
  - Rock Golems want buildings.
  - Hell Bears guard their dens.
  - Spiders come in swarms that flatten buildings.
  - Rust Spitters come in threes.

## Death and the dead

- **Gravestones.**
  - They last longer for higher levels: about a sixth of a day, plus a sixth per three levels.
  - Agrela's Resurrection costs 1,500. Krypta's Re-animate costs 2,000 and revives the hero where it fell, at half health.
  - A gravestone holds its guild slot. Recruiting a replacement dismisses the lowest-level stone.
- **Orphans.** If a hero's guild is destroyed, it walks to the Palace and leaves, unless a new guild is built in time.
- **Names.** Heroes are named, can be renamed from level 10, and each class has its own last words.

## Exclusions make each kingdom different

- **Temples.** Krolm excludes every other temple. Agrela and Dauros exclude Fervus and Krypta, and Helia and Lunord exclude each other.
- **Races.** A kingdom has only one of Elves, Dwarves or Gnomes.
- **Combinations.** Paladins need a Warriors Guild plus Dauros, and Warriors of Discord need a Warriors Guild plus Fervus.
- The Prima guide calls Fervus and Krypta "the hammer" and Agrela and Dauros "the scalpel".

## The sovereign's hand

- **Spells.** Each spell costs gold per cast.
  - Temple spells reach anywhere you can see.
  - Wizards Guild spells reach only as far as the guild and its towers.
- **Guild actions:**
  - Call to Arms: warriors teleport home, berserk.
  - Move Camp: the Rangers Guild moves, even mid-attack.
  - Extortion.
  - Market Day.
  - Rage of Krolm: every hero boosted for nearly half a day.
- **The expansion adds:**
  - Change of Heart: a fleeing hero goes berserk, and a fighting one runs.
  - Gate: heroes teleported to a chosen spot.
  - Dismiss: the target sent far away.
  - The Hall of Champions: one bounty on a whole monster kind, marked with medallions.

## What players say made it special

- **A toy as much as a game.** Players value how differently its characters behave, and the way watching heroes go about their lives makes you feel like a monarch.
- **Majesty 2 lost the personality.** Reviewers missed the original's "forgetful wizards and lazy elves". Its heroes stayed near home, felt uniform and needed more flags to get anything done.
- **The voice.** Fans recall the advisor's comic, often useless advice, the tax collectors, and each class's lines.
- **The designer's aim** (Jim DuBois, interview): to avoid micromanaging an army. The stories the player imagines about their subjects should matter as much as the game's events.

## Each hero at a glance

| Hero | Home | Signature | Grows by | Spends on | Works with |
|---|---|---|---|---|---|
| Warrior | Warriors Guild | Raids lairs and naps at the Inn | Hit points | Blacksmith, potions | Healers follow them |
| Ranger | Rangers Guild | Explores; journeys off the map | +10 sight and range per level | Little (least greedy) | Harvests Healers' herbs; supports Barbarians |
| Rogue | Rogues Guild | Steals; robs gravestones | Theft experience | Hoards; gambles | Harvests Cultists' poison |
| Wizard | Wizards Guild | Studies | Library spells at levels 3 and 7 | The Library | Follows Barbarians and Monks |
| Healer | Agrela | Heals passers-by; Rebirth | Healing experience | Almost nothing | Plants herbs for Rangers; follows Warriors |
| Priestess | Krypta | Raises skeletons | Controls undead at level 7 | The Library | Her skeletons guard her temple |
| Cultist | Fervus | Charms animals | Becomes a hell bear at level 7 | | Plants poison for Rogues |
| Paladin | Warriors Guild + Dauros | Raids lairs; guards the temple | Shield of Light at level 4 | Potions | |
| Monk | Dauros | Guards Healers and caravans | Iron Will at level 7 | Hardly anything | Wizards like to travel with them |
| Solarus | Helia | Senses lairs and raids them | Sun Scorch at level 4 | | Garrisons Guard Houses |
| Adept | Lunord | Patrols the kingdom | Teleport at level 4 | | |
| Barbarian | Krolm | Berserk | Critical hits grow with level | Rarely shops | Rangers support them |
| Dwarf | Dwarven Settlement | Builds and repairs | | Untaxable | Unlocks Ballista Towers |
| Elf | Elven Bungalow | Busks; gambles | | Lounge, Gambling Hall | Doubles Marketplace income |
| Gnome | Gnome Hovel | Builds fast, ignores danger | Champion at level 8 (expansion) | Untaxable | Luck for new heroes |
| Warrior of Discord | Warriors Guild + Fervus | Raids; may kill in one blow | | Can't buy | Call to Arms |
