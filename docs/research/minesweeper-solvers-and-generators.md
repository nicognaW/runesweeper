# One exact solver should power every mode

The strongest Minesweeper solvers all use the same recipe. They apply trivial single-number rules first. Then they compute **exact** mine probabilities by enumerating each connected frontier component and weighting by the global mine count. They choose guesses with a one-step look-ahead that rewards survival and information. In small endgames they search the full game tree. That recipe wins **about 40–41% of classic Expert games** (30×16/99, safe but not guaranteed-open first click), and **45.6–54.3% when the first click is a guaranteed opening**. The first-click rule moves the number more than any solver improvement since 2000.

The same engine is everything Runesweeper needs for its three modes and its hint helper:

- **OG** keeps today's Windows XP behaviour.
- **Guess-free** should use **rejection sampling against an exact solver**, generating the board on the first click with the 3×3 around it kept mine-free. This gives a uniform draw over every board a perfect logician can finish. Simon Tatham's perturbation generator is the most established guess-free algorithm, but its authors admit it packs mines into edges and corners.
- **Max-guess** has no published algorithm. The best-supported design inverts Tatham's loop: a solver-in-the-loop local search that maximises verified forced guesses, preferring true 50/50s, under a cap on how far the board may drift from a uniform-looking layout. The same search, given other objectives, produces solver load tests.
- **The hint helper** should follow the Sudoku-tool pattern. It reveals one hint in stages (nudge, then region, then cell, then reason). It always picks the deduction that uses the fewest numbers. Its wording is the vocabulary players already learn ("1-2-X", "reduction", "true 50/50", "dead cell"). When nothing can be proven it says so honestly and switches to probability.

One trap is specific to Runesweeper. XP's first-click relocation makes the posterior over boards non-uniform, so the mine odds of the top-left scan cell double. A solver or hint that ignores this is slightly wrong in OG mode.

## Exact counting plus look-ahead reaches about 40% on classic Expert

### The recipe every top solver shares

David N. Hill's JSMinesweeper is the reference implementation. Its `doSolve()` runs a fixed pipeline and returns as soon as one stage produces a move:

1. Trivial single-witness rules.
2. An exact BigInt probability engine.
3. Dead-tile and 50/50 detection.
4. Exact game-tree search ("BFDA") once few solutions remain.
5. Otherwise a heuristic guess.

Source: ([JSMinesweeper solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/solver_main.js)).

The probability engine groups hidden tiles into **"boxes"**: tiles touched by exactly the same set of revealed numbers. It then folds in one number at a time, like dynamic programming. Each state ("line") records a mine count, a BigInt solution count and the mines allocated per box. Once there are at least 100 lines, it **merges lines that agree on every boundary box**. This is exact, because the constraints not yet processed depend only on boundary boxes. That merging is what keeps long frontiers tractable ([solver_probability_engine.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/solver_probability_engine.js)).

Independent components are convolved by mine count. The unconstrained interior is never enumerated. Each total frontier mine count m is weighted by **C(minesLeft − m, tilesOffEdge)**, and every interior tile shares a single "off-edge" probability. So a large interior costs nothing.

