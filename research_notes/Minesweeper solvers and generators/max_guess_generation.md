# Generating maximally guessy / adversarial Minesweeper boards, and using them as solver load tests

Research method note: the first pass read the source of five projects by cloning them: Kaboom, gazpachoking/minesweeper, AlexBuz/mindsweeper, DavidNHill/JSMinesweeper, and a 2020 GitHub mirror of Simon Tatham's Puzzles `mines.c` (ghewgill/puzzles, 01c3e38, 2020-09-21). Several primary hosts were unreachable then.

The second pass (2026-11-06) had wider network access and checked the notes against primary sources:
- **Tatham's code.** I cloned upstream `https://git.tartarus.org/simon/puzzles.git` with full history (HEAD 616da16, 2026-09-22) and diffed `mines.c` against the mirror. The git.tartarus.org web viewer returns 403 to automated clients, so citations give viewer URLs pinned to that commit and line numbers from the clone.
- **Pages read:**
  - Tatham's quasiblog "Writing a soluble-grid generator for Mines" (2019-08-26)
  - pwmarcz.pl's Kaboom post (2019-11-21)
  - the Evil Minesweeper itch.io page
  - the Lutris and App Store listings for Evil Mineswifter
  - andrewt.net's Pure Skill Minesweeper page
  - Cicvárek's 2017 thesis PDF
  - Dempsey and Guinn's phase-transition paper
  - hellpig/minesweeper-puzzle-generator (cloned at 43a624d, 2026-07-16)
- **Still unreachable:** dspace.ut.ee (DNS fails) and lists.tartarus.org (proxy 502). The tartarus-commits claims were instead checked against the git history itself.

Remaining "(snippet only)" tags mark claims that are still unverified.

## 1. Adversarial / dynamic-placement Minesweepers: how they place mines lazily

### Takeaway
Every working adversarial variant I examined uses the same core mechanism. It keeps no fixed mine layout. On each click it asks a constraint solver whether the clicked cell *can* be a mine and whether it *can* be safe, given every revealed number and the total mine count. It then commits to one full layout consistent with the answer it picked. The solvers are MiniSat compiled to JS for Kaboom, z3 for gazpachoking, and an in-house component enumerator for Mindsweeper.

Kaboom's author states the "cruel, but fair" rule this way. If the cell can be a mine and a provably safe cell exists anywhere, the click becomes a mine. If no safe cell exists, a guess is allowed and the cell is empty. So the player loses only by guessing when they did not have to. They never lose to a forced guess.

[corrected after primary check] "Evil Minesweeper" (itch.io) has *not* implemented its adversary; it is currently a no-guess game. Pure Skill Minesweeper (andrewt) is itself a cruel-but-fair adversary, which its author calls buggy, and he points players to Kaboom.

### Cited Findings

**Kaboom: the author's own description (pwmarcz.pl, 2019-11-21)**, now verified
- Intent: "what if you had to play Minesweeper against the computer? ... It could be quite cruel: if you are playing on a square that could contain a mine, it would always contain one! So the you have to always prove the square is safe." Because forced guesses would make that unwinnable: "I'll modify the idea a bit and say you are allowed to guess, but only if there are no safe squares left. This way, the game will be cruel, but fair." — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)
- The rules, verbatim:
  - "If you play a square that is guaranteed safe, it's empty."
  - "If you play a square that has a guaranteed mine, it will contain a mine and you will blow up."
  - "If you play a square that is uncertain, then: If there are any safe squares on the board, you are punished for guessing and that square will contain a mine. Otherwise, guessing is allowed and that square will be empty."

  — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)
- Design history:
  1. **Backtracking first.** The first version enumerated all boundary arrangements by backtracking, tracking "5 mines at the boundary, 5 mines remaining on the outside". On reveal it would "Select a random possibility (but one that satisfies the 'cruel, but fair' rules)", scatter the rest outside, then "Forget the possibility we selected earlier! Only the labels will be binding from now on."
  2. **It blew up.** "Somehow I managed to unlock 18 million possible mine arrangements. My Firefox is taking up 12 gigabytes of memory and revealing a square takes half a minute."
  3. **Switch to SAT.** The author moved to MiniSat compiled to JavaScript by Joel Galenson ("just 200 kilobytes"), rejecting z3's WebAssembly build ("a 17 MB download"). He notes the hard case: "there will be 'evil' positions that take a lot of time to compute".

  — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)
- Encoding, as described by the author:
  - **Labels:** each label uses "binomial encoding", meaning all subsets.
  - **Global count:** binomial encoding failed for the global count (6-digit clause counts), so he implemented the totalizer from "Efficient CNF encoding of Boolean cardinality constraints" by Bailleux and Boufkhad. It "generates Ө(N^2) clauses. When the number of open squares is around 100, the game becomes sluggish."
  - **Model reuse:** reusing each returned model to answer other "can x be 0/1?" questions "reduce[s] the number of queries by about 2 to 5 times".
  - **Caching:** forced cells are cached and dropped from the formula. The author calls this "a bit scary: we no longer have a formula stating the correctness of the entire board."

  — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)
- Outside-the-boundary rule and its rationale (fills a first-pass gap):
  - "some friends thought it was weird that it would guarantee you an empty spot anywhere. So I changed it so that clicking outside is always punished (as long as the game has not run out of bombs to punish you, that is)".
  - Exceptions: "nothing is revealed yet, or all bombs in the game can be proven to be on boundary (clicking outside must be safe), or all fields on the [boundary] are certain to have bombs (you are forced to click outside)."
  - "Update: The change turned out to be controversial, as the restriction is somewhat artificial. I added a switch that will allow/disallow outside guesses."

  — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)

