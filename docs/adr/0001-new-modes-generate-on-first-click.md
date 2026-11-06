# New modes generate on the first click, with their own random numbers

Classic keeps Windows XP's generation exactly: mines laid when the field is made, the MSVC `rand()` stream continued across games, a first-click mine moved to the first free cell. Gentle No Guess, No Guess and Max Guess lay no mines until the first click, keep the 3×3 around it clear, and draw from a separate generator so a field is fully defined by its mode, level, seed and first click. Generating after the click is what lets those modes guarantee an opening and search for a solvable (or guess-heavy) field; keeping Classic apart preserves its fidelity to XP. Generation may take seconds and runs off the UI thread.

## Consequences

- A game has a state between the first click and a ready field; input waits and the timer has not started.
- Classic game records store the mine layout, since its fields cannot be rebuilt from a seed alone.
