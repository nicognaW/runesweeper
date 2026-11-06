//! Guess hints on fixed positions: the reason, the cell and the words in both languages.
//!
//! A position is drawn as rows: `.` an open cell (its number follows from the mines), `#` a
//! hidden safe cell, `*` a hidden mine, `F` a flagged mine, `x` a flagged safe cell.

use super::super::{Field, Guess, Hint, Step, Technique, Verdict, advise, words};
use crate::game::{Cover, Game};
use crate::solver::{Fact, View, dead, exact, fifty, relocation};
use crate::strings::Lang;

struct Pos {
    v: View,
    flags: Vec<bool>,
    w: usize,
}

fn pos(rows: &[&str]) -> Pos {
    let g = crate::test_fields::position(rows);
    let flags = g.cells.iter().map(|c| c.cover == Cover::Flag).collect();
    Pos { v: View::of(&g), flags, w: g.w }
}

impl Pos {
    fn at(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }
    fn hint(&self) -> Hint {
        self.with(&Field::default())
    }
    fn with(&self, field: &Field) -> Hint {
        advise(&self.v, &self.flags, None, field)
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

/// Every word a hint can show, at every stage.
fn all_words(h: &Hint, lang: Lang) -> String {
    [words::name(h.technique, lang), words::nudge(h, lang), words::look(h, lang), words::verdict(h, lang).unwrap_or("")].join(" ") + " " + &words::reason(h, lang).join(" ")
}

const BANNED: [&str; 9] = ["%", "percent", "probab", "chance", "likely", "odds", "概率", "几率", "可能性"];

fn no_odds(h: &Hint) {
    for lang in Lang::ALL {
        let text = all_words(h, lang).to_lowercase();
        for banned in BANNED {
            assert!(!text.contains(banned), "{text}");
        }
    }
}

#[test]
fn a_true_fifty_fifty_says_take_it_now() {
    // The two 1s below see both cells, and the mine count can't tell them apart.
    let p = pos(&["*#", ".."]);
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::FiftyFifty));
    assert_eq!(h.grade(), None);
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Guess), "tied: the first");
    assert_eq!(h.partner(), Some(p.at(1, 0)));
    assert_eq!(h.numbers, vec![p.at(0, 1), p.at(1, 1)]);
    assert_eq!(h.stages(), 4);
    assert_eq!(words::name(h.technique, Lang::En), "True 50/50");
    assert_eq!(words::name(h.technique, Lang::ZhCn), "死猜");
    assert_eq!(words::nudge(&h, Lang::En), "No cell can be proven. Look for two cells nothing can ever tell apart.");
    assert_eq!(words::verdict(&h, Lang::En), Some("Guess this cell."));
    assert_eq!(words::verdict(&h, Lang::ZhCn), Some("猜这一格。"));
    assert_eq!(
        en(&h),
        ["These two cells hold one mine between them, and every number open now or later touches both or neither, so nothing can ever tell them apart. Guess now: waiting gains nothing."]
    );
    assert_eq!(zh(&h), ["这两格里恰好有 1 个雷，现在和以后能打开的数字要么同时挨着这两格，要么都不挨着……永远分不出来，这是死猜。现在就猜，等下去也没用。"]);
    no_odds(&h);
}

#[test]
fn a_flagged_half_of_a_fifty_fifty_is_not_named() {
    let p = pos(&["x*", ".."]);
    assert_eq!(target(&p.hint()), (p.at(1, 0), Verdict::Guess));
}

#[test]
fn in_max_guess_the_hint_says_the_fifty_fifty_was_announced() {
    let p = pos(&["*#", ".."]);
    let h = p.with(&Field { announced: &[(0, 1)], ..Field::default() });
    assert_eq!(h.steps.last(), Some(&Step::Announced { count: 1 }));
    assert_eq!(en(&h)[1], "This is the 50/50 this field was built with.");
    assert_eq!(zh(&h)[1], "这就是本局开局时预告的那个死猜。");
    let h = p.with(&Field { announced: &[(0, 1), (5, 9), (12, 13)], ..Field::default() });
    assert_eq!(en(&h)[1], "This is one of the 3 50/50s this field was built with.");
    assert_eq!(zh(&h)[1], "这是本局开局时预告的 3 个死猜之一。");
    // A 50/50 the field wasn't built with says nothing more.
    let h = p.with(&Field { announced: &[(5, 9)], ..Field::default() });
    assert_eq!(h.steps, vec![Step::FiftyFifty { cells: [0, 1] }]);
}

