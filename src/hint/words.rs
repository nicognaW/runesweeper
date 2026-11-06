//! A hint in words, from per-language templates in the string table (never translated word by
//! word). English avoids coordinates: the callout highlights what "this" and "that" are.
//! Nothing here, or anywhere in a hint, gives a probability (ADR 0002).

use super::{Bound, Grade, Hint, Pattern, Step, Technique, Verdict};
use crate::strings::{Lang, S};

/// "1 mine", "3 mines" (zh-CN: "3 个雷").
fn mines(lang: Lang, n: usize) -> String {
    if n == 1 { S::CountMine1.get(lang).to_string() } else { S::CountMines.fill(lang, &[&n]) }
}

/// "1 hidden cell", "3 hidden cells" (zh-CN: "3 格").
fn cells(lang: Lang, n: usize) -> String {
    if n == 1 { S::CountCell1.get(lang).to_string() } else { S::CountCells.fill(lang, &[&n]) }
}

/// The technique's name, as the community calls it.
pub fn name(t: Technique, lang: Lang) -> &'static str {
    match t {
        Technique::WrongFlag => S::TechWrongFlag,
        Technique::Single => S::TechSingle,
        Technique::Satisfied => S::TechSatisfied,
        Technique::Pattern(p) => pattern(p),
        Technique::Reduction(p) => reduction(p),
        Technique::Chain => S::TechChain,
        Technique::Suppose => S::TechSuppose,
        Technique::Count => S::TechCount,
        Technique::Combinations => S::TechCombinations,
        Technique::Nothing => S::TechNothing,
    }
    .get(lang)
}

/// A named pattern's name (zh-CN: as listed in `src/zh_cn_terms.md`).
fn pattern(p: Pattern) -> S {
    match p {
        Pattern::OneOne => S::TechOneOne,
        Pattern::EdgeOneOne => S::TechEdgeOneOne,
        Pattern::OneOnePlus => S::TechOneOnePlus,
        Pattern::OneTwo => S::TechOneTwo,
        Pattern::OneTwoPlus => S::TechOneTwoPlus,
        Pattern::OneTwoC => S::TechOneTwoC,
        Pattern::OneTwoCPlus => S::TechOneTwoCPlus,
        Pattern::OneTwoOne => S::TechOneTwoOne,
        Pattern::OneTwoTwoOne => S::TechOneTwoTwoOne,
        Pattern::TwoOneTwo => S::TechTwoOneTwo,
        Pattern::EdgeTwoOneOne => S::TechEdgeTwoOneOne,
        Pattern::TwoTwoBetweenMines => S::TechTwoTwoBetweenMines,
        Pattern::NextToMineTwoTwoOne => S::TechNextToMineTwoTwoOne,
        Pattern::NextToMineThreeOne => S::TechNextToMineThreeOne,
        Pattern::NextToMineFourOne => S::TechNextToMineFourOne,
        Pattern::Hole1 => S::TechHole1,
        Pattern::Hole2 => S::TechHole2,
        Pattern::Hole3 => S::TechHole3,
        Pattern::Triangle1 => S::TechTriangle1,
        Pattern::Triangle2 => S::TechTriangle2,
        Pattern::Triangle3 => S::TechTriangle3,
        Pattern::Triangle4 => S::TechTriangle4,
        Pattern::Triangle5 => S::TechTriangle5,
        Pattern::Corner131 => S::TechCorner131,
        Pattern::Corner222 => S::TechCorner222,
        Pattern::Staggered121 => S::TechStaggered121,
        Pattern::TPattern => S::TechTPattern,
        Pattern::DependencyChain => S::TechDependencyChain,
    }
}

/// A reduction, by the pattern it leads to, or by its Chinese name.
fn reduction(p: Pattern) -> S {
    match p {
        Pattern::OneOne => S::TechReductionOneOne,
        Pattern::OneTwoOne => S::TechReductionOneTwoOne,
        Pattern::TwoTwoBetweenMines | Pattern::NextToMineTwoTwoOne | Pattern::NextToMineThreeOne | Pattern::NextToMineFourOne => pattern(p),
        _ => S::TechReductionOneTwo,
    }
}

/// What to look for, for a named pattern.
fn look_for(p: Pattern) -> S {
    use Pattern::*;
    match p {
        OneOne | EdgeOneOne | OneOnePlus => S::NudgeOneOne,
        OneTwo | OneTwoPlus | OneTwoC | OneTwoCPlus => S::NudgeOneTwo,
        OneTwoOne | OneTwoTwoOne | TwoOneTwo | EdgeTwoOneOne => S::NudgeRow,
        TwoTwoBetweenMines | NextToMineTwoTwoOne | NextToMineThreeOne | NextToMineFourOne => S::NudgeReduction,
        Hole1 | Hole2 | Hole3 => S::NudgeHole,
        Triangle1 | Triangle2 | Triangle3 | Triangle4 | Triangle5 => S::NudgeTriangle,
        Corner131 | Corner222 => S::NudgeCorner,
        Staggered121 => S::NudgeStaggered,
        TPattern => S::NudgeTPattern,
        DependencyChain => S::NudgeDependencyChain,
    }
}

pub fn grade(g: Grade, lang: Lang) -> &'static str {
    match g {
        Grade::Easy => S::Easy,
        Grade::Medium => S::Medium,
        Grade::Hard => S::Hard,
        Grade::Evil => S::Evil,
    }
    .get(lang)
}

