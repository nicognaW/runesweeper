//! Hints on fixed positions: the cell, the numbers and the reason, in both languages.
//!
//! A position is drawn as rows: `.` an open cell (its number follows from the mines), `#` a
//! hidden safe cell, `*` a hidden mine, `F` a flagged mine, `x` a flagged safe cell.

use super::*;
use crate::strings::Lang;

mod catalogue;

struct Pos {
    v: View,
    flags: Vec<bool>,
    w: usize,
}

fn pos(rows: &[&str]) -> Pos {
    let w = rows[0].len();
    let h = rows.len();
    let at = |c: char| rows.iter().enumerate().flat_map(move |(y, r)| r.char_indices().filter(move |&(_, ch)| ch == c).map(move |(x, _)| y * w + x)).collect::<Vec<_>>();
    let mines: Vec<usize> = [at('*'), at('F')].concat();
    let mut g = Game::with_mines(w, h, &mines);
    for i in at('.') {
        g.cells[i].cover = Cover::Open;
    }
    for i in [at('F'), at('x')].concat() {
        g.cells[i].cover = Cover::Flag;
    }
    let flags = g.cells.iter().map(|c| c.cover == Cover::Flag).collect();
    Pos { v: View::of(&g), flags, w }
}

impl Pos {
    fn at(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    fn hint(&self, last: Option<(usize, usize)>) -> Hint {
        find(&self.v, &self.flags, last.map(|(x, y)| self.at(x, y)))
    }
}

fn target(h: &Hint) -> (usize, Verdict) {
    let t = h.target.expect("a cell");
    (t.cell, t.verdict)
}

fn en(h: &Hint) -> Vec<String> {
    words::reason(h, Lang::En)
}

fn zh(h: &Hint) -> Vec<String> {
    words::reason(h, Lang::ZhCn)
}

#[test]
fn a_number_with_only_as_many_hidden_cells_as_it_needs() {
    // The 1 in the corner touches one hidden cell.
    let p = pos(&["..*#", "...."]);
    let h = p.hint(Some((0, 0)));
    assert_eq!(h.technique, Technique::Single);
    assert_eq!(h.grade(), Some(Grade::Easy));
    assert_eq!(target(&h), (p.at(2, 0), Verdict::Mine));
    assert_eq!(h.numbers, vec![p.at(1, 0)]);
    assert_eq!(en(&h), ["This 1 still needs 1 mine and touches only 1 hidden cell, so this cell must be a mine."]);
    assert_eq!(zh(&h), ["看这个 1，它周围还有 1 个雷，只剩下 1 格……所以这一格必须是雷。"]);
}

#[test]
fn a_number_whose_mine_another_number_proves_is_satisfied() {
    // The mine is flagged; the hint proves it again from the numbers, never from the flag.
    let p = pos(&["..F#", "...."]);
    let h = p.hint(Some((3, 1)));
    assert_eq!(h.technique, Technique::Satisfied);
    assert_eq!(target(&h), (p.at(3, 0), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(1, 0), p.at(3, 1)]);
    assert_eq!(
        en(&h),
        ["First, this 1 touches only one hidden cell, so that cell is a mine.", "Then this 1 already has its 1 mine, so this cell must be safe."]
    );
    assert_eq!(zh(&h), ["先看这个 1，它周围只剩下 1 格，所以那一格是雷。", "再看这个 1，它周围的 1 个雷已经有了……所以这一格必定安全。"]);
}

#[test]
fn one_one_from_a_wall() {
    //   * # # * #
    //   1 1 1 1 1
    let p = pos(&["*##*#", "....."]);
    let h = p.hint(Some((0, 1)));
    assert_eq!(h.technique, Technique::Pattern(Pattern::EdgeOneOne));
    assert_eq!(h.grade(), Some(Grade::Medium));
    assert_eq!(target(&h), (p.at(2, 0), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(0, 1), p.at(1, 1)]);
    assert_eq!(
        en(&h),
        [
            "This 1 still needs 1 mine, and every hidden cell around it also touches that 1, so the shared cells hold exactly 1 mine.",
            "That 1 still needs 1 mine, no more than the shared cells hold, so this cell must be safe.",
        ]
    );
    assert_eq!(
        zh(&h),
        ["看这个 1，它周围还有 1 个雷，这些格子都挨着那个 1，所以共用的格子里恰好有 1 个雷。", "再看那个 1，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。"]
    );
    // The same 1-1 read from the other wall, nearer a move there.
    let h = p.hint(Some((4, 1)));
    assert_eq!(h.numbers, vec![p.at(4, 1), p.at(3, 1)]);
    assert_eq!(target(&h), (p.at(2, 0), Verdict::Safe));
}

#[test]
fn one_two_puts_the_difference_in_the_larger_numbers_own_cells() {
    //   * # *
    //   1 2 1
    //   (a 1-2 inside a 1-2-1, which names it)
    let p = pos(&["*#*", "..."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Pattern(Pattern::OneTwoOne));
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Mine));
    assert_eq!(h.numbers, vec![p.at(2, 1), p.at(1, 1)]);
    assert_eq!(
        en(&h),
        [
            "This 1 still needs 1 mine, so the cells it shares with that 2 hold at most 1 mine.",
            "That 2 still needs 2 mines, but the shared cells hold at most 1 mine, so the cells it doesn't share hold the rest: this cell must be a mine.",
        ]
    );
    assert_eq!(
        zh(&h),
        [
            "看这个 1，它周围还有 1 个雷，所以和那个 2 共用的格子里最多只能有 1 个雷。",
            "再看那个 2，它周围还有 2 个雷，共用的格子里最多只有 1 个雷，剩下的雷只能在不共用的格子里……所以这一格必须是雷。",
        ]
    );
}

