# Board generation: Runesweeper vs Microsoft Minesweeper

Checked against Runesweeper at commit `95447de`, and against Windows XP `winmine.exe` as reconstructed from its assembly by repnz ([ReversingMinesweeper](https://github.com/repnz/ReversingMinesweeper) @ `9981360`, target sha256 `bcff8931…af41`). Researched 2026-11-06.

> **Update:** differences 1–4 are fixed. Runesweeper now uses the MSVC `rand()`, seeded once per session, draws the column then the row, and limits the first-click scan to XP's bounds, including XP's "no move" case. Still different: pre-2000 8×8 Beginner (not emulated), the Custom dialog's input handling, and XP's "one click bug" (no win check after relocation), which is not reproduced.

## Verdict

**The same distribution of boards, but not the same algorithm.** The presets, the custom limits, mines being laid when the field is created, rejection of cells that already hold a mine, and "move the mine, don't regenerate, no guaranteed opening" all match XP. These things differ:

1. **RNG.** Runesweeper uses splitmix64 with a fresh 64-bit seed for every game. XP uses the MSVC CRT LCG `rand()`, seeded once per process with `srand(GetTickCount())`. No XP board can be reproduced from a seed.
2. **Coordinate draw.** Runesweeper makes one draw, `% (w*h)`, over a flat index. XP makes two draws: `rand() % Width`, then `rand() % Height`. The distribution is the same to within XP's tiny modulo bias, but the sequence is different.
3. **First-click relocation scan bounds (a real behavioural difference).** XP scans rows `1..Height-1` and columns `1..Width-1`, so it **never** puts the mine in the last row or last column. Runesweeper scans every cell. They pick different cells whenever the first free cell in row-major order lies in the last column or the last row.
4. **First-click relocation when no cell is free.** If every cell in XP's `(W-1)×(H-1)` scan area is a mine, XP moves nothing. The click reveals nothing, the game isn't lost, and the next click is treated as a first click again. Runesweeper always finds a free cell.
5. **Pre-2000 versions.** Runesweeper copies XP. Windows 3.1/95/98 used an **8×8** Beginner board. The 9×9 board arrived in Windows 2000.

In practice, differences 3 and 4 show up only on dense custom boards. With standard densities, the top-left cell is almost always free and both pick it.

## Comparison

| Axis | Microsoft (XP winmine) | Runesweeper | Match? |
|---|---|---|---|
| PRNG | MSVC `rand()`: `s = s*214013 + 2531011; return (s>>16) & 0x7FFF` | splitmix64, `src/game.rs:300-312` | No |
| Seed | `srand(GetTickCount())` once at startup; later games continue the stream | `storage::seed()` (ns time ^ pid, or JS `Math.random`) for every game, `src/storage.rs:142-152`, `src/app.rs:96,151`; `seed \| 1`, `src/game.rs:116` | No |
| Coordinate formula | `col = rand()%Width + 1`, then `row = rand()%Height + 1` (1-based, inside a border ring) | `i = rng.next() % (w*h)`, 0-based flat index, `src/game.rs:126` | Distribution yes, sequence no |
| Placement loop | Redraw while the cell already holds a mine; repeat `Mines` times | Same rejection loop, `src/game.rs:123-133` | Yes |
| When mines are laid | At New Game (`InitializeNewGame`) | At `Game::new`, `src/game.rs:104-120` | Yes |
| First-click trigger | Left click on an unrevealed, unflagged cell while `NumberOfRevealedBlocks == 0` | `!started` on `open` of a Hidden/Question cell, `src/game.rs:167-182` | Yes |
| First-click action | Move the mine to the first non-mine cell, row-major from (1,1), **rows 1..H-1, cols 1..W-1**, then reveal the clicked cell | Move to the first non-mine cell, row-major over **all** cells, `src/game.rs:174-181` | **No** (bounds) |
| Opening guaranteed? | No. The clicked cell may show a number | No | Yes |
| Beginner | 9×9/10 (Windows 2000/XP). 8×8/10 in Windows 3.1/95/98 | 9×9/10, `src/game.rs:26` | XP yes, pre-2000 no |
| Intermediate | 16×16/40 | `src/game.rs:27` | Yes |
| Expert | 30×16/99 | `src/game.rs:28` | Yes |
| Custom dialog | H clamped 9..24, W 9..30, mines 10..min((H-1)(W-1), 999) | `clamp_custom`, `src/game.rs:43-48` (999 cap moot: max is 667) | Yes |
| Bad dialog input | `GetDlgItemInt` returns 0, which clamps to the minimum | Falls back to the previous value, `src/app.rs:279` | Minor no |
| Stored settings | Registry: H 9..25, W 9..25 (sic), mines 10..999, no (H-1)(W-1) cap | Re-clamped with `clamp_custom`, `src/storage.rs:48` | Minor no (only matters for hand-edited settings) |

## Details

### 1. RNG and seed

- **Microsoft CRT `rand`.** Microsoft's UCRT `rand.cpp` (copyright Microsoft, as mirrored in ReactOS: [reactos/sdk/lib/ucrt/stdlib/rand.cpp](https://github.com/reactos/reactos/blob/master/sdk/lib/ucrt/stdlib/rand.cpp)) has `srand` store the seed in the per-thread `_rand_state`. `rand` computes `state = state * 214013 + 2531011; return (state >> 16) & RAND_MAX;`, with `RAND_MAX = 0x7FFF`. The state is 32-bit and unsigned.
  - XP's winmine predates the UCRT and would use `msvcrt.dll`. The assumption that `msvcrt.dll` uses the same constants is **unverified here**: Microsoft's docs on learn.microsoft.com were unreachable from this session, and so was a binary to disassemble.
- **Seed.** XP calls `srand(GetTickCount())` once, in `InitMetricsAndFirstGame` (repnz `Minesweeper/config.c:197-202`). That function runs at window creation (`windowing.c:791`). Every later New Game draws from the same stream, and nothing reseeds it.
  - The seed sources for Windows 3.1/95/98 are **unverified**.
- **Runesweeper.** `struct Rng` is splitmix64 (`src/game.rs:300-312`), seeded with `seed | 1` (`src/game.rs:116`). A new seed comes from `storage::seed()` for every field (`src/app.rs:151`, `src/storage.rs:142-152`). The `lib.rs` scenes use fixed seeds 7, 11 and 5 (`src/lib.rs:40-46`).

### 2. Mine placement

repnz's `InitializeNewGame` (`Minesweeper/game.c:79-124`):

```c
do {
    do {
        randomPoint = (BoardPoint){ .Column = GetRandom(Width) + 1,
                                    .Row    = GetRandom(Height) + 1 };
    } while (ACCESS_BLOCK(randomPoint) & BLOCK_IS_BOMB);
    ACCESS_BLOCK(randomPoint) |= BLOCK_IS_BOMB;
    Mines_Copy--;
} while (Mines_Copy);
```

- `GetRandom(n)` is `rand() % n` (`Minesweeper/utils.c:5-6`).
- The board is `BYTE BlockArray[27][32]`, indexed `[row][column]` (`game.c:31`, `game.h:70`), with a ring of `0x10` border cells (`game.c:58-77`). Coordinates are therefore 1-based.
- The column (x) is drawn first. That order comes from repnz's reconstruction. Plain C leaves the evaluation order of initializers unspecified, so the order rests on repnz having transcribed it from the assembly. repnz says the reconstruction is "almost identical" to the leaked NT 4.0 source (README). The order is **not independently confirmed** against the binary.

Runesweeper (`src/game.rs:123-133`) draws one flat index `rng.next() % (w*h)` with the same rejection rule, then counts neighbours.

- **Distribution.** Two independent uniform axes give the same distribution as one uniform flat index, so the boards have the same distribution. XP's `% 30`, `% 16` and `% 9` on 15-bit outputs add a bias of about 1 in 32768. That doesn't matter in practice.
- **Sequence.** The two can never produce the same board from the same seed.

### 3. First-click safety

repnz `Minesweeper/game.c`:

- `HandleBlockClick` (`game.c:428-464`) passes an unrevealed, unflagged cell to `HandleNormalBlockClick`. A question-marked cell counts as unflagged. A 3×3 (chord) click on an unrevealed cell does nothing (`game.c:490-525`).
- `HandleNormalBlockClick` (`game.c:566-586`): if the cell is a mine and `NumberOfRevealedBlocks == 0`, it calls `ReplaceFirstNonBomb`.
- `ReplaceFirstNonBomb` (`game.c:544-564`):

  ```c
  // WIERD: LOOP IS WITHOUT AN EQUAL SIGN
  for (int current_row = 1; current_row < Height; ++current_row)
      for (int current_column = 1; current_column < Width; ++current_column)
          if (!(*pLoopBlock & BLOCK_IS_BOMB)) {
              *pFunctionBlock = BLOCK_STATE_EMPTY_UNCLICKED;  // clear the clicked cell
              *pLoopBlock |= BLOCK_IS_BOMB;                    // mine goes here
              ExpandEmptyBlock(point);                         // reveal the clicked cell
              return;
          }
  ```

  - The scan is row-major from the top-left. It excludes the last row (`Height`) and the last column (`Width`). repnz flags the missing `=` himself.
  - A flagged cell can be the target, because only the mine bit is checked.
  - If nothing is free in the scan area, the function returns without doing anything.
  - The clicked cell is revealed normally, so it can show a number. No opening is guaranteed.
- **Independent, empirical support for the bounds (secondary source).** Curtis Bright edited `winmine.ini` to make an 8×8 Beginner board with 63 mines. He reports first clicks on mines that did not move: "there are only 63 mines on this board; there should be space to move the mine somewhere!" ([1sectheory](https://www.curtisbright.com/msx/v0/1sectheory.html), seen via search snippet only; the page itself was unreachable). That is what the `<` bound predicts whenever the one free cell lies in the last row or the last column.
- **Related bug.** minesweepergame.com ([one-click-bug](https://minesweepergame.com/history/one-click-bug.php), secondary, seen via search snippet only) describes the "one click bug": the relocation path calls `ExpandEmptyBlock` but never checks for a win. repnz's code matches that (`game.c:578-580`; the win check is only in the non-mine branch, `game.c:573`). The page says the bug was fixed in Vista. It doesn't change the layout but explains the history.
- **Runesweeper** (`src/game.rs:171-181`):

  ```rust
  self.cells[i].mine = false;
  let free = (0..self.cells.len()).find(|&j| j != i && !self.cells[j].mine).unwrap();
  self.cells[free].mine = true;
  ```

  This is row-major over **all** cells, so its result differs from XP's in two cases:
  - **The first free cell is in the last column or the last row.** Example: on a 9×9 board where row 1, columns 1–8 are mines and (9,1) is free, a first click on a mine sends XP's mine to (1,2) and Runesweeper's to (9,1).
  - **XP's `(W-1)×(H-1)` area is entirely mined.** XP moves nothing; Runesweeper still moves the mine. This needs `mines == (W-1)(H-1)`, which the custom dialog allows.

  The test `the_first_click_moves_a_mine_to_the_top_left` (`src/game.rs:349-355`) uses a 3×3 board whose (0,0) is free, so it can't tell the two scans apart. The comment at `src/game.rs:175-176` and the README ("moves to the top-left-most free square") describe the scan only roughly.
- **Vista/7.** Claims that Vista/7 guarantees a safe first click, and possibly an opening, are **unverified**. Search found only low-authority FAQ pages and a Dummies article ("you never hit a mine on your first click"). No reverse-engineered source turned up, and the candidate write-ups (vollragm.github.io, fevral.github.io) were not reached.

### 4. Presets and custom limits

- **XP table** (repnz `Minesweeper/windowing.c:21-44`): Beginner 9×9/10, Intermediate 16×16/40, Expert 30×16/99. Choosing a level copies its row into `GameConfig` and calls `InitializeNewGame` (`windowing.c:617-627`).
- **Custom dialog** (`windowing.c:220-223`, `905-918`): `Height` is clamped to 9..24, then `Width` to 9..30, then mines to `10..min((Height-1)*(Width-1), 999)`. Runesweeper's `clamp_custom` (`src/game.rs:43-48`) and its test (`src/game.rs:419-422`) match exactly. The 999 cap can't bind, because the most a 30×24 board allows is 29×23 = 667.
- **Registry load** (`config.c:120-125`): Height 9..25, Width 9..25, mines 10..999, with no `(H-1)(W-1)` cap. The INI path is `config.c:235-238`, with Width 9..30.
  - The Width maximum of 25 may be a mistake in the reconstruction (the array has 32 columns): **unverified**.
  - Runesweeper re-clamps stored values with `clamp_custom` (`src/storage.rs:45-48`). This only matters for settings files edited by hand.
  - Runesweeper's defaults, `custom: (9, 9, 10)` (`src/storage.rs:21`), match XP's registry defaults.
- **Versions** (all secondary, via search snippets; TCRF and minesweepergame.com were unreachable):
  - Beginner was **8×8/10** in Windows 3.1/95/98 and became 9×9 in **Windows 2000** ([TCRF](https://tcrf.net/Minesweeper_(Windows,_1990)), [minesweepergame.com Windows 2000](https://minesweepergame.com/download/windows-2000-minesweeper.php)). One page credits XP; the weight of the sources says 2000.
  - The minimum custom size in older versions is reportedly 8×8 ([msgames.fandom](https://msgames.fandom.com/wiki/Minesweeper)): **unverified**.

### 5. Other layout-affecting rules

- `Game::new` clamps mines to `w*h-1` (`src/game.rs:108`). XP has no such clamp. The UI can't reach that limit in either game, so it doesn't matter in practice.
- Neighbour counts:
  - Runesweeper recomputes them after relocation (`src/game.rs:180`).
  - XP counts on reveal (`CountNearBombs`, `game.c:331`).
  - The result is the same either way.

## Open questions

- Confirm against the XP binary that the column is drawn before the row, and that the relocation loop really is `<` and not `<=`. That means disassembling `winmine.exe` (sha256 above), not relying on the reconstruction. The `<` bound is supported by Bright's 63-mine experiment.
- Does the `msvcrt.dll` that ships with XP use the same LCG? Is winmine statically linked? Which CRT did the 16-bit Windows 3.1 build use, and how did it seed?
- Is the relocation scan the same in Windows 3.1/95/98? repnz says the NT4 source is "almost identical", which suggests yes.
- What does Vista/7 actually do on the first click: relocate, regenerate, or clear a 3×3 area? This needs a reverse-engineering source.
- Is the XP registry's Width maximum really 25?
- To match exactly, Runesweeper would need:
  - the MSVC LCG, seeded once per session;
  - two draws per mine, column first;
  - the relocation scan limited to `y in 0..h-1, x in 0..w-1` (0-based), with a "no move" fallback.

  It also has to decide whether to emulate 8×8 Beginner for pre-2000 versions.

## Sources

- repnz, ReversingMinesweeper (XP winmine reconstruction): https://github.com/repnz/ReversingMinesweeper (`Minesweeper/game.c`, `config.c`, `windowing.c`, `utils.c`, `game.h`)
- Microsoft UCRT `rand.cpp`, mirrored by ReactOS: https://github.com/reactos/reactos/blob/master/sdk/lib/ucrt/stdlib/rand.cpp
- Curtis Bright, Minesweeper bugs / 1-second theory (secondary, snippet only): https://www.curtisbright.com/msx/v0/minesweeperbugs.html, https://www.curtisbright.com/msx/v0/1sectheory.html
- minesweepergame.com, One Click Bug and Windows 2000 pages (secondary, snippet only): https://minesweepergame.com/history/one-click-bug.php, https://minesweepergame.com/download/windows-2000-minesweeper.php
- The Cutting Room Floor, Minesweeper (Windows, 1990) (secondary, snippet only): https://tcrf.net/Minesweeper_(Windows,_1990)

Not reachable from this session's network, so not used directly: learn.microsoft.com, tcrf.net, minesweepergame.com, doxygen.reactos.org.