Drew Roos's minesweepr does the same in a geometry-agnostic way. It condenses cells into "supercells", reduces subset rules, cross-eliminates permutations and splits fronts. The global count is folded in with weights C(n,k)/C(n,k₀) ([minesweepr source](https://github.com/mrgriscom/minesweepr/blob/b9d9989ad8b2955c4c2a44690b34f88fc02ed4d5/minesweeper.py)). Its author's summary of why decomposition matters: "what would have taken a×b×c amount of work takes a+b+c instead" ([mrgris.com](https://mrgris.com/projects/minesweepr/)).

### Where Hill's solver gains its edge: how it guesses

Hill's lead over plain "safest cell" solvers comes from how it chooses guesses.

**Candidates.** The code takes tiles within **0.1 (subtractive)** of the best living safety, widening to 0.2 if fewer than two qualify. The README describes a multiplicative "within 10%" rule instead; the code is current.

**Secondary safety.** Each candidate is scored by revealing every value it could show, re-running the full engine, and summing P(value) × the best next-move safety. This is multiplied by a factor for how much the tile could resolve a 50/50 (1 + 0.9 × influence / safety tally).

**Progress.** "Progress" now enters only as a **0.001 tie-break**. The older 0.2 weight survives only in an unused path. Extra dominance checks swap in a tile that is strictly better in every outcome ([solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/solver_main.js)).

**Dead tiles and 50/50s.** A **dead tile** is a tile that is either a mine or can show only one value. A **true 50/50** is two tiles sharing one mine that can never receive different information. A **pseudo-50/50** is a tile that is either safe or part of a 50/50. The solver detects all three and **guesses 50/50s before playing available safe moves**: `EARLY_FIFTY_FIFTY_CHECKING` replaces the safe moves with the 50/50 guess. The README's reasoning: "Since the 50/50 will never change it is always correct to guess these at the earliest opportunity". It also notes the detector cannot find *extended* pseudo-50/50s ([JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/README.md); [LongTermRiskHelper.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/LongTermRiskHelper.js)).

**Endgame search (BFDA).** This runs below **1,000 remaining solutions in play** and 5,000 in analysis; the README's "750" is out of date. It is further capped at 75 million enumeration cycles and 100 million search nodes. It stores each solution as the value every tile would show, and partitions solutions by the value a probe reveals. It prunes alpha-style, merges tiles whose hashes show they split the solutions identically, caches positions in a transposition table, and stops looking at a node once it finds a certainly safe move ([BruteForceAnalysis.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/BruteForceAnalysis.js)).

The Java README shows why this matters. A 66.67%-safe tile wins **46.23%**, against 44.64% for the best 75%-safe tile, found by exploring 67,984 nodes over 1,008 solutions in **253 ms** ([Hill Java README](https://github.com/DavidNHill/Minesweeper/blob/827e5b9b898ec9b8a8b03383b42a374ad12586b1/README.md)).

### Measured win rates

| Solver | First-click rule | Expert 30×16/99 | Games | Status |
|---|---|---|---|---|
| Hill, Java ([JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/README.md)) | safe, corner start | **41%** (older README: 40%) | not stated | unreviewed |
| Hill, Java | opening at (3,3) | **54.3%** (older: 53%) | not stated | unreviewed |
| Tu et al. PSEQ-D256 ([AAAI-17 WS](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)) | safe, corner start | **40.06(2)%** | 1.88×10⁷ | peer-reviewed workshop |
| Tu et al. PSEQ | safe, corner start | 39.616(5)% | 3.2×10⁸ | peer-reviewed workshop |
| Liu et al. PAR / PAFG ([KBS 2022](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)) | safe / 3×3 safe | 39.7% / **45.6%** | 10⁴ | journal |
| ArolaunTech ([README](https://github.com/ArolaunTech/minesweeperbot)) | not stated | 39.33 ± 0.30% | 10⁵ | hobby |
| Buffet et al. UCT "OH" / HCSP ([ICS 2012](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)) | safe, corner | 38.7 ± 1.8% / 38.1 ± 0.5% | 10⁴ (HCSP) | conference |
| Legendre et al. ([INRIA RR-8041](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf)) | safe, corner | 37.3–37.6% | 10⁴, paired seeds | tech report |
| minesweepr, safest cell / plus corners ([mrgris](https://mrgris.com/projects/minesweepr/)) | safe | 33.1% / 37.8% | 10⁵ | hobby |
| Becerra CSCSP ([Harvard thesis](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf)) | safe | 32.90% | 10⁵ | thesis |
| Best CNN ([arXiv 2212.10446](https://arxiv.org/pdf/2212.10446)) | safe | 21.5% | ≥10⁵ | BSc report |

Tu et al.'s filter ablation shows where the points come from:

- Picking the minimum-probability cell alone ("P") wins **35.56%** with a corner start.
- Adding "S", which prefers cells where at least one cell is certainly safe on the next move, lifts it to **39.25%**.
- Expected safe cells ("E") and outcome entropy ("Q") add a further 0.37 points.
- Exact endgame recursion below 256 solutions adds 0.45 points.

Source: ([Tu et al. Tables 4–5](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)). ArolaunTech separately estimates its endgame search at about +0.7 points ([README](https://github.com/ArolaunTech/minesweeperbot)).

Guessing policy matters far more than deduction. Deterministic inference was essentially complete by 2000. The gains since came from global-count probabilities, look-ahead, 50/50 handling and corners. Studholme's CSP solver won only 5.4% of standard Expert games *without* guessing and 28.6% *with* guessing ([Studholme, Fig. 6](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf)). The true optimum for classic Expert is unknown. Three independent methods converge at 40–41%, so the optimum is probably only slightly higher. That is an inference, not a bound. Tu et al.'s provable two-move survival bound (0.897 with a corner start) is far looser.

### Hardness theory says exactness is cheap on random boards

The worst-case complexity is daunting:

- **Consistency** is NP-complete ([Kaye 2000](https://api.crossref.org/works/10.1007/BF03025367)).
- **Inference** (can any cell be proven?) is coNP-complete. The 2011 proof was flawed; the corrected proof is Hendrickson & Tockman's 2024 one ([arXiv 2404.14519](https://arxiv.org/pdf/2404.14519)).
- **Counting** layouts is #P-complete ([Nakov & Wei 2003](https://minesweepergame.com/math/minesweeper-minesweeper-2003.pdf)).
- **Maximising win probability** is PP-hard, and PSPACE-complete when the win chance may be exponentially small ([de Bondt 2012](https://arxiv.org/pdf/1204.4659)).

Random boards at standard densities split into small components. In Studholme's 600,000 games, more than 99% of constraint sets had under 1,000 solutions, and the largest had 1,620,880 ([Studholme p. 14](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf)). Hill reports "50x50/500 should be fine, 100x100/2500 might struggle" ([README](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/README.md)).

So a compiled Rust engine needs **no sampling on standard boards**. It does need big integers or log-domain arithmetic: a full-board count on Expert can approach C(381, 99), about 10⁹³ (arithmetic), far beyond u128. Hill decides certainty with exact comparisons (mine tally == 0, == total), not epsilons. That is a design point worth copying, because a hint must never call a 99.9999% cell "safe".

### Runesweeper's XP relocation breaks the uniform-posterior assumption

Every exact engine assumes all consistent layouts are equally likely. XP-style relocation violates this. minesweepergame.com warns that "the top left corner where the probability of being a mine doubles after the first click (due to mine shifting)" ([Guessing](https://minesweepergame.com/strategy/guessing.php)). Runesweeper reproduces the mechanism exactly: a first-click mine moves to the first free cell in row-major order, within rows 0..h−1 and columns 0..w−1 ([src/game.rs L175–187](https://github.com/nicognaW/runesweeper/blob/main/src/game.rs)).

Working it through gives an exact correction **[derived; not from a source]**. A final layout L has prior weight **1 + r(L)**. Here r(L) counts the mines at the start of the scan order, from the first cell up to the first non-mine, with the clicked cell counted as mined. Each such mine could have been the relocated one. In practice, r is 1 when the first scan cell holds a mine, and 0 otherwise. So layouts with that cell mined count twice, and its odds double, from about 21% to about 34% on a fresh Expert board (arithmetic).

An engine handles this exactly as a weighted mixture. Run the engine once as usual. Run it again with the first scan cell forced to a mine (and a third time with the first two forced, which is negligible in practice). Add the runs, weighting each by its solution count. Without this, BFDA's "exact" win chances and the hint helper's percentages are slightly off in OG mode near the top-left. The bias does not exist in Guess-free or Max-guess modes, which generate after the click.

## Benchmarks disagree on boards, first clicks and sample sizes

No shared benchmarking standard exists. Four choices drive the published numbers:

- **The preset.** "Beginner" is 8×8/10 in Tu et al. and Studholme, 9×9/10 in Legendre, Becerra and Liu, and 10×10/10 in Pedersen. Studholme's "Intermediate" was 15×13/40.
- **The first-click rule.** Unprotected, safe cell, or safe 3×3 neighbourhood.
- **The opening cell.** Corner, (3,3), centre or random.
- **The game count.** 10⁴ for Studholme, Legendre and Liu; 10⁵ for Pedersen, Becerra and mrgris; up to 3.2×10⁸ for Tu.

Only Legendre et al. state that every algorithm played the same seeded minefields ([RR-8041 p. 17](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf)). Tu et al. give Beta-quantile error estimates ([Tu et al.](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)). Buffet et al. print "±" values without saying what they are ([Buffet et al.](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)).

**Comparisons often mix boards.** Tu's own comparison table mixes 8×8 and 15×13 rows. A Hugging Face dataset claims 95.5% on 9×9 Beginner under a safe single-cell centre start, far outside every peer-reviewed result (87.9–91.4%). It should be treated as unreliable ([LKForge card](https://huggingface.co/datasets/LKForge/minesweeper-solver-benchmark/raw/main/README.md)).

**The opening rule dominates.** A guaranteed opening adds about 6 points on Expert in Liu (39.7% to 45.6%) and about 13 points in Hill (41% to 54.3%). Corner starts add 4.7–8.2 points to a naive solver ([mrgris](https://mrgris.com/projects/minesweepr/)). Under a safe-neighbourhood rule, corners stop being optimal: Liu measures a near-centre start at about 85% against 71.5% for the top-left corner ([Liu et al. p. 9](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)).

**Precision.** At p ≈ 0.4, 10,000 games give a 95% interval of about ±1.0 point, and 100,000 games about ±0.3 (binomial arithmetic). Separating two modern solvers that are 0.5 points apart therefore needs roughly 38,000 unpaired games each. Paired seeds cut this sharply.

**Secondary metrics:**

- Guesses per game: Becerra's CSP makes 4.73 guesses per Expert game ([Becerra Table 6.2](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf)); Hill's solver averages 3.3 per classic Expert win.
- The fraction of games won without guessing.
- The rate at which the exact-work cap fired: Legendre's 1-second timeout fired in up to 7.9% of Expert games for one variant.
- Time per move.

For Runesweeper, a bot benchmark should state its rule outright: "30×16/99, XP relocation (safe, not open), first click at (0,0), MSVC-LCG seeds s₁…sₙ, every solver on the same seeds". It should report a Wilson interval, guesses per game, the no-guess fraction, how often the cap fired, and p50/p99 move latency. Under that rule the targets are **Tu's 40.06% and Hill's 40–41%**. Runesweeper's relocation differs from Tu's "mines uniform over all other cells", so a small systematic gap is expected and should be measured rather than assumed.

## Experts score games with flood-fill metrics and replay probabilities

### Difficulty and efficiency metrics

The competitive community's metrics all rest on two flood-fill concepts. An **opening** is an 8-connected region of zeros plus its numbered border. An **island** is a group of numbers that touches no zero.

The owner definitions on minesweepergame.com:

- **3BV** is "the minimum number of left clicks required to win the game. Each opening is one click and each number that does not touch an opening is one click". It is a no-flag, no-chord count.
- **3BV/s** = 3BV / Time.
- **IOE** = 3BV / Total Clicks, "and the factors of IOE are Correctness and Throughput". So **IOE = Correctness × Throughput**, where Correctness is effective clicks over total clicks, and Throughput is 3BV over effective clicks.
- **IOS** = log(3BV) / log(Time).
- **RQP** = Time / (3BV/s). This equals Time²/3BV, so lower is better, despite the page's wording.

Source: ([Statistics](https://minesweepergame.com/statistics.php)).

The reference parser behind the world-ranking video player implements these exactly ([parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)). So does the Rust library **ms-toollib**, which backs the Chinese Metasweeper tools ([ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)).

**ZiNi** measures difficulty for flaggers. It is a *greedy simulation*, not an optimum. Each cell gets a "premium": the 3BV a chord there would clear, minus the clicks the chord costs. The simulation repeatedly flags and chords the highest-premium cell ([ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)). The board's "ZiNi" therefore depends on the variant: Greedy, Human, Random, 8-way or PTT. minesweeper.online's default is the PTT variant ([Gadgetlab overview](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)). STNB is not defined on any owner page, and ms-toollib's constants differ from the fan-wiki ones.

### Analysis tools and how rankings separate luck from skill

Analysis has two layers:

- **Video statistics.** Legacy clones record AVF, MVF, MVR or RMV videos. The RAWVF parser normalises them and computes 3BV, ZiNi, clicks by type, wasted clicks, IOE, Corr, ThrP, openings and islands.
- **Probability replay.** This is the newer layer.

ms-toollib tags each replayed click:

- **`high_risk_guess`** when the clicked cell's mine probability was ≥ 0.51.
- **`needless_guess`** when the player guessed while a provably safe cell lay within the surrounding 5×5.
- **`hard_judgment`** for cells deducible only by enumeration.

It also computes a luck score, **pluck = −Σ log₁₀(safety)**, over risky clicks ([analyse_methods.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/analyse_methods.rs); [pluck.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/pluck.rs)).

JSMinesweeper's analyser colours tiles: green for safe, red for mines, yellow for guess candidates (the best with a green centre), and grey for dead tiles ([README](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/README.md)). minesweeper.online tells players after a game whether "The safest move was chosen" ([Koro, The art of guessing](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)).

Rankings keep luck separate from skill with blunt filters rather than guess counts:

- 3BV floors for time records: **2/30/100** on minesweepergame.com, **5/30/100** on minesweeper.online ([Ranking Rules](https://minesweepergame.com/ranking-rules.php); [minesweeper.online Gameplay](https://minesweeper.online/help/gameplay)).
- A Beginner floor of 10 for 3BV/s rankings.
- A ban since 2011 on "lucky" games won by "random clicks and no solving".
- Video verification on secure clones.

The community vocabulary for guesses is precise. A **"forced guess"** is "a situation that must be solved with a guess regardless of future information (including minecount)". **NG logic** reasons from the fact that any layout requiring a guess is invalid on a no-guess board ([Archeaic, Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)).

Tools draw the line differently. ms-toollib's `is_solvable` ignores the remaining-mine count, so it disagrees with the community definition in endgames ([algorithms.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs)). Runesweeper should therefore say which definition its guess counts use.

## Pros learn two root patterns, then probability and speed

### Patterns and reduction

Teaching starts with two rules:

- **Rule 1:** a number touching as many hidden cells as its value has all of them as mines.
- **Rule 2:** a number with all its mines already flagged makes every other neighbour safe, and chording opens them at once.

minesweepergame.com builds everything else on two root patterns. **1-2-X**: "When you see 1-2-X on a row the X is always a mine". **1-1-X** from a border: the third square is safe. It treats the famous **1-2-1** and **1-2-2-1** as "Apply the 1-2-X pattern from the left. Apply the 1-2-X pattern from the right." **Reduction**: "You simply subtract the known mines from each number … if you flag a mine touching a 3 it becomes a 2" ([Patterns](https://minesweepergame.com/strategy/patterns.php)).

minesweeper.online publishes a 28-entry catalogue and tags each entry with the no-guess difficulty that needs it ([Patterns](https://minesweeper.online/help/patterns)):

| No-guess level | Patterns |
|---|---|
| Easy | B1, B2 (the two basic rules) |
| Medium | 1–1, 1–2 and their variants, 1–2–1, 1–2–2–1 |
| Hard | Reduction (1–1R, 1–2R, 1–2–1R), Holes |
| Evil | Triangles, 1–3–1 and 2–2–2 corners, 1>2<1, T-pattern, dependency chains, mine counting, combinations |

That is a ready-made, community-recognised difficulty ladder for both a hint helper and a guess-free grader.

### Guessing

Probability teaching descends from Sean Barrett's "Minesweeper: Advanced Tactics". It counts arrangements globally, warns that local probabilities "aren't independent of one another", and gives the rule of thumb "fewer mines means more unencountered variants" ([nothings.org](https://www.nothings.org/games/minesweeper/)).

minesweepergame.com lists six guessing strategies ([Guessing](https://minesweepergame.com/strategy/guessing.php)):

1. Guess quickly when no information can come.
2. Solve elsewhere first.
3. Look for safe squares, not mines.
4. Make the most useful guess. In a 33:66 row, opening the middle "will force you to make a second guess".
5. Use density: "When there is a 50:50 guess it is actually safer to open a random square!"
6. Calculate global probability.

The community guides add the named categories. **Linked 50/50s** are ones where "one guess resolves the situation, the other ends in another 50/50". **Pseudo 50/50s** "you should guess … immediately" ([Scar, What is a 50/50](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)). **Dead cells** "should not be played, even if the survival probabilities are good", and they "are the biggest enemy of hint users" ([Koro](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)). This is exactly JSMinesweeper's category set; the guides credit it as their analysis tool.

### Speed and the first click

Speed is mostly mechanics:

- Flag a mine only if you will chord on it.
- Use the three-action **"1.5 click"**, which cuts chording time by 25%.
- Choose flag or no-flag play locally: high 3BV and low numbers favour flagging.
- Shorten the mouse path.

Source: ([Efficiency](https://minesweepergame.com/strategy/efficiency.php)).

Kamil Murański, then holder of four world records, wrote that after years of practice "you don't have to think about where the mines are when you see these patterns", and that when forced you should "always open a square with the least chance of hitting a mine" ([PC World, 2011](https://www.pcworld.com/article/482195/how-to-play-minesweeper-like-a-pro.html)). The current Expert record is **26.59 s** (Ze-En Ju, 2022, 3BV 138) ([World Records](https://minesweepergame.com/world-records.php)). What separates the best players is automatic pattern recognition plus click economy, not new logic.

First-click advice conflicts because the rules differ. Kostka's exhaustive 2002 analysis found that openings are more likely at edges (0.12 to 0.50 on Expert) but larger in the middle. minesweepergame.com therefore tells new players to start in the middle ([First Click](https://minesweepergame.com/strategy/first-click.php)). minesweeper.online says "Start the game from the corners" ([Gameplay](https://minesweeper.online/help/gameplay)). Runesweeper should not hard-code either piece of advice in a hint.

## Tatham's perturbation is the established guess-free generator, rejection the cleanest

### How Tatham's generator works

Simon Tatham's Mines has shipped a guess-free generator since 2005. The original check-in "uses a solver during grid generation to arrange a mine layout that never requires guessing" ([git 739609c and history](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=739609c)). It is still maintained in 2026.

**Lay and solve.** It lays mines uniformly outside the first click's 3×3 area, so **the first click is always a zero**. It then runs a deliberately human-like set solver with three rules:

- single-number rules;
- pairwise "wing"/subset rules;
- a global mine-count step, enumerated only when there are **at most 10 sets**.

**Perturb when stuck.** When the solver stalls, `mineperturb()` picks a random stuck constraint set. It swaps mines between that set and other squares until the set is all-mine or all-clear, preferring frontier squares, then squares beyond the frontier, and only "as a very last resort" squares already solved.

**Re-solve and restart.** The solver re-runs from scratch until a run needs zero perturbations. The generator abandons a layout when the perturbation count fails to **strictly** decrease. Whole-region "big" perturbations are allowed only after 100 tries ([mines.c L2744–3290](https://github.com/samfundev/puzzles/blob/ede745aa858b1a47e3b12803879716ef173ed7a7/mines.c#L2744-L3290)).

The author is frank about the costs. Big perturbations make grids "a bit boring, with mines packed densely into far corners". A 2025 fix for a reproducible infinite loop added a progress counter, of which Tatham wrote, "I'm not at all happy with this fix … what I _ought_ to do is rethink the entire strategy of perturbation" ([commit a7c7826](https://github.com/samfundev/puzzles/commit/a7c7826bce5cbb9b9c337c11b9b7f8b278e76fba)). He also doubts a general forbidden-pattern rule exists, and warns that a cautious one would be "an extra hidden clue to the mine positions for anyone who knows how the generator is working" ([quasiblog](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/mines-solver/)).

### How the alternatives compare

| Approach | Example | Solver that defines "no guess" | Distribution | Termination |
|---|---|---|---|---|
| Perturbation | Tatham ([mines.c](https://github.com/samfundev/puzzles/blob/ede745aa858b1a47e3b12803879716ef173ed7a7/mines.c#L2744-L3290)) | single + pairwise + capped global | biased toward edges and corners | effectively always, even at 35% density |
| Repair | lrvideckis, quasarbright, sam-gc | rules + Gaussian elimination / SAT | authors admit mines drift outward and interiors get easier ([lrvideckis](https://github.com/lrvideckis/Minesweeper_Android/blob/7881613ab995e80da0f9e43e0898bede853b4754/app/src/main/java/com/LukeVideckis/minesweeper_android/minesweeperStuff/minesweeperHelpers/CreateSolvableBoard.java)) | usually |
| Hybrid: count-preserving repair, else reseed | JSMinesweeper ([MinesweeperGame.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/MinesweeperGame.js#L366-L500)) | exact probability engine (P = 1 only) | small local bias | up to 10⁶ solver calls |
| Rejection sampling | mindsweeper (Rust/WASM) ([local.rs](https://github.com/AlexBuz/mindsweeper/blob/b804faa439cf0c210289e7045f3cb7152fa386d1/src/server/local.rs#L335-L401)), Anonymine | exact component enumeration plus global count | **exactly uniform over solvable boards** | collapses at high density |
| Lazy / adversarial | Kaboom ([blog](https://pwmarcz.pl/blog/kaboom/)) | SAT with cardinality constraint | no fixed board | slows near 100 open squares |

**"No-guess" always means "solvable by a particular solver".** A weaker solver accepts a subset of what a stronger one accepts. Rejection with an exact solver therefore gives the largest, most varied pool. mindsweeper's author states the property directly: "If a particular mine arrangement is solvable by a perfect logician without guessing, then it's just as likely to be picked by the algorithm as every other viable arrangement", and claims generation takes "a matter of milliseconds" ([mindsweeper README](https://github.com/AlexBuz/mindsweeper/blob/b804faa439cf0c210289e7045f3cb7152fa386d1/README.md)). That timing claim is unverified, and **no measured Expert acceptance rate exists anywhere in the research**.

The best indirect evidence is Studholme. Only 5.4% of standard-rule Expert games were won without guessing, and those games had no guaranteed opening ([Studholme Fig. 6](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf)). A 3×3 opening and a perfect solver should raise acceptance, so tens of attempts per Expert board is a plausible order of magnitude. That is an inference that needs a benchmark.

At high density rejection collapses. Explainable Minesweeper needed about **250,000 attempts per 9×9/33 board** ([sublevelgames](https://sublevelgames.github.io/blogs/2025-07-06-making-explainable-minesweeper/)). This fits the measured phase transition: the fraction of mines that inference can flag falls to zero around **0.2 < ρ < 0.3** ([Dempsey & Guinn](https://arxiv.org/pdf/2008.04116)). Classic Expert (20.6%) sits at the low edge of that band.

**The verdict for Runesweeper.** Use rejection with an exact solver as the primary generator. It is principled, has no artifacts, and its failure mode is a redraw. A Rust reference implementation already exists in mindsweeper. Keep count-preserving repair, or Tatham-style perturbation, only as a labelled fallback for dense custom boards.

## Max-guess generation has no published algorithm, but its parts exist

### What exists

No paper or repository maximises forced guesses during play. The working "evil" games split into two designs.

**Lazy adversaries keep no board.** Kaboom asks MiniSat, on every click, whether the cell *can* be a mine and *can* be safe. Its rule is "cruel, but fair": "If there are any safe squares on the board, you are punished for guessing and that square will contain a mine. Otherwise, guessing is allowed and that square will be empty" ([pwmarcz.pl](https://pwmarcz.pl/blog/kaboom/)).

- gazpachoking's z3 version makes the rule a dial: NICE / FAIR / NORMAL / CRUEL ([repo](https://github.com/gazpachoking/minesweeper)).
- Pure Skill Minesweeper's author names the remaining unfairness: someone who takes an inevitable 50/50 early "will always lose" ([andrewt](https://github.andrewt.net/mines/)).
- Evil Minesweeper's adversary is still unimplemented ([itch.io](https://lightbulbmeow.itch.io/evil-minesweeper)).

These adversaries never make a player lose to a *forced* guess, so they are not "max-guess" at all.

**Static evil boards come from inverted rejection.** Evil Mineswifter "secretly runs an advanced Minesweeper AI to see if the board can be solved without guessing. If it solves the board, it randomly generates another board". Every board has at least one 50/50, and its daily challenges contain "exactly one 50/50 guess" ([App Store](https://apps.apple.com/us/app/id1523249883)). Inverted rejection guarantees at least one guess but cannot steer *how many*.

### Forced-guess structures and how to measure them

The structures to create are well defined:

- **True 2-tile 50/50s**, including arbitrarily extended chains.
- **Enclosed 2×2 boxes** with two mines.
- **Pseudo-50/50s.**
- **Dead tiles.**

Source: ([JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/README.md)). Scar gives the physical reason they exist: "There are two things in the game that don't give information: Mines, and the edges of the board". He also notes that some corner patterns are 50/50s only if the mine count cannot break them ([Scar](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)).

Measurable targets follow from the sources:

- **G**: forced guess points on a fixed solver's path.
- **Luck bits**: −log₂ of the product of the chosen guesses' survival probabilities. Each true 50/50 adds one bit.
- **Perfect-play win probability**: at most 2⁻ᵏ for k independent true 50/50s.
- **Deduction difficulty**: Cicvárek's "rank", the number of variables assumed in the cheapest step ([Cicvárek 2017](https://minesweepergame.com/math/algorithms-for-minesweeper-game-grid-generation-2017.pdf)), or Dempsey & Guinn's core size.

Deduction difficulty is what separates "guessy" boards from "hard but fair" ones.

### A proposed design: invert Tatham's loop

The nearest working templates are two. Tatham's loop is a heuristic local search driven by perturbation count. hellpig's 2026 evolutionary generator maximises the *deductive* difficulty of all-clues-visible puzzles, scored by SAT propagation counts, and its author suggests evolving the mine layout itself ([hellpig](https://github.com/hellpig/minesweeper-puzzle-generator)).

The proposed max-guess generator **[design synthesis, not from a source]** works like this:

1. Fix the first click. Start from a uniform layout with the 3×3 around the click kept mine-free.
2. Play the board with the reference solver, using an "oracle-continue" rule. At each stall, record whether the exact engine proves no safe cell, record the best guess's safety, and whether it is a true 50/50. Then reveal the true contents of that best guess and continue. This makes G and the luck bits deterministic functions of (board, click, solver).
3. Propose a move. Swap a mine with a non-mine, biased toward the constraint set behind the last deduction, so that the deduction disappears.
4. Accept the move if the objective improves, or with simulated-annealing probability.
5. Penalise local density variance, or cap the swap count, to avoid Tatham's packed-corner look.
6. Re-verify the final board from scratch with the exact engine.

Tatham's experience gives three warnings for this loop:

- whole-region moves can cycle;
- progress must count *newly* opened squares;
- a perturbation can silently break earlier structure, so the final re-solve is mandatory.

A small fixed catalogue of trap templates would recreate the "hidden clue" problem Tatham warns about. Templates may seed the search, but they must not define its output.

### The same search makes solver load tests

Max-guess boards double as solver load tests, because exact reasoning is hardest exactly where guessing is common:

- Mean core size "is strongly peaked in the vicinity of the phase transition" ([Dempsey & Guinn](https://arxiv.org/pdf/2008.04116)).
- Kaboom's first, backtracking engine hit "18 million possible mine arrangements … 12 gigabytes … half a minute" per reveal ([pwmarcz.pl](https://pwmarcz.pl/blog/kaboom/)).
- Tatham debugs generator hangs from fixed-seed strings such as `9x15n55X1Y1#2-94` ([commit a7c7826](https://github.com/samfundev/puzzles/commit/a7c7826bce5cbb9b9c337c11b9b7f8b278e76fba)).

Pointed at different objectives, the same local search yields a load-test suite:

- one giant connected frontier at density 0.2–0.3;
- many small components over a large interior, which stresses the convolution step;
- late-game near-ties that only the mine count resolves, which stresses capped global steps like Tatham's 10-set limit;
- chains of extended 50/50s;
- positions just above the BFDA threshold;
- fixed-seed regression boards.

Two further practices keep the engines honest. **Differential testing** runs two independent exact engines on the same position, such as component enumeration and a SAT or brute-force check on small boards, and asserts identical forced sets and counts. hellpig validates its solvers this way. **Property tests** like mindsweeper's "win all games" cover the generator invariants ([mindsweeper](https://github.com/AlexBuz/mindsweeper/blob/b804faa439cf0c210289e7045f3cb7152fa386d1/src/server/local.rs#L335-L401)).

## Good hints name the fewest numbers that force a cell

### Prior art

Most Minesweeper helpers show *what*, not *why*. JSMinesweeper shades cells and prints messages such as "Tile X is an unavoidable 50/50 guess" or "The best guess is off the edge", with tooltips like "NN.NN% safe" and "NN.NN% 2nd safety" ([solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/c30bd65ba7de859889ea76dccc1d227bed6e51d3/Minesweeper/client/solver_main.js)). Tatham's Mines has no hint, only Solve ([Mines manual](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/doc/mines.html)).

Explained hints have precedent. **MySweeper's "Clues"** mark only cells "an expert Minesweeper player could reasonably determine" using four documented rules: satisfied, saturated, subset and difference. Its help is graded: start with a count of findable cells in the title bar, then combined markers, then split safe/mine markers. It also tells the player: "attempt to figure out how each clue was determined" ([MySweeper Clues](https://sudleyplace.com/mysweeper/clues.htm); [Rules](https://sudleyplace.com/mysweeper/rules.htm)). **Explainable Minesweeper** uses three tiers: global, single-number and two-number "common area" patterns. It deliberately excludes 3+-number patterns "as they're too complex" ([sublevelgames](https://sublevelgames.github.io/blogs/2025-07-06-making-explainable-minesweeper/)).

### Lessons from Sudoku tools and explanation research

Sudoku tools are the mature model. **HoDoKu** offers three flavours: "Vague hint" (the step's name), "Concrete hint" (more detail), and "Show next step" ([HoDoKu manual](https://hodoku.sourceforge.net/en/docs_play.php)). **SudokuWiki** added Hints Mode in 2025 (strategy only, pattern only or eliminations only), and re-ordered its techniques for "humans entering notes on a pen and paper puzzle" ([What's New](https://www.sudokuwiki.org/Whats_New)).

Explanation research makes "simplest next step" precise. Bogaerts, Gamba and Guns minimise **f = basecost(S) + |E| + 5·|S|**, with a base cost of 100 per clue used. The design rests on the observation that "previously derived facts are easier to use than clues" ([arXiv 2006.06343](https://arxiv.org/pdf/2006.06343)). In Minesweeper terms, a one-number deduction beats a two-number one, which beats a chain, which beats the global count. Tatham's solver order and the minesweeper.online pattern tiers agree. Near the critical density, k-limited search "fail[s] to find a significant proportion of inferences" ([Dempsey & Guinn](https://arxiv.org/pdf/2008.04116)). A hint engine must therefore check exactly before it claims "nothing can be proven".

The HCI evidence is sobering. In Wauck et al., "no player experience metrics … were associated with the presence of hints", and 81% of players rated the hints 1–2 out of 5. Players given on-demand hints, however, "saw hints less often and had higher levels of the Engagement factor" ([Wauck et al. IUI 2017](https://www.sift.net/sites/default/files/publications/wauck_iui2017hints.pdf)). Surveyed players want hints that are "concise, accurate, implicit, text-based, and without revealing the whole game flow" ([He et al.](https://faculty.washington.edu/weicaics/paper/papers/HaoHZC2022.pdf)). Hints should be on demand and staged, and no one should expect them to make the game more fun on their own. No study has tested a nudge-to-reason ladder in Minesweeper.

## Conclusion

The research changes the framing from three generators and a hint feature to **one exact engine with several consumers**. The engine is a frontier-component probability engine with global mine-count weighting, a tiered local-deduction layer that records *which numbers* forced each cell, 50/50 and dead-tile detection, and an optional small-endgame tree search. Its consumers are:

- Guess-free mode uses it as an accept/reject oracle.
- Max-guess mode uses it as an objective function and verifier.
- The hint helper uses it as an explanation source.
- A benchmark bot plays with it.
- A load-test suite stresses it.
- A post-game panel can report community metrics alongside it: 3BV, 3BV/s and IOE, plus guesses forced versus needless and a pluck-style luck score.

Building it once, testing it differentially, and keeping its definitions explicit matters more than any single mode. The definitions to pin down are what "forced" means, whether the global count is used, and what the first-click rule is.

Two less obvious consequences follow.

**Every mode leaks information through its generator.** OG leaks through XP relocation: the top-left odds double. Guess-free leaks through "NG logic", which is valid only if the player's notion of solvable matches the generator's solver. Max-guess leaks through whatever bias its search has. The hint helper therefore has to decide which model its percentages describe.

**Win rate is the wrong headline for the new modes.** Guess-free boards should be graded by the hardest deduction tier they need, as minesweeper.online does. Max-guess boards should be graded by forced-guess count and luck bits.

## Recommendations for Runesweeper

### A shared solver core

Build one `solver` module, with no drawing code, that `game.rs`, the generators, the hint helper and the tests all use. It has five layers:

1. **Constraint builder.** Each revealed number becomes (hidden-neighbour set, remaining count). The remaining count uses the true number of proven mines, never the player's flags.
2. **Tiered local deduction.** The single-number rules, then pairs (subset/difference), then small groups of k ≤ 4–5 overlapping numbers enumerated exhaustively. Every forced cell records its minimal witness set.
3. **Exact engine.** Boxes, a frontier-state merge, convolution across components, and C(interior, minesLeft − m) weighting. Use `num-bigint`, or log-domain f64 with exact zero/total checks.
4. **Detection.** 50/50s, pseudo-50/50s and dead tiles, in the JSMinesweeper style.
5. **Guess ranking and endgame search.** Rank guesses by safety, then two-step safety within 0.1 of the best. Optionally add BFDA below about 1,000 solutions.

**Testing.** Run differential tests against a brute-force enumerator on boards up to about 6×6. Add property tests: every Guess-free board is won with zero guesses, and every reported Max-guess stall has no provably safe cell.

### The three generation modes

| | OG | Guess-free | Max-guess |
|---|---|---|---|
| When mines are laid | At New Game, as now (`Game::new` → `lay_mines`) | On the first click (requires a "no mines yet" `Ready` state) | On the first click, or ahead of time with a fixed start cell |
| First click | XP relocation within (W−1)×(H−1), "no move" case kept | 3×3 around the click mine-free (a guaranteed opening) | 3×3 mine-free (recommended) |
| Algorithm | Unchanged: MSVC `rand()`, column then row, rejection of occupied cells | Rejection sampling: uniform layout → play with the core solver (local tiers, then exact with global count) → accept on a full solve | Local search / simulated annealing from a uniform start, maximising forced guesses (true 50/50s weighted highest), with a density-variance penalty, then exact re-verification |
| Distribution | XP's, including the top-left odds doubling | Exactly uniform over boards the chosen solver can finish | Deliberately biased; no uniformity claim |
| Failure handling | None needed | Attempt and time budget; then a labelled count-preserving repair fallback, or refuse, for dense custom boards | Anytime: return the best board found within the time budget; report its guess count |
| Reproducibility | Session LCG stream | (seed, click) reproduces the board | (seed, click, budget) reproduces the board |
| Stats | Existing best times | Separate best times | Separate best times; show the board's forced-guess count |

**OG.** Leave it alone. Its value is fidelity, and [docs/research/minesweeper-generation.md](https://github.com/nicognaW/runesweeper/blob/main/docs/research/minesweeper-generation.md) already documents how it matches XP. Two additions are worthwhile. Give the solver the relocation weighting (the 1 + r(L) mixture above), so that hints and benchmarks in OG are exact. Document that a corner-start bot benchmark in OG is **not** the same rule as Tu et al.'s.

**Guess-free.** Start with the perfect-logician solver, including the global mine count. It accepts the most boards, and it has a Rust precedent in mindsweeper. Also record the hardest tier each accepted board needed, using the minesweeper.online labels Easy / Medium / Hard / Evil. That allows a later difficulty filter without changing the generator.

Before committing, measure three things at 9×9/10, 16×16/40 and 30×16/99: acceptance rate, p50/p99 latency on native and WASM, and distribution checks. The distribution checks are a mine-density heatmap and a 3BV histogram, compared against unconditioned uniform boards. No published numbers exist for any of these.

**Max-guess.** Ship a static board, not a lazy adversary. A static board stays honest, replayable from a seed and explainable after the game. A Kaboom-style adversary never makes forced guesses fail, which defeats the mode's point.

Use **true 50/50s** as the main currency, because hidden 70/30 guesses feel unfair. Show the count up front, in the spirit of Evil Mineswifter's "exactly one 50/50" dailies. Run generation off the UI thread: a background thread on desktop, a Web Worker or frame-sliced search on WASM. Ship the same search, with alternative objectives, as a `cargo test`/bench load-test suite with fixed-seed regression strings.

### The hint helper (OG and Max-guess)

**Disclosure.** One on-demand Hint action. Each press goes one level deeper, and any move resets it to level 1:

1. **Nudge.** "There's a safe cell you can find by logic" or "No cell can be proven right now", plus the technique name.
2. **Region.** Highlight the numbers in the minimal witness set.
3. **Cell.** Highlight the forced cell(s) and the action.
4. **Reason.** The full sentence, with an Apply button. The hint never plays the move unasked.

**Choosing the deduction.** Use the cheapest one, by Bogaerts-style cost: fewest numbers, then fewest cells, then reuse of already-proven cells, then safe before mine, then nearest the last click. Before falling to the guess tier, always confirm with the exact engine. A wrong player flag that contradicts proven facts gets the first hint ("This flag can't be right").

**Wording.** Use the players' own vocabulary, avoid coordinates, and use at most two numbers per guess explanation:

| Tier | Example reason text |
|---|---|
| One number, satisfied | "This 1 already touches its mine. Its other hidden neighbours are safe." |
| One number, saturated (with reduction) | "This 4 touches 2 known mines and needs 2 more. Only 2 hidden cells are left next to it, so both are mines." |
| Two numbers (1-1-X / 1-2-X) | "Every hidden cell next to this 1 is also next to that 1. The mine is in the shared cells, so the cells only the second 1 touches are safe." |
| Chain (3+ numbers) | One line per step: "First, … so this cell is a mine. Then this 2 is satisfied, so …" |
| Global count | "Only 2 mines remain, and these cells must hold them. Every other hidden cell is safe." |
| True 50/50 | "These two cells are a true 50/50. Nothing can ever tell them apart, so guess now rather than later." |
| Safest guess / progress / off-edge | "No cell can be proven. This one is about 85% safe; the next best is 78%." / "…and if it's safe it will likely open a certain move." / "Cells away from the numbers are each about X% safe." |
| Dead cell warning | "Don't pick this one: whatever number it shows, you'd have to guess again." |

**Mode differences.** In OG, the guess tier uses the relocation-weighted posterior. In Max-guess, the guess tier is the common case. Say explicitly when a stall is one of the board's announced forced guesses. Show uniform-model percentages, labelled as such, because the generator's bias is not modelled.

**Stats.** Mark any hinted game as assisted, and keep it out of best times.

## Open design questions for the grilling session

1. **Guess-free solver strength.** Perfect logician with the global mine count, or a capped human tier such as "no Evil patterns"? Should players choose a difficulty grade, and should the global count be allowed when Runesweeper's counter shows mines minus flags rather than the true number of mines left?
2. **Guess-free first click.** Is a guaranteed 3×3 opening acceptable, given that every surveyed generator uses one but it departs from XP's feel? Or should Guess-free keep XP's "safe but maybe a number" start, which lowers acceptance?
3. **Lazy generation.** Is it acceptable to change `Game` so that Guess-free and Max-guess fields have no mines until the first click? How should the face, the counter, F2 and "next field continues the session RNG stream" behave when a board takes measurable time to generate?
4. **RNG.** Should the new modes keep the MSVC LCG stream, with its 15-bit outputs and variable draw counts per board, or use a separate, better PRNG with a shareable (seed, click) board code in the style of Tatham's descriptions?
5. **Dense custom boards in Guess-free.** Refuse, cap the density, or fall back to a labelled repair that is biased toward edges? What attempt and time budget triggers the fallback?
6. **What Max-guess maximises.** The count of forced stalls, the total luck bits, or exactly k true 50/50s as a user-selectable dial? Are only true 50/50s allowed, or also clearly labelled lopsided guesses?
7. **Max-guess fairness.** Should the forced-guess count be announced before play? Is Undo after a forced-guess loss allowed, as Evil Mineswifter offers? Should there be a post-mortem that proves a loss was forced?
8. **Max-guess look.** How far may the search drift from a uniform-looking board, in density variance, swap cap or packed-edge tolerance, before it "looks generated"? Is any template seeding acceptable, given Tatham's hidden-clue warning?
9. **Max-guess latency and start cell.** What generation budget is acceptable on WASM? Should Max-guess use a provided start cell, like minesweeper.online's NG mode, so boards can be pre-generated in the background?
10. **Hint availability.** Should hints be available in Guess-free mode too, even though the brief scopes them to OG and Max-guess? Should there be a per-game limit or a time penalty, as in Mineswifter's +10 s for mistakes?
11. **Hint and stats.** Does any hint mark a game as assisted and exclude it from the three XP best-time slots? Does each mode get separate best-time tables?
12. **Hint probability model.** In OG, does the hint use the exact relocation-weighted posterior, or plain uniform probabilities for simplicity? In Max-guess, should it show uniform-model percentages that ignore generator bias, and say so?
13. **Hint and player flags.** Does the hint reason from the solver's knowledge, ignoring player flags, and actively call out wrong flags? Or does it trust flags as MySweeper does?
14. **Guess advice.** Does the hint advise taking 50/50s immediately, which maximises win rate and saves time, or leave timing to the player?
15. **Technique names.** Does the hint teach named patterns (1-2-X, reduction, holes) or use plain language only? Should hint usage feed a "techniques you've learned" view?
16. **Post-game analysis.** Is a post-game panel in scope: 3BV, 3BV/s and IOE, forced versus needless guesses, a pluck-style luck score? If so, which ZiNi variant, if any?
17. **Benchmark bot.** Is the solver bot a test-only artefact, or a shipped demo/autoplay? What win-rate target and rule statement should the README publish?
18. **Dependencies.** Write the engine from scratch, or reuse ms-toollib or mindsweeper code (licences unchecked)? Which of `num-bigint` and log-domain arithmetic is acceptable on WASM?
19. **Naming and glossary.** Are "OG", "Guess-free" and "Max-guess" the user-facing names? Which terms (forced guess, true 50/50, pseudo-50/50, dead cell, witness, frontier) go into a new `GLOSSARY.md`? And should the generation-mode decision be recorded as the first ADR in `docs/adr/`?
