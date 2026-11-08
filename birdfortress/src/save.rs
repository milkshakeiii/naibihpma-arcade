//! Progress on disk: per level, whether it's solved, the best of each score (kept separately,
//! like any optimization game), and the layout last worked on.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Record {
    pub solved: bool,
    pub ticks: Option<i32>,
    pub cost: Option<i32>,
    pub crew: Option<i32>,
    /// The combo goal (ticks and cost in one run) has been met.
    pub combo: bool,
    pub layout: String,
}

pub struct Save {
    pub path: Option<PathBuf>,
    pub levels: BTreeMap<String, Record>,
    pub seen_intro: bool,
    pub crt: bool,
    pub music: usize,
    /// One-time tips already shown.
    pub tips: Vec<String>,
}

fn opt(s: &str) -> Option<i32> {
    s.parse().ok()
}

impl Save {
    pub fn load(path: Option<PathBuf>) -> Save {
        let mut s = Save { path: path.clone(), levels: BTreeMap::new(), seen_intro: false, crt: true, music: 2, tips: Vec::new() };
        let Some(p) = path else { return s };
        let Ok(text) = std::fs::read_to_string(&p) else { return s };
        for line in text.lines() {
            let f: Vec<&str> = line.split('|').collect();
            match f.first().copied() {
                Some("intro") => s.seen_intro = true,
                Some("crt") => s.crt = f.get(1) != Some(&"0"),
                Some("music") => s.music = f.get(1).and_then(|v| v.parse().ok()).unwrap_or(2).min(3),
                Some("level") if f.len() >= 7 => {
                    s.levels.insert(
                        f[1].to_string(),
                        Record { solved: f[2] == "1", ticks: opt(f[3]), cost: opt(f[4]), crew: opt(f[5]), combo: false, layout: f[6..].join("|") },
                    );
                }
                Some("combo") => {
                    if let Some(id) = f.get(1) {
                        s.levels.entry(id.to_string()).or_default().combo = true;
                    }
                }
                Some("tips") => s.tips = f[1..].iter().filter(|t| !t.is_empty()).map(|t| t.to_string()).collect(),
                _ => {}
            }
        }
        s
    }

    pub fn write(&self) {
        let Some(p) = &self.path else { return };
        let mut out = String::new();
        if self.seen_intro {
            out.push_str("intro\n");
        }
        out.push_str(&format!("crt|{}\nmusic|{}\n", self.crt as u8, self.music));
        let o = |v: Option<i32>| v.map(|n| n.to_string()).unwrap_or_default();
        for (id, r) in &self.levels {
            out.push_str(&format!("level|{}|{}|{}|{}|{}|{}\n", id, r.solved as u8, o(r.ticks), o(r.cost), o(r.crew), r.layout));
        }
        for (id, r) in &self.levels {
            if r.combo {
                out.push_str(&format!("combo|{id}\n"));
            }
        }
        out.push_str(&format!("tips|{}\n", self.tips.join("|")));
        if let Ok(mut f) = std::fs::File::create(p) {
            let _ = f.write_all(out.as_bytes());
        }
    }

    pub fn rec(&self, id: &str) -> Record {
        self.levels.get(id).cloned().unwrap_or_default()
    }

    pub fn seen(&self, tip: &str) -> bool {
        self.tips.iter().any(|t| t == tip)
    }

    pub fn rec_mut(&mut self, id: &str) -> &mut Record {
        self.levels.entry(id.to_string()).or_default()
    }
}

/// Appends timestamped lines to the playtest log (the channel for learning how play went).
pub struct Log {
    file: Option<std::fs::File>,
    start: std::time::Instant,
}

impl Log {
    pub fn open(path: Option<PathBuf>) -> Log {
        let file = path.and_then(|p| std::fs::OpenOptions::new().create(true).append(true).open(p).ok());
        let mut l = Log { file, start: std::time::Instant::now() };
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        l.line(&format!("=== session start (unix {now})"));
        l
    }

    pub fn line(&mut self, s: &str) {
        let t = self.start.elapsed().as_secs();
        if let Some(f) = self.file.as_mut() {
            let _ = writeln!(f, "[{:02}:{:02}] {}", t / 60, t % 60, s);
        }
    }
}
