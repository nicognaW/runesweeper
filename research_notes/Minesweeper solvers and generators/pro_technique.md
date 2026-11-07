# How top human Minesweeper players play and teach: deduction techniques, patterns, guessing heuristics

> **Read this first: how these notes were made.** The first version of these notes was written while minesweepergame.com, minesweeper.online, nothings.org, minesweeper.info, Wikipedia and web.archive.org were blocked, so it relied on search-engine excerpts. On 2026-11-06 the notes were **re-checked against the primary pages**:
> - The minesweepergame.com strategy pages were fetched with curl and a browser user agent.
> - The minesweeper.online help pages are rendered by JavaScript, so they were read through headless Chrome.
> - Sean Barrett's page at www.nothings.org was fetched directly.
> - The community guides linked from minesweeper.online's Guides page were read as Google Docs text exports.
>
> Quotes below are verbatim from those pages. Corrections to the first version are marked **[corrected after primary check]**. The Verification log at the end lists what was confirmed, corrected, and still unverified.
>
> Still unreachable: the minesweepergame.com forum (403 to curl and to headless Chrome), web.archive.org (connection reset through the proxy), and YouTube video content. minesweeper.info/wiki/* now redirects to the minesweepergame.com home page.
>
> All solutions marked **[derived]** come from working through the constraints by hand. All ASCII diagrams are mine; the sources use PNG images, which I did not copy.

Diagram notation used throughout: `#` = unopened cell, `F` = flag/known mine, `.` = opened blank, `|` or `===` = wall/board edge or an already-cleared row. `M` = deduced mine, `S` = deduced safe. `?` = an open number the diagram's deduction doesn't read.

---

## 1. Basic rules and how teachers phrase them

### Takeaway
Teachers start with two single-number rules. If a number touches as many unopened cells as its value, all of them are mines. If a number already touches that many flags, every other neighbour is safe, and chording opens them in one action. minesweepergame.com teaches the first rule with a picture for every number 1 to 8, starting with a corner 1, and it warns about the "fake" corner 1. minesweeper.online labels the two rules **B1** and **B2**.

### Cited Findings
- minesweepergame.com "How To Play" gives the controls: "You open squares with the left mouse button and put flags on mines with the right mouse button. Pressing the right mouse button again changes your flag into a questionmark." An empty square opens its neighbours "automatically ... in all directions until reaching squares that contain numbers." The page also says "Windows Minesweeper always makes the first click safe" and "A common strategy for starting games is to randomly click until you get a big opening with lots of numbers." — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- Rule 1 on the same page: "When a number touches the same number of squares, those squares must be mines." The first example is "The 1 on the corner touches 1 square so it must be a mine", followed by one example each for 2 through 8. — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- Rule 2 is taught through chording: "If you flag all of the mines touching a number, chording on the number opens the remaining squares. Chording is when you press both mouse buttons at the same time ... if you place the correct number of flags on the wrong squares, chording will explode the mines." The page's counting trick: "press both buttons ("chord") on a number to depress (but not open) the squares it touches. This shows you how many squares are touching the number." — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- **New, found on the primary page:** a warning about the "fake" corner 1. "A common mistake is to see a 'fake' 1 on a corner where the 1 already touches a mine ... The pink square is not a mine." The same page shows a "pink 2 [that] touches three squares and cannot be solved". — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- minesweeper.online states the two rules as patterns. **B1**: "If a number is touching the same number of cells, then these cells are all mines." **B2**: "If a number is touching the same number of flags, then all adjacent cells can be opened." — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **[corrected after primary check]** The first version said How To Play "warns that new players flag and chord too much" and advises "flagging only the mines you plan to chord on". **That advice is on the Efficiency page, not How To Play.** The Efficiency page says: "The first rule is to never flag a mine unless you are going to chord on it", and "New players flag and then chord multiple times on all nearby numbers, hoping to open more squares. You should practice slowly at first". — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)

### Inferences
- Rule 1 ("saturated by unknowns"): `number − flagged neighbours = unopened neighbours` ⇒ all unopened neighbours are mines. Rule 2 ("satisfied"): `number − flagged neighbours = 0` ⇒ all unopened neighbours are safe. Both are the single-constraint case of "reduction" (section 2). **[derived]**
```
Rule 1 (corner):         Rule 2 (satisfied):
| 1 #                     F 1 #
| . .     -> # is M       . . #    -> both # are S (chord the 1)
```
- The "fake 1" warning is Rule 1 applied without first reducing the number. A hint system should check flagged or known mines before it applies Rule 1.

### Gaps
- None remaining for this section.

---

## 2. Named patterns, reduction, and subset/difference reasoning

### Takeaway
Both main teaching sites present patterns as memorised shortcuts.
- minesweepergame.com: "A pattern is a common arrangement of numbers with only one solution. Learning patterns is important because thinking wastes time."
- minesweeper.online: "A pattern is a common arrangement of numbers that has only one solution. If you memorise a pattern it will reduce the amount of time you use thinking."

minesweepergame.com builds everything on two root patterns:
- **1-2-X**: X is a mine.
- **1-1-X** from a border: X is safe.

It then generalises them to "subsets" and "reduction".

minesweeper.online has a 28-entry catalogue in these groups:
- Basic: B1, B2, 1–1, 1–1+, 1–2, 1–2+, 1–2C, 1–2C+, 1–2–1, 1–2–2–1
- Reduction: 1–1R, 1–2R, 1–2–1R
- Holes: H1–H3
- Triangles: T1–T5
- High complexity: 1–3–1 corner, 2–2–2 corner, 1>2<1, T–pattern, Dependency Chain
- Last turns: Mine counting, Combinations

**Each entry is tagged with the No-Guessing (NG) difficulty level at which it is needed** (Easy / Medium / Hard / Evil). That is a ready-made difficulty ladder for a generator.

### Cited Findings
- minesweepergame.com: "The two most famous patterns are 1-2-1 and 1-2-2-1. You should memorise these immediately." **[corrected after primary check]** The first version quoted this as "the two best-known patterns". — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **1-2-X**, as minesweepergame.com puts it: "The 1-2-1 and 1-2-2-1 patterns are actually combinations of a more important pattern. When you see 1-2-X on a row the X is always a mine." The page then explains: "There are two mines in three squares (because the 2 touches three squares) but there is one mine in the first two squares (because the 1 touches two of the three squares). The third square must contain the other mine." It solves 1-2-1 and 1-2-2-1 by "Apply the 1-2-X pattern from the left" and then "from the right". — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **1-1-X**, as minesweepergame.com puts it: "The other important pattern is 1-1-X on a row starting from a border. The first 1 touches two squares and the second 1 touches three squares. Both clues are true so the third square must be empty. This logic is the opposite of the 1-2-X pattern". It adds that "The 1-1-X pattern can also wrap around corners", with examples where a 1 touches five squares and the yellow 1 is a two-square subset. — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **Subset logic** ("Advanced Logic" on minesweepergame.com): "Sometimes a mine is in a subset of squares so the remaining squares must be safe. This is how the 1-1-X pattern works." The page also teaches the mine-finding direction: "Sometimes it is better to find mines instead of only looking for safe squares", for example "The 3 touches four squares. The yellow 1 touches a subset of two squares. The second and third mines must be in the pink squares." It then shows chaining: "You can combine patterns and logic to form chains of logic alternating between finding mines, using 1-1-X and using subsets. One of the earliest Minesweeper games was called Relentless Logic!" — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **Reduction**, as minesweepergame.com puts it: "Complex mine arrangements can be reduced to patterns. You simply subtract the known mines from each number. For example, if you flag a mine touching a 3 it becomes a 2." Its harder examples "are more complicated because you need to solve the entire situation in your head to see the pattern. If you do not see the patterns take turns reducing numbers and using 1-2-X". — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- minesweeper.online, **basic group** (verbatim key phrases):
  - **1–1**: the right 1 "also touches the two yellow cells, so it already has a mine and the third cell can be opened."
  - **1–1+**: the same, but "all the remaining cells can be opened."
  - **1–2**: "Its first mine is in the yellow cells, therefore, the second mine is in the third cell."
  - **1–2+**: the second number is a 4 that "also touches the two yellow cells, so it already has one mine, therefore, as there are only three remaining cells, they must be mines."
  - **1–2C** ("Classic version of the 1–2 pattern"): the purple cells "can't both be mines, because that would overflow the 1. They contain a maximum of one mine ... there must be a mine in the remaining third cell."
  - **1–2C+**: the same, with a 4: "there must be three mines in the three remaining cells."
  - **1–2–1**: "Actually, it's just a combination of two 1–2 patterns."
  - **1–2–2–1**: "It's also just a combination of two 1–2 patterns", applying "1–2 ... from the left" and "2–1 from the right".
  - Source: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- minesweeper.online, **reduction group**:
  - **1–1R**: "2 turns into 1, since it already has one mine. 4 also turns into 1, since it already has three mines. Therefore, 2–4 turns into the 1–1 pattern and the third cell can be opened."
  - **1–2R**: "2–3 turns into the 1–2 pattern."
  - **1–2–1R**: "2–3–2 turns into the 1–2–1 pattern."
  - Source: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **Holes H1, H2, H3** (minesweeper.online): all three have the same text: "Look at the bottom 1. It touches the two yellow cells, so they contain one mine. Now look at the top 1. It also touches the two yellow cells, so it already has a mine and all the remaining cells can be opened." They differ only in the picture: the geometry of the hole. So holes are the 1–1+ subset rule applied across a hole in the frontier. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
  - Layouts, here and for the triangles and the high-complexity group: each grid is the fixture for that pattern in `src/hint/tests/catalogue.rs` (which `src/hint/patterns.rs` recognises), cropped to the cells around the numbers the pattern reads. Legend: a digit is a number the pattern reads; `?` an open number it doesn't read; `.` an opened blank; `#` hidden and not settled; `S` proven safe and `M` proven mine by the read numbers together (checked by enumerating them); `|` or `===` a wall. None of these patterns has a read number touching a wall, so no wall is drawn. Where a fixture follows a diagram, the source cites its last step, `https://minesweeper.online/img/help/<code>/<n>.png`, which shows the same numbers and hidden cells with the solution marked; the fixture's other numbers can differ, since it fills in the field around the diagram so that it is consistent.
  - **H1**: the bottom 1's two cells lie either side of the top 1.
    ```
    S S S
    # 1 #
    ? 1 ?
    . . .
    ```
    Source: minesweeper.online diagram h1/4 (fixture `hole_1`).
  - **H2**: the bottom 1's two cells lie either side of the bottom 1 itself, so the hole is one row deeper.
    ```
    S S S
    ? 1 ?
    # 1 #
    ? ? ?
    ```
    Source: minesweeper.online diagram h2/4 (fixture `hole_2`).
  - **H3**: the two 1s are two apart; the bottom 1's two cells are the cell between them and one beside it.
    ```
    S S S
    ? 1 S
    ? # #
    ? 1 ?
    . . .
    ```
    Source: minesweeper.online diagram h3/4 (fixture `hole_3`).
- **Triangles T1–T5** (minesweeper.online) **[corrected after primary check]**. The first version said only that "entries start from a 1 at the bottom", with the excerpt cut off. The full logic:
  - **T1**: subset rule with three cells. "It touches the three yellow cells, so they contain one mine ... the top 1 ... already has a mine and all the remaining cells can be opened."
    Layout: the bottom 1's three cells are both cells beside the top 1 and one beside the bottom 1, so the open cells come to a point.
    ```
    S S S
    # 1 #
    # 1 ?
    ? ? .
    ```
    Source: minesweeper.online diagram t1/4 (fixture `triangle_1`).
  - **T2**: "The purple cells ... contain a maximum of one mine, because they touch 1 ... [the 2 has] a mine in the remaining cell. If you put the flag, you can see that there is one mine in the purple cells. So 1 already has a mine and all the remaining cells can be opened." This is a two-step chain: a 1-2C-style bound, then 1-1+.
    Layout: the 2 below the 1 shares three cells with it, coming to a point as in T1; the 2's one cell of its own is the mine.
    ```
    S S S
    # 1 #
    # 2 ?
    M ? .
    ```
    Source: minesweeper.online diagram t2/5 (fixture `triangle_2`).
  - **T3**: the same with a 2 and a 3: "a maximum of two mines ... there must be a mine in the remaining cell", then the 2 is satisfied.
    Layout: T2's shape with a 3 below a 2.
    ```
    S S S
    # 2 #
    # 3 ?
    M ? .
    ```
    Source: minesweeper.online diagram t3/5 (fixture `triangle_3`).
  - **T4**: a 2 and a 4: "there must be two mines in the remaining cells", then the 2 is satisfied.
    Layout: the 4 beside the 2 shares three cells with it, both beside the 2 and one beside the 4; the 4's two cells of its own are mines and the 2's own cells are safe.
    ```
    S # # M
    S 2 4 ?
    S # ? M
    ```
    Source: minesweeper.online diagram t4/5 (fixture `triangle_4`).
  - **T5**: like T2 with a different geometry: "a mine in the remaining third cell".
    Layout: the 2 below the 1 shares only the two cells either side of the 2; the 2's one cell of its own is the mine.
    ```
    S S S
    ? 1 ?
    # 2 #
    M ? ?
    ```
    Source: minesweeper.online diagram t5/5 (fixture `triangle_5`).
  - Source: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **High-complexity group** (minesweeper.online):
  - **1–3–1 corner**: two disjoint pairs, each "a maximum of one mine" because each touches a 1, "so the third mine must be in the last corner cell", and then each pair holds exactly one.
    Layout: the 3 with a 1 beside it along each side of its corner; each 1 shares two cells with the 3, and the 3's last cell is the corner cell.
    ```
    S # # M
    ? 1 3 #
    . . 1 #
    . . ? S
    ```
    Source: minesweeper.online diagram 1-3-1/7 (fixture `corner_131`).
  - **2–2–2 corner** (the only 2-2-2 pattern in either catalogue): two pairs that each "can't both be empty, because that would overflow the 2. So they contain a minimum of one mine"; "The purple and orange cells contain a minimum of two mines, so 2 already has mines and the last corner cell can be opened." This is the **minimum-bound** counterpart of 1-2C's maximum bound.
    Layout: the 1-3-1 corner's shape with three 2s. Each outer 2 has one cell of its own, so it shares at least one mine with the middle 2; the corner cell is safe and the outer 2s' own cells are mines.
    ```
    M # # S
    ? 2 2 #
    . . 2 #
    . . ? M
    ```
    Source: built from the described rule (fixture `corner_222`).
  - **1>2<1**: a 2 whose two mines must be split, one in each 1's region, "Therefore, all the remaining cells can be opened."
    Layout: the 2 is one row below the two 1s and between them; it shares two cells with each 1, and each pair holds one mine, so everything else around the 1s is safe.
    ```
    S S S # S S S
    S 1 # ? # 1 S
    S S # 2 # S S
    # ? ? ? ? ? #
    ```
    Source: built from the described rule (fixture `staggered_121`).
  - **T–pattern**: combines two 1-pairs, a 2, and a 3. "The yellow and orange cells contain a maximum of two mines, so the third mine must be in the last corner cell."
    Layout: a 3 and a 2 side by side, a 1 below the 3 and a 1 two rows below the 2; the 3's last mine is its top-left corner cell, and the 2's cells on the right are safe.
    ```
    M # # S
    ? 3 2 S
    # 1 # #
    ? ? 1 ?
    . . . .
    ```
    Source: minesweeper.online diagram tp/6 (fixture `t_pattern`).
  - **Dependency Chain**: "Usually long dependency chains can be resolved by parsing from both sides." The example tracks one-mine regions from the top and from the bottom until a 2 links them.
    Layout: the recogniser calls any deduction that reads five or more numbers a dependency chain. The fixture is a row 1-2-2-2-2-1 between two hidden rows; five of its numbers (all but the second 2 from the right) prove the cells at both ends, and the two cells above and below the first 2, safe.
    ```
    S # S # # # # S
    S 1 2 2 ? 2 1 S
    S # S # # # # S
    ```
    Source: built from the described rule (fixture `dependency_chain`).
  - Source: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **Last-turns group** (minesweeper.online):
  - **Mine counting**: "When the game comes to an end, you can determine empty cells by the number of remaining mines ... The colored cells contain 6 mines in total, just as much as the number of remaining mines. Therefore, cells marked with green circles can be opened."
  - **Combinations**: "In rare situations, it may be necessary to enumerate possible combinations: Choose an arbitrary cell ... If there is a mine in the yellow cell, then for the final solution we need 6 flags. This combination is not possible, because there are only 5 mines left."
  - The page ends by linking a community doc, "Advanced Solving Tactics with Practice Puzzles".
  - Source: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **NG difficulty tags (new).** Each catalogue entry carries a badge with the tooltip "NG Level: Easy/Medium/Hard/Evil". I read these from the rendered DOM:
  - Easy: B1, B2
  - Medium: 1–1, 1–1+, 1–2, 1–2+, 1–2C, 1–2C+, 1–2–1, 1–2–2–1
  - Hard: 1–1R, 1–2R, 1–2–1R, H1–H3
  - Evil: T1–T5, 1–3–1 corner, 2–2–2 corner, 1>2<1, T–pattern, Dependency Chain, Mine counting, Combinations
  - The Gameplay help page explains: "Higher difficulty NG boards require the use of more complex logical patterns. Evil difficulty contains at least one advanced situation in every game."
  - Sources: [minesweeper.online: Patterns](https://minesweeper.online/help/patterns); [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay)
- A community glossary linked from minesweeper.online's Guides page (Archeaic, "Minesweeper Terminology") gives the NG level sizes:
  - Easy: 9x9/10
  - Medium: 16x16/40
  - "Hard (NG) ... there is a guarantee of no guesses if one uses all patterns designated 'Hard' difficulty and lower. (30x16/99)"
  - "Evil (NG) - 30x20/130, can be solved without guessing, and guarantees a pattern with a depth of 3 or above, that also has 'evil' status."
  - This is a player-written document, not the site's own rules. — [Archeaic: Minesweeper Terminology (Google Doc)](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- **History**: minesweepergame.com's math-papers index says Philip Crow's 1997 paper "includes an early description of the 1-2-1 pattern and the first mention of 'relative' patterns where mines touching numbers can reduce those numbers to a known pattern." (Confirmed verbatim.) The site's strategy Links page goes further: "Minesweeper Guide (1992): First mention of the 1-2-1 and 1-2-2-1 patterns", and "Minesweeper Tips (1997): ... a brief mention of reduction ('equivalency')". **[corrected after primary check]** So Crow is not the earliest source for the patterns themselves; the 1992 guide is. — [minesweepergame.com: Math Papers](https://minesweepergame.com/math-papers.php); [minesweepergame.com: Strategy Links](https://minesweepergame.com/strategy/links.php)
- The No Flags page: "The special skill of NF technique is 'seeing' mines in order to use patterns such as 1-1-X and subsets to search for openings." — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php)
- One search summary said the 1-2-1 mine sits under the 2. **That is wrong**, and both primary pages now confirm it: 1-2-1 is solved by applying 1-2 from both sides. Low-quality SEO pattern sites (e.g. minesweeper.now, minesweeperblast, rarepike) were not used. — [search noise, e.g. minesweeper.now Pattern Bible](https://minesweeper.now/blog/the-minesweeper-pattern-bible-how-to-recognize-1-2-1-1-2-2-1-and-2-1-2-formations)

### Inferences
All solutions below are **[derived]** by working through the constraints. Each assumes a straight frontier, with the unopened row underneath and the row above already cleared or a wall.

**1-1-X from a wall** (minesweepergame.com's name; minesweeper.online's 1–1): the 1 against the wall sees only two cells, so the next 1's third cell is safe.
```
| 1 1 ?          numbers (cleared row)
| # # S          a,b share 1 mine; c is S
```
**1-2-X (general)**: the shared pair holds at most 1 mine (because of the 1), so the 2's far cell is a mine. The shared pair then holds exactly 1, so the 1's far cell is safe.
```
  ? 1 2 ?
  S # # M
```
**1-2-1 = 1-2 applied from both ends.** The mines sit under the 1s. The cell under the 2 is safe, and so are both cells beyond the 1s.
```
  1 2 1
# M S M #   ->  outer # are S
```
**1-2-2-1 = 1-2 applied from both ends.** The mines sit under the two 2s. The cells under the 1s are safe, and so are both cells beyond them.
```
  1 2 2 1
S S M M S S
```
**Reduction example**: an `F` touching a 2 makes it act as a 1, and that 1 can complete a 1-2-1 (cf. minesweeper.online's "2–3–2 turns into the 1–2–1 pattern"):
```
  F             F touches the left "2"
  2 2 1   reduces to   1 2 1  ->  same solution as 1-2-1
```
**Subset/difference rule**: this is the general principle behind every catalogue entry up to "Combinations". minesweepergame.com states it partly ("a mine is in a subset of squares so the remaining squares must be safe"). Neither site states it in full.
- Let A and B be two numbers after reduction, with unopened neighbour sets U(A) and U(B).
- If U(A) ⊆ U(B), then U(B)\U(A) holds exactly B−A mines.
  - If B−A = 0, every cell in the difference is safe: that is 1-1, holes, and T1.
  - If B−A = |U(B)\U(A)|, every cell in it is a mine: that is 1-2 next to a wall, and 1-2+.
- When the sets only partly overlap, the shared region holds at most min(A,B) mines: that is 1-2C, T2–T5, 1-3-1 corner, and 1>2<1.
- The shared region also holds at least A − |U(A)\U(B)| mines: that is the 2-2-2 corner.

The "Evil" entries chain these bounds; "Combinations" is case enumeration.
- **Generator difficulty ladder**: minesweeper.online's NG tags give an outside reference for grading. Single-cell rules (Easy) → pairwise subset with one number in each pair (Medium) → reduction and holes, which need flags or hole geometry (Hard) → bound-chaining, multi-region, and global mine count (Evil). A generator that solves with a tiered rule set can label boards the same way.
- **Learning order**: both sites run basic rules → 1-1/1-2 family → 1-2-1/1-2-2-1 as compositions → reduction → holes and triangles → multi-region and mine counting. The teachers want 1-2 and 1-1 seen as the roots.

### Gaps
- No "box" pattern under that name appears on either catalogue. Enclosed 2x2 50/50s are taught under "50/50" (section 3), not as logical patterns.
- The layouts of H1–H3, T1–T5 and the high-complexity entries are now recorded under each entry, from the hint fixtures. The 2–2–2 corner, 1>2<1 and Dependency Chain layouts were built from the described rule, not from a diagram. On 2026-11-07, `img/help/2-2-2/<n>.png` (steps 0–6) and `img/help/dc/<n>.png` (steps 0–8) were found to exist, but those two layouts have not been compared with them. No image code was found for 1>2<1.
- Neither pattern page names an author. minesweepergame.com's About page says the site "is a non-profit site run by 'thefinerminer'". Its Media page describes Damien Moore as the person "who created Authoritative Minesweeper in 2000". The strategy pages carry no byline. — [minesweepergame.com: About](https://minesweepergame.com/about.php); [minesweepergame.com: Media](https://minesweepergame.com/media.php)

---

## 3. Advanced: mine counting, global reasoning, linked 50/50s, dead squares

### Takeaway
Sean Barrett's "Minesweeper: Advanced Tactics" (nothings.org; dated 1999 by minesweepergame.com's Links page) is the classic advanced guide. It works through one real Expert position from local probabilities to an exact count of 118 arrangements. Its key rule of thumb is "fewer mines means more unencountered variants".

**[corrected after primary check]** The first version said there was no first-party teaching material on linked 50/50s or dead squares. There is: community guides linked from minesweeper.online's Guides page teach both.
- Scar's "What is a 50/50" defines forced 50/50s, the common patterns, **linked 50/50s** and **pseudo 50/50s**.
- Koro's "The art of guessing" defines **dead cells** and states the fewer-mines principle.

### Cited Findings
- **Barrett's section order** (verbatim headings):
  1. "A Nasty Minesweeper Position"
  2. "Minesweeper: Logic or Probability"
  3. "An Endgame Tactic"
  4. "Local Probabilities"
  5. "Resolving Local Probability Conflicts"
  6. "Counting Arrangements"
  7. "Counting Unencountered Mines"
  8. "A Final Solution"
  9. "Playing the Game"
  - Source: [nothings.org: Minesweeper: Advanced Tactics](https://www.nothings.org/games/minesweeper/)
- Barrett on logic versus probability: "Technically, probability subsumes logic. If you can logically prove a mine must be in a location, its probability must be 100%". — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Barrett on mine counting: "One very easy endgame tactic you can use is counting the number of remaining mines ... If you reach this position and the counter says there are only two mines left, you're done; it must be position B." (Confirmed verbatim.) — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Barrett defines local probability: "By locally, I mean that if you have a '1' next to two unknown squares, each has a 50% chance of being a mine." (Confirmed.) On why local probabilities can't simply be combined: "The data the probabilities are derived from aren't independent of one another, so no direct mathematics on the probabilities is valid. The reason a local guess of 50% is correct in the bottom center is because it really is independent of anything else." **[corrected after primary check]** The first version attributed the "really [are] independent" phrase to an HN commenter. It is Barrett's own text. — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Barrett's counting method: "measure across all possible arrangements that might have led to the current situation, assuming each was equally probable." In his worked bottom-right example with three mines left, configuration B (two mines) has nine variants against A's one, so "configuration B is 90% likely". The figures continue: 75% with four mines, "just over 66%" with five, "about 60% likely" with six, "only 50% likely" with seven, and "less likely than A" with eight. (Confirmed.) — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Barrett's rule of thumb, verbatim: "I applied the 'fewer mines means more unencountered variants' rule of thumb (which applies roughly until the number of unencountered-squares is less than double the number of unaccounted-for mines), which means that configurations that use fewer mines are much more likely." (Confirmed.) — [nothings.org](https://www.nothings.org/games/minesweeper/)
- **Linked mines in Barrett.** After the full count ("a total of 118 possible combinations"; configuration B 87%), he notes: "we know that the probability of the the two dark green '87' squares are linked--if one is true, the other must be." This is the earliest "linked" language I found. — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Barrett on information: "ideally I'd pick a square that will maximize the likelihood I'd get useful information". He took the 87% side and still lost: "Eight times out of nine, I would have been right." — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Date: minesweepergame.com's Links page lists "Minesweeper Advanced Tactics (1999): Detailed analysis of global probability". Barrett's own games index links "Advanced Minesweeper Tactics (a careful probabilistic analysis)" and gives no date. — [minesweepergame.com: Strategy Links](https://minesweepergame.com/strategy/links.php); [nothings.org/games](https://www.nothings.org/games/)
- minesweepergame.com Guessing: "Local probability is easy to calculate but global probability is much more difficult ... Sean Barrett has written an excellent guide to Minesweeper Advanced Tactics." (Confirmed.) — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- **Forced 50/50 definition** (Scar, "What is a 50/50", linked from minesweeper.online Guides): "a situation where there is a forced guess between cells, with a 50% chance of a mine being in one cell, and a 50% chance it is in the other. It applies to situations where this is certain - when no information will be able to differentiate between the two cells." The reason: "There are two things in the game that don't give information: Mines, and the edges of the board." The guide lists Pattern 1 and Pattern 2 in Full, Wall and Corner variants, plus "Pattern 1 Extended". It notes that the Pattern 2 corner "is only a confirmed 50/50 if it must have 2 mines. If there are 1 or 3 mines remaining on the board, it can be solved via minecount." — [Scar: What is a 50/50 (Google Doc)](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4); listed at [minesweeper.online: Guides](https://minesweeper.online/help/guides)
- **Linked 50/50** (Scar): "In some situations there are what looks like multiple 50/50s. However, one guess resolves the situation, the other ends in another 50/50." Before guessing: "Make sure there is no linked 50/50 ..., where guessing one cell may resolve a second 50/50." — [Scar: What is a 50/50](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)
- **Pseudo 50/50** (Scar): "situations where a cell is either safe, or a 50/50. In both situations it makes sense to click, so you should guess pseudo 50/50s immediately." Scar adds that a pseudo 50/50 can also arise "via minecount". — [Scar: What is a 50/50](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)
- Koro's formal version: "A cell is part of a pseudo 50/50 if and only if ... If the contents of the cell can be known with logic, then it is safe [and] If the cell is a mine, then it's part of a 50/50." — [Koro: Koro-Tzu, The art of guessing (Google Doc)](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)
- **Dead cells** (Koro): "'Dead cells' are cells with absolutely no progress, regardless of the number revealed. They should not be played, even if the survival probabilities are good, as you will be forced to guess another cell immediately afterwards." Example: "If you survive this move, the number revealed will always be a 3. And you can't do anything with it." On dead cells inside 50/50s: "The two black cells are dead ... In both cases, you end up having to play another 50/50. So you have to play the left 50/50 to ensure you only guess once." Also: "dead cells are the biggest enemy of hint users." — [Koro: The art of guessing](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)
- The community glossary separates "Forced guess - a situation that must be solved with a guess regardless of future information (including minecount)" from "50/50 ... It is also common to refer to any forced guess, or even any guess, as a '50/50'". It also defines "Floating Cell - An unopened cell that is only touched by other unopened cells." — [Archeaic: Minesweeper Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)

### Inferences
- **Why Barrett's rule holds [derived]**: suppose R unknown cells lie off the frontier and M mines are unaccounted for. A frontier configuration using k mines has weight C(R, M−k). The ratio of a k-mine configuration to a (k+1)-mine one is C(R,M−k)/C(R,M−k−1) = (R−M+k+1)/(M−k). That ratio exceeds 1 roughly when R > 2(M−k), which is exactly Barrett's "less than double" threshold. Barrett's 9-cell example checks out: with 3 mines, C(9,1)/C(9,0) = 9 → 90%. A solver or generator can use the same weighting.
- **Endgame mine counting [derived]**: if a frontier can be solved with either m or m+1 mines and the counter says m, the m+1 solutions are eliminated. minesweeper.online's "Mine counting" and "Combinations" entries are exactly this.
- The community vocabulary (forced 50/50, linked 50/50, pseudo 50/50, dead cell, floating cell) matches JSMinesweeper's analyser categories (see expert_analysis.md). The guessing-patterns guide on minesweeper.online credits "MSCoach - JSMinesweeper (primary analysis tool & board editor)", so the terms and the tool have grown up together. — [Guessing Patterns (Google Doc)](https://docs.google.com/document/d/12JaI7J9WDVKSwA7Smuzl5L0IfN_cKBAms4Ru8puEXAw)

### Gaps
- I could not see Barrett's GIF figures (mine_exclude.gif and others), so his board position is not reproduced.
- The Scar and Koro guides depend on images. Their text was read, but the diagrams of each 50/50 pattern were not.

---

## 4. Guessing: how strong players choose when forced

### Takeaway
minesweepergame.com's Guessing page lists **six numbered strategies**:
1. Guess quickly.
2. Guess only when forced; solve elsewhere first.
3. Play NF and look for safe squares instead of mines.
4. Make the most useful guess.
5. Use mine density: a random square can beat a 50:50.
6. Calculate global probability.

The page frames the choice as goal-dependent. Community guides add the "fewer mines" principle, "progress", and avoiding dead cells. Murański's own article says: "always open a square with the least chance of hitting a mine."

### Cited Findings
- "The optimal guessing strategy depends on whether your goal is to win or to win fewer games more quickly." (Confirmed.) — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- Strategy 1: "The first strategy is to guess quickly. This is the best approach when there is no possibility of obtaining further information." Strategy 2: "If the squares touch other unopened squares, solve the rest of the board first in the hope that approaching from a different direction will eliminate the guess. However, many professionals guess immediately to avoid incurring the time it takes to move the mouse to an easier location." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- **[corrected after primary check]** The phrase "you're probably better off taking a guess right away" is **Barrett's**, not minesweepergame.com's. The full sentence: "When you find something like this, you're probably better off taking a guess right away, rather than saving it for later--so if you guess wrong, you won't have wasted a lot of extra time solving the rest of the board. (I'm a completist, so I save it for last...)". — [nothings.org](https://www.nothings.org/games/minesweeper/)
- Strategy 3: "Players who enjoy flagging often make the mistake of guessing a mine location and chording when they could have opened a safe square instead." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- Strategy 4 (useful guess): "Sometimes one option eliminates another guess or makes the rest of the board easier to solve. For example, when there is a 33:66 situation on a row it is best to open the right or left squares. Opening the middle square will force you to make a second guess." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- Strategy 5 (density): "The mine density on Beginner (8x8) and Intermediate (16x16) is 0.156 and on Expert (16x30) is 0.206. When there is a 50:50 guess it is actually safer to open a random square! Edges are more likely to be openings than squares near the middle. A special case is the top left corner where the probability of being a mine doubles after the first click (due to mine shifting)." (Confirmed.) — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- Strategy 6, with a worked example: "There are 79 possible mine arrangements but only 1 solution has 9 mines ... global mine probability ... ranges from 0.392 (D, K) to 0.798 (J)." Another caption tells players to sequence risky clicks: "Instead of opening three squares in a row open the yellow squares first so you have time to react if the middle square is a mine." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- The 2015 paper hosted on minesweepergame.com is David J. Becerra's Harvard College bachelor's thesis. It confirms: "Some solvers attempt to identify these game instances and deal with them as soon as possible. The idea is to guess early so that the solver can move onto a new game quickly". It credits the "crap-shoot" name to Chris Studholme. — [Becerra 2015, Algorithmic Approaches to Playing Minesweeper](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf)
- **[corrected after primary check]** The first version cited two things to that 2015 paper:
  - the quote "When guessing, it is not sufficient to consider the probability of success";
  - the claim that "about 95% of Expert boards can't be solved by logic alone".
  
  **Neither appears in the PDF text**, so both are removed from this attribution. The idea that information gain matters is supported by Barrett ("maximize the likelihood I'd get useful information") and by minesweepergame.com's strategy 4. — [Becerra 2015](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf); [nothings.org](https://www.nothings.org/games/minesweeper/)
- **[corrected after primary check]** The first version quoted Barrett as saying "guess in favor of situations that use the fewest number of mines possible" and that mines stay less likely "at about 25% density". **Neither phrase is on Barrett's page.** His actual wording is the "fewer mines means more unencountered variants" rule (section 3). Community guides repeat the idea:
  - Koro's "1st principle: A configuration using fewer mines is more likely than one using more", and "2nd ... principle: with the same number of mines, all mine configurations are equally likely to appear".
  - Neutral's "Fewer Mines Lemma": "Guessing that a given area has 1 fewer mine is as safe as guessing a random cell".
  - Sources: [Koro: The art of guessing](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI); [Neutral: Mid-game Guessing (Google Doc)](https://docs.google.com/document/d/1de8roNxpA33Ndl2KGAAp77XgFeYctM_BjXaDqPzF-qg)
- Koro's "3rd principle": "If revealing a cell immediately reveals several others, it's best to choose one of those other cells." Koro also tells players to avoid dead cells and to favour "progress" (section 3). — [Koro: The art of guessing](https://docs.google.com/document/d/1TiRXuoiZHcIlVHvrPXYib6H2fQr7Qfuq6167XPe4lDI)
- Scar on timing: learn common 50/50 patterns "so you can guess them immediately. This has two benefits: It can save a lot of time, and it can improve your chances of winning". Scar also refutes a myth: "A common reason given for not guessing 50/50s immediately is to get more experience. This is not true!" — [Scar: What is a 50/50](https://docs.google.com/document/d/1Mh8bMPuKTWUpB2lbmPNwsrHYIS2nH-t9g835Xaen3x4)
- **Top player, first person**: Kamil Murański in PC World (2011): "If you are forced to guess, always open a square with the least chance of hitting a mine." His example compares two 50% squares with a square that has "only a 15.625 percent (Beginner and Intermediate) or 20.625 percent (Expert) chance to lose", which is the floating-cell density. — [PC World: How to play Minesweeper like a pro, by Kamil Murański](https://www.pcworld.com/article/482195/how-to-play-minesweeper-like-a-pro.html)
- Where 50/50s occur: "coin flips are most likely to occur in corners and edges". This is from an HN community comment; Scar's mines-and-edges explanation in section 3 supports it. — [HN thread (community comment)](https://news.ycombinator.com/item?id=4065036)
- An HN commenter: play "low mine probability with highest information gain". — [HN 16290552](https://news.ycombinator.com/item?id=16290552) (community comment, not a top player)

### Inferences
- The guessing priorities that combine these sources:
  1. Exhaust logic, including mine counting.
  2. Guess a provably forced 50/50, or a pseudo 50/50, immediately.
  3. Otherwise solve elsewhere first if the guess might be resolved later.
  4. When forced, choose by global safety, using the fewer-mines principle or floating-cell density as shortcuts.
  5. Prefer cells with "progress", avoid dead cells, and resolve linked 50/50s from the side that settles both.
  
  The ordering is my synthesis. minesweepergame.com's six strategies and Koro's principles each cover parts of it.
- For a no-guess generator, the human-salient failure modes are a forced 50/50 at a corner or wall (Scar's Patterns 1 and 2) and dead cells.

### Gaps
- I did not find a source for "about 95% of Expert boards need a guess". Treat the figure as unsourced.
- The Polish interview with Murański (babol.pl, 2011, hosted as a PDF on minesweepergame.com) was not read.

---

## 5. Opening strategy, chording, flagging vs no-flag (NF), efficiency habits

### Takeaway
The first click is a guess. Tim Kostka's 2002 analysis of Windows board cycles shows that "the probability of an opening increases towards edges but the size of openings increases towards the middle". minesweepergame.com tells new players to start in the middle. minesweeper.online's help says the opposite: "Start the game from the corners." Efficiency teaching rests on five numbered rules. The rules: flag only to chord; chord where it opens most; use the 1.5 click; switch between flagging and NF locally; shorten the mouse path.

### Cited Findings
- minesweepergame.com First Click: "In 2002, Tim Kostka generated every board and calculated the probability and average size of openings for each square. The probability of an opening increases towards edges but the size of openings increases towards the middle."
  - Opening probability ranges: "0.19 to 0.60" on Beginner, "0.21 to 0.60" on Intermediate, "0.12 to 0.50" on Expert.
  - Average opening size ranges: 18–32 (Beginner), 27–66 (Intermediate), 16–41 (Expert).
  - "new players should start in the middle on all versions because, despite losing more games in the first few clicks, they will finish more games due to larger openings." (Confirmed.)
  - For experienced players: "do you mind losing thousands of games an hour in the first three clicks just to get bigger openings?"
  - Source: [minesweepergame.com: First Click](https://minesweepergame.com/strategy/first-click.php)
- **[corrected after primary check]** Version note, with the exact wording: "Windows Minesweeper makes the first click safe by shifting the mine to the first empty square on the top row, starting from the left corner." The first version said "to the top-left corner". The page says the first free cell along the top row, which is usually the corner. Also: "Windows Vista introduced guaranteed openings on the first click so you should always start in the middle." — [minesweepergame.com: First Click](https://minesweepergame.com/strategy/first-click.php)
- **New:** minesweeper.online's tip contradicts that advice: "Start the game from the corners. Corners are the most likely place to get openings." The site's Beginner is 9x9/10. — [minesweeper.online: Gameplay](https://minesweeper.online/help/gameplay); [Archeaic: Terminology](https://docs.google.com/document/d/1LTgqiHvTHYdVNbHSPH3KbxnJ_QxYLzCWmrPFZxEDKOo)
- 3BV and IOE:
  - The Statistics page gives "IOE [ 3BV / Total Clicks ] ... It is possible to be more efficient than 1.00 by using chording." (Confirmed.)
  - **Owner-site inconsistency:** the Efficiency page words IOE backwards: "Index of Efficiency (IOE) which measures total clicks divided by 3BV". Its own example (3BV 50 in 50 clicks = 1.00) cannot tell the two forms apart.
  - minesweeper.online: "Efficiency is the 3BV of a board divided by the number of clicks used to solve it".
  - Sources: [minesweepergame.com: Statistics](https://minesweepergame.com/statistics.php); [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php); [minesweeper.online: Efficiency](https://minesweeper.online/help/efficiency)
- Flagging vs NF, verbatim: "The global rule is that high 3BV favours flagging because there are more numbers, while low 3BV favours NF because there are more openings. The local rule is that high numbers (5,6,7,8) favour NF because there are fewer adjacent squares to open, while low numbers (1,2,3,4) favour flagging because chording opens more adjacent squares." **[corrected after primary check]** This is on the Efficiency page only; the first version also cited No Flags. — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- **1.5 click**: "The amateur way to chord requires four actions ... The efficient way to chord involves three actions. You press the right mouse button to flag, start pressing the left mouse button as you slide onto a number then release both buttons to chord. This reduces time spent chording by 25%." (Confirmed.) — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- **Fifth efficiency rule (new):** "shorten your mouse path ... New players move the mouse when their eyes move ... Experienced players solve several moves in advance and scan the board while continuing to solve locally." — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- minesweeper.online: "The only way to gain efficiency is to chord. Efficiency is lost by placing flags, wasting clicks, or clicking cells that would be revealed by an opening. By combining NF technique and smart chording you can get more than 100% efficiency." — [minesweeper.online: Efficiency](https://minesweeper.online/help/efficiency)
- NF: "NF players look for openings instead of mines ... When a good NF player sees multiple safe squares, the ones containing numbers (which might be borders) are opened last." On a 50:50 the NF advice is: "The pink squares are 50:50 guesses so the yellow squares will contain numbers. A NF player should guess a blue square." — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php)
- **[corrected after primary check]** The No Flags page does not mention the "Elmar Technique". The Clone Cheats page lists it under "18. Illegal Clicking Techniques": "'Elmar Technique' is a NF method where you change the mouse to open cells on both pressing and releasing." — [minesweepergame.com: Clone Cheats](https://minesweepergame.com/clone-cheats.php)
- Tips page (new): "untick the option to use Questionmarks"; "use the [F2] button" to restart; "If you accidentally click onto a mine you can slide the cursor onto a safe square before releasing the mouse button"; "Ignore the timer"; "Look with your eyes not your mouse"; "Play custom games with more mines to practice your solving skills". — [minesweepergame.com: Tips](https://minesweepergame.com/strategy/tips.php)

### Inferences
- The efficiency rules imply that top players decide locally, number by number, whether to flag and chord or to open cells directly. They optimise clicks per 3BV, not "always flag".
- The two sites disagree on the first click (middle vs corners). They play different rule sets: Windows-style first-click shifting, as studied by Kostka, versus minesweeper.online's 9x9 Beginner with its own first-click handling. Our game should not hard-code either piece of advice.

### Gaps
- minesweeper.online's first-click rule (safe cell, or guaranteed opening, in standard mode) is not stated on the help pages I read.

---

## 6. Teaching: known players and sites, and the progression they recommend

### Takeaway
The structured teaching material comes from three places:
- minesweepergame.com's strategy section: How To Play → Patterns → First Click → Guessing → No Flags → Efficiency → Tips → Links. The menu labels the Patterns page "Advanced Patterns".
- minesweeper.online: the help pages Gameplay, Patterns and Efficiency, plus a large Guides index of player-written documents and videos.
- Sean Barrett's probability page.

**[corrected after primary check]** First-person teaching by a record holder does exist. The PC World article "How to play Minesweeper like a pro" (24 Aug 2011) is **written by Kamil Murański himself**, not secondhand coverage. minesweepergame.com's Expert record list shows that Ze-En Ju's first sub-30 was **29.43 (2020-09-12)**, not 28.84.

### Cited Findings
- minesweepergame.com strategy menu, verbatim: "How to play Minesweeper / Advanced Patterns / First Click / Guessing / No Flags / Efficiency / Tips / Links". Pages:
  - [How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
  - [Patterns](https://minesweepergame.com/strategy/patterns.php)
  - [First Click](https://minesweepergame.com/strategy/first-click.php)
  - [Guessing](https://minesweepergame.com/strategy/guessing.php)
  - [No Flags](https://minesweepergame.com/strategy/no-flags.php)
  - [Efficiency](https://minesweepergame.com/strategy/efficiency.php)
  - [Tips](https://minesweepergame.com/strategy/tips.php)
  - [Links](https://minesweepergame.com/strategy/links.php)
- The Links page is an annotated history of strategy writing, including:
  - "Minesweeper (1992): Lowenthal wrote a 117 page book on Minesweeper strategy!"
  - "The Minesweeper Page (1997): Detailed guide with many examples of patterns, reduction, basic logic and guessing strategies."
  - "Intermediate Hall of Fame (2001): Introduction to patterns and reduction ('hidden patterns') discussing efficiency and guessing quickly."
  - "The Minesweeper Handbook (2003): Basic gameplay including 1-1-X ('partial determinism') and efficient chording."
  - Source: [minesweepergame.com: Strategy Links](https://minesweepergame.com/strategy/links.php)
- minesweeper.online Guides lists community authors with their documents, for example:
  - Scar: "What is a 50/50", "Probability calculation"
  - Koro: "Koro - Tzu : The art of guessing"
  - Archeaic: "Advanced patterns", "No guessing logic", "Minesweeper terminology"
  - gramana: "Minecount 101"
  - Neutral31: "Mid-game Guessing"
  - GlennGould: videos "Opening Strategy: Breaking 50/50s", "Opening Strategy: The Classical", "Opening Strategy: Obelus' Principle"
  - Xilef: videos "The Ultimate Beginners Guide", "Minesweeper Patterns Guide"
  - Danoouch: "Efficiency guide"
  - SilentSlendy: "Advanced Efficiency Guide"
  - Source: [minesweeper.online: Guides](https://minesweeper.online/help/guides)
- **Kamil Murański, first person**, from the PC World byline "by Kamil Murański", dated Aug 24, 2011:
  - The article's intro says he "currently holds four world records ... Expert (31.133 seconds), Intermediate (7.503 seconds), Expert nonflagging (35.152 seconds), and Intermediate nonflagging (8.19 seconds)".
  - Setup: "I'm using an old Logitech MX518 set to 800 dpi with the sensitivity at the third position in the Windows control panel and mouse acceleration turned off." Also: "make sure your desk can't shake"; wash your hand if it sweats ("I wash mine every 10 to 15 minutes").
  - Patterns: "Learning to read these patterns is like learning to read a language ... after three years of playing Minesweeper a few hours a day, you don't have to think about where the mines are when you see these patterns."
  - Flags: use the double-click (chord) technique, but "In this situation you have only one safe square, so you should open that square with a left-click instead of flagging the mine."
  - Guessing: see section 4.
  - minesweepergame.com's Media page lists the article as "An introduction to Minesweeper strategy by Kamil Murański (1-8-32) in PC World magazine".
  - Sources: [PC World](https://www.pcworld.com/article/482195/how-to-play-minesweeper-like-a-pro.html); [minesweepergame.com: Media](https://minesweepergame.com/media.php)
- minesweepergame.com Expert Time World Records, read 2026-11-06:
  - Ze-En Ju: 29.43 (2020-09-12, 3BV 133), 28.84 (2020-12-27), 28.70 (2021-06-19), 27.53 (2021-10-16), 27.36 (2022-05-12), **26.59 (2022-08-23, 3BV 138)**.
  - Kamil Murański before him: 35.70 (2009-03-27), 34.65 (2009-09-02), 31.89 (2010-06-15), 31.13 (2010-07-03).
  - **[corrected after primary check]** By this list Ju's first sub-30 game was the 29.43. TheGamer's headline, "12 Year-Old Ze-En Ju Becomes First Person To Ever Clear A Minesweeper Expert Grid In Under 30 Seconds", is linked from the Media page; the 28.84 attribution in the first version came from secondary sources. The current record is 26.59; the conflict between 26.59 and 27.53 is resolved, since 27.53 is an earlier record.
  - Sources: [minesweepergame.com: World Records](https://minesweepergame.com/world-records.php); [TheGamer](https://www.thegamer.com/12-year-old-ze-en-ju-first-person-clear-minesweeper-expert-grid-under-30-seconds/)
- Ju's community profile lists his style as "flagging" and his grip as claw (Logitech G332). This is a fan wiki and was not re-checked. — [Fandom: Ze-En JU](https://minesweeper.fandom.com/wiki/Ze-En_JU_(JZE))
- Murański also gave a Polish interview, "This Pole is the best in the world!" (babol.pl, Roman Koń, 2011-11-23), hosted on minesweepergame.com. I did not read it. — [minesweepergame.com: Media](https://minesweepergame.com/media.php)

### Inferences
- The learning progression implied by the teaching material:
  1. Rules, controls, and the two single-number rules (B1/B2), including the fake-1 trap.
  2. 1-1-X and 1-2-X as the root patterns.
  3. 1-2-1 and 1-2-2-1 as compositions.
  4. Subsets and reduction.
  5. Holes and triangles.
  6. Multi-region patterns, mine counting, and enumeration (minesweeper.online's "Evil" tier).
  7. Probability (Barrett) and guessing policy: forced, linked and pseudo 50/50s, dead cells.
  8. Speed: first click, efficiency rules, flag vs NF, the 1.5 click, mouse path.
  
  Steps 1–6 follow the order of the pattern pages and minesweeper.online's NG tiers; steps 7–8 are my ordering of the remaining pages.
- What separates top players, in Murański's own words, is pattern recognition made automatic by volume, plus hardware, setup and click economy. It is not new logic.

### Gaps
- Ze-En Ju: I found no first-person tutorial or interview text. Chinese community sources such as saolei.wang were not searched in this pass.
- The YouTube guides listed on minesweeper.online (Xilef, GlennGould and others) were not watched.

---

## Verification log

Checked 2026-11-06 against the live pages. Method: curl with a browser user agent, and headless Chrome for minesweeper.online.

**Confirmed (wording now verbatim and cited):**
- How To Play: controls, Rule 1 with the corner-1 example, chording and the depress trick.
- Patterns page: the pattern definition, 1-2-X and its explanation, 1-2-1 and 1-2-2-1 as compositions, reduction ("a 3 becomes a 2"), and the NF link to 1-1-X and subsets.
- minesweeper.online: the pattern definition, 1–1, 1–1+, 1–2, 1–2C, the reduction quote ("2 turns into 1"), and holes as two 1s sharing two cells.
- Crow 1997 description on the Math Papers page.
- Barrett: mine-counting quote, local-probability definition, 60/50/less-than figures, "less than double" rule of thumb, "1999" label on the Links page.
- Guessing page: goal-dependence quote, "safer to open a random square", top-left corner exception, local vs global quote.
- First Click: probability ranges, edges vs middle, new players start in the middle, Vista guaranteed openings.
- Efficiency: 1.5 click and 25%, global and local flag/NF rules, "never flag a mine unless you are going to chord on it".
- Statistics: IOE > 1 is possible with chording.
- Becerra 2015: the "as soon as possible" quote.

**Corrected [corrected after primary check]:**
- "Flag only what you will chord" and the over-chording warning: Efficiency page, not How To Play.
- The quote is "two most famous patterns", not "two best-known".
- minesweepergame.com has no "1-1+"; its counterpart is 1-1-X from a border.
- The triangle catalogue is now complete (T1–T5).
- The earliest 1-2-1 and 1-2-2-1 source is the 1992 Minesweeper Guide, per the Links page; Crow 1997 is the earliest "relative" (reduction) source.
- "Really is independent of anything else" is Barrett's text, not an HN comment.
- "Probably better off taking a guess right away" is Barrett's text, not minesweepergame.com's.
- The two 2015-paper attributions ("not sufficient to consider the probability", "95% of Expert boards") are not in the PDF and were removed.
- The Barrett "fewest number of mines possible" and "25% density" phrases are not on his page; replaced with his actual rule and the community principles.
- First-click mine shifting goes to "the first empty square on the top row, starting from the left corner".
- The flag/NF rules are on Efficiency only.
- The Elmar Technique is on Clone Cheats, not No Flags.
- "No first-party material on linked 50/50s or dead squares" was wrong: Scar's and Koro's guides on minesweeper.online cover both.
- The PC World article is by Murański himself.
- Ju's first sub-30 was 29.43 (2020-09-12); the current record is 26.59.

**Newly added:**
- The full minesweeper.online catalogue (28 entries), including 2–2–2 corner, 1–3–1 corner, 1>2<1, T–pattern, Dependency Chain, Mine counting and Combinations.
- Each entry's NG difficulty tier.
- minesweepergame.com's subset and "find mines" logic, and its six guessing strategies.
- Barrett's section order and his "linked" squares.
- Definitions of forced, linked and pseudo 50/50s and dead cells.
- The minesweeper.online corner-start tip.
- The Efficiency fifth rule (mouse path) and the Tips page.
- The annotated history on the Links page.
- The Murański first-person advice.

**Still unverified:**
- The 2–2–2 corner and Dependency Chain layouts against their diagrams (`2-2-2/<n>`, `dc/<n>`), and the 1>2<1 diagram, whose code was not found.
- The source of "95% of Expert boards need guessing".
- Ju's fan-wiki style and grip data.
- The Polish Murański interview.
- minesweeper.online's first-click rule.
- HN community comments, which were left as community sources and not re-checked.
- minesweepergame.com forum pages: 403 to every method tried; web.archive.org was unreachable.
