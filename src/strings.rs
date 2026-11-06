//! Every string the game shows, in English and zh-CN, in one table owned by the crate.
//!
//! To add a string, add one line to the `strings!` table below: its key, the English, then the
//! zh-CN. Use it as `s.tr(S::Key)`, or `s.trf(S::Key, &[&value])` when it has `{}` slots, which
//! are filled in order. Both languages must have the same slots; the tests check every key.
//! zh-CN follows the Chinese community's own words (`docs/spec.md`, the zh-CN terminology
//! notes): 初级 / 中级 / 高级 / 自定义, 胜利 / 失败, 剩余雷数, 新游戏, 纪录 for a best result.

use std::fmt::Display;

use crate::game::{Level, Mode};

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
    // Modes.
    Classic: "Classic", "原版";
    GentleNoGuess: "Gentle No Guess", "简单无猜";
    NoGuess: "No Guess", "无猜";
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
    LevelPicker: "Mode and level", "模式与级别";
    ModeRow: "Mode", "模式";
    Width: "Width", "宽度";
    Height: "Height", "高度";
    Mines: "Mines", "雷数";
    Marks: "Marks (?)", "问号标记";
    Color: "Color", "颜色";
    Language: "Language", "语言";
    FontCredit: "Chinese text: MiSans by Xiaomi", "中文字体：小米 MiSans";
    // History.
    History: "History", "历史记录";
    LevelRow: "Level", "级别";
    FastestWin: "Fastest win", "最快纪录";
    NoGames: "No games yet", "还没有对局";
    Won: "Won", "胜利";
    LostGame: "Lost", "失败";
    Abandoned: "Abandoned", "放弃";
    Seconds: "{} s", "{} 秒";
    Bv3Clicks: "3BV {}/{} · {} clicks", "3BV {}/{} · 点击 {} 次";
    ForcedGuesses: "{} forced, {} survived", "必须猜 {} 次，猜对 {} 次";
    Assisted1: "Assisted, 1 hint", "使用提示 1 次";
    Assisted: "Assisted, {} hints", "使用提示 {} 次";
    // Hints (ADR 0002: reasons a player can check by hand, never a probability). The zh-CN
    // sentences follow the tutorials' 「看这个 N，它周围还有 M 个雷……所以这一格必定安全」.
    Hint: "Hint", "提示";
    HintApply: "Apply", "执行";
    HintClose: "Close", "关闭";
    Easy: "Easy", "简单";
    Medium: "Medium", "中等";
    Hard: "Hard", "困难";
    Evil: "Evil", "地狱";
    CountMine1: "1 mine", "1 个雷";
    CountMines: "{} mines", "{} 个雷";
    CountCell1: "1 hidden cell", "1 格";
    CountCells: "{} hidden cells", "{} 格";
    TechWrongFlag: "Wrong flag", "旗插错了";
    TechSingle: "Single number", "单数字判雷";
    TechSatisfied: "Satisfied number", "单数字判空";
    TechOneOne: "1-1 pattern", "11定式";
    TechOneTwo: "1-2 pattern", "12定式";
    TechReductionOneOne: "Reduction to a 1-1", "减雷变11定式";
    TechReductionOneTwo: "Reduction to a 1-2", "减雷变12定式";
    TechNothing: "No simple step", "没有简单推理";
    NudgeWrongFlag: "One of your flags can't be right.", "有一面旗插错了。";
    NudgeSingle: "Look for a number with only as many hidden cells as it needs.", "找一个周围只剩下对应数量格子的数字。";
    NudgeSatisfied: "Look for a number that already has all its mines.", "找一个周围的雷已经够数的数字。";
    NudgeOneOne: "Compare two numbers that share hidden cells.", "比较两个共用格子的数字，找 11定式。";
    NudgeOneTwo: "Compare two numbers that share hidden cells, and their difference.", "比较两个共用格子的数字，找 12定式。";
    NudgeReduction: "Subtract the mines you can prove, then compare two numbers.", "先减雷，再比较两个数字。";
    NudgeNothing: "No one or two numbers settle any cell right now.", "现在没有只靠一两个数字就能确定的格子。";
    LookOne: "Look at this number.", "看这个数字。";
    LookMany: "Look at these numbers.", "看这几个数字。";
    CellSafe: "This cell is safe.", "这一格必定安全。";
    CellMine: "This cell is a mine.", "这一格必须是雷。";
    CellWrongFlag: "This flag is wrong: the cell is safe.", "这面旗插错了：这一格是安全的。";
    StepSingle: "This {} still needs {} and touches only {}, so this cell must be a mine.", "看这个 {}，它周围还有 {}，只剩下 {}……所以这一格必须是雷。";
    StepProver1: "First, this {} touches only one hidden cell, so that cell is a mine.", "先看这个 {}，它周围只剩下 1 格，所以那一格是雷。";
    StepProver: "First, this {} touches only {}, so they are all mines.", "先看这个 {}，它周围只剩下 {}，所以这些格子全是雷。";
    StepSatisfied: "Then this {} already has its {}, so this cell must be safe.", "再看这个 {}，它周围的 {}已经有了……所以这一格必定安全。";
    StepReduced: "This {} touches {} of those, so it acts like a {}.", "这个 {} 挨着其中 {}，减雷后它实际上是 {}。";
    StepExactly: "This {} still needs {}, and every hidden cell around it also touches that {}, so the shared cells hold exactly {}.", "看这个 {}，它周围还有 {}，这些格子都挨着那个 {}，所以共用的格子里恰好有 {}。";
    StepAtLeast: "This {} still needs {} but has only {} of its own, so the cells it shares with that {} hold at least {}.", "看这个 {}，它周围还有 {}，不共用的只有 {}，所以和那个 {} 共用的格子里最少有 {}。";
    StepAtMost: "This {} still needs {}, so the cells it shares with that {} hold at most {}.", "看这个 {}，它周围还有 {}，所以和那个 {} 共用的格子里最多只能有 {}。";
    StepRestSafe: "That {} still needs {}, no more than the shared cells hold, so this cell must be safe.", "再看那个 {}，它周围还有 {}，已经在共用的格子里了……所以这一格必定安全。";
    StepRestMine: "That {} still needs {}, but the shared cells hold at most {}, so the cells it doesn't share hold the rest: this cell must be a mine.", "再看那个 {}，它周围还有 {}，共用的格子里最多只有 {}，剩下的雷只能在不共用的格子里……所以这一格必须是雷。";
    StepContradiction: "If this flagged cell were a mine, these numbers could not all be right, so it must be safe.", "假设这一格是雷，这些数字就不可能都对……所以这一格必定安全。";
    StepFlagWrong: "The flag on it is wrong.", "所以这面旗插错了。";
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

    pub fn mode(m: Mode) -> S {
        match m {
            Mode::Classic => S::Classic,
            Mode::GentleNoGuess => S::GentleNoGuess,
            Mode::NoGuess => S::NoGuess,
        }
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
        assert_eq!([zh(S::Classic), zh(S::NoGuess)], ["原版", "无猜"]);
        assert_eq!(zh(S::GentleNoGuess), "简单无猜");
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
