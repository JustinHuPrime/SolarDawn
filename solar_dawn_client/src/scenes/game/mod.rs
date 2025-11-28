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

use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};
use dioxus::prelude::*;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_cbor::{from_slice, to_vec};
use solar_dawn_common::{GameState, GameStateDelta, PlayerId, order::Order};
use web_sys::window;

use crate::{
    ClientState,
    scenes::game::map::Map,
    websocket::{Message, WebsocketClient},
};

mod map;
mod sidebar;

#[derive(Serialize, Deserialize)]
enum DisplayHostility {
    Own,
    Friendly,
    Neutral,
    Hostile,
}

#[derive(Serialize, Deserialize)]
struct ClientGameSettings {
    display_hostility: HashMap<PlayerId, DisplayHostility>,
}
impl ClientGameSettings {
    fn new<'a>(me: PlayerId, players: impl Iterator<Item = &'a PlayerId>) -> Self {
        Self {
            display_hostility: players
                .map(|&player| {
                    if player == me {
                        (player, DisplayHostility::Own)
                    } else {
                        (player, DisplayHostility::Neutral)
                    }
                })
                .collect::<HashMap<_, _>>(),
        }
    }
}

struct ClientViewSettings {
    x_offset: f32,
    y_offset: f32,
    zoom_level: i32,
}
impl ClientViewSettings {
    fn zoom(&self) -> f32 {
        1.1_f32.powi(self.zoom_level)
    }
}
impl Default for ClientViewSettings {
    fn default() -> Self {
        Self {
            x_offset: 0.0,
            y_offset: 0.0,
            zoom_level: 0,
        }
    }
}

#[component]
pub fn InGame(
    me: PlayerId,
    websocket: WebsocketClient,
    game_state: WriteSignal<GameState>,
    change_state: EventHandler<ClientState>,
) -> Element {
    let client_view_settings = use_signal(ClientViewSettings::default);

    let orders = use_signal(Vec::<Order>::new);
    let auto_orders = use_signal(Vec::<(Order, bool)>::new);

    // Game settings (unit hostility display, etc)
    let client_game_settings: Signal<ClientGameSettings> = use_signal(|| {
        let storage = window().unwrap().local_storage().ok().flatten();
        let game_settings = storage
            .and_then(|storage| {
                storage
                    .get_item(&format!(
                        "solar_dawn:game_settings:{}:{}",
                        game_state.read().game_id,
                        me
                    ))
                    .ok()
            })
            .flatten();
        let parsed = game_settings
            .and_then(|game_settings| BASE64_STANDARD.decode(&game_settings).ok())
            .and_then(|game_settings| from_slice::<ClientGameSettings>(&game_settings).ok());
        parsed.unwrap_or_else(|| ClientGameSettings::new(me, game_state.read().players.keys()))
    });
    use_effect(move || {
        let stringified =
            to_vec(&*client_game_settings.read()).expect("should always be serializable");
        let storage = window().unwrap().local_storage().ok().flatten();
        if let Some(storage) = storage {
            let _ = storage.set(
                &format!(
                    "solar_dawn:game_settings:{}:{}",
                    game_state.read().game_id,
                    me
                ),
                &BASE64_STANDARD.encode(stringified),
            );
        }
    });

    // New state updates
    spawn(async move {
        match websocket.next().await {
            Some(Ok(Message::Binary(bytes))) => {
                let Ok(delta) = from_slice::<GameStateDelta>(&bytes) else {
                    change_state(ClientState::Error(
                        "Protocol error: bad state message: couldn't parse".to_owned(),
                    ));
                    return;
                };
                let mut game_state = game_state.write();
                *game_state = game_state.apply(delta);
            }
            Some(Ok(Message::Text(_))) => {
                change_state(ClientState::Error(
                    "Protocol error: bad state message: bad format".to_owned(),
                ));
            }
            Some(Err(error)) => {
                change_state(ClientState::Error(format!("Connection lost: {error}")));
            }
            None => {
                unreachable!("we always give up upon seeing a close or error event");
            }
        }
    });

    rsx! {
        div {
            class: "container-fluid p-0 overflow-x-hidden overflow-y-hidden",
            style: "width:100vw; height:100vh",
            div { class: "row h-100 m-0",
                div { class: "col-9 h-100 p-0",
                    Map {
                        game_state,
                        orders,
                        auto_orders,
                        client_game_settings,
                        client_view_settings,
                        change_state,
                    }
                }
                div { class: "col-3 h-100 overflow-y-auto border-start border-black",
                    h1 { "Sidebar" }
                }
            }
        }
    }
}
