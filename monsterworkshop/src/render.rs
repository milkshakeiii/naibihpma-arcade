use crate::framebuffer::Framebuffer;
use crate::game::{Game, Species};
use crate::palette::*;
use crate::sprites::{Sprite, Sprites};
use crate::tutorial::{self, wrap, HELP, STAGES};
use crate::world::*;

pub const VIEW_H: i32 = MAP_H * TS; // 256; the HUD fills the remaining 44 rows
const HOP: [i32; 7] = [0, 2, 4, 5, 5, 4, 2];

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^ (h >> 16)
}

pub fn draw(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            draw_tile(g, sp, fb, tx, ty);
        }
    }
    fb.text_centered("SPINNERY", 13 * TS + 8, 5 * TS + 5, WOOD + 3, Some(OUTLINE), 1);
    for p in &g.world.plants {
        fb.blit(&sp.plant[p.stage() as usize], p.x * TS, p.y * TS, false);
    }
    let (wx, wy) = g.world.shop.wheel;
    let spinning = g.world.shop.crafting.is_some();
    fb.blit(&sp.wheel[if spinning { (g.frame / 2 % 2) as usize } else { 0 }], wx * TS, wy * TS - 5, false);
    for b in &g.world.bins {
        draw_bin(sp, fb, b);
    }
    draw_flags(g, sp, fb);
    draw_bodies(g, sp, fb);
    draw_particles(g, fb);
    draw_markers(g, sp, fb);
    // Lowest priority first, so the controlled monster's bubble ends up on top.
    for b in layout_bubbles(g).iter().rev() {
        draw_bubble(fb, sp, b);
    }
    if let Some((_, t)) = g.world.shop.crafting {
        let (x, y) = (wx * TS - 1, wy * TS - 13);
        let fill = (t as i32 * 16 / CRAFT_TICKS as i32).max(1);
        fb.rect_fill(x, y, 18, 5, OUTLINE);
        fb.rect_fill(x + 1, y + 1, 16, 3, HUD_SHADOW);
        fb.rect_fill(x + 1, y + 1, fill, 3, SPARK + 1);
        fb.rect_fill(x + 1, y + 1, fill, 1, SPARK + 2);
    }
    draw_hud(g, sp, fb);
    if g.help {
        draw_help(fb);
    }
}

