use crate::{
    cli::Options,
    game::Game,
    scores::{Scores, format_time},
};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{Stdout, Write, stdout},
    time::Duration,
};

pub struct App {
    game: Game,
    scores: Scores,
    modal: Modal,
    recorded: bool,
    geometry: Geometry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modal {
    None,
    Help,
    Scores,
}

#[derive(Debug, Clone, Copy, Default)]
struct Geometry {
    cell_width: u16,
    cell_height: u16,
    x: u16,
    y: u16,
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut Stdout) -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            Show,
            DisableMouseCapture,
            LeaveAlternateScreen,
            ResetColor
        );
        let _ = terminal::disable_raw_mode();
    }
}

impl App {
    pub fn new(options: Options) -> std::io::Result<Self> {
        Ok(Self {
            game: Game::new(options.difficulty, options.seed),
            scores: Scores::load(),
            modal: Modal::None,
            recorded: false,
            geometry: Geometry::default(),
        })
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut stdout = stdout();
        let _guard = TerminalGuard::enter(&mut stdout)?;
        loop {
            self.draw(&mut stdout)?;
            if event::poll(Duration::from_millis(100))? && !self.handle(event::read()?) {
                break;
            }
        }
        Ok(())
    }

    fn handle(&mut self, event: Event) -> bool {
        if self.modal != Modal::None {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.modal = Modal::None,
                    KeyCode::Char('?') if self.modal == Modal::Help => self.modal = Modal::None,
                    KeyCode::Char('s') if self.modal == Modal::Scores => self.modal = Modal::None,
                    _ => {}
                }
            }
            return true;
        }
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return false,
                KeyCode::Char('?') => self.modal = Modal::Help,
                KeyCode::Char('s') => self.modal = Modal::Scores,
                KeyCode::Char('n') => {
                    self.game.new_puzzle();
                    self.recorded = false;
                }
                KeyCode::Char('r') => {
                    self.game.restart();
                    self.recorded = false;
                }
                KeyCode::Char('p') => self.game.pencil = !self.game.pencil,
                KeyCode::Char('a') => self.game.auto_notes(),
                KeyCode::Char('u') => self.game.undo(),
                KeyCode::Char('i') => self.game.hint(),
                KeyCode::Char('x') | KeyCode::Backspace | KeyCode::Delete => self.game.erase(),
                KeyCode::Up | KeyCode::Char('k') => self.game.move_cursor(0, -1),
                KeyCode::Down | KeyCode::Char('j') => self.game.move_cursor(0, 1),
                KeyCode::Left | KeyCode::Char('h') => self.game.move_cursor(-1, 0),
                KeyCode::Right | KeyCode::Char('l') => self.game.move_cursor(1, 0),
                KeyCode::Char(value @ '1'..='9') => self.game.set(value as u8 - b'0'),
                _ => {}
            },
            Event::Mouse(mouse) => {
                if let Some(index) = self.mouse_cell(mouse.column, mouse.row) {
                    self.game.cursor = index;
                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Right)) {
                        self.game.pencil = !self.game.pencil;
                    }
                }
            }
            _ => {}
        }
        true
    }

    fn draw(&mut self, stdout: &mut Stdout) -> std::io::Result<()> {
        if self.game.won && !self.recorded {
            self.scores.record(
                self.game.difficulty,
                self.game.elapsed(),
                self.game.mistakes,
                self.game.hints,
            )?;
            self.recorded = true;
        }
        let (width, height) = terminal::size()?;
        self.geometry = geometry(width, height);
        queue!(stdout, Clear(ClearType::All))?;
        self.draw_header(stdout, width)?;
        self.draw_grid(stdout)?;
        self.draw_footer(stdout, width, height)?;
        match self.modal {
            Modal::Help => self.draw_help(stdout, width, height)?,
            Modal::Scores => self.draw_scores(stdout, width, height)?,
            Modal::None if self.game.won => self.draw_win(stdout, width, height)?,
            Modal::None => {}
        }
        stdout.flush()
    }

    fn draw_header(&self, stdout: &mut Stdout, width: u16) -> std::io::Result<()> {
        let text = format!(
            " SUDOKU  {}  time {}  mistakes {}  hints {}  {}  seed {} ",
            self.game.difficulty.name(),
            format_time(self.game.elapsed()),
            self.game.mistakes,
            self.game.hints,
            if self.game.pencil { "PENCIL" } else { "VALUE" },
            self.game.seed
        );
        queue!(
            stdout,
            MoveTo(0, 0),
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(center(&text, width as usize)),
            ResetColor
        )
    }

    fn draw_grid(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let geometry = self.geometry;
        let selected_value = self.game.values[self.game.cursor];
        for index in 0..81 {
            let column = index % 9;
            let row = index / 9;
            let x = geometry.x + column as u16 * geometry.cell_width;
            let y = geometry.y + row as u16 * geometry.cell_height;
            let peer = selected_value != 0 && self.game.values[index] == selected_value;
            let base = if (row / 3 + column / 3) % 2 == 0 {
                Color::Black
            } else {
                Color::Rgb {
                    r: 18,
                    g: 18,
                    b: 28,
                }
            };
            let background = if index == self.game.cursor {
                Color::DarkCyan
            } else if peer {
                Color::DarkBlue
            } else {
                base
            };
            for inner_y in 0..geometry.cell_height {
                let content = self.cell_line(index, inner_y, geometry.cell_width);
                let foreground = if self.game.conflict(index) {
                    Color::Red
                } else if self.game.givens[index] {
                    Color::White
                } else {
                    Color::Cyan
                };
                queue!(
                    stdout,
                    MoveTo(x, y + inner_y),
                    SetBackgroundColor(background),
                    SetForegroundColor(foreground),
                    SetAttribute(if self.game.givens[index] {
                        Attribute::Bold
                    } else {
                        Attribute::Reset
                    }),
                    Print(content),
                    ResetColor,
                    SetAttribute(Attribute::Reset)
                )?;
            }
        }
        self.draw_box_lines(stdout)
    }

    fn cell_line(&self, index: usize, inner_y: u16, width: u16) -> String {
        let value = self.game.values[index];
        if value != 0 {
            let text = if inner_y == self.geometry.cell_height / 2 {
                value.to_string()
            } else {
                String::new()
            };
            return center(&text, width as usize);
        }
        if self.geometry.cell_height >= 3 && self.game.notes[index] != 0 {
            let start = inner_y as u8 * 3 + 1;
            let text: String = (start..start + 3)
                .map(|candidate| {
                    if self.game.notes[index] & (1 << candidate) != 0 {
                        char::from(b'0' + candidate)
                    } else {
                        ' '
                    }
                })
                .collect();
            center(&text, width as usize)
        } else {
            center("·", width as usize)
        }
    }

    fn draw_box_lines(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        let geometry = self.geometry;
        for separator in [3, 6] {
            let x = geometry.x + separator * geometry.cell_width;
            for y in geometry.y..geometry.y + 9 * geometry.cell_height {
                queue!(
                    stdout,
                    MoveTo(x.saturating_sub(1), y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│")
                )?;
            }
            let y = geometry.y + separator * geometry.cell_height;
            queue!(
                stdout,
                MoveTo(geometry.x, y.saturating_sub(1)),
                SetForegroundColor(Color::DarkGrey),
                Print("─".repeat((9 * geometry.cell_width) as usize))
            )?;
        }
        Ok(())
    }

    fn draw_footer(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let text = " arrows/hjkl move  1-9 write  P pencil  A notes  X erase  U undo  I hint  S scores  ? help  N new  Q quit ";
        queue!(
            stdout,
            MoveTo(0, height.saturating_sub(1)),
            SetForegroundColor(Color::DarkGrey),
            Print(center(text, width as usize)),
            ResetColor
        )
    }

    fn draw_help(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let lines = vec![
            " SUDOKU HELP ".into(),
            "Fill every row, column, and 3x3 box with digits 1 through 9.".into(),
            "P toggles pencil marks; A calculates legal notes for every empty cell.".into(),
            "U undoes a value or note; I reveals the selected cell as a hint.".into(),
            "Wrong values stay visible in red so you remain in control.".into(),
            "Esc or ? closes help.".into(),
        ];
        overlay(stdout, width, height, &lines, Color::Cyan)
    }

    fn draw_scores(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let entries = self.scores.for_difficulty(self.game.difficulty);
        let mut lines = vec![format!(
            " {} BEST SOLVES ",
            self.game.difficulty.name().to_uppercase()
        )];
        if entries.is_empty() {
            lines.push("No completed puzzle yet.".into());
        } else {
            lines.extend(entries.iter().enumerate().map(|(index, score)| {
                format!(
                    "{:>2}. {}  mistakes {}  hints {}  {}",
                    index + 1,
                    format_time(Duration::from_millis(score.milliseconds as u64)),
                    score.mistakes,
                    score.hints,
                    score.date
                )
            }));
        }
        lines.push("Esc or S closes scores.".into());
        overlay(stdout, width, height, &lines, Color::Yellow)
    }

    fn draw_win(&self, stdout: &mut Stdout, width: u16, height: u16) -> std::io::Result<()> {
        let lines = vec![
            " PUZZLE SOLVED ".into(),
            format!(
                "{} · {} mistakes · {} hints",
                format_time(self.game.elapsed()),
                self.game.mistakes,
                self.game.hints
            ),
            "N new puzzle   R replay   S scores   Q quit".into(),
        ];
        overlay(stdout, width, height, &lines, Color::Green)
    }

    fn mouse_cell(&self, column: u16, row: u16) -> Option<usize> {
        let geometry = self.geometry;
        if column < geometry.x || row < geometry.y {
            return None;
        }
        let x = (column - geometry.x) / geometry.cell_width;
        let y = (row - geometry.y) / geometry.cell_height;
        (x < 9 && y < 9).then_some(y as usize * 9 + x as usize)
    }
}

