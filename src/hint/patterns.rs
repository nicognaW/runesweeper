//! The named pattern catalogue a hint recognises: minesweeper.online's 28 patterns and the
//! Chinese community's 定式, each with its grade (minesweeper.online's No Guess level).
//!
//! A pattern is a name for a deduction the hint has already found, given by the shape it
//! takes on the field: which numbers, in what line, with their hidden cells on which side.
//! Recognising one never changes what is proven, only what the hint calls it. Wording is in
//! [`super::words`]; the zh-CN names are listed in `src/zh_cn_terms.md`.
//!
//! The layouts of the holes, the triangles and the 1-3-1 corner and T-pattern follow
//! minesweeper.online's pattern diagrams; the 2-2-2 corner, 1>2<1 and the dependency chain
//! follow its descriptions; the Chinese row patterns follow 张砷镓's 扫雷定式及变化
//! (saolei.wang bbs/362).

use super::{Bound, Grade, Step, digit, pair_one, reduce};
use crate::solver::{Constraint, View};

/// A named arrangement of numbers with a known deduction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    /// One number's hidden cells all touch the other, which needs no more: its one other
    /// cell is safe (1-1, 11定式).
    OneOne,
    /// A 1-1 read from a number at the edge of the field (边部11定式).
    EdgeOneOne,
    /// A 1-1 whose larger number has several other cells, all safe (1-1+).
    OneOnePlus,
    /// A number's cells all touch a larger one, whose one other cell is a mine (1-2, 12定式).
    OneTwo,
    /// A 1-2 whose larger number has several other cells, all mines (1-2+).
    OneTwoPlus,
    /// Two numbers sharing some cells: the smaller allows at most so many there, so the
    /// larger's other cell is a mine, or the larger needs at least so many there, so the
    /// smaller's other cells are safe (1-2C, the classic 1-2).
    OneTwoC,
    /// A 1-2C whose larger number's several other cells are all mines (1-2C+).
    OneTwoCPlus,
    /// 1-2-1 in a row: a 1-2 from each end (121定式).
    OneTwoOne,
    /// 1-2-2-1 in a row: a 1-2 from each end (1221定式).
    OneTwoTwoOne,
    /// 2-1-2 in a row: a 2-1 from each end (212定式).
    TwoOneTwo,
    /// 2-1-1 in a row ending at the edge of the field (边部211定式).
    EdgeTwoOneOne,
    /// Two 2s, each beside a proven mine: less them, a 1-1 (夹雷22定式).
    TwoTwoBetweenMines,
    /// 2-2-1 with a proven mine beside the end 2: less it, a 1-2-1 (靠雷221定式).
    NextToMineTwoTwoOne,
    /// 3-1 with a proven mine beside the 3: less it, a 2-1 (靠雷31定式).
    NextToMineThreeOne,
    /// 4-1 with two proven mines beside the 4: less them, a 2-1 (靠雷41定式).
    NextToMineFourOne,
    /// A 1-1 across a hole: the smaller number's two cells are beside the larger (H1, 挖坑).
    Hole1,
    /// A 1-1 across a deeper hole: the two cells are beside the smaller number (H2).
    Hole2,
    /// A 1-1 across a gap: the two numbers face each other over hidden cells (H3).
    Hole3,
    /// A 1-1 where the open cells come to a point: three shared cells (T1, 三角定式).
    Triangle1,
    /// A 1 and a 2 at a point: the 1 allows one mine in the three shared cells (T2).
    Triangle2,
    /// T2 with a 2 and a 3 (T3).
    Triangle3,
    /// A 2 and a 4 at a point: the 4's two other cells are mines (T4).
    Triangle4,
    /// T2 with two shared cells, either side of the larger number (T5).
    Triangle5,
    /// Two 1s beside a 3 at a corner: at most one mine in each pair, so the 3's corner cell
    /// is a mine (1-3-1 corner, 角131定式).
    Corner131,
    /// Two 2s beside a 2 at a corner: at least one mine in each pair, so the corner cell is
    /// safe (2-2-2 corner, 角222定式).
    Corner222,
    /// A 2 whose mines must split, one into each of two 1s not in line with it (1>2<1,
    /// 隔行的121定式).
    Staggered121,
    /// Two 1s, a 2 and a 3: what the others allow in the 3's cells puts its last mine in its
    /// corner cell (T-pattern).
    TPattern,
    /// A long chain of numbers, each settling cells for the next (dependency chain, 连续减法).
    DependencyChain,
}

