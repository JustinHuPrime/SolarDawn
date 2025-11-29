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
use serde_cbor::to_vec;
use solar_dawn_common::{
    GameState, PlayerId,
    celestial::CelestialId,
    order::{Order, OrderError},
    stack::StackId,
};

use crate::{
    scenes::game::{
        ClientGameSettings, OutlinerState, SidebarState, SidebarStateStoreExt,
        SidebarStateStoreTransposed,
    },
    websocket::{Message, WebsocketClient},
};

#[component]
pub fn Sidebar(
    me: PlayerId,
    state: Store<SidebarState>,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    candidate_orders: ReadSignal<Vec<Order>>,
    order_errors: ReadSignal<Vec<Option<OrderError>>>,
    submitting_orders: Signal<bool>,
    client_game_settings: WriteSignal<ClientGameSettings>,
    change_state: EventHandler<SidebarState>,
    websocket: WebsocketClient,
) -> Element {
    match state.transpose() {
        SidebarStateStoreTransposed::Outliner(outliner_state) => {
            rsx! {
                Outliner {
                    me,
                    state: *outliner_state.read(),
                    game_state,
                    orders,
                    auto_orders,
                    candidate_orders,
                    order_errors,
                    submitting_orders,
                    client_game_settings,
                    change_state,
                    websocket,
                }
            }
        }
        SidebarStateStoreTransposed::CelestialDetails(celestial_id) => {
            rsx! {
                CelestialDetails {
                    id: *celestial_id.read(),
                    game_state,
                    change_state,
                }
            }
        }
        SidebarStateStoreTransposed::StackDetails(stack_id) => {
            rsx! {
                StackDetails {
                    me,
                    id: *stack_id.read(),
                    game_state,
                    orders,
                    auto_orders,
                    change_state,
                }
            }
        }
        SidebarStateStoreTransposed::Disambiguate(celestial_id, stack_ids) => {
            rsx! {
                Disambiguate {
                    celestial_id: *celestial_id.read(),
                    stack_ids: stack_ids.read().clone(),
                    game_state,
                    change_state,
                }
            }
        }
    }
}

#[component]
pub fn Outliner(
    me: PlayerId,
    state: OutlinerState,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    candidate_orders: ReadSignal<Vec<Order>>,
    order_errors: ReadSignal<Vec<Option<OrderError>>>,
    submitting_orders: Signal<bool>,
    client_game_settings: WriteSignal<ClientGameSettings>,
    change_state: EventHandler<SidebarState>,
    websocket: WebsocketClient,
) -> Element {
    rsx! {
        h1 { "Turn {game_state.read().turn} {game_state.read().phase}" }
        div { class: "btn-group", role: "toolbar",
            button {
                r#type: "button",
                class: "btn btn-primary",
                class: if matches!(state, OutlinerState::Overview) { "active" },
                onclick: move |_| {
                    change_state(SidebarState::Outliner(OutlinerState::Overview));
                },
                "Overview"
            }
            button {
                r#type: "button",
                class: "btn btn-primary",
                class: if matches!(state, OutlinerState::Orders) { "active" },
                onclick: move |_| {
                    change_state(SidebarState::Outliner(OutlinerState::Orders));
                },
                "Orders"
            }
            button {
                r#type: "button",
                class: "btn btn-primary",
                class: if matches!(state, OutlinerState::Settings) { "active" },
                onclick: move |_| {
                    change_state(SidebarState::Outliner(OutlinerState::Settings));
                },
                "Settings"
            }
        }
        match state {
            OutlinerState::Overview => {
                rsx! {
                    OutlinerOverview { me, game_state, change_state }
                }
            }
            OutlinerState::Orders => {
                rsx! {
                    OutlinerOrders {
                        me,
                        game_state,
                        orders,
                        auto_orders,
                        change_state,
                    }
                }
            }
            OutlinerState::Settings => {
                rsx! {
                    OutlinerSettings { me, game_state, client_game_settings }
                }
            }
        }
        button {
            r#type: "button",
            class: "btn btn-lg btn-primary",
            disabled: *submitting_orders.read() || !order_errors.read().is_empty(),
            onclick: move |_| {
                submitting_orders.set(true);
                websocket
                    .send(
                        Message::Binary(
                            to_vec(&*candidate_orders.read())
                                .expect("orders should always be serializable")
                                .into_boxed_slice(),
                        ),
                    );
            },
            "End Turn"
        }
    }
}

#[component]
pub fn OutlinerOverview(
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    rsx! {
        h2 { "Celestial Bodies" }
        // for &celestial_id in game_state.read().celestials.majors() {
        //     {
        //         let celestial = game_state.read().celestials.get(celestial_id).unwrap();
        //     }
        // }
        h2 { "Your Ships" }
        h2 { "Other Ships" }
    }
}

#[component]
pub fn OutlinerOrders(
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    rsx! {
        h2 { "Orders" }
        h2 { "Automatic Orders" }
    }
}

#[component]
pub fn OutlinerSettings(
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    client_game_settings: WriteSignal<ClientGameSettings>,
) -> Element {
    rsx! {
        h2 { "Display Settings" }
    }
}

#[component]
pub fn CelestialDetails(
    id: CelestialId,
    game_state: ReadSignal<GameState>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    rsx! {}
}

#[component]
pub fn StackDetails(
    me: PlayerId,
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    rsx! {}
}

#[component]
pub fn Disambiguate(
    celestial_id: Option<CelestialId>,
    stack_ids: Vec<StackId>,
    game_state: ReadSignal<GameState>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    rsx! {}
}
