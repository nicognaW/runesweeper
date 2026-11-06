//! Hints on fixed positions: the cell, the numbers and the reason, in both languages.
//!
//! A position is drawn as rows: `.` an open cell (its number follows from the mines), `#` a
//! hidden safe cell, `*` a hidden mine, `F` a flagged mine, `x` a flagged safe cell.

use super::*;
use crate::strings::Lang;

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
    assert_eq!(h.technique, Technique::Pattern(Pattern::OneOne));
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
    let p = pos(&["*#*", "..."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Pattern(Pattern::OneTwo));
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
fn nothing_one_or_two_numbers_settle_says_so() {
    // A true 50/50: the 1 sees two cells.
    let p = pos(&["*#", ".."]);
    let h = p.hint(None);
    assert_eq!(h.technique, Technique::Nothing);
    assert_eq!(h.target, None);
    assert_eq!(h.stages(), 1);
    assert_eq!(words::nudge(&h, Lang::En), "No one or two numbers settle any cell right now.");
}

#[test]
fn a_named_pattern_is_preferred_then_fewer_numbers_then_the_nearest() {
    // A Single far left, a 1-1 far right: the pattern wins even far from the move.
    let p = pos(&["*.....*##*#", "..........."]);
    let h = p.hint(Some((0, 1)));
    assert!(h.technique.named(), "{:?}", h.technique);
}


#[test]
fn reduction_subtracts_a_proven_mine_and_finds_a_one_one() {
    //   # # *
    //   * 3 1
    //   * 2 0     the 2 proves both mines on the left; the 3, less them, is a 1.
    let p = pos(&["##*", "*..", "*.."]);
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
