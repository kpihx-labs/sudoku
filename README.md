# sudoku

`sudoku` is a native Rust Sudoku game, generator, and solver designed for real terminal use. It keeps the core independent from rendering, generates symmetric puzzles with exactly one solution, and expands from a compact 27-column board to full 3×3 pencil grids when the terminal provides space.

## Features

- Easy, medium, hard, and expert generation.
- 180-degree symmetric clue removal with uniqueness checked after every removal.
- MRV bitmask solver and solution counter.
- Deterministic `--seed` support for shareable puzzles.
- Full-screen responsive cells with automatic pencil-mark grids.
- Given, entered, matching, selected, and conflicting cell styling.
- Pencil marks, legal auto-notes, undo, erase, and hints.
- Frozen solved board with explicit new/replay/scores/quit actions.
- Best solves ranked by assistance, mistakes, then elapsed time.
- Standalone generate and solve commands for shell pipelines.

## Install

```bash
make install
```

## Play

```bash
sudoku
sudoku --difficulty hard
sudoku --difficulty expert --seed 42
```

| Action | Key |
|---|---|
| Move | Arrows or `hjkl` |
| Enter value | `1`–`9` |
| Toggle pencil mode | `P` |
| Fill legal pencil marks | `A` |
| Erase | `X`, Backspace, or Delete |
| Undo | `U` |
| Hint selected cell | `I` |
| Best solves | `S` |
| New puzzle / replay | `N` / `R` |
| Help / quit | `?` / `Q` |

Right click toggles pencil mode; left click selects a cell. Keyboard control remains complete.

## Generate and solve

```bash
sudoku generate --difficulty expert --seed 42
sudoku solve 530070000600195000098000060800060003400803001700020006060000280000419005000080079
```

Input accepts digits plus `.`, `_`, or `0` for blanks. Generated puzzles are emitted as 81 characters for transparent scripting.

## Persistence

Best solves are stored under the platform data directory at `sudoku/scores.json`, normally `$HOME/.local/share/sudoku/scores.json` on Linux. Rankings are independent per difficulty.

## Research references

The ignored `refs/` workspace contains read-only clones used for architectural research:

- `ianprime0509/rsudoku`: solver/generator/TUI precedent.
- `huangsam/rustoku`: bitmask, MRV, human-solving, and WebAssembly precedent.
- `kcirtapfromspace/sudoku-core`: uniqueness, symmetric generation, hints, and difficulty research.

No source from those projects is copied into this original implementation.

## Quality

```bash
make check
```

The local gate runs formatting, strict Clippy, all tests, a locked release build, generation, solving, and CLI smoke tests.

## License

Business Source License 1.1. Personal, noncommercial, teaching, and academic production use is granted; the project converts to `AGPL-3.0-only` on 2030-11-05.
