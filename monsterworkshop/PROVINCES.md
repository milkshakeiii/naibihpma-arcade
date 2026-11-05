# Provinces and the tech tree (sketch, 2026-11-02)

The living world's five provinces, with every good in Henry's original tech tree placed in one of them. Five provinces and the principles below are agreed; the placements, boasts, expatriate industries and new varieties are proposals. Names are working names, not player-facing.

**Where the tree lives:** `data/good_types.json` holds all 105 goods of the second tree (`django/.../commands/tech_tree_two.py`) and 17 we've added. The first tree (`tech_tree.py`) also defines copper ore, the ingot smelted from it at a smelter, and primitive tongs, which aren't in the game's data yet. The second tree names about 60 more goods as inputs or tools without defining them (metal ingot, charcoal, beeswax, oil, mortar, forceps, shovel, sickle...).

**Marks:** ✓ in the game now · **A** added by us and approved, in the data, not in play yet · **T** defined in an original tree, not in play yet · **N** named by the tree as an input or tool, never defined · **+** in neither tree.

**Approval by mark** (agreed): T needs only a yes on where it's made or grows; N gets its recipe proposed from the book, in batches; + needs Henry's explicit approval, one good at a time.

## The world map (tentatively agreed 2026-11-04)
About 90 screens (`tools/world_map.py` draws it). The Cloth Reaches in the north-west, as built, with room for fields to the west; the Ore Hills east past the high pass; the Kiln Banks south of Temple Town; the capital at the crossroads in the middle; the Paddy Plains in the south-west; the Salt Coast in the south-east along the sea. Barren country between some of them: the high pass, a gorge, a range between the hills and the coast, a marsh between the plains and the coast with no road.
- **Gates,** each counting a rate of goods: the high pass (cloth, built), south of Temple Town into the kiln banks (coal or charcoal), the capital road (bricks and tiles), into the plains (sickles), to the coast (metal pots, for brine). The tunnel under the range, the hills to the coast, opens with the coast.
- **Rivers** (Henry: more of them): the pine river out of Tinless, south through the gorge, the kiln banks and the plains; its branch, splitting off in the kiln banks and running east past the marsh, which its mouth makes, to the sea; and the Rust Brook, a small river rising in the hills' Tin Gulch, through the springs and past the copper cut, down Iron Brook and west out of the hills to join the pine river in the kiln banks. Each is a reason for bridges and fords.

## Principles (agreed 2026-11-02)
- **Tasteful exceptions to strict compartments.** Each province has native industries, plus a secondary "expatriate" industry that supplements them.
- **Common materials everywhere, in provincial varieties.** Sticks, logs and fiber can be had in every province, each its own variety with a slightly different weight, color and value. Henry's few tree species get provincial ones added.
- **Many roads to every tag.** Recipes ask for tags several goods provide, so one output comes in flavors: light, cheap, expensive, heavy, and different colors.
- **The book's boasts are true.** Its funny value judgments ("this place does this best") are kept, and they turn out to be right.
- **The bell:** its recipe stays exactly as Henry wrote it (intentionally complicated). While only the first province is open, a temporary vendor or the like supplies what it needs from elsewhere.
- **Kits:** advanced to build, often from sawn planks; a province's workshops are built from its own goods, as a starting point with variations; one workshop of each kind, some humble and some hard to produce.

## Everywhere, in provincial varieties

| | Cloth Reaches | Ore Hills | Kiln Banks | Paddy Plains | Salt Coast |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **Wood** (logs, sticks, fuel) | hemlock ✓, oak ✓, elder ✓, rhododendron ✓ | pine + (also rosin and pitch) | willow + | elm +, jujube + | camphor +, palm + |
| **Fiber** (to thread anywhere) | cotton ✓, ramie ✓ | hemp T | climbing hempvine T | velvetleaf T, silk T | kudzu T |
| **Bamboo** | arrow bamboo ✓ | | | | a big coastal bamboo + |
| **Lime** | limestone ✓ | | | | oyster lime T |
| **Clay** | | | all five colors T | red clay T (the common one) | |

The new species (+) are placeholders for a later batch, each for approval; a second, coastal bamboo is agreed in principle (2026-11-02), so bamboo has more than one source. If I remember the book's cart chapter right, it names the scholartree, jujube, sandalwood and elm as the woods for axles, which would tie these into wagons.

