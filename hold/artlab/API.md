# The art interface Naibihpma Hold v1's game code calls

`src/art.rs` must provide exactly these public items (names, variants and signatures), plus everything
the current file already exports (Sprite and its methods, grid, every existing palette constant at
its existing index, rgb, POINTER_HOT). New palette entries go at indices 89 and up (at most 255).

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Look { Newt, Caecilian, TreeFrog, Salamander, ChorusToad, Toad, Axolotl, Bullfrog,
                Goliath, Olm, Daot, Heron, Snake, Crow, Wisp, Turtle, Gorf }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Age { Young, Adult, Old }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Pose { Walk0, Walk1, Work0, Work1, Strike, Fly0, Fly1, DugIn, Dream }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BLook { Hall, Burrow, TwigLoft, MorelCave, FairyRing, ChorusLog, EmberLog, IroncapCave,
                 ToadHall, ShellForge, Cauldron, StarPond, NewtHouse, SpawningPool, DaotsGrotto,
                 OlmsCave, Nest, LoneNest, GorfStone, LeechPool, CrowRoost, GorfHollow }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SpotLook { Fern, RedFungus, WhiteFungus, HybridFungus, Puffball }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TileLook { Water, Deep, Moss, Fern, Mud, Path, Sand, Rock, Reeds, Cursed, Puddle }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Deco { BigTree, Tree, Stump, Stone, LeyStone, Lily, ReedClump, Mushroom, Log }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ItemLook { Dew, Fungiwood, CapIngot, SpottedCap, GlowSpores, SunOrb, ClargentOrb,
                    MeteorOrb, WoodenShovel, IronShovel, VenomwoodShovel, Slingshot, Catapult,
                    IronSpear, VenomwoodSpear, VenomironSpear, Chanfron, Peytral, Croupiere,
                    VenomironChanfron, DewBead, GlowCharm, StarPearl }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CondLook { Dry, Poisoned, Blinded, Bolstered, Regenerating, Altered, Dreaming, Leeched,
                    Frightened }
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum UiLook { Pause, Play, Fast, Magic, Screens, Attack, Explore, Guard, Fetch, RainCall,
                  OldSong, Starlight, LeyLight, DeepCalling }

/// What the sky is doing; the palette remaps only the *background* ramps for it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Sky { pub dark: u8 /* 0 day, 1 dusk or dawn, 2 night, 3 deep night */,
                 pub rain: bool, pub dry: bool }

/// The 256-entry palette at tick `t` (for the slow glint/spore cycling) under `sky`,
/// every entry at 6-bit DAC precision (`& 0xFCFCFC`).
pub fn palette(t: u32, sky: Sky) -> [u32; 256];

/// Footprint in map squares (16 px each), width x height.
pub fn footprint(b: BLook) -> (i32, i32);

impl Art {
    pub fn new() -> Art;
    pub fn pointer(&self) -> &Sprite;                       // the mouse pointer (hot spot POINTER_HOT)
    pub fn creature(&self, look: Look, age: Age, pose: Pose, left: bool) -> &Sprite;
    pub fn building(&self, b: BLook) -> &Sprite;            // finished building
    pub fn site(&self, w: i32, h: i32) -> &Sprite;          // a building site of that footprint
    pub fn ruin(&self, w: i32, h: i32) -> &Sprite;          // rubble of that footprint
    pub fn spot(&self, s: SpotLook, stage: u8) -> &Sprite;  // stage 0 bare, 1 sprouting, 2 ripening, 3 ripe
    pub fn tile(&self, t: TileLook, variant: u8) -> &Sprite;// exactly 16 x 16; any variant number is fine
    pub fn deco(&self, d: Deco) -> &Sprite;
    pub fn item(&self, i: ItemLook) -> &Sprite;             // about 7 x 7, outlined
    pub fn cond(&self, c: CondLook) -> &Sprite;             // about 5 x 5
    pub fn ui(&self, u: UiLook) -> &Sprite;                 // exactly 11 x 11, for 13 x 13 buttons
    pub fn husk(&self) -> &Sprite;                          // a fallen amphibian's empty skin
    pub fn eggs(&self, stage: u8) -> &Sprite;               // 0 eggs, 1 tadpole, 2 froglet, in a pool
    pub fn flag(&self) -> &Sprite;                          // a posted reward's flag
}
```

Footprints (squares): Hall 4x3; Burrow, TwigLoft, MorelCave, FairyRing, EmberLog, IroncapCave,
Cauldron, NewtHouse, Nest, LoneNest, LeechPool, CrowRoost 2x2; ChorusLog, ToadHall, ShellForge,
DaotsGrotto, OlmsCave 3x2; StarPond, SpawningPool 3x3; GorfStone 1x2; GorfHollow 4x3.
