# Runtime Contract

## Engine

- Every generated puzzle has exactly one solution.
- Clue removals are 180-degree symmetric and revalidated after each removal.
- Solving chooses the empty cell with the fewest candidates and uses row/column/box bitmasks.
- Solution counting stops at its requested limit.
- Seeded generation is reproducible.

## Game

- Given cells are immutable.
- Entered conflicts and incorrect values remain visible in red rather than being silently rejected.
- Undo restores both a cell value and its pencil marks.
- Auto-notes use current legal candidates; placing a correct value clears that candidate from peers.
- The solved board remains visible under a completion overlay until explicit action.

## Interface and persistence

- A compact terminal shows all 81 cells; larger terminals expand to 3×3 pencil grids.
- Keyboard operation is complete without a mouse.
- Terminal raw mode, alternate screen, cursor, and mouse capture are restored on exit.
- Rankings are independent by difficulty and prefer fewer hints, fewer mistakes, then lower time.
