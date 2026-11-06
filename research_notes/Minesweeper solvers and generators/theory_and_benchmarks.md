# Minesweeper solving: theory, published solvers, and benchmarking

Method note: the first version of this note was written while arxiv.org, dash.harvard.edu, minesweepergame.com, inria.hal.science, cdn.aaai.org and www.cs.toronto.edu were blocked, and its figures from those hosts were tagged "(snippet)". On 2026-11-06 the primary PDFs were downloaded and read in full, and every snippet figure was re-checked against the tables. Citations now give the PDF page (and the printed page where it differs), or the table number. Statements changed by that check are marked **[corrected after primary check]**. What was confirmed, what was corrected and what is still unverified is listed in the "Verification log" at the end.

Access notes for the re-check:
- inria.hal.science returns a bot-challenge page to scripted clients. So the HAL documents for Buffet et al. (hal-00750577v2) and Legendre et al. (hal-00723550) were read from the HAL-stamped copies that minesweepergame.com hosts. Both copies carry the HAL cover page.
- Springer (link.springer.com) also returned a client challenge, so neither Kaye 2000 nor Scott et al. 2011 could be read. Their bibliographic data were checked against Crossref, and their content against later primary papers that restate and analyse them.
- www.mdpi.com and web.archive.org did not resolve from this session.

## Complexity: what is known and what it implies for exact solvers

### Takeaway
Deciding whether a board is consistent is NP-complete (Kaye 2000). Deciding whether a cell can be inferred safe is coNP-complete. Scott, Stege and van Rooij posed this inference problem in 2011, but a 2024 paper shows their hardness proof is flawed and supplies a corrected one **[corrected after primary check]**. Counting consistent layouts is #P-complete, first proved by Nakov & Wei (2003) and independently by Pedersen (2004) **[corrected after primary check]**. Playing to maximize the win probability is PP-hard, and PSPACE-complete when the target probability may be exponentially small (de Bondt 2012). Move-limited play is PSPACE-complete even for p = 1 (Gaspers et al., AAAI-18). In practice, exact solvers enumerate per frontier component, and this works on standard boards. Fully optimal play (game-tree search) is affordable only in small endgames.

