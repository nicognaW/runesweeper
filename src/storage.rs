//! Settings and best times, kept as `key=value` lines: a file in the user's config folder on
//! desktop, `localStorage` on the web.

use crate::game::Level;

pub struct Saved {
    pub level: Level,
    pub custom: (usize, usize, usize),
    pub marks: bool,
    pub color: bool,
    /// Kept when reading/writing earlier preference files; the UI no longer shows a tutorial.
    pub played: bool,
    /// Best times (seconds) and names for Beginner, Intermediate and Expert.
    pub best: [(u32, String); 3],
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            level: Level::Beginner,
            custom: (9, 9, 10),
            marks: true,
            color: true,
            played: false,
            best: std::array::from_fn(|_| (999, "Anonymous".to_string())),
        }
    }
}

impl Saved {
    pub fn parse(src: &str) -> Saved {
        let mut s = Saved::default();
        for line in src.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let num = |v: &str| v.trim().parse::<usize>().ok();
            match k.trim() {
                "level" => {
                    s.level = match v.trim() {
                        "intermediate" => Level::Intermediate,
                        "expert" => Level::Expert,
                        "custom" => Level::Custom,
                        _ => Level::Beginner,
                    }
                }
                "custom" => {
                    let p: Vec<usize> = v.split(',').filter_map(num).collect();
                    if let [w, h, m] = p[..] {
                        s.custom = crate::game::clamp_custom(w, h, m);
                    }
                }
                "marks" => s.marks = v.trim() == "1",
                "color" => s.color = v.trim() == "1",
                "played" => s.played = v.trim() == "1",
                k if k.starts_with("best") => {
                    let Ok(i) = k[4..].parse::<usize>() else { continue };
                    let Some((t, name)) = v.split_once(',') else { continue };
                    if let (Some(slot), Some(t)) = (s.best.get_mut(i), num(t)) {
                        *slot = ((t as u32).min(999), name.to_string());
                    }
                }
                _ => {}
            }
        }
        s
    }

    pub fn write(&self) -> String {
        let level = match self.level {
            Level::Beginner => "beginner",
            Level::Intermediate => "intermediate",
            Level::Expert => "expert",
            Level::Custom => "custom",
        };
        let (w, h, m) = self.custom;
        let mut out = format!("level={level}\ncustom={w},{h},{m}\nmarks={}\ncolor={}\nplayed={}\n", self.marks as u8, self.color as u8, self.played as u8);
        for (i, (t, name)) in self.best.iter().enumerate() {
            out += &format!("best{i}={t},{}\n", name.replace('\n', " "));
        }
        out
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    }?;
    Some(base.join("runesweeper").join("runesweeper.ini"))
}

#[cfg(not(target_arch = "wasm32"))]
/// Headless runs (snapshots, scripts, tests) neither read nor write the player's file.
fn enabled() -> bool {
    rune::tooling::headless_run().is_none() && !cfg!(test)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Saved {
    if !enabled() {
        return Saved::default();
    }
    path().and_then(|p| std::fs::read_to_string(p).ok()).map(|s| Saved::parse(&s)).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(s: &Saved) {
    if !enabled() {
        return;
    }
    if let Some(p) = path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, s.write());
    }
}

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
pub fn load() -> Saved {
    storage().and_then(|s| s.get_item("runesweeper").ok()?).map(|s| Saved::parse(&s)).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn save(s: &Saved) {
    if let Some(st) = storage() {
        let _ = st.set_item("runesweeper", &s.write());
    }
}

/// The session's seed: a millisecond count, as Windows uses `GetTickCount()`.
pub fn seed() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64 as u32
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u32).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let mut s = Saved { level: Level::Expert, custom: (20, 12, 50), marks: false, ..Saved::default() };
        s.best[1] = (88, "Ada, again".into());
        let back = Saved::parse(&s.write());
        assert_eq!(back.level, Level::Expert);
        assert_eq!(back.custom, (20, 12, 50));
        assert!(!back.marks && back.color);
        assert_eq!(back.best[1], (88, "Ada, again".into()));
    }
}
