# Runesweeper

Minesweeper with the rules of the Windows XP game, plus modes that change how the field is laid and a hint that explains its reasoning.

## Fields

**Level**:
The size and mine count of a field: Beginner, Intermediate, Expert or Custom.
_Avoid_: Difficulty, board size

**Mode**:
How a field's mines are laid, chosen independently of the Level.
_Avoid_: Generator, variant, game type

**Classic**:
The Mode that lays mines exactly as Windows XP does, first-click quirks included.
_Avoid_: OG, original, normal

**Gentle No Guess**:
The Mode whose fields can always be finished without guessing, using only deductions from one number or two overlapping numbers, plus the mine count near the end.
_Avoid_: Patterns, easy no-guess

**No Guess**:
The Mode whose fields can always be finished without guessing by perfect logic, including the count of mines left.
_Avoid_: Guess-free, NG, solvable mode

**Max Guess**:
The Mode whose fields are built to force as many true 50/50s as possible, and are otherwise solvable by logic.
_Avoid_: Evil, cruel, hard mode

**First click**:
The first cell opened on a field.

**Opening**:
A connected area of empty cells and the numbers around it, revealed by a single click.
_Avoid_: Zero, blank area

## Guessing

**Pattern**:
A named arrangement of numbers with a known deduction, such as 1-2-1, a hole or a triangle.
_Avoid_: Formation, trick

**Forced guess**:
A position where no hidden cell can be proven safe, so the player must risk a cell.

**True 50/50**:
A forced guess between two cells that no future information can ever tell apart, each a mine with equal chance.
_Avoid_: Coin flip

**Pseudo-50/50**:
A cell that is either safe or half of a true 50/50, so guessing it at once costs nothing.
_Avoid_: Fake 50/50

**Dead cell**:
A cell that, if safe, can show only one number, so opening it settles nothing and another guess follows.
_Avoid_: Useless cell

## Help

**Hint**:
On request, the simplest next deduction, or when none exists the reasoning a strong player would use to pick a guess, revealed step by step in a player's words.
_Avoid_: Clue, solver, help

**Assisted game**:
A game in which at least one Hint was shown.

## Records

**Game record**:
What is kept about one game that was won, lost, or abandoned after its first click.
_Avoid_: Score, entry

**History**:
All game records, kept without limit.
_Avoid_: Best times, high scores, stats

**Review**:
After a game, a walk through its moves that points out missed deductions, needless guesses and lost efficiency.
_Avoid_: Analysis, replay, post-mortem