#[test]
fn in_classic_a_fifty_fifty_names_the_half_away_from_the_top_left() {
    // XP's move puts the first click's mine in the corner more often: the other half is safer.
    let p = pos(&["*#", ".."]);
    let first = p.at(0, 1);
    let h = p.with(&Field { classic_first: Some(first), ..Field::default() });
    assert_eq!(h.technique, Technique::Guess(Guess::FiftyFifty));
    assert_eq!(target(&h), (p.at(1, 0), Verdict::Guess));
    let odds = relocation::odds(&p.v, &[Fact::Unknown; 4], first).unwrap();
    assert!(odds.p[p.at(1, 0)] < odds.p[p.at(0, 0)]);
}

#[test]
fn a_pseudo_fifty_fifty_says_guess_it_now() {
    let p = pos(&[".#.#", "*##*", "**#."]);
    let facts = vec![Fact::Unknown; p.v.len()];
    assert!(fifty::fifty_fifties(&p.v, &facts).is_empty(), "no true 50/50");
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Pseudo));
    let (cell, verdict) = target(&h);
    assert_eq!((cell, verdict), (p.at(1, 0), Verdict::Guess));
    let partner = h.partner().expect("a partner");
    assert!(fifty::pseudo(&p.v, &facts).contains(&(cell, partner)));
    assert_eq!(words::name(h.technique, Lang::ZhCn), "伪死猜");
    assert_eq!(
        en(&h),
        ["If this cell is a mine, it and that one form a 50/50 nothing can split, since the cells that could tell them apart would all be mines. If it is safe, it opens. Either way, guessing it now costs nothing."]
    );
    assert_eq!(zh(&h), ["如果这一格是雷，它和那一格就成了死猜：能分出它们的格子都会是雷。如果它不是雷，就能打开。所以现在猜它不吃亏。"]);
    no_odds(&h);
}

#[test]
fn a_dead_cell_says_avoid_it() {
    let p = pos(&["#.#*", "##*.", "#*##"]);
    let facts = vec![Fact::Unknown; p.v.len()];
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Dead));
    assert_eq!(target(&h), (p.at(0, 1), Verdict::Avoid));
    assert!(dead::dead_cells(&p.v, &facts).contains(&(p.at(0, 1), 1)));
    assert_eq!(h.target.unwrap().verdict.action(), None, "nothing to apply");
    assert_eq!(words::verdict(&h, Lang::En), Some("Avoid this cell."));
    assert_eq!(words::verdict(&h, Lang::ZhCn), Some("别点这一格。"));
    assert_eq!(en(&h), ["If this cell is safe it can only show a 1, which tells you nothing new, so you would have to guess again. Avoid it."]);
    assert_eq!(zh(&h), ["这一格如果安全，只能是 1，打开了也没有新信息，还得再猜。别点它。"]);
    no_odds(&h);
}

#[test]
fn a_few_arrangements_counted_by_hand() {
    //   # * * .      The 1 at the top right sees three hidden cells: it can be satisfied
    //   # * # #      three ways, and each of them is the mine in only one.
    //   # # # *
    let p = pos(&["#**.", "#*##", "###*"]);
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Ways));
    assert_eq!(target(&h), (p.at(2, 0), Verdict::Guess));
    assert_eq!(h.numbers, vec![p.at(3, 0)]);
    assert_eq!(en(&h), ["This 1 can be satisfied 3 ways, all with the same number of mines, so each comes up as often. This cell is a mine in only 1 of them."]);
    assert_eq!(zh(&h), ["这个 1 只有 3 种摆法，雷数都一样，每种都一样常见。这一格只在其中 1 种里是雷。"]);
    no_odds(&h);
    lowest_of(&p, &h, &[p.at(2, 0), p.at(2, 1), p.at(3, 1)]);
}