fn draw_tile(g: &Game, sp: &Sprites, fb: &mut Framebuffer, tx: i32, ty: i32) {
    let (x0, y0) = (tx * TS, ty * TS);
    let tile = g.world.tile(tx, ty);
    let ground = match tile {
        Tile::Bin => g.world.tile(tx, ty + 1),
        t => t,
    };
    for y in 0..TS {
        for x in 0..TS {
            let h = hash(x0 + x, y0 + y);
            let c = match ground {
                Tile::Grass | Tile::Bin => match h % 23 {
                    0 => GRASS + 3,
                    1 | 2 => GRASS + 2,
                    3 => GRASS,
                    _ => GRASS + 1,
                },
                Tile::Path => match h % 17 {
                    0 => PATH + 2,
                    1 | 2 => PATH,
                    _ => PATH + 1,
                },
                Tile::Hedge => {
                    let lit = (x + y + (h % 3) as i32) % 5 == 0;
                    if y < 2 && g.world.tile(tx, ty - 1) != Tile::Hedge {
                        HEDGE + 2
                    } else if lit {
                        HEDGE + 1
                    } else if h % 7 == 0 {
                        HEDGE + 2
                    } else {
                        HEDGE
                    }
                }
                Tile::Wall => {
                    if y == 0 {
                        WOOD + 3
                    } else if y == TS - 1 {
                        WOOD
                    } else if (x + if ty % 2 == 0 { 0 } else { 4 }) % 8 == 0 {
                        WOOD
                    } else {
                        WOOD + 1 + (h % 9 == 0) as u8
                    }
                }
                _ => {
                    // Planks: a seam every 4 rows with staggered joints.
                    let row = y / 4;
                    if y % 4 == 3 {
                        PLANK
                    } else if (x + row * 5 + tx * 3) % 16 == 0 {
                        PLANK
                    } else if y % 4 == 0 {
                        PLANK + 2
                    } else {
                        PLANK + 1 + (h % 13 == 0) as u8
                    }
                }
            };
            fb.pset(x0 + x, y0 + y, c);
        }
    }
    if ground == Tile::Grass && tile == Tile::Grass && hash(tx, ty) % 7 == 0 {
        let (fx, fy) = (x0 + 3 + (hash(ty, tx) % 9) as i32, y0 + 4 + (hash(tx + 7, ty) % 8) as i32);
        let col = FLOWER + (hash(tx, ty + 3) % 2) as u8;
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            fb.pset(fx + dx, fy + dy, col);
        }
        fb.pset(fx, fy, SPARK + 2);
    }
    if tile == Tile::Wall {
        for (dx, dy, edge) in [(0, -1, 0), (0, 1, 1), (-1, 0, 2), (1, 0, 3)] {
            if g.world.tile(tx + dx, ty + dy) != Tile::Wall {
                match edge {
                    0 => fb.rect_fill(x0, y0, TS, 1, OUTLINE),
                    1 => fb.rect_fill(x0, y0 + TS - 1, TS, 1, OUTLINE),
                    2 => fb.rect_fill(x0, y0, 1, TS, OUTLINE),
                    _ => fb.rect_fill(x0 + TS - 1, y0, 1, TS, OUTLINE),
                }
            }
        }
    }
    if tile == Tile::Door {
        let vertical = g.world.tile(tx, ty - 1) == Tile::Wall || g.world.tile(tx, ty + 1) == Tile::Wall;
        if vertical {
            fb.rect_fill(x0 + 6, y0, 4, TS, PLANK);
            fb.rect_fill(x0 + 7, y0, 2, TS, PLANK + 2);
        }
    }
    if tile == Tile::Wheel {
        // A red rug under the wheel so the work spot reads at a glance.
        fb.rect_fill(x0 + 1, y0 + 1, TS - 2, TS - 2, THREAD + 1);
        fb.rect_fill(x0 + 2, y0 + 2, TS - 4, TS - 4, THREAD);
        for k in (3..TS - 3).step_by(3) {
            fb.pset(x0 + k, y0 + 3, THREAD + 2);
            fb.pset(x0 + k, y0 + TS - 4, THREAD + 2);
        }
    }
    let pad = g.world.shop.pads.iter().position(|&p| p == (tx, ty));
    if pad.is_some() || tile == Tile::OutMat {
        let flash = pad.is_some_and(|p| g.world.shop.pad_flash[p] > 0);
        let edge = if flash { SPARK + 2 } else if tile == Tile::OutMat { MAT + 2 } else { MAT };
        fb.rect_fill(x0 + 1, y0 + 1, TS - 2, TS - 2, edge);
        for y in 2..TS - 2 {
            for x in 2..TS - 2 {
                let weave = ((x / 2) + (y / 2)) % 2 == 0;
                fb.pset(x0 + x, y0 + y, if weave { MAT + 1 } else { MAT });
            }
        }
        if g.world.item_at(tx, ty).is_none() {
            let icon = if tile == Tile::OutMat { &sp.icon_thread } else { &sp.icon_cotton };
            blit_ghost(fb, icon, x0 + 4, y0 + 4);
        }
    }
}

/// Every other pixel of a sprite: a dithered "this goes here" hint, no alpha.
fn blit_ghost(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + sx) as usize];
            if c > 1 && (x + sx + y + sy) % 2 == 0 {
                fb.pset(x + sx, y + sy, MAT + 2);
            }
        }
    }
}

