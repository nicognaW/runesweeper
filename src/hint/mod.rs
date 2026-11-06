//! The Hint: from what the player can see, the next step a strong player would take and the
//! reason for it, as data. Wording lives in [`words`]; drawing in `callout.rs`.
//!
//! [`find`] is a pure function of the open numbers and the player's flags. It reasons from the
//! numbers only (ADR 0002): flags are never facts, they only decide which cells are worth
//! naming (a flag on a proven mine needs no hint; a flag on a proven safe cell is wrong, and is
//! named first).
//!
//! Each hint carries its technique (and so its grade), the numbers it reads (its witnesses, in
//! the order the reason reads them), the cell it names with its verdict, and the reason as
//! [`Step`]s, so later techniques (named patterns, guesses) add steps and techniques rather
//! than new shapes.
//!
//! Deductions are named from the pattern catalogue in [`patterns`] (minesweeper.online's
//! patterns and the Chinese 定式) when their shape matches one; a named pattern is preferred to
//! plain logic on as many numbers.
//!
//! Any cell perfect logic proves gets a hint: deductions from more than two numbers, and from
//! the count of mines left, come from [`logic`], which finds the fewest numbers that prove the
//! cell and explains them one line per step.
//!
//! When no cell can be proven, [`advise`] adds the [`guess`] tier: how a strong player would
//! choose, from reasons a player can check by hand, never a probability.

pub mod guess;
mod logic;
mod patterns;
pub mod words;

pub use patterns::Pattern;

use crate::game::{Cover, Game};
use crate::solver::{Constraint, Fact, View, constraints};

/// How a hint's cell is worked out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Technique {
    /// A flag on a cell the numbers prove safe.
    WrongFlag,
    /// One number with only as many hidden cells as it needs: all mines.
    Single,
    /// A number whose mines are all proven by other numbers: its other cells are safe.
    Satisfied,
    /// A named pattern: two numbers sharing hidden cells, or a named shape of more.
    Pattern(Pattern),
    /// Mines proven by single numbers subtracted from two numbers, which then form a pattern
    /// (named for it, or by its Chinese name); also two numbers touching such mines, named as
    /// the reduction a player who flagged them sees.
    Reduction(Pattern),
    /// Any number of numbers, one or two at a time, each step settling cells for the next.
    Chain,
    /// Supposing the cell were the other way, a number breaks.
    Suppose,
    /// The count of mines left, with the numbers (mine counting).
    Count,
    /// Supposing the cell were the other way, the mines left could not all fit
    /// (combinations).
    Combinations,
    /// Nothing perfect logic proves.
    Nothing,
    /// No cell can be proven: how to choose the guess.
    Guess(Guess),
}

/// Why a guess is chosen, in the order a hint gives them: the first that holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Guess {
    /// Two cells nothing can ever tell apart: take it now.
    FiftyFifty,
    /// A cell that is either safe or half of a true 50/50: take it now.
    Pseudo,
    /// A cell that, if safe, would show a number that settles nothing: avoid it.
    Dead,
    /// A few arrangements, as common as each other, and the cell a mine in few of them.
    Ways,
    /// Arrangements with fewer mines come up more often, and the cell is safe in those.
    Fewer,
    /// A cell away from the numbers, on the edge, apt to open an area.
    Opening,
    /// Classic's first-click mine moved to the top left: avoid it early on.
    TopLeft,
    /// Nothing favours any cell: a guess is needed, and no cell is named.
    Blind,
}

/// How hard a technique is to see, on minesweeper.online's No Guess scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    Easy,
    Medium,
    Hard,
    Evil,
}

impl Technique {
    /// Whether it is a named pattern, which a hint prefers.
    pub fn named(self) -> bool {
        matches!(self, Technique::Pattern(_) | Technique::Reduction(_))
    }

    pub fn grade(self) -> Option<Grade> {
        match self {
            Technique::Single | Technique::Satisfied => Some(Grade::Easy),
            Technique::Pattern(p) => Some(p.grade()),
            Technique::Reduction(_) | Technique::Chain => Some(Grade::Hard),
            Technique::Suppose | Technique::Count | Technique::Combinations => Some(Grade::Evil),
            Technique::WrongFlag | Technique::Nothing | Technique::Guess(_) => None,
        }
    }
}

