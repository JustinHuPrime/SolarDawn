// Copyright 2025 Justin Hu
//
// This file is part of Solar Dawn.
//
// Solar Dawn is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// Solar Dawn is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Solar Dawn. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Server binary for Solar Dawn
//!
//! Expects to be run in a directory where:
//!
//! - ./assets/index.html is an appropriate HTML file to run the client in
//! - ./assets/pkg/* is the client WASM pkg
//! - ./cert.pem is a TLS certificate
//! - ./key.pem is the private key for the certificate

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{
    collections::HashMap,
    env::args_os,
    ffi::OsStr,
    fs::OpenOptions,
    io::Write,
    mem::{replace, take},
    net::SocketAddr,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, bail};
use axum::{
    Router,
    extract::{
        ConnectInfo, State,
        ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use axum_server::tls_rustls::RustlsConfig;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use rand::{Rng, rng};
use rand_distr::Alphanumeric;
use rand_pcg::Pcg64;
use serde::Serialize;
use serde_cbor::{from_slice, to_vec};
use solar_dawn_common::{GameState, GameStateInitializer, Phase, PlayerId};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

use crate::model::{GameServerState, IdGenerator};

mod model;

enum ServerState {
    New {
        join_code: String,
        num_players: usize,
        connections: HashMap<PlayerId, (String, SplitSink<WebSocket, Message>)>,
        player_id_generator: IdGenerator<PlayerId, u8>,
        scenario: GameStateInitializer,
        save_file: PathBuf,
    },
    Load {
        join_code: String,
        connections: HashMap<PlayerId, SplitSink<WebSocket, Message>>,
        game_state: GameServerState,
        save_file: PathBuf,
    },
    Running {
        join_code: String,
        connections: HashMap<PlayerId, SplitSink<WebSocket, Message>>,
        game_state: GameServerState,
        save_file: PathBuf,
    },
}
impl ServerState {
    /// Create a new server state for a new game
    fn new(num_players: usize, scenario: &str, save_file: PathBuf) -> Result<Self> {
        if !(1..=6).contains(&num_players) {
            bail!("can't start game with {} players", num_players);
        }

        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&save_file)
            .with_context(|| format!("while opening {}", save_file.to_string_lossy()))?;

        let scenario =
            GameState::new(scenario).map_err(|_| anyhow!("unknown scenario {scenario}"))?;

        Ok(Self::New {
            join_code: Self::join_code(),
            num_players,
            connections: HashMap::new(),
            player_id_generator: IdGenerator::default(),
            scenario,
            save_file,
        })
    }

    /// Create a new server state for a loaded game
    fn load(save_file: PathBuf) -> Result<Self> {
        Ok(Self::Load {
            join_code: Self::join_code(),
            connections: HashMap::new(),
            game_state: GameServerState::from_path(&save_file)?,
            save_file,
        })
    }

    fn lost_connection(&mut self, player_id: PlayerId) {
        match self {
            ServerState::Running { connections, .. } => {
                connections.remove(&player_id);
            }
            _ => panic!("tried to disconnect from an already-stopped server"),
        }
    }

    async fn server_disconnect(&mut self, player_id: PlayerId, message: Message) {
        match self {
            ServerState::Running { connections, .. } => {
                let _ = connections
                    .get_mut(&player_id)
                    .expect("should only drop connected players")
                    .send(message)
                    .await;
            }
            _ => panic!("tried to drop client from an already-stopped server"),
        }
        self.lost_connection(player_id);
    }

    async fn start(&mut self) {
        // change state
        match self {
            ServerState::New {
                join_code,
                connections,
                scenario,
                save_file,
                ..
            } => {
                let players = connections
                    .iter_mut()
                    .map(|(&player_id, (username, _))| (player_id, take(username)))
                    .collect();
                let connections = take(connections)
                    .into_iter()
                    .map(|(player_id, (_, send))| (player_id, send))
                    .collect();
                let game_state = GameServerState::new(players, *scenario);
                *self = ServerState::Running {
                    join_code: take(join_code),
                    connections,
                    game_state,
                    save_file: take(save_file),
                }
            }
            ServerState::Load {
                join_code,
                connections,
                game_state,
                save_file,
            } => {
                *self = ServerState::Running {
                    join_code: take(join_code),
                    connections: take(connections),
                    game_state: replace(game_state, Self::barely_init_game_server_state()),
                    save_file: take(save_file),
                }
            }
            ServerState::Running { .. } => panic!("tried to start already-running server"),
        }

        // send current game state to all players
        match self {
            ServerState::Running {
                connections,
                game_state,
                ..
            } => {
                let mut lost_connections = Vec::new();
                for (&player_id, connection) in connections.iter_mut() {
                    let message = Message::Binary(
                        to_vec(&game_state.game_state)
                            .expect("game state should always be serializable")
                            .into(),
                    );
                    if connection.send(message).await.is_err() {
                        lost_connections.push(player_id);
                    };
                }
                for player_id in lost_connections {
                    self.lost_connection(player_id);
                }
            }
            _ => {
                unreachable!("variant changed above");
            }
        }

        self.save();
    }

    async fn next(&mut self) {
        match self {
            ServerState::Running {
                connections,
                game_state,
                ..
            } => {
                let delta = game_state.game_state.next(
                    take(&mut game_state.orders),
                    &mut game_state.stack_id_generator,
                    &mut game_state.module_id_generator,
                    &mut game_state.rng,
                );
                let delta_bytes =
                    to_vec(&delta).expect("game state delta should always be serializable");
                game_state.game_state.apply(delta);

                let mut lost_connections = Vec::new();
                for (&player_id, connection) in connections.iter_mut() {
                    let message = Message::Binary(delta_bytes.clone().into());
                    if connection.send(message).await.is_err() {
                        lost_connections.push(player_id);
                    };
                }
                for player_id in lost_connections {
                    self.lost_connection(player_id);
                }
            }
            _ => panic!("tried to go to next phase on stopped server"),
        }
        self.save();
    }

    fn save(&self) {
        match self {
            ServerState::Running {
                game_state,
                save_file,
                ..
            } => {
                let Ok(mut file) = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(save_file)
                else {
                    eprintln!("couldn't save to {}", save_file.to_string_lossy());
                    return;
                };
                let Ok(()) = file.write_all(
                    &to_vec(game_state).expect("game state should always be serializable"),
                ) else {
                    eprintln!("couldn't save to {}", save_file.to_string_lossy());
                    return;
                };
            }
            _ => panic!("tried to save while not running"),
        }
    }

    fn join_code() -> String {
        (0..16)
            .map(|_| rng().sample(Alphanumeric) as char)
            .collect()
    }

    fn barely_init_game_server_state() -> GameServerState {
        GameServerState {
            game_state: GameState {
                phase: Phase::Logistics,
                players: Default::default(),
                celestials: Default::default(),
                earth: 0.into(),
                stacks: Default::default(),
            },
            orders: Default::default(),
            celestial_id_generator: Default::default(),
            stack_id_generator: Default::default(),
            module_id_generator: Default::default(),
            rng: Pcg64::new(0, 0),
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let Ok(args) = args_os()
        .map(|arg| arg.into_string())
        .collect::<Result<Vec<_>, _>>()
    else {
        eprintln!("Command line arguments had invalid characters");
        return ExitCode::FAILURE;
    };
    let executable = args
        .first()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .unwrap_or(env!("CARGO_BIN_NAME"));
    let folder = args
        .first()
        .map(Path::new)
        .and_then(Path::parent)
        .unwrap_or(Path::new(""));
    let args = args.iter().skip(1).map(String::as_str).collect::<Vec<_>>();

    let server_state = Arc::new(Mutex::new(match *args.as_slice() {
        ["new", num_players, scenario, save_file] => {
            let num_players = match num_players.parse::<usize>() {
                Ok(num_players) => num_players,
                Err(_) => {
                    eprintln!("couldn't parse '{num_players}'");
                    return ExitCode::FAILURE;
                }
            };
            match ServerState::new(num_players, scenario, PathBuf::from(save_file)) {
                Ok(server_state) => server_state,
                Err(err) => {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }
            }
        }
        ["load", save_file] => match ServerState::load(PathBuf::from(save_file)) {
            Ok(server_state) => server_state,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("Couldn't parse command line '{}'", args.join(" "));
            eprintln!("Usage:");
            eprintln!("\t{executable} new <num_players> <scenario> <save_file>");
            eprintln!("\t{executable} load <save_file>");
            return ExitCode::FAILURE;
        }
    }));

    let app = Router::new()
        .fallback_service(
            ServeDir::new(folder.join("assets")).append_index_html_on_directories(true),
        )
        .route("/ws", any(ws_handler))
        .with_state(server_state);

    let Ok(config) = RustlsConfig::from_pem_file("./cert.pem", "./key.pem").await else {
        eprintln!("Couldn't set up TLS");
        return ExitCode::FAILURE;
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], 443));
    match axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        Ok(()) => {
            unreachable!("axum::serve should never return");
        }
        Err(err) => {
            eprintln!("Couldn't start webserver: {err}");
            return ExitCode::FAILURE;
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    _: Option<TypedHeader<headers::UserAgent>>,
    _: ConnectInfo<SocketAddr>,
    State(server_state): State<Arc<Mutex<ServerState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, server_state))
}