**Kaboom source (pwmarcz/kaboom, `index.js` + `sat.js`, commit 2c5cc17, 2021-08-01)**, read in pass 1 and consistent with the blog
- No mine grid is stored during play. The `Game` constructor has `//this.mines = makeGrid(...)` commented out. The state is only `this.map = new LabelMap(...)`, the revealed numbers. A concrete `mineGrid` exists only transiently inside `Game.reveal()` or at death/win. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `LabelMap.recalc()` builds the **boundary**: unrevealed cells adjacent to a revealed number, each given an index in `boundaryGrid`. It counts `numOutside` = all cells − revealed − boundary. It also caches trivially forced mines: when a label equals its count of unknown neighbours, those neighbours are `setCache(id, true)`. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `makeSolver(map, maxMines)` builds a `Solver` with one boolean variable per boundary cell. It sets `minMines = maxMines - map.numOutside`, since the outside cells can absorb at most `numOutside` mines. Each revealed label becomes `addLabel(label, mineList)`. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- The encoding is in `Solver.run()` and `sat.js`. Each label is encoded as an exactly-k constraint using `Sat.assertAtLeast` / `Sat.assertAtMost`. These expand to **all combinations** of size (n−k+1) and (k+1) via `combinations()`, the binomial encoding, which is fine because n ≤ 8. The global mine count uses the recursive totalizer `Sat._addCounter`, which the blog confirms is Bailleux & Boufkhad's. `assertCounterAtLeast/AtMost` then bound it. — [Kaboom sat.js](https://github.com/pwmarcz/kaboom/blob/master/sat.js)
- For each uncached boundary cell i, `Solver.run()` makes up to two SAT calls, `solveWith(assert ¬x_i)` and `solveWith(assert x_i)`, to fill `_canBeSafe[i]` and `_canBeDangerous[i]`. An optimisation, `Solver.update(solution)`, marks every variable's observed polarity in each returned model as possible, which skips many later SAT calls. Results feed the cache (`map.setCache`). — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- **The "cruel but fair" decision in `Game.reveal(x, y)`:**
  - Boundary click: `if (canBeSafe(idx) && (!canBeDangerous(idx) || !hasSafeCells)) shape = anySafeShape(idx); else shape = anyDangerousShape(idx);`.
  - Outside click with the default `allowOutside=false`: the outside is treated as safe only if `boundary.length === 0 || outsideIsSafe() || !hasNonDeadlyCells()`, matching the blog's three exceptions.
  - With `allowOutside=true`: `outsideIsSafe() || (!hasSafeCells && outsideCanBeSafe())`.
  - Safe outcome: `anyShapeWithOneEmpty().mineGridWithEmpty(x,y)`. Mine outcome: `anyShapeWithRemaining().mineGridWithMine(x,y)`.

  — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `Shape` turns one SAT model into a full board. `baseMineGrid()` places boundary mines. `addRandom()` shuffles the outside cells and scatters the `remaining` mines among them. `floodReveal()` then computes real numbers from that transient grid and cascades on zeros. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- UI options and extras (`index.html`):
  - "Allow guessing anywhere (By default, you are allowed to guess only on the boundary of revealed tiles.)", "Safe mode (double-click to reveal)", "Countdown mode", and Debug mode.
  - Unlimited `undo()`.
  - A `hint()` message distinguishing "There are safe cells", "no safe cells, but you can guess", and "All surrounding cells are deadly. You need to play elsewhere."
  - Presets go up to 10x10 with 50 mines ("Ultra-Violence").

  — [Kaboom index.html](https://github.com/pwmarcz/kaboom/blob/master/index.html)
- [corrected after primary check] The phrase is the author's own ("cruel, but fair"), not a third-party paraphrase. The first pass cited it via GIGAZINE (snippet only) — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/)

**gazpachoking/minesweeper ("undetermined", Python + z3, commit d08da5d, 2025-07-12)** (pass 1, code-read)
- `NiceMode` enum in `undetermined/__init__.py`, comments verbatim:
  - NICE: "Any click that could result in an empty tile is an empty tile."
  - FAIR: "If guessing is the only move available, you will not guess wrong."
  - NORMAL: "Traditional minesweeper."
  - CRUEL: "Any click that could result in a mine is a mine. (Except when guessing is the only move available.)"

  The README credits Kaboom's "Cruel but fair" idea. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- `Board.solver()` builds a z3 solver with `z3.PbEq` (pseudo-Boolean exactly-k) for the global count over all undetermined tiles and for each revealed boundary number. Its comment reads: "Cheat a bit to randomize possible solution by adding constraints in a random order". — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- `Board.recalc()` "locks in" tiles via `solver.check(t.var)` and `solver.check(z3.Not(t.var))`. `Board.reveal()` encodes the fairness rule. If `not safe_moves and (not boundary_moves or tile in boundary_moves)`, the tile is forced safe. Otherwise, in CRUEL mode, it is forced to be a mine. `replace_mines()` then re-samples a full consistent layout. The first click is always forced safe. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)

**AlexBuz/mindsweeper (Rust/WASM, commit b804faa, 2023-12-29): the "no-guess + punish" opposite pole** (pass 1, code-read)
- README: it chooses "a random mine arrangement from the set of mine arrangements that would allow a perfect logician to win without guessing … sample uniformly". Guess punishment: "If you click on a tile that *can* be a mine, then it *will* be a mine, guaranteed." — [Mindsweeper README](https://github.com/AlexBuz/mindsweeper)
- `LocalGame::new()` in `src/server/local.rs` does **rejection sampling**. It shuffles mines, protects the first click and its neighbours, then auto-plays `analyzer.find_safe_moves(false)` until stuck. It retries unless the board is won by pure deduction. — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)
- `LocalGame::punish()` and `rearrange_mines()` produce a **uniform** sample of consistent layouts:
  - If the clicked tile `may_be_mine()`, the code restricts that tile's component's arrangements to those containing it.
  - If the tile is unconstrained, it pretends `known_mine_count += 1`.
  - `compute_weights()` weights each per-component mine-count combination by (product of arrangement counts) × `big_binomial(unconstrained, remaining)`.
  - It then picks via `choose_weighted`.
  - `punish_chord()` does the same for chords.

  — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)