/// What the hint says about its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Safe,
    Mine,
    /// Flagged, but proven safe.
    WrongFlag,
    /// Not proven: the cell to guess.
    Guess,
    /// Not proven: a cell not to guess.
    Avoid,
}

/// The move Apply plays for a verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Open,
    Flag,
    Unflag,
}

impl Verdict {
    /// What Apply plays; nothing for a cell to avoid.
    pub fn action(self) -> Option<Action> {
        match self {
            Verdict::Safe | Verdict::Guess => Some(Action::Open),
            Verdict::Mine => Some(Action::Flag),
            Verdict::WrongFlag => Some(Action::Unflag),
            Verdict::Avoid => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub cell: usize,
    pub verdict: Verdict,
}

/// How many mines a number puts in the cells it shares with another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    AtLeast(usize),
    AtMost(usize),
    Exactly(usize),
}

/// One line of a reason. `at` is always an open number; `n` is the digit it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// The number has exactly as many hidden cells as it needs: they are all mines.
    Full { at: usize, n: u8, cells: Vec<usize> },
    /// The number touches `known` mines proven above, so it acts like `n - known`.
    Reduced { at: usize, n: u8, known: usize },
    /// The number already has all its mines (proven above): its other cells are safe.
    Satisfied { at: usize, n: u8, cells: Vec<usize> },
    /// The number, still needing `need` with `own` hidden cells it doesn't share with
    /// `other`, puts `bound` mines in the shared cells.
    Shared { at: usize, n: u8, need: usize, own: usize, other: usize, other_n: u8, bound: Bound },
    /// The other number, still needing `need`, with the shared cells holding `held` (at least
    /// that, for safe; at most, for mines): its own `cells` are all safe or all mines.
    Rest { at: usize, n: u8, need: usize, held: usize, cells: Vec<usize>, mine: bool },
    /// Were the flagged cell a mine, these numbers could not all be right.
    Contradiction { cell: usize, numbers: Vec<usize> },
    /// The cell just proven safe carries a flag.
    FlagWrong { cell: usize },
    /// With what the lines above settle, the number still needs `need` mines among its
    /// `cells` left: all mines (`need` is their count), or all safe (`need` is 0).
    Now { at: usize, n: u8, need: usize, cells: Vec<usize>, mine: bool },
    /// The shared cells must hold all `need` of the number's mines left (`most`: as many as
    /// they can), or no more than it needs (as few as they must): each group holds exactly
    /// its bound.
    Split { at: usize, n: u8, need: usize, most: bool },
    /// Suppose the cell were a mine (or safe)...
    Suppose { cell: usize, mine: bool },
    /// ...then this number would have too many mines (`over`), or too few.
    Broken { at: usize, n: u8, over: bool },
    /// ...then these numbers (and, with `count`, the mines left) could not all be right.
    NoRoom { count: bool },
    /// So the cell is a mine (or safe).
    So { mine: bool },
    /// Only `left` mines are left to find, and the `numbers` need them all: every other
    /// hidden cell is safe.
    CountAll { left: usize, numbers: usize },
    /// Only `left` are left to find, and the `numbers` hold at most `held`: every other
    /// hidden cell is a mine.
    CountRest { left: usize, held: usize, numbers: usize },
    /// ...then these numbers would need at least `need`, more than the `left` to find.
    CountOver { need: usize, left: usize },
    /// ...then even with every other hidden cell a mine there would be at most `most`, fewer
    /// than the `left` to find.
    CountUnder { most: usize, left: usize },
    /// These two cells hold one mine, and no number, open or yet to open, tells them apart.
    FiftyFifty { cells: [usize; 2] },
    /// The 50/50 is one of the `count` a Max Guess field announced.
    Announced { count: usize },
    /// Were the cell a mine, it and `partner` would be a true 50/50.
    Pseudo { cell: usize, partner: usize },
    /// If safe, the cell can only show `shows`.
    Dead { cell: usize, shows: u8 },
    /// The numbers (one, showing `n`, or several) have `ways` arrangements, all with the same
    /// count of mines; the cell is a mine in `mine_in` of them.
    Ways { n: Option<u8>, ways: usize, mine_in: usize },
    /// The numbers need at least `least` mines; `away` hidden cells touch no number.
    Fewer { least: usize, away: usize },
    /// The cell is safe in every arrangement with only `least` mines.
    FewerSafe { least: usize },
    /// No number touches the cell, which has `neighbours`; `hidden` cells hold `mines`.
    Opening { neighbours: usize, hidden: usize, mines: usize },
    /// XP moves a first-click mine to the top left.
    TopLeft,
    /// ...so not `cell`, the top left, which would have been as good but for that.
    NotTopLeft { cell: usize },
}

