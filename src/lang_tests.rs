//! The language switch, driven headlessly through Rune's `Runtime`, as the toolbar tests are.

use rune::Runtime;

use crate::app::App;
use crate::game::{Cover, Level};
use crate::storage::Saved;
use crate::strings::Lang;
use crate::toolbar_tests::{click, runtime, settle, text_in, words};

/// Han characters, as the zh-CN table writes them.
fn han(w: &str) -> bool {
    w.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// No English left on screen: the only Latin letters are F2's, the font credit's "MiSans"
/// (a name) and the switch's own "English", which names the language in itself.
fn assert_all_zh_cn(rt: &Runtime<App>, when: &str) {
    for w in words(rt) {
        let rest = w.replace("F2", "").replace("MiSans", "").replace(Lang::En.endonym(), "");
        assert!(!rest.chars().any(|c| c.is_ascii_alphabetic()), "{w:?} is still English ({when})");
    }
}

/// No Chinese on screen but the switch's own "简体中文".
fn assert_all_english(rt: &Runtime<App>, when: &str) {
    for w in words(rt) {
        assert!(!han(&w.replace(Lang::ZhCn.endonym(), "")), "{w:?} is Chinese ({when})");
    }
}

fn zh() -> Saved {
    Saved { lang: Some(Lang::ZhCn), ..Saved::default() }
}

#[test]
fn switching_language_in_settings_changes_every_string_at_once() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "settings-button");
    settle(&mut rt);
    assert_all_english(&rt, "before the switch");
    assert_eq!(text_in(&rt, "level-chip"), "Beginner");
    click(&mut rt, "lang-zh-CN");
    // The very next frame, with no settling.
    assert_eq!(rt.state.lang, Lang::ZhCn);
    assert_eq!(text_in(&rt, "level-chip"), "初级");
    let shown = words(&rt);
    for w in ["设置", "问号标记", "颜色", "语言", "新游戏", "雷区"] {
        assert!(shown.iter().any(|s| s == w), "{w} on screen: {shown:?}");
    }
    assert!(shown.iter().any(|s| s.starts_with("剩余雷数")), "the mine counter: {shown:?}");
    assert_all_zh_cn(&rt, "after the switch");
    assert!(rt.find("settings").is_some(), "Settings stays open while switching");
    click(&mut rt, "lang-en");
    assert_eq!(text_in(&rt, "level-chip"), "Beginner");
    assert_all_english(&rt, "after switching back");
}

#[test]
fn the_language_survives_a_restart() {
    let mut rt = runtime(Saved::default());
    click(&mut rt, "settings-button");
    settle(&mut rt);
    click(&mut rt, "lang-zh-CN");
    // What the settings file would hold, read back by the next launch.
    let file = rt.state.saved.write();
    let rt = runtime(Saved::parse(&file));
    assert_eq!(rt.state.lang, Lang::ZhCn);
    assert_eq!(text_in(&rt, "level-chip"), "初级");
    assert_all_zh_cn(&rt, "after a restart");
}

#[test]
fn the_picker_speaks_zh_cn() {
    let mut rt = runtime(zh());
    click(&mut rt, "level-chip");
    settle(&mut rt);
    let shown = words(&rt);
    for w in ["初级", "中级", "高级", "自定义", "宽度", "高度", "雷数"] {
        assert!(shown.iter().any(|s| s == w), "{w} in the picker: {shown:?}");
    }
    assert_all_zh_cn(&rt, "the picker");
    click(&mut rt, "pick-expert");
    settle(&mut rt);
    assert_eq!(rt.state.level, Level::Expert);
    assert_eq!(text_in(&rt, "level-chip"), "高级");
}

#[test]
fn the_result_banner_speaks_zh_cn() {
    // A loss: a safe square first (so the mine isn't moved), then a mine.
    let mut rt = runtime(zh());
    rt.update(|s, _| {
        let g = &mut s.game;
        g.start();
        let safe = (0..g.cells.len()).find(|&i| !g.cells[i].mine).unwrap();
        let _ = g.open(safe);
        let mine = (0..g.cells.len()).find(|&i| g.cells[i].mine).unwrap();
        let _ = g.open(mine);
    });
    settle(&mut rt);
    assert!(words(&rt).iter().any(|w| w == "失败"), "{:?}", words(&rt));
    assert_all_zh_cn(&rt, "a loss");

    // A win.
    let mut rt = runtime(zh());
    rt.update(|s, _| {
        for i in 0..s.game.cells.len() {
            if !s.game.cells[i].mine && s.game.cells[i].cover != Cover::Open {
                let _ = s.game.open(i);
            }
        }
        s.seconds = 37;
    });
    settle(&mut rt);
    assert!(words(&rt).iter().any(|w| w == "胜利！用时 37 秒"), "{:?}", words(&rt));
    assert_all_zh_cn(&rt, "a win");
}

#[test]
fn settings_credits_misans_once_in_either_language() {
    for (saved, credit) in [(Saved::default(), "Chinese text: MiSans by Xiaomi"), (zh(), "中文字体：小米 MiSans")] {
        let mut rt = runtime(saved);
        assert!(!words(&rt).iter().any(|w| w.contains("MiSans")), "no credit until Settings opens");
        click(&mut rt, "settings-button");
        settle(&mut rt);
        let credits: Vec<String> = words(&rt).into_iter().filter(|w| w.contains("MiSans")).collect();
        assert_eq!(credits, [credit], "one credit line in Settings");
    }
}