## 1. The Cloth Reaches (today's Pinewater and Tanglemere: the first province)
River lowland with cotton fields, meadows, pine and oak woods, a tangled lake. The book's chapters on clothing materials and dyes, plus lime.
- **Grows:** cotton bolls ✓, ramie ✓, indigofera ✓, limestone ✓, the four woods ✓, arrow bamboo ✓; pigweed indigo T, scholartree buds T.
- **Cotton and bast:** cotton slivers ✓, thread ✓, fabric ✓, rope ✓; cotton seeds T (an oily byproduct of the gin), bast fiber T (needs a knife from the hills, a tub and a roller N).
- **Indigo and greens:** lye ✓, indigo ✓, dyed fabric ✓; seeth water T, scholartree bud cakes T, dark green dye T (needs alum from the hills), light green dye T (needs green vitriol from the hills).
- **Lime:** quicklime ✓.
- **Carpentry:** wooden tub ✓, workbench kit ✓, kiln kit ✓; spool A, spinning wheel A, waist loom A, cotton gin A, cotton bow A (none has anywhere to be made yet); planks N; die pattern T (planks) and mirror pattern T (a log, planks and a nail from the hills).
- **Boast:** the finest cotton cloth is woven here.
- **Expatriate industry:** the temple town's small works for the bell, the hills and the kiln banks in miniature: a copper seam, a clay bank and some hives.
- **Sends out:** fabric, dyed fabric, rope, lime, timber, tubs, the weaving tools, patterns. **Needs from elsewhere:** metal tools (knives, saws, chisels), mordants (alum, green vitriol).