impl Pattern {
    pub const ALL: [Pattern; 28] = [
        Pattern::OneOne,
        Pattern::EdgeOneOne,
        Pattern::OneOnePlus,
        Pattern::OneTwo,
        Pattern::OneTwoPlus,
        Pattern::OneTwoC,
        Pattern::OneTwoCPlus,
        Pattern::OneTwoOne,
        Pattern::OneTwoTwoOne,
        Pattern::TwoOneTwo,
        Pattern::EdgeTwoOneOne,
        Pattern::TwoTwoBetweenMines,
        Pattern::NextToMineTwoTwoOne,
        Pattern::NextToMineThreeOne,
        Pattern::NextToMineFourOne,
        Pattern::Hole1,
        Pattern::Hole2,
        Pattern::Hole3,
        Pattern::Triangle1,
        Pattern::Triangle2,
        Pattern::Triangle3,
        Pattern::Triangle4,
        Pattern::Triangle5,
        Pattern::Corner131,
        Pattern::Corner222,
        Pattern::Staggered121,
        Pattern::TPattern,
        Pattern::DependencyChain,
    ];

    /// The No Guess level at which minesweeper.online needs it; the Chinese patterns take
    /// the level of the English pattern they extend.
    pub fn grade(self) -> Grade {
        use Pattern::*;
        match self {
            OneOne | EdgeOneOne | OneOnePlus | OneTwo | OneTwoPlus | OneTwoC | OneTwoCPlus | OneTwoOne | OneTwoTwoOne | TwoOneTwo | EdgeTwoOneOne => Grade::Medium,
            TwoTwoBetweenMines | NextToMineTwoTwoOne | NextToMineThreeOne | NextToMineFourOne | Hole1 | Hole2 | Hole3 => Grade::Hard,
            Triangle1 | Triangle2 | Triangle3 | Triangle4 | Triangle5 | Corner131 | Corner222 | Staggered121 | TPattern | DependencyChain => Grade::Evil,
        }
    }

    /// The pattern a reduction leads to, as its name: a 1-1, a 1-2 or a 1-2-1, unless the
    /// Chinese name the shape before reduction.
    fn reduced(self) -> Pattern {
        use Pattern::*;
        match self {
            OneOne | EdgeOneOne | OneOnePlus | Hole1 | Hole2 | Hole3 | Triangle1 => OneOne,
            OneTwoOne => OneTwoOne,
            TwoTwoBetweenMines | NextToMineTwoTwoOne | NextToMineThreeOne | NextToMineFourOne => self,
            _ => OneTwo,
        }
    }
}

type Xy = (isize, isize);

fn xy(v: &View, i: usize) -> Xy {
    ((i % v.w) as isize, (i / v.w) as isize)
}

fn cell(v: &View, (x, y): Xy) -> Option<usize> {
    (x >= 0 && y >= 0 && (x as usize) < v.w && (y as usize) < v.h).then(|| y as usize * v.w + x as usize)
}

fn plus((x, y): Xy, (dx, dy): Xy, k: isize) -> Xy {
    (x + k * dx, y + k * dy)
}

/// The unit step from `a` to `b` when they share a row or column, and how many steps apart.
fn line(v: &View, a: usize, b: usize) -> Option<(Xy, isize)> {
    let ((ax, ay), (bx, by)) = (xy(v, a), xy(v, b));
    match (bx - ax, by - ay) {
        (0, 0) => None,
        (0, dy) => Some(((0, dy.signum()), dy.abs())),
        (dx, 0) => Some(((dx.signum(), 0), dx.abs())),
        _ => None,
    }
}

/// Where `c` lies from `a`, against the line through `a` along `d`: how far to one side, and
/// how far along.
fn offset(v: &View, a: usize, d: Xy, c: usize) -> (isize, isize) {
    let ((ax, ay), (cx, cy)) = (xy(v, a), xy(v, c));
    let (rx, ry) = (cx - ax, cy - ay);
    (ry * d.0 - rx * d.1, rx * d.0 + ry * d.1)
}

/// The side of the line all `cells` lie on, one row off it, if they do.
fn one_side(v: &View, a: usize, d: Xy, cells: &[usize]) -> Option<isize> {
    let s = offset(v, a, d, *cells.first()?).0;
    (s.abs() == 1 && cells.iter().all(|&c| offset(v, a, d, c).0 == s)).then_some(s)
}

/// Whether `cells` lie on both sides of the line.
fn straddles(v: &View, a: usize, d: Xy, cells: &[usize]) -> bool {
    let sides: Vec<isize> = cells.iter().map(|&c| offset(v, a, d, c).0).collect();
    sides.iter().any(|&s| s > 0) && sides.iter().any(|&s| s < 0)
}