/// A hint: what to look for, where, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub technique: Technique,
    /// The open numbers the reason reads, in its order.
    pub numbers: Vec<usize>,
    pub target: Option<Target>,
    pub steps: Vec<Step>,
}

impl Hint {
    pub fn grade(&self) -> Option<Grade> {
        self.technique.grade()
    }

    /// How many presses it takes to see all of it: nudge, numbers, cell, reason.
    pub fn stages(&self) -> u8 {
        if self.target.is_some() { 4 } else { 1 }
    }

    /// The other cell a guess talks about: the 50/50's other half, the pseudo-50/50's
    /// partner, or the top left an opening passes over.
    pub fn partner(&self) -> Option<usize> {
        let cell = self.target?.cell;
        self.steps.iter().find_map(|s| match *s {
            Step::FiftyFifty { cells } => cells.into_iter().find(|&c| c != cell),
            Step::Pseudo { partner, .. } => Some(partner),
            Step::NotTopLeft { cell } => Some(cell),
            _ => None,
        })
    }

    fn nothing() -> Hint {
        Hint { technique: Technique::Nothing, numbers: Vec::new(), target: None, steps: Vec::new() }
    }
}

/// The hint for `game` as the player sees it, the last move at `last`.
pub fn of(game: &Game, last: Option<usize>) -> Hint {
    let flags: Vec<bool> = game.cells.iter().map(|c| c.cover == Cover::Flag).collect();
    find(&View::of(game), &flags, last)
}

/// What a hint knows of the field beyond what the player sees.
#[derive(Clone, Copy, Debug, Default)]
pub struct Field<'a> {
    /// Classic's first click: a mine under it moved to the top left.
    pub classic_first: Option<usize>,
    /// Max Guess: the true 50/50s the field was built with.
    pub announced: &'a [(usize, usize)],
}

/// The hint for `game` as the player sees it, the last move at `last`, with the guess tier
/// when no cell can be proven.
pub fn advise_game(game: &Game, last: Option<usize>, field: &Field) -> Hint {
    let flags: Vec<bool> = game.cells.iter().map(|c| c.cover == Cover::Flag).collect();
    advise(&View::of(game), &flags, last, field)
}

/// [`find`], or when it finds nothing and no cell can be proven, how to choose a guess.
pub fn advise(v: &View, flags: &[bool], last: Option<usize>, field: &Field) -> Hint {
    let h = find(v, flags, last);
    if h.technique != Technique::Nothing {
        return h;
    }
    guess::choose(v, flags, last, field).unwrap_or(h)
}