#[test]
fn a_safe_cell_comes_before_a_mine_at_the_same_cost() {
    //   # * # *
    //   1 1 2 1
    let p = pos(&["#*#*", "...."]);
    let h = p.hint(None);
    assert!(h.technique.named());
    assert_eq!(h.target.unwrap().verdict, Verdict::Safe);
}

#[test]
fn a_provably_wrong_flag_is_reported_first() {
    // The 1-1 proves the flagged cell safe; the Single by the last move would come first.
    let p = pos(&["*#x*#", ".....", "....*"]);
    let h = p.hint(Some((4, 1)));
    assert_eq!(h.technique, Technique::WrongFlag);
    assert_eq!(target(&h), (p.at(2, 0), Verdict::WrongFlag));
    assert_eq!(h.target.unwrap().verdict.action(), Action::Unflag);
    let words = en(&h);
    assert_eq!(words.last().unwrap(), "The flag on it is wrong.");
    assert_eq!(zh(&h).last().unwrap(), "所以这面旗插错了。");
}

#[test]
fn a_flag_the_numbers_around_it_rule_out_is_still_reported() {
    // No one or two numbers show it, but a mine under the flag would overfill its numbers.
    let p = pos(&["#*.", ".x.", ".*."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::WrongFlag);
    assert_eq!(target(&h), (p.at(1, 1), Verdict::WrongFlag));
    assert!(h.numbers.iter().all(|&n| p.v.open[n].is_some()) && !h.numbers.is_empty());
    assert_eq!(en(&h), ["If this flagged cell were a mine, these numbers could not all be right, so it must be safe."]);
    assert_eq!(zh(&h), ["假设这一格是雷，这些数字就不可能都对……所以这一格必定安全。"]);
}

#[test]
fn a_flag_on_a_proven_mine_is_not_named_again() {
    let p = pos(&["..*#", "...."]);
    let first = p.hint(None);
    assert_eq!(target(&first), (p.at(2, 0), Verdict::Mine));
    let flagged = pos(&["..F#", "...."]);
    let next = flagged.hint(None);
    assert_ne!(next.target.map(|t| t.cell), Some(p.at(2, 0)), "the flagged mine is not named again");
    assert_eq!(target(&next), (p.at(3, 0), Verdict::Safe));
}

#[test]
fn nothing_perfect_logic_proves_says_so() {
    // A true 50/50: the 1 sees two cells, and the mine count can't tell them apart.
    let p = pos(&["*#", ".."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Nothing);
    assert_eq!(h.target, None);
    assert_eq!(h.stages(), 1);
    assert_eq!(words::nudge(&h, Lang::En), "No cell can be proven right now.");
    assert_eq!(words::nudge(&h, Lang::ZhCn), "现在推不出任何能确定的格子。");
}

#[test]
fn the_fewest_numbers_come_first_then_the_nearest() {
    // Singles to the left, a 1-1 by the last move on the right: one number beats two.
    let p = pos(&["*.....*##*#", "..........."]);
    let h = p.hint(Some((10, 1)));
    assert_eq!(h.technique, Technique::Single);
    assert_eq!(h.target.unwrap().cell, p.at(6, 0));
    // Two Singles: the one nearer the last move.
    let p = pos(&["*........*", ".........."]);
    assert_eq!(p.hint(Some((9, 1))).target.unwrap().cell, p.at(9, 0));
    assert_eq!(p.hint(Some((0, 1))).target.unwrap().cell, p.at(0, 0));
}


#[test]
fn reduction_subtracts_a_proven_mine_and_finds_a_one_one() {
    //   # # *
    //   * 3 1
    //   F 2 0     the 2 proves both (flagged) mines on the left; the 3, less them, is a 1.
    let p = pos(&["##*", "F..", "F.."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Reduction(Pattern::OneOne));
    assert_eq!(h.grade(), Some(Grade::Hard));
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(1, 2), p.at(2, 1), p.at(1, 1)]);
    assert_eq!(
        en(&h),
        [
            "First, this 2 touches only 2 hidden cells, so they are all mines.",
            "This 3 touches 2 mines of those, so it acts like a 1.",
            "This 1 still needs 1 mine, and every hidden cell around it also touches that 3, so the shared cells hold exactly 1 mine.",
            "That 3 still needs 1 mine, no more than the shared cells hold, so this cell must be safe.",
        ]
    );
    assert_eq!(
        zh(&h),
        [
            "先看这个 2，它周围只剩下 2 格，所以这些格子全是雷。",
            "这个 3 挨着其中 2 个雷，减雷后它实际上是 1。",
            "看这个 1，它周围还有 1 个雷，这些格子都挨着那个 3，所以共用的格子里恰好有 1 个雷。",
            "再看那个 3，它周围还有 1 个雷，已经在共用的格子里了……所以这一格必定安全。",
        ]
    );
}

#[test]
fn every_hint_on_real_fields_is_true_and_never_gives_odds() {
    // Expert fields played by the bot: each hint's cell is what it says, flags or not.
    let mut rng = crate::game::Rng::new(19);
    let mut named = 0;
    for _ in 0..12 {
        let mut g = Game::new(30, 16, 99, rng);
        rng = g.rng;
        g.open(0);
        for _ in 0..120 {
            if g.status != crate::game::Status::Playing {
                break;
            }
            let h = of(&g, None);
            for lang in Lang::ALL {
                let text: String = [words::name(h.technique, lang), words::nudge(&h, lang), words::look(&h, lang), words::verdict(&h, lang).unwrap_or("")].join(" ") + &words::reason(&h, lang).join(" ");
                for banned in ["%", "probab", "chance", "likely", "odds", "概率", "几率", "可能性"] {
                    assert!(!text.to_lowercase().contains(banned), "{text}");
                }
            }
            let Some(t) = h.target else {
                let v = crate::solver::View::of(&g);
                let Some(i) = crate::solver::play::guess(&v, &vec![Fact::Unknown; v.len()]) else { break };
                g.open(i);
                continue;
            };
            named += h.technique.named() as u32;
            assert!(h.numbers.iter().all(|&n| g.cells[n].cover == Cover::Open), "witnesses are open numbers");
            match t.verdict {
                Verdict::Mine => {
                    assert!(g.cells[t.cell].mine, "{h:?}");
                    g.mark(t.cell, false);
                }
                Verdict::Safe => {
                    assert!(!g.cells[t.cell].mine, "{h:?}");
                    g.open(t.cell);
                }
                Verdict::WrongFlag => panic!("no wrong flags here"),
            }
        }
    }
    assert!(named > 0);
}

/// Whether the open numbers at `numbers` alone settle `cell` as `mine`, by trying every
/// arrangement of their hidden cells: an independent check of the hint. With `count`, the
/// mines left to find (all but the flags in `known`) must fit the hidden cells too. None if
/// there are too many cells to try.
fn forces(v: &View, numbers: &[usize], known: &[usize], count: bool, cell: usize, mine: bool) -> Option<bool> {
    let hidden: Vec<usize> = (0..v.len()).filter(|&i| v.open[i].is_none() && !known.contains(&i)).collect();
    let mut cells: Vec<usize> = numbers.iter().flat_map(|&n| v.nbrs(n).iter().copied()).filter(|i| hidden.contains(i)).collect();
    cells.sort_unstable();
    cells.dedup();
    if cells.len() > 16 {
        return None;
    }
    let others = hidden.len() - cells.len();
    let left = v.mines - known.len();
    let (mut fits, mut against) = (false, false);
    for bits in 0u32..1 << cells.len() {
        let is_mine = |i: usize| cells.iter().position(|&c| c == i).is_some_and(|k| bits >> k & 1 == 1);
        let ok = numbers.iter().all(|&n| {
            let around = v.nbrs(n).iter().filter(|&&j| known.contains(&j) || is_mine(j)).count();
            around == v.open[n].unwrap() as usize
        });
        let m = bits.count_ones() as usize;
        if !ok || (count && (m > left || left - m > others)) {
            continue;
        }
        fits = true;
        let cell_can = |value: bool| match cells.iter().position(|&c| c == cell) {
            Some(k) => (bits >> k & 1 == 1) == value,
            // Outside the numbers: free without the count; with it, whatever the rest allows.
            None => !count || if value { left - m >= 1 } else { left - m < others },
        };
        against |= cell_can(!mine);
    }
    Some(fits && !against)
}

#[test]
fn every_hint_on_real_fields_uses_the_fewest_numbers_and_logic_never_goes_unnamed() {
    let mut rng = crate::game::Rng::new(5);
    let (mut deep, mut counted) = (0, 0);
    for (w, h, mines, games) in [(16, 16, 40, 8), (30, 16, 99, 2)] {
        for _ in 0..games {
            let mut g = Game::new(w, h, mines, rng);
            rng = g.rng;
            g.open(0);
            while g.status == crate::game::Status::Playing {
                let h = of(&g, None);
                let v = View::of(&g);
                let flags: Vec<bool> = g.cells.iter().map(|c| c.cover == Cover::Flag).collect();
                // Perfect logic proves a cell the player hasn't already marked: the hint names one.
                let odds = crate::solver::exact::odds(&v, &vec![Fact::Unknown; v.len()]).expect("small enough");
                let provable = (0..v.len()).any(|i| v.open[i].is_none() && (odds.safe[i] || (odds.mine[i] && !flags[i])));
                assert_eq!(h.target.is_some(), provable, "{h:?}");
                let Some(t) = h.target else {
                    let Some(i) = crate::solver::play::guess(&v, &vec![Fact::Unknown; v.len()]) else { break };
                    g.open(i);
                    continue;
                };
                let mine = t.verdict == Verdict::Mine;
                assert_eq!(g.cells[t.cell].mine, mine, "{h:?}");
                let count = matches!(h.technique, Technique::Count | Technique::Combinations);
                let known: Vec<usize> = if count { (0..v.len()).filter(|&i| flags[i]).collect() } else { Vec::new() };
                if let Some(ok) = forces(&v, &h.numbers, &known, count, t.cell, mine) {
                    assert!(ok, "its numbers prove it: {h:?}");
                    for k in 0..h.numbers.len() {
                        let mut less = h.numbers.clone();
                        less.remove(k);
                        assert_eq!(forces(&v, &less, &known, count, t.cell, mine), Some(false), "without {} it is not proven: {h:?}", h.numbers[k]);
                    }
                }
                deep += (h.numbers.len() > 2) as u32;
                counted += count as u32;
                if mine {
                    g.mark(t.cell, false);
                } else {
                    g.open(t.cell);
                }
            }
        }
    }
    println!("{deep} deep, {counted} counted");
    assert!(deep > 0 && counted > 0, "{deep} deep, {counted} counted");
}

#[test]
fn a_chain_reads_one_step_at_a_time_in_order() {
    //   F 2 F 2 #
    //   1 # 1 2 F     the 1 and the 2 on the left put a mine under the flag at the top; then
    //                 the middle 1 has its mine, so the cell beside it is safe.
    let p = pos(&["F.F.#", ".#..F"]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Chain);
    assert_eq!(h.grade(), Some(Grade::Hard));
    assert_eq!(target(&h), (p.at(1, 1), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(0, 1), p.at(1, 0), p.at(2, 1)]);
    assert_eq!(
        en(&h),
        [
            "This 1 still needs 1 mine, so the cells it shares with that 2 hold at most 1 mine.",
            "That 2 still needs 2 mines, but the shared cells hold at most 1 mine, so the cells it doesn't share are all mines.",
            "This 1 already has its 1 mine, so this cell must be safe.",
        ]
    );
    assert_eq!(
        zh(&h),
        [
            "看这个 1，它周围还有 1 个雷，所以和那个 2 共用的格子里最多只能有 1 个雷。",
            "再看那个 2，它周围还有 2 个雷，共用的格子里最多只有 1 个雷，剩下的雷只能在不共用的格子里……所以这些格子全是雷。",
            "看这个 1，它周围的 1 个雷已经够了……所以这一格必定安全。",
        ]
    );
}

#[test]
fn a_supposition_breaks_a_number_step_by_step() {
    //   1 # 2 # *
    //   1 * 2 * 2     were the corner safe, the 2 below it would need both cells to its left,
    //                 the middle 2 would be full, and the 1 on the left would get no mine.
    let p = pos(&[".#.#*", ".*.*."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Suppose);
    assert_eq!(h.grade(), Some(Grade::Evil));
    assert_eq!(target(&h), (p.at(4, 0), Verdict::Mine));
    assert_eq!(h.numbers, vec![p.at(4, 1), p.at(2, 0), p.at(0, 0)]);
    assert_eq!(
        en(&h),
        [
            "Suppose this cell were safe.",
            "This 2 still needs 2 mines and has only 2 hidden cells left, so they are all mines.",
            "This 2 already has its 2 mines, so its other hidden cells are safe.",
            "Then this 1 could not get all its mines.",
            "So this cell must be a mine.",
        ]
    );
    assert_eq!(
        zh(&h),
        ["假设这一格是安全的。", "这个 2 还差 2 个雷，只剩下 2 格……所以这些格子全是雷。", "这个 2 周围的 2 个雷已经够了……所以其余的格子都安全。", "那么这个 1 周围的雷就不够了。", "所以这一格必须是雷。"]
    );
}

#[test]
fn the_mine_count_settles_the_cells_no_number_needs() {
    //   # 1 # 2 #
    //   # # * 2 *     the 2s need both mines on the field: every other hidden cell is safe.
    let p = pos(&["#.#.#", "##*.*"]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Count);
    assert_eq!(h.grade(), Some(Grade::Evil), "mine counting is an Evil pattern on minesweeper.online");
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(3, 0)]);
    assert_eq!(en(&h), ["Only 2 mines left to find, and this number needs that many, so every other hidden cell is safe."]);
    assert_eq!(zh(&h), ["根据剩余雷数：只剩 2 个雷没找到，这个数字周围就要用掉这么多……所以其余的格子全部安全。"]);
    assert_eq!(words::name(h.technique, Lang::ZhCn), "剩余雷数");
}

