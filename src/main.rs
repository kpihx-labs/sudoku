use sudoku::{cli, ui::App};

fn main() {
    match cli::parse(std::env::args().skip(1)) {
        Ok(cli::Action::Play(options)) => {
            if let Err(error) = App::new(options).and_then(App::run) {
                eprintln!("sudoku: {error}");
                std::process::exit(1);
            }
        }
        Ok(cli::Action::Generate(options)) => {
            let puzzle = sudoku::engine::generate(options.difficulty, options.seed);
            println!("{}", sudoku::engine::encode(&puzzle.puzzle));
        }
        Ok(cli::Action::Solve(text)) => match sudoku::engine::parse(&text) {
            Ok(mut board) => {
                if sudoku::engine::solve(&mut board) {
                    println!("{}", sudoku::engine::encode(&board));
                } else {
                    eprintln!("sudoku: puzzle has no solution");
                    std::process::exit(1);
                }
            }
            Err(error) => {
                eprintln!("sudoku: {error}");
                std::process::exit(2);
            }
        },
        Ok(cli::Action::Print(text)) => println!("{text}"),
        Err(error) => {
            eprintln!("sudoku: {error}\n\n{}", cli::help());
            std::process::exit(2);
        }
    }
}
