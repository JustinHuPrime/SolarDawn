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

mod sidebar;

use dioxus::prelude::*;
use futures::StreamExt;
use serde_cbor::from_slice;
use solar_dawn_common::{GameState, GameStateDelta, PlayerId};

use crate::{
    ClientState,
    websocket::{Message, WebsocketClient},
};

#[component]
pub fn InGame(
    me: PlayerId,
    websocket: WebsocketClient,
    game_state: WriteSignal<GameState>,
    change_state: EventHandler<ClientState>,
) -> Element {
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
                    h1 { "Map" }
                }
                div { class: "col-3 h-100 overflow-y-auto",
                    h1 { "Sidebar" }
                }
            }
        }
    }
}
