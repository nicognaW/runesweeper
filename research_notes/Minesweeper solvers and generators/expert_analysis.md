# How the competitive Minesweeper community analyzes games and boards

> **Access note for the report writer.** This session's network blocked direct fetches of minesweepergame.com, minesweeper.info, minesweeper.fandom.com, minesweeper.online, web.archive.org and wikipedia.org. Findings about those sites come from **search-engine snippets** of their pages; I could not read the pages in full. To make up for this, the strongest material below comes from **primary source code** that I cloned and read directly:
> - `thefinerminer/minesweeper-rawvf`: the RAWVF parsers, including `parser_raw.c`. It was last revised by Damien Moore, the owner of minesweepergame.com, and its README says it is used by the minesweepergame.com world-rankings video player.
> - `eee555/ms-toollib`: the algorithm library behind Metasweeper/saolei.wang-era Chinese tools. It parses AVF/RMV/MVF/EVF videos and computes every metric.
> - `PTTACGfans/Minesweeper-ZiNi-Calculator`: a ZiNi calculator.
> - `DavidNHill/JSMinesweeper`: a player, solver and analyser.
>
> Where a definition comes only from code, I say so. If the owners' own wording is needed verbatim, someone should re-check minesweepergame.com/statistics.php and /ranking-rules.php from a session that can reach those sites.

## Exact definitions and algorithms for 3BV, 3BV/s, IOE, RQP, IOS, ZiNi, Openings, Islands, Corr/ThrP, STNB, clicks (L/R/D), flagging vs NF

### Takeaway
Every board metric is built from two flood-fill concepts:
- **Openings**: 8-connected regions of zero cells, together with their numbered border.
- **Islands**: 8-connected groups of numbered cells that touch no zero.

3BV = number of openings + number of numbered cells that do not border any opening. ZiNi is a greedy simulation, not an exact optimum. It estimates the minimum number of clicks (L+R+D) when flagging and chording are allowed.

Player metrics are simple ratios. Each one has a known reference formula:
- IOE = 3BV/clicks
- Corr = effective clicks / clicks
- ThrP = 3BV / effective clicks
- RQP = time²/3BV (equivalently time / (3BV/s))
- IOS = log(3BV)/log(time)
- STNB = level constant × 3BV / time^1.7, scaled by the square root of the fraction completed

### Cited Findings