fn geometry(width: u16, height: u16) -> Geometry {
    let available_height = height.saturating_sub(3);
    let cell_width = (width / 9).clamp(3, 9);
    let cell_height = (available_height / 9).clamp(1, 3);
    Geometry {
        cell_width,
        cell_height,
        x: width.saturating_sub(cell_width * 9) / 2,
        y: 1 + available_height.saturating_sub(cell_height * 9) / 2,
    }
}

fn overlay(
    stdout: &mut Stdout,
    width: u16,
    height: u16,
    lines: &[String],
    color: Color,
) -> std::io::Result<()> {
    let box_width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(20)
        .min(width.saturating_sub(4) as usize)
        + 4;
    let box_height = lines.len() as u16 + 2;
    let x = width.saturating_sub(box_width as u16) / 2;
    let y = height.saturating_sub(box_height) / 2;
    for row in 0..box_height {
        queue!(
            stdout,
            MoveTo(x, y + row),
            SetBackgroundColor(Color::Black),
            Print(" ".repeat(box_width))
        )?;
    }
    for (index, line) in lines.iter().enumerate() {
        queue!(
            stdout,
            MoveTo(x, y + index as u16 + 1),
            SetForegroundColor(color),
            SetBackgroundColor(Color::Black),
            SetAttribute(if index == 0 {
                Attribute::Bold
            } else {
                Attribute::Reset
            }),
            Print(center(&truncate(line, box_width - 2), box_width)),
            ResetColor,
            SetAttribute(Attribute::Reset)
        )?;
    }
    Ok(())
}

fn center(text: &str, width: usize) -> String {
    let text = truncate(text, width);
    let length = text.chars().count();
    let left = width.saturating_sub(length) / 2;
    format!(
        "{}{}{}",
        " ".repeat(left),
        text,
        " ".repeat(width - left - length)
    )
}

fn truncate(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_terminal_uses_full_pencil_grid() {
        let geometry = geometry(120, 34);
        assert_eq!(geometry.cell_height, 3);
        assert!(geometry.cell_width >= 7);
    }

    #[test]
    fn compact_terminal_still_fits_all_cells() {
        let geometry = geometry(30, 12);
        assert_eq!(geometry.cell_width, 3);
        assert_eq!(geometry.cell_height, 1);
    }
}
