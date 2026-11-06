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
        Technique::Pattern(Pattern::OneOne) => S::TechOneOne,
        Technique::Pattern(Pattern::OneTwo) => S::TechOneTwo,
        Technique::Reduction(Pattern::OneOne) => S::TechReductionOneOne,
        Technique::Reduction(Pattern::OneTwo) => S::TechReductionOneTwo,
        Technique::Nothing => S::TechNothing,
    }
    .get(lang)
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
        Technique::Pattern(Pattern::OneOne) => S::NudgeOneOne,
        Technique::Pattern(Pattern::OneTwo) => S::NudgeOneTwo,
        Technique::Reduction(_) => S::NudgeReduction,
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
        Step::Rest { n, need, mine: false, .. } => S::StepRestSafe.fill(lang, &[&n, &mines(lang, need)]),
        Step::Rest { n, need, held, mine: true, .. } => S::StepRestMine.fill(lang, &[&n, &mines(lang, need), &mines(lang, held)]),
        Step::Contradiction { .. } => S::StepContradiction.get(lang).to_string(),
        Step::FlagWrong { .. } => S::StepFlagWrong.get(lang).to_string(),
    }
}