fn draw_bin(sp: &Sprites, fb: &mut Framebuffer, b: &Bin) {
    let (x0, y0) = (b.x * TS, b.y * TS - (b.bounce > 1) as i32);
    fb.rect_bevel(x0 + 1, y0 + 2, 14, 14, WOOD + 1, WOOD + 3, WOOD);
    for y in [y0 + 6, y0 + 10] {
        fb.rect_fill(x0 + 2, y, 12, 1, WOOD);
    }
    let icon = if b.kind == Kind::Cotton { &sp.icon_cotton } else { &sp.icon_thread };
    fb.blit(icon, x0 + 4, y0 + 5, false);
    if let Some((mx, my)) = b.mouth {
        // Chute opening on the side facing the mouth tile, with an arrow while it refills.
        if my > b.y {
            fb.rect_fill(x0 + 4, y0 + 14, 8, 2, OUTLINE);
        }
        if b.count > 0 {
            let (ax, ay) = (mx * TS + 8, my * TS + 3);
            for (i, w) in [3, 2, 1].iter().enumerate() {
                fb.rect_fill(ax - w, ay + i as i32, w * 2 + 1, 1, PATH);
            }
        }
    }
    let label = b.count.to_string();
    let col = if b.count == 0 && b.mouth.is_some() { TEXT_WARN } else { TEXT };
    fb.text_centered(&label, x0 + 9, y0 - 8, col, Some(OUTLINE), 1);
}

fn draw_flags(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    for m in &g.monsters {
        let flag = if m.species == Species::Goblin { &sp.flag_goblin } else { &sp.flag_cyclops };
        if let Some((fx, fy)) = m.rec.as_ref().or(m.routine.as_ref()).map(|r| r.flag) {
            fb.blit(flag, fx * TS + 1, (fy + 1) * TS - flag.h, false);
        }
    }
}

/// Items and monsters, back to front by their on-screen feet.
fn draw_bodies(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    enum Body {
        Item(usize),
        Monster(usize),
    }
    let mut bodies: Vec<(i32, Body)> = Vec::new();
    for (i, it) in g.world.items.iter().enumerate() {
        bodies.push((it.y * TS + Slide::offset(it.slide).1, Body::Item(i)));
    }
    for (i, m) in g.monsters.iter().enumerate() {
        bodies.push((m.y * TS + Slide::offset(m.slide).1 + 1, Body::Monster(i)));
    }
    bodies.sort_by_key(|b| b.0);
    for (_, b) in bodies {
        match b {
            Body::Item(i) => {
                let it = &g.world.items[i];
                let s = if it.kind == Kind::Cotton { &sp.cotton } else { &sp.spool };
                let (ox, oy) = Slide::offset(it.slide);
                let jitter = if it.locked { (g.frame / 2 % 2) as i32 } else { 0 };
                let x = it.x * TS + ox + (TS - s.w) / 2 + jitter;
                let y = (it.y + 1) * TS - s.h + oy - HOP[it.hop.min(6) as usize];
                fb.rect_checker(it.x * TS + ox + 3, (it.y + 1) * TS + oy - 2, 10, 2, SHADOW);
                fb.blit(s, x, y, false);
            }
            Body::Monster(i) => {
                let (x, y, s, flip) = monster_pose(g, sp, i);
                let m = &g.monsters[i];
                let (ox, oy) = Slide::offset(m.slide);
                fb.rect_checker(m.x * TS + ox + 3, (m.y + 1) * TS + oy - 2, 10, 2, SHADOW);
                fb.blit(s, x, y, flip);
            }
        }
    }
}

fn monster_pose<'a>(g: &Game, sp: &'a Sprites, i: usize) -> (i32, i32, &'a Sprite, bool) {
    let m = &g.monsters[i];
    let frames = if m.species == Species::Goblin { &sp.goblin } else { &sp.cyclops };
    let frame = if m.slide.is_some() {
        2 + m.walk_frame as usize
    } else if m.busy {
        (g.frame / 2 % 2) as usize
    } else {
        (g.frame / 10 % 2) as usize
    };
    let s = &frames[frame];
    let (mut ox, mut oy) = Slide::offset(m.slide);
    if let Some((d, t)) = m.bump {
        let (dx, dy) = d.delta();
        let k = if t >= 2 { 2 } else { 1 };
        ox += dx * k;
        oy += dy * k;
    }
    (m.x * TS + ox + (TS - s.w) / 2, (m.y + 1) * TS - s.h + oy, s, m.flip)
}