/// The hint for the open numbers in `v`, with the player's flags at `flags` and their last
/// move at `last`.
///
/// A provably wrong flag comes first (#10: "the first hint says so before anything else").
/// Then one sort key:
/// - the fewest numbers, and at the same count a named pattern before raw logic. #16 has a
///   named pattern beat raw logic "on the same numbers", then the fewest numbers; #10's
///   "prefer a named pattern, then the fewest numbers" reads the same way here, as a pattern
///   on more numbers is no cheaper to see than logic on fewer.
/// - a safe cell before a mine: #10, "at equal cost, a safe cell is preferred to a mine".
/// - the nearest the last move (#10, #16).
pub fn find(v: &View, flags: &[bool], last: Option<usize>) -> Hint {
    let cs = constraints(v, &vec![Fact::Unknown; v.len()]);
    let mut found = Vec::new();
    singles(v, &cs, &mut found);
    satisfied(v, &cs, &mut found);
    pairs(v, &cs, &mut found);
    reductions(v, &cs, &mut found);
    let (wrong, right) = named(v, flags, &found);

    // Whatever else logic proves from as few numbers, explained only if chosen.
    let near = |h: &Hint| distance(v, h, last);
    let fewest = wrong.iter().chain(&right).map(|h| h.numbers.len()).min();
    let nameable = |cell: usize, mine: bool| v.open[cell].is_none() && !(flags[cell] && mine);
    let mut more = logic::fewest(&cs, fewest.unwrap_or(logic::EXHAUSTIVE).min(logic::EXHAUSTIVE), &nameable);
    if more.is_empty() && fewest.is_none() {
        let near_cell = |cell| near(&Hint { target: Some(Target { cell, verdict: Verdict::Safe }), ..Hint::nothing() });
        more = logic::deep(v, &cs, flags, &nameable, &near_cell);
    }
    more.retain(|s| !wrong.iter().chain(&right).any(|h| h.target.is_some_and(|t| t.cell == s.cell) && h.numbers.len() <= s.cons.len()));
    let mut wrong: Vec<(Hint, Option<usize>)> = wrong.into_iter().map(|h| (h, None)).collect();
    let mut right: Vec<(Hint, Option<usize>)> = right.into_iter().map(|h| (h, None)).collect();
    for (k, s) in more.iter().enumerate() {
        let verdict = match (flags[s.cell], s.mine) {
            (true, false) => Verdict::WrongFlag,
            (_, true) => Verdict::Mine,
            (false, false) => Verdict::Safe,
        };
        let technique = patterns::many(v, &s.cons).map_or(Technique::Chain, Technique::Pattern);
        let shell = Hint { technique, numbers: s.numbers(), target: Some(Target { cell: s.cell, verdict }), steps: Vec::new() };
        if verdict == Verdict::WrongFlag { &mut wrong } else { &mut right }.push((shell, Some(k)));
    }
    if wrong.is_empty() {
        wrong = contradictions(v, flags).into_iter().map(|h| (h, None)).collect();
    }
    let chosen = wrong.into_iter().min_by_key(|(h, _)| (h.numbers.len(), near(h), h.target.map(|t| t.cell), h.numbers.clone())).or_else(|| {
        right.into_iter().min_by_key(|(h, _)| {
            let mine = h.target.is_some_and(|t| t.verdict == Verdict::Mine);
            (h.numbers.len(), !h.technique.named(), mine, near(h), h.target.map(|t| t.cell), h.numbers.clone())
        })
    });
    match chosen {
        None => Hint::nothing(),
        Some((h, None)) => h,
        Some((h, Some(k))) => {
            let (technique, numbers, mut steps) = match h.technique {
                Technique::Pattern(p) => patterns::hub(v, p, &more[k].cons, more[k].cell, more[k].mine).map(|(numbers, steps)| (h.technique, numbers, steps)),
                _ => None,
            }
            .unwrap_or_else(|| logic::explain(v, &more[k]));
            let target = h.target.expect("a cell");
            if target.verdict == Verdict::WrongFlag {
                steps.push(Step::FlagWrong { cell: target.cell });
                return Hint { technique: Technique::WrongFlag, numbers, target: Some(target), steps };
            }
            let technique = match (h.technique, technique) {
                (named @ Technique::Pattern(_), _) => named,
                (_, Technique::Chain | Technique::Suppose) if numbers.len() >= DEPENDENCY_CHAIN => Technique::Pattern(Pattern::DependencyChain),
                (_, t) => t,
            };
            Hint { technique, numbers, target: Some(target), steps }
        }
    }
}

/// A deduction reading this many numbers or more, one or two at a time or by supposing, is a
/// dependency chain.
const DEPENDENCY_CHAIN: usize = 5;

/// The hints the deductions `found` give, as (wrong flags, the rest). A flag on a proven
/// mine needs no hint.
fn named(v: &View, flags: &[bool], found: &[Found]) -> (Vec<Hint>, Vec<Hint>) {
    let mut wrong = Vec::new();
    let mut right = Vec::new();
    for d in found {
        for &cell in &d.cells {
            if v.open[cell].is_some() {
                continue;
            }
            match (flags[cell], d.mine) {
                // Already flagged, and rightly: nothing to say.
                (true, true) => {}
                (true, false) => {
                    let mut steps = d.steps.clone();
                    steps.push(Step::FlagWrong { cell });
                    wrong.push(Hint { technique: Technique::WrongFlag, numbers: d.numbers.clone(), target: Some(Target { cell, verdict: Verdict::WrongFlag }), steps });
                }
                (false, mine) => right.push(Hint {
                    technique: d.technique,
                    numbers: d.numbers.clone(),
                    target: Some(Target { cell, verdict: if mine { Verdict::Mine } else { Verdict::Safe } }),
                    steps: d.steps.clone(),
                }),
            }
        }
    }
    (wrong, right)
}

