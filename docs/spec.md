# Modes, hints, history: the agreed design

Settled in a design session on 2026-11-06. Terms are defined in [GLOSSARY.md](../GLOSSARY.md); the research behind it is [docs/research/minesweeper-solvers-and-generators.md](research/minesweeper-solvers-and-generators.md). ADRs: [0001](adr/0001-new-modes-generate-on-first-click.md), [0002](adr/0002-hints-teach-never-compute.md), [0003](adr/0003-misans-for-chinese-text.md).

The player is assumed to know Minesweeper. The game carries no help text.

## Modes

Chosen independently of the Level (Beginner, Intermediate, Expert, Custom). Changing either starts a new field.

| Mode | zh-CN | Field |
|---|---|---|
| Classic | 原版 | Windows XP exactly: laid at New Game from the MSVC `rand()` stream, a first-click mine moved to the first free cell, no guaranteed opening. Default. |
| Gentle No Guess | 简单无猜 | Finishable without guessing using only one number, or two overlapping numbers, plus the mine count near the end. |
| No Guess | 无猜 | Finishable without guessing by perfect logic, including the count of mines left. |
| Max Guess | 死猜挑战 | Built to force as many true 50/50s as possible; every forced guess is a true 50/50; otherwise finishable by logic. |

- The three new modes lay mines only after the first click, keep the 3×3 around it clear (the first click is always an opening), and use their own random generator: a field is fully defined by (mode, level, seed, first click).
- Gentle No Guess and No Guess redraw uniform random fields until the solver can finish one, with no time limit. No mine-moving repair, so accepted fields are a uniform draw over solvable fields.
- Max Guess searches from a random field, with the solver in the loop, for the most true 50/50s within a fixed number of search steps per level (steps, not time, so the field stays determined by its inputs). The count is shown before play; the end banner says how many were survived and whether a loss was forced. No undo.
- Generation runs off the UI thread. On the first click the face shows a thinking state and the field takes no input; after 600 ms a slow band of light sweeps the hidden tiles until the field is ready. The timer starts when the field is ready. F2 or the face cancels. Seconds of generation are acceptable; the budget is not fixed yet.
- The mine counter stays mines minus flags.

## Hints

Available in every mode: the H key or the Hint button.

- Each press goes one step deeper: a nudge naming the technique, then the numbers involved, then the cell and action, then the full reason with Apply. Any move resets to the first step. The reason sits in a callout beside the field, never over the cells it talks about.
- Reasoning uses only opened numbers, never the player's flags. A flag that is provably wrong is pointed out first.
- Picks what a strong player sees first: a named pattern over raw logic on the same numbers, then fewest numbers, then nearest the last move.
- Covers the full catalogue of both the English and Chinese communities (minesweeper.online's 28 patterns, the Chinese 定式 set, mine counting) and names each pattern and its grade (Easy, Medium, Hard, Evil).
- Never shows a probability. A cell is named only with a reason the player can check by hand: a deduction, a named pattern, a small count of arrangements, fewer-mine arrangements being more likely, a true 50/50 (take it now), a cell that can only lead to another guess (avoid it), a cell likely to open the board, and in Classic the XP first-click quirk (a moved mine lands in the top row). The exact solver only chooses what to say. When nothing a player could follow favours any cell, the hint says a guess is needed and names no cell.
- Unlimited, no time penalty. A game with any hint is an assisted game.

## History and Review

- No best-times table, name prompt or saved best times (old data is dropped).
- A game record is written for every game won, lost, or abandoned after its first click: date, mode, level, result, seconds, 3BV and 3BV solved, clicks (left, right, chord), hints shown, forced guesses faced and survived, seed and first click (Classic stores its mine layout), and every move with its time. Kept without limit: a file beside settings natively, browser storage on the web (oldest dropped only if a write is refused).
- History lists records, filtered by mode and level, with the fastest win per mode × level on top. The timer turns gold when a win beats it.
- Review (复盘): from the result banner or any History record, the game replays on the field with a timeline below it. Markers flag missed deductions, needless guesses, forced guesses and clicks a chord would have saved; selecting one shows the same reason callout as a hint. Shows 3BV, 3BV/s and efficiency.
- A "techniques learned" view is deferred.

## Interface

- The XP menu bar, Windows-style dialogs, How to Play, About and the first-run controls line are removed.
- Kept: the field, the face, the LED counter and timer, the effects.
- A slim toolbar: a chip showing mode and level (opens a picker panel: four modes, four levels, Custom sizes inline; picking starts a new field), and Hint, History, Settings buttons. Panels slide in, styled like the field.
- Settings: Marks (?), Color, Language, and the line "Chinese text: MiSans by Xiaomi".

## Language

- English and zh-CN. Default from the system language (native) or browser language (web): any Chinese locale means zh-CN. Changeable in Settings, saved.
- Everything visible is translated, hints included. Chinese uses the community's words (定式, 121定式, 减雷, 死猜, 必须猜, 空, 双击, 纪录, 历史记录, 复盘) and its sentence form 「看这个 N，它周围还有 M 个雷……所以这一格必定安全」. Patterns without a Chinese name get descriptive names in the same style (三角定式); dead cell is 无信息格.
- Chinese is drawn in MiSans VF, Latin and digits in Inter. The official MiSans file and its agreement are committed under `assets/fonts/MiSans/`; the build cuts a subset of the characters used and compiles it into native and wasm binaries. The font is not MIT.

## Solver benchmark

A headless benchmark in `examples/` plays N seeded games per mode and level and reports win rate with a 95% Wilson interval, guesses per win and time per game. The README states Classic Expert results and the rules they were measured under.

## Build order

Each step is committed and pushed to `main` when it works.

1. Solver core and benchmark
2. Modes and generation
3. Game records and History
4. Toolbar and panels; help text removed
5. Hints
6. zh-CN and MiSans
7. Review