#[test]
fn the_mine_count_rules_out_a_cell_that_would_leave_too_few() {
    //   * # 1 # # 1
    //   # 1 1 * * 1   a mine in the second cell would serve the 1s on the left at once,
    //                 leaving room for only 2 of the 3 mines.
    let p = pos(&["*#.##.", "#..**."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Combinations, "a supposition the mine count breaks");
    assert_eq!(target(&h), (p.at(1, 0), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(2, 0), p.at(5, 0), p.at(1, 1)]);
    assert_eq!(
        en(&h),
        ["Suppose this cell were a mine.", "Then even with every other hidden cell a mine there would be at most 2 mines, fewer than the 3 mines left to find.", "So this cell must be safe."]
    );
    assert_eq!(zh(&h), ["假设这一格是雷。", "那么就算其余的格子全是雷，最多也只有 2 个雷，凑不够剩下的 3 个雷。", "所以这一格必定安全。"]);
}

#[test]
fn the_mine_count_leaves_out_the_mines_already_flagged() {
    //   1 1 1 2 F 2
    //   # * F # * #   the numbers prove both flags, so 2 of the 4 mines are left to find,
    //                 and the 1 and the 2 at the top need both.
    let p = pos(&["....F.", "#*F#*#"]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Count, "{h:?}");
    assert_eq!(target(&h), (p.at(0, 1), Verdict::Safe));
    assert_eq!(h.numbers, vec![p.at(2, 0), p.at(5, 0)]);
    assert_eq!(en(&h), ["Only 2 mines left to find, and these numbers need that many, so every other hidden cell is safe."]);
    assert_eq!(zh(&h), ["根据剩余雷数：只剩 2 个雷没找到，这几个数字周围就要用掉这么多……所以其余的格子全部安全。"]);
    // Every mine found: the rest is safe, with no number to read.
    let p = pos(&["##...", "##.F#"]);
    let h = p.hint(None);
    assert_eq!((h.technique, h.numbers.len()), (Technique::Count, 0));
    assert_eq!(en(&h), ["Every mine is found, so every other hidden cell is safe."]);
    assert_eq!(zh(&h), ["根据剩余雷数：雷已经全部找到了……所以其余的格子全部安全。"]);
}
