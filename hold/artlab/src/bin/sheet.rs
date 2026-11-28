//! Contact sheet: every sprite on one indexed canvas, labelled, at day (left) and night (right)
//! palettes, written as PPM at 3x (`sheet.ppm`). Also writes section sheets at 4x and a small marsh
//! scene under each sky (`scene_*.ppm`); add `zoom` for close-ups of every creature and building.
//!
//! Run: `cargo run --release --bin sheet [out_dir] [zoom]` (out_dir defaults to `target/sheet`),
//! then `sips -s format png sheet.ppm --out sheet.png` to view.

use artlab::art::*;
use artlab::fb::Fb;
use std::fs;
use std::io::Write;

/// An indexed canvas.
struct Canvas {
    w: i32,
    h: i32,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: i32, h: i32, bg: u8) -> Canvas {
        Canvas {
            w,
            h,
            px: vec![bg; (w * h) as usize],
        }
    }

    fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    fn blit(&mut self, s: &Sprite, x: i32, y: i32) {
        for sy in 0..s.h {
            for sx in 0..s.w {
                let c = s.get(sx, sy);
                if c != 0 {
                    self.set(x + sx, y + sy, c);
                }
            }
        }
    }

    fn text(&mut self, s: &str, x: i32, y: i32, col: u8) {
        let mut fb = Fb::default();
        fb.clear(0);
        fb.text(s, 0, 0, col, 1);
        let w = Fb::text_width(s, 1).min(399);
        for yy in 0..7 {
            for xx in 0..=w {
                let c = fb.get(xx, yy);
                if c != 0 {
                    self.set(x + xx, y + yy, c);
                }
            }
        }
    }

    fn paste(&mut self, o: &Canvas, x: i32, y: i32) {
        for yy in 0..o.h {
            for xx in 0..o.w {
                self.set(x + xx, y + yy, o.px[(yy * o.w + xx) as usize]);
            }
        }
    }
}

const BG: u8 = BTN;
const BAR: u8 = PANEL_HI;

/// Lays out labelled cells left to right, wrapping at the width.
struct Flow {
    c: Canvas,
    x: i32,
    y: i32,
    row_h: i32,
}

impl Flow {
    fn new(w: i32, title: &str) -> Flow {
        let mut c = Canvas::new(w, 2000, BG);
        c.rect(0, 0, w, 11, BAR);
        c.text(title, 4, 2, TEXT);
        Flow {
            c,
            x: 4,
            y: 15,
            row_h: 0,
        }
    }

    fn newline(&mut self) {
        if self.x > 4 {
            self.y += self.row_h + 4;
        }
        self.x = 4;
        self.row_h = 0;
    }

    fn label(&mut self, s: &str) {
        self.newline();
        self.c.text(s, 4, self.y, KEY);
        self.y += 10;
    }

    /// Place sprites side by side as one cell, bottom-aligned, with a caption underneath.
    fn cell(&mut self, sprites: &[&Sprite], caption: &str) {
        let gap = 2;
        let sw: i32 = sprites.iter().map(|s| s.w).sum::<i32>() + gap * (sprites.len() as i32 - 1);
        let sh = sprites.iter().map(|s| s.h).max().unwrap_or(0);
        let cw = sw.max(Fb::text_width(caption, 1)) + 6;
        let ch = sh + if caption.is_empty() { 2 } else { 10 };
        if self.x + cw > self.c.w - 2 {
            self.newline();
        }
        let mut x = self.x;
        for s in sprites {
            self.c.blit(s, x, self.y + sh - s.h);
            x += s.w + gap;
        }
        if !caption.is_empty() {
            self.c.text(caption, self.x, self.y + sh + 2, TEXT_DIM);
        }
        self.x += cw;
        self.row_h = self.row_h.max(ch);
    }

    fn finish(mut self) -> Canvas {
        self.newline();
        let h = self.y + 2;
        let mut out = Canvas::new(self.c.w, h, BG);
        for y in 0..h {
            for x in 0..self.c.w {
                out.px[(y * out.w + x) as usize] = self.c.px[(y * self.c.w + x) as usize];
            }
        }
        out
    }
}

