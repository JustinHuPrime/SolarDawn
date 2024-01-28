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
    mem::take,
    net::{TcpListener, TcpStream},
    process::ExitCode,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::spawn,
};

use game::state::GameState;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use rand::distributions::{Alphanumeric, DistString};
use tungstenite::{
    accept,
    protocol::{frame::coding::CloseCode, CloseFrame},
    Error, Message, WebSocket,
};

use crate::{
    game::{
        order::{parse_orders, Order},
        state::Owner,
    },
    semaphore::Semaphore,
};

type TlsWebSocket = WebSocket<TlsStream<TcpStream>>;

pub mod game;
pub mod semaphore;
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
    let (game_state, filename) = match args[1].as_str() {
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
    println!("info: password is {password}");

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

    let num_players = game_state.num_players();
    let mut num_threads: u8 = 0;
    let orders_semaphore = Arc::new(Semaphore::new(0));
    let (termination_sender, termination_receiver) = channel();
    struct ServerState {
        game_state: GameState,
        orders: HashMap<Owner, Vec<Order>>,
    }
    let game_state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(ServerState {
        game_state,
        orders: HashMap::new(),
    }));
    'acceptor: for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let termination_sender = termination_sender.clone();
                let password = password.clone();
                let game_state = game_state.clone();
                let orders_semaphore = orders_semaphore.clone();
                let filename = filename.clone();
                spawn(move || {
                    fn terminated(termination_sender: &Sender<Result<(), ()>>) {
                        termination_sender.send(Err(())).expect(
                            "main thread should always outlive this thread up to this point",
                        );
                    }

                    fn recv(websocket: &mut TlsWebSocket) -> Result<String, &'static str> {
                        match websocket.read() {
                            Ok(Message::Text(str)) => Ok(str),
                            Ok(Message::Ping(content)) => {
                                let _ = websocket.send(Message::Pong(content)); // try to send a pong
                                recv(websocket)
                            }
                            Ok(Message::Close(_))
                            | Err(Error::ConnectionClosed)
                            | Err(Error::AlreadyClosed) => Err("websocket closed"),
                            Ok(_) => Err("unexpected message type"),
                            Err(_) => Err("websocket errored"),
                        }
                    }

                    fn try_send(websocket: &mut TlsWebSocket, message: String) {
                        let _ = websocket.send(Message::Text(message));
                    }
                    fn try_close(mut websocket: TlsWebSocket, close_frame: Option<CloseFrame<'_>>) {
                        let _ = websocket.close(close_frame);
                    }
                    fn send_message(
                        websocket: &mut TlsWebSocket,
                        message: String,
                    ) -> Result<(), &'static str> {
                        match websocket.send(Message::Text(message)) {
                            Err(Error::ConnectionClosed) | Err(Error::AlreadyClosed) => {
                                Err("websocket closed")
                            }
                            Err(_) => Err("websocket errored"),
                            _ => Ok(()),
                        }
                    }

                    let stream = match acceptor.accept(stream) {
                        Ok(stream) => stream,
                        Err(err) => {
                            eprintln!("warning: tls connection failed: {err}");
                            terminated(&termination_sender);
                            return;
                        }
                    };
                    let mut websocket = match accept(stream) {
                        Ok(websocket) => websocket,
                        Err(err) => {
                            eprintln!("warning: websocket connection failed: {err}");
                            terminated(&termination_sender);
                            return;
                        }
                    };

                    // read login packet - expect a username and a password
                    match recv(&mut websocket) {
                        Ok(login) => {
                            let parts: Vec<&str> = login.split('\n').collect();
                            if parts.len() != 2 {
                                try_close(
                                    websocket,
                                    Some(CloseFrame {
                                        code: CloseCode::Protocol,
                                        reason: std::borrow::Cow::Borrowed(
                                            "invalid login packet format",
                                        ),
                                    }),
                                );
                                eprintln!(
                                    "info: connection rejected - invalid login packet format"
                                );
                                terminated(&termination_sender);
                                return;
                            }

                            if parts[0] != password {
                                try_send(&mut websocket, "incorrect password".to_owned());
                                try_close(websocket, None);
                                eprintln!("info: connection rejected - incorrect password");
                                terminated(&termination_sender);
                                return;
                            }

                            // if logged in successfully
                            let username = parts[1];

                            // send assigned player id
                            let mut game_state_locked =
                                game_state.lock().expect("workers should not panic");
                            let assigned = game_state_locked.game_state.assign_player(username);
                            drop(game_state_locked);
                            match assigned {
                                Some(player) => {
                                    if let Err(message) =
                                        send_message(&mut websocket, format!("ok\n{player}"))
                                    {
                                        eprintln!("warning: connection interrupted: {message}");
                                        terminated(&termination_sender);
                                    }

                                    // while game isn't over
                                    loop {
                                        // send game state
                                        let game_state_locked =
                                            game_state.lock().expect("workers should not panic");

                                        let serialized_state = game_state_locked
                                            .game_state
                                            .serialize_for_player(player);

                                        drop(game_state_locked);

                                        if let Err(message) =
                                            send_message(&mut websocket, (&serialized_state).into())
                                        {
                                            eprintln!("warning: connection interrupted: {message}");
                                            terminated(&termination_sender);
                                        }

                                        if serialized_state.is_terminal() {
                                            break;
                                        }

                                        // get orders
                                        match recv(&mut websocket) {
                                            Ok(player_orders) => {
                                                match parse_orders(&player_orders) {
                                                    Ok(player_orders) => {
                                                        let mut game_state_locked = game_state
                                                            .lock()
                                                            .expect("workers should not panic");
                                                        game_state_locked
                                                            .orders
                                                            .insert(player, player_orders);

                                                        // maybe update game state
                                                        if game_state_locked.orders.len()
                                                            == num_players as usize
                                                        {
                                                            debug_assert!(
                                                                orders_semaphore.get().expect(
                                                                    "workers should not panic"
                                                                ) == 0
                                                            );
                                                            let orders =
                                                                take(&mut game_state_locked.orders);
                                                            game_state_locked
                                                                .game_state
                                                                .process_orders(&orders);
                                                            game_state_locked
                                                                .game_state
                                                                .save_to_file(&filename);
                                                            orders_semaphore
                                                                .up_n(num_players as u64)
                                                                .expect("workers should not panic");
                                                        }

                                                        drop(game_state_locked);

                                                        // wait for updated game state
                                                        orders_semaphore
                                                            .down()
                                                            .expect("workers should not panic");
                                                    }
                                                    Err(message) => {
                                                        try_close(
                                                            websocket,
                                                            Some(CloseFrame {
                                                                code: CloseCode::Protocol,
                                                                reason: std::borrow::Cow::Borrowed(
                                                                    message,
                                                                ),
                                                            }),
                                                        );
                                                        eprintln!("warning: could not parse orders: {message}");
                                                        terminated(&termination_sender);
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(message) => {
                                                eprintln!(
                                                    "warning: connection interrupted: {message}"
                                                );
                                                terminated(&termination_sender);
                                                return;
                                            }
                                        }
                                    }
                                }
                                None => {
                                    try_send(&mut websocket, "game full".to_owned());
                                    try_close(websocket, None);
                                    eprintln!("info: connection rejected - game full");
                                    terminated(&termination_sender);
                                    return;
                                }
                            }
                        }
                        Err(message) => {
                            eprintln!("warning: connection interrupted: {message}");
                            terminated(&termination_sender);
                            return;
                        }
                    }

                    termination_sender
                        .send(Ok(()))
                        .expect("main thread should always outlive this thread up to this point");
                });
                num_threads += 1;
            }
            Err(err) => {
                eprintln!("info: got invalid connection: {err}");
            }
        }

        // if we have num_players threads, wait until one is done
        if num_threads == num_players {
            num_threads -= 1;

            // if it joined after sending a terminal state, wait for the rest and break
            if termination_receiver
                .recv()
                .expect("original sender should never be dropped")
                .is_ok()
            {
                for _ in 0..num_threads {
                    let _ = termination_receiver
                        .recv()
                        .expect("original sender should never be dropped");
                }
                break 'acceptor;
            }
        }
    }

    ExitCode::SUCCESS
}
