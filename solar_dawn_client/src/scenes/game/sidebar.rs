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
    collections::{HashMap, HashSet},
    ptr,
};

use dioxus::prelude::*;
use serde_cbor::to_vec;
use solar_dawn_common::{
    GameState, Phase, PlayerId, Vec2,
    celestial::{CelestialId, Resources},
    order::{ModuleTransferTarget, Order, OrderError, ResourceTransferTarget},
    stack::{Health, Module, ModuleDetails, ModuleId, StackId},
};
use strum::{EnumString, IntoStaticStr};

use crate::{
    scenes::game::{
        ClickBroker, ClientGameSettings, ClientViewSettings, DisplayHostility, OutlinerState,
        SidebarState, SidebarStateStoreExt, SidebarStateStoreTransposed, map::HEX_SCALE,
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
    click_broker: Signal<ClickBroker>,
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
                    order_errors,
                    change_state,
                    click_broker,
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
                        order_errors,
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
            disabled: *submitting_orders.read() || order_errors.read().iter().any(|error| error.is_some()),
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
    order_errors: ReadSignal<Vec<Option<OrderError>>>,
    change_state: EventHandler<SidebarState>,
) -> Element {
    let game_state = &*game_state.read();
    let order_errors = &*order_errors.read();
    rsx! {
        h2 { "Orders" }
        p {
            for (index , order) in orders.read().iter().enumerate() {
                Fragment { key: "{index}:{order:?}",
                    "{order.display_attributed(game_state)} "
                    if let Some(Some(error)) = order_errors.get(index) {
                        span { class: "text-danger", "{error.display(game_state)}" }
                        " "
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-secondary btn-sm",
                        onclick: move |_| {
                            orders.write().remove(index);
                        },
                        "Cancel"
                    }
                    br {}
                }
            }
        }
        h2 { "Automatic Orders" }
        p {
            for (index , (order , enabled)) in auto_orders.read().iter().enumerate() {
                Fragment { key: "{index}:{order:?}",
                    if *enabled {
                        "{order.display_attributed(game_state)}"
                        if let Some(Some(error)) = order_errors.get(orders.read().len() + index) {
                            " "
                            span { class: "text-danger", "{error.display(game_state)}" }
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| {
                                auto_orders.write()[index].1 = false;
                            },
                            "Cancel"
                        }
                    } else {
                        s { "{order.display_attributed(game_state)}" }
                        button {
                            r#type: "button",
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| {
                                auto_orders.write()[index].1 = true;
                            },
                            "Reinstate"
                        }
                    }
                }
            }
        }
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
    let game_state = &*game_state.read();
    let Some(celestial) = game_state.celestials.get(id) else {
        return rsx! {
            h1 { "Unknown Celestial Body" }
        };
    };

    let stacks_nearby = game_state
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
                    "Surface gravity: {celestial.surface_gravity:.1} m/s² ({celestial.surface_gravity / 2.0:.1} hex/turn²)"
                } else {
                    "Can't land"
                }
            }
        } else {
            p { "Low gravity" }
        }
        match celestial.resources {
            Resources::MiningBoth => rsx! {
                p { "May mine water and ore" }
            },
            Resources::MiningWater => rsx! {
                p { "May mine water" }
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

#[derive(Clone, Copy, EnumString, IntoStaticStr)]
enum DraftOrder {
    None,
    NameStack,
    ModuleTransfer,
    ModuleTransferNew,
    Board,
    IsruMine,
    IsruSkim,
    ResourceTransferFromModule,
    ResourceTransferToModule,
    ResourceTransferToStack,
    ResourceTransferJettison,
    Repair,
    Refine,
    Build,
    Salvage,
    Shoot,
    Arm,
    Disarm,
    Burn,
    OrbitAdjust,
    Land,
    TakeOff,
}

#[component]
pub fn StackDetails(
    me: PlayerId,
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    auto_orders: WriteSignal<Vec<(Order, bool)>>,
    order_errors: ReadSignal<Vec<Option<OrderError>>>,
    client_view_settings: WriteSignal<ClientViewSettings>,
    change_state: EventHandler<SidebarState>,
    click_broker: Signal<ClickBroker>,
) -> Element {
    let mut draft_order = use_signal(|| DraftOrder::None);

    use_effect(move || {
        if matches!(*draft_order.read(), DraftOrder::None) {
            click_broker.write().clear();
        }
    });

    let game_state_ref = &*game_state.read();
    let Some(stack) = game_state_ref.stacks.get(&id) else {
        return rsx! {
            h1 { "Unknown Stack" }
        };
    };
    let order_errors = &*order_errors.read();

    rsx! {
        h1 {
            "{stack.name} "
            button {
                r#type: "button",
                class: "btn btn-secondary btn-sm regular-font",
                onclick: {
                    let position = stack.position.cartesian() * HEX_SCALE;
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
        p {
            if stack.owner == me {
                "Your stack"
            } else {
                "Owned by {game_state_ref.players[&stack.owner]}"
            }
            br {}
            "Current mass: {stack.mass():.1}t"
            br {}
            "Empty mass: {stack.dry_mass()}t"
            br {}
            "Fully loaded mass: {stack.full_mass()}t"
            br {}
            {
                let (intact, damaged, destroyed) = stack.damage_status();
                let intact_word = if intact == 1 { "module" } else { "modules" };
                let damaged_word = if damaged == 1 { "module" } else { "modules" };
                let destroyed_word = if destroyed == 1 { "module" } else { "modules" };
                rsx! { "{intact} intact {intact_word}, {damaged} damaged {damaged_word}, {destroyed} destroyed {destroyed_word}" }
            }
            br {}
            "Current ΔV: {stack.current_dv():.1} hex/turn"
            br {}
            "Fully fuelled ΔV: {stack.max_dv():.1} hex/turn"
            br {}
            "Acceleration: {stack.acceleration() / 2.0:.1} hex/turn² ({stack.acceleration():.1} m/s²)"
        }
        h2 { "Modules" }
        p {
            for (& module_id , module) in stack.modules.iter() {
                Fragment { key: "{module_id:?}",
                    "{module:#}"
                    br {}
                }
            }
        }
        if stack.owner == me {
            h2 { "Orders" }
            select {
                class: "form-select",
                oninput: move |e| {
                    draft_order.set(e.value().parse::<DraftOrder>().unwrap_or(DraftOrder::None));
                },
                option {
                    value: <&'static str>::from(DraftOrder::None),
                    selected: matches!(&*draft_order.read(), DraftOrder::None),
                    "Give an order..."
                }
                option { value: <&'static str>::from(DraftOrder::NameStack), "Rename" }
                match game_state_ref.phase {
                    Phase::Logistics => rsx! {
                        if game_state_ref
                            .stacks
                            .values()
                            .any(|target| {
                                target.owner != me && target.rendezvoused_with(stack)
                                    && !target
                                        .modules
                                        .values()
                                        .any(|module| {
                                            matches!(
                                                module,
                                                Module {
                                                    health: Health::Intact,
                                                    details: ModuleDetails::Habitat { .. },
                                                }
                                            )
                                        })
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Board), "Board" }
                        }
                        if game_state_ref
                            .stacks
                            .values()
                            .any(|target| {
                                !ptr::eq(stack, target) && target.owner == me
                                    && target.rendezvoused_with(stack)
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::ModuleTransfer), "Transfer module" }
                        }
                        option { value: <&'static str>::from(DraftOrder::ModuleTransferNew), "Transfer module to new stack" }
                        if stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::CargoHold { .. } | ModuleDetails::Tank { .. },
                                    }
                                )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::ResourceTransferFromModule),
                                "Transfer resources from module"
                            }
                            option { value: <&'static str>::from(DraftOrder::ResourceTransferToModule),
                                "Transfer resources to module"
                            }
                        }
                        if game_state_ref
                            .stacks
                            .values()
                            .any(|target| {
                                !ptr::eq(stack, target) && target.owner == me
                                    && target.rendezvoused_with(stack)
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::ResourceTransferToStack),
                                "Transfer resources to stack"
                            }
                        }
                        option { value: <&'static str>::from(DraftOrder::ResourceTransferJettison), "Jettison resources" }
                        if game_state_ref
                            .celestials
                            .get_by_position(stack.position)
                            .is_some_and(|(_, celestial)| {
                                stack.landed(celestial)
                                    && matches!(
                                        celestial.resources,
                                        Resources::MiningBoth | Resources::MiningWater | Resources::MiningOre
                                    )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::IsruMine), "Mine" }
                        }
                        if game_state_ref
                            .celestials
                            .with_gravity()
                            .any(|(_, celestial)| {
                                stack.orbiting(celestial)
                                    && matches!(celestial.resources, Resources::Skimming)
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::IsruSkim), "Skim fuel" }
                        }
                        if stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module { health: Health::Intact, details: ModuleDetails::Refinery }
                                )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Refine), "Refine" }
                        }
                        if stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Factory | ModuleDetails::Habitat { .. },
                                    }
                                )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Repair), "Repair" }
                        }
                        if stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module { health: Health::Intact, details: ModuleDetails::Factory }
                                )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Build), "Build" }
                            option { value: <&'static str>::from(DraftOrder::Salvage), "Salvage" }
                        }
                    },
                    Phase::Combat => rsx! {
                        if stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module { health: Health::Intact, details: ModuleDetails::Gun }
                                )
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Shoot), "Shoot" }
                        }
                        if !stack
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module {
                                        health: Health::Intact | Health::Damaged,
                                        details: ModuleDetails::Habitat { .. },
                                    }
                                )
                            })
                        {
                            if stack
                                .modules
                                .values()
                                .any(|module| {
                                    matches!(
                                        module,
                                        Module {
                                            health: Health::Intact,
                                            details: ModuleDetails::Warhead { armed: false },
                                        }
                                    )
                                })
                            {
                                option { value: <&'static str>::from(DraftOrder::Arm), "Arm" }
                            }
                            if stack
                                .modules
                                .values()
                                .any(|module| {
                                    matches!(
                                        module,
                                        Module {
                                            health: Health::Intact,
                                            details: ModuleDetails::Warhead { armed: true },
                                        }
                                    )
                                })
                            {
                                option { value: <&'static str>::from(DraftOrder::Disarm), "Disarm" }
                            }
                        }
                    },
                    Phase::Movement => rsx! {
                        if stack.acceleration() >= 2.0 {
                            option { value: <&'static str>::from(DraftOrder::Burn), "Burn" }
                        }
                        if game_state_ref
                            .celestials
                            .with_gravity()
                            .any(|(_, celestial)| {
                                stack.orbiting(celestial) && stack.acceleration() >= 2.0
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::OrbitAdjust), "Adjust orbit" }
                        }
                        if game_state_ref
                            .celestials
                            .with_gravity()
                            .any(|(_, celestial)| {
                                stack.orbiting(celestial) && stack.acceleration() >= 2.0
                                    && stack.acceleration() >= celestial.surface_gravity
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::Land), "Land" }
                        }
                        if game_state_ref
                            .celestials
                            .get_by_position(stack.position)
                            .is_some_and(|(_, celestial)| {
                                stack.landed_with_gravity(celestial) && stack.acceleration() >= 2.0
                                    && stack.acceleration() >= celestial.surface_gravity
                            })
                        {
                            option { value: <&'static str>::from(DraftOrder::TakeOff), "Take off" }
                        }
                    },
                }
            }
            match *draft_order.read() {
                DraftOrder::None => {
                    // pass
                    rsx! {}
                }
                DraftOrder::NameStack => {
                    rsx! {
                        NameStack { id, orders, draft_order }
                    }
                }
                DraftOrder::Board => {
                    rsx! {
                        BoardStack {
                            id,
                            me,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ModuleTransfer => {
                    rsx! {
                        ModuleTransfer {
                            id,
                            me,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ModuleTransferNew => {
                    rsx! {
                        ModuleTransferNew {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ResourceTransferFromModule => {
                    rsx! {
                        ResourceTransferFromModule {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ResourceTransferToModule => {
                    rsx! {
                        ResourceTransferToModule {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ResourceTransferToStack => {
                    rsx! {
                        ResourceTransferToStack {
                            id,
                            me,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::ResourceTransferJettison => {
                    rsx! {
                        ResourceTransferJettison {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::IsruMine => {
                    rsx! {
                        IsruMine {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::IsruSkim => {
                    rsx! {
                        IsruSkim {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Repair => {
                    rsx! {
                        Repair {
                            id,
                            me,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Refine => {
                    rsx! {
                        Refine {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Build => {
                    rsx! {
                        Build {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Salvage => {
                    rsx! {
                        Salvage {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Shoot => {
                    rsx! {
                        Shoot {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Arm => {
                    rsx! {
                        Arm {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Disarm => {
                    rsx! {
                        Disarm {
                            id,
                            game_state,
                            orders,
                            draft_order,
                        }
                    }
                }
                DraftOrder::Burn => {
                    rsx! {
                        Burn {
                            id,
                            game_state,
                            orders,
                            draft_order,
                            click_broker,
                        }
                    }
                }
                DraftOrder::OrbitAdjust => {
                    rsx! {
                        OrbitAdjust {
                            id,
                            game_state,
                            orders,
                            draft_order,
                            click_broker,
                        }
                    }
                }
                DraftOrder::Land => {
                    rsx! {
                        Land {
                            id,
                            game_state,
                            orders,
                            draft_order,
                            click_broker,
                        }
                    }
                }
                DraftOrder::TakeOff => {
                    rsx! {
                        TakeOff {
                            id,
                            game_state,
                            orders,
                            draft_order,
                            click_broker,
                        }
                    }
                }
            }
            hr {}
            p {
                for (index , order) in orders.read().iter().enumerate().filter(|(_, order)| order.target() == id) {
                    Fragment { key: "{index}:{order:?}",
                        "{order.display_unattributed(game_state_ref)} "
                        if let Some(Some(error)) = order_errors.get(index) {
                            span { class: "text-danger", "{error.display(game_state_ref)}" }
                            " "
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| {
                                orders.write().remove(index);
                            },
                            "Cancel"
                        }
                        br {}
                    }
                }
            }
            h2 { "Automatic Orders" }
            p {
                for (index , (order , enabled)) in auto_orders.read().iter().enumerate().filter(|(_, (order, _))| order.target() == id) {
                    if *enabled {
                        "{order.display_unattributed(game_state_ref)} "
                        if let Some(Some(error)) = order_errors.get(orders.read().len() + index) {
                            span { class: "text-danger", "{error.display(game_state_ref)}" }
                            " "
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| {
                                auto_orders.write()[index].1 = false;
                            },
                            "Cancel"
                        }
                    } else {
                        s { "{order.display_unattributed(game_state_ref)}" }
                        button {
                            r#type: "button",
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| {
                                auto_orders.write()[index].1 = true;
                            },
                            "Reinstate"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NameStack(
    id: StackId,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut name = use_signal(String::new);
    rsx! {
        label { r#for: "new-name", class: "form-label", "New name" }
        input {
            r#type: "text",
            id: "new-name",
            class: "form-control",
            oninput: move |e| {
                name.set(e.value());
            },
            ""
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::NameStack {
                        stack: id,
                        name: name.read().clone(),
                    });
                draft_order.set(DraftOrder::None);
            },
            "Submit"
        }
    }
}

#[component]
fn BoardStack(
    id: StackId,
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected = use_signal(|| Option::<StackId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected.set(None);
                } else {
                    selected.set(Some(value.parse::<StackId>().unwrap()));
                }
            },
            option { value: "none", "Select target..." }
            for (target_id , stack) in game_state
                .stacks
                .iter()
                .filter(|(_, target)| {
                    target.owner != me && target.rendezvoused_with(stack)
                        && !target
                            .modules
                            .values()
                            .any(|module| {
                                matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Habitat { .. },
                                    }
                                )
                            })
                })
            {
                option { value: "{target_id}", "{stack.name}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected) = &*selected.read() {
                    orders
                        .write()
                        .push(Order::Board {
                            stack: id,
                            target: *selected,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn ModuleTransfer(
    id: StackId,
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let mut selected_target = use_signal(|| Option::<StackId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_module.set(None);
                } else {
                    selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select module..." }
            for (module_id , module) in stack.modules.iter() {
                option { value: "{module_id}", "{module}" }
            }
        }
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_target.set(None);
                } else {
                    selected_target.set(Some(value.parse::<StackId>().unwrap()));
                }
            },
            option { value: "none", "Select target stack..." }
            for (target_id , target_stack) in game_state
                .stacks
                .iter()
                .filter(|(target_id, target)| {
                    **target_id != id && target.owner == me && target.rendezvoused_with(stack)
                })
            {
                option { value: "{target_id}", "{target_stack.name}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_module) = &*selected_module.read()
                    && let Some(selected_target) = &*selected_target.read()
                {
                    orders
                        .write()
                        .push(Order::ModuleTransfer {
                            stack: id,
                            module: *selected_module,
                            to: ModuleTransferTarget::Existing(*selected_target),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_module.read().is_none() || selected_target.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn ModuleTransferNew(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let mut selected_target = use_signal(|| Option::<u32>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    let mut possible_new_stack_numbers = orders
        .read()
        .iter()
        .filter_map(|order| {
            if let Order::ModuleTransfer {
                stack: transferring,
                to: ModuleTransferTarget::New(to),
                ..
            } = order
            {
                let transferring = &game_state.stacks[transferring];
                if transferring.rendezvoused_with(stack) {
                    Some(*to)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    possible_new_stack_numbers.sort();
    possible_new_stack_numbers.push(
        orders
            .read()
            .iter()
            .filter_map(|order| {
                if let Order::ModuleTransfer {
                    to: ModuleTransferTarget::New(to),
                    ..
                } = order
                {
                    Some(*to)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0)
            + 1,
    );

    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_module.set(None);
                } else {
                    selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select module..." }
            for (module_id , module) in stack.modules.iter() {
                option { value: "{module_id}", "{module}" }
            }
        }
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_target.set(None);
                } else {
                    selected_target.set(Some(value.parse::<u32>().unwrap()));
                }
            },
            option { value: "none", "Select new stack number..." }
            for new_stack_number in possible_new_stack_numbers {
                option { value: "{new_stack_number}", "New stack #{new_stack_number}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_module) = &*selected_module.read()
                    && let Some(selected_target) = &*selected_target.read()
                {
                    orders
                        .write()
                        .push(Order::ModuleTransfer {
                            stack: id,
                            module: *selected_module,
                            to: ModuleTransferTarget::New(*selected_target),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_module.read().is_none() || selected_target.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn ResourceTransferFromModule(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let mut ore = use_signal(|| 0_u8);
    let mut materials = use_signal(|| 0_u8);
    let mut water = use_signal(|| 0_u8);
    let mut fuel = use_signal(|| 0_u8);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_module.set(None);
                } else {
                    selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select module..." }
            for (module_id , module) in stack
                .modules
                .iter()
                .filter(|(_, module)| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::CargoHold { .. } | ModuleDetails::Tank { .. },
                        }
                    )
                })
            {
                option { value: "{module_id}", "{module}" }
            }
        }
        match &*selected_module.read() {
            Some(id) => {
                match &stack.modules[id] {
                    Module {
                        // pass
                        // pass
                        details: ModuleDetails::CargoHold {
                            ore: max_ore,
                            materials: max_materials,
                        },
                        ..
                    } => {
                        rsx! {
                            label { r#for: "ore", class: "form-label", "Ore" }
                            input {
                                r#type: "range",
                                id: "ore",
                                class: "form-range",
                                min: 0,
                                max: *max_ore,
                                value: "{*ore.read()}",
                                oninput: move |e| {
                                    ore.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*ore.read() as f32 / 10.0:.1}t" }
                            br {}
                            label { r#for: "materials", class: "form-label", "Materials" }
                            input {
                                r#type: "range",
                                id: "materials",
                                class: "form-range",
                                min: 0,
                                max: *max_materials,
                                value: "{*materials.read()}",
                                oninput: move |e| {
                                    materials.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*materials.read() as f32 / 10.0:.1}t" }
                            br {}
                        }
                    }
                    Module {
                        details: ModuleDetails::Tank { water: max_water, fuel: max_fuel },
                        ..
                    } => {
                        rsx! {
                            label { r#for: "water", class: "form-label", "Water" }
                            input {
                                r#type: "range",
                                id: "water",
                                class: "form-range",
                                min: 0,
                                max: *max_water,
                                value: "{*water.read()}",
                                oninput: move |e| {
                                    water.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*water.read() as f32 / 10.0:.1}t" }
                            br {}
                            label { r#for: "fuel", class: "form-label", "Fuel" }
                            input {
                                r#type: "range",
                                id: "fuel",
                                class: "form-range",
                                min: 0,
                                max: *max_fuel,
                                value: "{*fuel.read()}",
                                oninput: move |e| {
                                    fuel.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*fuel.read() as f32 / 10.0:.1}t" }
                            br {}
                        }
                    }
                    _ => {
                        rsx! {}
                    }
                }
            }
            _ => {
                rsx! {}
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_module) = &*selected_module.read() {
                    orders
                        .write()
                        .push(Order::ResourceTransfer {
                            stack: id,
                            from: Some(*selected_module),
                            to: ResourceTransferTarget::FloatingPool,
                            ore: *ore.read(),
                            materials: *materials.read(),
                            water: *water.read(),
                            fuel: *fuel.read(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_module.read().is_none()
                || !(*ore.read() != 0 || *materials.read() != 0 || *water.read() != 0
                    || *fuel.read() != 0),
            "Submit"
        }
    }
}

#[component]
fn ResourceTransferToModule(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let mut ore = use_signal(|| 0_u8);
    let mut materials = use_signal(|| 0_u8);
    let mut water = use_signal(|| 0_u8);
    let mut fuel = use_signal(|| 0_u8);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_module.set(None);
                } else {
                    selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select module..." }
            for (module_id , module) in stack
                .modules
                .iter()
                .filter(|(_, module)| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::CargoHold { .. } | ModuleDetails::Tank { .. },
                        }
                    )
                })
            {
                option { value: "{module_id}", "{module}" }
            }
        }
        match &*selected_module.read() {
            Some(id) => {
                match &stack.modules[id] {
                    Module {
                        details: ModuleDetails::CargoHold {
                            ore: current_ore,
                            materials: current_materials,
                        },
                        ..
                    } => {
                        let max_ore = ModuleDetails::CARGO_HOLD_CAPACITY as u8 - current_ore
                            - current_materials;
                        let max_materials = ModuleDetails::CARGO_HOLD_CAPACITY as u8
                            - current_ore - current_materials;
                        rsx! {
                            label { r#for: "ore", class: "form-label", "Ore" }
                            input {
                                r#type: "range",
                                id: "ore",
                                class: "form-range",
                                min: 0,
                                max: max_ore - *materials.read(),
                                value: "{*ore.read()}",
                                oninput: move |e| {
                                    ore.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*ore.read() as f32 / 10.0:.1}t (free space: {(max_ore - *ore.read()) as f32 / 10.0:.1}t)" }
                            br {}
                            label { r#for: "materials", class: "form-label", "Materials" }
                            input {
                                r#type: "range",
                                id: "materials",
                                class: "form-range",
                                min: 0,
                                max: max_materials - *ore.read(),
                                value: "{*materials.read()}",
                                oninput: move |e| {
                                    materials.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output {
                                "{*materials.read() as f32 / 10.0:.1}t (free space: {(max_materials - *ore.read() - *materials.read()) as f32 / 10.0:.1}t)"
                            }
                            br {}
                        }
                    }
                    Module {
                        details: ModuleDetails::Tank {
                            water: current_water,
                            fuel: current_fuel,
                        },
                        ..
                    } => {
                        let max_water = ModuleDetails::TANK_CAPACITY as u8 - current_water
                            - current_fuel;
                        let max_fuel = ModuleDetails::TANK_CAPACITY as u8 - current_water
                            - current_fuel;
                        rsx! {
                            label { r#for: "water", class: "form-label", "Water" }
                            input {
                                r#type: "range",
                                id: "water",
                                class: "form-range",
                                min: 0,
                                max: max_water - *fuel.read(),
                                value: "{*water.read()}",
                                oninput: move |e| {
                                    water.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output { "{*water.read() as f32 / 10.0:.1}t (free space: {(max_water - *water.read()) as f32 / 10.0:.1}t)" }
                            br {}
                            label { r#for: "fuel", class: "form-label", "Fuel" }
                            input {
                                r#type: "range",
                                id: "fuel",
                                class: "form-range",
                                min: 0,
                                max: max_fuel - *water.read(),
                                value: "{*fuel.read()}",
                                oninput: move |e| {
                                    fuel.set(e.value().parse::<u8>().unwrap());
                                },
                            }
                            output {
                                "{*fuel.read() as f32 / 10.0:.1}t (free space: {(max_fuel - *water.read() - *fuel.read()) as f32 / 10.0:.1}t)"
                            }
                            br {}
                        }
                    }
                    _ => {
                        rsx! {}
                    }
                }
            }
            _ => {
                rsx! {}
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_module) = &*selected_module.read() {
                    orders
                        .write()
                        .push(Order::ResourceTransfer {
                            stack: id,
                            from: None,
                            to: ResourceTransferTarget::Module(*selected_module),
                            ore: *ore.read(),
                            materials: *materials.read(),
                            water: *water.read(),
                            fuel: *fuel.read(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_module.read().is_none()
                || !(*ore.read() != 0 || *materials.read() != 0 || *water.read() != 0
                    || *fuel.read() != 0),
            "Submit"
        }
    }
}

#[component]
fn ResourceTransferToStack(
    id: StackId,
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_stack = use_signal(|| Option::<StackId>::None);
    let mut ore = use_signal(|| 0_u8);
    let mut materials = use_signal(|| 0_u8);
    let mut water = use_signal(|| 0_u8);
    let mut fuel = use_signal(|| 0_u8);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_stack.set(None);
                } else {
                    selected_stack.set(Some(value.parse::<StackId>().unwrap()));
                }
            },
            option { value: "none", "Select target stack..." }
            for (target_id , target_stack) in game_state
                .stacks
                .iter()
                .filter(|(target_id, target)| {
                    **target_id != id && target.owner == me && target.rendezvoused_with(stack)
                })
            {
                option { value: "{target_id}", "{target_stack.name}" }
            }
        }
        label { r#for: "ore", class: "form-label", "Ore" }
        input {
            r#type: "range",
            id: "ore",
            class: "form-range",
            min: 0,
            max: ModuleDetails::CARGO_HOLD_CAPACITY,
            value: "{*ore.read()}",
            oninput: move |e| {
                ore.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*ore.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "materials", class: "form-label", "Materials" }
        input {
            r#type: "range",
            id: "materials",
            class: "form-range",
            min: 0,
            max: ModuleDetails::CARGO_HOLD_CAPACITY,
            value: "{*materials.read()}",
            oninput: move |e| {
                materials.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*materials.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "water", class: "form-label", "Water" }
        input {
            r#type: "range",
            id: "water",
            class: "form-range",
            min: 0,
            max: ModuleDetails::TANK_CAPACITY,
            value: "{*water.read()}",
            oninput: move |e| {
                water.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*water.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "fuel", class: "form-label", "Fuel" }
        input {
            r#type: "range",
            id: "fuel",
            class: "form-range",
            min: 0,
            max: ModuleDetails::TANK_CAPACITY,
            value: "{*fuel.read()}",
            oninput: move |e| {
                fuel.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*fuel.read() as f32 / 10.0:.1}t" }
        br {}
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_stack) = &*selected_stack.read() {
                    orders
                        .write()
                        .push(Order::ResourceTransfer {
                            stack: id,
                            from: None,
                            to: ResourceTransferTarget::Stack(*selected_stack),
                            ore: *ore.read(),
                            materials: *materials.read(),
                            water: *water.read(),
                            fuel: *fuel.read(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_stack.read().is_none()
                || !(*ore.read() != 0 || *materials.read() != 0 || *water.read() != 0
                    || *fuel.read() != 0),
            "Submit"
        }
    }
}

#[component]
fn ResourceTransferJettison(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut ore = use_signal(|| 0_u8);
    let mut materials = use_signal(|| 0_u8);
    let mut water = use_signal(|| 0_u8);
    let mut fuel = use_signal(|| 0_u8);
    rsx! {
        label { r#for: "ore", class: "form-label", "Ore" }
        input {
            r#type: "range",
            id: "ore",
            class: "form-range",
            min: 0,
            max: ModuleDetails::CARGO_HOLD_CAPACITY,
            value: "{*ore.read()}",
            oninput: move |e| {
                ore.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*ore.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "materials", class: "form-label", "Materials" }
        input {
            r#type: "range",
            id: "materials",
            class: "form-range",
            min: 0,
            max: ModuleDetails::CARGO_HOLD_CAPACITY,
            value: "{*materials.read()}",
            oninput: move |e| {
                materials.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*materials.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "water", class: "form-label", "Water" }
        input {
            r#type: "range",
            id: "water",
            class: "form-range",
            min: 0,
            max: ModuleDetails::TANK_CAPACITY,
            value: "{*water.read()}",
            oninput: move |e| {
                water.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*water.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "fuel", class: "form-label", "Fuel" }
        input {
            r#type: "range",
            id: "fuel",
            class: "form-range",
            min: 0,
            max: ModuleDetails::TANK_CAPACITY,
            value: "{*fuel.read()}",
            oninput: move |e| {
                fuel.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*fuel.read() as f32 / 10.0:.1}t" }
        br {}
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::ResourceTransfer {
                        stack: id,
                        from: None,
                        to: ResourceTransferTarget::Jettison,
                        ore: *ore.read(),
                        materials: *materials.read(),
                        water: *water.read(),
                        fuel: *fuel.read(),
                    });
                draft_order.set(DraftOrder::None);
            },
            disabled: !(*ore.read() != 0 || *materials.read() != 0 || *water.read() != 0
                || *fuel.read() != 0),
            "Submit"
        }
    }
}

#[component]
fn IsruMine(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut ore = use_signal(|| 0_u32);
    let mut water = use_signal(|| 0_u32);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];

    let total_capacity = stack
        .modules
        .values()
        .filter(|module| {
            matches!(
                module,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Miner,
                }
            )
        })
        .count() as u32
        * ModuleDetails::MINER_PRODUCTION_RATE;

    let celestial = game_state
        .celestials
        .get_by_position(stack.position)
        .map(|(_, celestial)| celestial);

    let can_mine_ore = celestial.is_some_and(|celestial| {
        matches!(
            celestial.resources,
            Resources::MiningBoth | Resources::MiningOre
        )
    });
    let can_mine_water = celestial.is_some_and(|celestial| {
        matches!(
            celestial.resources,
            Resources::MiningBoth | Resources::MiningWater
        )
    });

    rsx! {
        if can_mine_ore {
            label { r#for: "ore", class: "form-label", "Ore" }
            input {
                r#type: "range",
                id: "ore",
                class: "form-range",
                min: 0,
                max: total_capacity - *water.read(),
                value: "{*ore.read()}",
                oninput: move |e| {
                    ore.set(e.value().parse::<u32>().unwrap());
                },
            }
            output { "{*ore.read() as f32 / 10.0:.1}t" }
            br {}
        }
        if can_mine_water {
            label { r#for: "water", class: "form-label", "Water" }
            input {
                r#type: "range",
                id: "water",
                class: "form-range",
                min: 0,
                max: total_capacity - *ore.read(),
                value: "{*water.read()}",
                oninput: move |e| {
                    water.set(e.value().parse::<u32>().unwrap());
                },
            }
            output { "{*water.read() as f32 / 10.0:.1}t" }
            br {}
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::Isru {
                        stack: id,
                        ore: *ore.read(),
                        water: *water.read(),
                        fuel: 0,
                    });
                draft_order.set(DraftOrder::None);
            },
            disabled: *ore.read() == 0 && *water.read() == 0,
            "Submit"
        }
    }
}

#[component]
fn IsruSkim(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut fuel = use_signal(|| 0_u32);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];

    let total_capacity = stack
        .modules
        .values()
        .filter(|module| {
            matches!(
                module,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::FuelSkimmer,
                }
            )
        })
        .count() as u32
        * ModuleDetails::FUEL_SKIMMER_PRODUCTION_RATE;

    rsx! {
        label { r#for: "fuel", class: "form-label", "Fuel" }
        input {
            r#type: "range",
            id: "fuel",
            class: "form-range",
            min: 0,
            max: total_capacity,
            value: "{*fuel.read()}",
            oninput: move |e| {
                fuel.set(e.value().parse::<u32>().unwrap());
            },
        }
        output { "{*fuel.read() as f32 / 10.0:.1}t" }
        br {}
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::Isru {
                        stack: id,
                        ore: 0,
                        water: 0,
                        fuel: *fuel.read(),
                    });
                draft_order.set(DraftOrder::None);
            },
            disabled: *fuel.read() == 0,
            "Submit"
        }
    }
}

#[component]
fn Repair(
    id: StackId,
    me: PlayerId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_stack = use_signal(|| Option::<StackId>::None);
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_stack.set(None);
                } else {
                    selected_stack.set(Some(value.parse::<StackId>().unwrap()));
                }
                selected_module.set(None);
            },
            option { value: "none", "Select target stack..." }
            for (target_id , target_stack) in game_state
                .stacks
                .iter()
                .filter(|(target_id, target)| {
                    (**target_id == id || target.rendezvoused_with(stack)) && target.owner == me
                })
            {
                option { value: "{target_id}", "{target_stack.name}" }
            }
        }
        if let Some(target_id) = *selected_stack.read() {
            {
                let target_stack = &game_state.stacks[&target_id];
                rsx! {
                    select {
                        class: "form-select",
                        oninput: move |e| {
                            let value = e.value();
                            if value == "none" {
                                selected_module.set(None);
                            } else {
                                selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                            }
                        },
                        option { value: "none", "Select module..." }
                        for (module_id , module) in target_stack
                            .modules
                            .iter()
                            .filter(|(_, module)| { matches!(module.health, Health::Damaged) })
                        {
                            option { value: "{module_id}", "{module}" }
                        }
                    }
                }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_stack) = &*selected_stack.read()
                    && let Some(selected_module) = &*selected_module.read()
                {
                    orders
                        .write()
                        .push(Order::Repair {
                            stack: id,
                            target_stack: *selected_stack,
                            target_module: *selected_module,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_stack.read().is_none() || selected_module.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Refine(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut materials = use_signal(|| 0_u8);
    let mut fuel = use_signal(|| 0_u8);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];

    let total_capacity = stack
        .modules
        .values()
        .filter(|module| {
            matches!(
                module,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Refinery,
                }
            )
        })
        .count() as u8
        * ModuleDetails::REFINERY_CAPACITY as u8;

    rsx! {
        label { r#for: "materials", class: "form-label", "Materials" }
        input {
            r#type: "range",
            id: "materials",
            class: "form-range",
            min: 0,
            max: total_capacity - *fuel.read(),
            value: "{*materials.read()}",
            oninput: move |e| {
                materials.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*materials.read() as f32 / 10.0:.1}t" }
        br {}
        label { r#for: "fuel", class: "form-label", "Fuel" }
        input {
            r#type: "range",
            id: "fuel",
            class: "form-range",
            min: 0,
            max: total_capacity - *materials.read(),
            value: "{*fuel.read()}",
            oninput: move |e| {
                fuel.set(e.value().parse::<u8>().unwrap());
            },
        }
        output { "{*fuel.read() as f32 / 10.0:.1}t" }
        br {}
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::Refine {
                        stack: id,
                        materials: *materials.read(),
                        fuel: *fuel.read(),
                    });
                draft_order.set(DraftOrder::None);
            },
            disabled: *materials.read() == 0 && *fuel.read() == 0,
            "Submit"
        }
    }
}

#[component]
fn Build(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_type = use_signal(|| Option::<solar_dawn_common::order::ModuleType>::None);
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_type.set(None);
                } else {
                    selected_type
                        .set(
                            Some(
                                match value.as_str() {
                                    "miner" => solar_dawn_common::order::ModuleType::Miner,
                                    "fuel_skimmer" => {
                                        solar_dawn_common::order::ModuleType::FuelSkimmer
                                    }
                                    "cargo_hold" => {
                                        solar_dawn_common::order::ModuleType::CargoHold
                                    }
                                    "tank" => solar_dawn_common::order::ModuleType::Tank,
                                    "engine" => solar_dawn_common::order::ModuleType::Engine,
                                    "warhead" => solar_dawn_common::order::ModuleType::Warhead,
                                    "gun" => solar_dawn_common::order::ModuleType::Gun,
                                    "habitat" => solar_dawn_common::order::ModuleType::Habitat,
                                    "refinery" => solar_dawn_common::order::ModuleType::Refinery,
                                    "factory" => solar_dawn_common::order::ModuleType::Factory,
                                    "armour_plate" => {
                                        solar_dawn_common::order::ModuleType::ArmourPlate
                                    }
                                    _ => return,
                                },
                            ),
                        );
                }
            },
            option { value: "none", "Select module type..." }
            option { value: "miner", "Miner" }
            option { value: "fuel_skimmer", "Fuel Skimmer" }
            option { value: "cargo_hold", "Cargo Hold" }
            option { value: "tank", "Tank" }
            option { value: "engine", "Engine" }
            option { value: "warhead", "Warhead" }
            option { value: "gun", "Gun" }
            option { value: "habitat", "Habitat" }
            option { value: "refinery", "Refinery" }
            option { value: "factory", "Factory" }
            option { value: "armour_plate", "Armour Plate" }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_type) = &*selected_type.read() {
                    orders
                        .write()
                        .push(Order::Build {
                            stack: id,
                            module: *selected_type,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_type.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Salvage(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_module = use_signal(|| Option::<ModuleId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_module.set(None);
                } else {
                    selected_module.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select module..." }
            for (module_id , module) in stack.modules.iter() {
                option { value: "{module_id}", "{module}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_module) = &*selected_module.read() {
                    orders
                        .write()
                        .push(Order::Salvage {
                            stack: id,
                            salvaged: *selected_module,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_module.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Shoot(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_target = use_signal(|| Option::<StackId>::None);
    let mut shots = use_signal(|| 1_u32);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];

    let max_shots = stack
        .modules
        .values()
        .filter(|module| {
            matches!(
                module,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Gun,
                }
            )
        })
        .count() as u32;

    let mut targets = game_state
        .stacks
        .iter()
        .filter(|(target_id, _)| **target_id != id)
        .map(|(target_id, target)| {
            let distance = (target.position - stack.position).norm();
            (target_id, target, distance)
        })
        .collect::<Vec<_>>();
    targets.sort_by(|(_, target_a, dist_a), (_, target_b, dist_b)| {
        dist_a
            .partial_cmp(dist_b)
            .unwrap()
            .then(target_a.name.cmp(&target_b.name))
    });

    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_target.set(None);
                } else {
                    selected_target.set(Some(value.parse::<StackId>().unwrap()));
                }
            },
            option { value: "none", "Select target..." }
            for (target_id , target , distance) in targets {
                option { value: "{target_id}",
                    "{game_state.players[&target.owner]} - {target.name} ({distance:.1} hex)"
                }
            }
        }
        label { r#for: "shots", class: "form-label", "Shots" }
        input {
            r#type: "range",
            id: "shots",
            class: "form-range",
            min: 1,
            max: max_shots,
            value: "{*shots.read()}",
            oninput: move |e| {
                shots.set(e.value().parse::<u32>().unwrap());
            },
        }
        output { "{*shots.read()}" }
        br {}
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_target) = &*selected_target.read() {
                    orders
                        .write()
                        .push(Order::Shoot {
                            stack: id,
                            target: *selected_target,
                            shots: *shots.read(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_target.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Arm(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_warhead = use_signal(|| Option::<ModuleId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_warhead.set(None);
                } else {
                    selected_warhead.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select warhead..." }
            for (module_id , module) in stack
                .modules
                .iter()
                .filter(|(_, module)| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Warhead { armed: false },
                        }
                    )
                })
            {
                option { value: "{module_id}", "{module}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_warhead) = &*selected_warhead.read() {
                    orders
                        .write()
                        .push(Order::Arm {
                            stack: id,
                            warhead: *selected_warhead,
                            armed: true,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_warhead.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Disarm(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
) -> Element {
    let mut selected_warhead = use_signal(|| Option::<ModuleId>::None);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    rsx! {
        select {
            class: "form-select",
            oninput: move |e| {
                let value = e.value();
                if value == "none" {
                    selected_warhead.set(None);
                } else {
                    selected_warhead.set(Some(value.parse::<ModuleId>().unwrap()));
                }
            },
            option { value: "none", "Select warhead..." }
            for (module_id , module) in stack
                .modules
                .iter()
                .filter(|(_, module)| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Warhead { armed: true },
                        }
                    )
                })
            {
                option { value: "{module_id}", "{module}" }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(selected_warhead) = &*selected_warhead.read() {
                    orders
                        .write()
                        .push(Order::Arm {
                            stack: id,
                            warhead: *selected_warhead,
                            armed: false,
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_warhead.read().is_none(),
            "Submit"
        }
    }
}

#[component]
fn Burn(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
    click_broker: Signal<ClickBroker>,
) -> Element {
    let mut selected_position = use_signal(|| Option::<Vec2<i32>>::None);
    let mut fuel_from = use_signal(HashMap::<ModuleId, u8>::new);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    let stack_mass = stack.mass();
    let stack_position = stack.position;
    let stack_velocity = stack.velocity;

    let required_fuel = use_memo(move || {
        if let Some(selected_position) = &*selected_position.read() {
            let delta_v = (*selected_position - (stack_position + stack_velocity)).norm() as u32;
            let delta_p = stack_mass * delta_v as f32;
            Some((delta_p / ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32).ceil() as u32)
        } else {
            None
        }
    });
    let selected_fuel = use_memo(move || fuel_from.read().values().map(|&v| v as u32).sum::<u32>());

    rsx! {
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                click_broker
                    .write()
                    .register(
                        Box::new(move |position| {
                            selected_position.set(Some(position));
                        }),
                    );
            },
            "Select ΔV"
        }
        br {}
        if let Some(target) = *selected_position.read() {
            output { "ΔV required: {(target - (stack_position + stack_velocity)).norm()} hex/turn" }
            br {}
        }
        if let Some(required) = &*required_fuel.read() {
            output { "Fuel selected: {*selected_fuel.read() as f32 / 10.0:.1}t/{*required as f32 / 10.0:.1}t" }
            br {}
        }
        h3 { "Fuel Usage" }
        for (module_id , fuel_available) in stack
            .modules
            .iter()
            .filter_map(|(module_id, module)| {
                if let Module {
                    health: Health::Intact,
                    details: ModuleDetails::Tank { fuel, .. },
                } = module {
                    if *fuel > 0 { Some((*module_id, *fuel)) } else { None }
                } else {
                    None
                }
            })
        {
            Fragment { key: "{module_id:?}",
                label { class: "form-label", r#for: "tank-{module_id:?}",
                    "Tank with {fuel_available as f32 / 10.0:.1}t fuel"
                }
                input {
                    r#type: "range",
                    class: "form-range",
                    id: "tank-{module_id:?}",
                    min: 0,
                    max: fuel_available,
                    value: "{fuel_from.read().get(&module_id).copied().unwrap_or(0)}",
                    oninput: move |e| {
                        fuel_from.write().insert(module_id, e.value().parse::<u8>().unwrap());
                    },
                }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(target) = *selected_position.read() {
                    orders
                        .write()
                        .push(Order::Burn {
                            stack: id,
                            delta_v: target - (stack_position + stack_velocity),
                            fuel_from: fuel_from
                                .read()
                                .iter()
                                .filter_map(|(module_id, amount)| {
                                    if *amount > 0 { Some((*module_id, *amount)) } else { None }
                                })
                                .collect::<Vec<_>>(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_position.read().is_none()
                || required_fuel
                    .read()
                    .is_some_and(|required| required != *selected_fuel.read()),
            "Submit"
        }
    }
}

#[component]
fn OrbitAdjust(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
    click_broker: Signal<ClickBroker>,
) -> Element {
    let mut selected_position = use_signal(|| Option::<Vec2<i32>>::None);
    let mut fuel_from = use_signal(HashMap::<ModuleId, u8>::new);
    let mut clockwise = use_signal(|| true);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    let orbiting = game_state
        .celestials
        .with_gravity()
        .find_map(|(id, celestial)| {
            if stack.orbiting(celestial) {
                Some(id)
            } else {
                None
            }
        })
        .unwrap();

    let required_fuel =
        (stack.mass() / ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32).ceil() as u32;
    let selected_fuel = use_memo(move || fuel_from.read().values().map(|&v| v as u32).sum::<u32>());

    rsx! {
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                click_broker
                    .write()
                    .register(
                        Box::new(move |position| {
                            selected_position.set(Some(position));
                        }),
                    );
            },
            "Select Position"
        }
        br {}
        label { r#for: "clockwise", class: "form-check-label", "Clockwise" }
        input {
            r#type: "checkbox",
            id: "clockwise",
            class: "form-check-input",
            checked: *clockwise.read(),
            oninput: move |e| {
                clockwise.set(e.checked());
            },
        }
        br {}
        output { "Fuel selected: {*selected_fuel.read() as f32 / 10.0:.1}t/{required_fuel as f32 / 10.0:.1}t" }
        br {}
        h3 { "Fuel Usage" }
        for (module_id , fuel_available) in stack
            .modules
            .iter()
            .filter_map(|(module_id, module)| {
                if let Module {
                    health: Health::Intact,
                    details: ModuleDetails::Tank { fuel, .. },
                } = module {
                    if *fuel > 0 { Some((*module_id, *fuel)) } else { None }
                } else {
                    None
                }
            })
        {
            Fragment { key: "{module_id:?}",
                label { class: "form-label", r#for: "tank-{module_id:?}",
                    "Tank with {fuel_available as f32 / 10.0:.1}t fuel"
                }
                input {
                    r#type: "range",
                    class: "form-range",
                    id: "tank-{module_id:?}",
                    min: 0,
                    max: fuel_available,
                    value: "{fuel_from.read().get(&module_id).copied().unwrap_or(0)}",
                    oninput: move |e| {
                        fuel_from.write().insert(module_id, e.value().parse::<u8>().unwrap());
                    },
                }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(target_position) = *selected_position.read() {
                    orders
                        .write()
                        .push(Order::OrbitAdjust {
                            stack: id,
                            around: orbiting,
                            target_position,
                            clockwise: *clockwise.read(),
                            fuel_from: fuel_from
                                .read()
                                .iter()
                                .filter_map(|(module_id, amount)| {
                                    if *amount > 0 { Some((*module_id, *amount)) } else { None }
                                })
                                .collect::<Vec<_>>(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_position.read().is_none() || required_fuel != *selected_fuel.read(),
            "Submit"
        }
    }
}

#[component]
fn Land(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
    click_broker: Signal<ClickBroker>,
) -> Element {
    let mut fuel_from = use_signal(HashMap::<ModuleId, u8>::new);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    let orbiting = game_state
        .celestials
        .with_gravity()
        .find_map(|(id, celestial)| {
            if stack.orbiting(celestial) {
                Some(id)
            } else {
                None
            }
        })
        .unwrap();

    let required_fuel =
        (stack.mass() / ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32).ceil() as u32;
    let selected_fuel = use_memo(move || fuel_from.read().values().map(|&v| v as u32).sum::<u32>());

    rsx! {
        output { "Fuel required: {*selected_fuel.read() as f32 / 10.0:.1}t/{required_fuel as f32 / 10.0:.1}t" }
        br {}
        h3 { "Fuel Usage" }
        for (module_id , fuel_available) in stack
            .modules
            .iter()
            .filter_map(|(module_id, module)| {
                if let Module {
                    health: Health::Intact,
                    details: ModuleDetails::Tank { fuel, .. },
                } = module {
                    if *fuel > 0 { Some((*module_id, *fuel)) } else { None }
                } else {
                    None
                }
            })
        {
            Fragment { key: "{module_id:?}",
                label { class: "form-label", r#for: "tank-{module_id:?}",
                    "Tank with {fuel_available as f32 / 10.0:.1}t fuel"
                }
                input {
                    r#type: "range",
                    class: "form-range",
                    id: "tank-{module_id:?}",
                    min: 0,
                    max: fuel_available,
                    value: "{fuel_from.read().get(&module_id).copied().unwrap_or(0)}",
                    oninput: move |e| {
                        fuel_from.write().insert(module_id, e.value().parse::<u8>().unwrap());
                    },
                }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                orders
                    .write()
                    .push(Order::Land {
                        stack: id,
                        on: orbiting,
                        fuel_from: fuel_from
                            .read()
                            .iter()
                            .filter_map(|(module_id, amount)| {
                                if *amount > 0 { Some((*module_id, *amount)) } else { None }
                            })
                            .collect::<Vec<_>>(),
                    });
                draft_order.set(DraftOrder::None);
            },
            disabled: required_fuel != *selected_fuel.read(),
            "Submit"
        }
    }
}

#[component]
fn TakeOff(
    id: StackId,
    game_state: ReadSignal<GameState>,
    orders: WriteSignal<Vec<Order>>,
    draft_order: WriteSignal<DraftOrder>,
    click_broker: Signal<ClickBroker>,
) -> Element {
    let mut selected_position = use_signal(|| Option::<Vec2<i32>>::None);
    let mut fuel_from = use_signal(HashMap::<ModuleId, u8>::new);
    let mut clockwise = use_signal(|| true);
    let game_state = &*game_state.read();
    let stack = &game_state.stacks[&id];
    let landed_on = game_state
        .celestials
        .with_gravity()
        .find_map(|(id, celestial)| {
            if stack.landed_with_gravity(celestial) {
                Some(id)
            } else {
                None
            }
        })
        .unwrap();

    let required_fuel =
        (stack.mass() / ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32).ceil() as u32;
    let selected_fuel = use_memo(move || fuel_from.read().values().map(|&v| v as u32).sum::<u32>());

    rsx! {
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                click_broker
                    .write()
                    .register(
                        Box::new(move |position| {
                            selected_position.set(Some(position));
                        }),
                    );
            },
            "Select Destination"
        }
        br {}
        label { r#for: "clockwise", class: "form-check-label", "Clockwise" }
        input {
            r#type: "checkbox",
            id: "clockwise",
            class: "form-check-input",
            checked: *clockwise.read(),
            oninput: move |e| {
                clockwise.set(e.checked());
            },
        }
        br {}
        output { "Fuel required: {*selected_fuel.read() as f32 / 10.0:.1}t/{required_fuel as f32 / 10.0:.1}t" }
        br {}
        h3 { "Fuel Usage" }
        for (module_id , fuel_available) in stack
            .modules
            .iter()
            .filter_map(|(module_id, module)| {
                if let Module {
                    health: Health::Intact,
                    details: ModuleDetails::Tank { fuel, .. },
                } = module {
                    if *fuel > 0 { Some((*module_id, *fuel)) } else { None }
                } else {
                    None
                }
            })
        {
            Fragment { key: "{module_id:?}",
                label { class: "form-label", r#for: "tank-{module_id:?}",
                    "Tank with {fuel_available as f32 / 10.0:.1}t fuel"
                }
                input {
                    r#type: "range",
                    class: "form-range",
                    id: "tank-{module_id:?}",
                    min: 0,
                    max: fuel_available,
                    value: "{fuel_from.read().get(&module_id).copied().unwrap_or(0)}",
                    oninput: move |e| {
                        fuel_from.write().insert(module_id, e.value().parse::<u8>().unwrap());
                    },
                }
            }
        }
        button {
            class: "btn btn-primary",
            r#type: "button",
            onclick: move |_| {
                if let Some(destination) = *selected_position.read() {
                    orders
                        .write()
                        .push(Order::TakeOff {
                            stack: id,
                            from: landed_on,
                            destination,
                            clockwise: *clockwise.read(),
                            fuel_from: fuel_from
                                .read()
                                .iter()
                                .filter_map(|(module_id, amount)| {
                                    if *amount > 0 { Some((*module_id, *amount)) } else { None }
                                })
                                .collect::<Vec<_>>(),
                        });
                    draft_order.set(DraftOrder::None);
                }
            },
            disabled: selected_position.read().is_none() || required_fuel != *selected_fuel.read(),
            "Submit"
        }
    }
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