**3BV (Bechtel's Board Benchmark Value)**
- Authoritative Minesweeper defines 3BV as "the minimum number of left clicks required to win the game", and states its purpose as measuring board difficulty — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php); [minesweepergame.com Statistics (search snippet)](https://minesweepergame.com/statistics.php)
- The name expands as "Bechtel's Board Benchmark Value". The Stephan Bechtel PDF turned up in search, but I could not read it — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/); [stephan-bechtel.de PDF (unread)](http://www.stephan-bechtel.de/downloads/Minesweeper.pdf)
- Counting rule (the statistics page, as summarized by search): each opening costs one click, and each number not adjacent to an opening costs one click — [minesweepergame.com Statistics (search snippet)](https://minesweepergame.com/statistics.php)
- Algorithm in the reference RAW parser (Damien Moore, 2020). `calc_bbbv()` sets `bbbv = openings`, then adds one for every non-mine cell with `!board[i].opening`, that is, every cell that is not part of any opening or its border — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- The same algorithm in ms-toollib: `cal_bbbv = cal_bbbv_on_island(board) + cal_op(board)`.
  - `cal_op` counts 8-connected components of 0-cells by flood fill.
  - `cal_bbbv_on_island` counts cells with value > 0 that have no 0 among their 8 neighbours.
  - Source: [ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- Sources disagree on whether 3BV assumes chording. One glossary says "without using chording"; another says "assuming perfect play with chording". The owner's definition says "left clicks", and the algorithm above counts no chords — [rarepike glossary / minesweeperblast (search snippets)](https://rarepike.com/minesweeper/glossary/)

**Openings and Islands**
- Openings (Op) in ms-toollib are "the number of 8-connected domains of empty cells (0)". The code is a recursive flood fill over 0-cells in the 3×3 neighbourhood — [ms-toollib utils.rs `cal_op`](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- In the RAW parser, `process_opening()` floods 0-cells and assigns opening IDs to bordering numbers. A number can belong to two openings (`opening` and `opening2`). It also tracks each opening's size in cells — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- Islands (Isl). In the RAW parser, `process_island()` floods 8-connected non-mine cells that do not belong to any opening. Each such connected group of "isolated numbers" is one island. Island sizes are tracked too — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- ms-toollib `cal_isl` uses the same idea. It marks numbered cells with no adjacent 0 and counts their 8-connected components — [ms-toollib utils.rs `cal_isl`](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- ms-toollib also provides `cal_cell_nums`, a count of each number value 0–8 on the board — [ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)

**3BV/s**
- 3BV/s (written "3BVS" on minesweepergame.com) is defined as 3BV / Time and described as a measure of solving speed — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- For unfinished games, the tools use *solved* 3BV. RAW parser: `RAW_3BV/s = solved_bbbv / time`. ms-toollib: `get_bbbv_s = bbbv_solved / rtime` — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- ms-toollib also computes an estimated time, `etime = time / bbbv_solved × bbbv`. This is the projected finishing time of a lost game — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- The Viennasweeper feature list mentions counters for "time (or estimated time)" — [sweeper.wien About (search snippet)](https://sweeper.wien/about.html)

**Clicks (L/R/D), wasted clicks, effective clicks, IOE, Corr, ThrP**
- How the RAW parser counts clicks:
  - A **left click** (`l_clicks`) is counted when the left button is released on the board without chording.
  - A **double click / chord** (`d_clicks`) is counted when the left release happens while the right button is held, or with Shift-left, or with a middle click.
  - **Right clicks** (`r_clicks`) are counted on the right button.
  - "1.5 clicks" (flag, then slide onto a number to chord) are counted separately.
  - Total `clicks = l_clicks + r_clicks + d_clicks`.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- **Wasted clicks** in the RAW parser:
  - A left click on an already opened or flagged cell is a wasted left click.
  - A chord that opens nothing is a wasted double click.
  - Wasted right clicks are counted separately.
  - A flag counts as "wasted" until a successful chord uses it.
  - `w_clicks = wasted_l + wasted_r + wasted_d`, and `e_clicks` (effective) `= clicks − w_clicks`.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- Exact formulas in the reference parser (`parser_raw.c`):
  - `RAW_IOE = solved_bbbv / clicks`
  - `RAW_Correctness = e_clicks / clicks`
  - `RAW_Throughput = solved_bbbv / e_clicks`
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- ms-toollib uses the same structure:
  - `IOE = bbbv_solved / cl` with `cl = left + right + double`
  - `Corr = ce / cl` and `ThrP = bbbv_solved / ce`
  - `ce = lce + rce + dce` (effective left, right and double clicks)
  - An effective right click (`rce`) counts only the first flag placed on a given cell; re-flags are not counted.
  - Source: [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs); [ms-toollib minesweeper_board.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/minesweeper_board.rs)
- minesweepergame.com (search summary): IOE is 3BV divided by total clicks, 1.00 means no wasted clicks, and chording can push it above 1. Throughput uses effective clicks — [minesweepergame.com Statistics (search snippet)](https://minesweepergame.com/statistics.php); [minesweepergame.com Strategy – Efficiency (search snippet)](https://minesweepergame.com/strategy/efficiency.php)
- A fan page describes Corr as "how many board clicks weren't wasted" and ThrP as "the IOE for that board if the Correctness were 100%". That fits the formulas, since IOE = Corr × ThrP. The same page wrongly calls IOE "clicks divided by board time", which contradicts the owner's formula — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- Arbiter changelog (search snippet): "Correctness and other effective clicks related indices aren't shown in legal mode anymore" — [minesweepergame.com Arbiter download (search snippet)](https://minesweepergame.com/download/arbiter.php)
- JSMinesweeper's end-of-game message shows "3BV: solved/total, Actions: n, Efficiency: 100×3BV/actions %". It uses solved 3BV for lost games — [JSMinesweeper main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/main.js)
- Path / distance:
  - The RAW parser measures cursor `distance` with the **Manhattan** metric.
  - ms-toollib reports `path`, plus `c_path` and `e_path`. These sum the distances between consecutive effective clicks, normalized to 16 px per cell; `e_path` uses Euclidean distance between cells.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**RQP, IOS, QG**
- RQP ("Rapport Qualité-Prix") = time / (3BV/s) = time² / 3BV; lower is better — [minesweepergame.com Statistics via search snippet](https://minesweepergame.com/statistics.php)
- ms-toollib implements `rqp = rtime² / bbbv_solved` — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- IOS (Index of Speed): the search summary of minesweepergame.com gives log(3BV)/log(Time). The fan wiki gives ln(3bv)/ln(time−1). Higher is better either way. The two forms give different values, so check which one a tool uses — [minesweepergame.com Statistics (search snippet)](https://minesweepergame.com/statistics.php); contradicted by [minesweeper.fandom IOS (search snippet)](https://minesweeper.fandom.com/wiki/IOS)
- RQP is described as a legacy metric that "no one actually measures anything with anymore". Arbiter still keeps high scores for Time, 3BV/s, IOE, IOS and RQP — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- QG: ms-toollib computes `qg = time^1.7 / bbbv_solved`. This is the denominator used in STNB — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**STNB**
- The fan wiki gives Arbiter's formula as STNB = √(completion) × (87.420·mode² − 155.829·mode + 115.708) / (Time^1.7 / 3BV), with mode = 1, 2, 3 for Beginner, Intermediate, Expert. Older constants were 47.299, 153.73 and 435.001 — [minesweeper.fandom STNB (search snippet)](https://minesweeper.fandom.com/wiki/STNB)
- From that polynomial, mode 1 gives 47.299, mode 2 gives 153.730, and mode 3 gives 435.001. These match the "older constants" exactly, which suggests the constants and the polynomial are the same thing (my arithmetic).
- ms-toollib uses different constants:
  - `c = 36` for (8×8, 10 mines), `162` for (16×16, 40), `435` for (16×30, 99), and 0 for any other size.
  - `STNB = c × bbbv_solved / time^1.7 × (bbbv_solved/bbbv)^0.5`.
  - The Beginner and Intermediate constants therefore differ from Arbiter's 47.3 and 153.7.
  - Source: [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**ZiNi (Zimmermann–Nikolaus) and its algorithms**
- ms-toollib's comments describe ZiNi as a greedy algorithm estimating the minimum clicks (left + right + double) needed to clear a board. It was proposed by **Elmar Zimmermann and Christoph Nikolaus** and named after their initials. The original thread and source download are linked as minesweepergame.com forum t=70 / file id=120 — [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs). The original forum thread was not reachable in this session.
- ZiNi exists because "3BV is long known to favor NF players". ZiNi (Greedy ZiNi) and H.ZiNi (Human ZiNi) are algorithms that measure board difficulty "specifically for flag play" — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- **Premium initialization** (RAW parser and ms-toollib are identical):
  - Start with `premium = −number − 2`, then add `getadj3bv(cell)`.
  - `getadj3bv` returns 1 for a 0-cell.
  - For a number, it returns the count of non-mine neighbours (including itself) that are not in any opening, plus 1 if the number borders an opening, plus 1 more if it borders a second opening.
  - In plain terms, premium is the 3BV a chord on this cell would clear, minus the clicks it costs (flags needed, plus the chord, plus 1).
  - The code comments say: "The fewer clicks needed to perform a useful action (like a chord) the higher the premium… Each correct flag makes a number more useful… A higher number is less useful because more flags are required."
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Premium updates during the simulation**:
  - Opening a cell adds 1 to its own premium. If the cell is not part of an opening, it also subtracts 1 from each neighbour's premium.
  - Flagging a mine adds 1 to every neighbour's premium.
  - Revealing a 0 opens the whole opening and its border.
  - Source: [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Greedy ZiNi** (`calc_zini` / `cal_zini`):
  - While unopened non-mine cells remain, pick the non-mine cell with the highest premium, provided it is ≥ 0. Ties go to the first cell in column-major order.
  - If the cell is closed, click it (+1). Flag every adjacent unflagged mine (+1 each). Chord it (+1).
  - If no cell has premium ≥ 0, left-click the first closed non-mine cell that is either a 0 or not part of any opening (+1).
  - ZiNi is the total click count.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Human ZiNi (HZiNi)**:
  - First click every opening (`hit_openings`).
  - Then run the same greedy loop, restricted to cells that are **already opened**. This models a human, who chords only on numbers they can see.
  - The fallback click is the same as in Greedy ZiNi.
  - HZiNi ≥ ZiNi in practice. ms-toollib's test board (a 127-3BV Expert game) gives ZiNi = 96 and HZiNi = 103.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib zini.rs tests](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Random ZiNi** (`cal_rzini`, ms-toollib): run many iterations, choose uniformly at random among cells tied for the highest premium, and return the minimum over all runs. The test asserts random-min ≤ greedy — [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **PTTACGfans improvements**:
  - They are documented in a Chinese Google Doc and implemented in the PTTACGfans calculator.
  - `calculateZiNi(loop_count)` runs randomized ZiNi repeatedly and keeps the minimum along with its click path (left / right / chord per step).
  - When the loop count exceeds 100, every 2nd run uses a **"two_step"** lookahead and every 100th run uses a **"wide_candidate"** mode, which also considers candidates with premium ≤ 0.
  - ms-toollib's comment summarizes the improvements as "two-step, mixed, path optimization".
  - Source: [PTTACGfans Board.js](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator/blob/main/js/Board.js); [PTTACGfans README](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator/blob/main/README.md); [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Metrics derived from ZiNi** (RAW parser):
  - ZiNi/s = `gzini × solved_bbbv / (bbbv × time)`
  - ZNE (ZiNi efficiency) = `gzini × (solved/bbbv) / clicks`
  - ZNT (ZiNi throughput) = `gzini × (solved/bbbv) / e_clicks`
  - HZiNi has the analogous HZNE and HZNT.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)

**Flagging vs NF (no-flag)**
- A no-flag (NF) player uses only left clicks, so 3BV is exactly their minimum click count. A flagger can beat 3BV in clicks by chording, which is why IOE can exceed 1 — [minesweepergame.com Statistics (search snippet)](https://minesweepergame.com/statistics.php)
- 3BV "favors NF players", and ZiNi measures difficulty for flag play — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- JSMinesweeper's solver has four play styles: Flagging, No Flagging, Efficiency (chords allowed, flags only where they reduce clicks), and NF Efficiency ("minimise the number of clicks without chording… clicking openings and avoiding tiles which are next to an opening") — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- How the RAW parser describes chording:
  - Normal chording is "flag (RC-RR) then chord (LC-RC-LR-RR)". Shift+LC and middle click also chord.
  - A "1.5 Click" is a flag RC followed by sliding onto a number to finish the chord.
  - "Rilian Click" (Arbiter ≤ 0.44) is an illegal move: a failed chord released on an unopened cell, which opens that cell.
  - "Elmar Technique" means configuring the left mouse button to click on both press and release.
  - Source: [parser_raw.c header](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)

### Inferences
- Because the RAW parser was revised by Damien Moore, owner of minesweepergame.com and minesweeper.info, and is used by the minesweepergame.com video player, its formulas are the closest available stand-in for the owner's definitions. ms-toollib matches them for 3BV, IOE, Corr, ThrP and ZiNi but **not** for STNB constants.
- One identity is useful for a game implementation: IOE = Corr × ThrP. It follows from (3BV/clicks) = (eff/clicks) × (3BV/eff).
- ZiNi is a heuristic upper bound on the true minimum click count with flagging. Random and two-step variants lower it, so "the ZiNi" of a board depends on which algorithm and iteration count was used.

### Gaps
- I could not read the exact verbatim text of minesweepergame.com/statistics.php, including the precise IOS formula and any IOS/RQP history; it was blocked by the network. The IOS ln vs ln(t−1) conflict is unresolved.
- I could not read the original ZiNi forum thread or source, minesweepergame.com forum t=70, so the inventors' own wording is unconfirmed beyond ms-toollib's attribution.
- Who introduced STNB, and when the constants changed, could not be verified.

## Analysis tools: Arbiter, Clone, Viennasweeper, Minesweeper X, AVF/RAWVF, minesweeper.online, JSMinesweeper — what each reports

### Takeaway
The legacy clones record videos in their own formats:
- Arbiter: `.avf`
- Minesweeper Clone: `.mvf`
- Minesweeper X: `.mvr`
- Viennasweeper: `.umf` / `.rmv`
- FreeSweeper: `.fsvf`

The community-standard RAWVF text format, together with `parser_raw`, converts any of these into a common event log. It then computes the full statistics set: 3BV, solved 3BV, ZiNi/HZiNi, clicks by type, wasted clicks, IOE/Corr/ThrP/ZNE/ZNT, openings, islands, flags and misflags, and Rilian clicks. JSMinesweeper is a different kind of tool: a probability and solver analyser that marks safe tiles, mines, dead tiles, 50/50s and the best guess.

### Cited Findings
- RAWVF:
  - It is a "human readable minesweeper video format" created in 2009, with one text row per mouse event.
  - Supported sources: Arbiter 0.35–0.52.3 (AVF), Minesweeper Clone 0.76 beta–2007 r2 (MVF), Minesweeper X 1.00–1.16 (MVR, whose encryption is kept private by its programmer), Vienna MineSweeper up to 3.0C/3.0H (UMF/RMV), and FreeSweeper up to R10 (FSVF).
  - The format was invented by Cryslon (author of Arbiter and FreeSweeper) with help from Curtis (MSX) and Tommy (Viennasweeper). crazyks finalized "RAW 5" in 2014, and thefinerminer released RAW 6 in 2020.
  - Source: [minesweeper-rawvf README](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/README.md)
- The master parser `parser_raw` "adds events such as flags, questionmarks, numbers, mines, openings and tells you which squares were opened". The web player built on it (hgraceb/web-minesweeper-player) is used by the minesweepergame.com World Rankings and by saolei.wang (China rankings) — [minesweeper-rawvf README](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/README.md)
- The full statistics list from `parser_raw`:
  - Time, 3BV, Solved3BV, 3BV/s
  - ZiNi, ZiNi/s, HZiNi, HZiNi/s
  - Clicks, LeftClicks, RightClicks and DoubleClicks (each also per second)
  - WastedClicks, WastedLeftClicks, WastedRightClicks and WastedDoubleClicks (each also per second)
  - 1.5Clicks
  - IOE, Correctness, Throughput, ZNE, ZNT, HZNE, HZNT
  - Openings, Islands
  - Flags, WastedFlags, Unflags, Misflags, Misunflags
  - RilianClicks
  - Decimals are **truncated** to 3 places, not rounded.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- The `parser_raw` header also states the timing and legality rules: "The timer starts after the first cell is opened. Cells open after a LC-LR sequence." The program runs in legal and cheat modes and processes FreeSweeper cheat options (Nono, Superflag, Superclick) — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- The AVF parser was modified by ShenJia Zhang in 2013 to add Time and 3BV, and by Damien Moore in 2020 — [parser_avf.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_avf.c)
- Minesweeper Arbiter keeps high scores for Time, 3BV/s, IOE, IOS and RQP. Its changelog says effective-click indices such as Correctness are hidden in legal mode — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/); [minesweepergame.com Arbiter page (search snippet)](https://minesweepergame.com/download/arbiter.php)
- Viennasweeper:
  - It has a statistics window with 3BV, 3BV/s and click counts, and counters for "time (or estimated time), 3BV/3BVS, and L/D/R clicks".
  - It saves every finished game automatically as a checksummed `.rmv` replay and includes a replay viewer.
  - It is the official version for tournaments and integrates with Scoreganizer.
  - A new RMV version was co-specified in late 2024.
  - Sources disagree on timer precision: thousandths vs hundredths of a second.
  - Source: [minesweepergame.com Viennasweeper (search snippet)](https://minesweepergame.com/download/viennasweeper.php); [sweeper.wien About (search snippet)](https://sweeper.wien/about.html)
- A Python project for Scoreganizer, `ralokt/sweeping-view`, parses Arbiter and Viennasweeper videos — [minesweeper-rawvf README](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/README.md)
- ms-toollib (Metasweeper):
  - It parses AVF, RMV, MVF and its own EVF/EVFS formats.
  - Per event, it exposes all the metrics above plus Op/Isl solved, `etime`, `stnb`, `rqp`, `qg`, path metrics, `checksum` and probability boards.
  - Its "analyse" methods tag events with comments:
    - `error:high_risk_guess` when the clicked cell had a mine probability ≥ 0.51
    - `feature:hard_judgment` when a safe or mine cell was deducible only by enumeration, not basic logic
    - `warning:needless_guess` when the player guessed a cell with a mine probability > 0 while a provably safe cell lay within the surrounding 5×5
    - mouse-trace curvature warnings, vision transfer, and "textbook FL local" (super_fl_local)
  - Source: [ms-toollib analyse_methods.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/analyse_methods.rs)
- ms-toollib luck metric **"pluck"**:
  - For each risky click (not the first click), `pluck −= log10(safety)`, where safety = 1 − P(mine) of the clicked cell. For a chord, safety is the probability that all covered neighbours are safe.
  - So pluck = −log10(product of the safety probabilities), and higher means luckier.
  - It also computes the expected solved 3BV and expected time per attempt and per win.
  - Source: [ms-toollib pluck.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/pluck.rs)
- JSMinesweeper (David Hill):
  - It is a player plus an "Analyser" mode. You build a position (hidden, revealed, flags, mine count) and the analyser shows safe tiles in green, mines in red, and the best guess in yellow with a green centre. Other candidate guesses show in yellow, and "dead" tiles in grey ("can have only one possible value and it is never correct to play these").
  - It loads MinesweeperX `.mbf` and Arbiter `.abf` board files and exports `.mbf`.
  - Its techniques are trivial analysis, an exact probability engine, 50/50 and pseudo-50/50 detection, guessing logic, and brute-force analysis when 750 or fewer solutions remain. Brute force can state the exact win probability with best play.
  - The Java solver wins 41% of classic Expert games (safe start in a corner) and 54.3% of modern Expert games (opening start at (3,3)).
  - Source: [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- minesweeper.online: a player profile shows that the site tracks 3BV/s personal bests and efficiency percentages (e.g. "160%" on Beginner). I found no help page that defines its formulas or replay-analysis features — [minesweeper.online player profile (search snippet)](https://minesweeper.online/player/41811334)
- A fan-wiki note says Minesweeper Online's speed ranking omits Beginner times "because there is no established definition of boards that are 'too lucky'" — [minesweeper.fandom World Ranking (Speed) (search snippet)](https://minesweeper.fandom.com/wiki/World_Ranking_(Speed))

### Inferences
- In practice, "analysis" in the speedrunning community means **video statistics**: post-game parsing of a replay into counts and ratios. Probability analysis of individual decisions is a newer layer, found in ms-toollib/Metasweeper and JSMinesweeper.
- A "%" efficiency figure above 100% on minesweeper.online fits IOE × 100. This is consistent with the definition, but the site's own formula is unconfirmed.

### Gaps
- I could not read minesweeper.online's help, wiki or replay pages; the host was blocked and search returned almost nothing. Its exact metrics, replay/analysis features, and no-guess or leaderboard rules are **unverified**.
- I found no primary documentation for Minesweeper Clone's or Minesweeper X's built-in statistics windows.
- I found no primary specification of the AVF binary layout beyond the parser source, which I did not read in detail.

## How rankings and record verification work (Authoritative Minesweeper, minesweepergame.com, WMA, minesweeper.online): board sizes, 3BV limits, luck vs skill

### Takeaway
The minesweepergame.com (Authoritative Minesweeper) rules work as follows:
- Rankings use Beginner 8×8/10, Intermediate 16×16/40 and Expert 30×16/99.
- Players are ranked by the sum of their best times.
- Videos must come from approved clones: Arbiter 0.52.3, Minesweeper X 1.16 and Viennasweeper 3.0.2H.
- Luck is controlled with minimum-3BV limits. Time rankings require 3BV ≥ 2 / 30 / 100 for Beginner / Intermediate / Expert. 3BV/s rankings require a Beginner minimum of 10.

I found no evidence that a body called the "World Minesweeper Association" exists.

### Cited Findings
- Levels: Beginner (8×8, 10 mines), Intermediate (16×16, 40 mines) and Expert (30×16, 99 mines). Players are ranked by summed fastest times, and ties go to the better score on the harder level — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- "Rankings for Time enforce 3BV limits of 2 (Beginner), 30 (Intermediate) and 100 (Expert) to ensure the rankings reward skill." The rules also note "it is theoretically possible for a Beginner, Intermediate or Expert game to be solved in 1 click" — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- "The rankings for 3BVS enforce a limit of 10 for Beginner to give less weight to the random clicks used to start the game" — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- Accepted clones are Arbiter 0.52.3, Minesweeper X 1.16 and Viennasweeper 3.0.2H. "All Microsoft versions were banned in 2008 due to Cheats and the fact the Community has developed secure Clones with video recording." Microsoft scores remain only as historical entries — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- Players register and upload videos to join the rankings. Knowing a board in advance counts as cheating. Boards larger than the standard levels are not ranked — [minesweepergame.com Ranking Rules (search snippet)](https://minesweepergame.com/ranking-rules.php)
- Density rankings are separate. The official density boards are 8×8, 9×9, 16×16, 30×16 and 30×24 with at least 20, 25, 60, 120 and 150 mines. The 9×9 size came from Windows XP, which replaced 8×8 — [minesweepergame.com Density Rankings (search snippet)](https://minesweepergame.com/density-rankings.php)
- Background articles on minesweepergame.com: Ronny de Winter, "3BV Limits on Trial" (2009), and "lucky one second beginner games" (2010). Both were found but not readable — [3BV Limits on Trial PDF](https://minesweepergame.com/article/3bv-limits-on-trial-2009.pdf); [lucky one second beginner games PDF](https://mail.minesweepergame.com/article/lucky-one-second-beginner-games-2010.pdf)
- minesweeper.info hosts an "Active Ranking" and a "World Record History"; these are now archived by minesweepergame.com — [minesweeper.info Active Ranking (search snippet)](https://minesweeper.info/wiki/Active_Ranking)
- Other sites use different limits:
  - minesweeper.now: Beginner ≥ 5, Intermediate ≥ 30, Expert ≥ 100.
  - Minesweeper Pro: Beginner minimum 2.
  - Minesweeper 99: Beginner 3BV ≥ 3.
  - These are not Authoritative Minesweeper rules.
  - Source: [search summary citing minesweeper.now / minesweeper-pro / babafish](https://minesweeper-pro.com/championships/)
- No "World Minesweeper Association" turned up in any search. The nearest bodies are minesweepergame.com, which hosts World Championship results (Budapest 2006, Vienna 2008, Zurich 2011…), and a minesweeper.now mention of an "IMESA" that I could not verify — [WMA search results](https://minesweepergame.com/ranking-rules.php); [Vienna 2008 WC](https://minesweepergame.com/tournament/vienna-2008.php)

### Inferences
- Under these rules, luck versus skill is handled by (a) 3BV floors that exclude boards solvable in one or two clicks, (b) 3BV/s rankings that normalize time by difficulty, and (c) video verification on secure clones.
- Note that the ranking Beginner is 8×8, while Viennasweeper and Windows XP use 9×9. For our game, standard Beginner should be configurable.
- Minesweeper Online reportedly omits Beginner from its speed ranking because no definition of "too lucky" exists. That shows the limits are community conventions, not derived values.

### Gaps
- I could not confirm whether minesweepergame.com now also accepts Minesweeper Clone or newer Viennasweeper versions. Third-party minesweepergame.net lists Clone as approved, but the official snippet does not.
- I could not read the exact verification procedure: who reviews videos, checksum checks, and so on.
- minesweeper.online leaderboard rules (no-guess mode, rating systems) were not reachable.

## How experts classify a board as needing guesses; counting forced guesses

### Takeaway
Experts and their tools sort guesses into three kinds:
- **Forced guesses**: no safe cell can be deduced. These include unavoidable 50/50s, meaning a pair of cells sharing one mine where neither can ever receive information without the other.
- **Needless guesses**: a guess made while a safe cell existed.
- **Risk-weighted luck**: the product of the safety probabilities of every risky click.

A board is "no-guess" if a deterministic solver, starting from the first click, can clear it by logic alone.

### Cited Findings
- JSMinesweeper's definition of a 2-tile 50/50: "a pair of tiles which share 1 mine where one of the tiles can never receive information without the other tile also receiving the same information… they must be (and always will be) an unavoidable guess". It detects extended 2-tile 50/50s and enclosed 2×2/2 boxes — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- JSMinesweeper's pseudo-50/50: "a set of tiles is either safe or part of a 50/50… correct to guess immediately". It detects 2-tile and 4-tile 2×2 pseudo-50/50s but not extended ones — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- JSMinesweeper's dead tiles: tiles that "are either mines or have only one possible value… if all the tiles left in the game are dead… the result is down to luck" — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- A guess count statistic: "to win a classic expert game of minesweeper the solver is required, on average, to make 3.3 guesses" — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- JSMinesweeper's guessing logic considers tiles within 10% of the safest. It blends "progress percentage" (the chance the next move yields a certain safe play) with "secondary safety" — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- ms-toollib `is_solvable(board, x0, y0)` decides whether a board is no-guess when solved from a given start cell. It rejects the board if it contains an unsolvable structure.
  - Note: if a cell is surrounded by mines, it "is not a guess if the centre is a mine, but is a guess if it is not".
  - Note: the remaining-mine count is ignored.
  - `unsolvable_structure()` uses templates (工-, 回- and 器-shaped patterns) to spot obvious dead-guess structures quickly. The doc warns that passing this template check does not prove a board is no-guess.
  - Source: [ms-toollib algorithms.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/algorithms.rs); [ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- ms-toollib's per-click guess classification in replays:
  - A high-risk guess has a mine probability ≥ 0.51.
  - A needless guess has a mine probability > 0 while a basic- or enumeration-deducible safe cell exists in the surrounding 5×5.
  - The luck metric pluck = −Σ log10(safety). The first click is exempt.
  - Source: [ms-toollib analyse_methods.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/analyse_methods.rs); [ms-toollib pluck.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/pluck.rs)
- No-guess modes on other sites generate "boards that are fully solvable by logic alone. Every cell can be deduced" — [minesweeper.now help (search snippet)](https://minesweeper.now/help/efficiency)

### Inferences
- Counting forced guesses depends on the solver. A board's "guess count" is the number of times a perfect-logic solver (including global mine-count reasoning) has no safe move. Whether a guess counts as forced also depends on whether mine-count endgame logic is allowed; ms-toollib's `is_solvable` ignores it.
- For a game implementation, it is worth reporting the guess count, 50/50s, and a pluck-style luck score separately from speed and efficiency metrics, since the community treats luck and skill as separate axes.

### Gaps
- I found no official minesweepergame.com or minesweeper.info definition of "forced guess", and no rule that excludes guess-heavy boards from records. Records are filtered by 3BV, not by guess count.
- I could not confirm how minesweeper.online classifies or generates no-guess boards.