fn amphibians(a: &Art, w: i32) -> Canvas {
    let mut f = Flow::new(
        w,
        "AMPHIBIANS: WALK0 WALK1 WORK0 WORK1 STRIKE / DUGIN DREAM",
    );
    let looks = [
        Look::Newt,
        Look::Caecilian,
        Look::TreeFrog,
        Look::Salamander,
        Look::ChorusToad,
        Look::Toad,
        Look::Axolotl,
        Look::Bullfrog,
        Look::Goliath,
        Look::Olm,
        Look::Daot,
    ];
    for look in looks {
        f.label(&format!("{look:?}"));
        for age in Age::ALL {
            let poses: Vec<&Sprite> = [
                Pose::Walk0,
                Pose::Walk1,
                Pose::Work0,
                Pose::Work1,
                Pose::Strike,
            ]
            .iter()
            .map(|&p| a.creature(look, age, p, false))
            .collect();
            f.cell(&poses, &format!("{age:?}"));
            f.cell(
                &[
                    a.creature(look, age, Pose::DugIn, false),
                    a.creature(look, age, Pose::Dream, false),
                ],
                "",
            );
        }
        f.cell(&[a.creature(look, Age::Adult, Pose::Walk0, true)], "LEFT");
    }
    f.finish()
}

fn enemies(a: &Art, w: i32) -> Canvas {
    let mut f = Flow::new(w, "ENEMIES: WALK0 WALK1 WORK0 STRIKE FLY0 FLY1 DUGIN");
    for look in [
        Look::Heron,
        Look::Snake,
        Look::Crow,
        Look::Wisp,
        Look::Turtle,
        Look::Gorf,
    ] {
        let poses: Vec<&Sprite> = [
            Pose::Walk0,
            Pose::Walk1,
            Pose::Work0,
            Pose::Strike,
            Pose::Fly0,
            Pose::Fly1,
            Pose::DugIn,
        ]
        .iter()
        .map(|&p| a.creature(look, Age::Adult, p, false))
        .collect();
        f.cell(&poses, &format!("{look:?}"));
    }
    f.finish()
}

fn buildings(a: &Art, w: i32) -> Canvas {
    let mut f = Flow::new(w, "BUILDINGS AND LAIRS");
    for b in BLook::ALL {
        let (fw, fh) = footprint(b);
        f.cell(&[a.building(b)], &format!("{b:?} {fw}X{fh}"));
    }
    f.label("SITES AND RUINS");
    for (w, h) in [(1, 2), (2, 2), (3, 2), (3, 3), (4, 3)] {
        f.cell(&[a.site(w, h), a.ruin(w, h)], &format!("{w}X{h}"));
    }
    f.finish()
}

fn small(a: &Art, w: i32) -> Canvas {
    let mut f = Flow::new(w, "SPOTS, DECORATIONS, ICONS");
    for s in SpotLook::ALL {
        let st: Vec<&Sprite> = (0..4).map(|k| a.spot(s, k)).collect();
        f.cell(&st, &format!("{s:?}"));
    }
    f.label("DECORATIONS");
    for d in Deco::ALL {
        f.cell(&[a.deco(d)], &format!("{d:?}"));
    }
    f.label("ITEMS");
    for i in ItemLook::ALL {
        f.cell(
            &[a.item(i)],
            &format!("{i:?}").chars().take(8).collect::<String>(),
        );
    }
    f.label("CONDITIONS");
    for c in CondLook::ALL {
        f.cell(
            &[a.cond(c)],
            &format!("{c:?}").chars().take(6).collect::<String>(),
        );
    }
    f.label("UI ICONS ON BUTTONS");
    let mut buttons = vec![];
    for u in UiLook::ALL {
        let mut b = Sprite::new(13, 13);
        b.rect(0, 0, 13, 13, INK);
        b.rect(1, 1, 11, 11, BTN_HI);
        b.rect(2, 2, 10, 10, BTN_LO);
        b.rect(2, 2, 9, 9, BTN);
        b.blit(a.ui(u), 1, 1);
        buttons.push((b, u));
    }
    for (b, u) in &buttons {
        f.cell(&[b], &format!("{u:?}").chars().take(7).collect::<String>());
    }
    f.label("MISC");
    f.cell(&[a.husk()], "HUSK");
    f.cell(&[a.eggs(0), a.eggs(1), a.eggs(2)], "EGGS");
    f.cell(&[a.flag()], "FLAG");
    f.cell(&[a.pointer()], "POINTER");
    f.finish()
}

fn tiles(a: &Art, w: i32) -> Canvas {
    let mut f = Flow::new(w, "TILES: 8 VARIANTS, THEN A 4X3 PATCH OF MIXED VARIANTS");
    for t in TileLook::ALL {
        let mut vs = Sprite::new(4 * 17 - 1, 2 * 17 - 1);
        for v in 0..TILE_VARIANTS as i32 {
            vs.blit(a.tile(t, v as u8), (v % 4) * 17, (v / 4) * 17);
        }
        let mut patch = Sprite::new(64, 48);
        for ty in 0..3 {
            for tx in 0..4 {
                let v = variant(tx as i32, ty as i32);
                patch.blit(a.tile(t, v), tx * 16, ty * 16);
            }
        }
        f.cell(&[&vs, &patch], &format!("{t:?}"));
    }
    f.finish()
}

