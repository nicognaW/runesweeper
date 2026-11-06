//! The named pattern catalogue on fixed positions: for each of minesweeper.online's 28
//! patterns and each Chinese 定式, the hint names it and its grade, reads exactly its numbers,
//! names its cell, and gives its reason in both languages.
//!
//! The holes, the triangles, the 1-3-1 corner and the T-pattern follow minesweeper.online's
//! diagrams (their numbers and hidden cells; the field around them is filled in so it is
//! consistent and holds nothing simpler). The 2-2-2 corner, 1>2<1 and the dependency chain
//! follow its descriptions, the Chinese row patterns 张砷镓's 扫雷定式及变化. A position is drawn
//! as in the hint tests; `last` is the player's last move, (x, y).

use super::*;

struct Fixture {
    name: &'static str,
    rows: &'static [&'static str],
    last: Option<(usize, usize)>,
    technique: Technique,
    grade: Grade,
    /// In English and zh-CN.
    names: [&'static str; 2],
    /// (x, y), in the order the reason reads them.
    numbers: &'static [(usize, usize)],
    target: ((usize, usize), Verdict),
    en: &'static [&'static str],
    zh: &'static [&'static str],
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "b1_single",
        rows: &["..*#", "...."],
        last: None,
        technique: Technique::Single,
        grade: Grade::Easy,
        names: ["Single number", "单数字判雷"],
        numbers: &[(1, 0)],
        target: ((2, 0), Verdict::Mine),
        en: &["This 1 still needs 1 mine and touches only 1 hidden cell, so this cell must be a mine."],
        zh: &["看这个 1，它周围还有 1 个雷，只剩下 1 格……所以这一格必须是雷。"],
    },
    Fixture {
        name: "b2_satisfied",
        rows: &["..F#", "...."],
        last: Some((3, 1)),
        technique: Technique::Satisfied,
        grade: Grade::Easy,
        names: ["Satisfied number", "单数字判空"],
        numbers: &[(1, 0), (3, 1)],
        target: ((3, 0), Verdict::Safe),
        en: &["First, this 1 touches only one hidden cell, so that cell is a mine.", "Then this 1 already has its 1 mine, so this cell must be safe."],
        zh: &["先看这个 1，它周围只剩下 1 格，所以那一格是雷。", "再看这个 1，它周围的 1 个雷已经有了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_one",
        rows: &["......*", "......#", "#*.#*##", "#######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneOne),
        grade: Grade::Medium,
        names: ["1-1 pattern", "11定式"],
        numbers: &[(3, 1), (4, 1)],
        target: ((5, 2), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "edge_one_one",
        rows: &["*##*#", "....."],
        last: Some((0, 1)),
        technique: Technique::Pattern(Pattern::EdgeOneOne),
        grade: Grade::Medium,
        names: ["1-1 at the edge", "边部11定式"],
        numbers: &[(0, 1), (1, 1)],
        target: ((2, 0), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_one_plus",
        rows: &["......*", "......#", "*#.*###", "#######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneOnePlus),
        grade: Grade::Medium,
        names: ["1-1+ pattern", "多格11定式"],
        numbers: &[(5, 0), (5, 1)],
        target: ((4, 2), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_two",
        rows: &["......", "......", "*#**#*", "######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwo),
        grade: Grade::Medium,
        names: ["1-2 pattern", "12定式"],
        numbers: &[(0, 1), (1, 1)],
        target: ((2, 2), Verdict::Mine),
        en: &["This 1 still needs 1 mine, so the cells it shares with that 2 hold at most 1 mine.", "That 2 still needs 2 mines, but the shared cells hold at most 1 mine, so the cells it doesn't share hold the rest: this cell must be a mine."],
        zh: &["看这个 1，它周围还有 1 个雷，所以和那个 2 共用的格子里最多只能有 1 个雷。", "再看那个 2，它周围还有 2 个雷，共用的格子里最多只有 1 个雷，剩下的雷只能在不共用的格子里……所以这一格必须是雷。"],
    },
    Fixture {
        name: "one_two_plus",
        rows: &["......*", "......#", "#*.#***", "#*#####"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwoPlus),
        grade: Grade::Medium,
        names: ["1-2+ pattern", "多雷12定式"],
        numbers: &[(5, 0), (5, 1)],
        target: ((4, 2), Verdict::Mine),
        en: &["This 1 still needs 1 mine, so the cells it shares with that 4 hold at most 1 mine.", "That 4 still needs 4 mines, but the shared cells hold at most 1 mine, so the cells it doesn't share hold the rest: this cell must be a mine."],
        zh: &["看这个 1，它周围还有 1 个雷，所以和那个 4 共用的格子里最多只能有 1 个雷。", "再看那个 4，它周围还有 4 个雷，共用的格子里最多只有 1 个雷，剩下的雷只能在不共用的格子里……所以这一格必须是雷。"],
    },
    Fixture {
        name: "one_two_c",
        rows: &[".......", ".......", "*##**#*", "#######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwoC),
        grade: Grade::Medium,
        names: ["1-2C pattern", "交叉12定式"],
        numbers: &[(3, 1), (2, 1)],
        target: ((1, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_two_c_plus",
        rows: &["###*#*#", "*....*#", "#....*#", "#....##", "##*####"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwoCPlus),
        grade: Grade::Medium,
        names: ["1-2C+ pattern", "交叉多雷12定式"],
        numbers: &[(4, 1), (3, 1)],
        target: ((2, 0), Verdict::Safe),
        en: &["This 4 still needs 4 mines but has only 3 hidden cells of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 4，它周围还有 4 个雷，不共用的只有 3 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_two_one",
        rows: &["......", "......", "*#*#*#", "######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwoOne),
        grade: Grade::Medium,
        names: ["1-2-1 pattern", "121定式"],
        numbers: &[(3, 1), (2, 1)],
        target: ((1, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "one_two_two_one",
        rows: &["......", "......", "*##**#", "######"],
        last: None,
        technique: Technique::Pattern(Pattern::OneTwoTwoOne),
        grade: Grade::Medium,
        names: ["1-2-2-1 pattern", "1221定式"],
        numbers: &[(3, 1), (2, 1)],
        target: ((1, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "two_one_two",
        rows: &[".......", ".......", "*#*#**#", "#######"],
        last: None,
        technique: Technique::Pattern(Pattern::TwoOneTwo),
        grade: Grade::Medium,
        names: ["2-1-2 pattern", "212定式"],
        numbers: &[(3, 1), (2, 1)],
        target: ((1, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "edge_two_one_one",
        rows: &["......", "......", "#*#**#", "######"],
        last: None,
        technique: Technique::Pattern(Pattern::EdgeTwoOneOne),
        grade: Grade::Medium,
        names: ["2-1-1 at the edge", "边部211定式"],
        numbers: &[(2, 1), (1, 1)],
        target: ((0, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "reduction_one_one",
        rows: &["##*", "F..", "F.."],
        last: None,
        technique: Technique::Reduction(Pattern::OneOne),
        grade: Grade::Hard,
        names: ["Reduction to a 1-1", "减雷变11定式"],
        numbers: &[(1, 2), (2, 1), (1, 1)],
        target: ((0, 0), Verdict::Safe),
        en: &["First, this 2 touches only 2 hidden cells, so they are all mines.", "This 3 touches 2 mines of those, so it acts like a 1.", "This 1 still needs 1 mine, and every hidden cell around it also touches that 3, so the shared cells hold exactly 1 mine.", "That 3 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["先看这个 2，它周围只剩下 2 格，所以这些格子全是雷。", "这个 3 挨着其中 2 个雷，减雷后它实际上是 1。", "看这个 1，它周围还有 1 个雷，这些格子都挨着那个 3，所以共用的格子里恰好有 1 个雷。", "再看那个 3，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "reduction_one_two",
        rows: &["......", "F.....", "#F#FF#", "######"],
        last: None,
        technique: Technique::Reduction(Pattern::OneTwo),
        grade: Grade::Hard,
        names: ["Reduction to a 1-2", "减雷变12定式"],
        numbers: &[(0, 0), (2, 1), (1, 1)],
        target: ((0, 2), Verdict::Safe),
        en: &["First, this 1 touches only one hidden cell, so that cell is a mine.", "This 2 touches 1 mine of those, so it acts like a 1.", "This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 1 mine.", "That 2 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["先看这个 1，它周围只剩下 1 格，所以那一格是雷。", "这个 2 挨着其中 1 个雷，减雷后它实际上是 1。", "看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 1 个雷。", "再看那个 2，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "reduction_one_two_one",
        rows: &["..F....", ".......", "#F#F#F#", "#######"],
        last: None,
        technique: Technique::Reduction(Pattern::OneTwoOne),
        grade: Grade::Hard,
        names: ["Reduction to a 1-2-1", "减雷变121定式"],
        numbers: &[(2, 1), (1, 1)],
        target: ((0, 2), Verdict::Safe),
        en: &["This 3 still needs 3 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 2 mines.", "That 2 still needs 2 mines, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 3，它周围还有 3 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 2 个雷。", "再看那个 2，它周围还有 2 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "two_two_between_mines",
        rows: &[".....", "F..F.", ".*##*", "#####"],
        last: None,
        technique: Technique::Reduction(Pattern::TwoTwoBetweenMines),
        grade: Grade::Hard,
        names: ["2-2 between mines", "夹雷22定式"],
        numbers: &[(2, 0), (1, 1), (2, 1)],
        target: ((3, 2), Verdict::Safe),
        en: &["First, this 1 touches only one hidden cell, so that cell is a mine.", "This 2 touches 1 mine of those, so it acts like a 1.", "This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 1 mine.", "That 2 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["先看这个 1，它周围只剩下 1 格，所以那一格是雷。", "这个 2 挨着其中 1 个雷，减雷后它实际上是 1。", "看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 1 个雷。", "再看那个 2，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "next_to_mine_two_two_one",
        rows: &[".....F", "F....#", "#F#F##", "######"],
        last: None,
        technique: Technique::Reduction(Pattern::NextToMineTwoTwoOne),
        grade: Grade::Hard,
        names: ["2-2-1 next to a mine", "靠雷221定式"],
        numbers: &[(2, 1), (3, 1)],
        target: ((4, 2), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "next_to_mine_three_one",
        rows: &["......", "F.....", "FF##F#", "######"],
        last: None,
        technique: Technique::Reduction(Pattern::NextToMineThreeOne),
        grade: Grade::Hard,
        names: ["3-1 next to a mine", "靠雷31定式"],
        numbers: &[(1, 1), (2, 1)],
        target: ((3, 2), Verdict::Safe),
        en: &["This 3 still needs 3 mines but has only 2 hidden cells of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 3，它周围还有 3 个雷，不共用的只有 2 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "next_to_mine_four_one",
        rows: &["F.....", "F.....", "FF##F#", "######"],
        last: None,
        technique: Technique::Reduction(Pattern::NextToMineFourOne),
        grade: Grade::Hard,
        names: ["4-1 next to mines", "靠雷41定式"],
        numbers: &[(1, 1), (2, 1)],
        target: ((3, 2), Verdict::Safe),
        en: &["This 4 still needs 4 mines but has only 3 hidden cells of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 4，它周围还有 4 个雷，不共用的只有 3 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "hole_1",
        rows: &["#####", "#####", "#*.#*", ".....", "....."],
        last: None,
        technique: Technique::Pattern(Pattern::Hole1),
        grade: Grade::Hard,
        names: ["Hole H1", "挖坑定式 (1)"],
        numbers: &[(2, 3), (2, 2)],
        target: ((1, 1), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "hole_2",
        rows: &["####*", "*...*", "#*.#*", ".....", "....."],
        last: None,
        technique: Technique::Pattern(Pattern::Hole2),
        grade: Grade::Hard,
        names: ["Hole H2", "挖坑定式 (2)"],
        numbers: &[(2, 2), (2, 1)],
        target: ((1, 0), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "hole_3",
        rows: &["#*###", "*...#", "##.*#", ".....", "....."],
        last: Some((3, 3)),
        technique: Technique::Pattern(Pattern::Hole3),
        grade: Grade::Hard,
        names: ["Hole H3", "挖坑定式 (3)"],
        numbers: &[(3, 3), (3, 1)],
        target: ((2, 0), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "triangle_1",
        rows: &["#####", "#####", "##.*#", "##...", "*....", "#*...", "###*#"],
        last: Some((2, 2)),
        technique: Technique::Pattern(Pattern::Triangle1),
        grade: Grade::Evil,
        names: ["Triangle T1", "三角定式 (1)"],
        numbers: &[(2, 3), (2, 2)],
        target: ((1, 1), Verdict::Safe),
        en: &["This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "triangle_2",
        rows: &["#####", "#####", "##.*#", "##...", "#*...", "##...", "###*#"],
        last: Some((2, 2)),
        technique: Technique::Pattern(Pattern::Triangle2),
        grade: Grade::Evil,
        names: ["Triangle T2", "三角定式 (2)"],
        numbers: &[(2, 3), (2, 2)],
        target: ((1, 1), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "triangle_3",
        rows: &["#####", "#####", "##.*#", "#*...", "#*...", "##...", "###*#"],
        last: Some((2, 2)),
        technique: Technique::Pattern(Pattern::Triangle3),
        grade: Grade::Evil,
        names: ["Triangle T3", "三角定式 (3)"],
        numbers: &[(2, 3), (2, 2)],
        target: ((1, 1), Verdict::Safe),
        en: &["This 3 still needs 3 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 2 mines.", "That 2 still needs 2 mines, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 3，它周围还有 3 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 2 个雷。", "再看那个 2，它周围还有 2 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "triangle_4",
        rows: &["#####", "#***#", "#...#", "##.*#", "##...", "*....", "#*...", "###*#"],
        last: Some((2, 2)),
        technique: Technique::Pattern(Pattern::Triangle4),
        grade: Grade::Evil,
        names: ["Triangle T4", "三角定式 (4)"],
        numbers: &[(2, 2), (1, 2)],
        target: ((0, 1), Verdict::Safe),
        en: &["This 4 still needs 4 mines but has only 2 hidden cells of its own, so the cells it shares with that 2 hold at least 2 mines.", "That 2 still needs 2 mines, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 4，它周围还有 4 个雷，不共用的只有 2 格，所以和那个 2 共用的格子里最少有 2 个雷。", "再看那个 2，它周围还有 2 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "triangle_5",
        rows: &["#####", "*###*", "*...#", "*#.*#", "#*...", "#....", "##...", "###*#"],
        last: Some((2, 2)),
        technique: Technique::Pattern(Pattern::Triangle5),
        grade: Grade::Evil,
        names: ["Triangle T5", "三角定式 (5)"],
        numbers: &[(2, 3), (2, 2)],
        target: ((1, 1), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 1 hold at least 1 mine.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 1 共用的格子里最少有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "corner_131",
        rows: &["#######", "**##**#", "#....*#", "#....##", "*....##", "#....*#", "###*###"],
        last: None,
        technique: Technique::Pattern(Pattern::Corner131),
        grade: Grade::Evil,
        names: ["1-3-1 corner", "角131定式"],
        numbers: &[(3, 2), (4, 3), (4, 2)],
        target: ((2, 1), Verdict::Safe),
        en: &["This 1 still needs 1 mine, so the cells it shares with that 3 hold at most 1 mine.", "This 1 still needs 1 mine, so the cells it shares with that 3 hold at most 1 mine.", "That 3 still needs 3 mines, but the shared cells hold at most 2 mines, so the cells it doesn't share are all mines.", "So that 3 gets its 2 mines only if each group of shared cells holds as many as it can.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，所以和那个 3 共用的格子里最多只能有 1 个雷。", "看这个 1，它周围还有 1 个雷，所以和那个 3 共用的格子里最多只能有 1 个雷。", "再看那个 3，它周围还有 3 个雷，共用的格子里最多只有 2 个雷，剩下的雷只能在不共用的格子里……所以这些格子全是雷。", "所以那个 3 要凑够 2 个雷，每一组共用的格子都必须放满。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "corner_222",
        rows: &["#######", "*#**###", "#....##", "#....*#", "*....*#", "#....##", "###*###"],
        last: None,
        technique: Technique::Pattern(Pattern::Corner222),
        grade: Grade::Evil,
        names: ["2-2-2 corner", "角222定式"],
        numbers: &[(3, 2), (4, 3), (4, 2)],
        target: ((5, 1), Verdict::Safe),
        en: &["This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 1 mine.", "This 2 still needs 2 mines but has only 1 hidden cell of its own, so the cells it shares with that 2 hold at least 1 mine.", "That 2 still needs 2 mines, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 1 个雷。", "看这个 2，它周围还有 2 个雷，不共用的只有 1 格，所以和那个 2 共用的格子里最少有 1 个雷。", "再看那个 2，它周围还有 2 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "staggered_121",
        rows: &["#######", "###*###", "#.#.#.#", "##*.*##", "#.....#", "#######"],
        last: None,
        technique: Technique::Pattern(Pattern::Staggered121),
        grade: Grade::Evil,
        names: ["1>2<1 pattern", "隔行的121定式"],
        numbers: &[(1, 2), (5, 2), (3, 3)],
        target: ((0, 1), Verdict::Safe),
        en: &["This 1 still needs 1 mine, so the cells it shares with that 2 hold at most 1 mine.", "This 1 still needs 1 mine, so the cells it shares with that 2 hold at most 1 mine.", "So that 2 gets its 2 mines only if each group of shared cells holds as many as it can.", "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe."],
        zh: &["看这个 1，它周围还有 1 个雷，所以和那个 2 共用的格子里最多只能有 1 个雷。", "看这个 1，它周围还有 1 个雷，所以和那个 2 共用的格子里最多只能有 1 个雷。", "所以那个 2 要凑够 2 个雷，每一组共用的格子都必须放满。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"],
    },
    Fixture {
        name: "t_pattern",
        rows: &["#####", "#**##", "#...#", "*#.*#", ".....", "....."],
        last: None,
        technique: Technique::Pattern(Pattern::TPattern),
        grade: Grade::Evil,
        names: ["T-pattern", "T形定式"],
        numbers: &[(3, 2), (2, 2), (3, 4), (2, 3)],
        target: ((4, 1), Verdict::Safe),
        en: &["Suppose this cell were a mine.", "This 2 touches 1 mine of those, so it acts like a 1.", "This 3 still needs 3 mines but has only 2 hidden cells of its own, so the cells it shares with that 2 hold at least 1 mine.", "That 2 still needs 1 mine, no more than the shared cells hold, so its other cells are safe.", "This 1 still needs 1 mine and has only 1 hidden cell left, so that cell is a mine.", "This 2 already has its 2 mines, so its other hidden cells are safe.", "This 3 still needs 2 mines and has only 2 hidden cells left, so they are all mines.", "Then this 1 would have too many mines.", "So this cell must be safe."],
        zh: &["假设这一格是雷。", "这个 2 挨着其中 1 个雷，减雷后它实际上是 1。", "看这个 3，它周围还有 3 个雷，不共用的只有 2 格，所以和那个 2 共用的格子里最少有 1 个雷。", "再看那个 2，它周围还有 1 个雷，已经在共用的格子里了……所以它不共用的格子都安全。", "这个 1 还差 1 个雷，只剩下 1 格……所以那一格是雷。", "这个 2 周围的 2 个雷已经够了……所以其余的格子都安全。", "这个 3 还差 2 个雷，只剩下 2 格……所以这些格子全是雷。", "那么这个 1 周围的雷就太多了。", "所以这一格必定安全。"],
    },
    Fixture {
        name: "dependency_chain",
        rows: &["#F#FF#F#", "#......#", "########"],
        last: None,
        technique: Technique::Pattern(Pattern::DependencyChain),
        grade: Grade::Evil,
        names: ["Dependency chain", "连续减法"],
        numbers: &[(1, 1), (2, 1), (3, 1), (5, 1), (6, 1)],
        target: ((0, 0), Verdict::Safe),
        en: &["Suppose this cell were a mine.", "Then these numbers could not all be right.", "So this cell must be safe."],
        zh: &["假设这一格是雷。", "那么这几个数字就不可能都对。", "所以这一格必定安全。"],
    },
    Fixture {
        name: "mine_counting",
        rows: &["#.#.#", "##*.*"],
        last: None,
        technique: Technique::Count,
        grade: Grade::Evil,
        names: ["Mine counting", "剩余雷数"],
        numbers: &[(3, 0)],
        target: ((0, 0), Verdict::Safe),
        en: &["Only 2 mines left to find, and this number needs that many, so every other hidden cell is safe."],
        zh: &["根据剩余雷数：只剩 2 个雷没找到，这个数字周围就要用掉这么多……所以其余的格子全部安全。"],
    },
    Fixture {
        name: "combinations",
        rows: &["*#.##.", "#..**."],
        last: None,
        technique: Technique::Combinations,
        grade: Grade::Evil,
        names: ["Combinations", "穷举与假设"],
        numbers: &[(2, 0), (5, 0), (1, 1)],
        target: ((1, 0), Verdict::Safe),
        en: &["Suppose this cell were a mine.", "Then even with every other hidden cell a mine there would be at most 2 mines, fewer than the 3 mines left to find.", "So this cell must be safe."],
        zh: &["假设这一格是雷。", "那么就算其余的格子全是雷，最多也只有 2 个雷，凑不够剩下的 3 个雷。", "所以这一格必定安全。"],
    },
];

fn xy(p: &Pos, i: usize) -> (usize, usize) {
    (i % p.w, i / p.w)
}

#[test]
fn every_fixture_names_its_pattern_its_numbers_and_its_reason() {
    for f in FIXTURES {
        let p = pos(f.rows);
        let h = p.hint(f.last);
        let name = f.name;
        assert_eq!(h.technique, f.technique, "{name}: {h:?}");
        assert_eq!(h.grade(), Some(f.grade), "{name}");
        assert_eq!([words::name(h.technique, Lang::En), words::name(h.technique, Lang::ZhCn)], f.names, "{name}");
        assert_eq!(h.numbers.iter().map(|&i| xy(&p, i)).collect::<Vec<_>>(), f.numbers, "{name}");
        let t = h.target.expect("a cell");
        assert_eq!((xy(&p, t.cell), t.verdict), f.target, "{name}");
        assert_eq!(en(&h), f.en, "{name}");
        assert_eq!(zh(&h), f.zh, "{name}");
        // The nudge names what to look for in both languages.
        assert!(!words::nudge(&h, Lang::En).is_empty() && !words::nudge(&h, Lang::ZhCn).is_empty());
    }
}

#[test]
fn every_catalogue_pattern_has_a_fixture() {
    let named = |p: Pattern| FIXTURES.iter().any(|f| f.technique == Technique::Pattern(p) || f.technique == Technique::Reduction(p));
    for p in Pattern::ALL {
        assert!(named(p), "{p:?} has no fixture");
    }
    // The basic rules, the reductions and the last turns of minesweeper.online's catalogue.
    for t in [
        Technique::Single,
        Technique::Satisfied,
        Technique::Reduction(Pattern::OneOne),
        Technique::Reduction(Pattern::OneTwo),
        Technique::Reduction(Pattern::OneTwoOne),
        Technique::Count,
        Technique::Combinations,
    ] {
        assert!(FIXTURES.iter().any(|f| f.technique == t), "{t:?} has no fixture");
    }
}

#[test]
fn a_pattern_and_raw_logic_proving_the_same_cell_name_the_pattern() {
    // In each, the numbers alone prove the cell by plain logic, as any deduction does; the
    // hint calls it by its pattern.
    for name in ["hole_1", "triangle_2", "corner_131", "staggered_121", "t_pattern"] {
        let f = FIXTURES.iter().find(|f| f.name == name).unwrap();
        let p = pos(f.rows);
        let h = p.hint(f.last);
        let t = h.target.unwrap();
        assert_eq!(super::forces(&p.v, &h.numbers, &[], false, t.cell, t.verdict == Verdict::Mine), Some(true), "{name}");
        assert!(h.technique.named(), "{name}: {h:?}");
    }
}

#[test]
fn the_zh_cn_term_table_lists_every_pattern() {
    let table = include_str!("../../zh_cn_terms.md");
    let techniques = Pattern::ALL.iter().flat_map(|&p| [Technique::Pattern(p), Technique::Reduction(p)]).chain([Technique::Single, Technique::Satisfied, Technique::Chain, Technique::Suppose, Technique::Count, Technique::Combinations]);
    for t in techniques {
        let (en, zh) = (words::name(t, Lang::En), words::name(t, Lang::ZhCn));
        assert!(table.lines().any(|l| l.contains(&format!("| {en} |")) && l.contains(&format!("| {zh} |"))), "{en} / {zh} is not in src/zh_cn_terms.md");
    }
}
