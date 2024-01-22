use std::{env, process::ExitCode};

use game::state::GameState;

pub mod game;
pub mod vec2;

fn display_usage(name: &str) {
    eprintln!("usage:");
    eprintln!("  {name} new <filename> <player_count>");
    eprintln!("  {name} load <filename>");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        display_usage(if args.is_empty() {
            "solar_dawn_server"
        } else {
            &args[0]
        });
        return ExitCode::FAILURE;
    }

    // setup game state
    let (mut state, filename) = match args[1].as_str() {
        "new" => {
            if args.len() != 4 {
                display_usage(&args[0]);
                return ExitCode::FAILURE;
            }

            if let Ok(num_players) = args[3].parse::<u8>() {
                if let Ok(initial_state) = GameState::new(num_players) {
                    initial_state.save_to_file(&args[2]);
                    (initial_state, &args[2])
                } else {
                    eprintln!(
                        "invalid number of players - expected a number between 2 and 6, but got {}",
                        &args[3]
                    );
                    return ExitCode::FAILURE;
                }
            } else {
                eprintln!("could not parse number of players - expected a number between 2 and 6, but got {}", &args[3]);
                return ExitCode::FAILURE;
            }
        }
        "load" => {
            if args.len() != 3 {
                display_usage(&args[0]);
                return ExitCode::FAILURE;
            }

            match GameState::load_from_file(&args[2]) {
                Ok(state) => (state, &args[2]),
                Err(message) => {
                    eprintln!("could not parse save file: {message}");
                    return ExitCode::FAILURE;
                }
            }
        }
        _ => {
            display_usage(&args[0]);
            return ExitCode::FAILURE;
        }
    };

    // set up websocket server

    ExitCode::SUCCESS
}
