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
//!
//! Is a dioxus app

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use dioxus::{logger::tracing::Level, prelude::*};
use solar_dawn_common::{GameState, PlayerId};

use crate::{
    scenes::{Error, Join, WaitingForPlayers, game::InGame},
    websocket::WebsocketClient,
};

mod event_listener;
mod scenes;
mod websocket;

fn main() {
    if cfg!(debug_assertions) {
        dioxus::logger::init(Level::TRACE).unwrap();
    } else {
        dioxus::logger::init(Level::INFO).unwrap();
    }
    dioxus::launch(App);
}

#[derive(Store)]
enum ClientState {
    Error(String),
    Join,
    WaitingForPlayers {
        me: PlayerId,
        websocket: WebsocketClient,
    },
    InGame {
        me: PlayerId,
        websocket: WebsocketClient,
        game_state: GameState,
    },
}

#[component]
fn App() -> Element {
    let mut state = use_store(|| ClientState::Join);

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
        document::Link {
            rel: "stylesheet",
            href: asset!("/assets/main.css", AssetOptions::builder().with_hash_suffix(false)),
        }
        document::Link {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: asset!(
                "/assets/apple-touch-icon.png", AssetOptions::builder().with_hash_suffix(false)
            ),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "32x32",
            href: asset!("/assets/favicon-32x32.png", AssetOptions::builder().with_hash_suffix(false)),
        }
        document::Link {
            rel: "icon",
            r#type: "image/png",
            sizes: "16x16",
            href: asset!("/assets/favicon-16x16.png", AssetOptions::builder().with_hash_suffix(false)),
        }
        document::Link {
            rel: "manifest",
            href: {
                #[used]
                static _ANDROID_192: Asset = asset!(
                    "/assets/android-chrome-192x192.png", AssetOptions::builder()
                    .with_hash_suffix(false)
                );
                #[used]
                static _ANDROID_512: Asset = asset!(
                    "/assets/android-chrome-512x512.png", AssetOptions::builder()
                    .with_hash_suffix(false)
                );
                asset!(
                    "/assets/site.webmanifest", AssetOptions::builder().with_hash_suffix(false)
                )
            },
        }
        match state.transpose() {
            ClientStateStoreTransposed::Error(message) => {
                rsx! {
                    Error { message }
                }
            }
            ClientStateStoreTransposed::Join => {
                rsx! {
                    Join {
                        change_state: move |new_state| {
                            state.set(new_state);
                        },
                    }
                }
            }
            ClientStateStoreTransposed::WaitingForPlayers { me, websocket } => {
                rsx! {
                    WaitingForPlayers {
                        me: *me.read(),
                        websocket: websocket.read().clone(),
                        change_state: move |new_state| {
                            state.set(new_state);
                        },
                    }
                }
            }
            ClientStateStoreTransposed::InGame { me, websocket, game_state } => {
                rsx! {
                    InGame {
                        me: *me.read(),
                        websocket: websocket.read().clone(),
                        game_state,
                        change_state: move |new_state| {
                            state.set(new_state);
                        },
                    }
                }
            }
        }
    }
}
