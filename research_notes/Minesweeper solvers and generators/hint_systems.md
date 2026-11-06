# Explainable, human-style hint systems for Minesweeper (and related logic puzzles)

Research scope: how existing hint systems pick a hint and explain it, and a recommended tiered hint design for Runesweeper's classic (XP random) and max-guess modes whose explanations follow how expert players reason.

Method note: this session's network blocked many primary hosts: chiark.greenend.org.uk (Simon Tatham's docs), arxiv.org and ar5iv, sudokuwiki.org, minesweepergame.com, sudleyplace.com (MySweeper "Clues"), blogs.cs.st-andrews.ac.uk ("Minesweeper Hint System"), sift.net and faculty.washington.edu (HCI papers), and davidnhill.github.io. To work around this I cloned two codebases from GitHub and read them directly: JSMinesweeper (DavidNHill/JSMinesweeper, master, last commit 2026-10-04) and a GitHub mirror of Simon Tatham's Portable Puzzle Collection (ghewgill/puzzles, master, last commit 2020-09-21, so it may lag upstream). Findings that come only from search-result snippets are marked "(search snippet; page not fetched)".

## 1. What existing Minesweeper hint features show (a cell, a reason, probabilities) and how they phrase the reason

### Takeaway
Existing Minesweeper helpers show either *which* cells are safe or mines (as colours) or *how likely* a cell is to be safe (percentages). None that I could inspect explain *why*, for example "this 1 is already satisfied". JSMinesweeper is the most complete: colour-coded safe, mine and best-guess cells, per-tile percentages for safety, "2nd safety" and "progress", and status lines such as "Tile X is an unavoidable 50/50 guess". Simon Tatham's Mines has no hint feature at all, only a "Solve" option that reveals the whole layout. I found no reliable documentation of hint features in Microsoft Minesweeper (Windows 8/10), Google Minesweeper, or minesweeper.online.

### Cited Findings
**JSMinesweeper (David Hill): player hints and analyser**
- With the "Show hints" option on, the solver shadows the player and highlights safe plays and, when needed, the move it considers the best guess — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Analyser colours: "The safe tiles are shown in green and the mines in red". The best move when nothing is certain is yellow with a green centre. Other moves the solver considered and rejected are plain yellow. Grey marks tiles with only one possible value (dead tiles), and the README says it is never correct to play these while other moves are available — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Choosing a guess: only tiles within 10% of the safest tile's probability are analysed (if the safest is 80%, the cutoff is 72%). Each candidate gets a "progress" percentage (the chance the move leads to certain-safe moves next turn) and a "secondary safety" score (the chance of surviving this move and the next). The two are blended into a final score, which the README says performed better than simply picking the safest tile. A true 50/50 should always be guessed at the earliest opportunity — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- The code makes the 10% cutoff a constant, `SELECTION_THRESHOLD1 = 0.1`. The blend is `action.weight = action.prob * (1 + progress * PROGRESS_CONTRIBUTION)` with `PROGRESS_CONTRIBUTION = 0.2` — [solver_main.js L11-15, L1409](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- Brute force: once 750 or fewer solutions remain, the solver runs an exhaustive search that finds the best path and states the exact win probability under best play — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Hover tooltip wording (`Tile.getHintText`/`setProbability`):
  - `"Safe"` for a certain safe tile
  - `"Mine"` for a certain mine
  - otherwise `"NN.NN% safe"`, plus `"NN.NN% 2nd safety"` and `"NN.NN% progress"` when the guess analysis ran

  The tooltip shows only when hints or analysis are on — [Tile.js L88-113](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/Tile.js); [main.js L3240-3246](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/main.js)
