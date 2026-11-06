//! Every string the game shows, in English and zh-CN, in one table owned by the crate.
//!
//! To add a string, add one line to the `strings!` table below: its key, the English, then the
//! zh-CN. Use it as `s.tr(S::Key)`, or `s.trf(S::Key, &[&value])` when it has `{}` slots, which
//! are filled in order. Both languages must have the same slots; the tests check every key.
//! zh-CN follows the Chinese community's own words (`docs/spec.md`, the zh-CN terminology
//! notes): 初级 / 中级 / 高级 / 自定义, 胜利 / 失败, 剩余雷数, 新游戏, 纪录 for a best result.

use std::fmt::Display;

use crate::game::Level;

/// The game's language.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    En,
    ZhCn,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::ZhCn];

    /// The BCP 47 tag, for the settings file and for text shaping.
    pub fn tag(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::ZhCn => "zh-CN",
        }
    }

    /// The language's name in itself, the same whichever language the game is in.
    pub fn endonym(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::ZhCn => "简体中文",
        }
    }

    /// The game's language for a system or browser locale: any Chinese locale (`zh`, `zh-TW`,
    /// `zh_CN.UTF-8`, `zh-Hans-CN`...) means zh-CN, anything else English.
    pub fn from_locale(tag: &str) -> Lang {
        let primary = tag.trim().split(['-', '_', '.', '@']).next().unwrap_or("");
        if primary.eq_ignore_ascii_case("zh") { Lang::ZhCn } else { Lang::En }
    }
}

macro_rules! strings {
    ($($key:ident: $en:literal, $zh:literal;)*) => {
        /// A string's key. Its text comes from [`S::get`].
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum S { $($key),* }

        impl S {
            pub const ALL: &[S] = &[$(S::$key),*];

            pub fn get(self, lang: Lang) -> &'static str {
                match lang {
                    Lang::En => match self { $(S::$key => $en),* },
                    Lang::ZhCn => match self { $(S::$key => $zh),* },
                }
            }
        }
    };
}

strings! {
    // Levels.
    Beginner: "Beginner", "初级";
    Intermediate: "Intermediate", "中级";
    Expert: "Expert", "高级";
    Custom: "Custom", "自定义";
    // The console.
    MinesLeft: "Mines left: {}", "剩余雷数：{}";
    Time: "Time: {}", "用时：{}";
    NewGame: "New game", "新游戏";
    Minefield: "Minefield", "雷区";
    // The result banner.
    WonIn1: "Cleared in 1 second", "胜利！用时 1 秒";
    WonIn: "Cleared in {} seconds", "胜利！用时 {} 秒";
    NewBest: "New best for {}", "{}新纪录";
    WonAgain: "F2 or the face for another", "按 F2 或笑脸再来一局";
    Lost: "Boom.", "失败";
    LostAgain: "F2 or the face to try again", "按 F2 或笑脸重新开始";
    // The toolbar and its panels.
    FlagMode: "Flag mode", "标雷模式";
    Settings: "Settings", "设置";
    LevelPicker: "Level", "级别";
    Width: "Width", "宽度";
    Height: "Height", "高度";
    Mines: "Mines", "雷数";
    Marks: "Marks (?)", "问号标记";
    Color: "Color", "颜色";
    Language: "Language", "语言";
    FontCredit: "Chinese text: MiSans by Xiaomi", "中文字体：小米 MiSans";
}

impl S {
    /// The text with its `{}` slots filled by `args`, in order.
    pub fn fill(self, lang: Lang, args: &[&dyn Display]) -> String {
        let mut parts = self.get(lang).split("{}");
        let mut out = parts.next().unwrap_or("").to_string();
        for (i, part) in parts.enumerate() {
            if let Some(a) = args.get(i) {
                out.push_str(&a.to_string());
            }
            out.push_str(part);
        }
        out
    }

    pub fn level(l: Level) -> S {
        match l {
            Level::Beginner => S::Beginner,
            Level::Intermediate => S::Intermediate,
            Level::Expert => S::Expert,
            Level::Custom => S::Custom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_english_string_has_a_zh_cn_entry_with_the_same_slots() {
        for &k in S::ALL {
            let (en, zh) = (k.get(Lang::En), k.get(Lang::ZhCn));
            assert!(!en.trim().is_empty(), "{k:?} has no English");
            assert!(!zh.trim().is_empty(), "{k:?} has no zh-CN entry");
            assert_ne!(en, zh, "{k:?}'s zh-CN entry is the English, untranslated");
            assert!(zh.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)), "{k:?}'s zh-CN entry {zh:?} has no Chinese in it");
            assert_eq!(en.matches("{}").count(), zh.matches("{}").count(), "{k:?}: English and zh-CN take the same values");
        }
    }

    #[test]
    fn slots_are_filled_in_order() {
        assert_eq!(S::WonIn.fill(Lang::En, &[&42]), "Cleared in 42 seconds");
        assert_eq!(S::WonIn.fill(Lang::ZhCn, &[&42]), "胜利！用时 42 秒");
        assert_eq!(S::NewBest.fill(Lang::ZhCn, &[&S::Expert.get(Lang::ZhCn)]), "高级新纪录");
    }

    #[test]
    fn the_community_terms_are_used() {
        let zh = |k: S| k.get(Lang::ZhCn);
        assert_eq!([zh(S::Beginner), zh(S::Intermediate), zh(S::Expert), zh(S::Custom)], ["初级", "中级", "高级", "自定义"]);
        assert!(zh(S::WonIn).contains("胜利") && zh(S::Lost).contains("失败"));
        assert!(zh(S::MinesLeft).contains("剩余雷数"));
        assert_eq!(zh(S::NewGame), "新游戏");
        assert!(zh(S::NewBest).contains("纪录"), "a best result is a 纪录, not a 记录");
    }

    #[test]
    fn any_chinese_locale_means_zh_cn_and_anything_else_english() {
        for tag in ["zh", "zh-CN", "zh_CN.UTF-8", "zh-Hans-CN", "zh-TW", "zh_HK", "ZH-cn", "zh-Hant"] {
            assert_eq!(Lang::from_locale(tag), Lang::ZhCn, "{tag}");
        }
        for tag in ["en", "en-US", "en_GB.UTF-8", "C", "POSIX", "", "ja-JP", "ko", "fr_FR", "zu", "zhx"] {
            assert_eq!(Lang::from_locale(tag), Lang::En, "{tag}");
        }
    }
}