/// A small marsh scene: tiles, decorations, buildings and folk together.
fn scene(a: &Art) -> Canvas {
    use TileLook::*;
    let map = [
        "rrrrrmmmmmmmwwwwmmmfffmmm",
        "rrrrmmmmmmmwwwwwmmmffmmmm",
        "rrmmmmMMmmwwDDwwmmmmmmmpp",
        "rmmmmMMMmmwDDDDwRRmmmpppp",
        "mmmmmmMmmwwDDDwwRmmpppsss",
        "fmmmmmmmmwwwDwwmmmppssssc",
        "ffmmmmmmmmwwwwRmmppsssscc",
        "fffmmmmmmmmwwwmmpppsssccc",
        "ffmmmmudmmmmwwmmppmmsccccc",
        "fmmmmuuddmmmmmmmppmmmcccc",
        "mmmmmmuummmmmmmppmmmmmccc",
        "mmmmmmmmmmmmmmppmmmmmmmcc",
        "mmmmmmmmmmmmppmmmmmmmmmmm",
        "mmmmmmmmmmmpppmmmmmmmmmmm",
        "mmmmmmmmmmppmmmmmmmmmmmmm",
    ];
    let mut c = Canvas::new(400, 240, BG);
    for (ty, row) in map.iter().enumerate() {
        for (tx, ch) in row.chars().take(25).enumerate() {
            let t = match ch {
                'r' => Rock,
                'w' => Water,
                'D' => Deep,
                'f' => Fern,
                'M' => Mud,
                'u' => Puddle,
                'd' => Mud,
                'p' => Path,
                's' => Sand,
                'c' => Cursed,
                'R' => Reeds,
                _ => Moss,
            };
            let v = variant(tx as i32, ty as i32);
            c.blit(a.tile(t, v), tx as i32 * 16, ty as i32 * 16);
        }
    }
    let put = |c: &mut Canvas, s: &Sprite, x: i32, y: i32| c.blit(s, x - s.w / 2, y - s.h);
    put(&mut c, a.deco(Deco::BigTree), 22, 70);
    put(&mut c, a.deco(Deco::Tree), 58, 44);
    put(&mut c, a.deco(Deco::Lily), 185, 40);
    put(&mut c, a.deco(Deco::Lily), 210, 104);
    put(&mut c, a.deco(Deco::ReedClump), 268, 64);
    put(&mut c, a.deco(Deco::Stone), 140, 150);
    put(&mut c, a.deco(Deco::LeyStone), 330, 40);
    put(&mut c, a.deco(Deco::Stump), 30, 200);
    put(&mut c, a.deco(Deco::Mushroom), 120, 60);
    put(&mut c, a.deco(Deco::Log), 300, 150);
    // Buildings, bottom-aligned to their footprints.
    let bld = |c: &mut Canvas, b: BLook, tx: i32, ty: i32| {
        let s = a.building(b);
        let (_, fh) = footprint(b);
        c.blit(s, tx * 16, (ty + fh) * 16 - s.h);
    };
    bld(&mut c, BLook::Hall, 4, 9);
    bld(&mut c, BLook::Burrow, 9, 10);
    bld(&mut c, BLook::NewtHouse, 1, 12);
    bld(&mut c, BLook::ToadHall, 17, 12);
    bld(&mut c, BLook::FairyRing, 12, 5);
    bld(&mut c, BLook::GorfStone, 23, 7);
    bld(&mut c, BLook::Nest, 12, 0);
    c.blit(a.spot(SpotLook::Fern, 3), 0, 6 * 16);
    c.blit(a.spot(SpotLook::Puffball, 3), 16, 7 * 16);
    let cr = |c: &mut Canvas, l: Look, age: Age, p: Pose, left: bool, x: i32, y: i32| {
        let s = a.creature(l, age, p, left);
        c.blit(s, x - s.w / 2, y - s.h);
    };
    cr(&mut c, Look::Newt, Age::Adult, Pose::Walk0, false, 150, 176);
    cr(&mut c, Look::Newt, Age::Young, Pose::Walk1, true, 168, 182);
    cr(&mut c, Look::Toad, Age::Adult, Pose::Walk0, false, 250, 200);
    cr(&mut c, Look::Toad, Age::Old, Pose::Strike, true, 290, 214);
    cr(
        &mut c,
        Look::TreeFrog,
        Age::Adult,
        Pose::Work0,
        false,
        100,
        210,
    );
    cr(
        &mut c,
        Look::Caecilian,
        Age::Adult,
        Pose::Walk0,
        false,
        175,
        150,
    );
    cr(
        &mut c,
        Look::Axolotl,
        Age::Adult,
        Pose::Walk0,
        true,
        222,
        150,
    );
    cr(&mut c, Look::Heron, Age::Adult, Pose::Walk0, true, 205, 62);
    cr(&mut c, Look::Wisp, Age::Adult, Pose::Fly0, false, 345, 110);
    cr(&mut c, Look::Snake, Age::Adult, Pose::Walk0, true, 340, 190);
    cr(&mut c, Look::Crow, Age::Adult, Pose::Fly0, false, 300, 30);
    cr(
        &mut c,
        Look::Bullfrog,
        Age::Adult,
        Pose::Walk0,
        false,
        60,
        238,
    );
    c.blit(a.flag(), 120, 120);
    c
}