/// Stage 1: what to look for, with no place named.
pub fn nudge(h: &Hint, lang: Lang) -> &'static str {
    match h.technique {
        Technique::WrongFlag => S::NudgeWrongFlag,
        Technique::Single => S::NudgeSingle,
        Technique::Satisfied => S::NudgeSatisfied,
        Technique::Pattern(p) => look_for(p),
        Technique::Reduction(_) => S::NudgeReduction,
        Technique::Chain => S::NudgeChain,
        Technique::Suppose => S::NudgeSuppose,
        Technique::Count => S::NudgeCount,
        Technique::Combinations => S::NudgeCombinations,
        Technique::Nothing => S::NudgeNothing,
    }
    .get(lang)
}

/// Stage 2: the numbers, highlighted.
pub fn look(h: &Hint, lang: Lang) -> &'static str {
    if h.numbers.len() == 1 { S::LookOne.get(lang) } else { S::LookMany.get(lang) }
}

/// Stage 3: the cell and what it is.
pub fn verdict(h: &Hint, lang: Lang) -> Option<&'static str> {
    Some(
        match h.target?.verdict {
            Verdict::Safe => S::CellSafe,
            Verdict::Mine => S::CellMine,
            Verdict::WrongFlag => S::CellWrongFlag,
        }
        .get(lang),
    )
}

/// Stage 4: the reason, one sentence per step.
pub fn reason(h: &Hint, lang: Lang) -> Vec<String> {
    let last = h.steps.len().saturating_sub(1);
    h.steps.iter().enumerate().map(|(i, s)| step(s, i == last, lang)).collect()
}

fn step(s: &Step, last: bool, lang: Lang) -> String {
    match *s {
        // The number the cell is read from: it needs every hidden cell.
        Step::Full { n, ref cells, .. } if last => S::StepSingle.fill(lang, &[&n, &mines(lang, cells.len()), &self::cells(lang, cells.len())]),
        // A number proving mines for a later step.
        Step::Full { n, ref cells, .. } if cells.len() == 1 => S::StepProver1.fill(lang, &[&n]),
        Step::Full { n, ref cells, .. } => S::StepProver.fill(lang, &[&n, &self::cells(lang, cells.len())]),
        Step::Reduced { n, known, .. } => S::StepReduced.fill(lang, &[&n, &mines(lang, known), &(n as usize - known)]),
        Step::Satisfied { n, .. } => S::StepSatisfied.fill(lang, &[&n, &mines(lang, n as usize)]),
        Step::Shared { n, need, own, other_n, bound, .. } => match bound {
            Bound::Exactly(k) => S::StepExactly.fill(lang, &[&n, &mines(lang, need), &other_n, &mines(lang, k)]),
            Bound::AtLeast(k) => S::StepAtLeast.fill(lang, &[&n, &mines(lang, need), &self::cells(lang, own), &other_n, &mines(lang, k)]),
            Bound::AtMost(k) => S::StepAtMost.fill(lang, &[&n, &mines(lang, need), &other_n, &mines(lang, k)]),
        },
        Step::Rest { n, need, mine: false, .. } => (if last { S::StepRestSafe } else { S::StepRestSafes }).fill(lang, &[&n, &mines(lang, need)]),
        Step::Rest { n, need, held, mine: true, .. } => (if last { S::StepRestMine } else { S::StepRestMines }).fill(lang, &[&n, &mines(lang, need), &mines(lang, held)]),
        Step::Contradiction { .. } => S::StepContradiction.get(lang).to_string(),
        Step::FlagWrong { .. } => S::StepFlagWrong.get(lang).to_string(),
        Step::Now { n, need, ref cells, mine: true, .. } => (if last { S::StepNowMine } else if cells.len() == 1 { S::StepNowMine1 } else { S::StepNowMines }).fill(lang, &[&n, &mines(lang, need), &self::cells(lang, cells.len())]),
        Step::Now { n, mine: false, .. } => (if last { S::StepNowSafe } else { S::StepNowSafes }).fill(lang, &[&n, &mines(lang, n as usize)]),
        Step::Split { n, need, most, .. } => if most { S::StepSplitMost } else { S::StepSplitLeast }.fill(lang, &[&n, &mines(lang, need)]),
        Step::Suppose { mine, .. } => if mine { S::StepSupposeMine } else { S::StepSupposeSafe }.get(lang).to_string(),
        Step::Broken { n, over, .. } => if over { S::StepBrokenOver } else { S::StepBrokenUnder }.fill(lang, &[&n]),
        Step::NoRoom { count } => if count { S::StepNoRoomCount } else { S::StepNoRoom }.get(lang).to_string(),
        Step::So { mine } => if mine { S::StepSoMine } else { S::StepSoSafe }.get(lang).to_string(),
        Step::CountAll { left: 0, .. } => S::StepCountNone.get(lang).to_string(),
        Step::CountAll { left, numbers } => if numbers == 1 { S::StepCountAll1 } else { S::StepCountAll }.fill(lang, &[&mines(lang, left)]),
        Step::CountRest { numbers: 0, .. } => S::StepCountEvery.get(lang).to_string(),
        Step::CountRest { left, held, numbers } => if numbers == 1 { S::StepCountRest1 } else { S::StepCountRest }.fill(lang, &[&mines(lang, left), &mines(lang, held)]),
        Step::CountOver { need, left } => S::StepCountOver.fill(lang, &[&mines(lang, need), &mines(lang, left)]),
        Step::CountUnder { most, left } => S::StepCountUnder.fill(lang, &[&mines(lang, most), &mines(lang, left)]),
    }
}