fn draw_particles(g: &Game, fb: &mut Framebuffer) {
    for p in &g.world.particles {
        match p.kind {
            PKind::Dust => fb.pset(p.x, p.y, DUST),
            PKind::Leaf => fb.pset(p.x, p.y, LEAF + 2),
            PKind::Fluff => {
                fb.pset(p.x, p.y, if p.life > 4 { COTTON + 2 } else { COTTON + 1 });
                if p.life > 6 {
                    fb.pset(p.x + 1, p.y, COTTON + 1);
                }
            }
            PKind::Spark => {
                fb.pset(p.x, p.y, SPARK + 2);
                if p.life > 6 {
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        fb.pset(p.x + dx, p.y + dy, SPARK + 1);
                    }
                }
            }
        }
    }
}

fn draw_markers(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    for (i, m) in g.monsters.iter().enumerate() {
        let (x, _, s, _) = monster_pose(g, sp, i);
        let (_, oy) = Slide::offset(m.slide);
        let top = m.y * TS + oy + 2;
        if i == g.controlled {
            let bob = (g.frame / 8 % 2) as i32;
            fb.blit(&sp.marker, x + s.w / 2 - 3, top - 6 + bob, false);
        }
        let (bx, by) = (x + s.w - 3, top);
        if m.rec.is_some() {
            fb.rect_fill(bx, by, 5, 5, OUTLINE);
            fb.rect_fill(bx + 1, by + 1, 3, 3, REC + 1);
            fb.pset(bx + 1, by + 1, TEXT);
        } else if m.playing {
            fb.blit(&sp.loop_icon, bx - 1, by - 2, false);
        }
    }
}

struct BubbleBox {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cx: i32,
    above: bool, // bubble sits above the monster, tail points down
    lines: Vec<String>,
    icon: Option<Kind>,
    warn: bool,
}

/// Places speech bubbles so they never cover each other: above the head if there is room,
/// otherwise below the feet, otherwise nudged upward until clear. Controlled monster first.
fn layout_bubbles(g: &Game) -> Vec<BubbleBox> {
    let mut placed: Vec<BubbleBox> = Vec::new();
    let order = std::iter::once(g.controlled).chain((0..g.monsters.len()).filter(|&i| i != g.controlled));
    for i in order {
        let m = &g.monsters[i];
        let Some(b) = &m.bubble else { continue };
        // Monsters you are not steering only show what they wait for, not a sentence.
        let compact = b.waiting && i != g.controlled;
        let icon = match m.wait_reason {
            r if compact && r.contains("COTTON") => Some(Kind::Cotton),
            r if compact && r.contains("THREAD") => Some(Kind::Thread),
            _ => None,
        };
        let lines = match (compact, icon) {
            (false, _) => wrap(&b.text, 24),
            (true, Some(_)) => vec!["?".to_string()],
            (true, None) => vec!["...".to_string()],
        };
        let icon_w = if icon.is_some() { 9 } else { 0 };
        let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6 + icon_w;
        let h = lines.len() as i32 * 9 + 4;
        let (ox, oy) = Slide::offset(m.slide);
        let cx = m.x * TS + ox + 8;
        let (head, feet) = (m.y * TS + oy + 2, (m.y + 1) * TS + oy);
        let x = (cx - w / 2).clamp(2, 398 - w);
        let bodies: Vec<(i32, i32, i32, i32)> = (g.monsters.iter().enumerate())
            .filter(|(j, _)| *j != i)
            .map(|(_, o)| (o.x * TS + Slide::offset(o.slide).0, o.y * TS + Slide::offset(o.slide).1, TS, TS))
            .collect();
        let clear = |y: i32, placed: &[BubbleBox]| {
            let hits = |(px, py, pw, ph): (i32, i32, i32, i32)| x <= px + pw && px <= x + w && y <= py + ph && py <= y + h;
            y >= 2
                && y + h <= VIEW_H - 2
                && !placed.iter().any(|p| hits((p.x, p.y, p.w, p.h)))
                && !bodies.iter().any(|&r| hits(r))
        };
        let mut y = head - h - 5;
        let mut above = true;
        if !clear(y, &placed) {
            if clear(feet + 4, &placed) {
                y = feet + 4;
                above = false;
            } else {
                while y > 2 && !clear(y, &placed) {
                    y -= 1;
                }
                y = y.max(2);
            }
        }
        placed.push(BubbleBox { x, y, w, h, cx, above, lines, icon, warn: b.warn });
    }
    placed
}

