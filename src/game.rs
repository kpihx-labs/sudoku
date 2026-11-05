use crate::engine::{Board, Difficulty, Puzzle, candidates, conflict, generate};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct Move {
    index: usize,
    value: u8,
    notes: u16,
}

pub struct Game {
    pub values: Board,
    pub givens: [bool; 81],
    pub notes: [u16; 81],
    pub solution: Board,
    pub cursor: usize,
    pub difficulty: Difficulty,
    pub seed: u64,
    pub pencil: bool,
    pub mistakes: u32,
    pub hints: u32,
    pub won: bool,
    history: Vec<Move>,
    started: Instant,
    finished: Option<Duration>,
}

impl Game {
    pub fn new(difficulty: Difficulty, seed: Option<u64>) -> Self {
        Self::from_puzzle(generate(difficulty, seed))
    }

    fn from_puzzle(puzzle: Puzzle) -> Self {
        Self {
            values: puzzle.puzzle,
            givens: puzzle.puzzle.map(|value| value != 0),
            notes: [0; 81],
            solution: puzzle.solution,
            cursor: puzzle
                .puzzle
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(0),
            difficulty: puzzle.difficulty,
            seed: puzzle.seed,
            pencil: false,
            mistakes: 0,
            hints: 0,
            won: false,
            history: Vec::new(),
            started: Instant::now(),
            finished: None,
        }
    }

    pub fn restart(&mut self) {
        let difficulty = self.difficulty;
        let seed = self.seed;
        *self = Self::new(difficulty, Some(seed));
    }

    pub fn new_puzzle(&mut self) {
        *self = Self::new(self.difficulty, None);
    }

    pub fn elapsed(&self) -> Duration {
        self.finished.unwrap_or_else(|| self.started.elapsed())
    }

    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        let x = self.cursor % 9;
        let y = self.cursor / 9;
        let x = (x as isize + dx).clamp(0, 8) as usize;
        let y = (y as isize + dy).clamp(0, 8) as usize;
        self.cursor = y * 9 + x;
    }

    pub fn set(&mut self, value: u8) {
        if self.givens[self.cursor] || self.won || !(1..=9).contains(&value) {
            return;
        }
        self.remember();
        if self.pencil {
            self.notes[self.cursor] ^= 1 << value;
        } else {
            self.values[self.cursor] = value;
            self.notes[self.cursor] = 0;
            if value != self.solution[self.cursor] {
                self.mistakes += 1;
            } else {
                self.clear_peer_note(value);
            }
            self.check_win();
        }
    }

    pub fn erase(&mut self) {
        if self.givens[self.cursor] || self.won {
            return;
        }
        self.remember();
        self.values[self.cursor] = 0;
        self.notes[self.cursor] = 0;
    }

    pub fn undo(&mut self) {
        let Some(change) = self.history.pop() else {
            return;
        };
        self.values[change.index] = change.value;
        self.notes[change.index] = change.notes;
        self.cursor = change.index;
        self.won = false;
        self.finished = None;
    }

    pub fn hint(&mut self) {
        if self.givens[self.cursor] || self.won {
            return;
        }
        self.remember();
        let value = self.solution[self.cursor];
        self.values[self.cursor] = value;
        self.notes[self.cursor] = 0;
        self.hints += 1;
        self.clear_peer_note(value);
        self.check_win();
    }

    pub fn auto_notes(&mut self) {
        for index in 0..81 {
            if !self.givens[index] && self.values[index] == 0 {
                self.notes[index] = candidates(&self.values, index);
            }
        }
    }

    pub fn conflict(&self, index: usize) -> bool {
        conflict(&self.values, index)
            || (self.values[index] != 0 && self.values[index] != self.solution[index])
    }

    fn remember(&mut self) {
        self.history.push(Move {
            index: self.cursor,
            value: self.values[self.cursor],
            notes: self.notes[self.cursor],
        });
    }

    fn clear_peer_note(&mut self, value: u8) {
        let row = self.cursor / 9;
        let column = self.cursor % 9;
        for offset in 0..9 {
            self.notes[row * 9 + offset] &= !(1 << value);
            self.notes[offset * 9 + column] &= !(1 << value);
        }
        let box_row = row / 3 * 3;
        let box_column = column / 3 * 3;
        for y in box_row..box_row + 3 {
            for x in box_column..box_column + 3 {
                self.notes[y * 9 + x] &= !(1 << value);
            }
        }
    }

    fn check_win(&mut self) {
        if self.values == self.solution {
            self.won = true;
            self.finished = Some(self.started.elapsed());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn givens_cannot_be_changed() {
        let mut game = Game::new(Difficulty::Easy, Some(42));
        game.cursor = game.givens.iter().position(|given| *given).expect("given");
        let value = game.values[game.cursor];
        game.set(if value == 9 { 1 } else { value + 1 });
        assert_eq!(game.values[game.cursor], value);
    }

    #[test]
    fn undo_restores_value_and_notes() {
        let mut game = Game::new(Difficulty::Easy, Some(42));
        game.cursor = game.givens.iter().position(|given| !given).expect("empty");
        game.pencil = true;
        game.set(3);
        game.pencil = false;
        game.set(game.solution[game.cursor]);
        game.undo();
        assert_eq!(game.values[game.cursor], 0);
        assert_ne!(game.notes[game.cursor] & (1 << 3), 0);
    }
}