fn minus(a: &[usize], b: &[usize]) -> Vec<usize> {
    a.iter().copied().filter(|c| !b.contains(c)).collect()
}

/// The open number at `at` as the hint reads it, with the mines `known` taken out.
fn number(cs: &[Constraint], known: &[usize], at: usize) -> Option<Constraint> {
    cs.iter().find(|c| c.at == at).map(|c| reduce(c, known).0)
}

/// A row pattern the two neighbouring numbers `a` and `b` sit in: the numbers in line with
/// them whose cells are all on the same side, read as 1-2-2-1, 1-2-1, 2-1-2, or 2-1-1 ending
/// at the edge. Returns it with the numbers in it.
fn row(v: &View, cs: &[Constraint], known: &[usize], a: &Constraint, b: &Constraint) -> Option<(Pattern, Vec<usize>)> {
    let (d, 1) = line(v, a.at, b.at)? else { return None };
    let side = one_side(v, a.at, d, &[a.cells.as_slice(), &b.cells].concat())?;
    let along = |from: usize, dir: Xy| {
        let mut out = Vec::new();
        let mut p = xy(v, from);
        loop {
            p = plus(p, dir, 1);
            let Some(c) = cell(v, p).and_then(|i| number(cs, known, i)) else { break };
            if c.need == 0 || one_side(v, a.at, d, &c.cells) != Some(side) {
                break;
            }
            out.push((c.at, c.need));
        }
        out
    };
    let mut run: Vec<(usize, usize)> = along(a.at, (-d.0, -d.1));
    run.reverse();
    let ia = run.len();
    run.push((a.at, a.need));
    run.push((b.at, b.need));
    run.extend(along(b.at, d));
    let needs: Vec<usize> = run.iter().map(|r| r.1).collect();
    let edge_before = cell(v, plus(xy(v, run[0].0), d, -1)).is_none();
    let edge_after = cell(v, plus(xy(v, run[run.len() - 1].0), d, 1)).is_none();
    let window = |pat: &[usize]| (ia.saturating_sub(pat.len() - 2)..=ia).find(|&s| s + pat.len() <= run.len() && needs[s..s + pat.len()] == *pat);
    let numbers = |s: usize, n: usize| run[s..s + n].iter().map(|r| r.0).collect::<Vec<usize>>();
    for (pat, p) in [(&[1, 2, 2, 1][..], Pattern::OneTwoTwoOne), (&[1, 2, 1], Pattern::OneTwoOne), (&[2, 1, 2], Pattern::TwoOneTwo)] {
        if let Some(s) = window(pat) {
            return Some((p, numbers(s, pat.len())));
        }
    }
    // 2-1-1 with the pair as the 2 and the 1 beside it, the far 1 at the edge.
    if needs[ia..].starts_with(&[2, 1, 1]) && ia + 3 == run.len() && edge_after {
        return Some((Pattern::EdgeTwoOneOne, numbers(ia, 3)));
    }
    if needs[..ia + 2].ends_with(&[1, 1, 2]) && ia == 1 && edge_before {
        return Some((Pattern::EdgeTwoOneOne, numbers(0, 3)));
    }
    None
}

