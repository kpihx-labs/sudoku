use crate::engine::Difficulty;

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub difficulty: Difficulty,
    pub seed: Option<u64>,
}

pub enum Action {
    Play(Options),
    Generate(Options),
    Solve(String),
    Print(String),
}

pub fn parse(args: impl Iterator<Item = String>) -> Result<Action, String> {
    let mut args = args.peekable();
    let mut command = "play".to_string();
    if matches!(
        args.peek().map(String::as_str),
        Some("play" | "generate" | "solve")
    ) {
        command = args.next().expect("peeked argument");
    }
    if command == "solve" {
        return args
            .next()
            .map(Action::Solve)
            .ok_or_else(|| "solve requires an 81-character puzzle".into());
    }

    let mut difficulty = Difficulty::Medium;
    let mut seed = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-h" | "--help" => return Ok(Action::Print(help())),
            "-V" | "--version" => {
                return Ok(Action::Print(format!(
                    "sudoku {}",
                    env!("CARGO_PKG_VERSION")
                )));
            }
            "-d" | "--difficulty" => {
                let value = args.next().ok_or("--difficulty requires a value")?;
                difficulty = value.parse()?;
            }
            "--seed" => {
                seed = Some(
                    args.next()
                        .ok_or("--seed requires a value")?
                        .parse()
                        .map_err(|_| "invalid seed")?,
                );
            }
            unknown => return Err(format!("unknown option: {unknown}")),
        }
    }
    let options = Options { difficulty, seed };
    Ok(if command == "generate" {
        Action::Generate(options)
    } else {
        Action::Play(options)
    })
}

pub fn help() -> String {
    format!(
        "sudoku {}\nA responsive Sudoku game, generator, and solver.\n\nUSAGE:\n  sudoku [play] [OPTIONS]\n  sudoku generate [OPTIONS]\n  sudoku solve <81 CELLS>\n\nOPTIONS:\n  -d, --difficulty <LEVEL>  easy, medium, hard, or expert [default: medium]\n      --seed <NUMBER>        Reproducible puzzle seed\n  -h, --help                 Print help\n  -V, --version              Print version\n\nEXAMPLES:\n  sudoku --difficulty hard\n  sudoku generate --difficulty expert --seed 42\n  sudoku solve 530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_medium_play() {
        let Action::Play(options) = parse(std::iter::empty()).expect("default") else {
            panic!("expected play");
        };
        assert_eq!(options.difficulty, Difficulty::Medium);
    }

    #[test]
    fn parses_seeded_generation() {
        let args = ["generate", "-d", "expert", "--seed", "42"];
        let Action::Generate(options) =
            parse(args.into_iter().map(str::to_string)).expect("generate")
        else {
            panic!("expected generation");
        };
        assert_eq!(options.difficulty, Difficulty::Expert);
        assert_eq!(options.seed, Some(42));
    }
}