async fn handle_socket(socket: WebSocket, server_state_mutex: Arc<Mutex<ServerState>>) {
    fn protocol_error() -> Message {
        Message::Close(Some(CloseFrame {
            code: 4002,
            reason: "protocol error".into(),
        }))
    }
    fn join_code_error() -> Message {
        Message::Close(Some(CloseFrame {
            code: 4101,
            reason: "bad join code".into(),
        }))
    }
    fn serialize<T: Serialize>(message: &T) -> Message {
        Message::Binary(
            to_vec(&message)
                .expect("messages should always be serializable")
                .into(),
        )
    }

    let (mut send, mut recv) = socket.split();
    let Some(Ok(Message::Text(login))) = recv.next().await else {
        // protocol error
        let _ = send.send(protocol_error()).await;
        return;
    };
    let login = login.split('\n').collect::<Vec<_>>();
    let [attempt_join_code, attempt_username] = *login.as_slice() else {
        let _ = send.send(protocol_error()).await;
        return;
    };
    let mut server_state = server_state_mutex.lock().await;
    let player_id = match &mut *server_state {
        ServerState::New {
            join_code,
            num_players,
            connections,
            player_id_generator,
            ..
        } => {
            if join_code != attempt_join_code {
                // join code error
                drop(server_state);
                let _ = send.send(join_code_error()).await;
                return;
            }

            let player_id = player_id_generator.next().expect("should be infinite");

            // report login success
            let Ok(()) = send.send(serialize(&player_id)).await else {
                // couldn't reply - connection dropped?
                return;
            };

            // add player to connections (guaranteed to have a slot)
            connections.insert(player_id, (attempt_username.into(), send));

            // maybe promote server to running
            if *num_players == connections.len() {
                server_state.start().await;
            }

            player_id
        }
        ServerState::Load {
            join_code,
            connections,
            game_state,
            ..
        } => {
            if join_code != attempt_join_code {
                // join code error
                drop(server_state);
                let _ = send.send(join_code_error()).await;
                return;
            }

            let Some((&player_id, _)) = game_state
                .game_state
                .players
                .iter()
                .find(|&(_, username)| username == attempt_username)
            else {
                drop(server_state);
                let _ = send
                    .send(Message::Close(Some(CloseFrame {
                        code: 4102,
                        reason: "unknown username".into(),
                    })))
                    .await;
                return;
            };
            if connections.contains_key(&player_id) {
                // can't reconnect - someone already connected
                drop(server_state);
                let _ = send
                    .send(Message::Close(Some(CloseFrame {
                        code: 4103,
                        reason: "user already connected".into(),
                    })))
                    .await;
                return;
            }

            // report login success
            let Ok(()) = send.send(serialize(&player_id)).await else {
                // couldn't reply - connection dropped?
                return;
            };

            // add player to connections (guaranteed to have a slot)
            connections.insert(player_id, send);

            // maybe promote server to running
            if game_state.game_state.players.len() == connections.len() {
                server_state.start().await;
            }

            player_id
        }
        ServerState::Running {
            join_code,
            connections,
            game_state,
            ..
        } => {
            if join_code != attempt_join_code {
                // join code error
                drop(server_state);
                let _ = send.send(join_code_error()).await;
                return;
            }

            let Some((&player_id, _)) = game_state
                .game_state
                .players
                .iter()
                .find(|&(_, username)| username == attempt_username)
            else {
                drop(server_state);
                let _ = send
                    .send(Message::Close(Some(CloseFrame {
                        code: 4102,
                        reason: "unknown username".into(),
                    })))
                    .await;
                return;
            };
            if connections.contains_key(&player_id) {
                // can't reconnect - someone already connected
                drop(server_state);
                let _ = send
                    .send(Message::Close(Some(CloseFrame {
                        code: 4103,
                        reason: "user already connected".into(),
                    })))
                    .await;
                return;
            }

            // report login success
            let Ok(()) = send.send(serialize(&player_id)).await else {
                // couldn't reply - connection dropped?
                return;
            };

            // send this player specifically the game state
            let message = Message::Binary(
                to_vec(&game_state.game_state)
                    .expect("game state should always be serializable")
                    .into(),
            );
            let Ok(()) = send.send(message).await else {
                server_state.lost_connection(player_id);
                return;
            };

            // add player to connections (guaranteed to have a slot)
            connections.insert(player_id, send);

            player_id
        }
    };
    drop(server_state);

    // get orders from player
    loop {
        let Some(Ok(Message::Binary(orders))) = recv.next().await else {
            // protocol error
            let mut server_state = server_state_mutex.lock().await;
            server_state
                .server_disconnect(player_id, protocol_error())
                .await;
            return;
        };

        let Ok(parsed) = from_slice(&orders) else {
            // protocol error
            let mut server_state = server_state_mutex.lock().await;
            server_state
                .server_disconnect(player_id, protocol_error())
                .await;
            return;
        };
        let mut server_state = server_state_mutex.lock().await;
        match &mut *server_state {
            ServerState::Running { game_state, .. } => {
                if game_state.orders.contains_key(&player_id) {
                    // protocol error - client sent multiple orders
                    server_state
                        .server_disconnect(player_id, protocol_error())
                        .await;
                    return;
                }

                game_state.orders.insert(player_id, parsed);

                // maybe broadcast next orders
                if game_state.orders.len() == game_state.game_state.players.len() {
                    server_state.next().await;
                }
            }
            _ => {
                // protocol error - client sent packet too early
                server_state
                    .server_disconnect(player_id, protocol_error())
                    .await;
                return;
            }
        }
        drop(server_state);
    }
}
