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

//! Client for Solar Dawn

mod scenes;

use dioxus::prelude::*;
use solar_dawn_common::{GameState, PlayerId};
use ws_queue_web::WebSocketClient;

use crate::scenes::*;

static WEBSOCKET: GlobalSignal<Option<WebSocketClient>> = Global::new(|| None);

enum ClientState {
    Error(String),
    Login,
    WaitingForPlayers(PlayerId),
    InGame(GameState, PlayerId),
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let state = use_signal(|| ClientState::Login);

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/css/bootstrap.min.css",
            integrity: "sha384-sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB",
            crossorigin: "anonymous",
        }
        document::Script {
            src: "https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/js/bootstrap.bundle.min.js",
            integrity: "sha384-FKyoEForCGlyvwx9Hj09JcYn3nv7wiPVlz7YYwJrWVcXK/BmnVDxM+D2scQbITxI",
            crossorigin: "anonymous",
        }
        document::Link { rel: "stylesheet", href: asset!("/assets/main.css") }
        document::Link {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: asset!("/assets/apple-touch-icon.png"),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "32x32",
            href: asset!("/assets/favicon-32x32.png"),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "16x16",
            href: asset!("/assets/favicon-16x16.png"),
        }
        document::Link {
            rel: "manifest",
            href: {
                let _ = asset!("/assets/android-chrome-192x192.png");
                let _ = asset!("/assets/android-chrome-512x512.png");
                asset!("/assets/site.webmanifest")
            },
        }
        match &*state.read() {
            ClientState::Error(message) => {
                rsx! {
                    Error { message }
                }
            }
            ClientState::Login => {
                rsx! {
                    Join { state }
                }
            }
            ClientState::WaitingForPlayers(..) => {
                rsx! {
                    WaitingForPlayers { state }
                }
            }
            ClientState::InGame(..) => {
                rsx! {
                    InGame { state }
                }
            }
        }
    }
}