/// A tile variant picked by a position hash, so features never line up in rows.
fn variant(x: i32, y: i32) -> u8 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    ((h >> 8) % TILE_VARIANTS as u32) as u8
}

fn write_ppm(path: &str, c: &Canvas, pals: &[[u32; 256]], scale: i32) {
    let n = pals.len() as i32;
    let gap = 4;
    let (w, h) = ((c.w * n + gap * (n - 1)) * scale, c.h * scale);
    let mut out = Vec::with_capacity((w * h * 3) as usize + 32);
    write!(out, "P6\n{w} {h}\n255\n").unwrap();
    for y in 0..h {
        for x in 0..w {
            let lx = x / scale;
            let panel = lx / (c.w + gap);
            let px = lx % (c.w + gap);
            let rgb = if px >= c.w {
                0
            } else {
                pals[panel as usize][c.px[((y / scale) * c.w + px) as usize] as usize]
            };
            out.extend_from_slice(&[(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]);
        }
    }
    fs::write(path, out).unwrap();
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/sheet".into());
    fs::create_dir_all(&dir).unwrap();
    let a = Art::new();
    let day = palette(0, Sky::default());
    let night = palette(
        15,
        Sky {
            dark: 2,
            rain: false,
            dry: false,
        },
    );
    let w = 430;
    let sections = [
        ("creatures", amphibians(&a, w)),
        ("enemies", enemies(&a, w)),
        ("buildings", buildings(&a, w)),
        ("tiles", tiles(&a, w)),
        ("small", small(&a, w)),
    ];
    let total_h: i32 = sections.iter().map(|s| s.1.h + 4).sum();
    let mut full = Canvas::new(w, total_h, BG);
    let mut y = 0;
    for (name, s) in &sections {
        full.paste(s, 0, y);
        y += s.h + 4;
        write_ppm(&format!("{dir}/sec_{name}.ppm"), s, &[day], 4);
        write_ppm(&format!("{dir}/sec_{name}_night.ppm"), s, &[night], 4);
    }
    write_ppm(&format!("{dir}/sheet.ppm"), &full, &[day, night], 3);
    let sc = scene(&a);
    let skies = [
        ("day", Sky::default()),
        (
            "dusk",
            Sky {
                dark: 1,
                ..Sky::default()
            },
        ),
        (
            "night",
            Sky {
                dark: 2,
                ..Sky::default()
            },
        ),
        (
            "deep",
            Sky {
                dark: 3,
                ..Sky::default()
            },
        ),
        (
            "rain",
            Sky {
                rain: true,
                ..Sky::default()
            },
        ),
        (
            "dry",
            Sky {
                dry: true,
                ..Sky::default()
            },
        ),
    ];
    for (name, sky) in skies {
        write_ppm(
            &format!("{dir}/scene_{name}.ppm"),
            &sc,
            &[palette(15, sky)],
            3,
        );
    }
    if std::env::args().nth(2).as_deref() == Some("zoom") {
        zooms(&a, &dir, &day, &night);
    }
    println!("wrote {dir}/sheet.ppm and sections");
}

/// Close-up sheets at 8x for checking pixels.
fn zooms(a: &Art, dir: &str, day: &[u32; 256], night: &[u32; 256]) {
    let creature_rows = |f: &mut Flow, looks: &[Look], ages: &[Age]| {
        for &look in looks {
            for &age in ages {
                let poses: Vec<&Sprite> = Pose::ALL
                    .iter()
                    .map(|&p| a.creature(look, age, p, false))
                    .collect();
                f.cell(&poses, &format!("{look:?} {age:?}"));
            }
        }
    };
    for look in Look::ALL {
        let ages: &[Age] = if look.is_enemy() {
            &[Age::Adult]
        } else {
            &Age::ALL
        };
        let wmax = ages
            .iter()
            .map(|&g| {
                Pose::ALL
                    .iter()
                    .map(|&p| a.creature(look, g, p, false).w + 8)
                    .sum::<i32>()
            })
            .max()
            .unwrap();
        let mut f = Flow::new(wmax + 8, &format!("{look:?}"));
        for &age in ages {
            let poses: Vec<&Sprite> = Pose::ALL
                .iter()
                .map(|&p| a.creature(look, age, p, false))
                .collect();
            for p in poses {
                f.cell(&[p], "");
            }
            f.newline();
        }
        let scale = (1400 / f.c.w).clamp(4, 12);
        write_ppm(
            &format!("{dir}/c_{look:?}.ppm"),
            &f.finish(),
            &[*day],
            scale,
        );
    }
    let mut f = Flow::new(150, "WALK0: YOUNG ADULT OLD");
    for look in Look::ALL.iter().filter(|l| !l.is_enemy()) {
        let v: Vec<&Sprite> = Age::ALL
            .iter()
            .map(|&g| a.creature(*look, g, Pose::Walk0, false))
            .collect();
        f.cell(&v, "");
    }
    write_ppm(&format!("{dir}/z_walk.ppm"), &f.finish(), &[*day], 9);
    for (k, chunk) in BLook::ALL.chunks(6).enumerate() {
        let mut f = Flow::new(240, "BUILDINGS");
        for &b in chunk {
            f.cell(&[a.building(b)], "");
        }
        write_ppm(&format!("{dir}/b_{k}.ppm"), &f.finish(), &[*day], 6);
    }
    let mut f = Flow::new(190, "FOLK");
    creature_rows(
        &mut f,
        &[
            Look::Newt,
            Look::Caecilian,
            Look::TreeFrog,
            Look::Salamander,
        ],
        &Age::ALL,
    );
    write_ppm(&format!("{dir}/z_folk.ppm"), &f.finish(), &[*day], 8);
    let mut f = Flow::new(190, "HEROES");
    creature_rows(&mut f, &[Look::Toad, Look::Axolotl, Look::Olm], &Age::ALL);
    write_ppm(&format!("{dir}/z_heroes.ppm"), &f.finish(), &[*day], 8);
    let mut f = Flow::new(190, "TOADS");
    creature_rows(&mut f, &[Look::ChorusToad, Look::Daot], &Age::ALL);
    write_ppm(&format!("{dir}/z_toads.ppm"), &f.finish(), &[*day], 8);
    let mut f = Flow::new(230, "BIG");
    creature_rows(&mut f, &[Look::Bullfrog, Look::Goliath], &Age::ALL);
    write_ppm(&format!("{dir}/z_big.ppm"), &f.finish(), &[*day], 6);
    let mut f = Flow::new(230, "ENEMIES");
    creature_rows(
        &mut f,
        &[
            Look::Heron,
            Look::Snake,
            Look::Crow,
            Look::Wisp,
            Look::Turtle,
            Look::Gorf,
        ],
        &[Age::Adult],
    );
    write_ppm(&format!("{dir}/z_enemies.ppm"), &f.finish(), &[*day], 6);
    let mut f = Flow::new(190, "ICONS");
    for i in ItemLook::ALL {
        f.cell(&[a.item(i)], "");
    }
    f.newline();
    for c in CondLook::ALL {
        f.cell(&[a.cond(c)], "");
    }
    f.newline();
    for u in UiLook::ALL {
        f.cell(&[a.ui(u)], "");
    }
    f.newline();
    for s in SpotLook::ALL {
        let st: Vec<&Sprite> = (0..4).map(|k| a.spot(s, k)).collect();
        f.cell(&st, "");
    }
    f.cell(&[a.husk(), a.eggs(0), a.eggs(1), a.eggs(2)], "");
    write_ppm(&format!("{dir}/z_icons.ppm"), &f.finish(), &[*day], 8);
    let mut f = Flow::new(200, "DECOS");
    for d in Deco::ALL {
        f.cell(&[a.deco(d)], "");
    }
    write_ppm(
        &format!("{dir}/z_decos.ppm"),
        &f.finish(),
        &[*day, *night],
        5,
    );
}
