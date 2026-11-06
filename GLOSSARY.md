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

**Max Guess**:
The Mode whose fields are built to force as many true 50/50s as possible, and are otherwise solvable by logic.
_Avoid_: Evil, cruel, hard mode

**First click**:
The first cell opened on a field.

**Opening**:
A connected area of empty cells and the numbers around it, revealed by a single click.
_Avoid_: Zero, blank area

## Guessing

**Forced guess**:
A position where no hidden cell can be proven safe, so the player must risk a cell.

**True 50/50**:
A forced guess between two cells that no future information can ever tell apart, each a mine with equal chance.
_Avoid_: Coin flip

## Help

**Hint**:
On request, the simplest next deduction, revealed step by step, with the reason in words a player would use.
_Avoid_: Clue, solver, help

**Assisted game**:
A game in which at least one Hint was shown.
