//! Settings, kept as `key=value` lines: a file in the user's config folder on
//! desktop, `localStorage` on the web.

use crate::game::{Level, Mode};
use crate::strings::Lang;

pub struct Saved {
    pub mode: Mode,
    pub level: Level,
    pub custom: (usize, usize, usize),
    pub marks: bool,
    pub color: bool,
    /// The language picked in Settings; until one is, the game follows the system's.
    pub lang: Option<Lang>,
}

impl Default for Saved {
    fn default() -> Self {
        Saved {
            mode: Mode::Classic,
            level: Level::Beginner,
            custom: (9, 9, 10),
            marks: true,
            color: true,
            lang: None,
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
                "mode" => s.mode = Mode::from_tag(v.trim()).unwrap_or_default(),
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
                "lang" => s.lang = Lang::ALL.into_iter().find(|l| l.tag() == v.trim()),
                // Anything else (the old best times and first-run flag among them) is dropped.
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
        let mut out = format!("mode={}\nlevel={level}\ncustom={w},{h},{m}\nmarks={}\ncolor={}\n", self.mode.tag(), self.marks as u8, self.color as u8);
        if let Some(l) = self.lang {
            out += &format!("lang={}\n", l.tag());
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
        let mut s = Saved::default();
        s.level = Level::Expert;
        s.custom = (20, 12, 50);
        s.marks = false;
        let back = Saved::parse(&s.write());
        assert_eq!(back.level, Level::Expert);
        assert_eq!(back.custom, (20, 12, 50));
        assert!(!back.marks && back.color);
    }

    #[test]
    fn the_language_is_saved_once_chosen_and_follows_the_system_until_then() {
        assert_eq!(Saved::parse("level=expert\n").lang, None, "never chosen");
        assert!(!Saved::default().write().contains("lang"), "nothing written until chosen");
        for lang in Lang::ALL {
            let s = Saved { lang: Some(lang), ..Saved::default() };
            assert_eq!(Saved::parse(&s.write()).lang, Some(lang));
        }
        assert_eq!(Saved::parse("lang=zh-CN\n").lang, Some(Lang::ZhCn));
        assert_eq!(Saved::parse("lang=klingon\n").lang, None, "an unknown language counts as none");
    }

    #[test]
    fn old_best_times_and_first_run_keys_are_neither_read_nor_written() {
        let old = "level=intermediate\nmarks=0\nplayed=1\nbest0=12,Ada\nbest1=88,Ada, again\nbest2=301,Bob\n";
        let back = Saved::parse(old);
        assert_eq!(back.level, Level::Intermediate);
        assert!(!back.marks);
        let out = back.write();
        assert!(!out.contains("best"), "no best-time keys written: {out}");
        assert!(!out.contains("played"), "no first-run key written: {out}");
    }
}
