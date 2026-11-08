//! Painted backdrops and the title painting, stored as indexed images (made offline by
//! `tools/convert_art.py`): each pixel is a slot number, and each picture brings its own small
//! palette of 6-bit colours that the game loads into reserved palette slots.

use crate::framebuffer::{Framebuffer, FB_W};
use crate::palette::Rgb;

pub struct Picture {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
    pub colours: Vec<Rgb>,
}

impl Picture {
    fn load(png: &[u8], pal: &str) -> Picture {
        let colours: Vec<Rgb> = pal
            .lines()
            .filter_map(|l| {
                let v: Vec<u8> = l.split_whitespace().filter_map(|n| n.parse().ok()).collect();
                (v.len() == 3).then(|| (v[0], v[1], v[2]))
            })
            .collect();
        match image::load_from_memory(png) {
            Ok(img) => {
                let g = img.to_luma8();
                let (w, h) = (g.width() as i32, g.height() as i32);
                Picture { w, h, px: g.into_raw(), colours }
            }
            Err(_) => Picture { w: 0, h: 0, px: Vec::new(), colours },
        }
    }

    /// Draws the picture at (0, 0), mapping slot k to palette index `slots[k]`.
    pub fn draw(&self, fb: &mut Framebuffer, slots: &[u8], max_h: i32) {
        let h = self.h.min(max_h);
        for y in 0..h {
            let row = (y * self.w) as usize;
            let out = y as usize * FB_W;
            for x in 0..self.w.min(FB_W as i32) {
                let s = self.px[row + x as usize] as usize;
                fb.px[out + x as usize] = slots[s.min(slots.len() - 1)];
            }
        }
    }
}

pub struct Gallery {
    pub title: Picture,
    /// One backdrop per chapter, first chapter first.
    pub backdrops: Vec<Picture>,
}

impl Gallery {
    pub fn new() -> Gallery {
        macro_rules! pic {
            ($n:literal) => {
                Picture::load(include_bytes!(concat!("../assets/art/", $n, ".png")), include_str!(concat!("../assets/art/", $n, ".pal")))
            };
        }
        Gallery { title: pic!("title"), backdrops: vec![pic!("meadow"), pic!("quarry"), pic!("canopy"), pic!("bastion"), pic!("sky")] }
    }

    pub fn backdrop(&self, chapter: usize) -> &Picture {
        &self.backdrops[chapter.clamp(1, self.backdrops.len()) - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_load_with_their_palettes() {
        let g = Gallery::new();
        assert_eq!((g.title.w, g.title.h), (400, 300));
        assert_eq!(g.title.colours.len(), crate::palette::TITLE_SLOTS);
        for b in &g.backdrops {
            assert_eq!((b.w, b.h), (400, 240));
            assert_eq!(b.colours.len(), crate::palette::BACKDROP.len());
            assert!(b.px.iter().all(|&s| (s as usize) < b.colours.len()));
        }
    }
}
