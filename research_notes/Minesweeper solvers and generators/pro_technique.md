# How top human Minesweeper players play and teach: deduction techniques, patterns, guessing heuristics

> **Read this first: how these notes were made.** The primary sources (minesweepergame.com, minesweeper.online, nothings.org, minesweeper.info, Wikipedia, YouTube, Reddit, web.archive.org) could **not be fetched** from this session: the network policy blocked them (DNS/CONNECT 403). The only access was WebSearch, whose results include short excerpts and summaries of the indexed pages. Every finding below comes from those excerpts, which are restricted to the primary domains where possible. Treat them as near-primary, not verbatim.
>
> I caught one search summary getting the 1-2-1 answer backwards ("middle cell under the 2 is the mine"), so I checked every pattern solution by working through its constraints by hand. Those checks are marked **[derived]** under Inferences. All ASCII diagrams are mine, drawn to illustrate patterns the sources name. They are not copied from the sources.
>
> To verify the details, allow these hosts and re-fetch: `minesweepergame.com/strategy/*.php`, `minesweeper.online/help/patterns`, `nothings.org/games/minesweeper/`.

Diagram notation used throughout: `#` = unopened cell, `F` = flag/known mine, `.` = opened blank, `|` or `===` = wall/board edge or an already-cleared row. `M` = deduced mine, `S` = deduced safe.

---

## 1. Basic rules and how teachers phrase them

### Takeaway
Teachers start with two single-number rules. If a number equals its count of unopened neighbours, all of them are mines. If a number's flags already match it, every other neighbour is safe, and chording opens them in one action. minesweepergame.com teaches these rules together with the controls (open, flag, chord). The corner "1" is its standard first example.

### Cited Findings
- minesweepergame.com "How To Play": left-click opens a square, right-click places a flag, and a second right-click makes it a question mark. Opening a blank square cascades outward until it reaches numbered squares. — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- The same page teaches rule 1 this way: if a number equals the count of its unrevealed neighbours, all of them are mines. Its example is a 1 in a corner, which has only one unopened neighbour to check. — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- Rule 2 is taught through chording. Pressing both buttons on a number opens its remaining neighbours once all the mines it touches are flagged. A wrong flag makes the chord detonate a mine. The page also suggests pressing a chord on a number just to see which neighbours are still unopened. — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php)
- The page warns that new players flag and chord too much. It advises flagging only the mines you plan to chord on and practising chording slowly. — [minesweepergame.com: How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php); [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)

### Inferences
- Rule 1 ("saturated by unknowns"): `number − flagged neighbours = unopened neighbours` ⇒ all unopened neighbours are mines. Rule 2 ("satisfied"): `number − flagged neighbours = 0` ⇒ all unopened neighbours are safe. Both rules are the single-constraint case of "reduction" (section 2). **[derived]**
```
Rule 1 (corner):         Rule 2 (satisfied):
| 1 #                     F 1 #
| . .     -> # is M       . . #    -> both # are S (chord the 1)
```

### Gaps
- I could not get the exact wording of the How To Play page. The phrasing above is paraphrased from search excerpts.

---

## 2. Named patterns, reduction, and subset/difference reasoning

### Takeaway
Both main teaching sites present patterns as memorised shortcuts. minesweeper.online defines a pattern as "a common arrangement of numbers that has only one solution. If you memorise a pattern it will reduce the amount of time you use thinking". The curriculum runs 1-1 → 1-1+ → 1-2 → 1-2+ / 1-2C → 1-2-1 → 1-2-2-1 → **Reduction** → **Holes** (H1, H2, …) → **Triangles** (T1, …). Teachers stress that 1-2-1 and 1-2-2-1 are just combinations of the 1-2 rule. Reduction means subtracting known mines from a number, which turns a messy position into a known pattern.