/// How far the hint's cells and numbers are from the last move (squared, in cells).
fn distance(v: &View, h: &Hint, last: Option<usize>) -> usize {
    let Some(l) = last else { return 0 };
    let (lx, ly) = ((l % v.w) as isize, (l / v.w) as isize);
    h.target.map(|t| t.cell).into_iter().chain(h.numbers.iter().copied()).map(|i| ((i % v.w) as isize - lx).pow(2) as usize + ((i / v.w) as isize - ly).pow(2) as usize).min().unwrap_or(0)
}

/// A deduction: cells it settles, all safe or all mines, and why.
struct Found {
    technique: Technique,
    numbers: Vec<usize>,
    cells: Vec<usize>,
    mine: bool,
    steps: Vec<Step>,
}

fn digit(v: &View, at: usize) -> u8 {
    v.open[at].unwrap_or(0)
}

fn full(c: &Constraint) -> bool {
    c.need > 0 && c.need == c.cells.len()
}

fn singles(v: &View, cs: &[Constraint], out: &mut Vec<Found>) {
    for c in cs.iter().filter(|c| full(c)) {
        out.push(Found {
            technique: Technique::Single,
            numbers: vec![c.at],
            cells: c.cells.clone(),
            mine: true,
            steps: vec![Step::Full { at: c.at, n: digit(v, c.at), cells: c.cells.clone() }],
        });
    }
}

/// The numbers needing every hidden cell (other than `not`) that touch `cells`.
fn provers<'a>(cs: &'a [Constraint], cells: &[usize], not: &[usize]) -> Vec<&'a Constraint> {
    cs.iter().filter(|p| full(p) && !not.contains(&p.at) && p.cells.iter().any(|c| cells.contains(c))).collect()
}

/// The mines `ps` prove.
fn mines_of(ps: &[&Constraint]) -> Vec<usize> {
    let mut k: Vec<usize> = ps.iter().flat_map(|p| p.cells.iter().copied()).collect();
    k.sort_unstable();
    k.dedup();
    k
}

/// The smallest non-empty subset of `items` (by position) for which `ok` holds, smallest
/// first; past 10 items only the whole set is tried.
fn smallest<T: Copy>(items: &[T], ok: impl Fn(&[T]) -> bool) -> Option<Vec<T>> {
    let n = items.len();
    if n > 10 {
        return ok(items).then(|| items.to_vec());
    }
    let mut masks: Vec<u32> = (1..1u32 << n).collect();
    masks.sort_by_key(|m| (m.count_ones(), *m));
    masks.into_iter().map(|m| (0..n).filter(|&i| m >> i & 1 == 1).map(|i| items[i]).collect::<Vec<T>>()).find(|s| ok(s))
}

fn prover_steps(v: &View, ps: &[&Constraint]) -> Vec<Step> {
    ps.iter().map(|p| Step::Full { at: p.at, n: digit(v, p.at), cells: p.cells.clone() }).collect()
}

/// A number whose mines single numbers prove: the rest of its cells are safe.
fn satisfied(v: &View, cs: &[Constraint], out: &mut Vec<Found>) {
    for b in cs.iter().filter(|c| c.need > 0 && !full(c)) {
        let ps = provers(cs, &b.cells, &[b.at]);
        let known = |ps: &[&Constraint]| mines_of(ps).iter().filter(|c| b.cells.contains(c)).count();
        let Some(ps) = smallest(&ps, |s| known(s) == b.need) else { continue };
        let k = mines_of(&ps);
        let cells: Vec<usize> = b.cells.iter().copied().filter(|c| !k.contains(c)).collect();
        let mut steps = prover_steps(v, &ps);
        steps.push(Step::Satisfied { at: b.at, n: digit(v, b.at), cells: cells.clone() });
        let numbers = ps.iter().map(|p| p.at).chain([b.at]).collect();
        out.push(Found { technique: Technique::Satisfied, numbers, cells, mine: false, steps });
    }
}

/// Every pair of numbers sharing hidden cells, neither of which settles its cells alone.
fn overlapping(cs: &[Constraint]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for (x, a) in cs.iter().enumerate() {
        for (y, b) in cs.iter().enumerate().skip(x + 1) {
            if !full(a) && !full(b) && a.cells.iter().any(|c| b.cells.binary_search(c).is_ok()) {
                v.push((x, y));
            }
        }
    }
    v
}

