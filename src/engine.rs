use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use std::str::FromStr;

pub type Board = [u8; 81];
const ALL: u16 = 0b11_1111_1110;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    pub fn clues(self) -> usize {
        match self {
            Self::Easy => 42,
            Self::Medium => 34,
            Self::Hard => 28,
            Self::Expert => 24,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Medium => "medium",
            Self::Hard => "hard",
            Self::Expert => "expert",
        }
    }
}

impl FromStr for Difficulty {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            _ => Err("difficulty must be easy, medium, hard, or expert".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Puzzle {
    pub puzzle: Board,
    pub solution: Board,
    pub difficulty: Difficulty,
    pub seed: u64,
}

pub fn generate(difficulty: Difficulty, seed: Option<u64>) -> Puzzle {
    let seed = seed.unwrap_or_else(rand::random);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut solution = [0; 81];
    fill_random(&mut solution, &mut rng);
    let mut puzzle = solution;
    let mut positions: Vec<usize> = (0..41).collect();
    positions.shuffle(&mut rng);
    for index in positions {
        if puzzle.iter().filter(|value| **value != 0).count() <= difficulty.clues() {
            break;
        }
        let mirror = 80 - index;
        let first = puzzle[index];
        let second = puzzle[mirror];
        puzzle[index] = 0;
        puzzle[mirror] = 0;
        let removed = if index == mirror { 1 } else { 2 };
        if puzzle.iter().filter(|value| **value != 0).count() < difficulty.clues()
            || count_solutions(&mut puzzle, 2) != 1
        {
            puzzle[index] = first;
            puzzle[mirror] = second;
        } else if removed == 1
            && puzzle.iter().filter(|value| **value != 0).count() < difficulty.clues()
        {
            puzzle[index] = first;
        }
    }
    Puzzle {
        puzzle,
        solution,
        difficulty,
        seed,
    }
}

pub fn solve(board: &mut Board) -> bool {
    let Some((index, mask)) = next_cell(board) else {
        return valid_complete(board);
    };
    for value in digits(mask) {
        board[index] = value;
        if solve(board) {
            return true;
        }
    }
    board[index] = 0;
    false
}

pub fn count_solutions(board: &mut Board, limit: usize) -> usize {
    count_recursive(board, limit)
}

fn count_recursive(board: &mut Board, limit: usize) -> usize {
    let Some((index, mask)) = next_cell(board) else {
        return usize::from(valid_complete(board));
    };
    let mut count = 0;
    for value in digits(mask) {
        board[index] = value;
        count += count_recursive(board, limit - count);
        if count >= limit {
            break;
        }
    }
    board[index] = 0;
    count
}

fn fill_random(board: &mut Board, rng: &mut StdRng) -> bool {
    let Some((index, mask)) = next_cell(board) else {
        return true;
    };
    let mut values = digits(mask);
    values.shuffle(rng);
    for value in values {
        board[index] = value;
        if fill_random(board, rng) {
            return true;
        }
    }
    board[index] = 0;
    false
}

fn next_cell(board: &Board) -> Option<(usize, u16)> {
    board
        .iter()
        .enumerate()
        .filter(|(_, value)| **value == 0)
        .map(|(index, _)| (index, candidates(board, index)))
        .min_by_key(|(_, mask)| mask.count_ones())
}

pub fn candidates(board: &Board, index: usize) -> u16 {
    if board[index] != 0 {
        return 0;
    }
    let row = index / 9;
    let column = index % 9;
    let mut used = 0;
    for offset in 0..9 {
        used |= 1 << board[row * 9 + offset];
        used |= 1 << board[offset * 9 + column];
    }
    let box_row = row / 3 * 3;
    let box_column = column / 3 * 3;
    for y in box_row..box_row + 3 {
        for x in box_column..box_column + 3 {
            used |= 1 << board[y * 9 + x];
        }
    }
    ALL & !used
}

pub fn conflict(board: &Board, index: usize) -> bool {
    let value = board[index];
    if value == 0 {
        return false;
    }
    let row = index / 9;
    let column = index % 9;
    (0..9).any(|offset| {
        (offset != column && board[row * 9 + offset] == value)
            || (offset != row && board[offset * 9 + column] == value)
    }) || {
        let box_row = row / 3 * 3;
        let box_column = column / 3 * 3;
        (box_row..box_row + 3).any(|y| {
            (box_column..box_column + 3).any(|x| y * 9 + x != index && board[y * 9 + x] == value)
        })
    }
}

fn valid_complete(board: &Board) -> bool {
    board.iter().all(|value| *value != 0)
        && board
            .iter()
            .enumerate()
            .all(|(index, _)| !conflict(board, index))
}

fn digits(mask: u16) -> Vec<u8> {
    (1..=9).filter(|value| mask & (1 << value) != 0).collect()
}

pub fn parse(text: &str) -> Result<Board, String> {
    let cells: Vec<_> = text
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '|')
        .collect();
    if cells.len() != 81 {
        return Err("puzzle must contain exactly 81 cells".into());
    }
    let mut board = [0; 81];
    for (index, character) in cells.into_iter().enumerate() {
        board[index] = match character {
            '.' | '_' | '0' => 0,
            '1'..='9' => character as u8 - b'0',
            _ => return Err(format!("invalid puzzle character: {character}")),
        };
    }
    if board
        .iter()
        .enumerate()
        .any(|(index, value)| *value != 0 && conflict(&board, index))
    {
        return Err("puzzle contains conflicting givens".into());
    }
    Ok(board)
}

pub fn encode(board: &Board) -> String {
    board
        .iter()
        .map(|value| {
            if *value == 0 {
                '.'
            } else {
                char::from(b'0' + *value)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn solves_known_puzzle() {
        let mut board = parse(PUZZLE).expect("valid puzzle");
        assert!(solve(&mut board));
        assert!(valid_complete(&board));
    }

    #[test]
    fn generated_puzzle_is_unique_and_symmetric() {
        let generated = generate(Difficulty::Medium, Some(42));
        let mut puzzle = generated.puzzle;
        assert_eq!(count_solutions(&mut puzzle, 2), 1);
        for index in 0..81 {
            assert_eq!(
                generated.puzzle[index] == 0,
                generated.puzzle[80 - index] == 0
            );
        }
    }

    #[test]
    fn deterministic_seed_reproduces_puzzle() {
        assert_eq!(
            generate(Difficulty::Hard, Some(7)).puzzle,
            generate(Difficulty::Hard, Some(7)).puzzle
        );
    }

    #[test]
    fn rejects_conflicting_givens() {
        let mut invalid = PUZZLE.to_string();
        invalid.replace_range(1..2, "5");
        assert!(parse(&invalid).is_err());
    }
}
