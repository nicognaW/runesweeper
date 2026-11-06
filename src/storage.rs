//! Settings, kept as `key=value` lines, and History, one game record per line: files in the
//! user's config folder on desktop, `localStorage` on the web.
//!
//! The game in play is kept too, after every click, so a game left by closing the window or
//! the tab is recorded as abandoned at the next launch.

use crate::game::{Level, Mode};
use crate::record::Record;
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
fn dir() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    }?;
    Some(base.join("runesweeper"))
}

#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<std::path::PathBuf> {
    Some(dir()?.join("runesweeper.ini"))
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

/// Every record in a History text, oldest first; lines that aren't records are skipped.
pub fn parse_history(text: &str) -> Vec<Record> {
    text.lines().filter_map(Record::parse).collect()
}

/// History with `line` added, kept under one storage key: `set` tries a write and says whether
/// the store took it. While it refuses, the oldest records go, one at a time. False if even
/// the line alone was refused.
pub fn append_fitting(old: &str, line: &str, mut set: impl FnMut(&str) -> bool) -> bool {
    let mut keep: Vec<&str> = old.lines().filter(|l| !l.trim().is_empty()).collect();
    loop {
        let mut text = keep.join("\n");
        if !text.is_empty() {
            text.push('\n');
        }
        text += line;
        text.push('\n');
        if set(&text) {
            return true;
        }
        if keep.is_empty() {
            return false;
        }
        keep.remove(0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod history_file {
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use super::*;

    pub fn history() -> Option<PathBuf> {
        Some(dir()?.join("history.txt"))
    }

    pub fn draft() -> Option<PathBuf> {
        Some(dir()?.join("game.txt"))
    }

    pub fn append(path: &Path, r: &Record) -> std::io::Result<()> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(f, "{}", r.write())
    }

    pub fn load(path: &Path) -> Vec<Record> {
        std::fs::read_to_string(path).map(|t| parse_history(&t)).unwrap_or_default()
    }
}

/// Every game record, oldest first.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_history() -> Vec<Record> {
    if !enabled() {
        return Vec::new();
    }
    history_file::history().map(|p| history_file::load(&p)).unwrap_or_default()
}

/// Adds a record to History.
#[cfg(not(target_arch = "wasm32"))]
pub fn append_record(r: &Record) {
    if !enabled() {
        return;
    }
    if let Some(p) = history_file::history() {
        let _ = history_file::append(&p, r);
    }
}

/// Keeps the game in play, replacing the last one kept; None forgets it.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_draft(r: Option<&Record>) {
    if !enabled() {
        return;
    }
    let Some(p) = history_file::draft() else { return };
    match r {
        Some(r) => {
            let _ = std::fs::create_dir_all(p.parent().unwrap_or(&p)).and_then(|_| std::fs::write(&p, r.write()));
        }
        None => {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// The game a previous run left in play, if any, and forgets it.
#[cfg(not(target_arch = "wasm32"))]
pub fn take_draft() -> Option<Record> {
    if !enabled() {
        return None;
    }
    let p = history_file::draft()?;
    let r = std::fs::read_to_string(&p).ok().and_then(|t| Record::parse(t.trim()));
    let _ = std::fs::remove_file(p);
    r
}

#[cfg(target_arch = "wasm32")]
const HISTORY_KEY: &str = "runesweeper-history";
#[cfg(target_arch = "wasm32")]
const DRAFT_KEY: &str = "runesweeper-game";

#[cfg(target_arch = "wasm32")]
pub fn load_history() -> Vec<Record> {
    storage().and_then(|s| s.get_item(HISTORY_KEY).ok()?).map(|t| parse_history(&t)).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn append_record(r: &Record) {
    let Some(st) = storage() else { return };
    let old = st.get_item(HISTORY_KEY).ok().flatten().unwrap_or_default();
    append_fitting(&old, &r.write(), |text| st.set_item(HISTORY_KEY, text).is_ok());
}

#[cfg(target_arch = "wasm32")]
pub fn save_draft(r: Option<&Record>) {
    let Some(st) = storage() else { return };
    let _ = match r {
        Some(r) => st.set_item(DRAFT_KEY, &r.write()),
        None => st.remove_item(DRAFT_KEY),
    };
}

#[cfg(target_arch = "wasm32")]
pub fn take_draft() -> Option<Record> {
    let st = storage()?;
    let r = st.get_item(DRAFT_KEY).ok()?.and_then(|t| Record::parse(t.trim()));
    let _ = st.remove_item(DRAFT_KEY);
    r
}

/// Milliseconds since the Unix epoch: when a game record's first click came.
pub fn unix_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
    }
}

/// Minutes to add to UTC for the player's local time at `ms` since the Unix epoch, daylight
/// saving included: the browser's on the web, the system's time zone natively (no new
/// dependency: the C library's `localtime_r`, or Windows' own conversion).
pub fn utc_offset_minutes(ms: u64) -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        -(js_sys::Date::new(&(ms as f64).into()).get_timezone_offset() as i64)
    }
    #[cfg(unix)]
    {
        use std::ffi::{c_char, c_int, c_long};
        /// C's `struct tm` as glibc, musl and macOS lay it out.
        #[repr(C)]
        struct Tm {
            fields: [c_int; 9],
            gmtoff: c_long,
            zone: *const c_char,
        }
        unsafe extern "C" {
            fn tzset();
            fn localtime_r(t: *const c_long, out: *mut Tm) -> *mut Tm;
        }
        let t = (ms / 1000) as c_long;
        let mut tm = Tm { fields: [0; 9], gmtoff: 0, zone: std::ptr::null() };
        // SAFETY: `t` and `tm` are valid for the call, which only writes `tm`.
        let ok = unsafe {
            tzset();
            !localtime_r(&t, &mut tm).is_null()
        };
        if ok { tm.gmtoff as i64 / 60 } else { 0 }
    }
    #[cfg(windows)]
    {
        /// FILETIME, as one count of 100 ns since 1601.
        #[repr(C)]
        #[derive(Default)]
        struct FileTime(u32, u32);
        #[repr(C)]
        #[derive(Default)]
        struct SystemTime([u16; 8]);
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn FileTimeToSystemTime(ft: *const FileTime, st: *mut SystemTime) -> i32;
            fn SystemTimeToTzSpecificLocalTime(tz: *const std::ffi::c_void, utc: *const SystemTime, local: *mut SystemTime) -> i32;
            fn SystemTimeToFileTime(st: *const SystemTime, ft: *mut FileTime) -> i32;
        }
        let ticks = (ms + 11_644_473_600_000) * 10_000;
        let utc = FileTime(ticks as u32, (ticks >> 32) as u32);
        let (mut st, mut local, mut back) = (SystemTime::default(), SystemTime::default(), FileTime::default());
        // SAFETY: every pointer is to a live local of the type the call takes; a null zone is
        // the system's current one.
        let ok = unsafe { FileTimeToSystemTime(&utc, &mut st) != 0 && SystemTimeToTzSpecificLocalTime(std::ptr::null(), &st, &mut local) != 0 && SystemTimeToFileTime(&local, &mut back) != 0 };
        let local_ticks = ((back.1 as u64) << 32) | back.0 as u64;
        if ok { (local_ticks as i64 - ticks as i64) / 600_000_000 } else { 0 }
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

    /// History's dates are the player's local time, daylight saving included, for the
    /// moment each game was played. Checked in a child run per time zone (POSIX zone strings,
    /// which need no time zone files), against offsets worked by hand.
    #[cfg(unix)]
    #[test]
    fn native_dates_are_local_time_for_each_moment() {
        // 2023-01-15 12:00 and 2023-07-15 12:00 UTC.
        let (winter, summer) = (1_673_784_000_000, 1_689_422_400_000);
        let zones = [("UTC0", 0, 0), ("IST-5:30", 330, 330), ("EST5EDT,M3.2.0,M11.1.0", -300, -240), ("ACST-9:30ACDT,M10.1.0,M4.1.0/3", 630, 570)];
        if let Ok(tz) = std::env::var("RUNESWEEPER_TZ_CHILD") {
            let &(_, w, s) = zones.iter().find(|z| z.0 == tz).expect("a zone of the table");
            assert_eq!((utc_offset_minutes(winter), utc_offset_minutes(summer)), (w, s), "{tz}");
            return;
        }
        for (tz, ..) in zones {
            let out = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "storage::tests::native_dates_are_local_time_for_each_moment", "--test-threads=1"])
                .env("TZ", tz)
                .env("RUNESWEEPER_TZ_CHILD", tz)
                .output()
                .unwrap();
            let said = String::from_utf8_lossy(&out.stdout);
            assert!(out.status.success() && said.contains("1 passed"), "{tz}: {said}");
        }
    }

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

    fn record(at: u64) -> Record {
        let mut r = Record::new(Mode::Classic, (9, 9, 10));
        r.at = at;
        r.end = crate::record::End::Won;
        r.seconds = 12;
        r.layout = crate::record::Layout::Mines(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        r.moves = vec![crate::record::Move { ms: 0, act: crate::record::Act::Open, cell: 40 }];
        r
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn records_round_trip_through_the_history_file() {
        let dir = std::env::temp_dir().join(format!("runesweeper-history-test-{}", std::process::id()));
        let path = dir.join("history.txt");
        let _ = std::fs::remove_file(&path);
        assert!(history_file::load(&path).is_empty(), "no file, no records");
        for at in [1, 2, 3] {
            history_file::append(&path, &record(at)).unwrap();
        }
        let back = history_file::load(&path);
        assert_eq!(back, vec![record(1), record(2), record(3)], "appended in order, read back whole");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Browser storage as a quota: it refuses a write longer than `cap` bytes.
    struct Quota {
        cap: usize,
        held: String,
    }

    impl Quota {
        fn set(&mut self, text: &str) -> bool {
            if text.len() > self.cap {
                return false;
            }
            self.held = text.to_string();
            true
        }
    }

    #[test]
    fn records_round_trip_through_browser_storage_and_the_oldest_go_only_when_it_is_full() {
        let line = record(1).write();
        let mut store = Quota { cap: (line.len() + 1) * 3, held: String::new() };
        for at in [1, 2, 3] {
            let old = store.held.clone();
            assert!(append_fitting(&old, &record(at).write(), |t| store.set(t)));
        }
        assert_eq!(parse_history(&store.held), vec![record(1), record(2), record(3)], "nothing dropped while it fits");
        let old = store.held.clone();
        assert!(append_fitting(&old, &record(4).write(), |t| store.set(t)));
        assert_eq!(parse_history(&store.held), vec![record(2), record(3), record(4)], "the oldest goes when the store refuses");
        let mut tiny = Quota { cap: 10, held: "kept".into() };
        assert!(!append_fitting("", &line, |t| tiny.set(t)), "a record the store can't hold at all");
        assert_eq!(tiny.held, "kept");
    }
}
