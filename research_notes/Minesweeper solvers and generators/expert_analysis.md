# How the competitive Minesweeper community analyzes games and boards

> **Access note for the report writer.** The first version of these notes was written while minesweepergame.com, minesweeper.info, minesweeper.fandom.com, minesweeper.online, web.archive.org and wikipedia.org were blocked, so the site-level claims came from search snippets. On 2026-11-06 the owner pages were **re-checked directly**:
> - minesweepergame.com `statistics.php`, `ranking-rules.php`, `density-rankings.php`, `world-records.php`, `about.php`, `media.php`, `clone-cheats.php`, `download/arbiter.php`, `download/viennasweeper.php`, and the strategy pages: fetched with curl and a browser user agent.
> - minesweeper.online `/help/gameplay`, `/help/efficiency`, `/help/patterns`, `/help/guides`: rendered with headless Chrome, because the site is a JavaScript app.
> - Community guides linked from minesweeper.online's Guides page: read as Google Docs text exports.
>
> Still unreachable:
> - The minesweepergame.com forum, including the ZiNi thread t=70: 403 to curl and to headless Chrome.
> - web.archive.org: connections reset through the proxy.
> - minesweeper.info/wiki/*: every page now redirects to the minesweepergame.com home page, so the old wiki can't be read.
>
> Corrections are marked **[corrected after primary check]**. Fan-wiki (minesweeper.fandom.com) and somnolescent claims were outside the re-check list and keep their "(search snippet)" tags.
>
> The strongest material on algorithms still comes from **primary source code** that I cloned and read directly:
> - `thefinerminer/minesweeper-rawvf`: the RAWVF parsers, including `parser_raw.c`. It was last revised by Damien Moore, and its README says it is used by the minesweepergame.com world-rankings video player.
> - `eee555/ms-toollib`: the algorithm library behind Metasweeper/saolei.wang-era Chinese tools. It parses AVF/RMV/MVF/EVF videos and computes every metric.
> - `PTTACGfans/Minesweeper-ZiNi-Calculator`: a ZiNi calculator.
> - `DavidNHill/JSMinesweeper`: a player, solver and analyser.

## Exact definitions and algorithms for 3BV, 3BV/s, IOE, RQP, IOS, ZiNi, Openings, Islands, Corr/ThrP, STNB, clicks (L/R/D), flagging vs NF

### Takeaway
Every board metric is built from two flood-fill concepts:
- **Openings**: 8-connected regions of zero cells, together with their numbered border.
- **Islands**: 8-connected groups of numbered cells that touch no zero.

3BV = number of openings + number of numbered cells that do not touch an opening. ZiNi is a greedy simulation, not an exact optimum. It estimates the minimum number of clicks (L+R+D) when flagging and chording are allowed.

The owner's Statistics page (minesweepergame.com) now confirms these formulas verbatim:
- 3BV/s = 3BV / Time
- IOS = log(3BV) / log(Time)
- RQP = Time / 3BV/s
- IOE = 3BV / Total Clicks, "and the factors of IOE are Correctness and Throughput"
- Correctness = Effective Clicks / Total Clicks
- Throughput = 3BV / Effective Clicks

**STNB is not on the owner's page.** The STNB formula below still comes only from code and the fan wiki.

### Cited Findings

**3BV (Bechtel's Board Benchmark Value)**
- minesweepergame.com Statistics, verbatim: "Purpose is to measure board difficulty. 3BV is the minimum number of left clicks required to win the game. Each opening is one click and each number that does not touch an opening is one click. The full title is Bechtel's Board Benchmark Value." (Confirmed; this replaces the earlier snippet.) — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- The Ranking Rules page says the same: "3BV is the minimum number of left clicks needed to win a game. It is a popular way to measure board difficulty". — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- The No Flags page explains why opening borders are not counted: "Numbers surrounding openings are not counted because clicking them is redundant." — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php)
- minesweeper.online's wording: "3BV (Bechtel's Board Benchmark Value) is the minimum number of clicks required to complete a board without using flags. 3BV is used to measure the relative difficulty of a board, and the speed you are playing at (often expressed as 3BV/s, or 3BV per second)." — [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay)
- The Stephan Bechtel PDF turned up in search but was not read in either pass. — [stephan-bechtel.de PDF (unread)](http://www.stephan-bechtel.de/downloads/Minesweeper.pdf)
- Algorithm in the reference RAW parser (Damien Moore, 2020). `calc_bbbv()` sets `bbbv = openings`, then adds one for every non-mine cell with `!board[i].opening`, that is, every cell that is not part of any opening or its border — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- The same algorithm in ms-toollib: `cal_bbbv = cal_bbbv_on_island(board) + cal_op(board)`.
  - `cal_op` counts 8-connected components of 0-cells by flood fill.
  - `cal_bbbv_on_island` counts cells with value > 0 that have no 0 among their 8 neighbours.
  - Source: [ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- **[corrected after primary check]** On chording: both owner sites define 3BV as a no-chord, no-flag count. minesweepergame.com says "left clicks"; minesweeper.online says "without using flags". The third-party glossary phrase "assuming perfect play with chording" is wrong. — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php); [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay); [rarepike glossary (search snippet)](https://rarepike.com/minesweeper/glossary/)

**Openings and Islands**
- Openings (Op) in ms-toollib are "the number of 8-connected domains of empty cells (0)". The code is a recursive flood fill over 0-cells in the 3×3 neighbourhood — [ms-toollib utils.rs `cal_op`](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- A community glossary (Archeaic, "Minesweeper Terminology", linked from minesweeper.online Guides) defines an opening this way: "0 Cells, and the numbers that touch 0 cells are part of an opening." Its islands example says: "we consider all openings and mines to be 'sea' ... diagonally adjacent squares are considered to be part of the same island." — [Archeaic: Minesweeper Terminology (Google Doc)](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- In the RAW parser, `process_opening()` floods 0-cells and assigns opening IDs to bordering numbers. A number can belong to two openings (`opening` and `opening2`). It also tracks each opening's size in cells — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- Islands (Isl). In the RAW parser, `process_island()` floods 8-connected non-mine cells that do not belong to any opening. Each such connected group of "isolated numbers" is one island. Island sizes are tracked too — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- ms-toollib `cal_isl` uses the same idea. It marks numbered cells with no adjacent 0 and counts their 8-connected components — [ms-toollib utils.rs `cal_isl`](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- ms-toollib also provides `cal_cell_nums`, a count of each number value 0–8 on the board — [ms-toollib utils.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/utils.rs)
- The owner's Statistics page does not define Openings or Islands as separate statistics. — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)

**3BV/s**
- **[corrected after primary check]** The definition is on the Statistics page, not Ranking Rules: "3BV/s [ 3BV / Time ] Purpose is to measure solving speed." Ranking Rules uses the name "3BVS (your effective solving speed)". — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php); [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- For unfinished games, the tools use *solved* 3BV. RAW parser: `RAW_3BV/s = solved_bbbv / time`. ms-toollib: `get_bbbv_s = bbbv_solved / rtime` — [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- ms-toollib also computes an estimated time, `etime = time / bbbv_solved × bbbv`. This is the projected finishing time of a lost game — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- The Viennasweeper feature list mentions counters for "time (or estimated time)" — [sweeper.wien About (search snippet)](https://sweeper.wien/about.html)
- Decimal time and rounding, from Ranking Rules: "Microsoft Minesweeper starts the clock at 1 second on the first click. Official clones start decimal timers at 0.00 seconds. The community rounds up to the nearest integer thus 0.49s is a 1 second game ... 49.01s is a 50 second game and not 'Sub50'." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)

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
- minesweepergame.com Statistics, verbatim (confirmed; this replaces the earlier snippets):
  - "IOE [ 3BV / Total Clicks ] Purpose is to measure efficiency. A 3BV 50 Intermediate game completed in 50 clicks has an IOE of 1.00. It is possible to be more efficient than 1.00 by using chording. The full title is Index of Efficiency and the factors of IOE are Correctness and Throughput."
  - "Correctness [ Effective Clicks / Total Clicks ] Purpose is to measure percentage of effective clicks. Effective means the click changed the state of the board. Another statistic is CeS which is effective clicks per second."
  - "Throughput [ 3BV / Effective Clicks ] ... if you solve a 3BV 50 Intermediate game in 85 clicks but only 62 clicks change the state of the board so your Throughput is 0.81."
  - Source: [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- **Owner-site inconsistency (new):** the Efficiency strategy page words IOE backwards: "Index of Efficiency (IOE) which measures total clicks divided by 3BV". The Statistics page and every tool use 3BV/clicks. — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- minesweeper.online, verbatim: "Efficiency is the 3BV of a board divided by the number of clicks used to solve it ... An Efficiency of 100% means you solved a 3BV 50 board in 50 clicks." Also: "Efficiency is lost by placing flags, wasting clicks, or clicking cells that would be revealed by an opening." **[corrected after primary check]** This confirms the earlier inference that the site's "%" figure is IOE × 100. — [minesweeper.online: Efficiency](https://minesweeper.online/help/efficiency)
- A community glossary defines Corr differently: "Clicks / ( Clicks + Wasted Clicks )". It notes that "placing unnecessary flags will not negatively impact your Correctness" and that "Corr also punishes prechording". That definition disagrees with the owner's (effective/total). Prefer the owner's. — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- A fan page describes Corr as "how many board clicks weren't wasted" and ThrP as "the IOE for that board if the Correctness were 100%". The same page wrongly calls IOE "clicks divided by board time" — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- Arbiter changelog (search snippet): "Correctness and other effective clicks related indices aren't shown in legal mode anymore". **Not on the current Arbiter download page**, which lists features but no changelog; still unverified — [minesweepergame.com: Arbiter](https://minesweepergame.com/download/arbiter.php)
- JSMinesweeper's end-of-game message shows "3BV: solved/total, Actions: n, Efficiency: 100×3BV/actions %". It uses solved 3BV for lost games — [JSMinesweeper main.js](https://github.com/DavidNHill/JSMinesweeper/blob/master/Minesweeper/client/main.js)
- Path / distance:
  - The RAW parser measures cursor `distance` with the **Manhattan** metric.
  - ms-toollib reports `path`, plus `c_path` and `e_path`. These sum the distances between consecutive effective clicks, normalized to 16 px per cell; `e_path` uses Euclidean distance between cells.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**RQP, IOS, QG**
- minesweepergame.com Statistics, verbatim: "RQP [ Time / 3BV/s ] Purpose is to measure level of solving skill. RQP increases with faster scores and higher solving rates. The full title is Rapport Qualité Prix." (Confirmed.)
  - Note: arithmetically RQP = Time²/3BV, so it *falls* as play gets faster. The page's "increases with faster scores" sentence contradicts its own formula; read "lower is better".
  - Source: [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- A community glossary: RQP "has different formulas, such as: Time / ( 3BV/s ), (Time)^2 / (3BV/s), on World of Minesweeper it's (Time + 1) / (3BV/s) ... Lower is better". — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- ms-toollib implements `rqp = rtime² / bbbv_solved` — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)
- **IOS [corrected after primary check]**: the owner's Statistics page gives "IOS [ log (3BV) / log (Time) ] Purpose is to measure solving speed." That settles the owner's definition.
  - The "(Time − 1)" variant appears in community sources: the fan wiki gives ln(3bv)/ln(time−1), and the Archeaic glossary gives "log (3BV) / log (Time - 1)".
  - Neither says which program uses it. Tools may differ, so check per tool.
  - Sources: [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php); [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo); [minesweeper.fandom IOS (search snippet)](https://minesweeper.fandom.com/wiki/IOS)
- RQP is described as a legacy metric that "no one actually measures anything with anymore" — [somnolescent archive (search snippet)](https://archives.somnolescent.net/web/cammy_v1/games/minesweeper/)
- The Arbiter high-score list is confirmed on the owner page: "Highscores saved for Time, 3BV/s, IOE, IOS, RQP." — [minesweepergame.com: Arbiter](https://minesweepergame.com/download/arbiter.php)
- QG: ms-toollib computes `qg = time^1.7 / bbbv_solved`. This is the denominator used in STNB — [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**STNB**
- **Not defined on minesweepergame.com's Statistics page** (checked 2026-11-06). The community glossary on minesweeper.online lists "STNB - Needs data" and "QG - Needs data". — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php); [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- The fan wiki gives Arbiter's formula as STNB = √(completion) × (87.420·mode² − 155.829·mode + 115.708) / (Time^1.7 / 3BV), with mode = 1, 2, 3 for Beginner, Intermediate, Expert. Older constants were 47.299, 153.73 and 435.001 — [minesweeper.fandom STNB (search snippet)](https://minesweeper.fandom.com/wiki/STNB)
- From that polynomial, mode 1 gives 47.299, mode 2 gives 153.730, and mode 3 gives 435.001. These match the "older constants" exactly, which suggests the constants and the polynomial are the same thing (my arithmetic).
- ms-toollib uses different constants:
  - `c = 36` for (8×8, 10 mines), `162` for (16×16, 40), `435` for (16×30, 99), and 0 for any other size.
  - `STNB = c × bbbv_solved / time^1.7 × (bbbv_solved/bbbv)^0.5`.
  - The Beginner and Intermediate constants therefore differ from Arbiter's 47.3 and 153.7.
  - Source: [ms-toollib base_video_metrics.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/videos/base_video_metrics.rs)

**Other community metrics (new)**
- **OBV** (also "MBV" or "M3BV"), as given by the community glossary: a level-weighted linear combination of numbers, 3BV and openings, credited to Andrew McCauley: "OBV is like 3BV except that it should be just as easy for any given OBV value to get a high OBV/s score." The glossary marks it "(Needs data)". — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)

**ZiNi (Zimmermann–Nikolaus) and its algorithms**
- minesweepergame.com Statistics, verbatim: "ZiNi: Purpose is to measure board difficulty for players who use flags. This is difficult to calculate because the most efficient way to solve a board depends on your starting position (or requires perfect knowledge of the board in advance). The three versions are Greedy ZiNi (ZiNi), Random ZiNi and Human ZiNi. All three are algorithms for finding the solution to the board with the fewest clicks. Additional statistics have been proposed for Flaggers such as ZiNi/s (instead of 3BV/s), ZNE (instead of IOE) and ZNT (instead of Throughput)." — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- **Origin (new, secondary).** Gadgetlab's "ZiNi: An Overview" (dated 11 July 2025, updated 29 Oct 2026, linked from minesweeper.online Guides): "The original ZiNi algorithms were proposed in July 2009 by Elmar Zimmerman and Christoph Nikolaus on the Authoritative Minesweeper Forum as '3BV for flaggers'". It also says "Arbiter - an official clone - uses G.ZiNi and H.ZiNi. Minesweeper.Online (WoM) also use ZiNi algorithms." The forum thread itself (t=70) returned 403, so the inventors' own wording is still unread. — [Gadgetlab: ZiNi: An Overview (Google Doc)](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- Gadgetlab's rephrasing of Elmar's Greedy ZiNi:
  - "Premium (uncovered or 3BV) = adjacent covered 3BV - adjacent unflagged mines - 1"; "Premium (covered non-3BV) = adjacent covered 3BV - adjacent unflagged mines - 2".
  - Select the highest premium, with ties going to "the left-topmost cell".
  - If the premium is non-negative, flag the adjacent mines and chord, clicking the cell first if it is covered.
  - "If the highest Premium (4) is negative, the left-topmost cell is clicked."
  - Source: [Gadgetlab: ZiNi: An Overview](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- ms-toollib's comments describe ZiNi as a greedy algorithm estimating the minimum clicks (left + right + double). They attribute it to **Elmar Zimmermann and Christoph Nikolaus**, named after their initials, and link minesweepergame.com forum t=70 / file id=120 — [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Premium initialization** (RAW parser and ms-toollib are identical):
  - Start with `premium = −number − 2`, then add `getadj3bv(cell)`.
  - `getadj3bv` returns 1 for a 0-cell.
  - For a number, it returns the count of non-mine neighbours (including itself) that are not in any opening, plus 1 if the number borders an opening, plus 1 more if it borders a second opening.
  - In plain terms, premium is the 3BV a chord on this cell would clear, minus the clicks it costs (flags needed, plus the chord, plus 1).
  - This matches Gadgetlab's −2 (covered) / −1 (uncovered) forms, since opening a cell adds 1.
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
  - **Conflict (new):** the Archeaic glossary says H.ZiNi "chooses randomly rather than the top-left most" among tied premiums, and that "H.ZiNi can technically be more efficient than ZiNi". Gadgetlab describes H.ZiNi as G.ZiNi that "first clicks all openings ... then only selects uncovered cells", with no random tie-break. The reference code matches Gadgetlab. — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo); [Gadgetlab: ZiNi: An Overview](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- **Random ZiNi** (`cal_rzini`, ms-toollib): run many iterations, choose uniformly at random among cells tied for the highest premium, and return the minimum over all runs. The test asserts random-min ≤ greedy — [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs)
- **Later variants (new, from Gadgetlab):**
  - "8-way ZiNi" applies ZiNi with eight different selection biases and keeps the best. "8-way G.ZiNi ... is used as a standard for bulk analysis."
  - minesweeper.online's "Legacy ZiNi and H.ZiNi (L_ZiNi and L_H.ZiNi)" choose the cell that leaves the highest-premium cell after the chord.
  - PTTACGfans' calculator "can be activated on WoM in Settings as 'ZiNi' - using a dynamic (3BV-dependent) Loop Count ... it is the most commonly used ZiNi algorithm". WoM games can be opened in that calculator "by clicking next to the ZiNi statistic after a game".
  - LlamaSweeper offers Chain and DeepChain ZiNi.
  - Source: [Gadgetlab: ZiNi: An Overview](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- **PTTACGfans improvements**:
  - They are documented in a Chinese Google Doc and implemented in the PTTACGfans calculator.
  - `calculateZiNi(loop_count)` runs randomized ZiNi repeatedly and keeps the minimum along with its click path (left / right / chord per step).
  - When the loop count exceeds 100, every 2nd run uses a **"two_step"** lookahead and every 100th run uses a **"wide_candidate"** mode, which also considers candidates with premium ≤ 0.
  - ms-toollib's comment summarizes the improvements as "two-step, mixed, path optimization".
  - Gadgetlab describes the adjusted premium: "Premium (adjusted) = P + M", where M is the best neighbouring premium, used on odd loop counts.
  - Source: [PTTACGfans Board.js](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator/blob/main/js/Board.js); [PTTACGfans README](https://github.com/PTTACGfans/Minesweeper-ZiNi-Calculator/blob/main/README.md); [ms-toollib zini.rs](https://github.com/eee555/ms-toollib/blob/main/base/src/zini.rs); [Gadgetlab](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- **Metrics derived from ZiNi** (RAW parser):
  - ZiNi/s = `gzini × solved_bbbv / (bbbv × time)`
  - ZNE (ZiNi efficiency) = `gzini × (solved/bbbv) / clicks`
  - ZNT (ZiNi throughput) = `gzini × (solved/bbbv) / e_clicks`
  - HZiNi has the analogous HZNE and HZNT.
  - The owner's Statistics page names ZiNi/s, ZNE and ZNT as "proposed" statistics for flaggers.
  - Source: [parser_raw.c](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c); [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)

**Flagging vs NF (no-flag)**
- **[corrected after primary check: now cited to owner pages]** minesweepergame.com No Flags: "The statistic for board difficulty is 3BV, which counts the minimum number of left clicks required to clear a board ... Low 3BV boards favour NF players because these have more openings with the mines grouped into islands." The Statistics page gives ZiNi's purpose as "to measure board difficulty for players who use flags". — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php); [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- A no-flag (NF) player uses only left clicks, so 3BV is exactly their minimum click count. A flagger can beat 3BV in clicks by chording, which is why IOE can exceed 1: "It is possible to be more efficient than 1.00 by using chording." — [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php)
- JSMinesweeper's solver has four play styles: Flagging, No Flagging, Efficiency (chords allowed, flags only where they reduce clicks), and NF Efficiency ("minimise the number of clicks without chording… clicking openings and avoiding tiles which are next to an opening") — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- How the RAW parser describes chording:
  - Normal chording is "flag (RC-RR) then chord (LC-RC-LR-RR)". Shift+LC and middle click also chord.
  - A "1.5 Click" is a flag RC followed by sliding onto a number to finish the chord.
  - "Rilian Click" (Arbiter ≤ 0.44) is an illegal move: a failed chord released on an unopened cell, which opens that cell.
  - "Elmar Technique" means configuring the left mouse button to click on both press and release.
  - Source: [parser_raw.c header](https://github.com/thefinerminer/minesweeper-rawvf/blob/main/src/parser_raw.c)
- The owner's cheats page lists the Elmar Technique under "18. Illegal Clicking Techniques": "'Elmar Technique' is a NF method where you change the mouse to open cells on both pressing and releasing." — [minesweepergame.com: Clone Cheats](https://minesweepergame.com/clone-cheats.php)

### Inferences
- The RAW parser was revised by Damien Moore and is used by the minesweepergame.com video player. minesweepergame.com's Media page names Damien Moore as the person "who created Authoritative Minesweeper in 2000", and the About page says the site is "run by 'thefinerminer'", the same account that hosts the RAWVF repo. So its formulas are the closest available stand-in for the owner's definitions. They match the owner's Statistics page for IOE, Correctness and Throughput. ms-toollib matches them for 3BV, IOE, Corr, ThrP and ZiNi, but **not** for STNB constants.
- One identity is useful for a game implementation: IOE = Corr × ThrP. It follows from (3BV/clicks) = (eff/clicks) × (3BV/eff). **The owner page now states this outright** ("the factors of IOE are Correctness and Throughput").
- ZiNi is a heuristic upper bound on the true minimum click count with flagging. Random, two-step, 8-way and PTT variants lower it, so "the ZiNi" of a board depends on which algorithm and iteration count was used. minesweeper.online's default "ZiNi" is the PTT variant, per Gadgetlab.

### Gaps
- I could not read the original ZiNi forum thread (minesweepergame.com forum t=70; 403). The origin is known only from Gadgetlab's secondary account and ms-toollib's attribution.
- Who introduced STNB, and when its constants changed: not on any owner page I read.
- Which programs use IOS with log(Time − 1): unknown.

## Analysis tools: Arbiter, Clone, Viennasweeper, Minesweeper X, AVF/RAWVF, minesweeper.online, JSMinesweeper — what each reports

### Takeaway
The legacy clones record videos in their own formats:
- Arbiter: `.avf`
- Minesweeper Clone: `.mvf`
- Minesweeper X: `.mvr`
- Viennasweeper: `.umf` / `.rmv`
- FreeSweeper: `.fsvf`

The community-standard RAWVF text format, together with `parser_raw`, converts any of these into a common event log. It then computes the full statistics set: 3BV, solved 3BV, ZiNi/HZiNi, clicks by type, wasted clicks, IOE/Corr/ThrP/ZNE/ZNT, openings, islands, flags and misflags, and Rilian clicks.

JSMinesweeper is a different kind of tool: a probability and solver analyser that marks safe tiles, mines, dead tiles, 50/50s and the best guess. minesweeper.online shows per-game ZiNi and efficiency and offers a post-game analysis of guesses. That analysis is known only from a player guide.

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
- **Arbiter** (owner page, confirmed): "Decimal timer accurate to hundredths of a second"; "Highscores saved for Time, 3BV/s, IOE, IOS, RQP"; "Statistics counter with all popular metrics"; "Create your own custom statistics"; "Detailed searchable game history"; "Cheat modes (infinite lives, show mines...)". Ranking Rules adds that "Arbiter (stats.dat) automatically save[s] all finished games" and that "You can upload your Arbiter history to join the Arbiter Rankings". — [minesweepergame.com: Arbiter](https://minesweepergame.com/download/arbiter.php); [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- **Viennasweeper** (owner page, confirmed):
  - "Decimal timer accurate to thousandths of a second"; "Finished games are automatically saved as videos"; "Statistics window with 3BV, 3BV/s and click counts"; "Highscores saved for Time and 3BV/s".
  - "Viennasweeper is the official version used for Minesweeper tournaments and you can organise competitions on Scoreganizer. Players download a special version of Viennasweeper and videos upload automatically to the site".
  - **[corrected after primary check]** The timer-precision conflict is resolved: the owner page says thousandths for Viennasweeper and hundredths for Arbiter.
  - The checksum and the late-2024 RMV revision come from sweeper.wien and remain search-snippet only.
  - Sources: [minesweepergame.com: Viennasweeper](https://minesweepergame.com/download/viennasweeper.php); [sweeper.wien About (search snippet)](https://sweeper.wien/about.html)
- **Minesweeper X**, per Ranking Rules: "Accurate clone of Winmine with unlimited density options". It "has an 'Auto-Options' menu with some simple rules for saving videos" and "is the only version that can save statistics for lost games but its default setting does not create a history file!" — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
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
- **minesweeper.online [corrected after primary check: help pages now read]**:
  - Efficiency is defined as 3BV/clicks shown as a percentage (see above).
  - The Efficiency help page points players to replays: "A good way to learn is to watch best efficiency arena gameplay", followed by a table of record holders for "Efficiency L1" through "L8" arena levels.
  - The Gameplay page describes NG mode: "a starting position is provided, and you never need to guess to complete the board. If you get stuck, there is a free hint button at the bottom right."
  - A player guide describes the post-game guess analysis and its message "The safest move was chosen": "If the analysis tells you that you haven't played the safest move, question yourself, go and see what other guesses you may have missed using the replay feature."
  - Per Gadgetlab, games can be exported to the PTT ZiNi calculator from the ZiNi statistic.
  - Sources: [minesweeper.online: Efficiency](https://minesweeper.online/help/efficiency); [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay); [Koro: The art of guessing (Google Doc)](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI); [Gadgetlab: ZiNi: An Overview](https://docs.google.com/document/d/1dVxVY8lpIzVsODpW6sFEJMhsHD0v7vMf-TsBbu_Mx-U)
- minesweeper.online player statistics, per the community glossary:
  - "Mastery ... how many games out of a 100 you have won for each standard board size ... last 100 games"
  - "Endurance - How much time it took to solve 100 games"
  - "High Difficulty ... any board with a difficulty of 1000+. Difficulty is calculated based on winrate and misclick factor"
  - "Winstreak"
  - These are player-written descriptions, not the site's own. — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- A fan-wiki note says Minesweeper Online's speed ranking omits Beginner times "because there is no established definition of boards that are 'too lucky'". **This conflicts with minesweeper.online's own help**, which sets a Beginner 3BV limit of 5 for "the time rankings" (see the rankings section). Treat the fan-wiki claim as unverified or out of date — [minesweeper.fandom World Ranking (Speed) (search snippet)](https://minesweeper.fandom.com/wiki/World_Ranking_(Speed)); [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay)

### Inferences
- In practice, "analysis" in the speedrunning community means **video statistics**: post-game parsing of a replay into counts and ratios. Probability analysis of individual decisions is a newer layer, found in ms-toollib/Metasweeper, JSMinesweeper, and minesweeper.online's "safest move" analysis.
- **[corrected after primary check]** The earlier inference that minesweeper.online's "%" efficiency equals IOE × 100 is now confirmed by its help page.

### Gaps
- minesweeper.online's own description of its analysis feature was not found on its help pages. It is known from Koro's guide. Its probability model (exact global or local) is unknown.
- I found no primary documentation for Minesweeper Clone's built-in statistics window.
- I found no primary specification of the AVF binary layout beyond the parser source, which I did not read in detail.

## How rankings and record verification work (Authoritative Minesweeper, minesweepergame.com, WMA, minesweeper.online): board sizes, 3BV limits, luck vs skill

### Takeaway
The minesweepergame.com (Authoritative Minesweeper) rules, now verified verbatim:
- Rankings use Beginner 8×8/10, Intermediate 16×16/40 and Expert 30×16/99.
- Players are ranked by the sum of their best times. Ties go to the better score on the harder level.
- Videos must come from Arbiter 0.52.3, Minesweeper X 1.16 or Viennasweeper 3.0.2H.
- Time rankings require 3BV ≥ 2 / 30 / 100. 3BVS rankings require a Beginner minimum of 10.
- "Lucky" games (random clicks, no solving) have been banned since 2011.

minesweeper.online uses **5 / 30 / 100** for its time rankings, and its Beginner is 9x9/10. I found no evidence that a body called the "World Minesweeper Association" exists.

### Cited Findings
- Levels and tie-breaks, from Ranking Rules: "Players are ranked by the fastest sum of their time records for Beginner (8x8 with 10 mines), Intermediate (16x16 with 40 mines) and Expert (30x16 with 99 mines). Ties are broken starting with the most difficulty level; for example, a player with 5-20-75 will be ranked higher than a player with 5-19-76". Also: "Other world rankings use highest sum of solving speed (3BVS) or highest sum of mines (Density)." (Confirmed.) — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- 3BV limits (confirmed verbatim): "Boards are randomly generated so it is theoretically possible for a Beginner, Intermediate or Expert game to be solved in 1 click. Rankings for Time enforce 3BV limits of 2 (Beginner), 30 (Intermediate) and 100 (Expert) to ensure the rankings reward skill. The rankings for 3BVS enforce a limit of 10 for Beginner to give less weight to the random clicks used to start the game." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- **Lucky games (new):** "The rankings only accept games solved with skill. Beginner games solved in 1 click were banned in 2005 and 'lucky' games were banned in 2011. A 'lucky' game involves random clicks and no solving." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- Official versions (confirmed): "Official versions have identical gameplay to the original Microsoft Minesweeper ('Winmine') but save videos with statistics": Arbiter 0.52.3, Minesweeper X 1.16, Viennasweeper 3.0.2H. "Windows Vista was banned in 2006 because it introduced the ability to play the same board multiple times. All Microsoft versions were banned in 2008 due to Cheats and the fact the Community has developed secure Clones with video recording." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- **Minesweeper Clone [corrected after primary check, gap resolved]:** "Minesweeper Clone was an official version from 2004-2013. It was the only official version for the China Ranking until being banned at which point the China Ranking switched to Arbiter." So the third-party claim that Clone is currently approved is out of date. — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- Unfair prior knowledge (confirmed and expanded): "Knowing information about a board in advance is cheating. For example, using a program to read mine locations from computer memory and abusing Board Cycles to play memorised boards are both UPK. In 2008, a player hacked Minesweeper Clone 0.97 to show 3BV before starting the game and this allowed players to choose and play the easiest boards." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- Board sizes: "You can play larger boards using Arbiter (30x30) and Minesweeper X (unlimited) but for historical reasons only these 5 levels are used for rankings." — [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- Density (confirmed): "The official Density boards are 8x8, 9x9, 16x16, 30x16 and 30x24 with at least 20, 25, 60, 120 and 150 mines. The maximum density for each level is (x-1)(y-1) mines." Ranking Rules gives the reason for the cap: "increasing the density reduces the number of unique boards and makes it easier to win by luck instead of skill. For example, a Beginner (8x8) game with 63 mines has only 64 unique boards and all 64 boards can be won with 1 click." On the 9x9 origin: "The original Microsoft Minesweeper levels were Beginner (8x8), Intermediate (9x9) and Expert (30x16). Windows XP changed the Beginner grid to 9x9". **[corrected after primary check]** The first version said 9x9 "replaced 8x8"; the page also says the original Intermediate was 9x9. — [minesweepergame.com: Density Rankings](https://minesweepergame.com/density-rankings.php); [minesweepergame.com: Ranking Rules](https://minesweepergame.com/ranking-rules.php)
- **minesweeper.online (new, owner page):** "To prevent super lucky games, there are 3BV limits for beginner, intermediate and expert. These are 5/30/100 respectively. Games with a 3BV lower than the limits will not be recorded in the time rankings." The community glossary confirms: "5/30/100 respectively for WoM, 2/30/100 for Authoritative Minesweeper", and notes that WoM's Beginner is 9x9/10. **[corrected after primary check]** The first version attributed 5/30/100 only to minesweeper.now. — [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay); [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- Background articles on minesweepergame.com: Ronny de Winter, "3BV Limits on Trial" (2009), and "lucky one second beginner games" (2010). Both were found but not read — [3BV Limits on Trial PDF](https://minesweepergame.com/article/3bv-limits-on-trial-2009.pdf); [lucky one second beginner games PDF](https://mail.minesweepergame.com/article/lucky-one-second-beginner-games-2010.pdf)
- **minesweeper.info [corrected after primary check]:** minesweeper.info/wiki/Active_Ranking, /IOS, /STNB, /ZiNi, /RQP and /3BV all now redirect to the minesweepergame.com home page. The "Active Ranking" claim can't be checked. minesweepergame.com's About page says "Over the years several competing rankings have existed ... Those rankings can now be found in the Website Archive." — [minesweeper.info Active Ranking](https://minesweeper.info/wiki/Active_Ranking); [minesweepergame.com: About](https://minesweepergame.com/about.php)
- **Record figures conflict on the owner site (new):**
  - The home page says "current world records are 1 second for the Easy level, 8 seconds for Intermediate and 29 seconds for Expert".
  - world-records.php says "1 second, Intermediate 6 seconds and Expert 27 seconds" and lists the Expert record as 26.59 (Ze-En Ju, 2022-08-23, 3BV 138).
  - The world-records page is the more specific of the two.
  - Sources: [minesweepergame.com home](https://minesweepergame.com/); [minesweepergame.com: World Records](https://minesweepergame.com/world-records.php)
- Other sites use different limits. These are not Authoritative Minesweeper rules:
  - minesweeper.now: Beginner ≥ 5, Intermediate ≥ 30, Expert ≥ 100.
  - Minesweeper Pro: Beginner minimum 2.
  - Minesweeper 99: Beginner 3BV ≥ 3.
  - Source: [search summary citing minesweeper.now / minesweeper-pro / babafish](https://minesweeper-pro.com/championships/)
- No "World Minesweeper Association" turned up. The nearest bodies are minesweepergame.com, which hosts World Championship results (Budapest 2006, Vienna 2008, Zurich 2011…), and a minesweeper.now mention of an "IMESA" that I could not verify. The minesweepergame.com home page now mentions a "Minesweeper World Championship 2025" played on Viennasweeper — [minesweepergame.com home](https://minesweepergame.com/); [Vienna 2008 WC](https://minesweepergame.com/tournament/vienna-2008.php)
- Guinness: "Scores on this ranking are valid for submission to Guinness World Records." — [minesweepergame.com: About](https://minesweepergame.com/about.php)

### Inferences
- Under these rules, luck versus skill is handled by four mechanisms:
  - (a) 3BV floors that exclude boards solvable in one or two clicks
  - (b) 3BV/s rankings that normalize time by difficulty, with their own higher Beginner floor
  - (c) an explicit ban on "lucky" random-click games
  - (d) video verification on secure clones
- The two big sites use different Beginner boards and floors: 8x8 with 3BV ≥ 2 on minesweepergame.com, and 9x9 with 3BV ≥ 5 on minesweeper.online. For our game, Beginner size and the 3BV floor should be configurable.

### Gaps
- I could not read the exact verification procedure: who reviews videos, what checksum checks are made, and so on. Ranking Rules does not describe it.
- minesweeper.online's rating systems, and its rules for NG-mode leaderboards specifically, were not found on the help pages I read.

## How experts classify a board as needing guesses; counting forced guesses

### Takeaway
Experts and their tools sort guesses into three kinds:
- **Forced guesses**: no safe cell can be deduced, even with future information and mine count. These include unavoidable 50/50s, meaning a pair of cells sharing one mine where neither can ever receive information without the other.
- **Needless guesses**: a guess made while a safe cell existed.
- **Risk-weighted luck**: the product of the safety probabilities of every risky click.

A board is "no-guess" if a deterministic solver, starting from the first click, can clear it by logic and mine counting alone. The community glossary on minesweeper.online uses the same definitions.

### Cited Findings
- Community definitions, verbatim:
  - "Forced guess - a situation that must be solved with a guess regardless of future information (including minecount)"
  - "NG ... Any NG board will be solvable with standard logic and/or minecounting"
  - "NG Logic - Any form of logic that is exclusive to NG boards, and uses the concept that any mine combination that involves guessing is invalid"
  - "50/50 Breaker - A pattern exclusive to No-Guessing boards"
  - Source: [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- Scar's 50/50 definition: "It applies to situations where this is certain - when no information will be able to differentiate between the two cells." Linked 50/50s: "one guess resolves the situation, the other ends in another 50/50." — [Scar: What is a 50/50 (Google Doc)](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)
- minesweeper.online NG mode: "you never need to guess to complete the board ... Higher difficulty NG boards require the use of more complex logical patterns. Evil difficulty contains at least one advanced situation in every game." Its pattern catalogue tags each pattern with the NG level that requires it (Easy / Medium / Hard / Evil; see pro_technique.md §2). — [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay); [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- JSMinesweeper's definition of a 2-tile 50/50: "a pair of tiles which share 1 mine where one of the tiles can never receive information without the other tile also receiving the same information… they must be (and always will be) an unavoidable guess". It detects extended 2-tile 50/50s and enclosed 2×2/2 boxes — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- JSMinesweeper's pseudo-50/50: "a set of tiles is either safe or part of a 50/50… correct to guess immediately". It detects 2-tile and 4-tile 2×2 pseudo-50/50s but not extended ones — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md)
- JSMinesweeper's dead tiles: tiles that "are either mines or have only one possible value… if all the tiles left in the game are dead… the result is down to luck". Koro gives the player-side version: dead cells "have absolutely no progress, regardless of the number revealed" — [JSMinesweeper README](https://github.com/DavidNHill/JSMinesweeper/blob/master/README.md); [Koro: The art of guessing](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)
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
- Counting forced guesses depends on the solver. A board's "guess count" is the number of times a perfect-logic solver (including global mine-count reasoning) has no safe move. The community definition of "forced guess" includes mine count; ms-toollib's `is_solvable` ignores it. So the two can disagree on endgame positions.
- minesweeper.online grades NG boards by the hardest pattern tier needed. A generator can do the same: run a tiered solver and record the highest tier used.
- For a game implementation, it is worth reporting the guess count, 50/50s, and a pluck-style luck score separately from speed and efficiency metrics, since the community treats luck and skill as separate axes.

### Gaps
- I found no official minesweepergame.com definition of "forced guess", and no rule that excludes guess-heavy boards from records. Records are filtered by 3BV and by the "lucky games" ban, not by guess count.
- I could not find how minesweeper.online *generates* NG boards (algorithm, start cell). Only the outcome is documented: no guesses, tiered pattern difficulty, and a free hint button.

## Verification log

Checked 2026-11-06. Method: curl with a browser user agent for minesweepergame.com, headless Chrome for minesweeper.online, and Google Docs text export for the guides.

**Confirmed (snippet tags replaced with verbatim owner citations):**
- Statistics: the 3BV definition, counting rule and full name; 3BV/s; IOS = log(3BV)/log(Time); RQP = Time/3BV/s and its full name; IOE with ">1.00 by using chording"; Correctness; Throughput with the 0.81 example; ZiNi's purpose and its three versions; ZiNi/s, ZNE and ZNT.
- Ranking Rules: levels and tie-break; the 2/30/100 time limits; the 3BVS Beginner limit of 10; official clones; Microsoft ban (2008) and Vista ban (2006); registration and video upload; UPK; only 5 levels ranked.
- Density boards and minimums.
- Arbiter: high scores for Time, 3BV/s, IOE, IOS, RQP.
- Viennasweeper: statistics window, tournaments, Scoreganizer, autosave.
- "3BV favours NF" and "ZiNi for flaggers", now cited to owner pages.

**Corrected [corrected after primary check]:**
- 3BV/s is defined on Statistics, not Ranking Rules.
- 3BV is a no-chord, no-flag count on both owner sites.
- IOS conflict: the owner uses log(Time); log(Time−1) is a community variant.
- The owner's RQP "increases with faster scores" wording contradicts its own formula.
- The owner's Efficiency page words IOE backwards.
- minesweeper.online efficiency = IOE × 100 is confirmed (was an inference).
- Viennasweeper timer is thousandths, Arbiter hundredths (conflict resolved).
- Minesweeper Clone was official 2004–2013 and is now banned (gap resolved).
- 5/30/100 is minesweeper.online's own rule.
- The 9x9 history: the original Intermediate was 9x9; XP made Beginner 9x9.
- minesweeper.info wiki pages now redirect, so the Active Ranking claim is unverifiable.
- The fan-wiki "Beginner omitted" claim conflicts with minesweeper.online's help.

**Newly added:**
- The "lucky games" ban (2011) and the 1-click Beginner ban (2005).
- Decimal-time rounding.
- The density-cap rationale.
- Minesweeper X history-file notes.
- The CeS statistic.
- minesweeper.online: efficiency definition, NG mode with hint button, efficiency arena replays, post-game "safest move" analysis (via Koro), and ZiNi settings and variants (via Gadgetlab).
- ZiNi origin, July 2009, "3BV for flaggers" (via Gadgetlab), with the premium formula; 8-way, Legacy and PTT ZiNi.
- H.ZiNi tie-break conflict between community docs.
- The community definitions of forced guess, NG, NG logic and 50/50 breaker.
- The OBV metric.
- The owner-site record-figure inconsistency.

**Still unverified:**
- The original ZiNi forum thread (t=70): 403 to every method; web.archive.org unreachable.
- STNB on any owner page: absent from Statistics; only the fan wiki and ms-toollib give it.
- The Arbiter changelog line about Correctness in legal mode.
- Viennasweeper checksum and 2024 RMV revision (sweeper.wien only).
- The Bechtel PDF and the 3BV-limits background PDFs: not read.
- The video verification procedure.
- minesweeper.online's NG generation algorithm and leaderboard rating rules.
- All somnolescent and fandom snippets, which were not in the re-check scope.
