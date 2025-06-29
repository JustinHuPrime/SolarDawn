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

use std::{
    env::args_os,
    ffi::OsStr,
    net::SocketAddr,
    path::Path,
    process::ExitCode,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::{
        ConnectInfo, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use tower_http::services::ServeDir;

mod model;

enum ServerState {
    New,
    Load,
    Running,
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

    let server_state = Arc::new(Mutex::new(match args.as_slice() {
        ["new"] => ServerState::New,
        ["load"] => ServerState::Load,
        _ => {
            eprintln!("Couldn't parse command line '{}'", args.join(" "));
            eprintln!("Usage:");
            eprintln!("\t{executable} load");
            eprintln!("\t{executable} new");
            return ExitCode::FAILURE;
        }
    }));

    let app = Router::new()
        .fallback_service(
            ServeDir::new(folder.join("assets")).append_index_html_on_directories(true),
        )
        .route("/ws", any(ws_handler))
        .with_state(server_state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:80").await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Couldn't start webserver: {e}");
            return ExitCode::FAILURE;
        }
    };
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    ExitCode::SUCCESS
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    _: Option<TypedHeader<headers::UserAgent>>,
    _: ConnectInfo<SocketAddr>,
    State(server_state): State<Arc<Mutex<ServerState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, server_state))
}

async fn handle_socket(mut socket: WebSocket, server_state: Arc<Mutex<ServerState>>) {
    // handle login
    let Some(msg) = socket.recv().await else {
        // stream closed
        return;
    };
    match msg {
        Ok(message) => {}
        Err(e) => {}
    }
}