### Cited Findings
- minesweeper.online defines a pattern this way: "A pattern is a common arrangement of numbers that has only one solution. If you memorise a pattern it will reduce the amount of time you use thinking." — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **1-1**, as taught by minesweeper.online: two 1s touch the same pair of unopened cells, so that pair holds the single mine. If the second 1 has one more neighbour beyond the pair, that cell is safe. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns); also described at [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **1-1+**: the second 1 "also touches the two yellow cells, so it already has a mine and all the remaining cells can be opened". The second source describes it as the two shared cells holding at most one mine. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns); [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **1-2**: a 1 and a 2 share two cells. The 1 puts at most one mine in that pair, so the 2's other mine must be in its unshared cell. minesweeper.online also has a **1-2C** ("classic") variant: the shared cells hold at most one mine, so the 2 forces a mine in its third cell. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- minesweepergame.com: "The 1-2-1 and 1-2-2-1 patterns are actually combinations of a more important pattern", namely the 1-2 rule, also given as "1-2-X": the X must be a mine. 1-2-1 and 1-2-2-1 are "the two best-known patterns" and each has a single solution. — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php)
- **Reduction**, as minesweepergame.com puts it: "Complex mine arrangements can be reduced to patterns. You simply subtract the known mines from each number", for example a flagged mine touching a 3 makes it act as a 2. minesweeper.online says the same: a 2 touching one flagged mine "turns into 1". Reduction is how a messy position comes to match 1-2-1 and the like. — [minesweepergame.com: Patterns](https://minesweepergame.com/strategy/patterns.php); [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **Holes**: minesweeper.online labels these H1, H2, …. In its hole example, two 1s each touch the same two cells, so those cells hold one mine and the other cells around the hole can be opened. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **Triangles**: minesweeper.online labels these T1, …, and the entries start from a 1 at the bottom of the shape. The search excerpt cut off there, so I could not confirm their full logic. — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **History**: according to minesweepergame.com's math-papers index, Philip Crow's 1997 paper has an early description of the 1-2-1 pattern and "the first mention of 'relative' patterns", meaning known mines reduce numbers to a standard pattern. That makes it the earliest written form of "reduction" I found. — [minesweepergame.com: Math Papers](https://minesweepergame.com/math-papers.php); [Crow 1997 PDF](https://minesweepergame.com/math/a-mathematical-introduction-to-the-game-of-minesweeper-1997.pdf)
- The no-flag (NF) guide treats spotting mine patterns such as "1-1-X", which show which squares are safe to open, as the core NF skill. — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php)
- One search summary said the 1-2-1 mine sits under the 2. **That is wrong.** See the derived solution below. Low-quality SEO pattern sites (e.g. minesweeper.now, minesweeperblast, rarepike) also disagree with each other on 1-2-1 and 1-2-2-1, so I did not use them. — [search noise, e.g. minesweeper.now Pattern Bible](https://minesweeper.now/blog/the-minesweeper-pattern-bible-how-to-recognize-1-2-1-1-2-2-1-and-2-1-2-formations)

### Inferences
All solutions below are **[derived]** by working through the constraints. Each assumes a straight frontier, with the unopened row underneath and the row above already cleared or a wall.

**1-1 from a wall (1-1+ edge)**: the 1 against the wall sees only two cells, so the next 1's third cell is safe.
```
| 1 1 ?          numbers (cleared row)
| # # S          a,b share 1 mine; c is S
```
**1-2 (general)**: the shared pair holds at most 1 mine (because of the 1), so the 2's far cell is a mine. The shared pair then holds exactly 1, so the 1's far cell is safe.
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
**Reduction example**: an `F` touching a 2 makes it act as a 1, and that 1 can complete a 1-2-1:
```
  F             F touches the left "2"
  2 2 1   reduces to   1 2 1  ->  same solution as 1-2-1
```
**Subset/difference rule**: this is the general principle behind 1-1, 1-1+, 1-2, holes, and probably triangles. Let A and B be two numbers after reduction, with unopened neighbour sets U(A) and U(B). If U(A) ⊆ U(B), then U(B)\U(A) holds exactly B−A mines. If B−A = 0, every cell in the difference is safe; that is 1-1. If B−A = |U(B)\U(A)|, every cell in it is a mine; that is 1-2 next to a wall. When the sets only partly overlap, the shared region holds at most min(A,B) mines. That bound gives 1-2C: "the shared cells can hold at most one mine, so the far cell is a mine". The sites never state this rule abstractly. They teach the instances.
- **Learning order**: the site curricula present patterns in this order: basic 1-1/1-2 family → combined 1-2-1/1-2-2-1 → reduction → holes/triangles. The order suggests teachers want players to see the 1-2 rule as the root and the longer patterns as compositions, not as more things to memorise.

### Gaps
- I could not fetch the exact catalogue entries for holes (H1–H?), triangles (T1–T?), boxes, "1-2+", "1-2C+", "2-2-2", or "1-1 from wall" as minesweepergame.com names it, and I could not reproduce the sites' own diagrams. The sites are blocked here and the excerpts were truncated. Re-fetch [minesweeper.online/help/patterns](https://minesweeper.online/help/patterns) and [minesweepergame.com/strategy/patterns.php](https://minesweepergame.com/strategy/patterns.php) to get them.
- I could not confirm which human author wrote each site's pattern page. Both pages appear to be site-authored; minesweepergame.com is run by Damien Moore, but that attribution is unverified here.

---

## 3. Advanced: mine counting, global reasoning, linked 50/50s, dead squares

### Takeaway
Sean Barrett's "Minesweeper: Advanced Tactics" (nothings.org, about 1999) is the classic human-written advanced guide. It covers endgame mine counting, local versus global probability, and a key heuristic: configurations that use fewer mines are much more likely, until unknown squares get scarce relative to remaining mines. minesweepergame.com repeats the local-vs-global distinction. I found no first-party teaching material that names "linked 50/50s" or "dead squares".

### Cited Findings
- Barrett: "One very easy endgame tactic you can use is counting the number of remaining mines." — [nothings.org: Minesweeper: Advanced Tactics](https://nothings.org/games/minesweeper/)
- Barrett defines local probability: "By locally, I mean that if you have a '1' next to two unknown squares, each has a 50% chance of being a mine." He then shows why the local view can mislead once the rest of the board is taken into account. — [nothings.org](https://nothings.org/games/minesweeper/)
- Barrett's key heuristic: "configurations that use fewer mines are much more likely". He applies it as a rule of thumb "until the number of unencountered-squares is less than double the number of unaccounted-for mines". His worked example compares two configurations that differ by one mine. The fewer-mine one is about 60% likely when six mines remain, about 50% at seven, and less likely than the alternative at eight. — [nothings.org](https://nothings.org/games/minesweeper/); discussion at [Hacker News 13566730](https://news.ycombinator.com/item?id=13566730)
- His site lists the page as "Advanced Minesweeper Tactics (1999)". HN titled it with 2005, so the date of the latest revision is uncertain. — [nothings.org/games](http://www.nothings.org/games/); [HN](https://news.ycombinator.com/item?id=13566730)
- minesweepergame.com Guessing: "Local probability is easy to calculate but global probability is much more difficult." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- From an HN solver thread: a local 50% estimate is right only when the two squares "really [are] independent of anything else". Where constraints overlap, simple per-number probability arithmetic is not valid. — [HN: Solving Minesweeper and making it better](https://news.ycombinator.com/item?id=16290552) (community comment, not a top player)

### Inferences
- **Why Barrett's rule holds [derived]**: suppose R unknown cells lie off the frontier and M mines are unaccounted for. A frontier configuration using k mines has weight C(R, M−k). The ratio of a (k)-mine configuration to a (k+1)-mine one is C(R,M−k)/C(R,M−k−1) = (R−M+k+1)/(M−k). That ratio exceeds 1 roughly when R > 2(M−k), which is exactly Barrett's "less than double" threshold. A generator or solver can use the same weighting.
- **Endgame mine counting [derived]**: if a frontier can be solved with either m or m+1 mines and the remaining mine counter says m, the m+1 solutions are eliminated. Every interior (non-frontier) cell is then safe if the frontier must use all remaining mines, or a mine if no mines are left for it.
- **Safe cells by elimination across several constraints [derived]**: a cell is safe if it is a mine in none of the consistent assignments of the coupled frontier. Humans approximate this with chains of pattern and subset reasoning; solvers enumerate assignments.

### Gaps
- I found no first-party human teaching source for "linked 50/50s" (several 50/50s whose outcomes are coupled, so one guess resolves several) or "dead squares" (cells whose reveal can't give useful information, or whose value is already forced). As far as I know these terms come from solver communities, such as the JSMinesweeper solver by David Hill, but I could not verify that here, so they are not cited.
- I could not get Barrett's full sections in order, his ASCII figures, or his advice on choosing between equal-probability guesses.

---

## 4. Guessing: how strong players choose when forced

### Takeaway
The teaching sources frame guessing as goal-dependent. Speed players resolve forced 50/50s **immediately**, so a loss costs as little time as possible. Players chasing win rate first **solve the rest of the board** in case new information removes the guess. On how to choose a guess, sources mention true probability (local vs global), information gain, chance of an opening, corners/edges as typical 50/50 locations, and Barrett's rule of favouring configurations with fewer mines.

### Cited Findings
- minesweepergame.com: "The optimal guessing strategy depends on whether your goal is to win or to win fewer games more quickly." — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- On guessing early, one guide says: "When you find something like this, you're probably better off taking a guess right away", because a wrong guess then hasn't wasted time solving the rest of the board. The search excerpt did not say whether this came from minesweepergame.com Guessing or Barrett's page. The alternative it gives is to "solve the rest of the board first in the hope that approaching from a different direction will eliminate the guess". — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php); [nothings.org](https://nothings.org/games/minesweeper/)
- An algorithmic paper hosted on minesweepergame.com describes the same speed logic: some solvers identify these games and deal with the guess "as soon as possible", so they can move on to a new game quickly. — [Algorithmic Approaches to Playing Minesweeper (2015)](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf)
- A random square can beat a local 50/50: "When there is a 50:50 guess it is actually safer to open a random square!" Exception: "the top left corner where the probability of being a mine doubles after the first click (due to mine shifting)". This applies to old Windows versions, which moved a first-click mine to the top-left. — [minesweepergame.com: Guessing](https://minesweepergame.com/strategy/guessing.php)
- When choosing a random guess, edges are more likely than the middle to be openings. — [minesweepergame.com: First Click](https://minesweepergame.com/strategy/first-click.php)
- Barrett-style guessing appears in other guides too: "guess in favor of situations that use the fewest number of mines possible", and mines remain less likely than non-mines even at about 25% density. — [nothings.org](https://nothings.org/games/minesweeper/) (via search excerpt; the exact source page is uncertain)
- Probability is not the only factor: "When guessing, it is not sufficient to consider the probability of success (probability of not blowing up)". The information gained also matters. — [Algorithmic Approaches (2015)](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf). An HN commenter makes the same point: play "low mine probability with highest information gain", which requires looking ahead. — [HN 16290552](https://news.ycombinator.com/item?id=16290552)
- Where 50/50s occur: "coin flips are most likely to occur in corners and edges". In the middle you usually have information from several directions. A corner 1 makes each neighbour a 1-in-3 risk, while a middle 1 makes each a 1-in-8 risk. — [HN thread (community comment)](https://news.ycombinator.com/item?id=4065036)
- One academic report (via a minesweepergame.com-hosted paper) estimates that about 95% of Expert boards can't be solved by logic alone. I could not confirm which paper the figure comes from or what definition it uses. — [Algorithmic Approaches (2015)](https://minesweepergame.com/math/algorithmic-approaches-to-playing-minesweeper-2015.pdf) (unverified attribution)

### Inferences
- The guessing priorities that combine these sources are: (1) exhaust logic, including counting; (2) if a forced guess can never be resolved, take it immediately when playing for time; (3) otherwise choose by global safety probability, break ties by information gain or chance of an opening, and favour fewer-mine configurations per Barrett. The ordering is my synthesis, not something any single teacher states.
- For a no-guess generator, "unresolvable 50/50 at a corner or edge" is the human-salient failure mode the sources describe.

### Gaps
- I found no top-player interview (Murański, Ju, etc.) that gives an explicit guessing algorithm.

---

## 5. Opening strategy, chording, flagging vs no-flag (NF), efficiency habits

### Takeaway
The first click is always a guess. Openings are more likely near the edges but bigger near the middle, and minesweepergame.com tells new players to start in the middle. Efficiency is measured by 3BV and IOE, and teachers push "flag only what you will chord". The site gives local rules for when to flag and when to play NF, plus the "1.5-click" chording technique.

### Cited Findings
- The first click is always a guess: the opener is a guess because the player starts with a covered board. Opening probability ranges from 0.19 to 0.60 on Beginner and from 0.12 to 0.50 on Expert. "The probability of an opening increases towards edges but the size of openings increases towards the middle." New players "should start in the middle on all versions because, despite losing more games in the first few clicks, they will finish more games due to larger openings." These figures come from Windows Minesweeper analysis, and versions that differ in first-click behaviour may differ. — [minesweepergame.com: First Click](https://minesweepergame.com/strategy/first-click.php)
- Version note: according to the search excerpt, Windows Vista and later guarantee the first click opens an area, while older versions shift a first-click mine to the top-left corner. — [minesweepergame.com: First Click](https://minesweepergame.com/strategy/first-click.php) (excerpt only)
- 3BV is the minimum number of left clicks needed to win without flags. IOE = 3BV / total clicks. Chording can push IOE above 1.00; the search summary said "below", which reverses the definition. — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- The efficiency page's rules of thumb for flagging vs NF: globally, high 3BV favours flagging (more numbers) and low 3BV favours NF (more openings, mines in islands). Locally, high numbers (5–8) favour NF because there are fewer neighbours to open, and low numbers (1–4) favour flagging because a chord opens more neighbours. — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php); [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php)
- **1.5-click chord**: flag with the right button, then hold left, slide onto the number, and release both to chord. That is three actions instead of four, and the page says it cuts chording time by 25%. Other habits: don't flag a mine unless you'll chord on it, and don't chord numbers that open nothing new. — [minesweepergame.com: Efficiency](https://minesweepergame.com/strategy/efficiency.php)
- NF players look for openings instead of marking mines. Some NF techniques (e.g. the "Elmar Technique") are banned for ranked play. — [minesweepergame.com: No Flags](https://minesweepergame.com/strategy/no-flags.php); [minesweepergame.com: Clone Cheats](https://minesweepergame.com/clone-cheats.php)

### Inferences
- The efficiency rules imply that top players decide locally, number by number, whether to flag and chord or to open cells directly. They are optimising clicks per 3BV, not "always flag".

### Gaps
- I could not get the exact first-click advice for specific versions (e.g. whether experts click corners on XP to raise the chance of an opening).

---

## 6. Teaching: known players and sites, and the progression they recommend

### Takeaway
The structured teaching material is site-authored. minesweepergame.com has a strategy section (How To Play → Patterns → First Click → Guessing → Efficiency → No Flags), minesweeper.online has a Patterns help page, and Sean Barrett's page covers probability and endgames. Record holders such as Kamil Murański and Ze-En Ju appear in secondary coverage, which is mostly about hardware and practice volume. I found no first-person tutorials from them in this session.

### Cited Findings
- minesweepergame.com strategy pages found: How To Play, Patterns, First Click, Guessing, Efficiency, No Flags, and a Links page to other guides. — [How To Play](https://minesweepergame.com/strategy/how-to-play-minesweeper.php); [Patterns](https://minesweepergame.com/strategy/patterns.php); [First Click](https://minesweepergame.com/strategy/first-click.php); [Guessing](https://minesweepergame.com/strategy/guessing.php); [Efficiency](https://minesweepergame.com/strategy/efficiency.php); [No Flags](https://minesweepergame.com/strategy/no-flags.php); [Links](https://minesweepergame.com/strategy/links.php)
- minesweeper.online teaches patterns as memorisation to "reduce the amount of time you use thinking". — [minesweeper.online: Patterns](https://minesweeper.online/help/patterns)
- **Kamil Murański** (former Expert world-record holder), from secondhand PCWorld coverage that features his replays: the mouse must give solid control and accuracy. He uses an older Logitech mouse with acceleration off, and the article also recommends a steady desk and the right chair height. After "three years of playing Minesweeper a few hours a day, you don't have to think about where the mines are when you see these patterns", which describes pattern automaticity built by volume. — [PCWorld: How to play Minesweeper like a pro](https://www.pcworld.com/article/482195/how-to-play-minesweeper-like-a-pro.html). Coolmath repeats that he values a setup "optimized for speed and accuracy". — [Coolmath: Minesweeper world records](https://www.coolmathgames.com/blog/minesweeper-world-records). Wiki profile: [Fandom: Kamil Muranski](https://minesweeper.fandom.com/wiki/Kamil_Muranski)
- **Ze-En Ju (JZE)**: the first person to clear Expert in under 30 seconds (28.84 s), aged 12 by TheGamer's account. His community profile lists his style as "flagging" and his grip as claw (Logitech G332). Sources disagree on his current Expert record (26.59 s and 27.53 s are both cited), so check the current ranking before quoting either. — [TheGamer](https://www.thegamer.com/12-year-old-ze-en-ju-first-person-clear-minesweeper-expert-grid-under-30-seconds/); [Fandom: Ze-En JU](https://minesweeper.fandom.com/wiki/Ze-En_JU_(JZE)); [YouTube WR 28.84](https://www.youtube.com/watch?v=aKML1EbVCvY); [minesweepergame.com: World Records](https://minesweepergame.com/world-records.php)

### Inferences
- The learning progression implied by the teaching material: (1) rules, controls, and the two single-number rules; (2) the 1-1 and 1-2 families as the root patterns; (3) 1-2-1 and 1-2-2-1 as compositions of 1-2; (4) reduction, to apply patterns next to flags; (5) holes and triangles, i.e. shared-region reasoning around enclosed or angled frontiers; (6) mine counting in the endgame and global probability (Barrett); (7) guessing policy by goal; (8) speed: first-click choice, efficiency (3BV/IOE), the flag-vs-NF choice, and 1.5-click chording. Steps 1–5 follow the order of the pattern pages; steps 6–8 are my ordering of the remaining pages.
- What separates top players, in the secondary coverage, is automatic pattern recognition built by volume plus mechanical efficiency, not new logic.

### Gaps
- No first-person teaching content from Murański, Ju, or other record holders (videos, interviews, Chinese community tutorials such as those on saolei.wang) could be reached or found. YouTube, Reddit, and Chinese community sites were unreachable or did not surface in search.
- Damien Moore (minesweepergame.com) as author of the strategy pages, and the authorship of the minesweeper.online guide, are unverified.
- These notes would get much stronger if the session's network allowlist included minesweepergame.com, minesweeper.online, nothings.org, and minesweeper.info, so the exact pattern catalogue and diagrams could be fetched.
