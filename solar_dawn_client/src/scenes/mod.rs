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

mod game;

use dioxus::prelude::*;
use serde_cbor::from_slice;
use solar_dawn_common::{GameState, PlayerId};
use ws_queue_web::{Message, WebSocketClient};

use crate::{ClientState, WEBSOCKET};

pub use game::InGame;

#[component]
pub fn Error(message: String) -> Element {
    rsx! {
        div { class: "container",
            h1 { "Something Went Wrong" }
            p { "{message}" }
            p {
                "To try again "
                a { href: "/", class: "btn btn-primary", "refresh the page" }
            }
        }
    }
}

#[component]
pub fn Join(state: Signal<ClientState>) -> Element {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let mut username = use_signal(String::new);
    let mut join_code = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "container",
            div { class: "row",
                div { class: "col-12",
                    h1 { "Solar Dawn version {VERSION}" }
                }
            }
            div { class: "row",
                div { class: "col-lg-1",
                    label { r#for: "username", class: "form-label col-form-label", "Username" }
                }
                div { class: "col-lg-5",
                    input {
                        r#type: "text",
                        id: "username",
                        class: "form-control",
                        oninput: move |e| username.set(e.value()),
                        ""
                    }
                }
            }
            div { class: "row mb-3",
                label {
                    r#for: "join-code",
                    class: "form-label col-lg-1 col-form-label",
                    "Join Code"
                }
                div { class: "col-lg-5",
                    input {
                        r#type: "password",
                        id: "join-code",
                        class: "form-control",
                        oninput: move |e| join_code.set(e.value()),
                        ""
                    }
                }
            }
            if let Some(ref error) = *error_message.read() {
                div { class: "row",
                    p { class: "text-danger", "{error}" }
                }
            }
            div { class: "row",
                div { class: "col-12",
                    button {
                        class: "btn btn-primary",
                        r#type: "submit",
                        onclick: move |_| {
                            submitting.set(true);
                            let mut client = match WebSocketClient::new(
                                "/ws",
                                Some(Message::Text(format!("{}\n{}", username.read(), join_code.read()))),
                            ) {
                                Ok(client) => client,
                                Err(err) => {
                                    error_message
                                        .set(Some(format!("Couldn't connect to server: {:#?}", err)));
                                    return;
                                }
                            };
                            client
                                .set_onmessage(
                                    Some(
                                        Box::new(move |message| {
                                            match message {
                                                Message::Text(message) => {
                                                    match message.as_str() {
                                                        "bad join code" => {
                                                            error_message.set(Some("Incorrect join code".to_string()));
                                                            *submitting.write() = false;
                                                            *WEBSOCKET.write() = None;
                                                        }
                                                        "game full" => {
                                                            error_message.set(Some("No open seats".to_string()));
                                                            *submitting.write() = false;
                                                            *WEBSOCKET.write() = None;
                                                        }
                                                        "user already connected" => {
                                                            error_message
                                                                .set(Some("Username already connected".to_string()));
                                                            *submitting.write() = false;
                                                            *WEBSOCKET.write() = None;
                                                        }
                                                        _ => protocol_error(state),
                                                    }
                                                }
                                                Message::Binary(bytes) => {
                                                    let Ok(player_id) = from_slice::<PlayerId>(&bytes) else {
                                                        protocol_error(state);
                                                        return;
                                                    };
                                                    state.set(ClientState::WaitingForPlayers(player_id));
                                                    WEBSOCKET
                                                        .write()
                                                        .as_mut()
                                                        .expect("got message from socket")
                                                        .set_onmessage(None);
                                                }
                                            }
                                        }),
                                    ),
                                );
                            client
                                .set_onerror(
                                    Some(
                                        Box::new(move |err| {
                                            state
                                                .set(
                                                    ClientState::Error(format!("Connection lost\n{err:#?}")),
                                                )
                                        }),
                                    ),
                                );
                            *WEBSOCKET.write() = Some(client);
                        },
                        disabled: *submitting.read(),
                        "Join Game"
                    }
                }
            }
            div { class: "row",
                div { class: "col-12",
                    hr {}
                }
            }
            div { class: "row",
                div { class: "col-12",
                    p {
                        a { href: asset!("assets/guide.html"), "Read the guide" }
                    }
                }
            }
            div { class: "row",
                div { class: "col-12",
                    p {
                        "Solar Dawn is free software licenced under the "
                        a { href: "https://www.gnu.org/licenses/agpl.html",
                            "GNU Affero General Public License"
                        }
                        br {}
                        a { href: "https://github.com/JustinHuPrime/SolarDawn",
                            "View the source code here"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn WaitingForPlayers(state: Signal<ClientState>) -> Element {
    let ClientState::WaitingForPlayers(me) = *state.read() else {
        unreachable!();
    };

    WEBSOCKET
        .write()
        .as_mut()
        .expect("state transition guarded")
        .set_onmessage(Some(Box::new(move |message| {
            let Message::Binary(bytes) = message else {
                protocol_error(state);
                return;
            };
            let Ok(game_state) = from_slice::<GameState>(&bytes) else {
                protocol_error(state);
                return;
            };
            WEBSOCKET
                .write()
                .as_mut()
                .expect("state transition guarded")
                .set_onmessage(None);
            state.set(ClientState::InGame(game_state, me));
        })));
    rsx! {
        div { class: "container",
            h1 { "Waiting for players..." }
        }
    }
}

pub fn protocol_error(mut state: Signal<ClientState>) {
    state.set(ClientState::Error(
        "Connection lost: protocol error".to_string(),
    ));
    *WEBSOCKET.write() = None;
}