#[test]
fn fewer_mines_come_up_more_often() {
    let p = pos(&["##*##", "*#.*#", "*..#*"]);
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Fewer));
    assert_eq!(target(&h), (p.at(3, 1), Verdict::Guess));
    assert_eq!(
        en(&h),
        [
            "These numbers can be satisfied with as few as 3 mines, or with more. The other mines go among the 4 hidden cells away from the numbers, and each mine fewer here leaves more ways to place them, so arrangements with fewer mines here come up more often.",
            "This cell is safe in every arrangement with only 3 mines.",
        ]
    );
    assert_eq!(
        zh(&h),
        ["这几个数字最少用 3 个雷就能满足，也可以更多。其余的雷在不挨着数字的 4 格里，这里每少一个雷，其余的雷就有更多摆法，所以雷少的摆法更常见。", "在只用 3 个雷的摆法里，这一格都是安全的。"]
    );
    no_odds(&h);
}

#[test]
fn a_corner_away_from_the_numbers_may_open_an_area() {
    let p = pos(&["#*##", "#*#*", "..*#"]);
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Opening));
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Guess));
    assert!(h.numbers.is_empty());
    assert_eq!(
        en(&h),
        ["No number touches this cell, so nothing makes it riskier than the rest: the 10 hidden cells left hold 4 mines. With only 3 neighbours, it is the most apt to be empty and open an area."]
    );
    assert_eq!(zh(&h), ["这一格不挨着任何数字，没有额外的风险：剩下的 10 格里有 4 个雷。它只有 3 个相邻格，最容易是空，一下开出一片。"]);
    no_odds(&h);
}

#[test]
fn early_in_classic_the_hint_teaches_the_top_left_quirk() {
    // The first click in the bottom right opened only its 1. Every cell away from it is
    // alike, but for XP's move of a first-click mine to the top left.
    let p = pos(&["#####", "*#*##", "####*", "*###."]);
    let first = p.at(4, 3);
    let h = p.with(&Field { classic_first: Some(first), ..Field::default() });
    assert_eq!(h.technique, Technique::Guess(Guess::Opening));
    assert_eq!(target(&h), (p.at(4, 0), Verdict::Guess), "the other top corner");
    assert_eq!(h.partner(), Some(p.at(0, 0)), "the top left is ringed too");
    assert_eq!(
        en(&h)[1],
        "Not the top-left corner, though: in Classic, as in Windows XP, a mine under the first click moves there (or along the top row), so early on it hides a mine more often."
    );
    assert_eq!(zh(&h)[1], "但别选左上角：原版里，第一下点到的雷会被移到那里（已有雷就往右移），所以开局时那一格更容易是雷。");
    no_odds(&h);
    // Outside Classic, the corners are alike and the first wins.
    let h = p.hint();
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Guess));
    assert_eq!(h.steps.len(), 1);
}

/// A Classic first click in the middle that opened only its 1, on a dense field: the 1's
/// cells are safer than those away from it, but by no reason a hand count shows.
const EARLY: [&str; 5] = ["####*#", "######", "##.##*", "###*##", "*####*"];

#[test]
fn early_in_classic_with_no_other_reason_the_hint_says_avoid_the_top_left() {
    let p = pos(&EARLY);
    let first = p.at(2, 2);
    let h = p.with(&Field { classic_first: Some(first), ..Field::default() });
    assert_eq!(h.technique, Technique::Guess(Guess::TopLeft));
    assert_eq!(target(&h), (p.at(0, 0), Verdict::Avoid));
    assert_eq!(words::name(h.technique, Lang::En), "Classic's top-left quirk");
    assert_eq!(words::name(h.technique, Lang::ZhCn), "原版首击移雷");
    assert_eq!(
        en(&h),
        ["In Classic, as in Windows XP, a mine under the first click moves to the top-left corner (or the next free cell along the top row), so early on this cell hides a mine more often than it would otherwise. Avoid it."]
    );
    assert_eq!(zh(&h), ["原版里，第一下点到的雷会被移到左上角（那里已有雷就往右移），所以开局时这一格比平常更容易是雷。别点它。"]);
    no_odds(&h);
    // Outside Classic nothing favours a cell a hand count could show.
    assert_eq!(p.hint().technique, Technique::Guess(Guess::Blind));
}

#[test]
fn when_nothing_favours_a_cell_the_hint_names_none() {
    let p = pos(&["#*.#", "##.#", "*##*"]);
    let h = p.hint();
    assert_eq!(h.technique, Technique::Guess(Guess::Blind));
    assert_eq!(h.target, None);
    assert!(h.numbers.is_empty());
    assert_eq!(h.stages(), 1, "a nudge, and nothing more");
    assert_eq!(words::nudge(&h, Lang::En), "No cell can be proven, and nothing you could check by hand favours any cell: a guess is needed here.");
    assert_eq!(words::nudge(&h, Lang::ZhCn), "推不出能确定的格子，也没有能手算的理由偏向哪一格：这里只能猜。");
    assert_eq!(words::name(h.technique, Lang::ZhCn), "必须猜");
    no_odds(&h);
}