/// The pattern two numbers form, as the hint reads them (with the mines `known` taken out):
/// `a` puts a bound on the cells it shares with `b`, which settles `b`'s other cells, `mine`
/// or safe. `subset` says `a`'s cells all touch `b` and the two need the same (a 1-1);
/// otherwise their difference decides (a 1-2).
pub(super) fn pair(v: &View, cs: &[Constraint], known: &[usize], a: &Constraint, b: &Constraint, subset: bool, mine: bool) -> Pattern {
    let only_b = minus(&b.cells, &a.cells);
    let shared: Vec<usize> = a.cells.iter().copied().filter(|c| b.cells.contains(c)).collect();
    if subset {
        return match line(v, a.at, b.at) {
            Some((d, 1)) => {
                if one_side(v, a.at, d, &[a.cells.as_slice(), &b.cells].concat()).is_some() {
                    if cell(v, plus(xy(v, a.at), d, -1)).is_none() {
                        Pattern::EdgeOneOne
                    } else if only_b.len() == 1 {
                        Pattern::OneOne
                    } else {
                        Pattern::OneOnePlus
                    }
                } else if straddles(v, a.at, d, &a.cells) {
                    let along: Vec<isize> = a.cells.iter().map(|&c| offset(v, a.at, d, c).1).collect();
                    if along.iter().all(|&s| s == 1) {
                        Pattern::Hole1
                    } else if along.iter().all(|&s| s == 0) {
                        Pattern::Hole2
                    } else {
                        Pattern::Triangle1
                    }
                } else {
                    Pattern::OneOnePlus
                }
            }
            // Two apart: across hidden cells between them, or the same hole as H2 read from
            // one row further out.
            Some((d, 2)) if cell(v, plus(xy(v, a.at), d, 1)).is_some_and(|m| a.cells.contains(&m)) => Pattern::Hole3,
            Some((d, 2)) if cell(v, plus(xy(v, a.at), d, 1)).is_some_and(|m| v.open[m].is_some()) && straddles(v, a.at, d, &a.cells) => Pattern::Hole2,
            _ => Pattern::OneOne,
        };
    }
    if let Some((p, numbers)) = row(v, cs, known, a, b) {
        // A 1-2-1 whose end 2 is a 1 only less a proven mine beside it.
        let digits: Vec<u8> = numbers.iter().map(|&at| digit(v, at)).collect();
        if p == Pattern::OneTwoOne && (digits == [2, 2, 1] || digits == [1, 2, 2]) {
            return Pattern::NextToMineTwoTwoOne;
        }
        return p;
    }
    // The number with fewer mines bounds the shared cells from above.
    let (small, big) = if mine { (a, b) } else { (b, a) };
    if small.cells.iter().all(|c| big.cells.contains(c)) {
        return if minus(&big.cells, &small.cells).len() == 1 { Pattern::OneTwo } else { Pattern::OneTwoPlus };
    }
    if let Some((d, 1)) = line(v, small.at, big.at) {
        let all = [small.cells.as_slice(), &big.cells].concat();
        if one_side(v, small.at, d, &all).is_none() && straddles(v, small.at, d, &shared) {
            let own_big = minus(&big.cells, &small.cells).len();
            return if own_big >= 2 {
                Pattern::Triangle4
            } else if shared.len() == 2 {
                Pattern::Triangle5
            } else if small.need >= 2 {
                Pattern::Triangle3
            } else {
                Pattern::Triangle2
            };
        }
    }
    // Named for the pair, whichever way it is read: the larger number's own cells are all mines.
    if minus(&big.cells, &small.cells).len() > 1 { Pattern::OneTwoCPlus } else { Pattern::OneTwoC }
}

/// The name two numbers touching proven mines (`known`) go by as a reduction: the pattern
/// they make once the mines are taken out (a 1-1, a 1-2 or a 1-2-1), or the Chinese name for
/// the digits as shown and that pattern (夹雷22, 靠雷221, 靠雷31, 靠雷41). The numbers may
/// settle their cells as shown or only once reduced. None if they touch no proven mine, or
/// form nothing once reduced.
pub(super) fn reduced(v: &View, cs: &[Constraint], known: &[usize], a: &Constraint, b: &Constraint) -> Option<Pattern> {
    let ((ra, ka), (rb, kb)) = (reduce(a, known), reduce(b, known));
    if ka + kb == 0 {
        return None;
    }
    let (kind, _, mine, _) = pair_one(v, &ra, &rb).or_else(|| pair_one(v, &rb, &ra))?;
    let p = pair(v, cs, known, &ra, &rb, kind == Pattern::OneOne, mine);
    let straight = line(v, a.at, b.at).is_some_and(|(d, k)| k == 1 && one_side(v, a.at, d, &[ra.cells.as_slice(), &rb.cells].concat()).is_some());
    if !straight {
        return Some(p.reduced());
    }
    let mut digits = [(digit(v, a.at), ka), (digit(v, b.at), kb)];
    digits.sort();
    match (p.reduced(), digits) {
        (Pattern::OneOne, [(2, 1), (2, 1)]) => return Some(Pattern::TwoTwoBetweenMines),
        (Pattern::OneTwo, [(1, 0), (3, 1)]) => return Some(Pattern::NextToMineThreeOne),
        (Pattern::OneTwo, [(1, 0), (4, 2)]) => return Some(Pattern::NextToMineFourOne),
        _ => {}
    }
    Some(p.reduced())
}

