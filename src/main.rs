// Copyright 2023 Justin Hu
//
// This file is part of the Solar Dawn Server.
//
// The Solar Dawn Server is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// The Solar Dawn Server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero
// General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with the Solar Dawn Server. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    env, fs,
    net::{TcpListener, TcpStream},
    process::ExitCode,
    sync::Arc,
    thread::spawn,
};

use game::state::GameState;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use tungstenite::{accept, Error, Message, WebSocket};

pub mod game;
pub mod vec2;

fn display_usage(name: &str) {
    eprintln!("usage:");
    eprintln!("  {name} new <filename> <player_count>");
    eprintln!("  {name} load <filename>");
}

fn main() -> ExitCode {
    println!("Solar Dawn version 0.1.0");
    println!("Copyright 2024 Justin Hu");
    println!("This is free software; see the source for copying conditions. There is NO");
    println!("warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.");
    println!();

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
    let (mut gameState, filename) = match args[1].as_str() {
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
    let password = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    println!("password is {password}");

    let identity = match fs::read("cert.p12") {
        Ok(identity) => identity,
        Err(err) => {
            eprintln!("error: could not read certificate: {err}");
            return ExitCode::FAILURE;
        }
    };
    let identity = match Identity::from_pkcs12(&identity, "") {
        Ok(identity) => identity,
        Err(err) => {
            eprintln!("error: could not read certificate: {err}");
            return ExitCode::FAILURE;
        }
    };
    let listener = match TcpListener::bind("127.0.0.1:21316") {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("error: could not start server: {err}");
            return ExitCode::FAILURE;
        }
    };
    let acceptor = match TlsAcceptor::new(identity) {
        Ok(acceptor) => Arc::new(acceptor),
        Err(err) => {
            eprintln!("error: could not use certificate: {err}");
            return ExitCode::FAILURE;
        }
    };

    let mut clients = vec![];
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                clients.push(spawn(move || {
                    let stream = match acceptor.accept(stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("error: tls connection failed: {err}");
                            return;
                        }
                    };
                    let mut websocket = match accept(stream) {
                        Ok(websocket) => websocket,
                        Err(err) => {
                            eprintln!("error: websocket connection failed: {err}");
                            return;
                        }
                    };

                    let _ = websocket.send(Message::Text("Hello, world!".to_owned()));

                    // get the first message
                }));
            }
            Err(err) => {
                eprintln!("info: got invalid connection: {err}");
            }
        }
    }

    ExitCode::SUCCESS
}
