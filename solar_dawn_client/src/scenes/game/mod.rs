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
use solar_dawn_common::{
    GameState, GameStateDelta, PlayerId, Vec2, celestial::CelestialId, order::Order, stack::StackId,
};
use web_sys::window;

use crate::{
    ClientState,
    scenes::game::{map::Map, sidebar::Sidebar},
    websocket::{Message, WebsocketClient},
};

mod map;
mod sidebar;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayHostility {
    Own,
    Friendly,
    Neutral,
    Hostile,
}

impl DisplayHostility {
    fn display_colour(&self) -> &'static str {
        // value from APP-6D
        match self {
            DisplayHostility::Own => "#80e0ff",
            DisplayHostility::Friendly => "#aaffaa",
            DisplayHostility::Neutral => "#ffff80",
            DisplayHostility::Hostile => "#ff8080",
        }
    }
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

struct ClickBroker {
    listeners: Vec<Box<dyn FnOnce(Vec2<i32>)>>,
    default_listener: Box<dyn FnMut(Vec2<i32>)>,
}
impl ClickBroker {
    fn new(default_listener: Box<dyn FnMut(Vec2<i32>)>) -> Self {
        Self {
            listeners: vec![],
            default_listener,
        }
    }

    fn click(&mut self, location: Vec2<i32>) {
        if let Some(listener) = self.listeners.pop() {
            listener(location);
        } else {
            (self.default_listener)(location);
        }
    }

    #[expect(dead_code)]
    fn register(&mut self, action: Box<dyn FnOnce(Vec2<i32>)>) {
        self.listeners.push(action);
    }
}

#[derive(Store)]
enum SidebarState {
    Outliner(OutlinerState),
    CelestialDetails(CelestialId),
    StackDetails(StackId),
    Disambiguate(Option<CelestialId>, Vec<StackId>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OutlinerState {
    Overview,
    Orders,
    Settings,
}

#[component]
pub fn InGame(
    me: PlayerId,
    websocket: WebsocketClient,
    game_state: WriteSignal<GameState>,
    change_state: EventHandler<ClientState>,
) -> Element {
    let client_view_settings = use_signal(ClientViewSettings::default);

    let mut orders = use_signal(Vec::<Order>::new);
    let mut auto_orders = use_signal(Vec::<(Order, bool)>::new);
    let mut submitting_orders = use_signal(|| false);

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

    let mut sidebar_state = use_store(|| SidebarState::Outliner(OutlinerState::Overview));

    let mut click_broker = use_signal(|| {
        ClickBroker::new(Box::new(move |location| {
            let game_state = game_state.read();
            let selected_celestial = game_state
                .celestials
                .get_by_position(location)
                .map(|(id, _)| id);
            let mut selected_stacks = game_state
                .stacks
                .iter()
                .filter_map(|(&id, stack)| {
                    if stack.position == location {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            selected_stacks.sort();
            match (selected_celestial, selected_stacks.len()) {
                (None, 0) => {
                    if !matches!(&*sidebar_state.read(), SidebarState::Outliner(_)) {
                        sidebar_state.set(SidebarState::Outliner(OutlinerState::Overview));
                    }
                }
                (Some(selected), 0) => {
                    sidebar_state.set(SidebarState::CelestialDetails(selected));
                }
                (None, 1) => {
                    sidebar_state.set(SidebarState::StackDetails(selected_stacks[0]));
                }
                _ => {
                    sidebar_state.set(SidebarState::Disambiguate(
                        selected_celestial,
                        selected_stacks,
                    ));
                }
            }
        }))
    });

    let candidate_orders = use_memo(move || {
        let mut candidate_orders = orders.read().clone();
        candidate_orders.extend(auto_orders.read().iter().filter_map(|(order, enabled)| {
            if *enabled { Some(order.clone()) } else { None }
        }));
        candidate_orders
    });

    let order_errors = use_memo(move || {
        let game_state = &*game_state.read();
        let mut to_validate = game_state
            .players
            .keys()
            .map(|&id| (id, vec![]))
            .collect::<HashMap<PlayerId, Vec<Order>>>();
        to_validate
            .get_mut(&me)
            .unwrap()
            .extend(candidate_orders.read().iter().cloned());
        Order::validate(game_state, &to_validate)
            .1
            .remove(&me)
            .unwrap()
    });

    // New state updates
    spawn({
        let mut websocket = websocket.clone();
        async move {
            match websocket.next().await {
                Some(Ok(Message::Binary(bytes))) => {
                    trace!("Starting to parse delta");
                    let Ok(delta) = from_slice::<GameStateDelta>(&bytes) else {
                        trace!("Parse of delta failed");
                        change_state(ClientState::Error(
                            "Protocol error: bad state message: couldn't parse".to_owned(),
                        ));
                        return;
                    };
                    trace!(delta = ?delta);
                    orders.clear();
                    auto_orders.clear();
                    submitting_orders.set(false);

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
        }
    });
    // subscribe to signal to ensure next state listener respawns
    let _ = game_state.read();

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
                        select: move |location| {
                            click_broker.write().click(location);
                        },
                    }
                }
                div { class: "col-3 h-100 overflow-y-auto border-start border-black",
                    Sidebar {
                        me,
                        state: sidebar_state,
                        game_state,
                        orders,
                        auto_orders,
                        candidate_orders,
                        order_errors,
                        submitting_orders,
                        client_game_settings,
                        client_view_settings,
                        change_state: move |new_state| {
                            sidebar_state.set(new_state);
                        },
                        websocket,
                    }
                }
            }
        }
    }
}