fn draw_bubble(fb: &mut Framebuffer, sp: &Sprites, b: &BubbleBox) {
    let edge = if b.warn { TEXT_WARN } else { OUTLINE };
    fb.rect_fill(b.x, b.y, b.w, b.h, edge);
    fb.rect_fill(b.x + 1, b.y + 1, b.w - 2, b.h - 2, BUBBLE);
    // Three-row tail pointing at the speaker.
    let (base, dir) = if b.above { (b.y + b.h - 1, 1) } else { (b.y, -1) };
    let cx = b.cx.clamp(b.x + 4, b.x + b.w - 5);
    fb.rect_fill(cx - 2, base, 5, 1, BUBBLE);
    fb.rect_fill(cx - 1, base + dir, 3, 1, BUBBLE);
    for (dx, dy) in [(-3, 0), (3, 0), (-2, 1), (2, 1), (-1, 2), (0, 2), (1, 2)] {
        fb.pset(cx + dx, base + dy * dir, edge);
    }
    let mut tx = b.x + 4;
    if let Some(kind) = b.icon {
        fb.blit(if kind == Kind::Cotton { &sp.icon_cotton } else { &sp.icon_thread }, b.x + 3, b.y + 2, false);
        tx += 9;
    }
    for (k, line) in b.lines.iter().enumerate() {
        fb.text(line, tx, b.y + 3 + k as i32 * 9, INK, None, 1);
    }
}

fn draw_hud(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(0, VIEW_H, 400, 300 - VIEW_H, HUD_FILL, HUD_HI, HUD_SHADOW);
    let done = g.tutorial.stage >= tutorial::DONE;
    let objective = STAGES[g.tutorial.stage.min(tutorial::DONE)];
    for (k, line) in wrap(objective, 64).iter().enumerate() {
        fb.text(line, 6, VIEW_H + 5 + k as i32 * 10, if done { TEXT_GOOD } else { TEXT }, Some(HUD_SHADOW), 1);
    }
    fb.rect_fill(4, VIEW_H + 26, 392, 1, HUD_SHADOW);
    fb.rect_fill(4, VIEW_H + 27, 392, 1, HUD_MID);
    let (status, warn) = g.status(g.controlled);
    fb.text(&status, 6, VIEW_H + 32, if warn { TEXT_WARN } else { TEXT_DIM }, Some(HUD_SHADOW), 1);
    let count = format!("{}", g.delivered);
    let right = 394 - Framebuffer::text_width(&count, 1);
    fb.text(&count, right, VIEW_H + 32, TEXT, Some(HUD_SHADOW), 1);
    fb.blit(&sp.icon_thread, right - 11, VIEW_H + 31, false);
}

fn draw_help(fb: &mut Framebuffer) {
    let (w, h) = (252, HELP.len() as i32 * 11 + 34);
    let (x, y) = ((400 - w) / 2, (VIEW_H - h) / 2);
    fb.rect_checker(0, 0, 400, VIEW_H, OUTLINE);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered("CONTROLS", x + w / 2, y + 7, SPARK + 2, Some(HUD_SHADOW), 1);
    for (k, line) in HELP.iter().enumerate() {
        fb.text(line, x + 12, y + 20 + k as i32 * 11, TEXT, Some(HUD_SHADOW), 1);
    }
    fb.text_centered("F1 CLOSES", x + w / 2, y + h - 11, TEXT_DIM, Some(HUD_SHADOW), 1);
}
