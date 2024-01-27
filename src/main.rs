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
    collections::HashMap,
    env, fs,
    net::{TcpListener, TcpStream},
    process::ExitCode,
    sync::{Arc, Mutex},
    thread::spawn,
};

use game::state::GameState;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use rand::distributions::{Alphanumeric, DistString};
use tungstenite::{accept, WebSocket};

use crate::game::{order::Order, state::Owner};

pub mod game;
pub mod vec2;

fn display_usage(name: &str) {
    eprintln!("usage:");
    eprintln!("  {name} new <filename> <player_count>");
    eprintln!("  {name} load <filename>");
}

fn display_cert_hint() {
    eprintln!("info: try running `openssl req -x509 -keyout key.pem -out cert.pem -sha256 -days 365 -noenc`");
    eprintln!(
        "info:    and then `openssl pkcs12 -export -out cert.p12 -inkey key.pem -in cert.pem`"
    );
    eprintln!("info: and using an empty password");
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
    let (mut game_state, filename) = match args[1].as_str() {
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
                        "error: invalid number of players - expected a number between 2 and 6, but got {}",
                        &args[3]
                    );
                    return ExitCode::FAILURE;
                }
            } else {
                eprintln!("error: could not parse number of players - expected a number between 2 and 6, but got {}", &args[3]);
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
                    eprintln!("error: could not parse save file: {message}");
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
            display_cert_hint();
            return ExitCode::FAILURE;
        }
    };
    let identity = match Identity::from_pkcs12(&identity, "") {
        Ok(identity) => identity,
        Err(err) => {
            eprintln!("error: could not read certificate: {err}");
            display_cert_hint();
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
            display_cert_hint();
            return ExitCode::FAILURE;
        }
    };

    type TlsWebSocket = WebSocket<TlsStream<TcpStream>>;

    let mut threads = vec![];
    let mut clients: Arc<Mutex<HashMap<Owner, (TlsWebSocket, &str, Option<Vec<Order>>)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    for stream in listener.incoming() {
        // TODO: make listener nonblocking
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                threads.push(spawn(move || -> ExitCode {
                    let stream = match acceptor.accept(stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("error: tls connection failed: {err}");
                            return ExitCode::FAILURE;
                        }
                    };
                    let mut websocket = match accept(stream) {
                        Ok(websocket) => websocket,
                        Err(err) => {
                            eprintln!("error: websocket connection failed: {err}");
                            return ExitCode::FAILURE;
                        }
                    };

                    // read login packet - expect a username and a password

                    // if logged in successfully

                    // send assigned player id

                    // while game isn't over

                    // send game state

                    // get orders

                    // maybe update game state

                    ExitCode::SUCCESS
                }));
            }
            Err(err) => {
                eprintln!("info: got invalid connection: {err}");
            }
        }
    }

    ExitCode::SUCCESS
}
