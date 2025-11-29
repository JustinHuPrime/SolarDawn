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
    celestial::{CelestialId, Resources},
    order::{Order, OrderError},
    stack::StackId,
};

use crate::{
    scenes::game::{
        ClientGameSettings, ClientViewSettings, DisplayHostility, OutlinerState, SidebarState,
        SidebarStateStoreExt, SidebarStateStoreTransposed, map::HEX_SCALE,
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
    client_view_settings: WriteSignal<ClientViewSettings>,
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
                    client_view_settings,
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
                    client_view_settings,
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
                    client_view_settings,
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
    let game_state = &*game_state.read();
    rsx! {
        h2 { "Major Bodies" }
        p {
            for & celestial_id in game_state.celestials.majors().iter() {
                {
                    let celestial = game_state.celestials.get(celestial_id).unwrap();
                    rsx! {
                        Fragment { key: "{celestial_id:?}",
                            a {
                                href: "#",
                                role: "button",
                                onclick: move |event| {
                                    event.prevent_default();
                                    change_state(SidebarState::CelestialDetails(celestial_id));
                                },
                                "{celestial.name}"
                            }
                            br {}
                        }
                    }
                }
            }
        }
        h2 { "Your Stacks" }
        p {
            for (& stack_id , stack) in game_state.stacks.iter().filter(|(_, stack)| stack.owner == me) {
                {
                    rsx! {
                        Fragment { key: "{stack_id:?}",
                            a {
                                href: "#",
                                role: "button",
                                onclick: move |event| {
                                    event.prevent_default();
                                    change_state(SidebarState::StackDetails(stack_id));
                                },
                                "{stack.name}"
                            }
                            br {}
                        }
                    }
                }
            }
        }
        h2 { "Other Stacks" }
        p {
            for (& stack_id , stack) in game_state.stacks.iter().filter(|(_, stack)| stack.owner != me) {
                {
                    rsx! {
                        Fragment { key: "{stack_id:?}",
                            a {
                                href: "#",
                                role: "button",
                                onclick: move |event| {
                                    event.prevent_default();
                                    change_state(SidebarState::StackDetails(stack_id));
                                },
                                "{stack.name}"
                            }
                            br {}
                        }
                    }
                }
            }
        }
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
    // TODO
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
    let game_state = &*game_state.read();
    rsx! {
        h2 { "Icon Settings" }
        table { class: "table",
            thead {
                tr {
                    th { scope: "col", "Player" }
                    th { scope: "col", "Hostile" }
                    th { scope: "col", "Neutral" }
                    th { scope: "col", "Friendly" }
                }
            }
            tbody {
                for (& player , name) in game_state.players.iter().filter(|&(&player, _)| player != me) {
                    tr { key: "{player:?}",
                        td { "{name}" }
                        td {
                            input {
                                class: "form-check-input",
                                r#type: "radio",
                                name: "hostility-{player:?}",
                                checked: matches!(
                                    client_game_settings.read().display_hostility[&player],
                                    DisplayHostility::Hostile
                                ),
                                onchange: move |_| {
                                    client_game_settings
                                        .write()
                                        .display_hostility
                                        .insert(player, DisplayHostility::Hostile);
                                },
                            }
                        }
                        td {
                            input {
                                class: "form-check-input",
                                r#type: "radio",
                                name: "hostility-{player:?}",
                                checked: matches!(
                                    client_game_settings.read().display_hostility[&player],
                                    DisplayHostility::Neutral
                                ),
                                onchange: move |_| {
                                    client_game_settings
                                        .write()
                                        .display_hostility
                                        .insert(player, DisplayHostility::Neutral);
                                },
                            }
                        }
                        td {
                            input {
                                class: "form-check-input",
                                r#type: "radio",
                                name: "hostility-{player:?}",
                                checked: matches!(
                                    client_game_settings.read().display_hostility[&player],
                                    DisplayHostility::Friendly
                                ),
                                onchange: move |_| {
                                    client_game_settings
                                        .write()
                                        .display_hostility
                                        .insert(player, DisplayHostility::Friendly);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CelestialDetails(
    id: CelestialId,
    game_state: ReadSignal<GameState>,
    client_view_settings: WriteSignal<ClientViewSettings>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    let game_state_ref = &*game_state.read();
    let Some(celestial) = game_state_ref.celestials.get(id) else {
        return rsx! {
            h1 { "Unknown Celestial Body" }
        };
    };

    let stacks_nearby = game_state_ref
        .stacks
        .iter()
        .filter(|(_, stack)| (stack.position - celestial.position).norm() <= 1)
        .collect::<Vec<_>>();

    rsx! {
        h1 {
            "{celestial.name} "
            button {
                r#type: "button",
                class: "btn btn-secondary btn-sm regular-font",
                onclick: {
                    let position = celestial.position.cartesian() * HEX_SCALE;
                    move |_| {
                        client_view_settings
                            .set(ClientViewSettings {
                                x_offset: -position.x,
                                y_offset: -position.y,
                                zoom_level: 0,
                            })
                    }
                },
                "Go To"
            }
        }
        if celestial.orbit_gravity {
            p {
                "Can orbit"
                br {}
                if celestial.can_land() {
                    "Surface gravity: {celestial.surface_gravity:.1} m/s² ({celestial.surface_gravity / 2.0:.1} hex/turn)²"
                } else {
                    "Can't land"
                }
            }
        } else {
            p { "Low gravity" }
        }
        match celestial.resources {
            Resources::MiningBoth => rsx! {
                p { "May mine ice and ore" }
            },
            Resources::MiningIce => rsx! {
                p { "May mine ice" }
            },
            Resources::MiningOre => rsx! {
                p { "May mine ore" }
            },
            Resources::Skimming => rsx! {
                p { "May skim fuel" }
            },
            Resources::None => rsx! {
                p { "No available resources" }
            },
        }
        if celestial.is_minor {
            p { "Minor body" }
        }
        if !stacks_nearby.is_empty() {
            h2 { "Stacks Nearby" }
            p {
                for (& stack_id , stack) in stacks_nearby {
                    {
                        rsx! {
                            Fragment { key: "{stack_id:?}",
                                a {
                                    href: "#",
                                    role: "button",
                                    onclick: move |event| {
                                        event.prevent_default();
                                        change_state(SidebarState::StackDetails(stack_id));
                                    },
                                    "{stack.name}"
                                }
                                br {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn StackDetails(
    me: PlayerId,
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    client_view_settings: WriteSignal<ClientViewSettings>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    // TODO
    rsx! {}
}

#[component]
pub fn Disambiguate(
    celestial_id: Option<CelestialId>,
    stack_ids: Vec<StackId>,
    game_state: ReadSignal<GameState>,
    client_view_settings: WriteSignal<ClientViewSettings>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    let game_state = &*game_state.read();

    rsx! {
        h1 { "Multiple Objects" }
        if let Some(celestial_id) = celestial_id {
            {
                let celestial = game_state.celestials.get(celestial_id).unwrap();
                rsx! {
                    Fragment { key: "{celestial_id:?}",
                        a {
                            href: "#",
                            role: "button",
                            onclick: move |event| {
                                event.prevent_default();
                                change_state(SidebarState::CelestialDetails(celestial_id));
                            },
                            "{celestial.name}"
                        }
                        br {}
                    }
                }
            }
        }
        for stack_id in stack_ids {
            {
                let stack = game_state.stacks.get(&stack_id).unwrap();
                rsx! {
                    Fragment { key: "{stack_id:?}",
                        a {
                            href: "#",
                            role: "button",
                            onclick: move |event| {
                                event.prevent_default();
                                change_state(SidebarState::StackDetails(stack_id));
                            },
                            "{stack.name}"
                        }
                        br {}
                    }
                }
            }
        }
    }
}