/// What `a` tells about the cells only `b` touches: the pattern, those cells, whether they
/// are mines, and the two steps of the reason.
fn pair_one(v: &View, a: &Constraint, b: &Constraint) -> Option<(Pattern, Vec<usize>, bool, [Step; 2])> {
    let shared = a.cells.iter().filter(|c| b.cells.binary_search(c).is_ok()).count();
    let own = a.cells.len() - shared;
    let only_b: Vec<usize> = b.cells.iter().copied().filter(|c| a.cells.binary_search(c).is_err()).collect();
    if shared == 0 || only_b.is_empty() || a.need == 0 || b.need == 0 {
        return None;
    }
    let (an, bn) = (digit(v, a.at), digit(v, b.at));
    let shared_step = |bound| Step::Shared { at: a.at, n: an, need: a.need, own, other: b.at, other_n: bn, bound };
    let rest = |held, mine| Step::Rest { at: b.at, n: bn, need: b.need, held, cells: only_b.clone(), mine };
    // At least this many of a's mines must go in the shared cells.
    let lo = a.need.saturating_sub(own);
    if lo > 0 && lo == b.need {
        let (pattern, bound) = if own == 0 { (Pattern::OneOne, Bound::Exactly(a.need)) } else { (Pattern::OneTwo, Bound::AtLeast(lo)) };
        return Some((pattern, only_b.clone(), false, [shared_step(bound), rest(lo, false)]));
    }
    // At most this many fit there, so b's own cells hold the rest.
    let hi = a.need.min(shared);
    if b.need > hi && b.need - hi == only_b.len() {
        return Some((Pattern::OneTwo, only_b.clone(), true, [shared_step(Bound::AtMost(hi)), rest(hi, true)]));
    }
    None
}

fn pairs(v: &View, cs: &[Constraint], out: &mut Vec<Found>) {
    let proven = mines_of(&cs.iter().filter(|c| full(c)).collect::<Vec<_>>());
    for (x, y) in overlapping(cs) {
        for (a, b) in [(&cs[x], &cs[y]), (&cs[y], &cs[x])] {
            if let Some((p, cells, mine, steps)) = pair_one(v, a, b) {
                // Numbers touching proven mines are named as the reduction a player sees once
                // those are flagged, though they need no subtracting here.
                let both: Vec<usize> = a.cells.iter().chain(&b.cells).copied().collect();
                let known: Vec<usize> = proven.iter().copied().filter(|m| both.contains(m)).collect();
                let technique = match patterns::reduced(v, cs, &known, a, b) {
                    Some(r) => Technique::Reduction(r),
                    None => match patterns::pair(v, cs, &proven, a, b, p == Pattern::OneOne, mine) {
                        // A row the Chinese name for the proven mine beside it.
                        r @ Pattern::NextToMineTwoTwoOne => Technique::Reduction(r),
                        r => Technique::Pattern(r),
                    },
                };
                out.push(Found { technique, numbers: vec![a.at, b.at], cells, mine, steps: steps.into() });
            }
        }
    }
}

/// `c` with the mines `k` taken out.
fn reduce(c: &Constraint, k: &[usize]) -> (Constraint, usize) {
    let known = c.cells.iter().filter(|x| k.contains(x)).count();
    let cells = c.cells.iter().copied().filter(|x| !k.contains(x)).collect();
    (Constraint { at: c.at, cells, need: c.need.saturating_sub(known) }, known)
}