## 2. The Ore Hills
Rocky hills, mines and charcoal woods. The book's chapters on metals, hammer forging, and burning stones (coal).
- **Mined or found:** copper ore T (first tree), tin ore N, iron ore N, silver ore N (the tree asks for silver, bronze and iron), coal T (needs a pick and a shovel), hard stone N, pyrite N, alum T, green vitriol (found) T, well salt T; hemp T and pine + grow here.
- **Fuel and smelting:** charcoal N, coke T, ingots T (first tree: the ore's metal carries over), bronze ingot N (copper with tin), solder N; the smelter T (first tree: a workshop that is itself a good, as our kits are).
- **Smithing:** forceps N (the first tree's primitive tongs T may be the same thing), metal anvil T, stone anvil T, stone hammer T, metal hammer T, metal chisel T, saw T, file T, wire T, needle T, nail T, knife T, metal axe T, pick A, shovel N, sickle N, scythe N, billhook N.
- **Small castings:** metal pot T, sycee T, die T (each in a mold from the kiln banks); green vitriol T (from pyrite and iron).
- **Boast:** the most copper comes out of these hills, and nowhere else has tin.
- **Expatriate industry:** hemp cloth: the miners' coarse cloth and sacks (bags, which the tree needs as a tool).
- **Sends out:** ingots and every metal tool, which every other province needs (sickles for grain, billhooks for cane, picks and shovels for coal), fuel, mordants. **Needs from elsewhere:** molds, food, rope, timber for the mines.

### The Ore Hills, screen by screen (a proposal, 2026-11-04)
Fifteen screens east of the high pass, five across and three down (world screens 6–10, 2–4; `tools/hills_map.py` draws them). The first two screens show what's new to the game, before anything seen already (Henry): a cliff and the vine at the way in, then copper; charcoal and woven cloth, already known from the Cloth Reaches, come a row further in. The road comes in from the high pass gate on the north-west and runs east along the top; a trunk road drops south through the springs to the tunnel's mouth; a cross road runs west to the copper cut and the stone stair and east to Bellows and the foundry. The Rust Brook rises in the Tin Gulch, runs west under Bellows and the springs, past Hemp Hollow's retting pond, down Iron Brook and out of the hills to the pine river, bridged where the roads cross it.

| | west | | | | east |
| :-- | :-- | :-- | :-- | :-- | :-- |
| **north** | The Stone Stair, the way in: the road from the pass climbs a cliff, the cliff-lift vine hauling goods up beside it, a hard-stone quarry below | Copper Cut: the most copper, terraced pits, rails and carts, the copper miner and his board | Verdigris: the town square, the gathering hall, order boards, the smithy and the smith | Verdigris East: houses, piles of spare picks and shovels for loops, a tool drop-off | The Tin Gulch: the only tin in the world, a narrow gorge where the Rust Brook rises, a prospector's camp |
| **middle** | Charcoal Wood: dark pines, smoking charcoal mounds, the charcoal burner, the stonemason's yard | Hemp Hollow: hemp fields, a retting pond on the Rust Brook, the hemp house (the side industry), the hemp weaver | Alum Springs: colored mineral springs (alum, green vitriol), a brine well, pyrite | Bellows: the smelters, the miners' outfitter (the +2 belt), an assay office taking ingots | The Foundry: where the Great Statue and the Large Mirror are cast; pots, sycee and dies; the founder |
| **south** | Iron Brook: iron seams along the Rust Brook, red with iron, pines, a sawpit | The Great Furnace: kept fed with ore and charcoal for a set time, it gives something special | The Deep Drift: the tunnel's mouth to the coast (open with the coast), deep seams of silver | Slag Slopes: slag heaps, scree, hard stone, a lookout over the range | The High Scree: wild heights, wild hemp, a hermit's cairn where a +3 piece is given |

- **Working townsfolk,** each at their own work, with a refilling order board like Old Spoke's and their trade's secret on a stand: the charcoal burner (firing), the hemp weaver (textiles), the smith (blacksmithing), the copper miner (prospecting, as Old Soot's), the founder (casting), the stonemason (stone carving). Six may be too many to build at once; the smith, the copper miner and the founder first.
- **New workshop kinds:** the charcoal mound, the hemp house, the smithy (a forge with an anvil), the smelter (the first tree's, a workshop that is itself a good, as kits are), the brine boiler; the foundry is Temple Town's kind; the great furnace is one of a kind.
- **New gatherables:** pines, hemp, copper, tin and iron seams (worked with a pick and a shovel, as coal is), hard stone, pyrite, alum and green vitriol at the springs, silver in the drift.
- **Drop-offs:** a tool drop-off in Verdigris East, the assay office in Bellows (ingots), and the foundry's boards for the statue and the mirror.
- **New props and terrain** (drawn in the one style, in the hills' palette): cliffs and scree, rust-red and stony streams, charcoal mounds, ore and slag heaps, a smelter's chimney, timbered adits, retting ponds, a cairn; carried over: conifers, mine mouths, rails and carts, lanterns, boulders, wells, barrels.

**The goods, by mark** (approval follows the mark):
- **T, needing only a yes on where:** copper ore (the first tree; the Copper Cut); alum and green vitriol, found (Alum Springs); green vitriol, made from pyrite and iron (Alum Springs); well salt (the brine well); hemp stalks (Hemp Hollow); the smelter and the ingots smelted in it (Bellows); the metal anvil, metal hammer, metal chisel, saw, file, wire, needle, nail, knife and metal axe (the smithy in Verdigris); the stone anvil and stone hammer (the stonemason's yard in Charcoal Wood, from the Stone Stair's quarry); the metal pot, sycee and die (the foundry, in molds from the kiln banks).
- **N, recipes to propose from the book, as a batch:** tin ore, iron ore, silver ore, hard stone, pyrite, bronze ingot, solder, forceps, sickle, scythe, billhook, a real recipe for the shovel (the data's has no inputs), and hemp's fiber, coarse cloth and sacks. The metal ones are agreed, below; hemp's are the next batch.
- **+, each needing your approval on its own:** the pine (its logs and sticks, and rosin and pitch).
- **Data fix, done 2026-11-05:** found green vitriol's cauldron was split into letters; it asks for a cauldron now, which is the metal pot.

### Metals, from ore to edge (agreed 2026-11-05)
Metalworking was thin at the start (the ingots were stand-ins from the trader, and there was no iron) and flat in the middle (ten tools on nearly one recipe). This batch fills it in from the book's chapters on the five metals, hammer forging and casting: 23 goods, all but wrought iron, steel and the iron forceps asked for by recipes already in the trees. Henry approved it on 2026-11-05, wrought iron, steel and the tool-life numbers included, and it's in the data (`data/good_types.json`) with tool life and the cards that teach it; the smelter and smithy, the goods' art, and the smith's and assay office's words come with the hills. The die and mirror molds aren't missing after all: the sand mold takes its pattern's name.

"Heat" is coal, coke or charcoal; logs and sticks are only fuel. "Hard metal" is steel, pig iron or bronze. Each ingot is its metal's variety, so what's made from it carries the name: a steel chisel, a cast iron pot, a bronze bell.

**Gathered,** no workshop:
- Copper ore T: pick and shovel; the Copper Cut's seams.
- Tin sand N (the tree's tin ore): a shovel and a basket, washed in the Rust Brook in the Tin Gulch.
- Iron ore N: a shovel; Iron Brook's red banks.
- Silver ore N: pick and shovel, rare and slow; the Deep Drift.
- Hard stone N, as granite (a new stone): hammer and chisel; the Stone Stair's quarry. A second kind (basalt, on the Slag Slopes) would give it the usual several.
- Pyrite N: a pick; Alum Springs and the Slag Slopes.

**The smelter,** in Bellows (the first tree's workshop kind, new to the game): five pads; it lends tongs and a hammer, and every recipe uses the tongs.
- Copper ingot T: 2 copper ore and heat, making 1.
- Tin ingot N: 2 tin sand and heat, making 1.
- Silver ingot N: 2 silver ore and 2 heat, making 1.
- Pig iron N (the tree's "iron"; in the data the Pig Iron Ingot, as are the Wrought Iron and Steel Ingots, so every metal ingot reads as one): 2 iron ore and heat, making 1.
- Wrought iron +: pig iron, clay and sticks, making 1, with no heat of its own (the book stirs the iron as it runs from the furnace). The smelter's art gets a stirring pond beside it.
- Steel +: wrought iron, pig iron, stalks, clay and heat, making 2; tongs and hammer.

**The foundry** (the hills' foundry, Temple Town's kind):
- Bronze ingot N: 2 copper ingots, a tin ingot and heat, making 3.
- Solder N: a tin ingot and heat, making 2.
- The metal pot (now also the cauldron), sycee and die, as written.

**The smithy,** in Verdigris (a forge with an anvil, new to the game): five pads; it lends tongs, a hammer, a stone anvil and a tub.
- Forceps N, iron tongs: a metal ingot and heat; tongs, hammer and anvil. They fill the same slots as tongs; being metal, they last longer (Henry: the tongs are wood, so the forceps stay).
- Shovel N: a metal ingot, heat and sticks.
- Sickle N: a metal ingot, heat and sticks.
- Scythe N: a metal ingot, heat, 2 sticks and rope.
- Billhook N: a metal ingot, heat and sticks; a file as well.
- Strong pan N, for boiling brine: 2 metal ingots and 2 nails, riveted cold; hammer and anvil. The book's other kind, woven bamboo plastered with oyster lime, belongs to the coast's batch.
- The tree's other smithing recipes, with the changes below.

**Any bench:** primitive tongs T (sticks; they count as forceps), double tongs T (2 tongs) and the basket N (2 sticks; the silkworms' basket too).

**Changes to recipes already written:**
- Agreed: the metal chisel, file and our pick take a hard metal ingot, and a tub for the quench. The knife and axe stay any metal (Henry: a simple knife or axe shouldn't need steel), and so do the new sickle, scythe and billhook.
- Agreed: the metal pot is also the cauldron, which silk thread, the scholartree cakes and both green vitriols need and nothing provides.
- Agreed: forging takes heat. The fuel pad becomes a heat pad in the knife, metal axe, metal hammer, metal chisel, file, saw, wire, nail, pick and metal anvil. Unchanged: the castings (the pot, sycee and die take coke already; the bell and statue as written) and everything outside metalwork.
- Agreed: the needle, now a wire and a file anywhere, becomes a smithy recipe: wire, heat and clay, making 4; file, cauldron and tub. The book packs needles in earth, pine charcoal and fermented soybeans before the quench; soybeans would make needles wait for the plains, and the hemp batch may want needles, so they're left out.

**Tool life by material.** A tool's material sets its durability: the material of its working part, which is its metal if it carries one, otherwise its stone, then its wood or bamboo, then its clay. A hammer with an iron head on an oak handle is an iron tool. Weight matters already (a tool in hand fills body capacity, and loose goods push slower), and value when a tool is delivered (renown), but neither is felt while working. The pick and shovel keep the fixed weights their recipes give them: weighing by metal brought back the trouble of 2026-11-03, a pick and shovel too heavy to hold together.

| Material | Durability | Tools made of it |
| :-- | :-- | :-- |
| Steel | ×2 | every metal tool |
| Bronze | ×1.5 | |
| Wrought iron | ×1.25 | |
| Copper, pig iron | ×0.75 | pig iron is hard but brittle |
| Tin, silver | ×0.5 | |
| Granite | ×1 | stone anvil, stone hammer |
| Oak, rhododendron | ×1 | tongs, double tongs, basket, flail, spool, cotton gin, spinning wheel, waist loom, tub |
| Hemlock | ×0.75 | |
| Elder | ×0.5 | |
| Arrow bamboo | ×0.75 | cotton bow |
| Fired clay | ×0.5 | pot and sycee molds (the book breaks a clay mold to free what's cast in it) |

The metal tools: the metal pot (the cauldron), the metal anvil, knife, axe, hammer, chisel, file, saw, die, pick, shovel, forceps, sickle, scythe, billhook and strong pan. A new variety gets its number with its data, as its color: pine and willow ×0.75 and elm and camphor ×1, say, jujube ×1.25 (the book's wood for axles), palm ×0.5. So elder tongs, oak tongs, iron forceps and steel forceps make a ladder: half, one, a quarter more, twice.

Durability, not uses: each go wears every tool it uses by that recipe's weight for the tool, 1 to 8 and mostly 2 or 3 (a dig wears a pick and a shovel 3 each, so a pick of 100 lasts 33 digs; weaving wears a loom 8), halved by a whetstone and doubled by a hasty apron. A town workshop's own tools never wear, nor do a working townsperson's.

**Where players learn it** (Henry: it must be discoverable). Every surface gives the same numbers, as durability against the usual 100:
- A material's card (an ingot, a log, sticks, bamboo, granite, clay), where some tool can be made of it: "TOOL DURABILITY 200 (MOST 100).", under the card's own name for the material. Built.
- A tool's card: its durability left out of its own ("DURABILITY 143/200"), and why: "STEEL: 200 (MOST 100)." The character sheet says the same. Built.
- A workshop's card while it's making a tool, at the moment of choosing what goes on the pad: what fits the material pad, by durability ("DURABILITY BY METAL: STEEL 200, BRONZE 150, WROUGHT IRON 125, COPPER 75..."), only what that recipe takes (a pick lists "STEEL 200, BRONZE 150, CAST IRON 75"). Built.
- A plant's card and a workshop's card: what one go takes off each tool that would wear there, your own gear counted ("WEARS PICK 3, SHOVEL 3.", 1.5 each with a whetstone). Nothing for a town workshop's own tools. Built.
- When a tool wears out, the bubble names it and something that lasts longer that grows or is made here: "THE ELDER SPOOL WORE OUT. RHODODENDRON LASTS LONGER: DURABILITY 100, ELDER 50." Built.
- The world says it too: the smith in Verdigris talks of steel holding an edge, and the assay office in Bellows, which takes ingots, ranks the metals on its card. With the hills.

**Cut from the first draft** (2026-11-05): lead ore, the bellows and the crucible (the smelter and foundry have them in their art instead). The iron forceps and the tin basket were cut and then kept.

**Left out:** gold, brass and zinc, silver's two furnaces as two goods, and the great anchor. Mixed metals that make no alloy (a bell of copper and pig iron) need a name rule when this is built.

## 3. The Kiln Banks
A river valley with clay banks of five colors, reed beds and resinous pines. The book's chapter on ceramics.
- **Dug or gathered:** red, yellow, blue, white and black clay T; sand N, rushes N, ochre N, rosin N; pyrolusite N (perhaps from the hills); climbing hempvine T and willow + grow here.
- **Building:** bricks T, clay tiles T, mortar N; palace bricks T and palace tiles T (prestige goods: yellow clay with pyrolusite, palm leaves from the coast, ochre, rosin, rushes, fuel).
- **Pottery and porcelain:** porcelain clay paste T, ceramic cup T, ash glaze T, palace glaze T, decorated ceramic T; tools: potter's wheel N, brick frames N, tile bow and cylinder N, brush N, earthenware funnel N.
- **Molds for casting:** pot mold T, sycee mold T, sand mold T (which is the mirror and die molds: it takes its pattern's name); palace brick and tile molds N.
- **Boast:** the finest porcelain in the world is fired here, and it really is.
- **Expatriate industry:** a few mulberry terraces and silkworm houses on the river, the plains' silk in miniature.
- **Sends out:** bricks, tiles and mortar (building everywhere), molds for the hills and the foundry, cups. **Needs from elsewhere:** fuel, pyrolusite, palm leaves, patterns from the cloth reaches.

## 4. The Paddy Plains
Wet plains of paddies, wheat and millet fields, mulberry groves, orchards and lotus ponds. The book's chapters on grain, preparing grain, oils, yeasts and silk.
- **Harvested:** rice, irrigated rice, hungry rice, wheat, barley, dark barley, sorghum, little millet, sesame T (all need sickles or scythes from the hills); soybeans N, mulberry leaves N, beeswax N, plums T, lotus blossoms T, safflower blossoms T; velvetleaf T, red clay T, elm + and jujube + grow here.
- **Grain:** flail T, whole grain T, hulled grain T, flour T (two ways), malt T; tools: barrel N, huller N, winnower N, miller N.
- **Oil:** pressed oil N, from oily seeds: sesame, safflower seeds, and the cloth reaches' cotton seeds.
- **Silk:** silkworm cocoons T (mulberry leaves, salt, charcoal, writing material N; baskets and braziers N), silk thread T.
- **Reds:** safflower seeds T, safflower cakes T, crimson dye T (with plums and lye), pink dye T, tea brown dye T (lotus with green vitriol).
- **Boast:** the best silkworms are raised here.
- **Expatriate industry:** a village foundry casting cooking pots from ingots brought down from the hills.
- **Sends out:** grain and flour, oil, beeswax, silk thread, red dyes. **Needs from elsewhere:** sickles and scythes, salt and charcoal for the silkworms, lye.

## 5. The Salt Coast
A warm coast of salt flats, cane fields, palms and dye-wood forests. The book's chapters on salt and sugar.
- **Gathered:** sea salt T (needs a strong pan N), white and brown sugar cane T (need billhooks from the hills), oyster lime T, gallnuts T; palm leaves N, sappanwood N, berberine wood N, smoke tree wood N, bayberry wood N; kudzu T, camphor + and palm + grow here.
- **Sugar:** sugar T (roll crusher, barrel, earthenware funnel), shaped candy T (with soybeans from the plains, in a candy mold N).
- **Dye woods:** wood red, purple, canary yellow, golden yellow, bright green, deep sky blue, eggshell blue and black dyes T; royal blue dyed fabric T (deep sky blue fabric, soybeans, indigo, pitch N).
- **Boast:** only this coast knows how to make sugar.
- **Expatriate industry:** potters making the earthenware funnels that sugar needs.
- **Sends out:** salt, sugar, the many colors, palm leaves, oyster lime. **Needs from elsewhere:** fabric to dye, billhooks, mordants, barrels.

## The great works
All three are cast at a foundry, and each needs most of the provinces. Their recipes stay as Henry wrote them.

| Great work | Its inputs, by province |
| :-- | :-- |
| **Temple Bell** | lime (1); clay and mortar (3); beeswax and oil (4); charcoal, coke, two metal ingots, a chisel and a shovel (2); fuel (anywhere) |
| **Great Metal Statue** | the bell's inputs twice over, plus statue schematics N |
| **Large Mirror** | a mirror mold (3, from a pattern made in 1), a silver ingot, a bronze ingot and coke (2) |

**The bell in 1.0:** the first province's third town, the temple's, sits where the cloth reaches meet the hills. Its expatriate works give copper, clay and beeswax; its oil can be pressed from cotton seeds. A temporary vendor covers the rest (tin above all: Tinless was named for this) until the hills open.

## Still open
- **The boasts** are from my memory of the book; check them against Henry's notes.
- **Expatriate picks** and **new species** above are first suggestions.

## Data fixes found while placing everything
- Seeth water asks for "idigo leaves" (should be indigo leaves).
- Shaped candy asks for "soy beans" and royal blue fabric for "soybeans": one good, two spellings.
- The palace bricks and tiles ask for clay tagged "yellow", but yellow clay is tagged "yellow clay", so nothing matches.
- Two tools were split into letters (climbing hempvine's trellis, found green vitriol's cauldron). Fixed 2026-11-05, and a test keeps any tag from being one letter. Nothing makes a trellis yet: the hemp batch.
- Wire and nails need a die to draw through, and the wooden die pattern is tagged "die" too, so a pattern would do. They should ask for a metal die. (Found in the tool pass, 2026-11-05; fixed with the metal batch. The die itself is cast in a sand mold made on the die pattern, which carries the pattern's name.)
- Malt, both glazes and the porcelain paste need a pot, and the ceramic pot mold is tagged "pot" too, so a mold would do. The mold could be tagged "pot mold" instead, with the metal pot asking for that.
