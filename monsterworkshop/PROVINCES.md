# Provinces and the tech tree (sketch, 2026-11-02)

The living world's five provinces, with every good in Henry's original tech tree placed in one of them. Five provinces and the principles below are agreed; the placements, boasts, expatriate industries and new varieties are proposals. Names are working names, not player-facing.

**Where the tree lives:** `data/good_types.json` holds all 105 goods of the second tree (`django/.../commands/tech_tree_two.py`) and 17 we've added. The first tree (`tech_tree.py`) also defines copper ore, the ingot smelted from it at a smelter, and primitive tongs, which aren't in the game's data yet. The second tree names about 60 more goods as inputs or tools without defining them (metal ingot, charcoal, beeswax, oil, mortar, forceps, shovel, sickle...).

**Marks:** ✓ in the game now · **A** added by us and approved, in the data, not in play yet · **T** defined in an original tree, not in play yet · **N** named by the tree as an input or tool, never defined · **+** in neither tree.

**Approval by mark** (agreed): T needs only a yes on where it's made or grows; N gets its recipe proposed from the book, in batches; + needs Henry's explicit approval, one good at a time.

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

## 3. The Kiln Banks
A river valley with clay banks of five colors, reed beds and resinous pines. The book's chapter on ceramics.
- **Dug or gathered:** red, yellow, blue, white and black clay T; sand N, rushes N, ochre N, rosin N; pyrolusite N (perhaps from the hills); climbing hempvine T and willow + grow here.
- **Building:** bricks T, clay tiles T, mortar N; palace bricks T and palace tiles T (prestige goods: yellow clay with pyrolusite, palm leaves from the coast, ochre, rosin, rushes, fuel).
- **Pottery and porcelain:** porcelain clay paste T, ceramic cup T, ash glaze T, palace glaze T, decorated ceramic T; tools: potter's wheel N, brick frames N, tile bow and cylinder N, brush N, earthenware funnel N.
- **Molds for casting:** pot mold T, sycee mold T, sand mold T; mirror mold N, die mold N, palace brick and tile molds N.
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
- Two tools are split into letters (climbing hempvine's trellis, found green vitriol's cauldron), as noted under Tech tree gaps.