#[test]
fn a_provable_cell_never_gets_a_guess() {
    // The 1 in the corner touches one hidden cell.
    let p = pos(&["..*#", "...."]);
    assert!(!matches!(p.hint().technique, Technique::Guess(_)));
}

/// The named cell is no likelier a mine than any of `covered`, by the exact odds.
fn lowest_of(p: &Pos, h: &Hint, covered: &[usize]) {
    let odds = exact::odds(&p.v, &vec![Fact::Unknown; p.v.len()]).unwrap();
    let cell = h.target.unwrap().cell;
    for &c in covered {
        assert!(odds.p[cell] <= odds.p[c] + 1e-9, "{cell} at {} but {c} at {}", odds.p[cell], odds.p[c]);
    }
}

#[test]
fn on_random_positions_guesses_name_the_least_likely_covered_cell_and_never_give_odds() {
    // Small fields with cells open: wherever no cell can be proven, the hint's cell is
    // the least likely mine among the cells its reason covers, worked out here from the
    // solver's own detectors.
    let mut seed = crate::generate::Stream::new(77);
    let mut seen = std::collections::HashMap::new();
    for round in 0..20000 {
        let (w, h) = (4 + round % 3, 3 + (round / 3) % 2);
        let mines = 1 + seed.below(w * h / 3);
        let mut picks: Vec<usize> = (0..w * h).collect();
        for i in 0..mines {
            let j = i + seed.below(picks.len() - i);
            picks.swap(i, j);
        }
        let mut g = Game::with_mines(w, h, &picks[..mines]);
        let safe: Vec<usize> = (0..w * h).filter(|&i| !g.cells[i].mine).collect();
        if round % 2 == 0 {
            // Cells open at random, as no game could leave them...
            for &i in &safe {
                if seed.below(100) < 55 {
                    g.cells[i].cover = Cover::Open;
                }
            }
        } else {
            // ...or as a few clicks do.
            for _ in 0..1 + seed.below(3) {
                g.open(safe[seed.below(safe.len())]);
            }
            if g.status != crate::game::Status::Playing {
                continue;
            }
        }
        let v = View::of(&g);
        let flags = vec![false; w * h];
        let hint = advise(&v, &flags, None, &Field::default());
        let Technique::Guess(reason) = hint.technique else { continue };
        *seen.entry(format!("{reason:?}")).or_insert(0) += 1;
        no_odds(&hint);
        let facts = vec![Fact::Unknown; v.len()];
        let odds = exact::odds(&v, &facts).unwrap();
        assert!((0..v.len()).all(|i| v.open[i].is_some() || !(odds.safe[i] || odds.mine[i])), "nothing provable");
        let covered: Vec<usize> = match reason {
            Guess::FiftyFifty => fifty::fifty_fifties(&v, &facts).into_iter().flat_map(|(a, b)| [a, b]).collect(),
            Guess::Pseudo => fifty::pseudo(&v, &facts).into_iter().map(|(x, _)| x).collect(),
            Guess::Dead => dead::dead_cells(&v, &facts).into_iter().map(|(x, _)| x).collect(),
            Guess::Opening => (0..v.len()).filter(|&i| odds.interior[i]).collect(),
            Guess::Blind => {
                assert_eq!(hint.target, None);
                continue;
            }
            _ => continue,
        };
        let cell = hint.target.expect("a cell").cell;
        assert!(covered.contains(&cell), "round {round}: {hint:?}");
        for c in covered {
            assert!(odds.p[cell] <= odds.p[c] + 1e-9, "round {round}: {cell} at {} but {c} at {}", odds.p[cell], odds.p[c]);
        }
    }
    // True 50/50s are left to the fixtures and Max Guess: small fields rarely stall on one.
    for reason in ["Pseudo", "Dead", "Ways", "Fewer", "Opening", "Blind"] {
        assert!(seen.contains_key(reason), "{reason} never seen: {seen:?}");
    }
}
