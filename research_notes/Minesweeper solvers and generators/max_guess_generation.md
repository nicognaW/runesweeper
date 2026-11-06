# Generating maximally guessy / adversarial Minesweeper boards, and using them as solver load tests

Research method note: I read the source of five projects directly by cloning them (Kaboom, gazpachoking/minesweeper, AlexBuz/mindsweeper, DavidNHill/JSMinesweeper, and a GitHub mirror of Simon Tatham's Puzzles `mines.c`). The commits I read are listed in the citations. Several primary hosts were **unreachable from this sandbox** (DNS blocked): pwmarcz.pl (Kaboom's blog post), chiark.greenend.org.uk (Tatham's "Writing a soluble-grid generator for Mines" quasiblog), minesweepergame.com (the math paper archive), github.andrewt.net (Pure Skill Minesweeper), gigazine.net and lutris.net. Claims that rest only on search-result snippets of those pages are labelled "(snippet only)". To verify those claims, the hosts would have to be allowed in the session's network settings.

## 1. Adversarial / dynamic-placement Minesweepers: how they place mines lazily

### Takeaway
Every adversarial variant I examined uses the same core mechanism. It keeps no fixed mine layout. On each click it asks a constraint solver (Kaboom: MiniSat compiled to JS; gazpachoking: z3; Mindsweeper: its own component enumerator) whether the clicked cell *can* be a mine and whether it *can* be safe, given every revealed number and the total mine count. It then commits to one full layout consistent with the answer it picked.

The "cruel but fair" rule works like this. If the cell can be a mine and a provably safe cell exists anywhere, the click becomes a mine. If no safe cell exists, any cell that can be safe is made safe. So the player loses only by guessing when they did not have to. They never lose to a forced guess.

### Cited Findings

**Kaboom (pwmarcz/kaboom, `index.js` + `sat.js`, commit 2c5cc17, 2021-08-01)**
- No mine grid is stored during play. The `Game` constructor has `//this.mines = makeGrid(...)` commented out. The state is only `this.map = new LabelMap(...)`, the revealed numbers. A concrete `mineGrid` exists only transiently inside `Game.reveal()` or at death/win. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `LabelMap.recalc()` builds the **boundary**: unrevealed cells adjacent to a revealed number, each given an index in `boundaryGrid`. It counts `numOutside` = all cells − revealed − boundary. It also caches trivially forced mines: when a label equals its count of unknown neighbours, those neighbours are `setCache(id, true)`. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `makeSolver(map, maxMines)` builds a `Solver` with one boolean variable per boundary cell. It sets `minMines = maxMines - map.numOutside`, since the outside cells can absorb at most `numOutside` mines. Each revealed label becomes `addLabel(label, mineList)`. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- The encoding is in `Solver.run()` and `sat.js`. Each label is encoded as an exactly-k constraint using `Sat.assertAtLeast` / `Sat.assertAtMost`. These expand to **all combinations** of size (n−k+1) and (k+1) via `combinations()`, a naive binomial encoding that is fine because n ≤ 8. The global mine count uses a recursive totalizer-style unary counter, `Sat._addCounter` (split in half, merge with clauses). `assertCounterAtLeast/AtMost` then bound it. — [Kaboom sat.js](https://github.com/pwmarcz/kaboom/blob/master/sat.js)
- For each uncached boundary cell i, `Solver.run()` makes up to two SAT calls, `solveWith(assert ¬x_i)` and `solveWith(assert x_i)`, to fill `_canBeSafe[i]` and `_canBeDangerous[i]`. An optimisation, `Solver.update(solution)`, marks every variable's observed polarity in each returned model as possible. This skips many later SAT calls. Results feed the cache (`map.setCache`) so later turns treat forced cells as constants. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- **The "cruel but fair" decision is in `Game.reveal(x, y)`, verbatim logic:**
  - Boundary click: `if (canBeSafe(idx) && (!canBeDangerous(idx) || !hasSafeCells)) shape = anySafeShape(idx); else shape = anyDangerousShape(idx);`. So a cell that can be a mine *is* a mine whenever `hasSafeCells()` is true, meaning some boundary cell has `!canBeDangerous`.
  - Outside (non-boundary) click with the default `allowOutside=false`: the outside is treated as safe only if `boundary.length === 0 || outsideIsSafe() || !hasNonDeadlyCells()`. You may only guess outside the frontier when every boundary cell is a certain mine.
  - With `allowOutside=true`: `outsideIsSafe() || (!hasSafeCells && outsideCanBeSafe())`. Outside guesses are forgiven whenever no safe cell exists.
  - Safe outcome: `anyShapeWithOneEmpty().mineGridWithEmpty(x,y)`. Mine outcome: `anyShapeWithRemaining().mineGridWithMine(x,y)`. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- `Shape` turns one SAT model into a full board. `baseMineGrid()` places boundary mines. `addRandom()` shuffles the outside cells and scatters the `remaining` mines among them. `floodReveal()` then computes real numbers from that transient grid and cascades on zeros. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- UI options in `index.html`: "Allow guessing anywhere (By default, you are allowed to guess only on the boundary of revealed tiles.)", "Safe mode (double-click to reveal)", "Countdown mode", and Debug mode. Kaboom also has unlimited `undo()`, which pops reveals and calls `map.resetCache()`. A `hint()` message distinguishes "There are safe cells", "no safe cells, but you can guess", and "All surrounding cells are deadly. You need to play elsewhere." Presets go up to 10x10 with 50 mines ("Ultra-Violence"). — [Kaboom index.html](https://github.com/pwmarcz/kaboom/blob/master/index.html)
- Description by third parties: the author describes it as "a very cruel but fair game". Guessing is punished immediately unless no other move exists. (snippet only) — [GIGAZINE](https://www.gigazine.net/gsc_news/en/20191227-kaboom)

**gazpachoking/minesweeper ("undetermined", Python + z3, commit d08da5d, 2025-07-12)**
- `NiceMode` enum in `undetermined/__init__.py`, comments verbatim: NICE "Any click that could result in an empty tile is an empty tile."; FAIR "If guessing is the only move available, you will not guess wrong."; NORMAL "Traditional minesweeper."; CRUEL "Any click that could result in a mine is a mine. (Except when guessing is the only move available.)" The README credits Kaboom's "Cruel but fair" idea. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- `Board.solver()` builds a z3 solver with `z3.PbEq` (pseudo-Boolean exactly-k) for the global count over all undetermined tiles and for each revealed boundary number. Its comment reads: "Cheat a bit to randomize possible solution by adding constraints in a random order". The shuffle randomises which model z3 returns. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- `Board.recalc()` "locks in" tiles via `solver.check(t.var)` and `solver.check(z3.Not(t.var))`, two checks per tile. `Board.reveal()` encodes the fairness rule. If `not safe_moves and (not boundary_moves or tile in boundary_moves)`, the tile is forced safe. Otherwise, in CRUEL mode, the tile is forced to be a mine. `replace_mines()` then re-samples a full consistent layout. The first click is always forced safe (`tile.mine = False`). — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)

**AlexBuz/mindsweeper (Rust/WASM, commit b804faa, 2023-12-29): the "no-guess + punish" opposite pole**
- README: it chooses "a random mine arrangement from the set of mine arrangements that would allow a perfect logician to win without guessing … sample uniformly". Guess punishment: "If you click on a tile that *can* be a mine, then it *will* be a mine, guaranteed." — [Mindsweeper README](https://github.com/AlexBuz/mindsweeper)
- `LocalGame::new()` in `src/server/local.rs` does **rejection sampling**. It shuffles mines, protects the first click and its neighbours, then auto-plays `analyzer.find_safe_moves(false)` until stuck. It `continue`s (retries) unless the board is won by pure deduction. — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)
- `LocalGame::punish()` and `rearrange_mines()`: if the clicked tile `may_be_mine()`, the code restricts that tile's component's arrangements to those containing it. If the tile is unconstrained, it pretends `known_mine_count += 1`. `compute_weights()` then weights each per-component mine-count combination by (product of arrangement counts) × `big_binomial(unconstrained, remaining)` using `BigUint`. It picks via `choose_weighted`, so the punished layout is a **uniform** sample of consistent layouts. `punish_chord()` does the same for chords. — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)

**Other "evil" variants (lower confidence)**
- "Evil Minesweeper" (itch.io, WIP): "All the mines are adversary, rearranging if you ever make a choice that's not a valid logical deduction." (snippet only) — [Evil Minesweeper](https://lightbulbmeow.itch.io/evil-minesweeper)
- "Evil Mineswifter" goes the other way: a static board with forced guesses. It "secretly runs an advanced Minesweeper AI to see if the board can be solved without guessing, and if it solves the board, it randomly generates another board and tries again". (snippet only; I could not fetch the page) — [Lutris: Evil Mineswifter](https://lutris.net/games/evil-mineswifter/)
- "Pure Skill Minesweeper" (andrewt) is described as a version "where you will never be penalised when forced to guess". Its author recommends Kaboom. (snippet only) — [Pure Skill Minesweeper](https://github.andrewt.net/mines/)

### Inferences
- Kaboom does **not** sample uniformly. Whichever model MiniSat returns from `anySafeShape` / `anyDangerousShape` becomes the boundary layout, and only the outside mines are shuffled. gazpachoking only randomises constraint order. Mindsweeper's `compute_weights` is the only one I read that samples uniformly over consistent layouts with exact counts. For a "max-guess" mode that must look like a fair random board after the fact, Mindsweeper's weighting approach is the model to copy.
- The adversary's rule reduces to two per-cell booleans, `canBeSafe` and `canBeDangerous`, plus one global `hasSafeCells`. Any solver that computes exact "forced safe / forced mine" sets, the same information a probability engine produces as P=0 / P=1, can drive this mode. Full probabilities are not needed.
- Kaboom never makes a *forced* guess fail. A "maximise forced guessing" generator would need a separate, nastier adversary. One version: at forced-guess points, make the click safe but choose the layout (among consistent ones) that leaves the fewest safe deductions, i.e. that creates the next forced guess soonest. (Speculation: I found no implementation of this "lazy max-guess" policy.)

### Gaps
- I could not read Kaboom's blog post (pwmarcz.pl blocked), so the author's own wording of the rules and any design rationale (e.g. why outside guesses are disallowed by default) is unverified beyond the source code.
- I found no source code for "Evil Minesweeper" (itch) or "Evil Mineswifter", so their algorithms are known only from one-line descriptions.

## 2. Static boards that force guesses: forced 50/50 structures and how to embed them

### Takeaway
The most precise, code-backed catalog of unavoidable-guess structures I found is in DavidNHill/JSMinesweeper. It names "dead tiles", arbitrarily extended 2-tile 50/50s, enclosed 2x2 boxes holding 2 mines, and 2-tile and 2x2 "pseudo-50/50s", and it ships detection code. These structures can be embedded deliberately by placing the mines and then checking with the same detector. I found no authoritative catalog beyond this. Pattern sites are low-authority.

### Cited Findings
- JSMinesweeper README definition: "A 2-tile 50/50 is a pair of tiles which share 1 mine where one of the tiles can never receive information without the other tile also receiving the same information… they must be (and always will be) an unavoidable guess". The solver detects "Arbitrarily extended 2-tile 50/50s" and "Enclosed 2x2/2 boxes". A *pseudo-50/50* is "a set of tiles [that] is either safe or part of a 50/50". It detects "2-tile pseudo-50/50s" and "4-tile 2x2 pseudo-50/50s". It is "unable to discover extended pseudo-50/50s". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- "Dead tiles": tiles that "are either mines or have only one possible value… it is never correct to choose a dead tile unless all the tiles left in the game are dead. In which case… the result is down to luck." — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- How `FiftyFiftyHelper.process()` (`Minesweeper/client/FiftyFiftyHelper.js`) detects a horizontal 2-tile 50/50: for adjacent covered tiles (i,j),(i+1,j), it rejects the pair if `isPotentialInfo` holds for any of the six tiles at columns i−1 and i+2 (rows j−1..j+1). It then tests whether "both mines" is a valid option via `solver.countSolutions` with both tiles set as found bombs. For 2x2 boxes it enumerates all mine patterns in the static `PATTERNS` table (4, 3, 2 and 1 mines). The starting index depends on `minesLeft` (>3 → 0, =3 → 1, =2 → 5, else 9). It tracks `risky[l]` to tell a true box 50/50 from a pseudo-50/50, and skips candidates where `allDead`. — [JSMinesweeper FiftyFiftyHelper.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/FiftyFiftyHelper.js)
- `LongTermRiskHelper.js` estimates per-tile "50/50 influence": aggregate 2-tile 50/50 blast percentage, highest-impact 4-tile 50/50, and "enabler" influence for tiles that would complete a 50/50. It only considers "possible 50/50s with less than 3 missing mines". — [JSMinesweeper LongTermRiskHelper.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/LongTermRiskHelper.js)
- Low-authority pattern sites say common 50/50 locations are "isolated corners, end-of-wall segments, and detached mine groups". Frequency claims conflict: "15–30% of Expert boards" on one site versus "30-40 percent depending on the opening rules" on another. Treat them as unreliable. — [minesweeper.now FAQ](https://minesweeper.now/faq/minesweeper-50-50); [Minesweeper Blast](https://minesweeperblast.pages.dev/minesweeper-fifty-fifty)

### Inferences
- Templates follow from the JSMinesweeper definition: two cells A, B sharing exactly one mine, where every cell that could witness A also witnesses B identically. Concretely:
  - **Edge 50/50:** A, B adjacent along a board edge with the off-board side giving no information. The two tiles beyond them (i−1, i+2 columns) are mines or are themselves never revealable.
  - **Corner 2x2/2 box:** a 2x2 block in a corner with 2 mines on a diagonal, sealed by mines or walls.
  - **Extended chain:** an edge corridor of width 2 whose numbers cannot distinguish the two parities of mine placement.

  To embed one deliberately: place the template mines, fill the rest randomly, and verify with an exact checker that both configurations of the structure are globally consistent and indistinguishable. FiftyFiftyHelper's `countSolutions` with forced cells does exactly this check. (Inference from the code, not a documented generator.)
- A forced 50/50 only counts as "forced" if the global mine count cannot break it. JSMinesweeper's `minesLeft`-dependent pattern start index shows that late-game mine counting can collapse candidate patterns, so embedded traps should not rely on structures that become decidable by counting at the end.
- Simple independent 50/50s multiply. k independent embedded true 50/50s give a perfect-play win probability ≤ 2^−k. That is the cleanest "measurable" knob for a static max-guess board. (Inference.)

### Gaps
- I found no authoritative, exhaustive catalog of unavoidable guess shapes, such as a proven list of minimal forcing configurations. minesweeper.info/wiki was only seen as a search snippet.
- I could not read Tatham's quasiblog on what his solver can and cannot deduce (host blocked).

## 3. Search-based generation with a solver in the loop

### Takeaway
I found **no published paper or repo** that runs hill climbing, simulated annealing or a GA to *maximise* forced guesses or minimise solver win rate in Minesweeper. Existing solver-in-the-loop generators go the other way: they maximise solvability. They do it by rejection sampling (Mindsweeper, JSMinesweeper "No Guess", and in reverse Evil Mineswifter) or by local perturbation (Tatham's `mineperturb`). Tatham's perturbation loop is the closest ready-made template for a local-search "guess maximiser" with its objective inverted.

### Cited Findings
- Tatham `mines.c` (GitHub mirror ghewgill/puzzles, commit 01c3e38, 2020-09-21): `minesolve(w,h,n,grid,open,perturb,ctx,rs)` returns "-1 means deduction stalled… 0 means deduction succeeded fully… >0 means… the exact return value is the number of perturb calls". When stuck: "Our only remaining option is to call a perturb function and ask it to modify the grid to make it easier". It picks a random remaining constraint set (`index234(ss->sets, random_upto(...))`) and calls `perturb(ctx, grid, s->x, s->y, s->mask)`. — [ghewgill/puzzles mines.c](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- `mineperturb()` builds a preference-ordered list of squares: "first, unknown squares on the boundary of known space; next, unknown squares beyond that boundary; as a very last resort, known squares, but not within one square of the starting position". The whole-unknown-region perturbation (`mask==0`) is gated by `allow_big_perturbs`, which `minegen()` sets only when `ntries > 100`. The reason: such grids "tend to be a bit boring, with mines packed densely into far corners". `minegen()` gives up a candidate "if the number of perturb steps ever goes up or it ever returns -1". — [ghewgill/puzzles mines.c](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- A 2005 commit says the `mineperturb()` enhancement lets Mines generate grids "at (as far as I can tell) any mine density you like, up to and including w*h-9 mines". (snippet only) — [tartarus-commits 2005-05](https://lists.tartarus.org/pipermail/tartarus-commits/2005-May/008179.html)
- Mindsweeper `LocalGame::new()` and JSMinesweeper "No Guess" ("Attempts to generate a board which contains no guesses. The board is generated on your first click.") are both solver-gated. — [mindsweeper local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs); [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Analogous published technique from another game: Angry Birds level generation uses a fitness based on an agent's observed solve rate against a target solve rate. (snippet only) — [arXiv 1902.02518](https://arxiv.org/pdf/1902.02518)
- A 2017 thesis, "Algorithms for Minesweeper Game Grid Generation", targets no-guess boards and proposes a difficulty rating. I could not read it (host blocked). — [minesweepergame.com PDF](https://minesweepergame.com/math/algorithms-for-minesweeper-game-grid-generation-2017.pdf)

### Inferences (speculative design, clearly not from a source)
- **Inverted Tatham loop:** run a deterministic logic solver. Wherever it makes a deduction, perturb mines in that constraint set so the deduction disappears. Accept the move if the objective improves. The objective can be the number of forced guesses on the solver's path, or −log of the product of guess survival probabilities. This is ordinary hill climbing or simulated annealing over mine layouts, with moves = "swap one mine with one empty cell" (keeps the count fixed).
- **Fitness choices:** cheap = forced-guess count from a deterministic logic-only solver (e.g. Tatham-style set solver). Expensive = Monte-Carlo win rate of a strong probabilistic solver (e.g. JSMinesweeper-style) over many seeds and first clicks. Rejection sampling (Evil Mineswifter style) needs no search but cannot steer *how many* guesses a board needs.
- Fix the first-click policy (e.g. a guaranteed opening at a chosen cell) while optimising. Otherwise the objective is ill-defined, because guesses depend on the opening.

### Gaps
- I found no academic work on adversarial or max-guess Minesweeper generation by metaheuristics. The GA hits in search were GA *solvers* (e.g. PatrickGhadban/Minesweeper-AI), not generators.

## 4. Measurable targets for "guessiness"

### Takeaway
Concrete, code-backed metrics exist: guesses per won game (JSMinesweeper reports 3.3 on classic expert), solver win rate over many games (41% / 54.3% for expert variants), the perturbation count from Tatham's `minesolve`, the exact best-play win probability from brute-force analysis, and the counts of "dead" or 50/50 tiles. Entropy-remaining-after-deduction is a natural extension. I found no source that uses it.

### Cited Findings
- "To win a classic expert game of minesweeper the solver is required, on average, to make 3.3 guesses… For higher density boards the number of guesses required goes up". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Win rates: "The Java solver has a 41% win rate on classic expert (30x16/99 safe start in a corner) and 54.3% on modern expert (30x16/99 open start at (3,3)). The Javascript version is slightly weaker." The first-click rule alone moves the win rate by about 13 points. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Exact best-play win probability: brute-force analysis "(conceptionally) plays every combination of move against every combination of mines… It also allows the solver to say precisely how likely the game is to be won with best play". It is attempted "when the number of solutions remaining is 750 or less". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Tatham `minesolve` return value = number of perturb calls. This is a built-in measure of how many times pure deduction stalled. — [ghewgill/puzzles mines.c](https://github.com/ghewgill/puzzles/blob/master/mines.c)
- The JSMinesweeper guessing heuristic blends "Progress percentage" (chance the move leads to 100%-safe moves next) and "Secondary safety". Candidate tiles are those "within 10% of the safest guess". — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- A Harvard thesis evaluated solvers over 100,000 games across difficulties and densities. (snippet only) — [Harvard DASH](https://dash.harvard.edu/server/api/core/bitstreams/7312037d-80a6-6bd4-e053-0100007fdf3b/content)

### Inferences
Candidate metric set for a generator or benchmark (my synthesis):
1. **G** = number of forced guess points on a fixed solver's path, i.e. points where no tile has P(mine)=0. Use a fixed opening and a fixed tie-break.
2. **S** = ∏ over guesses of P(safe) of the chosen guess. −log₂ S is "bits of luck required". Each true 50/50 adds 1 bit.
3. **W** = Monte-Carlo win rate of the reference solver over N first-click and seed variations. Report it with a confidence interval. At p≈0.4, N=10,000 gives a standard error of about ±0.5 percentage points (binomial arithmetic, not from a source).
4. **H** = log₂(number of consistent configurations) remaining at each stall, or summed over stalls. This needs exact counting, which is #P-complete (see §5), so it is itself a load test.
5. Counts of dead tiles and of 50/50 / pseudo-50/50 structures, as detected by a FiftyFiftyHelper-like pass.

### Gaps
- I found no source that uses remaining entropy (H) as a published generator target.

## 5. Using such boards as solver load tests

### Takeaway
Exact Minesweeper reasoning is NP-complete (consistency) and #P-complete (counting configurations). Exact probability engines are exponential in the size of a connected frontier component. So the stress cases are boards with large connected frontiers and many near-indistinguishable configurations: wide 50/50 chains and dense, long, linked frontiers. Real solver authors bound the work explicitly with node or cycle caps, solution-count thresholds and component decomposition.

### Cited Findings
- Consistency is NP-complete via Kaye's reduction. The counting version, #MINESWEEPER, is #P-complete (Nakov & Wei). Hardness persists in restricted cases; e.g. an MIT thesis shows that consistency restricted to {0,1} numbers is NP-complete. (snippet only for each) — [Minesweeper counting paper](https://mail.minesweepergame.com/math/minesweeper-minesweeper-2003.pdf); [MIT DSpace 1721.1/119557](https://dspace.mit.edu/handle/1721.1/119557)
- JSMinesweeper's probability engine: "The calculation is 100% accurate, but this comes at the price of exponential processing cost based on the number of revealed tiles which can be strung together… 50x50/500 should be fine, 100x100/2500 might struggle." — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- JSMinesweeper hard caps in `BruteForceAnalysis.js`: `BRUTE_FORCE_ANALYSIS_MAX_NODES = 100000000`, `BRUTE_FORCE_CYCLES_THRESHOLD = 75000000`, `BRUTE_FORCE_ANALYSIS_TREE_DEPTH = 6`. Solver toggles in `solver_main.js`: `PRUNE_GUESSES`, `EARLY_FIFTY_FIFTY_CHECKING`, `CALCULATE_LONG_TERM_SAFETY`. — [JSMinesweeper BruteForceAnalysis.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/BruteForceAnalysis.js); [solver_main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/solver_main.js)
- The JSMinesweeper Analyser lets users construct arbitrary positions (toggle revealed tiles, set numbers, set the mine count). Users can download/load MBF/ABF boards "to see how the solver would approach playing it". This is the author's mechanism for hand-built edge-case testing. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Mindsweeper decomposes the frontier into independent `Component`s (`Analyzer::partition()`). It enumerates each component's arrangements by mine count (`find_possible_mine_arrangements_by_mine_count`) and combines them with BigUint binomial weights. `find_safe_moves(exhaustive)` first tries per-component analysis, then the global mine distribution. Its tests `win_all_games_with_punishment` / `win_all_games_without_punishment` in `local.rs` are property-style regression tests over many generated games. — [mindsweeper analyzer.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/analyzer.rs); [local.rs](https://github.com/AlexBuz/mindsweeper/blob/main/src/server/local.rs)
- Kaboom's cost model is two SAT calls per boundary cell per move, mitigated by `Solver.update` model reuse and the trivial-mine cache. It times itself (`this.lastDuration` in `recalc()`), and `debugMessage()` reports boundary size, uncached count and clause count. — [Kaboom index.js](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- Tatham's 2025 commits add "extra diagnostics for grid perturbation" and fix "a hang during grid generation". Even mature generators hit pathological cases. (snippet only) — [tartarus-commits 2025-07](https://lists.tartarus.org/pipermail/tartarus-commits/2025-July/015086.html)

### Inferences
- Load-test classes worth generating (synthesis):
  - (a) **One giant connected frontier component.** Long snake-like revealed regions with numbers 1–3, which maximise the "revealed tiles which can be strung together". This blows up exact per-component enumeration.
  - (b) **Many independent small components plus a large unconstrained interior.** This stresses the combination step: big-integer binomials and the global mine-count convolution, as in Mindsweeper's `compute_weights`.
  - (c) **Late-game near-ties** where the global mine count is the only discriminator. This stresses correct handling of `minesLeft`.
  - (d) **Chains of extended 50/50s** with up to 2^k consistent configurations that are locally indistinguishable.
  - (e) **Boards just above a brute-force threshold**, e.g. 751+ remaining solutions for JSMinesweeper, to test the fallback path.
- Differential testing: run two independent exact engines (e.g. a SAT/z3-based checker like Kaboom or gazpachoking versus a component-enumeration engine like Mindsweeper's) on the same adversarial positions. Assert identical forced-safe and forced-mine sets and identical configuration counts. (Speculation; no source describes this exact practice.)

### Gaps
- I found no published benchmark suite of "hard" Minesweeper positions or timing tables for exact engines beyond JSMinesweeper's qualitative 50x50 versus 100x100 note.

## 6. Fairness and game design for a max-guess mode

### Takeaway
Existing designs converge on a few rules: never lose on the first click, punish only *avoidable* guesses, make forced guesses free or at least honestly random, restrict where guesses are allowed (Kaboom's frontier-only default), and explain the loss afterwards. A deliberate max-guess mode would invert the "free forced guess" rule. The fairness levers that remain are guessing only at true 50/50s and telling the player when a guess is forced.

### Cited Findings
- Kaboom: forced guesses always succeed and avoidable guesses always fail. By default, guessing is allowed only on the frontier ("By default, you are allowed to guess only on the boundary of revealed tiles"). The hint text distinguishes safe / guessable / "All surrounding cells are deadly. You need to play elsewhere." Unlimited undo is provided. — [Kaboom index.js / index.html](https://github.com/pwmarcz/kaboom/blob/master/index.js)
- gazpachoking makes the forgiveness level a dial (NICE / FAIR / NORMAL / CRUEL). The first click is forced safe. — [gazpachoking/minesweeper](https://github.com/gazpachoking/minesweeper)
- Mindsweeper: guessing is never necessary. The first click is unrestricted (the first click and its neighbours are protected in `LocalGame::new`). A post-mortem shows "which tiles you could (and could not) have safely revealed". — [Mindsweeper README](https://github.com/AlexBuz/mindsweeper)
- JSMinesweeper: it is "always correct to guess these [true 50/50s] at the earliest opportunity, since they might provide information to their neighbouring tiles". The "Opening on start" option chooses between a guaranteed opening and only a guaranteed-safe first click. — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper)
- Tatham disables big whole-region perturbations at first because those grids are "a bit boring, with mines packed densely into far corners". Search objectives can degrade aesthetics. — [ghewgill/puzzles mines.c](https://github.com/ghewgill/puzzles/blob/master/mines.c)

### Inferences
- For a max-guess mode (design suggestions, not sourced):
  - Make every forced guess a **true** 50/50, or a clearly-labelled ratio. Hidden 70/30 guesses that a player misjudges feel unfair.
  - Expose the guess count and the luck budget (−log₂ S) up front, e.g. "this board needs 4 coin-flips".
  - Guarantee a first-click opening.
  - Place traps where perfect play would guess them early (JSMinesweeper's "guess 50/50s ASAP" principle), so players don't lose a nearly finished board to a forced coin flip.
  - Add a Mindsweeper-style post-mortem showing that a loss was a forced 50/50, not an error.
  - Keep mine density away from the "packed corners" artefact by limiting how far the search may move mines from a random baseline. For example, cap the swap count, or add a penalty term for local density variance.
- Uniform-sampling hygiene matters for perceived fairness. If the adversary or generator picks among consistent layouts, a biased choice (Kaboom-style "first SAT model") could produce recognisable, exploitable patterns. (Speculation.)

### Gaps
- I found no user-study or player-feedback data on how fun max-guess or adversarial modes are. Evidence is limited to the design statements quoted above.