- Status-line wording in the solver, verbatim from `showMessage(...)` calls:
  - "No mines left to find, all the remaining tiles are safe"
  - "No safe tiles left to find, all the remaining tiles are mines"
  - "Found N trivial safe moves"
  - "Found N safe tiles."
  - "The probability engine has found some safe tiles."
  - "Tile X is an unavoidable 50/50 guess." / "... unavoidable 50/50 guess, or safe."
  - "Tile X has a NN% chance to solve the isolated edge."
  - "Tile X has a NN% chance to win the game."
  - "Best follow up move is X with NN% win rate."
  - "All tiles are dead, try tile X."
  - "Guessing heuristic suggests clearing tile X."
  - "Found the safest guess using the probability engine."
  - "The best guess is off the edge."

  Most of these messages end with a count of the remaining solutions — [solver_main.js L305-942](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- Solver pipeline order:
  1. Global trivial cases: no mines left, or no safe tiles left.
  2. "Trivial" single-number analysis. In `trivial_actions`, a revealed number whose count of solver-known mines equals its value clears its other covered neighbours.
  3. A full probability engine.
  4. 50/50 detection, guess analysis and brute force.

  Trivial analysis counts mines the solver itself found (`isSolverFoundBomb`), not the player's flags — [solver_main.js L1164-1194, L300-375](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)

**Simon Tatham's Mines**
- The docs advertise the generator: it places mines "in such a way as to ensure that you never need to *guess* where a mine is: you will always be able to deduce it somehow" — [puzzles.but, Mines chapter (GitHub mirror)](https://github.com/ghewgill/puzzles/blob/master/puzzles.but)
- There is no hint feature. Of the full layout the docs say: "(If you really want to know the full layout of the grid, which other implementations will show you after you die, you can always use the Solve menu option.)" — [puzzles.but (GitHub mirror)](https://github.com/ghewgill/puzzles/blob/master/puzzles.but)
- `solve_game` only refuses with "Game has not been started yet" before the first click. Otherwise it returns the move "S", which reveals everything — [mines.c L2306-2315 (mirror)](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- Across the collection, Solve "transforms the puzzle instantly into its solved state", and it is not supported in some games (Cube) (search snippet; page not fetched) — [Tatham collection docs mirror](https://www.chiark.greenend.org.uk/~jharvey/puzzles/puzzles.html)

**Other web and mobile implementations**
- A minesweeper.online-style clone describes a hint that reveals one safe cell, at most 3 per game, each costing 200 points (search snippet; page not fetched; this is not minesweeper.online itself) — [minesweeper-online.pages.dev](https://minesweeper-online.pages.dev/)
- The WaveSpeed free Minesweeper describes a hint that names a cell only when every possible mine arrangement agrees on it, explains the reasoning, says so when nothing can be proven, and keeps the game out of your stats once you use a hint (search snippet; page not fetched) — [wavespeed.ai/free-tools/minesweeper](https://wavespeed.ai/free-tools/minesweeper)
- minesweeper.one describes a hover-and-press-H option that reveals what is under a square, which is a peek rather than a reasoned hint (search snippet) — [minesweeper.one](https://minesweeper.one/)
- Microsoft Minesweeper (Windows 8/10): my searches found no page describing a hint button or its behaviour. Results covered only basic rules and features such as "Play Same Board" — [Microsoft Store listing (third-party app)](https://microsoft.com/en-id/p/minesweeper-online-classic-challenge-for-windows10/9n3k5vvj235d); [Ask Dave Taylor](https://askdavetaylor.com/how-to-play-microsoft-minesweeper-in-windows-10)

### Inferences
- Even the strongest open-source tool stops at "this cell, and how safe it is". No tool I inspected tells the player *which numbers* force a cell, so a reason that names the technique would be new.
- Two choices in JSMinesweeper are worth copying:
  - **Use the solver's own knowledge, not the player's flags**, when deciding whether a number is satisfied. A wrong player flag should produce a correction ("this flag can't be right"), not a bad hint.
  - **The order of its guess advice:** a certain move first; then an unavoidable 50/50 *early*; then the best guess by safety blended with progress; then "off the edge" (a cell away from all numbers).
- Commercial games seem to treat a hint as a scored or limited resource (a cap per game, a point cost, or exclusion from stats). Runesweeper should decide how hint use interacts with best times and stats.

### Gaps
- I could not confirm whether Microsoft Minesweeper (Windows 8/10 app), Google's search-embedded Minesweeper or minesweeper.online offer hints at all. All three need hands-on checking.
- The MySweeper "Clues" pages (sudleyplace.com, including a "Transitive" clue page) and the St Andrews "Minesweeper Hint System" blog post showed up in search results but were blocked here. They look like the closest prior art for *named* Minesweeper clue types and deserve a follow-up fetch.
- I did not inspect mobile apps with hints (for example Minesweeper Q or Minesweeper GO).

## 2. Hint systems in comparable logic puzzles: technique tiers, hint choice, phrasing, and research on hints and learning

### Takeaway
Sudoku tools are the mature precedent:
- **They rank techniques by difficulty and give the easiest step that applies.** Tatham's Solo defines its difficulty levels by the reasoning each one requires. HoDoKu's solver works through techniques in a configurable order.
- **They reveal a hint in stages.** HoDoKu's three stages are: name only ("vague hint"), more detail ("concrete hint"), then "show next step" with grid highlighting and a text description.

Research on explaining constraint problems step by step (Bogaerts, Gamba, Guns) formalises "the simplest next step" as the cheapest minimal set of constraints that forces a new fact. HCI research on hint design in games exists, but I found no controlled study of nudge → region → cell → reason tiers.

### Cited Findings
**HoDoKu (Sudoku)**
- Its manual describes three hint types, all on the Puzzle menu:
  - **Vague hint** (Alt+F12): "only displays the name of the step."
  - **Concrete hint** (Ctrl+F12): adds detail such as the affected candidates.
  - **Show next step** (F12 or the "Next hint" button): highlights the step in the grid and writes a text description into the hint area.

  — [HoDoKu manual, Playing the game](https://hodoku.sourceforge.net/en/docs_play.php)
- Most steps take one line of description. Complex techniques (Forcing Chains, Forcing Nets, Kraken Fish) take one line per sub-step, and clicking a line shows only that sub-step. After highlighting, "Execute" applies the step and "Cancel" closes it without applying — [HoDoKu manual](https://hodoku.sourceforge.net/en/docs_play.php)
- The solver's configuration covers "Solver strategy", "Changing order of techniques" and "Progress measurement". I saw only the table of contents, not the section text — [HoDoKu docs index](https://hodoku.sourceforge.net/en/docs.php)

**Simon Tatham's Solo (Sudoku) and Mines solver**
- Solo's documentation: "Difficulty levels are judged by the complexity of the techniques of deduction required to solve the puzzle: each level requires a mode of reasoning which was not necessary in the previous one. In particular, on difficulty levels 'Trivial' and 'Basic' there will be a square you can fill in with a single number at all times, whereas at 'Intermediate' level and beyond you will have to make partial deductions about the *set* of squares a number could be in ... At 'Unreasonable' level, even this is not enough, and you will eventually have to make a guess, and then backtrack" — [puzzles.but, Solo chapter (mirror)](https://github.com/ghewgill/puzzles/blob/master/puzzles.but)
- In code the tiers are `DIFF_BLOCK, DIFF_SIMPLE, DIFF_INTERSECT, DIFF_SET, DIFF_EXTREME, DIFF_RECURSIVE` (named Trivial, Basic, Intermediate, Advanced, Extreme, Unreasonable). The standalone solver's "show working" mode prints a reason line per step, followed by actions such as "placing %d at (%d,%d)" and "ruling out %d at (%d,%d)" — [solo.c L120-122, L880-905 (mirror)](https://github.com/ghewgill/puzzles/blob/master/solo.c)
- **The Mines solver has a fixed order that mirrors human reasoning.** Each number becomes a "set" of hidden squares with a mine count. The solver tries three things in turn:
  1. **One set alone.** A set with 0 mines, or with as many mines as squares, is fully known.
  2. **Pairs of overlapping sets.** For two overlapping sets it computes the "wings", the squares each set has that the other lacks: "If one set has more mines than the other, and the number of extra mines is equal to the cardinality of that set's wing, then we can mark every square in the wing as a known mine, and every square in the other wing as known clear." Failing that, "see if one set is a subset of the other. If so, we can divide up the mine count of the larger set between the smaller set and its complement".
  3. **Global mine count, last.** Only "when all else fails" does it "resort to global deduction based on the total mine count. This is computationally expensive compared to any of the above deductions, which is why we only ever do it when all else fails".

  — [mines.c L296-312, L812-873, L884-897, L939-953 (mirror)](https://github.com/ghewgill/puzzles/blob/master/mines.c)

**Other puzzles**
- Nonograms:
  - One logic-puzzle site's Hint button highlights the next cell that line logic forces and explains why (search snippet; page not fetched) — [logic-puzzles-online nonogram solver](https://logic-puzzles-online-vas3p.kinsta.page/tools/nonogram-solver/)
  - Another solver lists every legal pattern for a line and marks cells that all remaining patterns agree on (search snippet) — [Towards Data Science, "Solving nonograms with 120 lines of code"](https://towardsdatascience.com/solving-nonograms-with-120-lines-of-code-a7c6e0f627e4)
- Hexcells: the public tools are full solvers without step explanations. HexSolver reads the screen and executes the next valid step, and HexcellsAI reduces the game to SAT (search snippets) — [HexSolver](https://github.com/Mikescher/HexSolver); [HexcellsAI](https://github.kxpsado.workers.dev/polarbart/HexcellsAI)

**Research on step-wise explanations**
- Bogaerts, Gamba and Guns, "A framework for step-wise explaining how to solve constraint satisfaction problems" (arXiv 2020; *Artificial Intelligence* vol. 300, 2021), using logic grid puzzles from the 2019 "Holy Grail Challenge":
  - The algorithm generates candidate explanation steps and picks the smallest by a cost function.
  - Clues are presented in natural language.
  - Experiments showed the approach is feasible, judged on the quality of individual explanations and of whole explanation sequences.

  (search snippets; arXiv blocked) — [arXiv 2006.06343](https://arxiv.org/abs/2006.06343); [VUB research portal](https://researchportal.vub.be/en/publications/step-wise-explanations-of-constraint-satisfaction-problems/)
- Follow-up work by Gamba, Bogaerts and Guns (IJCAI 2021; JAIR 2023) finds optimal explanation steps faster using optimal constrained unsatisfiable subsets. One report says it beats the MUS-based approach in explanation quality and speed, up to 56% faster on average (search snippets) — [arXiv 2105.11763](https://arxiv.org/abs/2105.11763v4); [arXiv 2303.11712](https://arxiv.org/pdf/2303.11712v3.pdf)
- Foschini et al. (2025) study preference elicitation, learning which explanation steps users prefer, on Sudoku and logic grid puzzles (search snippet) — [arXiv 2511.10436](https://arxiv.org/html/2511.10436v1)

**HCI research on hints in games**
- Wauck et al. (IUI 2017) studied hints in *Foldit* and a *Three Body Puzzle* reproduction. Hints' effects on players were complicated, and helpfulness depended on player experience and behaviour (search snippet; PDF blocked) — [Wauck et al., "A Data-Driven, Multidimensional Approach to Hint Design in Video Games"](https://www.sift.net/sites/default/files/publications/wauck_iui2017hints.pdf)
- He et al. (2022) built an adaptive, multimodal hint system for puzzle games and ran a user study with 18 participants across 3 puzzle games (search snippet; PDF blocked; venue unconfirmed) — [He et al., "An Adaptive Hint System for Puzzle Games"](https://faculty.washington.edu/weicaics/paper/papers/HaoHZC2022.pdf)
- Thumu et al. (CHI PLAY Companion 2023) used a hint that pulses clickable objects after 5 seconds of inactivity and planned a step-by-step hint system (search snippet) — [ACM DL](https://dl.acm.org/doi/fullHtml/10.1145/3573382.3616070)
- A practitioner blog argues that progressive hint levels act as sub-goals and smooth the difficulty curve. This is anecdotal, not empirical — [Tau Games blog](https://taugames.ca/blog/hints.html)

### Inferences
- The Sudoku pattern carries over directly. Rank Minesweeper deductions by how many numbers they need, offer the easiest one available, and reveal it in stages (technique name or area, then cell, then the full reason). HoDoKu's "vague hint", which shows only the technique name, is a real and useful middle stage between "there is a move" and "here is the move".
- Tatham's Mines solver order — one number, then a pair of overlapping numbers (the wing/subset rules), then the global mine count, used reluctantly — is a ready-made human-difficulty ladder that a real generator already uses.
- The Bogaerts/Gamba work gives a formal definition of "simplest hint": the minimal, lowest-cost set of constraints that forces a new fact. In Minesweeper a constraint is one revealed number, or the total mine count.

### Gaps
- I could not read HoDoKu's solver-ordering and progress sections, SudokuWiki's strategy list and explanation wording (blocked), or the full Bogaerts/Gamba papers (arXiv blocked). Their exact cost-function weights are therefore unverified. I recall the ECAI/AIJ paper weighting the number of constraints used much more heavily than the number of facts used, but could not check this.
- I found no controlled study comparing nudge → region → cell → reason tiers for learning or enjoyment. I also found nothing on KenKen hint systems.

## 3. Explaining a Minesweeper deduction in human terms at each tier

### Takeaway
Expert players build every local deduction from two primitives:
- **A single number** that is satisfied (enough mines found) or saturated (as many hidden neighbours as its remaining count).
- **A two-number comparison**: the 1-1 and 1-2 patterns, subset/difference, which Tatham calls "wings". Larger patterns such as 1-2-1 and 1-2-2-1 are chains of these, often after "reduction", which means subtracting known mines from each number first.

Beyond that come case splits over 3 or more numbers, the global mine count, and finally a guess, explained by its probability and what it buys.

### Cited Findings
- Pattern guides say there are two basic patterns, 1-1 and 1-2, "which combine to make all other patterns". "When you see 1-2-X on a row the X is always a mine" (search snippet; page not fetched) — [minesweepergame.com Patterns](https://minesweepergame.com/strategy/patterns.php); [minesweeper.now pattern guide](https://minesweeper.now/blog/the-minesweeper-pattern-bible-how-to-recognize-1-2-1-1-2-2-1-and-2-1-2-formations)
- Reduction: "Each set of numbers reduces when you subtract known mines". For example, a 2-3-2 next to flags reduces to 1-2-1 (search snippet) — [minesweepergame.com Patterns](https://minesweepergame.com/strategy/patterns.php)
- 1-2-1 along a wall: mines at the ends, middle cell safe (search snippet) — [minesweeper.now](https://minesweeper.now/faq/minesweeper-patterns)
- 1-2-2-1: sources conflict. One says the mines sit under the two middle 2s and the outer cells are safe. Another says the mines go at the ends and the middle is safe. Working the constraints supports mines under the middle 2s (search snippets) — [burmly 1-2-2-1](https://burmly.pages.dev/blog/minesweeper-1-2-2-1-pattern/) vs [minesweeperblast 1-2-2-1](https://minesweeperblast.pages.dev/patterns/1-2-2-1-pattern)
- Pattern guides also caution that genuine 50/50s exist where no pattern helps (search snippet) — [minesweeper.now](https://minesweeper.now/faq/minesweeper-patterns)
- The formal versions of these rules:
  - single-set rule
  - wing rule ("number of extra mines is equal to the cardinality of that set's wing")
  - subset split
  - global count ("minesleft == 0 || minesleft == squaresleft")

  — [mines.c (mirror)](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- Guess explanations in an expert tool. JSMinesweeper frames a guess as:
  - its safety ("NN% safe")
  - the chance it leads to a certain move next ("progress")
  - survival over two moves ("2nd safety")
  - an "unavoidable 50/50", which should be taken early
  - "off the edge"

  — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper); [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)

### Inferences
Example explanation text, one template per deduction type. In the UI, "this 2" means the number is highlighted; avoid coordinates.

- **D1a, satisfied number (safe):** "This 1 already touches its mine (marked). Its other hidden neighbours must be safe."
  - If the player has no flag there, the mine is one the hint system has proven: "This 1 already touches a mine we know about…"
- **D1b, saturated number (mines):** "This 3 has exactly 3 hidden neighbours left, so all of them are mines."
  - Reduced form: "This 4 touches 2 flagged mines and needs 2 more. Only 2 hidden cells are left next to it, so both are mines."
- **D2a, subset, safe (the 1-1 shape):** "This 1's hidden neighbours are all also next to that 1. Both numbers need exactly one mine, so it lies in the shared cells. The cells only the second 1 touches are safe."
- **D2b, difference, mine (the 1-2 shape):** "The 2 needs one more mine than the 1. The 2 has only one cell the 1 doesn't touch, so that cell must be the extra mine." Optionally add: "…and the 1's own extra cell is safe."
- **D2 after reduction:** begin with "After subtracting the flagged mines, this 3 acts like a 1…" and then continue as D2a or D2b. This mirrors how players are taught reduction.
- **D3, 3+ numbers (chain or case split).** Prefer a two-step chain ("First, … so this cell is a mine. Then this 2 is satisfied, so …"). Only when no chain exists, use a case split: "Suppose this cell were safe. Then this 1 would need its mine here, which would give that 2 three mines, which is impossible. So this cell is a mine." This is like HoDoKu's one-line-per-sub-step layout.
- **D4, global mine count:** "Only 2 mines remain on the board, and these 2 cells must hold them (from the numbers here). Every other hidden cell is safe." Or the reverse: "There are as many mines left as hidden cells, so all are mines."
- **G, no certain move.** Lead with "There is no safe cell you can prove right now", then give one reason:
  - **Unavoidable 50/50:** "These two cells are a true 50/50. Nothing you uncover later can tell them apart, so guess now rather than at the end, when it would cost more."
  - **Safest guess:** "This cell is the safest: about 85% safe. The next safest is 78%."
  - **Most informative / progress:** "This cell is 80% safe, and if it's safe it will very likely open up a certain move (≈60%)."
  - **Off the edge:** "Cells away from the numbers are each about X% safe (mines left ÷ unknown cells), safer than any cell on the edge."
- Prefer explaining safe deductions before mine deductions. In classic play the player progresses by revealing safe cells, and flags are optional.

### Gaps
- The 1-2-2-1 conflict is settled only by my own reasoning plus one source. The fetched primary pattern page (minesweepergame.com) was blocked.
- I found no source that tests which wording players understand best. All the templates above are my design, not sourced.

## 4. Detecting the "simplest" deduction (fewest constraints) so the hint matches what a human sees first

### Takeaway
Rank candidate deductions by the smallest set of revealed numbers that forces a cell, trying 1 number, then 2, then 3, and so on, with the global mine count as an expensive last resort. This is Tatham's Mines solver order and the "smallest explanation" idea from the step-wise explanation research. Both can be computed cheaply by enumerating small sets of overlapping numbers.

### Cited Findings
- **Tatham's escalation:** first a single set, then overlapping pairs (wing and subset rules), and global mine-count deduction only "when all else fails" because it is "computationally expensive" — [mines.c L812-897 (mirror)](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- **JSMinesweeper's escalation:** it tries "trivial" single-number moves before running the probability engine, and skips trivial analysis when the engine is needed anyway ("Skipping trivial analysis since Probability Engine analysis is required") — [solver_main.js L328-334, L1164-1194](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- **Explanation research:** the next step is the cheapest candidate explanation under a cost function. Later work finds optimal (cost-minimal) unsatisfiable subsets directly (search snippets) — [arXiv 2006.06343](https://arxiv.org/abs/2006.06343); [arXiv 2303.11712](https://arxiv.org/pdf/2303.11712v3.pdf)
- **Tiers keyed to technique:** Sudoku tools key difficulty to the technique required, each level needing "a mode of reasoning which was not necessary in the previous one" — [puzzles.but, Solo (mirror)](https://github.com/ghewgill/puzzles/blob/master/puzzles.but)

### Inferences
Suggested algorithm for Runesweeper:
1. **Build constraints.** Each revealed number becomes (hidden neighbour set, remaining count). Remaining count = value − proven mines, computed from the solver's own knowledge rather than player flags. Separately, check player flags against proven mines and give a "wrong flag" hint first if one contradicts a number.
2. **k = 1.** Any constraint with remaining 0 gives safe cells (D1a). Remaining = |set| gives mines (D1b).
3. **k = 2.** For each pair of constraints that share hidden cells, apply the wing rule and the subset split (D2). Pairs come only from numbers within 2 cells of each other, so this step is cheap.
4. **k = 3 and up (to about 4 or 5).** Grow connected groups of overlapping constraints. Enumerate all mine assignments over their combined hidden cells. A cell that is a mine in every assignment, or safe in every assignment, is forced by that group. Keep the smallest group that forces something. That group is the "minimal explanation", and it gives the D3 text. A minimal subset can also be found by deleting constraints one at a time while the cell stays forced, which is the MUS idea.
5. **Global.** Add the total-mine-count constraint (D4) only when no local group forces anything. Report it as the global-count technique.
6. **No forced cell.** Fall back to probabilities (G), using a guess policy like JSMinesweeper's: safest cell first, re-ranked within about 10% of the best by progress; an unavoidable 50/50 is recommended early; an off-edge cell when it is safer.

Tie-breaks within the same k:
- fewer hidden cells involved
- a safe result before a mine result
- nearer the player's last click or cursor (keeps attention local, as a human would)
- more cells resolved at once

Classic mode reaches the guess tier far less often than max-guess mode, so step 6 matters most there.

### Gaps
- I found no measurements of how well "fewest constraints" predicts what humans find first. Treat it as a plausible proxy, consistent with how pattern guides teach 1-1 and 1-2 as the base patterns. Bogaerts et al.'s cost weights could not be verified here.

## 5. Recommended tiered hint design for Runesweeper (classic XP random and max-guess modes)

### Takeaway
Use one "Hint" action that steps through four levels of disclosure per stuck moment: nudge → region → cell → full reason. The deduction behind it is the simplest one available from section 4 (D1 → D2 → D3 → D4 → G). The phrasing uses the section 3 templates, and the system always says honestly when no certain move exists. Mark any game where a hint was used as assisted.

### Cited Findings
- Staged disclosure has precedent in HoDoKu: name only, then concrete detail, then highlighted step with text, and Execute/Cancel — [HoDoKu manual](https://hodoku.sourceforge.net/en/docs_play.php)
- Precedents for honesty about guesses: JSMinesweeper says "unavoidable 50/50 guess", "best guess is off the edge" and so on — [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js). WaveSpeed says so when nothing can be proven (search snippet) — [wavespeed.ai](https://wavespeed.ai/free-tools/minesweeper)
- Hint cost and stats precedents:
  - 3 hints per game at −200 points each (search snippet) — [minesweeper-online.pages.dev](https://minesweeper-online.pages.dev/)
  - a game that used a hint is kept out of stats (search snippet) — [wavespeed.ai](https://wavespeed.ai/free-tools/minesweeper)

### Inferences
**Disclosure levels.** Each press of the Hint key goes one level deeper. Making a move resets to level 1.
1. **Nudge** (no location). Examples:
   - "There's a safe cell you can find by logic."
   - "You can find a mine by logic, but no safe cell yet."
   - "No cell can be proven right now. Press Hint again for the best guess."
   - "This flag can't be right."

   Add the technique name, as in HoDoKu's vague hint: "Look for a number that's already satisfied" (D1a), "Compare two neighbouring numbers" (D2), "Count the mines left" (D4).
2. **Region.** Highlight the numbers in the minimal explanation set, but not the target cell: "Look at these two numbers."
3. **Cell.** Highlight the forced cell(s) and the action: "This cell is safe." / "This cell is a mine."
4. **Reason.** Give the full sentence from the section 3 templates. For D3, give one line per sub-step, as HoDoKu does. Offer an "Apply" button that reveals or flags the cell, so the hint never plays the move unasked.

**Mode differences.**
- **Classic (XP random)** boards often have long logical stretches and occasional forced guesses. Levels 1–4 cover D1–D4 most of the time. In the G tier, explain *why* a guess is needed ("No cell can be proven…") and show the probability.
- **Max-guess mode** reaches the G tier often. Make the G explanation strongest here:
  - percent safe, compared with the next best option
  - "unavoidable 50/50, guess it now"
  - "this guess will likely open up certain moves (progress)"
  - "off-edge cells are X% safe (N mines left / M unknown cells)"

  Show at most two numbers so players are not overloaded. Mode semantics: this repo has no document defining "max-guess mode"; see Gaps.

**Stats.** Mark a game where any hint was used as assisted, and keep it out of best times. This follows precedent: WaveSpeed excludes it from stats, and some clones cap or penalise hints.

### Gaps
- I could not find a definition of Runesweeper's "max-guess" mode in the repository (no matches in `src/` or `docs/`). The recommendations assume it produces more forced guesses than classic.
- No usability evidence backs the specific four-level split. Validate it with playtests.
