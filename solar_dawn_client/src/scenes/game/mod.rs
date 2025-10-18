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

use dioxus::prelude::*;
use serde_cbor::from_slice;
use solar_dawn_common::GameStateDelta;
use ws_queue_web::Message;

use crate::{ClientState, WEBSOCKET, scenes::protocol_error};

struct ClientGameSettings {}
struct ClientViewSettings {
    x_offset: f64,
    y_offset: f64,
}

#[component]
pub fn InGame(state: Signal<ClientState>) -> Element {
    let ClientState::InGame(game_state, me) = &*state.read() else {
        unreachable!()
    };

    let mut game_state = use_signal(|| game_state.clone());
    let me = *me;

    WEBSOCKET
        .write()
        .as_mut()
        .expect("state transition guarded")
        .set_onmessage(Some(Box::new({
            move |message| {
                let Message::Binary(bytes) = message else {
                    protocol_error(state);
                    return;
                };
                let Ok(delta) = from_slice::<GameStateDelta>(&bytes) else {
                    protocol_error(state);
                    return;
                };
                game_state.write().apply(delta);
            }
        })));

    rsx! {
        div { class: "container-fluid",
            h1 { "Game scene" }
            p { {format!("{:#?}", game_state.read())} }
            button {
                onclick: {
                    move |_| {
                        game_state.write().turn += 1;
                    }
                },
                "Test!"
            }
        }
    }
}
