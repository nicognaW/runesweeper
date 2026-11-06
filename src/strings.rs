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
    MaxGuess: "Max Guess", "死猜挑战";
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
    // Max Guess: the true 50/50s the field forces, before play and on the result banner.
    FiftyCount: "Forced 50/50s: {}", "死猜：{} 处";
    FiftySurvived: "Forced 50/50s survived: {} of {}", "闯过死猜：{} / {} 处";
    LostForced: "A forced 50/50 · survived {} of {}", "死猜，并非失误 · 闯过 {} / {} 处";
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
    // Review (复盘): the timeline under the field and its markers' callout. Reasons come
    // from the hint's own wording; nothing here gives a probability either.
    Review: "Review", "复盘";
    ReviewLeave: "Leave review", "退出复盘";
    ReviewPrev: "Previous move", "上一步";
    ReviewNext: "Next move", "下一步";
    ReviewTimeline: "Moves", "步骤";
    ReviewMove: "Move {} of {}", "第 {} / {} 步";
    ReviewStats: "3BV {} · 3BV/s {} · Efficiency {}", "3BV {} · 3BV/s {} · 效率 {}";
    ReviewClean: "Nothing to point out.", "没有需要指出的地方。";
    MarkNeedless: "Needless guess", "不必要的猜雷";
    MarkMissed: "Missed deduction", "错过的推理";
    MarkForced: "Forced guess", "必须猜";
    MarkChord: "A chord saves clicks", "双击可省点击";
    SumNeedless: "You opened a cell nothing proved, while a safe cell could be proven.", "这一步猜了一格，其实这时已经能推出安全格。";
    SumMissed: "You paused {} here, while a safe cell could be proven.", "你在这里停了 {}，其实这时已经能推出安全格。";
    SumForced: "No cell could be proven safe: the guess was forced, but not a true 50/50.", "推不出任何安全格，只能猜，但这不是死猜。";
    SumForcedFifty: "No cell could be proven safe: the guess was forced, a true 50/50.", "推不出任何安全格，只能猜：这是死猜。";
    ChordReason1: "This {} has its {} flagged, so the cells left around it are safe. One chord on it opens them all: 1 click saved.", "看这个 {}，它周围的 {}都已标好……所以剩下的格子都安全。在它上面双击一次就能全部打开，省下 1 次点击。";
    ChordReason: "This {} has its {} flagged, so the cells left around it are safe. One chord on it opens them all: {} clicks saved.", "看这个 {}，它周围的 {}都已标好……所以剩下的格子都安全。在它上面双击一次就能全部打开，省下 {} 次点击。";
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
    TechEdgeOneOne: "1-1 at the edge", "边部11定式";
    TechOneOnePlus: "1-1+ pattern", "多格11定式";
    TechOneTwoPlus: "1-2+ pattern", "多雷12定式";
    TechOneTwoC: "1-2C pattern", "交叉12定式";
    TechOneTwoCPlus: "1-2C+ pattern", "交叉多雷12定式";
    TechOneTwoOne: "1-2-1 pattern", "121定式";
    TechOneTwoTwoOne: "1-2-2-1 pattern", "1221定式";
    TechTwoOneTwo: "2-1-2 pattern", "212定式";
    TechEdgeTwoOneOne: "2-1-1 at the edge", "边部211定式";
    TechTwoTwoBetweenMines: "2-2 between mines", "夹雷22定式";
    TechNextToMineTwoTwoOne: "2-2-1 next to a mine", "靠雷221定式";
    TechNextToMineThreeOne: "3-1 next to a mine", "靠雷31定式";
    TechNextToMineFourOne: "4-1 next to mines", "靠雷41定式";
    TechHole1: "Hole H1", "挖坑定式 (1)";
    TechHole2: "Hole H2", "挖坑定式 (2)";
    TechHole3: "Hole H3", "挖坑定式 (3)";
    TechTriangle1: "Triangle T1", "三角定式 (1)";
    TechTriangle2: "Triangle T2", "三角定式 (2)";
    TechTriangle3: "Triangle T3", "三角定式 (3)";
    TechTriangle4: "Triangle T4", "三角定式 (4)";
    TechTriangle5: "Triangle T5", "三角定式 (5)";
    TechCorner131: "1-3-1 corner", "角131定式";
    TechCorner222: "2-2-2 corner", "角222定式";
    TechStaggered121: "1>2<1 pattern", "隔行的121定式";
    TechTPattern: "T-pattern", "T形定式";
    TechDependencyChain: "Dependency chain", "连续减法";
    TechReductionOneOne: "Reduction to a 1-1", "减雷变11定式";
    TechReductionOneTwo: "Reduction to a 1-2", "减雷变12定式";
    TechReductionOneTwoOne: "Reduction to a 1-2-1", "减雷变121定式";
    TechChain: "Chain of steps", "多步推理";
    TechSuppose: "Proof by contradiction", "假设法";
    TechCount: "Mine counting", "剩余雷数";
    TechCombinations: "Combinations", "穷举与假设";
    TechNothing: "No sure move", "没有确定的格子";
    NudgeWrongFlag: "One of your flags can't be right.", "有一面旗插错了。";
    NudgeSingle: "Look for a number with only as many hidden cells as it needs.", "找一个周围只剩下对应数量格子的数字。";
    NudgeSatisfied: "Look for a number that already has all its mines.", "找一个周围的雷已经够数的数字。";
    NudgeOneOne: "Compare two numbers that share hidden cells.", "比较两个共用格子的数字，找 11定式。";
    NudgeOneTwo: "Compare two numbers that share hidden cells, and their difference.", "比较两个共用格子的数字，找 12定式。";
    NudgeReduction: "Subtract the mines you can prove, then compare two numbers.", "先减雷，再比较两个数字。";
    NudgeRow: "Read the row of numbers as 1-2s, one from each end.", "把这一排数字拆成 12定式，从两头分别看。";
    NudgeHole: "Find a hole in the open cells: compare the number by it with the number past it.", "找一个坑：比较坑边的数字和坑另一头的数字。";
    NudgeTriangle: "Where the open cells come to a point, compare the number at the tip with the one next to it.", "在打开的区域收成尖角的地方，比较尖上的数字和它旁边的数字。";
    NudgeCorner: "Look at the number on the corner and the two numbers beside it.", "看拐角上的数字和它两边的数字。";
    NudgeStaggered: "Find a number whose mines must split between two numbers beside it.", "找一个数字，它的雷只能分到旁边两个数字里。";
    NudgeTPattern: "Add up how many mines the numbers around it allow in its cells.", "把周围几个数字允许的雷数加起来，看看还差几个。";
    NudgeDependencyChain: "Follow the chain from both ends, one group of cells at a time.", "从两头顺着推，一段一段确定雷的位置。";
    NudgeCombinations: "Try where the last mines could go, and count them.", "试一试剩下的雷可能在哪里，数一数够不够。";
    NudgeChain: "Work it out a step at a time: each step settles cells for the next.", "一步一步推：前一步确定的格子，帮下一步判雷。";
    NudgeSuppose: "Suppose a cell is a mine, or safe, and see which number breaks.", "假设某一格是雷或安全，看哪个数字出矛盾。";
    NudgeCount: "Count the mines left to find.", "根据剩余雷数来判断。";
    NudgeNothing: "No cell can be proven right now.", "现在推不出任何能确定的格子。";
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
    StepRestSafes: "That {} still needs {}, no more than the shared cells hold, so its other cells are safe.", "再看那个 {}，它周围还有 {}，已经在共用的格子里了……所以它不共用的格子都安全。";
    StepRestMines: "That {} still needs {}, but the shared cells hold at most {}, so the cells it doesn't share are all mines.", "再看那个 {}，它周围还有 {}，共用的格子里最多只有 {}，剩下的雷只能在不共用的格子里……所以这些格子全是雷。";
    StepNowMines: "This {} still needs {} and has only {} left, so they are all mines.", "这个 {} 还差 {}，只剩下 {}……所以这些格子全是雷。";
    StepNowMine1: "This {} still needs {} and has only {} left, so that cell is a mine.", "这个 {} 还差 {}，只剩下 {}……所以那一格是雷。";
    StepNowMine: "This {} still needs {} and has only {} left, so this cell must be a mine.", "看这个 {}，它还差 {}，只剩下 {}……所以这一格必须是雷。";
    StepNowSafes: "This {} already has its {}, so its other hidden cells are safe.", "这个 {} 周围的 {}已经够了……所以其余的格子都安全。";
    StepNowSafe: "This {} already has its {}, so this cell must be safe.", "看这个 {}，它周围的 {}已经够了……所以这一格必定安全。";
    StepSplitMost: "So that {} gets its {} only if each group of shared cells holds as many as it can.", "所以那个 {} 要凑够 {}，每一组共用的格子都必须放满。";
    StepSplitLeast: "So that {} has room for no more than its {} only if each group of shared cells holds as few as it must.", "所以那个 {} 只能有 {}，每一组共用的格子都只能放最少的雷。";
    StepSupposeMine: "Suppose this cell were a mine.", "假设这一格是雷。";
    StepSupposeSafe: "Suppose this cell were safe.", "假设这一格是安全的。";
    StepBrokenOver: "Then this {} would have too many mines.", "那么这个 {} 周围的雷就太多了。";
    StepBrokenUnder: "Then this {} could not get all its mines.", "那么这个 {} 周围的雷就不够了。";
    StepNoRoom: "Then these numbers could not all be right.", "那么这几个数字就不可能都对。";
    StepNoRoomCount: "Then these numbers and the mines left to find could not all fit.", "那么根据剩余雷数，这几个数字就不可能都对。";
    StepSoMine: "So this cell must be a mine.", "所以这一格必须是雷。";
    StepSoSafe: "So this cell must be safe.", "所以这一格必定安全。";
    StepCountNone: "Every mine is found, so every other hidden cell is safe.", "根据剩余雷数：雷已经全部找到了……所以其余的格子全部安全。";
    StepCountEvery: "As many mines are left to find as hidden cells, so every one is a mine.", "根据剩余雷数：剩下的雷和没打开的格子一样多……所以这些格子全是雷。";
    StepCountAll: "Only {} left to find, and these numbers need that many, so every other hidden cell is safe.", "根据剩余雷数：只剩 {}没找到，这几个数字周围就要用掉这么多……所以其余的格子全部安全。";
    StepCountAll1: "Only {} left to find, and this number needs that many, so every other hidden cell is safe.", "根据剩余雷数：只剩 {}没找到，这个数字周围就要用掉这么多……所以其余的格子全部安全。";
    StepCountRest: "Only {} left to find, and these numbers hold at most {}, so every other hidden cell is a mine.", "根据剩余雷数：还剩 {}没找到，这几个数字周围最多只有 {}……所以其余的格子全是雷。";
    StepCountRest1: "Only {} left to find, and this number holds at most {}, so every other hidden cell is a mine.", "根据剩余雷数：还剩 {}没找到，这个数字周围最多只有 {}……所以其余的格子全是雷。";
    StepCountOver: "Then these numbers would need at least {}, but only {} left to find.", "那么这几个数字周围最少要有 {}，可是根据剩余雷数，只剩 {}没找到。";
    StepCountUnder: "Then even with every other hidden cell a mine there would be at most {}, fewer than the {} left to find.", "那么就算其余的格子全是雷，最多也只有 {}，凑不够剩下的 {}。";
    // Guesses (ADR 0002): when no cell can be proven, reasons a player can check by hand.
    TechFiftyFifty: "True 50/50", "死猜";
    TechPseudo: "Pseudo 50/50", "伪死猜";
    TechDead: "Dead cell", "无信息格";
    TechWays: "Counting arrangements", "数摆法";
    TechFewer: "Fewer mines", "少雷原则";
    TechOpening: "Guess for an opening", "猜空";
    TechTopLeft: "Classic's top-left quirk", "原版首击移雷";
    TechBlind: "Forced guess", "必须猜";
    NudgeFiftyFifty: "No cell can be proven. Look for two cells nothing can ever tell apart.", "推不出能确定的格子。找两格永远分不出来的死猜。";
    NudgePseudo: "No cell can be proven. Look for a cell that is either safe or half of a 50/50.", "推不出能确定的格子。找一格：它要么安全，要么就是死猜的一半。";
    NudgeDead: "No cell can be proven. One cell would tell you nothing if you opened it.", "推不出能确定的格子。有一格就算打开了也不会给出新信息。";
    NudgeWays: "No cell can be proven. Count the ways the numbers can be satisfied.", "推不出能确定的格子。数一数这几个数字有几种摆法。";
    NudgeFewer: "No cell can be proven. Arrangements with fewer mines come up more often.", "推不出能确定的格子。雷少的摆法更常见。";
    NudgeOpening: "No cell can be proven. Look away from the numbers, where a cell may open an area.", "推不出能确定的格子。看看不挨着数字的格子，可能一下开出一片空。";
    NudgeTopLeft: "No cell can be proven. Remember where Classic moves a mine under the first click.", "推不出能确定的格子。想想原版把第一下点到的雷移到了哪里。";
    NudgeBlind: "No cell can be proven, and nothing you could check by hand favours any cell: a guess is needed here.", "推不出能确定的格子，也没有能手算的理由偏向哪一格：这里只能猜。";
    CellGuess: "Guess this cell.", "猜这一格。";
    CellAvoid: "Avoid this cell.", "别点这一格。";
    StepFiftyFifty: "These two cells hold one mine between them, and every number open now or later touches both or neither, so nothing can ever tell them apart. Guess now: waiting gains nothing.", "这两格里恰好有 1 个雷，现在和以后能打开的数字要么同时挨着这两格，要么都不挨着……永远分不出来，这是死猜。现在就猜，等下去也没用。";
    StepAnnounced1: "This is the 50/50 this field was built with.", "这就是本局开局时预告的那个死猜。";
    StepAnnounced: "This is one of the {} 50/50s this field was built with.", "这是本局开局时预告的 {} 个死猜之一。";
    StepPseudo: "If this cell is a mine, it and that one form a 50/50 nothing can split, since the cells that could tell them apart would all be mines. If it is safe, it opens. Either way, guessing it now costs nothing.", "如果这一格是雷，它和那一格就成了死猜：能分出它们的格子都会是雷。如果它不是雷，就能打开。所以现在猜它不吃亏。";
    StepDead: "If this cell is safe it can only show a {}, which tells you nothing new, so you would have to guess again. Avoid it.", "这一格如果安全，只能是 {}，打开了也没有新信息，还得再猜。别点它。";
    StepWaysOne: "This {} can be satisfied {} ways, all with the same number of mines, so each comes up as often. This cell is a mine in only {} of them.", "这个 {} 只有 {} 种摆法，雷数都一样，每种都一样常见。这一格只在其中 {} 种里是雷。";
    StepWaysMany: "These numbers can be satisfied {} ways, all with the same number of mines, so each comes up as often. This cell is a mine in only {} of them.", "这几个数字只有 {} 种摆法，雷数都一样，每种都一样常见。这一格只在其中 {} 种里是雷。";
    StepFewer: "These numbers can be satisfied with as few as {}, or with more. The other mines go among the {} away from the numbers, and each mine fewer here leaves more ways to place them, so arrangements with fewer mines here come up more often.", "这几个数字最少用 {}就能满足，也可以更多。其余的雷在不挨着数字的 {}里，这里每少一个雷，其余的雷就有更多摆法，所以雷少的摆法更常见。";
    StepFewerSafe: "This cell is safe in every arrangement with only {}.", "在只用 {}的摆法里，这一格都是安全的。";
    StepOpening: "No number touches this cell, so nothing makes it riskier than the rest: the {} left hold {}. With only {} neighbours, it is the most apt to be empty and open an area.", "这一格不挨着任何数字，没有额外的风险：剩下的 {}里有 {}。它只有 {} 个相邻格，最容易是空，一下开出一片。";
    StepTopLeft: "In Classic, as in Windows XP, a mine under the first click moves to the top-left corner (or the next free cell along the top row), so early on this cell hides a mine more often than it would otherwise. Avoid it.", "原版里，第一下点到的雷会被移到左上角（那里已有雷就往右移），所以开局时这一格比平常更容易是雷。别点它。";
    StepNotTopLeft: "Not the top-left corner, though: in Classic, as in Windows XP, a mine under the first click moves there (or along the top row), so early on it hides a mine more often.", "但别选左上角：原版里，第一下点到的雷会被移到那里（已有雷就往右移），所以开局时那一格更容易是雷。";
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
            Mode::MaxGuess => S::MaxGuess,
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