/// Mines proven by single numbers, subtracted from two numbers that then form a pattern
/// they don't form as shown.
fn reductions(v: &View, cs: &[Constraint], out: &mut Vec<Found>) {
    for (x, y) in overlapping(cs) {
        for (a, b) in [(&cs[x], &cs[y]), (&cs[y], &cs[x])] {
            let both: Vec<usize> = a.cells.iter().chain(&b.cells).copied().collect();
            let ps = provers(cs, &both, &[a.at, b.at]);
            if ps.is_empty() {
                continue;
            }
            let raw = pair_one(v, a, b).map(|r| (r.1, r.2));
            let with = |s: &[&Constraint]| {
                let k = mines_of(s);
                let ((ra, ka), (rb, kb)) = (reduce(a, &k), reduce(b, &k));
                pair_one(v, &ra, &rb).map(|r| (r, ka, kb))
            };
            let Some(((_, cells, mine, _), _, _)) = with(&ps) else { continue };
            for cell in cells {
                if raw.as_ref().is_some_and(|(c, m)| *m == mine && c.contains(&cell)) {
                    continue;
                }
                let Some(sub) = smallest(&ps, |s| with(s).is_some_and(|((_, c, m, _), _, _)| m == mine && c.contains(&cell))) else { continue };
                let ((pattern, cells, mine, two), ka, kb) = with(&sub).expect("found above");
                // Named for every proven mine the two touch, as a player who flagged them sees it.
                let k: Vec<usize> = mines_of(&ps).into_iter().filter(|m| both.contains(m)).collect();
                let pattern = patterns::reduced(v, cs, &k, a, b).unwrap_or(pattern);
                let mut steps = prover_steps(v, &sub);
                for (c, k) in [(a, ka), (b, kb)] {
                    if k > 0 {
                        steps.push(Step::Reduced { at: c.at, n: digit(v, c.at), known: k });
                    }
                }
                steps.extend(two);
                let numbers = sub.iter().map(|p| p.at).chain([a.at, b.at]).collect();
                out.push(Found { technique: Technique::Reduction(pattern), numbers, cells, mine, steps });
            }
        }
    }
}

/// Flags the numbers around them prove wrong, when no one- or two-number step shows it: with
/// a mine under the flag, the numbers touching it and their neighbours could not all be
/// satisfied. Only those numbers are read, never the count of mines left.
fn contradictions(v: &View, flags: &[bool]) -> Vec<Hint> {
    let cs = constraints(v, &vec![Fact::Unknown; v.len()]);
    (0..v.len())
        .filter(|&i| flags[i] && v.open[i].is_none())
        .filter_map(|cell| {
            let numbers = impossible(&cs, cell)?;
            Some(Hint { technique: Technique::WrongFlag, numbers: numbers.clone(), target: Some(Target { cell, verdict: Verdict::WrongFlag }), steps: vec![Step::Contradiction { cell, numbers }] })
        })
        .collect()
}

/// Cells a contradiction search may try, so a hint stays quick.
const LOCAL_CELLS: usize = 24;

/// The numbers that rule out a mine at `cell`: those touching it, and those sharing a hidden
/// cell with them. None if a mine there fits them.
fn impossible(cs: &[Constraint], cell: usize) -> Option<Vec<usize>> {
    let near: Vec<&Constraint> = cs.iter().filter(|c| c.cells.contains(&cell)).collect();
    let mut group: Vec<&Constraint> = cs.iter().filter(|c| near.iter().any(|n| n.cells.iter().any(|x| c.cells.contains(x)))).collect();
    group.sort_by_key(|c| c.at);
    let mut cells: Vec<usize> = group.iter().flat_map(|c| c.cells.iter().copied()).filter(|&x| x != cell).collect();
    cells.sort_unstable();
    cells.dedup();
    if near.is_empty() || cells.len() > LOCAL_CELLS {
        return None;
    }
    // Mines still needed and cells still open to each number, with the flag's cell a mine.
    let mut need: Vec<isize> = group.iter().map(|c| c.need as isize - 1).collect();
    for (k, c) in group.iter().enumerate() {
        if !c.cells.contains(&cell) {
            need[k] += 1;
        }
    }
    let mut room: Vec<usize> = group.iter().map(|c| c.cells.iter().filter(|&&x| x != cell).count()).collect();
    let touches: Vec<Vec<usize>> = cells.iter().map(|x| (0..group.len()).filter(|&k| group[k].cells.contains(x)).collect()).collect();
    fn fits(i: usize, touches: &[Vec<usize>], need: &mut [isize], room: &mut [usize]) -> bool {
        if need.iter().zip(room.iter()).any(|(&n, &r)| n < 0 || n as usize > r) {
            return false;
        }
        if i == touches.len() {
            return true;
        }
        for mine in [false, true] {
            for &k in &touches[i] {
                room[k] -= 1;
                need[k] -= mine as isize;
            }
            let ok = fits(i + 1, touches, need, room);
            for &k in &touches[i] {
                room[k] += 1;
                need[k] += mine as isize;
            }
            if ok {
                return true;
            }
        }
        false
    }
    (!fits(0, &touches, &mut need, &mut room)).then(|| group.iter().map(|c| c.at).collect())
}

#[cfg(test)]
mod tests;
