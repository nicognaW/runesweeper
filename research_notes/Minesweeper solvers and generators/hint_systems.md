# Explainable, human-style hint systems for Minesweeper (and related logic puzzles)

Research scope: how existing hint systems pick a hint and explain it, and a recommended tiered hint design for Runesweeper's classic (XP random) and max-guess modes whose explanations follow how expert players reason.

Method note: the first pass ran with many primary hosts blocked. It read two codebases cloned from GitHub: JSMinesweeper (DavidNHill/JSMinesweeper, master, last commit 2026-10-04) and a 2020 GitHub mirror of Simon Tatham's Portable Puzzle Collection (ghewgill/puzzles, commit 01c3e38, 2020-09-21).

This second pass (2026-11-06) had wider network access and checked the notes against primary sources:
- **Tatham.** I cloned upstream `https://git.tartarus.org/simon/puzzles.git` (HEAD 616da16, 2026-09-22) and diffed it against the mirror. The git.tartarus.org web viewer returns 403 to automated clients ("restricted because of issues we're having with AI crawler bots"), so citations below give viewer URLs pinned to that commit, plus line numbers from the clone. I also fetched the chiark.greenend.org.uk Mines manual page.
- **Other primary sources fetched:** sudleyplace.com (MySweeper Clues and Rules), sudokuwiki.org (solver page, Strategy Families, What's New), HoDoKu's "Playing the game" page, arXiv PDFs of the Bogaerts/Gamba/Guns papers, the Wauck et al. and He et al. PDFs, and the App Store listings for Mineswifter and Evil Mineswifter.
- **Still unreachable:** blogs.cs.st-andrews.ac.uk (DNS fails; the host must be allowed in the session's network settings).
- **Not usable:** minesweeper.online serves a JavaScript app with no static help text.

Remaining "(search snippet; page not fetched)" tags mark claims that are still unverified.

## 1. What existing Minesweeper hint features show (a cell, a reason, probabilities) and how they phrase the reason

### Takeaway
Most Minesweeper helpers show *which* cells are safe or mines, or *how likely* a cell is to be safe. They do not say *why*.

[corrected after primary check] The first pass said no tool explains why. That is wrong. Some tools do give a reason:
- **Explainable Minesweeper** (sublevelgames, 2025) gives a natural-language reason for two-number deductions.
- **MySweeper "Clues"** (sudleyplace.com) marks cells that an expert "could reasonably determine" using four documented rules, and shows a live count of such cells in the title bar.
- **Mineswifter's** hint "show[s] you exactly which part of the board is solvable next".

JSMinesweeper remains the most complete *probability* helper. Upstream Simon Tatham Mines (checked at the 2026-09-22 HEAD) still has no hint feature, only Solve, which reveals the whole layout.

I still found no reliable documentation of hint features in Microsoft Minesweeper (Windows 8/10), Google's Minesweeper or minesweeper.online.

### Cited Findings
**JSMinesweeper (David Hill): player hints and analyser** (read from code in pass 1; not re-checked)
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

**MySweeper "Clues" (sudleyplace.com, v1.08, © 2006-2020)**, verified on the primary pages
- Clue modes:
  - In "Clues|Combined" mode one marker is used for both blanks and mines.
  - In "Clues|Split" mode one marker means a blank and another means a mine.
  - Markers appear only "on those cells for which an expert Minesweeper player could reasonably determine, based upon the surrounding cells and following certain rules, are of the specified type."

  — [MySweeper Clues](https://sudleyplace.com/mysweeper/clues.htm)
- Graded help, verbatim: "start with Clues|None and Clues|Enumerate (both the default). When the going gets tough, check the clue count in the window title bar. If it's non-zero, there's something you're missing -- look more carefully at the layout. If that's not enough help, turn on Clues|Combined. If you need even more help, turn on Clues|Split." It adds a teaching instruction: "When you use the clues, attempt to figure out how each clue was determined -- how could you have figured it out yourself?" — [MySweeper Clues](https://sudleyplace.com/mysweeper/clues.htm)
- The rules engine, verbatim from the rules page: "MySweeper implements one-level rules, not transitive cases. That is, it doesn't act upon hints, only exposed cells and (possibly incorrectly) flagged cells." The four implemented rules:
  1. All adjacent mines flagged → the remaining unexposed cells are empty (`ADJMINE == ADJFLAG`).
  2. The mine count equals flags plus unexposed cells → all unexposed cells are mines.
  3. Subset rule: "If the unexposed cells surrounding cell #1 are a subset of the unexposed cells surrounding cell #2, and they have the same number of still buried (unflagged) mines, then none of the unexposed cells in R2 \ R1 are mines."
  4. Difference rule: if `REST1 < #(R2 INT R1)` and `(REST2 − REST1) == #(R2 \ R1)`, then R2 \ R1 are all mines and R1 \ R2 are all blank.

  Rules 3 and 4 consider only a second cell "within two cell radius". The unimplemented rules are (5) transitive chains, where the page suggests one would "recurse at each stage, or ... solve a set of simultaneous linear equations" (credited to Dave Rusin), and (6) endgame mine-count rules — [MySweeper Rules](https://sudleyplace.com/mysweeper/rules.htm)

**Explainable Minesweeper (sublevelgames, July 2025)**, a new find in this pass
- The developer's write-up groups the hint rules into three tiers:
  - **"Global Patterns":** all mines found, so the rest are safe; or all safe cells found, so the rest are mines.
  - **"Simple Patterns":** a single number is saturated or satisfied.
  - **"Advanced Patterns":** two numbers A and B, comparing "the common area" with "the unique area that each cell has separate from the common area".

  The developer says the inspiration was *14 Minesweeper Variants* hints, which use symbols that "took me a long time to interpret", and that 3+-number patterns were deliberately excluded "as they're too complex". Placing a flag in the wrong cell is also game over — [sublevelgames blog](https://sublevelgames.github.io/blogs/2025-07-06-making-explainable-minesweeper/)
- Example hint text as shown in-game, reported by a third-party review: "Analysis shows the common area must have exactly 1 mines. Therefore, the remaining area of number 2(B) is safe." The common area is highlighted in purple — [GIGAZINE](https://gigazine.net/gsc_news/en/20250710-explainable-minesweeper)

**Mineswifter / Evil Mineswifter (Bonzai Games, iOS/macOS)**, from the developer's App Store listings
- Mineswifter: "we added a hint system that uses the power of the Minesweeper AI to show you exactly which part of the board is solvable next." Also: "The main statistic tracked in Mineswifter is the perfect game, which is a board completed without mistakes or hints", and "Mistakes add 10 seconds to your time" (Undo is unlimited) — [App Store: Mineswifter](https://apps.apple.com/app/id1521190195)
- Evil Mineswifter uses the same hint system, but the listing says it was "ruined ... by having it complain every time" — [App Store: Evil Mineswifter](https://apps.apple.com/us/app/id1523249883)

**Simon Tatham's Mines** (upstream checked; unchanged in substance from the mirror)
- The upstream manual still advertises the no-guess generator: "it will generate its mine positions in such a way as to ensure that you never need to *guess* where a mine is: you will always be able to deduce it somehow" — [Mines manual, chiark](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/doc/mines.html); [upstream puzzles.but @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=puzzles.but;hb=616da16f25da428ccda805536209095bd062676e)
- There is still no hint feature. The manual says: "(If you really want to know the full layout of the grid, which other implementations will show you after you die, you can always use the Solve menu option.)" The only new user preference is "Highlight adjacent flags when clicking on a clue", which highlights flagged neighbours while the button is held "to count them". This counting aid is the closest thing to a hint — [Mines manual, chiark](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/doc/mines.html); [commit 3855a39, 2026-08-31](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=3855a39)
- Upstream `solve_game` (mines.c L4040-4048 @616da16) is unchanged: before the first click it returns "Game has not been started yet"; otherwise it returns the move "S", which reveals everything — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e)
- [corrected after primary check] The collection-wide Solve wording is verified in upstream source: Solve "Transforms the puzzle instantly into its solved state. For some games (Cube) this feature is not supported at all because it is of no particular use." The first pass cited this from a snippet of a third-party mirror — [upstream puzzles.but L148-156 @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=puzzles.but;hb=616da16f25da428ccda805536209095bd062676e)
- New since the 2020 mirror and relevant to hint design:
  - Mines now plays on any grid.c tiling (triangles, hexagons, Penrose, hats, Spectres) and has "wrapping" (torus) modes. Neighbours are any tile that "touches it, even if only at a corner".
  - The solver now works on generic neighbour sets rather than 3×3 bitmasks. Hint code that assumes 8 neighbours would not carry over.

  — [commit d324fe9, 2026-09-02](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=d324fe9); [commit 428913c](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=428913c); [Mines manual](https://www.chiark.greenend.org.uk/~sgtatham/puzzles/doc/mines.html)

**Other web and mobile implementations** (still snippet-only unless noted)
- A minesweeper.online-style clone describes a hint that reveals one safe cell, at most 3 per game, each costing 200 points (search snippet; page not fetched; this is not minesweeper.online itself) — [minesweeper-online.pages.dev](https://minesweeper-online.pages.dev/)
- The WaveSpeed free Minesweeper describes a hint that names a cell only when every possible mine arrangement agrees on it, says so when nothing can be proven, and counts a hinted game as practice that is kept out of your stats (search snippet; the page was downloaded but its hint text was not inspected) — [wavespeed.ai/free-tools/minesweeper](https://wavespeed.ai/free-tools/minesweeper)
- minesweeper.one describes a hover-and-press-H option that reveals what is under a square, which is a peek rather than a reasoned hint (search snippet) — [minesweeper.one](https://minesweeper.one/)
- Microsoft Minesweeper (Windows 8/10): a second search again found no page describing a hint button. Reviews describe the modes (Classic, Adventure, Daily Challenges) without mentioning hints — [Wikipedia: Microsoft Minesweeper](https://en.wikipedia.org/wiki/Microsoft_Minesweeper); [Windows Report review](https://windowsreport.com/microsoft-minesweeper-windows-8-rtreview/)
- Google's search-embedded Minesweeper: search results (low-quality SEO pages only) mention three difficulty levels and no hint feature (search snippet) — [SlashGear](https://www.slashgear.com/1460391/microsoft-minesweeper-on-google-browser)
- St Andrews "Minesweeper Hint System" (Doors Open blog): still unreachable. The only snippet says the system gives a hint when you don't know which square to click next and combines constraint programming with a second method that the snippet cuts off (search snippet; page not fetched) — [St Andrews blog](https://blogs.cs.st-andrews.ac.uk/doorsopen/?p=970)

### Inferences
- **Named-rule hints have precedent.** MySweeper (2006-2020) and Explainable Minesweeper (2025) both use the same short ladder: single number, then a two-number subset or difference, then (MySweeper, unimplemented) transitive chains and endgame counting. A reason that names *which numbers* force a cell is therefore prior art, not new. What is still uncommon is a reason for chains of 3+ numbers and for the guess tier.
- **Teaching-oriented graded help also has precedent.** MySweeper's sequence (a count of deducible cells, then which cells, then whether each is a blank or a mine) is a nudge → cell → type progression, and it explicitly tells the player to work out the reason.
- Two choices in JSMinesweeper are worth copying:
  - **Use the solver's own knowledge, not the player's flags**, when deciding whether a number is satisfied. MySweeper does the opposite: it acts on "(possibly incorrectly) flagged cells" and assumes flags are correct. JSMinesweeper's approach is safer, and a wrong player flag should produce a correction ("this flag can't be right"), not a bad hint.
  - **The order of its guess advice:** a certain move first; then an unavoidable 50/50 *early*; then the best guess by safety blended with progress; then "off the edge" (a cell away from all numbers).
- Commercial games treat a hint as a scored or limited resource:
  - Mineswifter: a "perfect game" means no mistakes *or* hints.
  - Some clones cap hints per game or charge points.
  - WaveSpeed counts a hinted game as practice.

  Runesweeper should decide how hint use interacts with best times and stats.
- Tatham's move to arbitrary tilings shows that a constraint-set formulation (hidden-neighbour set plus count) generalises. A hint engine built on sets rather than 3×3 patterns would survive non-square boards.

### Gaps
- Microsoft Minesweeper (Windows 8/10 app), Google's Minesweeper and minesweeper.online: still unconfirmed whether any offers hints. All three need hands-on checking. minesweeper.online's help is rendered client-side, so a static fetch shows nothing.
- The St Andrews "Minesweeper Hint System" post is still unread (the host is blocked by DNS in this session).
- I did not inspect mobile apps with hints (for example Minesweeper Q or Minesweeper GO), or *14 Minesweeper Variants*' symbolic hint UI.

## 2. Hint systems in comparable logic puzzles: technique tiers, hint choice, phrasing, and research on hints and learning

### Takeaway
Sudoku tools are the mature precedent:
- **They rank techniques by difficulty and give the easiest step that applies.** SudokuWiki tries a numbered list of 42 strategies in order, from Singles to "Last Resort". HoDoKu "searches for steps in a predefined order" that is configurable. Tatham's Solo defines difficulty levels by the reasoning each requires.
- **They reveal a hint in stages.**
  - HoDoKu has three stages: name only ("vague hint"), detail ("concrete hint"), then "show next step".
  - SudokuWiki added a "Hints Mode" in April 2025: strategy name only, pattern only, or eliminations only.

Research on step-wise explanation (Bogaerts, Gamba, Guns) formalises "simplest next step" as the minimum of an explicit cost function. The cost function is now verified: in the AIJ paper, f(E,S,N) = basecost(S) + |E| + 5·|S|, with base cost 100 per clue used.

HCI studies on game hints (Wauck et al. 2017; He et al. 2022) found mixed effects. Players preferred fewer, on-demand, concise and implicit hints. I found no controlled study of nudge → region → cell → reason tiers.

### Cited Findings
**SudokuWiki (Andrew Stuart)**, verified on the primary site
- On ordering: "Difficulty is rather subjective but necessary, for example, when selecting the order in which strategies are tried by the solver ... I believe I have chosen an ordering which is not too controversial" — [SudokuWiki: Strategy Families](https://www.sudokuwiki.org/Strategy_Families)
- The current solver's numbered order, as listed on the solver page:
  - **Basic:** 1 Singles; 2 Pairs (Naked + Hidden); 3 Pointing Pairs; 4 Box/Line Reduction; 5 Triples (Naked + Hidden).
  - **"Tough Strategies":** 6 BUG+1; 7 X-Wing; 8 Unique Rectangles 1; 9 Chute Remote Pairs; 10 Y-Wing; 11 W-Wing; 12 Rectangle Elimination; 13 Simple Colouring; 14 Quads; 15 Swordfish; 16 XYZ Wing.
  - **"Diabolical Strategies":** 17–26, X-Cycles through Aligned Pair Exclusion.
  - **"Extreme Strategies":** 27–41, Grouped X-Cycles through Quad Forcing Chains.
  - **"Last Resort":** 42 Bowman's Bingo.

  Unchecked strategies are excluded from the solve path ("Solve Path excludes unchecked strategies") — [SudokuWiki solver](https://www.sudokuwiki.org/sudoku.htm)
- Hints Mode (solver v2.34, 9 April 2025): "added 'Hints Mode' next to Take Step. Choose between only revealing the next strategy, just the pattern or just the eliminations. The last option will apply the eliminations so you can 'take step' but see not explanations. The other two require you to find the eliminations." The dropdown labels are "Hints off / Strategy only / Pattern only / Elims only" — [SudokuWiki What's New](https://www.sudokuwiki.org/Whats_New); [SudokuWiki solver](https://www.sudokuwiki.org/sudoku.htm)
- The ordering is tuned to human perception (8 Dec 2025): "Some of the early strategies have been shuffled in their order. I've been persuaded that certain strategies are slightly easier that others if considered from the point of view of humans entering notes on a pen and paper puzzle rather than surveying the full panoply of candidates the solver offers." — [SudokuWiki What's New](https://www.sudokuwiki.org/Whats_New)
- Learner player (18 Feb 2026): "a learner can get the next best strategy and display the pattern, the eliminations or just the strategy name. It defaults to the full explanation." The author plans to "offer an alternative hint if the found strategy isn't desired" — [SudokuWiki What's New](https://www.sudokuwiki.org/Whats_New)

**HoDoKu (Sudoku)**, re-checked
- "Hints come in tree [sic] flavors (all in the Puzzle menu)":
  - **"Vague hint"** "only displays the name of the step".
  - **"Concrete hint"** "gives more details (e.g. affected candidates)".
  - **"Show next step"** "highlights the step in the sudoku area and writes a textual representation into the hint area text box".

  The manual also says "which step is hinted at depends on the current solver configuration" and "The solver searches for steps in a predefined order" — [HoDoKu manual, Playing the game](https://hodoku.sourceforge.net/en/docs_play.php)
- From the first pass (not re-checked this time):
  - Keyboard shortcuts: Alt+F12 vague, Ctrl+F12 concrete, F12 / "Next hint" for the next step.
  - Complex techniques (Forcing Chains, Forcing Nets, Kraken Fish) take one line per sub-step.
  - "Execute" / "Cancel" buttons.

  — [HoDoKu manual](https://hodoku.sourceforge.net/en/docs_play.php)
- The solver-order configuration pages ("Solver strategy", "Changing order of techniques") returned 403 to a direct fetch. Their text remains unread — [HoDoKu docs index](https://hodoku.sourceforge.net/en/docs.php)

**Simon Tatham's Solo (Sudoku) and Mines solver** (upstream re-checked; text unchanged)
- Solo's documentation, still verbatim in upstream: "Difficulty levels are judged by the complexity of the techniques of deduction required to solve the puzzle: each level requires a mode of reasoning which was not necessary in the previous one. In particular, on difficulty levels 'Trivial' and 'Basic' there will be a square you can fill in with a single number at all times, whereas at 'Intermediate' level and beyond you will have to make partial deductions about the *set* of squares a number could be in (or the set of numbers that could be in a square). ... At 'Unreasonable' level, even this is not enough, and you will eventually have to make a guess, and then backtrack if it turns out to be wrong." — [upstream puzzles.but L1094-1106 @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=puzzles.but;hb=616da16f25da428ccda805536209095bd062676e)
- Solo's tiers are `DIFF_BLOCK, DIFF_SIMPLE, DIFF_INTERSECT, DIFF_SET, DIFF_EXTREME, DIFF_RECURSIVE` (upstream solo.c L124). "Show working" prints actions such as "placing %d at (%d,%d)" and "ruling out %d at (%d,%d)" (upstream solo.c L895, L976) — [upstream solo.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=solo.c;hb=616da16f25da428ccda805536209095bd062676e)
- **The Mines solver's fixed order is unchanged upstream.** Each number becomes a "set" of hidden squares with a mine count. The solver tries three things in turn:
  1. **One set alone** (`s->mines == 0 || s->mines == s->n`, L2106).
  2. **Pairs of overlapping sets.** It computes the "wings" (`setwings`, L1566-1590; rule at L2143-2182): "If one set has more mines than the other, and the number of extra mines is equal to the cardinality of that set's wing, then we can mark every square in the wing as a known mine, and every square in the other wing as known clear. ... Failing that, see if one set is a subset of the other. If so, we can divide up the mine count of the larger set between the smaller set and its complement".
  3. **Global mine count, last** (L2195-2205): "resort to global deduction based on the total mine count. This is computationally expensive compared to any of the above deductions, which is why we only ever do it when all else fails". This tier enumerates combinations only if there are at most 10 sets (`bool setused[10]`; `if (nsets <= lenof(setused))`, L2211, L2290). The mirror has the same cap.

  — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e)

**Other puzzles** (not re-checked; snippet-only)
- Nonograms:
  - One logic-puzzle site's Hint button highlights the next cell that line logic forces and explains why (search snippet; page not fetched) — [logic-puzzles-online nonogram solver](https://logic-puzzles-online-vas3p.kinsta.page/tools/nonogram-solver/)
  - Another solver lists every legal pattern for a line and marks cells that all remaining patterns agree on (search snippet) — [Towards Data Science, "Solving nonograms with 120 lines of code"](https://towardsdatascience.com/solving-nonograms-with-120-lines-of-code-a7c6e0f627e4)
- Hexcells: the public tools are full solvers without step explanations. HexSolver reads the screen and executes the next valid step, and HexcellsAI reduces the game to SAT (search snippets) — [HexSolver](https://github.com/Mikescher/HexSolver); [HexcellsAI](https://github.kxpsado.workers.dev/polarbart/HexcellsAI)

**Research on step-wise explanations** (now read in full)
- Bogaerts, Gamba and Guns, "A framework for step-wise explaining how to solve constraint satisfaction problems". The arXiv preprint is from June 2020; the published version is *Artificial Intelligence* 300 (2021) 103550, doi:10.1016/j.artint.2021.103550. The goal, verbatim: find "a sequence of simple explanations, where each explanation should aim to be as cognitively easy as possible for a human to verify and understand" — [arXiv 2006.06343](https://arxiv.org/abs/2006.06343)
- [corrected after primary check] The cost function is now verified. The first pass said the weights could not be checked. For logic grid puzzles the paper uses:
  - f(E, S, N) = basecost(S) + |E| + 5·|S|, where E is the set of facts used, S the set of constraints used, N the newly derived facts, and nc(S) the number of clues in S.
  - basecost(S) = 0 if |S| = 1 and nc(S) = 0; 100 if |S| > 1 and nc(S) = 0; 100·nc(S) otherwise.

  "The number 100 is taken here to be larger than any reasonable explanation size." The design rests on stated observations, including "Previously derived facts are easier to use than clues or implicit constraints" and that humans "never needed to combine two clues". So the search order is:
  1. one implicit constraint;
  2. all implicit constraints plus at most one clue;
  3. clue pairs and triples.

  The hardest step found cost 108. "Nested explanations" let the user zoom into a hard step — [arXiv 2006.06343, §7 and §5](https://arxiv.org/pdf/2006.06343)
- Gamba, Bogaerts and Guns, "Efficiently Explaining CSPs with Unsatisfiable Subset Optimization", *JAIR* 78 (2023) 709-746, doi:10.1613/jair.1.14260. It extends their IJCAI 2021 paper of the same title (arXiv 2105.11763).
  - Method: a hitting-set algorithm finds the "optimal constrained unsatisfiable subsets" (OCUS).
  - Result: "our algorithms outperform the MUS approach in terms of explanation quality and computational time (on average up to 56 % faster than a standard MUS approach)".
  - Costs used in experiments: "for Logic Grid puzzles, we assign a cost of 60 for puzzle-agnostic constraints; 100 for puzzle-specific constraints; and a cost of 1 for facts. For all other puzzles [including Sudoku], we assign a cost of 60 when using a constraint and a cost of 1 for facts."

  — [arXiv 2303.11712](https://arxiv.org/abs/2303.11712); [arXiv 2105.11763](https://arxiv.org/abs/2105.11763)
- Foschini, Defresne, Gamba, Bogaerts and Guns, "Preference Elicitation for Step-Wise Explanations in Logic Puzzles" (arXiv, 13 Nov 2025):
  - Approach: learn the explanation objective "from pairwise comparisons", with dynamic normalisation of sub-objectives and a query strategy called MACHOP.
  - Evaluation: Sudokus and logic-grid puzzles, with artificial users and a real-user study.
  - Result: "MACHOP consistently produces higher-quality explanations than the standard approach."

  — [arXiv 2511.10436](https://arxiv.org/abs/2511.10436)

**HCI research on hints in games** (Wauck and He PDFs now read)
- Wauck et al., "A Data-Driven, Multidimensional Approach to Hint Design in Video Games", IUI 2017, doi:10.1145/3025171.3025224.
  - [corrected after primary check] The first pass said the study used *Foldit* and the *Three Body Puzzle*. It used only one game: "a reproduction of a notoriously tricky puzzle minigame known as 'The Master Sword Puzzle' in ... The Legend of Zelda: Twilight Princess", which the paper calls the "Three Body Puzzle". *Foldit* appears only as cited prior work, in which on-demand hints improved engagement.
  - The abstract reports 25 participants in the first experiment. Three hint types (adaptive, automatic, on-demand) were then evaluated with more players.
  - Findings, verbatim: "no player experience metrics or player behaviors (fun, frustration, easiness, boredom) were associated with the presence of hints or any of the hint types", and "54 out of the 66 players who saw hints (81%) rated the hints 1 or 2 out of 5".
  - Further findings: less-experienced players showed a "hint placebo effect", and "players in the on-demand hint condition saw hints less often and had higher levels of the Engagement factor".
  - Recommendation: "implement on-demand hints first to see how many times and at what points players typically request help".

  — [Wauck et al. PDF](https://www.sift.net/sites/default/files/publications/wauck_iui2017hints.pdf)
- He, Zhu and Cai (CUHK-Shenzhen), "An Adaptive Hint System for Puzzle Games: A Multimodal-based Approach".
  - Survey study: 18 participants across *Baba Is You*, *Gorogoa* and *The Room 3*. Takeaway: players consider a hint "that is concise, accurate, implicit, text-based, and without revealing the whole game flow as the most promising".
  - The adaptive system detects being stuck from webcam face, gaze and head pose. The training data came from 25 volunteers.
  - Controlled experiment: 30 participants, 5 per game per group.
  - [corrected after primary check] The first pass reported the 18-participant study as the user study of the system. It was the formative survey; the system evaluation used 30.
  - The PDF is IEEE-formatted, but the venue is not stated in the text I read.

  — [He et al. PDF](https://faculty.washington.edu/weicaics/paper/papers/HaoHZC2022.pdf)
- Thumu et al. (CHI PLAY Companion 2023) used a hint that pulses clickable objects after 5 seconds of inactivity and planned a step-by-step hint system (search snippet; not re-checked) — [ACM DL](https://dl.acm.org/doi/fullHtml/10.1145/3573382.3616070)
- A practitioner blog argues that progressive hint levels act as sub-goals and smooth the difficulty curve. This is anecdotal, not empirical — [Tau Games blog](https://taugames.ca/blog/hints.html)

### Inferences
- The Sudoku pattern carries over directly. Rank Minesweeper deductions by how many numbers they need, offer the easiest one available, and reveal it in stages. SudokuWiki's three hint modes map onto Minesweeper:
  - "Strategy only" → technique name.
  - "Pattern only" → highlight the numbers involved.
  - "Elims only" → mark the cell.
- The Bogaerts cost function translates cleanly. For a Minesweeper hint step, cost could be 100 × (revealed numbers used) + 5 × (constraints) + 1 × (already-known cells used), plus a large surcharge for the global mine count. This reproduces "one number < two numbers < chains < global count" and prefers steps that reuse already-proven cells. Their observation that humans rarely combine clues is consistent with Tatham's and Explainable Minesweeper's ordering.
- Tatham's Mines solver order is a ready-made human-difficulty ladder that a real generator still uses unchanged in 2026: one number, then a pair of overlapping numbers (the wing/subset rules), then the global count, used reluctantly.
- The HCI evidence argues for *on-demand* hints, not automatic ones, and for concise, implicit text by default. Hints should not be counted on to improve enjoyment by themselves.

### Gaps
- SudokuWiki's exact step-explanation sentences (the text "Take Step" prints) were not extracted; it is rendered by the solver script. HoDoKu's solver-order pages returned 403 to direct fetches.
- I still found no controlled study comparing nudge → region → cell → reason tiers for learning or enjoyment, and nothing on KenKen hint systems.
- The He et al. venue is unconfirmed.

## 3. Explaining a Minesweeper deduction in human terms at each tier

### Takeaway
Expert players build every local deduction from two primitives:
- **A single number** that is satisfied (enough mines found) or saturated (as many hidden neighbours as its remaining count).
- **A two-number comparison**, which minesweepergame.com teaches as "1-2-X" (a mine) and "1-1-X" (a safe cell), i.e. subset/difference. Tatham calls these "wings", and MySweeper encodes them as its Rules 3 and 4.

Larger patterns such as 1-2-1 and 1-2-2-1 are applications of 1-2-X from both sides, often after "reduction" (subtracting known mines from each number). Beyond that come chains or case splits over 3 or more numbers, the global mine count, and finally a guess, explained by its probability and what it buys.

### Cited Findings
- [corrected after primary check] minesweepergame.com frames 1-2-X and 1-1-X, not "1-1 and 1-2", as the base patterns. Verbatim:
  - "The two most famous patterns are 1-2-1 and 1-2-2-1 ... The 1-2-1 and 1-2-2-1 patterns are actually combinations of a more important pattern. When you see 1-2-X on a row the X is always a mine."
  - The page explains 1-2-X: "There are two mines in three squares (because the 2 touches three squares) but there is one mine in the first two squares (because the 1 touches two of the three squares). The third square must contain the other mine."
  - On 1-1-X: "The other important pattern is 1-1-X on a row starting from a border ... Both clues are true so the third square must be empty."
  - Both 1-2-1 and 1-2-2-1 are solved by "Apply the 1-2-X pattern from the left. Apply the 1-2-X pattern from the right."
  - Advanced logic is phrased as subsets: "Sometimes a mine is in a subset of squares so the remaining squares must be safe."
  - On chains: "You can combine patterns and logic to form chains of logic alternating between finding mines, using 1-1-X and using subsets."

  The snippet-sourced phrase "which combine to make all other patterns" does not appear on the page — [minesweepergame.com Patterns](https://minesweepergame.com/strategy/patterns.php)
- Reduction, verbatim: "Complex mine arrangements can be reduced to patterns. You simply subtract the known mines from each number. For example, if you flag a mine touching a 3 it becomes a 2." Reduced numbers are described in phrases like "the yellow 4 (effectively a 1)" — [minesweepergame.com Patterns](https://minesweepergame.com/strategy/patterns.php)
- 1-2-1 along a wall: mines at the ends, middle cell safe (search snippet) — [minesweeper.now](https://minesweeper.now/faq/minesweeper-patterns)
- 1-2-2-1: two low-authority sources conflict on where the mines go (search snippets) — [burmly 1-2-2-1](https://burmly.pages.dev/blog/minesweeper-1-2-2-1-pattern/) vs [minesweeperblast 1-2-2-1](https://minesweeperblast.pages.dev/patterns/1-2-2-1-pattern). The authoritative page (minesweepergame.com) gives the result as images only, but says it follows from applying 1-2-X from each end. Applying 1-2-X from each end puts a mine under each 2, so the cells under the 1s are safe. This agrees with the first pass's own working of the constraints.
- Prose phrasings already in use by other tools:
  - **MySweeper:** "If an exposed cell has all its adjacent mines flagged, the remaining unexposed cells are empty"; and the subset and difference rules quoted in §1 — [MySweeper Rules](https://sudleyplace.com/mysweeper/rules.htm)
  - **Explainable Minesweeper:** "the common area must have exactly 1 mines. Therefore, the remaining area of number 2(B) is safe" — [GIGAZINE](https://gigazine.net/gsc_news/en/20250710-explainable-minesweeper)
  - **The same developer's walk-through of a 1-3 difference:** "B needs 3 mines but already has 1 flagged, so it only needs 2 more. However, we cannot place 2 or more mines in the common area with A, because A must have only 1 mine as a neighbor. We also can't place 0 ... Therefore ... A's unique area becomes safe." — [sublevelgames blog](https://sublevelgames.github.io/blogs/2025-07-06-making-explainable-minesweeper/)
- Pattern guides also caution that genuine 50/50s exist where no pattern helps (search snippet) — [minesweeper.now](https://minesweeper.now/faq/minesweeper-patterns)
- The formal versions of these rules, verified in upstream code:
  - single-set rule
  - wing rule ("number of extra mines is equal to the cardinality of that set's wing")
  - subset split
  - global count (`minesleft == 0 || minesleft == squaresleft`, L2263)

  — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e)
- Guess explanations in an expert tool. JSMinesweeper frames a guess as:
  - its safety ("NN% safe")
  - the chance it leads to a certain move next ("progress")
  - survival over two moves ("2nd safety")
  - an "unavoidable 50/50", which should be taken early
  - "off the edge"

  — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper); [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)

### Inferences
Example explanation text, one template per deduction type. In the UI, "this 2" means the number is highlighted; avoid coordinates. The "common area / only-this-number area" vocabulary used by Explainable Minesweeper is clearer than Tatham's "wings" for players.

- **D1a, satisfied number (safe):** "This 1 already touches its mine (marked). Its other hidden neighbours must be safe."
  - If the player has no flag there, the mine is one the hint system has proven: "This 1 already touches a mine we know about…"
- **D1b, saturated number (mines):** "This 3 has exactly 3 hidden neighbours left, so all of them are mines."
  - Reduced form: "This 4 touches 2 flagged mines and needs 2 more. Only 2 hidden cells are left next to it, so both are mines."
- **D2a, subset, safe (the 1-1-X shape):** "Every hidden cell next to this 1 is also next to that 1. Both need exactly one mine, so it lies in the shared cells. The cells only the second 1 touches are safe."
- **D2b, difference, mine (the 1-2-X shape):** "The 2 needs one more mine than the 1. The 2 has only one cell the 1 doesn't touch, so that cell must be the extra mine." Optionally add: "…and the 1's own extra cell is safe."
- **D2 after reduction:** begin with "After subtracting the flagged mines, this 3 acts like a 1…" and then continue as D2a or D2b. minesweepergame.com uses this phrasing ("effectively a 1").
- **D3, 3+ numbers (chain or case split).** Prefer a two-step chain ("First, … so this cell is a mine. Then this 2 is satisfied, so …"), as minesweepergame.com teaches. Only when no chain exists, use a case split: "Suppose this cell were safe. Then this 1 would need its mine here, which would give that 2 three mines, which is impossible. So this cell is a mine." This matches HoDoKu's one-line-per-sub-step layout and Bogaerts' "nested explanations".
- **D4, global mine count:** "Only 2 mines remain on the board, and these 2 cells must hold them (from the numbers here). Every other hidden cell is safe." Or the reverse: "There are as many mines left as hidden cells, so all are mines."
- **G, no certain move.** Lead with "There is no safe cell you can prove right now", then give one reason:
  - **Unavoidable 50/50:** "These two cells are a true 50/50. Nothing you uncover later can tell them apart, so guess now rather than at the end, when it would cost more."
  - **Safest guess:** "This cell is the safest: about 85% safe. The next safest is 78%."
  - **Most informative / progress:** "This cell is 80% safe, and if it's safe it will very likely open up a certain move (≈60%)."
  - **Off the edge:** "Cells away from the numbers are each about X% safe (mines left ÷ unknown cells), safer than any cell on the edge."
- Prefer explaining safe deductions before mine deductions. In classic play the player progresses by revealing safe cells, and flags are optional.

### Gaps
- The 1-2-2-1 placement on minesweepergame.com is shown only in images. The text-level reasoning (1-2-X from both ends) supports mines under the 2s, but I did not view the images.
- I found no source that tests which wording players understand best. All the templates above are my design, not sourced. The only evidence on style is He et al.'s survey preference for "concise, accurate, implicit, text-based" hints.

## 4. Detecting the "simplest" deduction (fewest constraints) so the hint matches what a human sees first

### Takeaway
Rank candidate deductions by the smallest set of revealed numbers that forces a cell, trying 1 number, then 2, then 3, and so on, with the global mine count as an expensive last resort. Several independent primary sources agree:
- Tatham's Mines solver order.
- MySweeper's and Explainable Minesweeper's rule tiers.
- The verified Bogaerts cost function: 100 per clue dominates.
- Cicvárek's 2017 "rank": the number of variables that must be assumed in the cheapest step.
- Dempsey and Guinn's 2020 measure: the size of the minimal unsatisfiable core needed for an inference.

All of these can be computed cheaply for small k by enumerating small sets of overlapping numbers.

### Cited Findings
- **Tatham's escalation (upstream):** first a single set, then overlapping pairs (wing and subset rules), and global mine-count deduction only "when all else fails" because it is "computationally expensive" — [upstream mines.c L2106-2205 @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e)
- **JSMinesweeper's escalation:** it tries "trivial" single-number moves before running the probability engine, and skips trivial analysis when the engine is needed anyway ("Skipping trivial analysis since Probability Engine analysis is required") — [solver_main.js L328-334, L1164-1194](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- **Explanation research (verified):**
  - f = basecost(S) + |E| + 5·|S|, with base 100 per clue. Candidate S are generated from one constraint upward — [arXiv 2006.06343 §7](https://arxiv.org/pdf/2006.06343)
  - OCUS finds a cost-optimal unsatisfiable subset directly, and is up to 56% faster than MUS-based search — [JAIR 2023 / arXiv 2303.11712](https://arxiv.org/abs/2303.11712)
- **Cicvárek (CTU Prague bachelor thesis, 2017)** defines a board "rank": "Rank is the maximal necessary amount of variables on which we have to apply our assumptions when making one step during a single game". His figures show "The cheapest available step is rank 1" versus "rank 2". He notes "The increase is exponential and higher rank steps are thus very hard to detect", and "Some very easy boards can have a rank of 0", meaning steps made "just by comparing the constants and the number of known mines and unknown squares that surround them" — [Cicvárek 2017 PDF](https://minesweepergame.com/math/algorithms-for-minesweeper-game-grid-generation-2017.pdf)
- **Dempsey and Guinn, "A Phase Transition in Minesweeper" (arXiv 2008.04116, 2020).**
  - Hardness measure: the "maximum size of the grouped minimal unsatisfiable (GMUS) cores" needed for an inference. The core size "is strongly peaked in the vicinity of the phase transition ... playing Minesweeper near the critical mine density requires looking at large patches of the board to make inferences".
  - Fast approximate solvers: polynomial "k-set search" algorithms, which only find inferences of core size ≤ k, "perform well below the critical mine density, but fail to find a significant proportion of inferences" near 0.2 < ρ < 0.3.

  — [arXiv 2008.04116](https://arxiv.org/pdf/2008.04116)
- **Tiers keyed to technique:** Sudoku tools key difficulty to the technique required, each level needing "a mode of reasoning which was not necessary in the previous one" — [upstream puzzles.but, Solo](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=puzzles.but;hb=616da16f25da428ccda805536209095bd062676e). SudokuWiki re-tuned its order for "humans entering notes on a pen and paper puzzle" — [SudokuWiki What's New](https://www.sudokuwiki.org/Whats_New)

### Inferences
Suggested algorithm for Runesweeper:
1. **Build constraints.** Each revealed number becomes (hidden neighbour set, remaining count). Remaining count = value − proven mines, computed from the solver's own knowledge rather than player flags. Separately, check player flags against proven mines and give a "wrong flag" hint first if one contradicts a number.
2. **k = 1.** Any constraint with remaining 0 gives safe cells (D1a). Remaining = |set| gives mines (D1b).
3. **k = 2.** For each pair of constraints that share hidden cells, apply the wing rule and the subset split (D2). Pairs come only from numbers within 2 cells of each other (MySweeper's "within two cell radius"), so this step is cheap.
4. **k = 3 and up (to about 4 or 5).** Grow connected groups of overlapping constraints. Enumerate all mine assignments over their combined hidden cells. A cell that is a mine in every assignment, or safe in every assignment, is forced by that group. Keep the smallest group that forces something. That group is the "minimal explanation" (a GMUS core in Dempsey and Guinn's terms), and it gives the D3 text. A minimal subset can also be found by deleting constraints one at a time while the cell stays forced, which is the MUS idea.
5. **Global.** Add the total-mine-count constraint (D4) only when no local group forces anything. Report it as the global-count technique.
6. **No forced cell.** Fall back to probabilities (G), using a guess policy like JSMinesweeper's: safest cell first, re-ranked within about 10% of the best by progress; an unavoidable 50/50 is recommended early; an off-edge cell when it is safer.

Tie-breaks within the same k:
- fewer hidden cells involved (Bogaerts' |E| term)
- reuse of already-proven cells rather than new numbers (Bogaerts' "previously derived facts are easier")
- a safe result before a mine result
- nearer the player's last click or cursor (keeps attention local, as a human would)
- more cells resolved at once

The step-4 k cap matters for latency. Dempsey and Guinn show that k-limited search misses many inferences near density 0.2–0.3, which Runesweeper's max-guess boards may approach. So the hint engine should fall back to an exact check before saying "nothing can be proven".

Classic mode reaches the guess tier far less often than max-guess mode, so step 6 matters most there.

### Gaps
- I found no measurements of how well "fewest constraints" predicts what humans find first. Bogaerts' weights are hand-chosen from observations, not fitted. Foschini et al. (2025) try to learn them from pairwise preferences, but only on Sudoku and logic grids.

## 5. Recommended tiered hint design for Runesweeper (classic XP random and max-guess modes)

### Takeaway
Use one on-demand "Hint" action that steps through four levels of disclosure per stuck moment: nudge → region → cell → full reason. The deduction behind it is the simplest one available from section 4 (D1 → D2 → D3 → D4 → G). The phrasing uses the section 3 templates, and the system always says honestly when no certain move exists. Mark any game where a hint was used as assisted.

### Cited Findings
- Staged disclosure has several verified precedents:
  - **HoDoKu:** name only, then concrete detail, then the highlighted step with text — [HoDoKu manual](https://hodoku.sourceforge.net/en/docs_play.php)
  - **SudokuWiki Hints Mode:** strategy only, pattern only, or eliminations only — [SudokuWiki What's New](https://www.sudokuwiki.org/Whats_New)
  - **MySweeper:** clue count in the title bar, then combined markers, then split markers — [MySweeper Clues](https://sudleyplace.com/mysweeper/clues.htm)
- Precedents for honesty about guesses:
  - JSMinesweeper says "unavoidable 50/50 guess", "best guess is off the edge" and so on — [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
  - WaveSpeed says so when nothing can be proven (search snippet) — [wavespeed.ai](https://wavespeed.ai/free-tools/minesweeper)
- Hint cost and stats precedents:
  - Mineswifter's tracked "perfect game" is "a board completed without mistakes or hints"; mistakes add 10 seconds — [App Store: Mineswifter](https://apps.apple.com/app/id1521190195)
  - 3 hints per game at −200 points each (search snippet) — [minesweeper-online.pages.dev](https://minesweeper-online.pages.dev/)
  - a hinted game counts as practice and stays out of stats (search snippet) — [wavespeed.ai](https://wavespeed.ai/free-tools/minesweeper)
- On-demand rather than automatic: on-demand hint players "saw hints less often and had higher levels of the Engagement factor" — [Wauck et al. 2017](https://www.sift.net/sites/default/files/publications/wauck_iui2017hints.pdf). Players prefer hints that are "concise, accurate, implicit, text-based, and without revealing the whole game flow" — [He et al.](https://faculty.washington.edu/weicaics/paper/papers/HaoHZC2022.pdf)

### Inferences
**Disclosure levels.** Each press of the Hint key goes one level deeper. Making a move resets to level 1.
1. **Nudge** (no location). Examples:
   - "There's a safe cell you can find by logic."
   - "You can find a mine by logic, but no safe cell yet."
   - "No cell can be proven right now. Press Hint again for the best guess."
   - "This flag can't be right."

   Add the technique name, as in HoDoKu's vague hint and SudokuWiki's "Strategy only": "Look for a number that's already satisfied" (D1a), "Compare two neighbouring numbers" (D2), "Count the mines left" (D4). An optional MySweeper-style count ("3 cells can be worked out") is an even lighter nudge.
2. **Region.** Highlight the numbers in the minimal explanation set, but not the target cell: "Look at these two numbers." This is SudokuWiki's "Pattern only".
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

**Stats.** Mark a game where any hint was used as assisted, and keep it out of best times. This follows precedent: Mineswifter's "perfect game" excludes hints, WaveSpeed counts hinted games as practice, and some clones cap or penalise hints.

### Gaps
- I could not find a definition of Runesweeper's "max-guess" mode in the repository (no matches in `src/` or `docs/`). The recommendations assume it produces more forced guesses than classic.
- No usability evidence backs the specific four-level split. Validate it with playtests.

## Verification log

Second pass, 2026-11-06, with wider network access.

**Confirmed against primary sources**
- **Upstream Tatham Mines** (git clone of `git.tartarus.org/simon/puzzles.git`, HEAD 616da16, 2026-09-22) matches the 2020 mirror on everything these notes rely on:
  - No hint feature; Solve reveals everything.
  - `solve_game` behaviour.
  - The manual's no-guess claim.
  - The three-tier solver order (single set → wing/subset → global count "when all else fails").
  - The global tier capped at 10 sets.
  - The collection-wide "Solve" wording and Cube exception.
  - The Solo difficulty text and `DIFF_*` tiers.
  - The chiark manual page.
- **JSMinesweeper** claims were read from code in pass 1. They were not re-checked, but they were already primary.
- **HoDoKu's three hint flavours,** plus "The solver searches for steps in a predefined order".
- **Bogaerts/Gamba/Guns:**
  - Titles, venues and DOIs: AIJ 300 (2021) 103550; JAIR 78 (2023) 709-746; the IJCAI 2021 version is arXiv 2105.11763.
  - The "up to 56% faster" result.
  - Foschini et al. 2025 (arXiv 2511.10436).
- **Pattern teaching on minesweepergame.com:** "1-2-X ... always a mine"; reduction by subtracting known mines.

**Corrected** (each marked "[corrected after primary check]" in the text)
- "No tool explains why" → MySweeper Clues and Explainable Minesweeper do give rule-based reasons. Mineswifter shows "which part of the board is solvable next".
- The Bogaerts cost function was "unverified, recalled" → verified as f = basecost(S) + |E| + 5·|S| with base 100 per clue. The JAIR experiments use 60/100 per constraint and 1 per fact.
- Wauck et al.: the study game was only the Twilight Princess "Master Sword Puzzle" reproduction; Foldit was prior work. Key numbers added (25 participants in the first experiment; 54/66 rated hints 1–2 of 5).
- He et al.: the 18 participants were a formative survey; the system's controlled evaluation used 30. Authors and games added.
- Base patterns are 1-2-X and 1-1-X, per minesweepergame.com. The snippet phrase "1-1 and 1-2 ... combine to make all other patterns" is not on the page.
- The Tatham collection "Solve" wording is now cited from upstream source rather than a third-party mirror snippet.

**Added (new primary sources)**
- MySweeper Clues and Rules (sudleyplace.com).
- Explainable Minesweeper (sublevelgames blog).
- Mineswifter and Evil Mineswifter App Store listings.
- SudokuWiki strategy order, Hints Mode (v2.34) and ordering rationale.
- Cicvárek 2017 "rank".
- Dempsey and Guinn 2020 GMUS core size.
- Upstream Mines changes since 2020: arbitrary tilings and wrapping grids; the "highlight adjacent flags" preference.

**Still unverified**
- St Andrews "Minesweeper Hint System" blog (host blocked by DNS).
- Microsoft Minesweeper (Windows 8/10), Google Minesweeper and minesweeper.online hint features (no primary documentation found; minesweeper.online is client-rendered).
- minesweeper-online.pages.dev, WaveSpeed and minesweeper.one hint descriptions (snippet-only).
- HoDoKu solver-order configuration pages (403).
- SudokuWiki's exact per-step explanation sentences.
- Thumu et al. 2023 and the Tau Games blog.
- Nonogram and Hexcells tools.
- The minesweeper.now and burmly/minesweeperblast pattern pages.
- The He et al. venue.