**Other "evil" variants** (now checked on their own pages)
- [corrected after primary check] **Evil Minesweeper** (lightbulbMEOW, itch.io, "Prototype", Godot/HTML5). The page states the intent: "All the mines are adversary, rearranging if you ever make a choice that's not a valid logical deduction." But it continues: "However I haven't implemented that part yet. Right now, the grid can always be solved logically without needing to guess." No source code is linked — [Evil Minesweeper](https://lightbulbmeow.itch.io/evil-minesweeper)
- **Evil Mineswifter** (Bonzai Games, 2020, iOS/macOS) goes the other way: a static board with a forced guess, produced by rejection.
  - "Every time you start a game, Evil Mineswifter secretly runs an advanced Minesweeper AI to see if the board can be solved without guessing. If it solves the board, it randomly generates another board and tries again."
  - "every board contains a 50/50 guess and the hint system complains a lot."
  - Its daily challenges "all of which contain exactly one 50/50 guess".
  - Undo is offered for losses to a 50/50.

  The first pass had this from a snippet only. No source code or developer write-up was found — [Lutris: Evil Mineswifter](https://lutris.net/games/evil-mineswifter/); [App Store: Evil Mineswifter](https://apps.apple.com/us/app/id1523249883)
- [corrected after primary check] **Pure Skill Minesweeper** (andrewt) is not just "never penalised when forced to guess". It is a full cruel-but-fair adversary:
  - "Any time there simply isn't enough information to deduce a safe move, we'll pull Windows's trick and quietly rearrange the mines so you don't get penalised."
  - And: "if you can in theory deduce a safe move but guess anyway, the code will quietly rearrange the mines so that your unnecessary risk backfires and you lose."
  - The author says "the bugs started to appear" once people played it, and recommends "Paweł Marczewski's Kaboom" instead.
  - He names an open improvement: "make the game recognise when there's a forced guess _coming_". Today, a player who takes an obvious future 50/50 early, while a safe cell exists elsewhere, "will always lose".
  - He rejects a harsher rule, insisting the player choose "the square with the best odds of being safe": "I think that would be taking things too far."

  — [Pure Skill Minesweeper](https://github.andrewt.net/mines/)

### Inferences
- Kaboom does **not** sample uniformly in its SAT version. Whichever model MiniSat returns from `anySafeShape` / `anyDangerousShape` becomes the boundary layout, and only the outside mines are shuffled. The blog's "Select a random possibility" describes the earlier backtracking version. gazpachoking only randomises constraint order. Mindsweeper's `compute_weights` is the only one I read that samples uniformly over consistent layouts with exact counts. For a "max-guess" mode that must look like a fair random board after the fact, Mindsweeper's weighting approach is the model to copy.
- The adversary's rule reduces to two per-cell booleans, `canBeSafe` and `canBeDangerous`, plus one global `hasSafeCells`. Any solver that computes exact "forced safe / forced mine" sets, the P=0 / P=1 output of a probability engine, can drive this mode.
- Kaboom never makes a *forced* guess fail. andrewt names the known weakness: taking an inevitable 50/50 early is punished. A "max-guess" adversary needs a different policy. One version: at forced-guess points, make the click safe, but choose the layout (among consistent ones) that leaves the fewest safe deductions, creating the next forced guess soonest. (Speculation: I found no implementation of this "lazy max-guess" policy. Evil Minesweeper's planned adversary is unimplemented.)

### Gaps
- No source code is available for Evil Mineswifter, and Evil Minesweeper's adversary does not exist yet. So the only working max-guess generator is Evil Mineswifter's documented rejection loop, and its solver ("advanced Minesweeper AI") is unspecified.

## 2. Static boards that force guesses: forced 50/50 structures and how to embed them

### Takeaway
The most precise, code-backed catalog of unavoidable-guess structures I found is in DavidNHill/JSMinesweeper. It names "dead tiles", arbitrarily extended 2-tile 50/50s, enclosed 2x2 boxes holding 2 mines, and 2-tile and 2x2 "pseudo-50/50s", and it ships detection code. These structures can be embedded deliberately by placing the mines and then checking with the same detector. Tatham's quasiblog confirms that he knows of no forbidden-pattern characterisation of soluble grids and doubts a general one exists.

### Cited Findings
- JSMinesweeper README definition: "A 2-tile 50/50 is a pair of tiles which share 1 mine where one of the tiles can never receive information without the other tile also receiving the same information… they must be (and always will be) an unavoidable guess". The solver detects "Arbitrarily extended 2-tile 50/50s" and "Enclosed 2x2/2 boxes". A *pseudo-50/50* is "a set of tiles [that] is either safe or part of a 50/50". It detects "2-tile pseudo-50/50s" and "4-tile 2x2 pseudo-50/50s". It is "unable to discover extended pseudo-50/50s". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- "Dead tiles": tiles that "are either mines or have only one possible value… it is never correct to choose a dead tile unless all the tiles left in the game are dead. In which case… the result is down to luck." — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- How `FiftyFiftyHelper.process()` detects a horizontal 2-tile 50/50:
  - For adjacent covered tiles (i,j),(i+1,j), it rejects the pair if `isPotentialInfo` holds for any of the six tiles at columns i−1 and i+2 (rows j−1..j+1).
  - It then tests whether "both mines" is a valid option via `solver.countSolutions` with both tiles set as found bombs.
  - For 2x2 boxes it enumerates all mine patterns in the static `PATTERNS` table (4, 3, 2 and 1 mines). The starting index depends on `minesLeft` (>3 → 0, =3 → 1, =2 → 5, else 9).
  - It tracks `risky[l]` to tell a true box 50/50 from a pseudo-50/50, and skips candidates where `allDead`.

  — [JSMinesweeper FiftyFiftyHelper.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/FiftyFiftyHelper.js)
- `LongTermRiskHelper.js` estimates per-tile "50/50 influence": aggregate 2-tile 50/50 blast percentage, highest-impact 4-tile 50/50, and "enabler" influence for tiles that would complete a 50/50. It only considers "possible 50/50s with less than 3 missing mines". — [JSMinesweeper LongTermRiskHelper.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/LongTermRiskHelper.js)
- Tatham (2019) on characterising soluble grids: "I don't know of any rule of that kind; I very much doubt that there's a fully general one (that leaves every soluble grid still permitted)". A cautious rule would "constrain layouts far more than necessary, reducing variety of gameplay, and worse still, constituting an extra hidden clue to the mine positions for anyone who knows how the generator is working." — [Tatham quasiblog](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/mines-solver/)
- Low-authority pattern sites say common 50/50 locations are "isolated corners, end-of-wall segments, and detached mine groups". Frequency claims conflict: "15–30% of Expert boards" on one site versus "30-40 percent depending on the opening rules" on another. Treat them as unreliable. — [minesweeper.now FAQ](https://minesweeper.now/faq/minesweeper-50-50); [Minesweeper Blast](https://minesweeperblast.pages.dev/minesweeper-fifty-fifty)

### Inferences
- Templates follow from the JSMinesweeper definition: two cells A, B sharing exactly one mine, where every cell that could witness A also witnesses B identically. Concretely:
  - **Edge 50/50:** A, B adjacent along a board edge with the off-board side giving no information. The two tiles beyond them (i−1, i+2 columns) are mines or are themselves never revealable.
  - **Corner 2x2/2 box:** a 2x2 block in a corner with 2 mines on a diagonal, sealed by mines or walls.
  - **Extended chain:** an edge corridor of width 2 whose numbers cannot distinguish the two parities of mine placement.

  To embed one deliberately: place the template mines, fill the rest randomly, and verify with an exact checker that both configurations are globally consistent and indistinguishable. FiftyFiftyHelper's `countSolutions` with forced cells does exactly this check. (Inference from the code, not a documented generator.)
- Tatham's "hidden clue" warning applies in reverse to a max-guess generator. Embedding a fixed small set of trap templates would teach players where the traps are. Varying the templates, and verifying traps with an exact checker rather than drawing them from a catalog, reduces this.
- A forced 50/50 only counts as "forced" if the global mine count cannot break it. JSMinesweeper's `minesLeft`-dependent pattern start index shows that late-game counting can collapse candidate patterns.
- Simple independent 50/50s multiply. k independent embedded true 50/50s give a perfect-play win probability ≤ 2^−k, the cleanest measurable knob for a static max-guess board. (Inference.)

### Gaps
- I found no authoritative, exhaustive catalog of unavoidable guess shapes, such as a proven list of minimal forcing configurations. minesweeper.info/wiki was seen only as a search snippet.

## 3. Search-based generation with a solver in the loop

### Takeaway
I still found **no published paper or repo** that runs hill climbing, simulated annealing or a GA to *maximise forced guesses in play*. The closest new find is hellpig/minesweeper-puzzle-generator's `gen_sat_evolve.cpp` (2026). It is an evolutionary search that maximises the *deductive* difficulty of all-clues-visible ("Tentaizu-like") Minesweeper puzzles, scored by SAT-solver propagation counts.

Existing play-board generators otherwise go the other way and maximise solvability:
- **Rejection sampling:** Mindsweeper, JSMinesweeper "No Guess", Explainable Minesweeper, and in reverse Evil Mineswifter.
- **Local perturbation:** Tatham's `mineperturb`.

Tatham's quasiblog confirms his loop is a heuristic local search ("discrete but imprecise"), so it remains the closest template for an inverted-objective guess maximiser.

### Cited Findings
- **Tatham's own account of the generator** (quasiblog, 2019-08-26), now read in full:
  - **Core:** "the code has to include an actual solver, which simulates a human playing the game, and performs the same types of deduction that the human can use".
  - **Perturbation:** "If the solver gets stuck part way through solving the grid, I don't just throw the whole thing away and try again from scratch. Instead, I rearrange mines in the parts of the grid the solver hasn't opened up yet, so as to make sure at least one new deduction can be made at the current frontier."
  - **No safety guarantee:** "I don't make any effort to completely guarantee that the perturbations I make are safe", so after a perturbed run "I run the solver again, from the same starting point, and see if it can get all the way through this time without getting stuck!"
  - **Restart rule:** "as long as each run of the solver requires fewer perturbations to the grid, we consider that things are still improving, but if a run through finds more problems than the previous run, we conclude that we're stuck in a local optimum".
  - **Endgame:** "if the last 2 or 4 squares have a problem ... I just have to pitch the mine randomly into the already-opened area and hope it doesn't cause trouble."
  - **Outcome:** "default settings with over twice the mine density of the Windows standard ones".

  — [Tatham quasiblog](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/mines-solver/)
- **Upstream `mines.c`** (@616da16, 2026-09-22). The mirror's description still holds, with these differences:
  - `minesolve` still returns "-1 means deduction stalled ... 0 means deduction succeeded fully ... >0 ... the exact return value is the number of perturb calls" (L1956-1962).
  - [corrected after primary check] The perturb callback signature changed. The 2020 mirror had `(void *, signed char *, int x, int y, int mask)`, a 3×3 bitmask. Upstream has `(void *, signed char *, struct set *)` (L1965), because the 2026 rewrite supports arbitrary tilings ([commit d324fe9](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=d324fe9), mostly by Anders Höglund).
  - The `mineperturb()` preference list is unchanged (L2809-2816): "first, unknown squares on the boundary of known space; next, unknown squares beyond that boundary; as a very last resort, known squares, but not within one square of the starting position".
  - The whole-region perturb (`s == NULL`) is still gated by `allow_big_perturbs = (ntries > 100)` (L2766, L3271), because such grids "tend to be a bit boring, with mines packed densely into far corners".
  - [corrected after primary check] The give-up test is `solveret < 0 || (prevret >= 0 && solveret >= prevret)` (L3281, identical in the mirror at L1822). It abandons the grid when the perturb count fails to *strictly decrease*, not only when it "goes up" as the comment and the first-pass note said.
  - **New since 2020 (commit a7c7826, 2025-07-29), a "progress" counter.** `nperturbs_since_last_new_open` counts perturbations since a never-before-opened square was opened. `mineperturb` refuses to perturb further once it reaches the board width or height (L2773-2780). This fixed a reproducible hang (`mines --generate 1 9x15n55X1Y1#2-94`) in which the perturber cycled between filling and emptying the cleared area.
  - Tatham's commit message: "I'm not at all happy with this fix ... what I _ought_ to do is rethink the entire strategy of perturbation, in the large-scale case where we've run out of perturbations we can make within a single 3×3 block". A double-increment bug in that counter was fixed in fc9c4ab (2026-08-31), which "changes some mine generation outputs for the same random seed".
  - Other generator-relevant upstream changes:
    - "X0Y5" parameters fix the first-click location for reproducible `--generate` runs (7799786, 2025-07-29).
    - The mine-count validation was removed (7c79620, 2026-08-31). Counts above n−9 are accepted, and unplaced mines are reported as "+nnn" in the game description.
    - Mine counts can be stored as a percentage of *eligible* tiles, meaning tiles that are not neighbours of the start (d385acc, 2026-09-03).

  — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e); commits [a7c7826](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=a7c7826), [fc9c4ab](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=fc9c4ab), [7799786](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=7799786), [7c79620](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=7c79620), [d385acc](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=d385acc)
- [corrected after primary check] The 2005 claim, now verified in git history (commit 739609c, 2005-05-31; previously snippet-only): "when the Mines unique grid generator fails at high mine densities it is _almost always_ for the same reason ... here's an enhancement to mineperturb() which enables Mines to generate a grid at (as far as I can tell) any mine density you like, up to and including w*h-9 mines. At densities of 1 in 2 or thereabouts the grids start to look rather strange". The original check-in (6b9e690, 2005-05-30) reads "uses a solver during grid generation to arrange a mine layout that never requires guessing." — [upstream git history](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=739609c)
- **hellpig/minesweeper-puzzle-generator** (new find; commit 43a624d, 2026-07-16):
  - **Goal:** "my code attempts to generate the hardest possible puzzles". The puzzles have "all clue numbers exposed before any clicks. There is one unique solution."
  - **Method:** `gen_sat_evolve.cpp` was "inspired by how tdoku evolves Sudoku puzzles". It fixes one mine arrangement, minimises clues for uniqueness, keeps only puzzles that "the one-number/two-number solver cannot finish", mutates by restoring clues "mainly inside a local region" (9×9), re-minimises, and keeps a pool of the hardest.
  - **Parameters:** `numberOfGenerations = 100`, `poolSize = 30`, `parentsPerGeneration = 10`, `childrenPerParent = 8`.
  - **Fitness:** fresh CaDiCaL propagation counts (first solve plus uniqueness proof), which the author found "correlate very strongly with the old recursive solver's first-solution depth".
  - **Author's caveats:** "certain regions of the final puzzle are still relatively easy". "occasionally evolving the underlying mine arrangement instead of only the exposed clues may unlock truly diabolical puzzles". The README says the C++ ports were written with ChatGPT's help and the evolutionary version is not "tested ... deeply".

  — [hellpig/minesweeper-puzzle-generator README](https://github.com/hellpig/minesweeper-puzzle-generator); [gen_sat_evolve.cpp](https://github.com/hellpig/minesweeper-puzzle-generator/blob/main/gen_sat_evolve.cpp)
- **Explainable Minesweeper** (sublevelgames, 2025), rejection sampling with start-cell retries:
  - Steps: generate a random board, list candidate starts (cells of value 0, 1 or 2, lowest first), run the logic solver from each, and keep the board if any start solves it.
  - Cost: "finding boards solvable only through logical deduction required about 250,000 attempts per level on a 9x9 board with 33 mines".

  — [sublevelgames blog](https://sublevelgames.github.io/blogs/2025-07-06-making-explainable-minesweeper/)
- Mindsweeper `LocalGame::new()` and JSMinesweeper "No Guess" ("Attempts to generate a board which contains no guesses. The board is generated on your first click.") are both solver-gated. Evil Mineswifter inverts the gate: it rejects boards the solver *can* finish — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs); [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper); [Lutris: Evil Mineswifter](https://lutris.net/games/evil-mineswifter/)
- **Cicvárek, "Algorithms for Minesweeper Game Grid Generation"** (CTU Prague bachelor thesis, 2017), now read:
  - Contribution: he implements "an algorithm that can generate a minefield that is always solvable and introduce[s] new difficulty rating system that can also be used as an input for this new minefield generator".
  - The rating is "rank", the maximal number of variables that must be assumed in one step (see §4).

  — [Cicvárek 2017 PDF](https://minesweepergame.com/math/algorithms-for-minesweeper-game-grid-generation-2017.pdf)
- An analogous published technique from another game: Angry Birds level generation uses a fitness based on an agent's observed solve rate against a target solve rate. (snippet only) — [arXiv 1902.02518](https://arxiv.org/pdf/1902.02518)
- A 2025 University of Tartu thesis, "Time Complexity of Generating Solvable Boards for Minesweeper", concerns solvable-board generation (search snippet; dspace.ut.ee blocked) — [UT DSpace](https://dspace.ut.ee/items/f37ff00b-5db3-4101-b93d-96b48acab9a6/full)

### Inferences (speculative design, clearly not from a source)
- **Inverted Tatham loop:** run a deterministic logic solver. Wherever it makes a deduction, perturb mines in that constraint set so the deduction disappears. Accept the move if the objective improves. The objective can be the number of forced guesses on the solver's path, or −log of the product of guess survival probabilities. Tatham's own experience gives three warnings for this design:
  - the "fill/empty the whole unknown region" fallback can cycle;
  - progress must count *newly* opened squares (his `nperturbs_since_last_new_open`);
  - a perturbation can silently break earlier deductions, so the final board needs a full re-solve from the start (his "run the solver again" step).
- **hellpig's evolutionary loop** is the nearest working metaheuristic. Swapping its genome (clue visibility → mine layout plus fixed first click) and its fitness (SAT propagations → forced-guess count or −log₂ S from a play-through solver) would give a max-guess GA. The author himself suggests evolving the mine arrangement.
- **Fitness choices:** cheap = forced-guess count from a deterministic logic-only solver (e.g. a Tatham-style set solver). Expensive = Monte-Carlo win rate of a strong probabilistic solver (e.g. JSMinesweeper-style) over many seeds and first clicks. Rejection sampling (Evil Mineswifter style) needs no search but cannot steer *how many* guesses a board needs.
- Fix the first-click policy (e.g. a guaranteed opening at a chosen cell, as Tatham's new X/Y parameters allow) while optimising. Otherwise the objective is ill-defined, because guesses depend on the opening.

### Gaps
- I found no academic work on adversarial or max-guess Minesweeper generation by metaheuristics. Searches in this pass ("maximize guesses", "simulated annealing", "worst case" generator) surfaced only no-guess generators, solvers and hellpig's deduction-difficulty evolver.
- The Tartu thesis was unreadable (host blocked).

## 4. Measurable targets for "guessiness"

### Takeaway
Concrete, sourced metrics exist:
- guesses per won game (JSMinesweeper reports 3.3 on classic expert)
- solver win rate over many games (41% / 54.3% for expert variants)
- the perturbation count from Tatham's `minesolve`
- the exact best-play win probability from brute-force analysis
- counts of dead or 50/50 tiles

Two *deduction-difficulty* metrics are newly verified: Cicvárek's "rank" (variables assumed in the cheapest step) and Dempsey and Guinn's maximum GMUS core size. hellpig uses SAT propagation counts. Mine density itself matters: inference hardness peaks near ρ ≈ 0.2–0.3. Entropy remaining after deduction is a natural extension that I found no source using.

### Cited Findings
- "To win a classic expert game of minesweeper the solver is required, on average, to make 3.3 guesses… For higher density boards the number of guesses required goes up". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Win rates: "The Java solver has a 41% win rate on classic expert (30x16/99 safe start in a corner) and 54.3% on modern expert (30x16/99 open start at (3,3)). The Javascript version is slightly weaker." The first-click rule alone moves the win rate by about 13 points. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Exact best-play win probability: brute-force analysis "(conceptionally) plays every combination of move against every combination of mines… It also allows the solver to say precisely how likely the game is to be won with best play". It is attempted "when the number of solutions remaining is 750 or less". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Tatham's `minesolve` return value is the number of perturb calls, a built-in measure of how many times pure deduction stalled. It is unchanged upstream. Tatham describes using it as the improvement signal: "as long as each run of the solver requires fewer perturbations ... things are still improving". — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e); [Tatham quasiblog](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/mines-solver/)
- The JSMinesweeper guessing heuristic blends "Progress percentage" and "Secondary safety". Candidate tiles are those "within 10% of the safest guess". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- **Rank** (Cicvárek 2017): "Rank is the maximal necessary amount of variables on which we have to apply our assumptions when making one step during a single game". In contrast to 3BV, "rank doesn't consider the number of steps needed". "The increase [in difficulty from rank 1 to rank 2] is exponential and higher rank steps are thus very hard to detect." — [Cicvárek 2017 PDF](https://minesweepergame.com/math/algorithms-for-minesweeper-game-grid-generation-2017.pdf)
- **GMUS core size and the phase transition** (Dempsey & Guinn, arXiv 2008.04116, 2020):
  - The fraction of mines flaggable by inference falls "to 0 roughly in the interval 0.2 < ρ < 0.3, and the decline is steeper at higher values of N".
  - The average maximum grouped-MUS core size "is strongly peaked in the vicinity of the phase transition, and the peak core size grows with the size of the lattice ... we clearly see the easy-hard-easy pattern".
  - "This suggests that the hardest Minesweeper instances arise at the critical mine density."
  - A percolation estimate puts opening-cluster connectivity near ρ ≈ 0.1.

  — [arXiv 2008.04116](https://arxiv.org/pdf/2008.04116)
- **SAT propagation count** as a difficulty proxy: in hellpig's evolver, ranking by "fresh raw-SAT propagation count" correlated "very strongly" with recursive-solver depth in the author's tests — [gen_sat_evolve.cpp](https://github.com/hellpig/minesweeper-puzzle-generator/blob/main/gen_sat_evolve.cpp)
- A Harvard thesis evaluated solvers over 100,000 games across difficulties and densities. (snippet only) — [Harvard DASH](https://dash.harvard.edu/server/api/core/bitstreams/7312037d-80a6-6bd4-e053-0100007fdf3b/content)

### Inferences
Candidate metric set for a generator or benchmark (my synthesis):
1. **G** = number of forced guess points on a fixed solver's path, i.e. points where no tile has P(mine)=0. Use a fixed opening and a fixed tie-break.
2. **S** = ∏ over guesses of P(safe) of the chosen guess. −log₂ S is "bits of luck required". Each true 50/50 adds 1 bit.
3. **W** = Monte-Carlo win rate of the reference solver over N first-click and seed variations. Report it with a confidence interval. At p≈0.4, N=10,000 gives a standard error of about ±0.5 percentage points (binomial arithmetic, not from a source).
4. **H** = log₂(number of consistent configurations) remaining at each stall, or summed over stalls. This needs exact counting, which is #P-complete (see §5), so it is itself a load test.
5. Counts of dead tiles and of 50/50 / pseudo-50/50 structures, as detected by a FiftyFiftyHelper-like pass.
6. **R** = maximum rank / GMUS core size of the deductions made between guesses. This separates "guessy" (high G, low R) from "hard but fair" (low G, high R). A max-guess mode probably wants high G with low or moderate R.
7. **Density as a control knob:** near ρ ≈ 0.2–0.3 even exact inference resolves little, so G rises naturally. Classic expert (99/480 ≈ 0.206) already sits at the low edge of that band.

### Gaps
- I found no source that uses remaining entropy (H) as a published generator target, or that combines guess count with deduction rank.

## 5. Using such boards as solver load tests

### Takeaway
Exact Minesweeper reasoning is computationally hard:
- consistency is NP-complete;
- counting configurations is #P-complete;
- playing (inference) is co-NP-complete, per Dempsey & Guinn.

Exact probability engines are exponential in the size of a connected frontier component, and inference hardness peaks at the critical density. So the stress cases are boards with large connected frontiers and many near-indistinguishable configurations, ideally at density ≈ 0.2–0.3. Real solver authors bound the work explicitly with node or cycle caps, solution-count thresholds, component decomposition and caching.

### Cited Findings
- Consistency is NP-complete via Kaye's reduction. The counting version, #MINESWEEPER, is #P-complete (Nakov & Wei). Hardness persists in restricted cases; e.g. an MIT thesis shows that consistency restricted to {0,1} numbers is NP-complete. (snippet only for each) — [Minesweeper counting paper](https://mail.minesweepergame.com/math/minesweeper-minesweeper-2003.pdf); [MIT DSpace 1721.1/119557](https://dspace.mit.edu/handle/1721.1/119557)
- "Playing Minesweeper is known to be co-NP-complete." Above the critical density "it becomes almost impossible to play Minesweeper by logical inference". Polynomial k-set search matches a SAT solver at low density but falls behind near 0.2 < ρ < 0.3 — [Dempsey & Guinn, arXiv 2008.04116](https://arxiv.org/pdf/2008.04116)
- JSMinesweeper's probability engine: "The calculation is 100% accurate, but this comes at the price of exponential processing cost based on the number of revealed tiles which can be strung together… 50x50/500 should be fine, 100x100/2500 might struggle." — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- JSMinesweeper hard caps in `BruteForceAnalysis.js`: `BRUTE_FORCE_ANALYSIS_MAX_NODES = 100000000`, `BRUTE_FORCE_CYCLES_THRESHOLD = 75000000`, `BRUTE_FORCE_ANALYSIS_TREE_DEPTH = 6`. Solver toggles in `solver_main.js`: `PRUNE_GUESSES`, `EARLY_FIFTY_FIFTY_CHECKING`, `CALCULATE_LONG_TERM_SAFETY`. — [JSMinesweeper BruteForceAnalysis.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/BruteForceAnalysis.js); [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- Tatham's global mine-count deduction enumerates set combinations only when there are at most 10 constraint sets (`bool setused[10]`; `if (nsets <= lenof(setused))`, upstream L2211, L2290). This is a hard cap that skips the expensive tier on large frontiers — [upstream mines.c @616da16](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e)
- The JSMinesweeper Analyser lets users construct arbitrary positions and download/load MBF/ABF boards "to see how the solver would approach playing it". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Mindsweeper decomposes the frontier into independent `Component`s (`Analyzer::partition()`). It enumerates each component's arrangements by mine count and combines them with BigUint binomial weights. Its tests `win_all_games_with_punishment` / `win_all_games_without_punishment` are property-style regression tests over many generated games. — [mindsweeper analyzer.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/analyzer.rs); [local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)
- Kaboom's cost model, in the author's words:
  - Backtracking enumeration hit 18 million arrangements, 12 GB and 30 s per reveal.
  - The SAT totalizer is Θ(N²) clauses and "sluggish" near 100 open squares, with formulas up to about 10,000 clauses on big boards.
  - Model reuse cuts queries 2–5×, and caching removes forced cells.

  In code it times itself (`this.lastDuration`) and reports boundary size, uncached count and clause count in `debugMessage()` — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/); [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- [corrected after primary check] The 2025 Tatham commits are verified in git (previously snippet-only):
  - "Mines: add extra diagnostics for grid perturbation" (38b2991): "A bug I was investigating was still confusing until I made the perturber explain what it was doing in more detail".
  - "Mines: fix a hang during grid generation" (a7c7826), with a reproducible seed and the cycling failure described in §3.

  Even mature generators hit pathological cases, and a fixed-seed reproduction string is how Tatham debugs them — [commit 38b2991](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=38b2991); [commit a7c7826](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=a7c7826)
- hellpig validated its hybrid solvers by "making a version (not included) that runs both solvers and compares their solution counts", a differential test between a recursive solver and CP-SAT or CaDiCaL — [hellpig README](https://github.com/hellpig/minesweeper-puzzle-generator)

### Inferences
- Load-test classes worth generating (synthesis):
  - (a) **One giant connected frontier component.** Long snake-like revealed regions with numbers 1–3, maximising the "revealed tiles which can be strung together", at density about 0.2–0.3 where GMUS cores peak.
  - (b) **Many independent small components plus a large unconstrained interior.** This stresses the combination step, as in Mindsweeper's `compute_weights`.
  - (c) **Late-game near-ties** where the global mine count is the only discriminator. This stresses `minesLeft` handling and caps such as Tatham's 10-set limit.
  - (d) **Chains of extended 50/50s** with up to 2^k consistent configurations that are locally indistinguishable.
  - (e) **Boards just above a brute-force threshold**, e.g. 751+ remaining solutions for JSMinesweeper.
  - (f) **Fixed-seed regression boards** in the style of Tatham's `9x15n55X1Y1#2-94`, so a generator hang becomes a reproducible test case.
- Differential testing: run two independent exact engines (e.g. a SAT/z3 checker like Kaboom or gazpachoking versus a component-enumeration engine like Mindsweeper's) on the same adversarial positions. Assert identical forced-safe and forced-mine sets and identical configuration counts. hellpig describes this practice for solution counts.

### Gaps
- I found no published benchmark suite of "hard" Minesweeper positions or timing tables for exact engines beyond JSMinesweeper's qualitative note, Kaboom's anecdotes and Dempsey & Guinn's density curves.

## 6. Fairness and game design for a max-guess mode

### Takeaway
Existing designs converge on a few rules:
- never lose on the first click;
- punish only *avoidable* guesses;
- make forced guesses free or at least honestly random;
- restrict where guesses are allowed (Kaboom's frontier-only default, which its author admits is "somewhat artificial" and made optional);
- explain the loss afterwards.

The two existing "evil" designs show the alternatives. Evil Mineswifter guarantees a 50/50 on every board and offers Undo. andrewt names the remaining unfairness of cruel-but-fair adversaries: an inevitable 50/50 taken early is punished.

### Cited Findings
- Kaboom: forced guesses always succeed and avoidable guesses always fail. Outside-the-frontier guessing is punished by default, because friends found it "weird that it would guarantee you an empty spot anywhere". The author later called the restriction "somewhat artificial" and added a toggle. The hint text distinguishes safe / guessable / "All surrounding cells are deadly. You need to play elsewhere." Unlimited undo — [pwmarcz.pl: Kaboom](https://pwmarcz.pl/blog/kaboom/); [Kaboom index.js / index.html](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- gazpachoking makes the forgiveness level a dial (NICE / FAIR / NORMAL / CRUEL). The first click is forced safe. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- Mindsweeper: guessing is never necessary. The first click and its neighbours are protected. A post-mortem shows "which tiles you could (and could not) have safely revealed". — [Mindsweeper README](https://github.com/AlexBuz/mindsweeper)
- JSMinesweeper: it is "always correct to guess these [true 50/50s] at the earliest opportunity, since they might provide information to their neighbouring tiles". The "Opening on start" option chooses between a guaranteed opening and only a guaranteed-safe first click. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Evil Mineswifter:
  - "every board contains a 50/50 guess";
  - "Click on a mine due to a 50/50 guess? No problem! Simply click the Undo popover and keep playing until the next 50/50 guess";
  - the main stat is the "perfect game", completed "without mistakes or hints";
  - daily challenges contain "exactly one 50/50 guess".

  — [App Store: Evil Mineswifter](https://apps.apple.com/us/app/id1523249883)
- andrewt (Pure Skill Minesweeper): cruel-but-fair adversaries should "recognise when there's a forced guess _coming_". Otherwise taking a future 50/50 early "will always lose". Requiring the best-odds square "would be taking things too far" — [Pure Skill Minesweeper](https://github.andrewt.net/mines/)
- Tatham disables big whole-region perturbations at first because those grids are "a bit boring, with mines packed densely into far corners". In 2005 he noted that at densities around 1 in 2 "the grids start to look rather strange". He also warns that a generator constraint can become "an extra hidden clue to the mine positions for anyone who knows how the generator is working". — [upstream mines.c L2747-2753](https://git.tartarus.org/?p=simon/puzzles.git;a=blob;f=mines.c;hb=616da16f25da428ccda805536209095bd062676e); [commit 739609c](https://git.tartarus.org/?p=simon/puzzles.git;a=commit;h=739609c); [Tatham quasiblog](https://www.chiark.greenend.org.uk/~sgtatham/quasiblog/mines-solver/)

### Inferences
- For a max-guess mode (design suggestions, not sourced):
  - Make every forced guess a **true** 50/50, or a clearly-labelled ratio. Hidden 70/30 guesses that a player misjudges feel unfair.
  - Expose the guess count and the luck budget (−log₂ S) up front, e.g. "this board needs 4 coin-flips". Evil Mineswifter's daily "exactly one 50/50" is a precedent for announcing the count.
  - Guarantee a first-click opening.
  - Place traps where perfect play would guess them early (JSMinesweeper's "guess 50/50s ASAP" principle). If the mode is adversarial rather than static, honour andrewt's point: never punish taking an inevitable 50/50 early.
  - Offer Undo after a forced-guess loss, as both Kaboom and Evil Mineswifter do, and/or a Mindsweeper-style post-mortem showing that a loss was a forced 50/50, not an error.
  - Keep mine density away from the "packed corners" artefact by limiting how far the search may move mines from a random baseline. For example, cap the swap count, or add a penalty term for local density variance.
  - Avoid a small fixed trap catalog, which would become a "hidden clue" (Tatham).
- Uniform-sampling hygiene matters for perceived fairness. If the adversary or generator picks among consistent layouts, a biased choice (Kaboom-style "first SAT model") could produce recognisable, exploitable patterns. (Speculation.)

### Gaps
- I found no user-study or player-feedback data on how fun max-guess or adversarial modes are. The only reported reaction is Kaboom's: the outside-guess restriction was "controversial".

## Verification log

Second pass, 2026-11-06, with wider network access.

**Confirmed against primary sources**
- **Upstream Tatham Mines** (full git clone, HEAD 616da16, 2026-09-22) agrees with the 2020 mirror on:
  - the `minesolve` return-value contract (perturb count);
  - the `mineperturb` preference order;
  - the `allow_big_perturbs = (ntries > 100)` gate and its "boring ... far corners" rationale;
  - the give-up rule (same code line);
  - the three-tier solver;
  - `solve_game` behaviour.
- **Tatham's quasiblog** (2019-08-26) confirms the solver-plus-perturber design, the re-solve check and the "fewer perturbations = improving" restart rule.
- **Kaboom:** every code-level claim from pass 1 is consistent with the author's blog post (pwmarcz.pl, 2019-11-21), including the rules, the totalizer encoding, model reuse, caching and the outside-guess rule.
- **Evil Mineswifter's** rejection loop, now from Lutris and the developer's App Store text rather than a snippet.

**Corrected** (each marked "[corrected after primary check]" in the text)
- Evil Minesweeper (itch.io): its adversary is not implemented; it is currently a no-guess game.
- Pure Skill Minesweeper: a full cruel-but-fair adversary (it punishes avoidable guesses too), self-described as buggy, with an explicit "forced guess coming" limitation.
- The Kaboom "cruel but fair" phrase is the author's own, not GIGAZINE's.
- Tatham's give-up rule fires when perturbations fail to strictly decrease (`>=`), not only when they go up.
- Tatham's perturb callback signature changed from (x, y, 3×3 mask) to `struct set *` in the 2026 arbitrary-tiling rewrite.
- The 2005 "any mine density ... up to and including w*h-9" commit and the 2025 diagnostics/hang commits, verified in git history (previously snippet-only).

**Added (new primary sources)**
- Upstream Mines changes since 2020:
  - the `nperturbs_since_last_new_open` give-up (2025) and its double-increment fix (2026);
  - first-click X/Y parameters;
  - removal of mine-count validation;
  - percentage mine counts;
  - arbitrary tilings and wrapping grids.
- Kaboom design history and performance anecdotes.
- hellpig/minesweeper-puzzle-generator's evolutionary difficulty maximiser (the only metaheuristic Minesweeper generator found).
- Explainable Minesweeper's rejection cost (≈250,000 attempts per 9×9/33 level).
- Cicvárek 2017 "rank".
- Dempsey & Guinn 2020 phase transition (critical density ≈ 0.2–0.3; GMUS core size).
- Evil Mineswifter daily challenges with "exactly one 50/50".

**Still unverified**
- The Tartu 2025 thesis (dspace.ut.ee blocked by DNS).
- tartarus-commits mailing-list pages (proxy 502; the content was verified via git instead).
- The Angry Birds fitness paper (arXiv 1902.02518) and the Harvard DASH thesis: not re-fetched.
- The Kaye / Nakov & Wei / MIT complexity results: not re-fetched; still snippet-level.
- minesweeper.now and Minesweeper Blast 50/50 frequency claims (low authority).
- minesweeper.info/wiki.
- No source code exists publicly for Evil Mineswifter, so its "advanced Minesweeper AI" is unspecified.