/// The pattern three or more numbers form, read from their positions and digits.
pub(super) fn many(v: &View, cons: &[Constraint]) -> Option<Pattern> {
    let mut needs: Vec<usize> = cons.iter().map(|c| c.need).collect();
    needs.sort_unstable();
    let beside = |a: &Constraint, b: &Constraint| line(v, a.at, b.at).filter(|l| l.1 == 1).map(|l| l.0);
    // A number with two others beside it, one along each side of a corner.
    let corner = |hub: usize| {
        cons.iter().filter(|h| h.need == hub).any(|h| {
            let dirs: Vec<Xy> = cons.iter().filter(|o| o.at != h.at).filter_map(|o| beside(h, o)).collect();
            dirs.len() == 2 && dirs[0].0 * dirs[1].0 + dirs[0].1 * dirs[1].1 == 0
        })
    };
    let shares = |a: &Constraint, b: &Constraint| a.cells.iter().any(|c| b.cells.contains(c));
    match needs[..] {
        [1, 1, 3] if corner(3) => Some(Pattern::Corner131),
        [2, 2, 2] if corner(2) => Some(Pattern::Corner222),
        [1, 1, 2] => {
            let two = cons.iter().find(|c| c.need == 2)?;
            let ones: Vec<&Constraint> = cons.iter().filter(|c| c.need == 1).collect();
            let staggered = ones.iter().all(|o| shares(o, two) && line(v, o.at, two.at).is_none());
            (staggered && !shares(ones[0], ones[1])).then_some(Pattern::Staggered121)
        }
        [1, 1, 2, 3] => {
            let (two, three) = (cons.iter().find(|c| c.need == 2)?, cons.iter().find(|c| c.need == 3)?);
            beside(two, three).map(|_| Pattern::TPattern)
        }
        _ => None,
    }
}

/// The reason for a cell a 1-3-1 corner, a 2-2-2 corner or a 1>2<1 settles, read the way
/// minesweeper.online teaches it: each number beside the middle one bounds what it shares
/// with it, the bounds add up to what the middle one needs, so its own cells are settled, and
/// each shared group holds exactly its bound. The numbers in reading order, and the lines;
/// None if the cell is not one this reading settles.
pub(super) fn hub(v: &View, p: Pattern, cons: &[Constraint], cell: usize, mine: bool) -> Option<(Vec<usize>, Vec<Step>)> {
    let (need, most) = match p {
        Pattern::Corner131 => (3, true),
        Pattern::Staggered121 => (2, true),
        Pattern::Corner222 => (2, false),
        _ => return None,
    };
    let shares = |a: &Constraint, b: &Constraint| a.cells.iter().any(|c| b.cells.contains(c));
    let hub = cons.iter().find(|h| h.need == need && cons.iter().filter(|o| o.at != h.at).all(|o| shares(o, h)))?;
    let spokes: Vec<&Constraint> = cons.iter().filter(|o| o.at != hub.at).collect();
    if spokes.len() != 2 || spokes[0].cells.iter().any(|c| spokes[1].cells.contains(c) && hub.cells.contains(c)) {
        return None;
    }
    let (n, at) = (digit(v, hub.at), hub.at);
    let mut steps = Vec::new();
    let mut bounds = Vec::new();
    for s in &spokes {
        let shared = s.cells.iter().filter(|c| hub.cells.contains(c)).count();
        let own = s.cells.len() - shared;
        let (k, bound) = if most { let k = s.need.min(shared); (k, Bound::AtMost(k)) } else { let k = s.need.saturating_sub(own); (k, Bound::AtLeast(k)) };
        bounds.push(k);
        steps.push(Step::Shared { at: s.at, n: digit(v, s.at), need: s.need, own, other: hub.at, other_n: n, bound });
    }
    let held: usize = bounds.iter().sum();
    let rest: Vec<usize> = hub.cells.iter().copied().filter(|c| !spokes.iter().any(|s| s.cells.contains(c))).collect();
    let fits = if most { hub.need == held + rest.len() } else { hub.need == held };
    if !fits || held == 0 {
        return None;
    }
    let numbers = spokes.iter().map(|s| s.at).chain([hub.at]).collect();
    let rest_mine = most && !rest.is_empty();
    if rest.contains(&cell) {
        return (rest_mine == mine).then(|| {
            steps.push(Step::Rest { at, n, need: hub.need, held, cells: rest, mine });
            (numbers, steps)
        });
    }
    let j = spokes.iter().position(|s| s.cells.contains(&cell) && !hub.cells.contains(&cell))?;
    let own: Vec<usize> = spokes[j].cells.iter().copied().filter(|c| !hub.cells.contains(c)).collect();
    let left = spokes[j].need - bounds[j];
    let own_mine = left > 0;
    if (left != 0 && left != own.len()) || own_mine != mine {
        return None;
    }
    if !rest.is_empty() {
        steps.push(Step::Rest { at, n, need: hub.need, held, cells: rest, mine: rest_mine });
    }
    steps.push(Step::Split { at, n, need: held, most });
    steps.push(Step::Rest { at: spokes[j].at, n: digit(v, spokes[j].at), need: spokes[j].need, held: bounds[j], cells: own, mine });
    Some((numbers, steps))
}