### Cited Findings
- **Kaye 2000.** "Minesweeper is NP-complete", *The Mathematical Intelligencer* 22(2), 9–15, March 2000 — [Crossref record, DOI 10.1007/BF03025367](https://api.crossref.org/works/10.1007/BF03025367)
  - The paper defines the Minesweeper (consistency) problem: does a partially revealed board admit some mine placement? It proves NP-completeness by building Boolean-circuit gadgets (wires, AND/OR/NOT) on the grid.
  - This content is restated in two later primary sources. de Bondt adds that Kaye's construction "ignores the fact that the total number of mines is also known" — [de Bondt, arXiv 1204.4659, p. 2](https://arxiv.org/pdf/1204.4659). Nakov & Wei describe the AND/NOT gate construction — [Nakov & Wei 2003, PDF](https://minesweepergame.com/math/minesweeper-minesweeper-2003.pdf).
  - Kaye's paper itself was not readable here (the Springer client challenge). The minesweepergame.com list says the result was first posted on Kaye's website in 1998 — [minesweepergame.com math papers](https://minesweepergame.com/math-papers.php).
- **Scott, Stege, van Rooij 2011.** "Minesweeper May Not Be NP-Complete but Is Hard Nonetheless", *The Mathematical Intelligencer* 33(4), 5–17, December 2011 — [Crossref record, DOI 10.1007/s00283-011-9256-x](https://api.crossref.org/works/10.1007/s00283-011-9256-x)
  - The paper poses **Minesweeper Inference**: given a consistent partial board, can some cell be deduced? It claims the problem is coNP-complete.
  - **[corrected after primary check]** Hendrickson & Tockman (MIT Hardness Group, arXiv 2404.14519; FUN 2024, LIPIcs vol. 291, paper 25) state that the 2011 coNP-hardness proof "is incorrect". Its OR gate admits an unintended inference when the two inputs cannot both be true. They say Thieme & Basten's 2022 proof "suffers from the same issue", and they give a fixed proof that inference is coNP-complete — [arXiv 2404.14519, abstract and pp. 1, 3–4](https://arxiv.org/pdf/2404.14519); [Dagstuhl FUN 2024 record](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FUN.2024.25)
  - Their definition differs from the 2011 one in two ways. Deducing a mine does not count as an inference, and no known mines are given in the input. They state that their results hold under either definition — [arXiv 2404.14519, p. 3, Def. 2.4](https://arxiv.org/pdf/2404.14519)
  - They also introduce **solvability**: given the full state including the hidden mines, can the player win by safe clicks alone? They prove it coNP-complete "even after a single click" — [arXiv 2404.14519, abstract, p. 1](https://arxiv.org/pdf/2404.14519)
  - Still true: consistency is not the problem a player faces, because a real board is always consistent. Inference is the right question.
- **Becerra 2015** summarizes the inference result as "co-NP-complete", following Scott et al. — [Becerra 2015, PDF pp. 13–14 (thesis pp. 8–9)](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf)
- **Counting.** **[corrected after primary check]**
  - Nakov & Wei, "MINESWEEPER, #MINESWEEPER" (unpublished, UC Berkeley, 14 May 2003) define #MINESWEEPER and prove it #P-complete. They also model the game as an MDP and solve it exactly for boards up to 4×4 — [Nakov & Wei 2003, abstract p. 1](https://minesweepergame.com/math/minesweeper-minesweeper-2003.pdf). The earlier note said this paper "was not found". It is now located.
  - Pedersen's 2004 Warwick thesis independently proves "#Consistency is #P-complete" by a parsimonious reduction from #SAT — [Pedersen 2004, PDF pp. 32–33](https://minesweepergame.com/math/the-complexity-of-minesweeper-and-strategies-for-game-playing-2004.pdf).
  - Pedersen also claims that "Solution" (does the configuration have a unique solution?) and "Safety" (is a given move safe?) are **DP-complete** — [Pedersen 2004, abstract PDF p. 3; theorems PDF pp. 34, 36](https://minesweepergame.com/math/the-complexity-of-minesweeper-and-strategies-for-game-playing-2004.pdf). This sits awkwardly with the later coNP-completeness of inference. Inference is in coNP, so DP-hardness of a single-move safety question would imply NP ⊆ coNP unless the problem definitions differ. Pedersen's DP claims have not been examined in the later literature found here, so treat them with caution.
  - Golan attributes #P-completeness of counting to Nakov & Wei (his ref. [11]) — [Golan, "Minesweeper on graphs", p. 2](https://minesweepergame.com/math/minesweeper-on-graphs-2011.pdf).
- **Golan 2011.** "Minesweeper on graphs", *Applied Mathematics and Computation* 217(14), 6616–6623 — [minesweepergame.com list](https://minesweepergame.com/math-papers.php). Theorems, from the preprint [PDF](https://minesweepergame.com/math/minesweeper-on-graphs-2011.pdf):
  - Theorem 2.1 (p. 3): consistency on trees in linear time.
  - Theorem 2.2 (p. 3): counting on trees in O(n² log n).
  - Theorem 2.3 (p. 3): *constrained* counting on trees, where the total mine count is given, in O(n³ log n).
  - Theorem 2.4 (p. 4): constrained counting on graphs of bounded treewidth in polynomial time. Counting for treewidth k runs in O(n^{k+2} log n), and constrained counting in O(n^{k+3} log n).
- **de Bondt 2012.** "The computational complexity of Minesweeper", arXiv 1204.4659v1 (20 Apr 2012), Radboud University Nijmegen; unpublished per the minesweepergame.com list — [arXiv 1204.4659](https://arxiv.org/pdf/1204.4659)
  - Abstract (p. 1): Minesweeper "is PP-hard, when the object is to locate all mines with the highest probability. When the probability of locating all mines may be infinitesimal, the Minesweeper game is even PSPACE-complete."
  - In the construction, the circuit is revealed in polynomial time after one initial guess that "has 99 percent probability of success". The number of mines can be computed in polynomial time, so a mine counter adds no information.
  - Theorem 2 (p. 11): "Minesweeper is PP-hard". It is proved by reduction from weak MAJSAT (Lemma 1, p. 11).
  - Theorem 3 (p. 12): "Minesweeper is PSPACE-complete when the probability of revealing all mines may be infinitesimal". It is proved by reduction from QBF or "weak nonalternating SSAT".
  - **Formal meaning of "infinitesimal"**, which the earlier note listed as a gap: in the construction, the win probability "will lie between [3/4]·2^−e and 2^−e inclusive", where e is the number of existentially guessed variables. This is exponentially small in the input size. (The fraction is extracted as "34 · 2−e" in the text layer, presumably ¾.)
  - The paper also proves consistency NP-complete on hexagonal and triangular grids, not only square ones (abstract, p. 1).
- **Gaspers, Rümmele, Saffidine, Tran, "Minesweeper with Limited Moves"** (AAAI-18) **[corrected after primary check: attribution now confirmed]**. The question is: given a partially revealed board, k clicks and a target probability p, can the player win with probability at least p? It is PSPACE-complete, "and it remains PSPACE-complete when p is a constant, in particular when p = 1". With k = 0 it is PP-complete in general and co-NP-complete for constant p. Their Table 1 credits two entries to de Bondt — [Gaspers et al. 2018, abstract and Table 1, p. 1](https://minesweepergame.com/math/minesweeper-with-limited-moves-2018.pdf)
- **Recent theory**:
  - Hendrickson & Tockman 2024, above.
  - Louf, "Phase transition for Minesweeper" (arXiv 2506.01634, 2 June 2025), proves a *coarse* phase transition. "Above a certain critical mine density, the game becomes unsolvable with high probability, whereas below the critical mine density it can be solved with a linear time algorithm." — [arXiv 2506.01634, abstract p. 1](https://arxiv.org/pdf/2506.01634)
  - An earlier unpublished Princeton paper, Dempsey & Guinn (2020), reports a phase transition at which hardness peaks — [minesweepergame.com list](https://minesweepergame.com/math-papers.php)
  - Mossel, "The Minesweeper Game: Percolation and Complexity" (*Combinatorics, Probability and Computing* 11(5), 487–499, 2002) — [CUP record](https://www.cambridge.org/core/journals/combinatorics-probability-and-computing/article/minesweeper-game-percolation-and-complexity/C96A8C5AF6E7FC75C389483020B4DCA3)
- In practice an exact probability engine is fine on standard boards. David Hill's JSMinesweeper says its exact engine's cost "grows exponentially", but "50x50/500 should be fine, 100x100/2500 might struggle". — [DavidNHill/JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Studholme, 600,000 games: "the largest problem solved had 69 variables, 30 constraints (39 degrees of freedom), and 1,620,880 solutions". More than 99% of constraint sets had fewer than 1,000 solutions — [Studholme, p. 14](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf)
- Practical caps on game-tree search:
  - JSMinesweeper switches to brute-force endgame analysis only when "the number of solutions remaining is 750 or less".
  - Hill's Java solver runs "brute force deep analysis", a game tree that maximizes win probability, when there are fewer than **4,000** candidate solutions. One logged run had 1,008 solutions, 67,984 nodes, 28,432 cache entries and 11,453 cache hits, and took **253 ms**. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper); [DavidNHill/Minesweeper README](https://github.com/DavidNHill/Minesweeper)
  - Tu et al. switch to the exact OPTIMAL recursion only when the number of consistent situations S is at or below a criterion ("PSEQ-D256" uses 256). They say the full optimal procedure would need "roughly 10^19" function calls even on beginner — [Tu et al. 2017, p. 1003 (PDF p. 5), Algorithm 3](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)
  - Legendre et al. interrupt the enumeration after one second (p. 16) — [Legendre et al. 2012](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf)

### Inferences
- The worst-case results are about adversarial or constructed boards. On random boards at standard densities (about 12–21% mines), the frontier splits into small, weakly coupled components. Studholme's ">99% of constraint sets under 1,000 solutions" (p. 14) supports this directly. Exact enumeration per component, combined with the global mine count by combinatorial weighting, stays tractable.
- The real hardness for "optimal" play is the game tree: PP-hard, or PSPACE-complete. This is why every strong solver plays greedily or with a short look-ahead, and searches the full tree only below a few hundred to a few thousand remaining layouts.
- Golan's bounded-treewidth result suggests a principled exact method for large frontiers: dynamic programming over a tree decomposition of the frontier. It avoids the full product of component enumerations, and his constrained-counting variant handles the global mine count. No published Minesweeper solver benchmarking this approach was found.
- Anyone citing "inference is coNP-complete" should now cite Hendrickson & Tockman 2024 for the proof, and Scott et al. 2011 only for posing the problem.

### Gaps
- Kaye's own text, and his "Infinite versions of minesweeper are Turing complete", could not be read. web.mat.bham.ac.uk returns 404 and the Wayback Machine did not resolve. The Turing-completeness claim is confirmed only by secondary mentions: Nakov & Wei 2003, and the minesweepergame.com list ("unpublished, University of Birmingham; updated 2007").
- Scott et al. 2011 was not read directly (paywalled). Its definitions are known only through Hendrickson & Tockman's account.
- No later paper found here examines Pedersen's DP-completeness claims.

## Published solver approaches and their measured win rates

### Takeaway
Published solvers progressed in this order:
1. Single-point rules.
2. CSP enumeration (Studholme 2000).
3. CSP with tie-breaking and probe-selection heuristics (Legendre et al. 2012 "HCSP", Pedersen 2004, Becerra 2015).
4. UCT on top of HCSP (Buffet et al. 2012).
5. Tuned heuristic combinations plus exact endgame search (Tu et al. 2017, PSEQ and PSEQ-D256).
6. Exact probability plus look-ahead and endgame tree search (Hill, about 2019 onward).

The best peer-reviewed classic-rule Expert figure is Tu et al.'s 40.06(2)%. Liu et al. (2022, *Knowledge-Based Systems*) report 39.7% under a safe first click and 45.6% when the first click and its neighbours are guaranteed mine-free **[corrected after primary check: previously missing]**. Hill's unreviewed solvers claim about 40–41% classic and about 53–54% with a zero start.

Learning agents remain well below this on Expert **[corrected after primary check]**:
- the best CNN is 21.5% (safe first click, 2212.10446);
- the bandit agents are 4.1% (2007.12824);
- a hobby DDQN project reports 23.6%.

### Cited Findings
**Chronology of classic-rule results**

**Studholme (2000; web page dated 16 April 2001), University of Toronto, "Minesweeper as a Constraint Satisfaction Problem"** (CSC2512 course project; PGMS framework) — [paper PDF](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf); [project page](https://www.cs.toronto.edu/~cvs/minesweeper/)
- **[corrected after primary check]** The levels tested were beginner **8×8/10**, intermediate **15×13/40** (not 16×16) and expert 30×16/99 (p. 11).
- Each level was run in three game modes: standard (cannot lose on the first step), hard (can lose) and hinted (p. 11).
- Each data point is **100 sets of 100 games = 10,000 games**, with "standard error of the mean … 0.5 or less" (pp. 10–11).
- **[corrected after primary check]** The paper reports the win rates only as bar charts (Figs. 2–4, p. 12). The numbers **80.0%** (Beginner) and **33.9%** (Expert) do not appear in its text. They come from secondary tabulations:
  - [Tu et al. 2017, Table 5, p. 1005](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf);
  - [Liu et al. 2022, Table 2, p. 9](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf), which also gives intermediate 44.3% on 15×13/40;
  - [minesweepergame.com list](https://minesweepergame.com/math-papers.php).
- The Expert figure is consistent with Fig. 6 (p. 13): over 10,000 standard-rule Expert games, "Won (no guessing) 5.4%" plus "Won (guessing) 28.6%" gives 34.0%.
- CSPStrategy always opens in a fixed corner (p. 4). Later guesses prefer unconstrained corners, then edges (p. 7).
- Tu et al. summarize it as "tries to probe the block in the corner, winning 33.9 % of expert level deterministically" — [Tu et al., p. 999](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)

**Pedersen (2004), Warwick undergraduate thesis** — [PDF](https://minesweepergame.com/math/the-complexity-of-minesweeper-and-strategies-for-game-playing-2004.pdf). **[corrected after primary check]**
- The best strategy is **limited search with probability estimates**, not single-point. It wins **92.5%** Beginner, **67.7%** Intermediate and **25.0%** Expert (Table 5.3, PDF p. 66 / thesis p. 56).
- When the first probe is a zero, it wins 97.1%, 77.8% and 30.0%.
- The single-point strategy wins only **77.4%**, **29.1%** and **0.5%** (Table 5.1, PDF p. 48).
- Plain limited search wins 91.7%, 64.3% and 17.0% (PDF p. 58).
- **[corrected after primary check]** Pedersen's "beginner" board is **10×10 with 10 mines** (density 10%), not 8×8 or 9×9 (PDF p. 42). This resolves the earlier conflict: 92.5% sits on an easier board than Studholme's 8×8.
- Pedersen also mislabels Studholme's benchmark (80%/45%/34%) as being on 10×10/16×16/30×16, which is not what Studholme used.
- Protocol: **1,000 sets of 100 games** (100,000 per strategy), with per-set variance reported (PDF p. 42). The first move is always safe and always probes the top-left corner (PDF pp. 43–44).
- The minesweepergame.com list credits Pedersen with "77.8% on Intermediate … and 30.0% on Expert". These are the *first-probe-was-zero* conditional rates, not the overall rates.

**Legendre, Hollard, Buffet, Dutech, "MineSweeper: Where to Probe?"** (INRIA Research Report RR-8041, July 2012, 26 pp.; hal-00723550) — [PDF](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf). Report page numbers are given below; PDF page = report page + 3.
- It models the game as a POMDP and studies first-move and tie-breaking heuristics.
- **Does it report win rates? Yes [corrected after primary check].**
- Rules:
  - The minefield is generated after the first move, so the first click is safe (p. 6).
  - Every algorithm at a level plays "the same set of minefields … (using the same random seed)" (p. 17).
  - The enumeration times out at 1 s (p. 16).
- Grids (Table 2, p. 17): Beginner 9×9/10 (alternative 8×8/10), Intermediate 16×16/40 (alternative 13×15/40), Expert 16×30/99.
- Main results with a corner start, 10,000 games each (Table 10, p. 20; three-decimal values from Table 12, p. 21):

  | Strategy | Beginner 9×9 | Intermediate 16×16 | Expert 16×30 |
  |---|---|---|---|
  | cSimEnuLoAw | 0.914 | 0.775 | 0.373 |
  | cSimEnuLoCl | 0.902 | 0.756 | 0.376 |
  | cSimEnuLoClf | 0.902 | 0.756 | 0.375 |
  | cSimEnuAwLo | 0.911 | 0.753 | 0.291 |

- On the alternative grids (Table 11, p. 20): 8×8 Beginner 0.80–0.81, and 13×15 Intermediate 0.42–0.50.
- First move on 9×9 Beginner (Table 7, p. 19): corner 0.90–0.91, random 0.85–0.87, middle 0.82–0.85.
- No error bars are given.
- Tu et al. cite cSimEnuLoClf as 80% / 75.6% / 37.5% (Table 5, p. 1005). That mixes the 8×8 alternative Beginner with the 16×16 Intermediate.

**Buffet, Lee, Lin, Teytaud, "Optimistic Heuristics for MineSweeper"** (ICS 2012, Hualien, pp. 199–207; Springer *Advances in Intelligent Systems and Applications* vol. 20, 2013; DOI 10.1007/978-3-642-35452-6_22; hal-00750577v2) — [PDF](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)
- OH ("Optimistic Heuristics") is UCT that expands only the moves with minimum mine probability. It uses HCSP, Legendre et al.'s CSP with frontier tie-breaking, as the Monte-Carlo policy, and fixes the first move in a corner (PDF p. 4).
- Rule used: "one can not lose at the first move" (PDF p. 7).
- Table 1 (PDF p. 8) **[corrected after primary check: adds the 9×9 row, error bars, simulation budgets and the HCSP column]**:

  | Board | OH | HCSP |
  |---|---|---|
  | 8×8/10 | **80.2 ± 0.48%** | 79.9% |
  | 9×9/10 | **89.9 ± 0.3%** | 90.5% |
  | 16×16/40 | **74.4 ± 0.5%** | 76.4 ± 0.4% |
  | 16×30/99 | **38.7 ± 1.8%** | 38.1 ± 0.5% |

- OH uses 10,000 simulations per move, except on 16×16 and 16×30, which use **100** simulations per move. Teytaud's "10,000 UCT simulations per move" is therefore right only for the small boards.
- HCSP figures average 10⁵ games, except 16×30, which averages 10⁴. OH's game counts and the meaning of "±" (standard error or confidence interval) are not stated.
- OH is *below* HCSP on 9×9 and 16×16. Its Expert lead over HCSP (0.6 points) is well inside its ±1.8.
- The CSP-PGMS column of Table 1 (80% on 9×9, 45% on 16×16, 34% on Expert) reproduces Studholme-era figures.
- Teytaud's slides remain garbled and are superseded by Table 1 — [Teytaud slides](https://slideshare.net/teytaud/mines-ics2)

**TeddySweeper (Wu et al., 18th Game Programming Workshop, Japan, 2013)**
- Claimed rates: **91.78%** Beginner 9×9, **81.52%** Beginner 8×8, **77.84%** Intermediate, **38.84%** Expert.
- **[corrected after primary check]** The list describes the method as reducing "the computational complexity of a pure CSP approach", not as POMDP-based.
- The list adds that the claim "may be correct but depends on statistical error with previous records being 91.4%, 81%, 77.8% and 38.7%".
- Source: [minesweepergame.com list, now read directly](https://minesweepergame.com/math-papers.php). The paper itself was not retrieved (its link returns an HTML page).

**Becerra (2015), Harvard senior thesis "Algorithmic Approaches to Playing Minesweeper"** — [PDF (DASH-stamped copy)](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf); citable link nrs.harvard.edu/urn-3:HUL.InstRepos:14398552
- Boards: **9×9/10, 16×16/40, 16×30/99**.
- "Soft rules for first clicks": the solver cannot lose on the first click, but the first click is not guaranteed to be a zero (PDF p. 21 / thesis p. 16).
- Each algorithm plays **1,000 sets of 100 games**, 100,000 in total (PDF p. 20). The thesis reports the mean and the across-set standard deviation.
- Coupled-subsets CSP (CSCSP), Table 6.2 (PDF p. 39 / thesis p. 34): **91.25%** (SD 2.77), **75.94%** (SD 4.25), **32.90%** (SD 4.75). Mean guesses per game: 1.70, 2.39 and 4.73.
- Connected-components CSP (C3SP), Table 6.1: 90.85%, 74.18% and **25.96%**. The text gives 74.14% for Intermediate, so table and text disagree slightly.
- Single-point baselines (thesis p. 23): naive SP 64.95%, 30.79% and 0.51%; DSSP 68.19%, 35.92% and 0.89%.
- Conclusion (PDF p. 40): C3SP did "slightly worse, especially at higher difficulties, a result of it using simpler probabilistic heuristics."
- Studholme and Becerra both argue that corners are the best opener because a zero is most likely there — [Tu et al., p. 1000](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)

**Tu, Li, Chen, Zu, Gu, "Exploring Efficient Strategies for Minesweeper"** (AAAI-17 Workshop "What's Next for AI in Games?", WS-17-15, pp. 999–1005; affiliations Tsinghua, CAS, UC Berkeley, HKU) — [PDF](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf); [AAAI copy](https://cdn.aaai.org/ocs/ws/ws0294/15091-68459-1-PB.pdf); [HKU Scholars Hub](https://hub.hku.hk/handle/10722/245453)
- Rules (p. 1000):
  - The first probe is safe, with mines uniform over all other cells.
  - Levels are "8-8-T10 (beginner), 16-16-T40 (intermediate) and 30-16-T99 (expert)".
  - Zeros are not auto-expanded, which does not affect the win rate.
  - All strategies open in a corner (p. 1002).
- PSEQ, which filters by P (minimum mine probability), then S, E and Q in that order, reaches **81.627(8)%**, **78.122(8)%** and **39.616(5)%** (abstract p. 999; Table 5, p. 1005).
- The quasi-optimal **PSEQ-D256** reaches **81.79(2)%**, **78.22(3)%** and **40.06(2)%** (Table 5).
- **[corrected after primary check: game counts]** PSEQ used 10⁸ games on Beginner and Intermediate and 3.2×10⁸ on Expert. PSEQ-D256 used 10⁷ and 1.88×10⁷ (Table 5 footnotes).
- Uncertainty is given in last-digit notation, "estimated by 0.05 quantiles of β-distribution" (Table 2 note, p. 1002; §7, p. 1003).
- Other Expert rows in Table 4 (p. 1004), Expert only:
  - single filter P: 35.56(1)% over 5×10⁷ games;
  - single filter U: 39.14(9)%;
  - PS: 39.25(3)%;
  - PSE: 39.60(2)%.
- Corner first move against the best non-corner first move, 30-16-T99, naive "P" guessing: 35.56(1)% against 34.50(2)% (Table 2, p. 1002).
- Tu et al. claim PSEQ is "1.427% (3.722%, 0.916%) higher" than the state of the art (p. 1004). Their Intermediate baseline was OH's 74.4%. But Legendre et al.'s cSimEnuLoAw had already reached 77.5% and Becerra 75.94% (with a different first-click protocol). So the Intermediate margin over prior work is about 0.6 points, not 3.7 **[corrected after primary check]**.
- Authors: the task brief's "Tu… Wei" is wrong. The paper lists Jinzheng Tu, Tianhong Li, Shiteng Chen, Chong Zu and Zhaoquan Gu (p. 999).

**Liu, Huang, Gao, Khalid, Iida, "A solver of single-agent stochastic puzzle: A case study with Minesweeper"** (*Knowledge-Based Systems* 246, 108630, 2022; open access) **[corrected after primary check: missing from the earlier note]** — [PDF](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)
- The PAFG solver combines primary reasoning, advanced reasoning by Gauss–Jordan elimination, a first-action strategy and a guessing strategy.
- Safe first action, "PAR" variant, 10,000 games (Table 1, p. 8): **87.9%** on 9×9/10, **78.2%** on 16×16/40 and **39.7%** on 16×30/99.
- When the first click can be a mine, PAR wins 77%, 66.2% and 31.6%.
- The headline **96.4% / 86.3% / 45.6%** (PAFG, Table 2, p. 9; abstract p. 1) uses the **"safe neighborhood rule"**: the first cell and all its neighbours are mine-free, a guaranteed opening (p. 7).
- The same table lists PAFG at 82% on 8×8/10.
- A paper published in 2022 under a zero-start rule reports Expert 45.6% over 10,000 games (±≈1.0 point at 95%). The comparable safe-first-click Expert figure is 39.7%.

**mrgris "Minesweepr"** (a web solver, not a paper), standard rules (first click always safe), 100,000 games, "margin of error ≈0.15%" — [mrgris.com/projects/minesweepr](https://mrgris.com/projects/minesweepr/), now read directly:

| Board | Always safest cell | Corners first |
|---|---|---|
| 9×9/10 Beginner | **84.8%** | **91.4%** |
| 8×8/10 Beginner "classic" | 74.1% | 81.0% |
| 16×16/40 Intermediate | **69.2%** | **77.4%** |
| 16×30/99 Expert | **33.1%** | **37.8%** |

"Corners, then edges" adds about 0.1 point on each board.

**David N. Hill's solvers** (fetched directly in the first session):
- Java solver: **41%** on "classic expert (30x16/99 safe start in a corner)". **54.3%** on "modern expert (30x16/99 open start at (3,3))". The JavaScript version is "slightly weaker". A classic Expert win needs **3.3 guesses** on average. Guess candidates are tiles within 10% of the safest guess. One-move look-ahead blends "progress percentage" with "secondary safety". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Another of his READMEs gives **40%** for "traditional rules (30x16x99 safe but not always zero start)" and **53%** for "modern rules (30x16x99 zero start)". Neither page states the number of games or seeds. — [DavidNHill/Minesweeper README](https://github.com/DavidNHill/Minesweeper)
- A Minesweeper Fandom wiki credits it with "the highest winrate of any solver", **41.7%** on Expert. Not re-checked (wiki). — [Fandom JSMinesweeper](https://minesweeper.fandom.com/wiki/JSMinesweeper)

**Tang, Jiang, Hu, "A Minesweeper Solver Using Logic Inference, CSP and Sampling"** (arXiv 1810.03151, 2018, 3 pp., ShanghaiTech) — [arXiv 1810.03151](https://arxiv.org/pdf/1810.03151)
- Results appear only as a bar chart (Fig. 3, p. 3) with no stated board sizes, game counts or first-click rule. Read off the chart, the program wins about **52%** "easy", about **48%** "intermediate" and about **10%** "hard".
- These are well below every CSP result above, so the levels are probably not comparable.
- Each move is limited to 5 s (abstract).

**Sinha, Malviya, Nayak, "Fast constraint satisfaction problem and learning-based algorithm for solving Minesweeper"** (arXiv 2105.04120, 2021) — [arXiv 2105.04120](https://arxiv.org/pdf/2105.04120)
- On the standard beginner board (9×9/10), its versions reach 77.34–**92.02%** accuracy; the best is "version 5.0". The comparison row is OH 89.9 (Table 4, PDF p. 21).
- Other experiments vary board sizes of 5–30 and densities of 5–30%, with only 5–200 games per configuration (PDF pp. 16–17).
- No Expert figure is given. The earlier "beats average human players" snippet was not found in the PDF text.

**Learning-based solvers**
- **Lordeiro, Haddad, Cardoso, "Multi-Armed Bandits for Minesweeper: Profiting from Exploration-Exploitation Synergy"** (arXiv 2007.12824v2, 17 Jun 2021; published in *IEEE Transactions on Games*, 2021) — [arXiv 2007.12824](https://arxiv.org/pdf/2007.12824)
  - Boards: Windows 98 presets, Beginner 8×8/10, Intermediate 16×16/40, Expert 16×30/99. The first click is always safe (p. 2).
  - The agents use no heuristics and exceed "75% and 55%" on Beginner and Intermediate (p. 1).
  - **[corrected after primary check]** The best agent (greedy, transfer-trained) wins **76.96%** Beginner, **57.94%** Intermediate and **4.13%** Expert (p. 9). The earlier "about 4%" fragment was this Expert figure.
  - Liu et al.'s Table 2 lists the UCB agent at 76.4%, 44.5% and 0.62% — [Liu et al. 2022, p. 9](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf).
  - Training used 10⁶ games; the cross-scenario tests used 10⁴ games each (p. 5).
- **Sajjad, "Neural Network Learner for Minesweeper"** (Loughborough BSc report, May 2022; arXiv 2212.10446) — [arXiv 2212.10446](https://arxiv.org/pdf/2212.10446)
  - Boards: 9×9/10, 16×16/40, 16×30/99 (PDF p. 10).
  - **[corrected after primary check]** The 10.1% Expert figure belongs to one weak learner (cnnlearner3, zero first click), not to the best one.
  - Best CNN with a **safe** first click (Table 5.3, PDF p. 44): Beginner **88.4%**, Intermediate **68.8%**, Expert **21.5%**.
  - With a zero first click it reaches 91.8% Beginner and 72.9% Intermediate.
  - The project first used a zero first click, then switched to safe-first-click "for a fair comparison" (PDF p. 45).
  - Its CSP baseline is Becerra's own code, run for 100,000 games per level, giving 91.25% / 75.94% / 32.90%. The CNN was evaluated on at least 100,000 games per level (PDF p. 54).
  - **[corrected after primary check]** The "52.4% on Beginner when the first click may be a mine" figure is not in this paper. It is Castillo & Wrobel's 2003 "Mio" multirelational learner (IJCAI 2003), per the [minesweepergame.com list](https://minesweepergame.com/math-papers.php).
- **Mehta, "Reinforcement Learning For Constraint Satisfaction Game Agents"** (arXiv 2102.06019, 2021; Cupertino High School) — [arXiv 2102.06019](https://arxiv.org/pdf/2102.06019)
  - Architecture: a DQN on 16×16×2 input with 256 actions (PDF p. 10).
  - **[corrected after primary check]** The density classes are not standard Minesweeper densities: "low (density < 35%), medium (35% <= density < 70%) and high (density >= 70%)" on a 16×16 board (PDF p. 10).
  - Low and high density reach "close to 45%" wins, and medium density 15% (PDF p. 14).
  - These numbers say nothing about standard Beginner, Intermediate or Expert play.
- **AlexMGitHub/Minesweeper-DDQN** (hobby project, not peer-reviewed): Double DQN at **23.60%** on Expert over **25,000** games. Not re-checked in this session; fetched from GitHub in the first session. — [GitHub Minesweeper-DDQN](https://github.com/AlexMGitHub/Minesweeper-DDQN)
- **"Training a Minesweeper Agent Using a Convolutional Neural Network"** (MDPI *Applied Sciences* 15(5):2490, 2025): **93.3%** win rate (SD 0.77%), with CSP at 84–90% and humans at 87% ± 4.5% in its comparison. **Still unverified (snippet)**: www.mdpi.com does not resolve from this session, so the level, first-click rule and game count remain unknown. — [MDPI](https://www.mdpi.com/2076-3417/15/5/2490)
- **Li, Wang, Zhang (Georgia Tech), "Assessing Logical Puzzle Solving in Large Language Models: Insights from a Minesweeper Case Study"** (arXiv 2311.07387) **[corrected after primary check: results now retrieved]** — [arXiv 2311.07387](https://arxiv.org/pdf/2311.07387)
  - Boards: 5×5 with 4 mines, starting from a fixed click L(3,3) that reveals at least 10 cells.
  - GPT-3.5 variants solved **0 of 100** games; GPT-4 solved **0 of 10**, with a 70% failure rate.
  - A human on a GUI solved 89%.
  - GPT-4 produced "accurate and coherent logic" in 12–26% of selected reasoning chains (Table 2, PDF p. 6).
- A 2025 Springer chapter ("Intelligent Gameplay Strategies…: CSP and Decision-Making") and a 2025/26 "three-phase" approach (ACM DL) also exist. No numbers were retrieved; both are paywalled and not re-checked. — [Springer](https://link.springer.com/chapter/10.1007/978-981-96-9196-8_43); [ACM DL](https://dl.acm.org/doi/abs/10.1007/978-3-032-18477-1_2)
- Sheppard, "A simple Minesweeper algorithm" (2023) rephrases Minesweeper as entropy optimization. It reports no win-rate benchmark in its abstract or introduction — [PDF](https://minesweepergame.com/math/a-simple-minesweeper-algorithm-2023.pdf)

### Inferences
- **Superseded results**:
  - On Expert with a safe first click, Studholme's 33.9% (2000) was beaten by:
    - Legendre et al. (37.3–37.6%, 2012);
    - OH (38.7 ± 1.8%, 2012), which is statistically tied with HCSP's 38.1 ± 0.5%;
    - TeddySweeper (38.84%, 2013, unverified);
    - Tu et al. PSEQ (39.616%) and PSEQ-D256 (40.06%, 2017);
    - Liu et al. PAR (39.7%, 2022);
    - Hill (about 40–41%).
  - Becerra's 32.90% Expert (2015) was below the 2012 state of the art when published. It is often re-cited as "the CSP baseline", for example in arXiv 2212.10446.
  - Pedersen's 25.0% Expert (2004) is also superseded. His 92.5% "Beginner" is on 10×10/10 and is not comparable to the others.
- Learning-based agents have not approached exact-probability solvers on Expert. The best reviewed figures are 21.5% (CNN, safe first click) and 4.1% (bandits). The hobby DDQN reports 23.6%. All are against about 40%.
- Most of the gain from 2000 to 2017+ comes from four sources: (a) exact probabilities that use the global mine count, (b) where-to-guess heuristics (corners, frontier tie-breaking, information-related filters), (c) look-ahead, and (d) endgame tree search. Better deterministic inference contributed little, because it was essentially complete by 2000.

### Gaps
- TeddySweeper's paper was not retrieved, so its game counts and first-click rule are unknown.
- OH's game counts, and what its "±" denotes, are not stated in Buffet et al.
- No 2023–2026 peer-reviewed paper with a classic safe-first-click Expert result above Tu et al.'s 40.06% was found. The MDPI 2025 CNN paper could not be read.

## Best known win rates per level and rule, and the theoretical optimum

### Takeaway
Best reported Expert (30×16, 99 mines) rates are as follows.
- **Safe (not necessarily zero) first click:** about **40–41%**. The sources are Tu et al. PSEQ-D256 at 40.06(2)% (peer-reviewed workshop paper, 1.88×10⁷ games) and Hill at 40–41% (unreviewed, game count unstated).
- **Zero or opening first click:** about **45.6%** (Liu et al. 2022, peer-reviewed, 10,000 games) and **53–54.3%** (Hill, unreviewed) **[corrected after primary check: Liu et al. added]**.
- **Intermediate, safe first click:** the best is 78.22(3)% (Tu).
- **Beginner** depends on the board:
  - 8×8/10: about 81.6–81.8% (Tu);
  - 9×9/10: about 91.4% (Legendre et al. 0.914; mrgris 91.4%);
  - 9×9/10 with a zero start: 96.4% (Liu).

No proven optimum exists for any standard level.

### Cited Findings
- **Expert, safe first click:**
  - Tu et al. PSEQ **39.616(5)%** and PSEQ-D256 **40.06(2)%** — [Tu et al., Table 5, p. 1005](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)
  - Liu et al. PAR **39.7%** — [Liu et al., Table 1, p. 8](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)
  - Hill **41%** (corner start) and **40%** ("safe but not always zero") — [JSMinesweeper](https://github.com/DavidNHill/JSMinesweeper); [DavidNHill/Minesweeper](https://github.com/DavidNHill/Minesweeper)
- **Expert, zero or open first click:**
  - Liu et al. PAFG **45.6%** (first cell and its neighbours safe) — [Liu et al., Table 2, p. 9](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)
  - Hill **54.3%** (open start at (3,3)) and **53%** (zero start) — [JSMinesweeper](https://github.com/DavidNHill/JSMinesweeper); [DavidNHill/Minesweeper](https://github.com/DavidNHill/Minesweeper)
  - The 9-point gap between Liu and Hill under similar opening rules is unexplained. Hill's open start at (3,3) and Liu's neighbourhood-safe start may differ in detail, and Hill gives no game counts.
- **Intermediate (16×16/40):**
  - Tu PSEQ **78.122(8)%**, PSEQ-D256 **78.22(3)%** — [Tu et al., Table 5](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf)
  - Liu PAR 78.2% — [Liu et al., Table 1](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)
  - Legendre cSimEnuLoAw 0.775 — [Legendre et al., Table 12, p. 21](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf)
  - TeddySweeper **77.84%** (unverified) — [minesweepergame.com list](https://minesweepergame.com/math-papers.php)
  - mrgris with corners **77.4%** — [mrgris](https://mrgris.com/projects/minesweepr/)
  - With a zero start, Liu PAFG reaches 86.3%.
- **Beginner 8×8/10** (Windows 98 preset):
  - Tu **81.627(8)%**, PSEQ-D256 **81.79(2)%**
  - TeddySweeper **81.52%**
  - mrgris with corners **81.0%**
  - Legendre 0.80–0.81
  - OH **80.2 ± 0.48%** — [Buffet et al., Table 1](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)
- **Beginner 9×9/10:**
  - Legendre cSimEnuLoAw **0.914**
  - TeddySweeper **91.78%**
  - mrgris with corners **91.4%**
  - Becerra **91.25%**
  - HCSP 90.5%
  - OH 89.9 ± 0.3%
  - Liu PAR 87.9%
  - With a zero start, Liu PAFG reaches 96.4%.
- Lordeiro et al. state the presets as "beginner (8×8 board, 10 mines), intermediate (16×16 board, 40 mines) and expert (16×30 board, 99 mines)" from the classic Windows 98 version — [arXiv 2007.12824, p. 2](https://arxiv.org/pdf/2007.12824)
- **Unreviewed benchmark dataset** (Hugging Face "LKForge/minesweeper-solver-benchmark"; the card cites "LK Forge (2026)"): 1,000 seeded games per level give **95.5%** Beginner 9×9, **85.6%** Intermediate and **36.2%** Expert. No-guess games make up 78.5% (Beginner), 52.4% (Intermediate) and 3.8% (Expert).
  - **[corrected after primary check]** The card states the rule explicitly: "the first click is guaranteed safe (a single non-mine cell in the centre — not a guaranteed large opening)".
  - The card says best-known Expert solvers land at about 34–40% under that rule — [dataset card README](https://huggingface.co/datasets/LKForge/minesweeper-solver-benchmark/raw/main/README.md)
- A popular guide puts the "theoretical max" Expert rate at about 30–40%. This is not rigorous, and several solvers exceed it. Not re-checked. — [minesweeperblast.com](https://minesweeperblast.com/how-to-win-minesweeper/)
- **Upper bounds**:
  - Tu et al.'s Table 1 (p. 1001) gives a provable upper bound on survival over the first two moves only. With a corner first probe it is 0.9396 (8×8/10), 0.9381 (16×16/40) and 0.8970 (30×16/99), against 0.8983, 0.8963 and 0.8530 for a central first probe. This bound is far above the achieved rates.
  - Nakov & Wei compute exact optimal policies only up to 4×4 — [Nakov & Wei 2003](https://minesweepergame.com/math/minesweeper-minesweeper-2003.pdf)

### Inferences
- Tu's 8×8 Beginner (about 81.6%) matches TeddySweeper's 8×8 figure (81.52%), as inferred earlier. Tu's text now confirms the board as 8-8-T10. Studholme's 80.0% and OH's 80.2% are also on 8×8, both confirmed. Becerra, Legendre's main tables, mrgris and Liu use 9×9. Pedersen uses 10×10. **Comparing Beginner numbers across papers without checking the board is a common error.** It is made in Tu et al.'s own Table 5, which mixes 8×8 and 15×13 rows, and in Liu et al.'s Table 2, which mixes 8×8, 9×9 and 10×10.
- **[corrected after primary check]** The Hugging Face numbers (95.5% and 85.6%) are claimed under a safe single-cell centre first click, not under a zero start as the earlier inference guessed. That claim conflicts with every peer-reviewed safe-first-click 9×9 result (about 87.9–91.4%) and 16×16 result (about 75–78%). With 1,000 games, the 95% interval is about ±1.3 points at 95% and about ±3.0 points at 36%, so the gap on Beginner is far outside the noise. Possible explanations, all unverified:
  - an undocumented opening rule;
  - a bug in the win check;
  - optimistic selection of the reported run.

  Treat the source as unreliable.
- Optimum estimates: classic Expert optimal play is probably only slightly above 40–41%. Tu's PSEQ-D256 (40.06%), Hill's tree search (about 41%) and Liu's PAR (39.7%) converge from different methods. In Tu's Fig. 4 and Table 5, endgame search adds about 0.45 points on Expert over PSEQ. This is an inference, not a proven bound.
- The opening rule matters more than any solver improvement since 2000. A zero or neighbourhood-safe start adds about 6 points on Expert (Liu: 39.7% to 45.6%) to about 13 points (Hill: 41% to 54%).

### Gaps
- No rigorous upper bound or exact optimal win probability for any standard level was found. Exact optimal values exist only for tiny boards: Nakov & Wei up to 4×4, and the small boards in Buffet et al.'s Table 1.
- Hill's game counts and confidence intervals are not stated on the fetched README pages.
- The Liu–Hill gap under opening-start rules is unresolved.

## Benchmarking methodology

### Takeaway
No benchmarking standard is shared across papers. Four choices drive the results:
- **Board preset**: Beginner appears as 8×8, 9×9 or 10×10; Intermediate as 16×16, 15×13 or 13×15.
- **First-click rule**: unprotected, safe cell, or safe zero/neighbourhood.
- **Opening cell**: corner, (3,3), centre or random.
- **Game count**: 10⁴ for Studholme, Legendre and Liu; 10⁵ for Pedersen, Becerra, mrgris and HCSP; 10⁷–3.2×10⁸ for Tu.

**[corrected after primary check]** Several papers report uncertainty, but in different ways:
- Tu et al.: Beta-distribution quantiles in last-digit notation.
- Buffet et al.: "±", with the type unstated.
- Studholme: standard error of the mean over 100 sets of 100 games.
- Pedersen and Becerra: variance or SD across 1,000 sets of 100 games.
- mrgris: "margin of error".

Only Legendre et al. state that all algorithms played the same seeded minefields (paired design).

### Cited Findings
- **First-click rules in use**:
  - *Unprotected* (can lose on the first click). Used in Studholme's "hard rules" mode (p. 11), PGMS by default, and Liu et al.'s "first action rule" (Table 1). The 52.4% Beginner learning result (Castillo & Wrobel 2003) uses this rule — [minesweepergame.com list](https://minesweepergame.com/math-papers.php)
  - *Safe cell* ("classic"; Windows relocates the mine). Used by Studholme's "standard rules", Pedersen, Legendre ("minefield … generated after the first move", p. 6), Buffet et al. (PDF p. 7), Becerra ("soft rules", PDF p. 21), Tu et al. (p. 1000), Lordeiro et al. (p. 2), Liu PAR, Sajjad (after correction) and mrgris.
  - *Zero or opening.* Used by Hill "modern", Liu et al.'s "safe neighborhood rule" (p. 7) and Sajjad's early learners.
  - Buffet et al. note that the optimal strategies are the same for the unprotected and safe-cell variants. Under a "no mine in the neighbourhood of the first move" variant (GnoMine), corners are not optimal, and a 5×5/15 example is won 100% by opening in the centre (PDF pp. 6–7) — [Buffet et al.](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)
- **Start position**:
  - Studholme (p. 4), Pedersen (PDF pp. 43–44), Becerra, Buffet (OH) and Tu et al. all fix or prefer a corner.
  - Tu et al.'s Lemma 1 proves that a corner first probe maximizes survival within two moves. Corner-first is otherwise only a conjecture (Conjecture 1, p. 1001).
  - Legendre et al. measure corner against random against middle on 9×9: 0.90–0.91, 0.85–0.87 and 0.82–0.85 (Table 7, p. 19).
  - mrgris shows that a corner preference adds about 4.7–8.2 points: Expert 33.1% to 37.8%, Intermediate 69.2% to 77.4%, Beginner 9×9 84.8% to 91.4%.
  - Hill benchmarks the classic rule with a corner start and the modern rule with an open start at (3,3).
  - Liu et al. find that under the safe-neighbourhood rule a near-centre start is better, at about 85% against 71.5% for the top-left corner (p. 9).
  - Sources: [Tu et al.](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf); [Legendre et al.](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf); [mrgris](https://mrgris.com/projects/minesweepr/); [JSMinesweeper](https://github.com/DavidNHill/JSMinesweeper); [Liu et al.](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf)
- **Number of games and precision** **[corrected after primary check]**:
  - Studholme: 100 sets × 100 = 10⁴ games per data point; SEM ≤ 0.5 point (pp. 10–11).
  - Pedersen: 1,000 sets × 100 = 10⁵ per strategy, with variance across sets (PDF p. 42). Some search-heavy variants used only 1,000 or 600 games (PDF p. 70).
  - Legendre et al.: 10⁴ games per algorithm and level, on identical seeded minefields; no error bars (pp. 17–20).
  - Buffet et al.: HCSP over 10⁵ games (10⁴ on Expert); OH "±" values given, but the game count is not (Table 1).
  - Becerra: 1,000 sets × 100 = 10⁵, with the SD across sets (PDF pp. 7, 20; Table 6.2). The standard error of the mean is then about SD/√1000, roughly 0.09–0.15 point.
  - Tu et al.: PSEQ 10⁸ games (Beginner, Intermediate) and 3.2×10⁸ (Expert); PSEQ-D256 10⁷ and 1.88×10⁷; first-move study at 2.5×10⁷ per cell and 5×10⁷ per corner (p. 1002). Uncertainties come from 0.05 quantiles of a Beta distribution. The earlier inference of "on the order of 10⁷ games" was too low for PSEQ.
  - Liu et al.: 10⁴ games (pp. 8–9); no error bars.
  - Sajjad (CNN): at least 10⁵ test games per level (PDF p. 54).
  - mrgris: 10⁵ games; "margin of error ≈0.15%".
  - Hugging Face dataset: 1,000 seeded games per level.
  - DDQN hobby project: 25,000 games.
  - Sources: [Studholme](https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf); [Pedersen](https://minesweepergame.com/math/the-complexity-of-minesweeper-and-strategies-for-game-playing-2004.pdf); [Legendre et al.](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf); [Buffet et al.](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf); [Becerra](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf); [Tu et al.](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf); [Liu et al.](https://minesweepergame.com/math/a-solver-of-single-agent-stochastic-puzzle.pdf); [arXiv 2212.10446](https://arxiv.org/pdf/2212.10446); [mrgris](https://mrgris.com/projects/minesweepr/); [Hugging Face](https://huggingface.co/datasets/LKForge/minesweeper-solver-benchmark/raw/main/README.md); [Minesweeper-DDQN](https://github.com/AlexMGitHub/Minesweeper-DDQN)
- **Seeds and pairing**:
  - Legendre et al.: "for each game level and each algorithm, we randomly generate the same set of minefields to play with (using the same random seed)" (p. 17).
  - Buffet et al. list "pairing simulations" inside UCT as future work (PDF p. 7).
  - The Hugging Face dataset uses a seeded RNG.
  - No other paper reviewed states how it handled seeds.
- **Other metrics reported**:
  - Guesses per game: Pedersen Table 5.1; Becerra Tables 6.1–6.2 (1.70, 2.39 and 4.73 for CSCSP); Hill 3.3 per classic Expert win; Hugging Face 0.30, 0.89 and 3.82.
  - No-guess fraction: Studholme 5.4% of Expert games won without guessing (Fig. 6, p. 13); Hugging Face 78.5%, 52.4% and 3.8%.
  - Loss reasons, Studholme on Expert: craps shoot 12.1%, educated guess 17.8%, other guess 36.1% (Fig. 6).
  - Calibration of predicted against observed guess success: Studholme Tables 1–3, pp. 8–10.
  - Time per game: Tu Table 4; Legendre Tables 4–13; Studholme Fig. 5.
  - Frontier size and solution counts: Legendre Tables 5–6, 8–9, 12–13.
- **Paired comparison tooling**: Hill's repo has `MinesweeperBulk` and `MinesweeperBulkCompare` modules for bulk and head-to-head runs. Their method (shared seeds or not) is undocumented in the README. — [DavidNHill/Minesweeper](https://github.com/DavidNHill/Minesweeper)
- **Runtime limits and hard instances**:
  - Hill caps exact work by solution count: 750 for brute force in JSMinesweeper, under 4,000 for deep analysis in the Java solver.
  - Tu caps the exact optimal recursion at S ≤ 256 consistent situations (PSEQ-D256).
  - Legendre et al. cap enumeration at 1 s. The timeout fired in up to 7.9% of Expert games for cSimEnuAwLo and 0.13% for cSimEnuLoClf (Table 12, p. 21).
  - Buffet et al.'s OH uses 100 simulations per move on Intermediate and Expert, and 10,000 on smaller boards.
  - Sources: [JSMinesweeper](https://github.com/DavidNHill/JSMinesweeper); [DavidNHill/Minesweeper](https://github.com/DavidNHill/Minesweeper); [Tu et al.](https://minesweepergame.com/math/exploring-efficient-strategies-for-minesweeper-2017.pdf); [Legendre et al.](https://minesweepergame.com/math/minesweeper-where-to-probe-2012.pdf); [Buffet et al.](https://minesweepergame.com/math/optimistic-heuristics-for-minesweeper-2012.pdf)

### Inferences
- Recommended protocol, synthesized from the above:
  - Fix the board preset explicitly (W×H/M).
  - Fix the first-click rule (unprotected, safe-cell or zero) and the opening cell.
  - Generate boards from recorded seeds *after* the first click, applying the rule.
  - Run every solver on the same seeds (Legendre et al.'s design), so differences are paired and the variance is lower.
  - Report n, the win rate and a binomial 95% interval (Wilson, or the Beta/Clopper-Pearson interval as in Tu et al.).
  - Also report guesses per game, the no-guess fraction, and time per game or per move.
  - Cap exact enumeration by solution count or frontier size, with a deterministic fallback. Report the fraction of games where the cap fired (Legendre et al. report this).
- Resolving a 0.5-point difference near 40% at about 2σ needs about 4×0.24/0.005² ≈ 38,000 games per solver if unpaired. Paired seeds reduce this a lot. So 1,000-game benchmarks cannot separate modern solvers, and 10,000-game benchmarks (Legendre, Liu) resolve only about 2-point differences. Tu et al.'s 10⁸ games are the only results precise enough to rank solvers within a point.
- Hard instances for exact enumeration are boards with a long, connected frontier and many undetermined cells. Legendre et al.'s Expert runs reach up to 3.7×10⁶ solutions for one CSP (Table 13). Expert's 20.6% density versus the critical density in Louf 2025 is only a plausible link: Louf proves a coarse transition without locating the threshold numerically in the abstract checked here.

### Gaps
- Apart from Legendre et al. and the Hugging Face card, none of the retrieved sources documents seed handling or paired comparisons.
- Buffet et al. do not say whether "±" is one standard error or a 95% interval.
- No published benchmark of hard frontier instances for exact enumeration (runtime against frontier size) was found. Legendre et al.'s per-game solution-count statistics are the closest.

## Verification log

Primary documents read in full on 2026-11-06:
- Tu et al. 2017
- Studholme 2000
- Pedersen 2004
- Becerra 2015
- Legendre et al. 2012 (RR-8041)
- Buffet et al. 2012 (hal-00750577v2)
- de Bondt 2012
- Golan 2011 (preprint)
- Nakov & Wei 2003
- Gaspers et al. 2018
- Hendrickson & Tockman 2024
- Louf 2025 (abstract and introduction)
- Liu et al. 2022
- Lordeiro et al. 2021 (v2)
- Sajjad 2022
- Mehta 2021
- Tang et al. 2018
- Sinha et al. 2021
- Li, Wang, Zhang 2023
- Sheppard 2023 (abstract)
- the minesweepergame.com math-papers list
- the mrgris page
- the Hugging Face dataset card

**Confirmed (snippet tag removed, page or table citation added)**
- Kaye 2000 venue and pages, 22(2):9–15 (Crossref). Scott et al. 2011 venue and pages, 33(4):5–17 (Crossref).
- de Bondt: PP-hard (Thm 2, p. 11); PSPACE-complete when the win probability may be infinitesimal (Thm 3, p. 12); hexagonal and triangular NP-completeness.
- Golan: polynomial time on trees and bounded treewidth (Thms 2.1–2.4, pp. 3–4).
- Tu et al. PSEQ 81.627(8)%, 78.122(8)%, 39.616(5)%, and PSEQ-D256 81.79(2)%, 78.22(3)%, 40.06(2)% (abstract; Table 5). Board 8-8-T10. Safe first probe. Corner advantage.
- Studholme Expert about 34% (Fig. 6 sums to 34.0%). 8×8 Beginner. Corner opening.
- Becerra 91.25%, 75.94%, 32.90% (Table 6.2). C3SP "slightly worse".
- OH 80.2%, 74.4%, 38.7% (Buffet et al. Table 1).
- TeddySweeper 91.78%, 81.52%, 77.84%, 38.84% (as listed by minesweepergame.com).
- mrgris 84.8%, 69.2%, 33.1% and corner variants 91.4%, 77.4%, 37.8%, over 100,000 games.
- Lordeiro "over 75% and 55%".
- Sajjad's CSP baseline 32.90% equals Becerra's (it is Becerra's code).
- Mehta's DQN on 16×16×2 input with 256 actions, and "close to 45%".
- Hugging Face figures 95.5%, 85.6%, 36.2%, 78.5%, 3.8%.
- The Tu author list.
- Hill's figures (fetched directly in the first session; not re-fetched).

**Corrected [corrected after primary check]**
- Scott et al. 2011's coNP-hardness proof is flawed. The fixed proof is Hendrickson & Tockman 2024 (arXiv 2404.14519 / FUN 2024).
- #P-completeness of counting was first proved by Nakov & Wei (2003). Their paper was located; it also solves the MDP exactly up to 4×4. Pedersen (2004) reproved it.
- Pedersen claims DP-completeness of "Solution" and "Safety". This was added, with a caveat.
- Pedersen: the 92.5%, 67.7%, 25.0% figures belong to limited search with probability estimates, not to single-point (77.4%, 29.1%, 0.5%). His Beginner is 10×10/10. He ran 100,000 games.
- Studholme's 80.0% and 33.9% are not printed in his paper; they are secondary readings of bar charts. His Intermediate was 15×13/40. He ran 10⁴ games per point with SEM ≤ 0.5.
- Legendre et al. **do** report win rates: about 0.91 / 0.78 / 0.37–0.38 on 9×9, 16×16 and 16×30 with a corner start, and 0.80–0.81 on 8×8. They used 10⁴ games on seeded, paired minefields, with no error bars.
- Buffet et al.: error bars ±0.48, ±0.3, ±0.5 and ±1.8 added. OH uses 100 simulations per move on Intermediate and Expert. A 9×9 row (89.9%) was added. HCSP beats OH on 9×9 and 16×16.
- TeddySweeper is described as a reduced-complexity CSP, not POMDP-based.
- Tu et al.'s game counts are 10⁸–3.2×10⁸, not about 10⁷. Their errors come from Beta-distribution 0.05 quantiles. Their Intermediate margin over prior work is overstated.
- "Minesweeper with Limited Moves" is confirmed as Gaspers, Rümmele, Saffidine, Tran, AAAI-18.
- Added Liu et al. 2022 (KBS): PAR 87.9%, 78.2%, 39.7% with a safe first click; PAFG 96.4%, 86.3%, 45.6% with a safe neighbourhood; 10⁴ games.
- Lordeiro's best agent: 76.96%, 57.94%, 4.13%, on Beginner 8×8.
- arXiv 2212.10446: the best CNN with a safe first click is 88.4%, 68.8%, 21.5%. The 10.1% figure is a weak zero-start learner. The 52.4% figure is Castillo & Wrobel 2003, not this paper.
- arXiv 2102.06019's "45%" uses non-standard densities (<35% and ≥70%) on 16×16.
- Sinha et al.: best 92.02% on 9×9/10, with no Expert figure.
- Tang et al.: figures exist only as a bar chart (about 52%, 48%, 10%).
- LLM paper: GPT models solved 0% of 5×5/4 games; a human solved 89%.
- Hugging Face: the card claims a safe centre first click, not a zero start. Its Beginner and Intermediate numbers therefore conflict with the literature.
- de Bondt's "infinitesimal" means a win probability in [¾·2^−e, 2^−e].

**Still unverified**
- The full texts of Kaye 2000 and Scott et al. 2011 (Springer challenge). Kaye's "Infinite Minesweeper is Turing complete" (Birmingham page 404; the Wayback Machine did not resolve).
- The MDPI *Applied Sciences* 2025 CNN paper (93.3%, SD 0.77%): www.mdpi.com does not resolve here.
- The TeddySweeper paper itself, beyond the minesweepergame.com summary.
- The meaning of OH's "±" and its game counts.
- Hill's game counts and seeds.
- Fandom's 41.7% figure.
- The DDQN 23.60% figure (not re-fetched).
- The Springer 2025 and ACM 2025/26 papers (paywalled, no numbers).
- The exact fraction in de Bondt's bound (the text layer reads "34 · 2−e", read here as ¾).
- No 2023–2026 peer-reviewed paper beating Tu et al.'s 40.06% under the classic safe-first-click Expert rule was found. The coverage of recent work is still incomplete.
