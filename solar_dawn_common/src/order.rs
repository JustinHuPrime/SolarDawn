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

//! Orders that may be given to stacks

use std::collections::HashMap;
#[cfg(feature = "client")]
use std::fmt::Display;

#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    GameState, Phase, PlayerId, Vec2,
    celestial::CelestialId,
    celestial::{Celestial, Resources},
    stack::{Health, Module, Stack},
    stack::{ModuleDetails, ModuleId, StackId},
};

/// An order that can be given
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(Clone, PartialEq, Eq))]
pub enum Order {
    /// Name a stack
    ///
    /// Always valid, never considered in aggregate orders
    NameStack {
        /// Stack to name
        stack: StackId,
        /// Name to set
        name: String,
    },
    /// Transfer modules from this stack to another
    ///
    /// Logistics phase
    ModuleTransfer {
        /// Stack to transfer from
        stack: StackId,
        /// Module to transfer
        module: ModuleId,
        /// Where to transfer to (another stack owned by you or a new stack)
        to: ModuleTransferTarget,
    },
    /// Forcefully dock another stack to this stack
    ///
    /// Interrupts any orders the target might have been given
    ///
    /// Also prevents other orders to this stack this phase
    ///
    /// Logistics phase
    Board {
        /// Who is doing the boarding
        stack: StackId,
        /// Who is boarded
        ///
        /// Must have no functioning habitats
        target: StackId,
    },
    /// Miner/skimmer activation
    ///
    /// Puts resources into floating resource pool
    ///
    /// Logistics phase
    Isru {
        /// Which stack
        stack: StackId,
        /// How much ore to produce, 0.1 tonnes
        ore: u32,
        /// How much water to produce, 0.1 tonnes
        water: u32,
        /// How much fuel to produce, 0.1 tonnes
        fuel: u32,
    },
    /// Transfer resources - only between your stacks
    ///
    /// Must be between floating pool and something
    ///
    /// Logistics phase
    ResourceTransfer {
        /// Stack to transfer from
        stack: StackId,
        /// Which module, if any, to transfer from
        ///
        /// If None, indicates that this a transfer from the floating pool
        from: Option<ModuleId>,
        /// Where to transfer to
        ///
        /// If to another stack, must be your stack
        to: ResourceTransferTarget,
        /// How much ore to transfer, 0.1 tonnes
        ore: u8,
        /// How much materials to transfer, 0.1 tonnes
        materials: u8,
        /// How much water to transfer, 0.1 tonnes
        water: u8,
        /// How much fuel to transfer, 0.1 tonnes
        fuel: u8,
    },
    /// Repair another module - must be your module
    ///
    /// Costs 1/10th of the module's mass in materials
    ///
    /// Logistics phase
    Repair {
        /// Which stack
        stack: StackId,
        /// Target stack to repair (could be this stack)
        ///
        /// Must be rendezvoused
        target_stack: StackId,
        /// Which module to repair
        target_module: ModuleId,
    },
    /// Refine some resources
    ///
    /// A stack may not refine more resources than it has refinery capacity
    ///
    /// Logistics phase
    Refine {
        /// Which stack
        stack: StackId,
        /// How many materials to produce, 0.1 tonnes
        materials: u8,
        /// How much fuel to produce, 0.1 tonnes
        fuel: u8,
    },
    /// Build a module
    ///
    /// May only build habitats in orbit of Earth
    ///
    /// Logistics phase
    Build {
        /// Which stack
        stack: StackId,
        /// What sort of module
        module: ModuleType,
    },
    /// Salvage a module
    ///
    /// Returns half module mass in materials
    ///
    /// Logistics phase
    Salvage {
        /// Which stack
        stack: StackId,
        /// Which module in that stack
        salvaged: ModuleId,
    },
    /// Shoot some target
    ///
    /// Combat phase
    Shoot {
        /// Which stack
        stack: StackId,
        /// Which target
        target: StackId,
        /// How many shots to take
        shots: u32,
    },
    /// Set warhead arming status
    ///
    /// Fails to arm if the warhead is on a stack with a habitat
    ///
    /// Combat phase
    Arm {
        /// Which stack
        stack: StackId,
        /// Which warhead
        warhead: ModuleId,
        /// What status
        armed: bool,
    },
    /// Change velocity
    ///
    /// Aggregate order
    ///
    /// Stack must have enough thrust to generate the requested delta-v
    ///
    /// Fuel tanks must have enough fuel to cover the fuel consumption
    ///
    /// Movement phase
    Burn {
        /// Which stack
        stack: StackId,
        /// How much change in velocity
        delta_v: Vec2<i32>,
        /// Where to draw fuel from
        fuel_from: Vec<(ModuleId, u8)>,
    },
    /// Adjust orbit to target orbital hex and direction
    ///
    /// Requires functional engine and one hex/turn of delta-v
    ///
    /// Target must be a valid orbital position around the currently orbited body
    OrbitAdjust {
        /// Which stack
        stack: StackId,
        /// Where
        around: CelestialId,
        /// Target orbital position
        target_position: Vec2<i32>,
        /// Clockwise or counterclockwise orbit direction
        clockwise: bool,
        /// Where to draw fuel from
        fuel_from: Vec<(ModuleId, u8)>,
    },
    /// Land on the body currently orbited
    ///
    /// Requires enough thrust to cover the surface gravity and one hex/turn of delta-v
    Land {
        /// Which stack
        stack: StackId,
        /// Where
        on: CelestialId,
        /// Where to draw fuel from
        fuel_from: Vec<(ModuleId, u8)>,
    },
    /// Take off from the body currently orbited
    ///
    /// Requires enough thrust to cover the surface gravity and one hex/turn of delta-v
    TakeOff {
        /// Which stack
        stack: StackId,
        /// From where
        from: CelestialId,
        /// Launching to where
        destination: Vec2<i32>,
        /// Starting orbit clockwise or counterclockwise
        clockwise: bool,
        /// Where to draw fuel from
        fuel_from: Vec<(ModuleId, u8)>,
    },
}

/// Where a transferred module should go
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(Clone, PartialEq, Eq))]
pub enum ModuleTransferTarget {
    /// An existing stack
    Existing(StackId),
    /// To the nth new stack this player is creating
    New(u32),
}

/// Where a resource transfer should go
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(Clone, PartialEq, Eq))]
pub enum ResourceTransferTarget {
    /// This stack's floating pool
    FloatingPool,
    /// Jettison
    Jettison,
    /// A module in this stack
    Module(ModuleId),
    /// That stack's floating pool
    Stack(StackId),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(PartialEq, Eq))]
#[expect(missing_docs)]
/// Type of module to build
pub enum ModuleType {
    Miner,
    FuelSkimmer,
    CargoHold,
    Tank,
    Engine,
    Warhead,
    Gun,
    Habitat,
    Refinery,
    Factory,
    ArmourPlate,
}
impl ModuleType {
    /// Cost of this module, in 0.1 tonnes of materials
    pub fn cost(&self) -> i32 {
        match self {
            ModuleType::Miner => ModuleDetails::MINER_MASS as i32 * 10,
            ModuleType::FuelSkimmer => ModuleDetails::FUEL_SKIMMER_MASS as i32 * 10,
            ModuleType::CargoHold => ModuleDetails::CARGO_HOLD_MASS as i32 * 10,
            ModuleType::Tank => ModuleDetails::TANK_MASS as i32 * 10,
            ModuleType::Engine => ModuleDetails::ENGINE_MASS as i32 * 10,
            ModuleType::Warhead => ModuleDetails::WARHEAD_MASS as i32 * 10,
            ModuleType::Gun => ModuleDetails::GUN_MASS as i32 * 10,
            ModuleType::Habitat => ModuleDetails::HABITAT_MASS as i32 * 10,
            ModuleType::Refinery => ModuleDetails::REFINERY_MASS as i32 * 10,
            ModuleType::Factory => ModuleDetails::FACTORY_MASS as i32 * 10,
            ModuleType::ArmourPlate => ModuleDetails::ARMOUR_PLATE_MASS as i32 * 10,
        }
    }
}

#[cfg(feature = "client")]
impl Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::Miner => write!(f, "a miner"),
            ModuleType::FuelSkimmer => write!(f, "a fuel skimmer"),
            ModuleType::CargoHold => write!(f, "a cargo hold"),
            ModuleType::Tank => write!(f, "a tank"),
            ModuleType::Engine => write!(f, "an engine"),
            ModuleType::Warhead => write!(f, "a warhead"),
            ModuleType::Gun => write!(f, "a gun"),
            ModuleType::Habitat => write!(f, "a habitat"),
            ModuleType::Refinery => write!(f, "a refinery"),
            ModuleType::Factory => write!(f, "a factory"),
            ModuleType::ArmourPlate => write!(f, "an armour plate"),
        }
    }
}

/// Why an order could not be applied
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(PartialEq, Eq))]
pub enum OrderError {
    /// Order issued during invalid phase
    WrongPhase,
    /// Stack id is invalid
    InvalidStackId(StackId),
    /// Module and stack id combination is invalid
    InvalidModuleId(StackId, ModuleId),
    /// Celestial id is invalid
    InvalidCelestialId(CelestialId),
    /// Module is not of the right type
    InvalidModuleType(StackId, ModuleId),
    /// Stack id is not owned by the right player
    BadOwnership(StackId),
    /// Stacks are not rendezvoused
    NotRendezvoused(StackId, StackId),
    /// Not in orbit of a body
    NotInOrbit,
    /// Target can't be landed on
    NotLandable,
    /// Not landed on a body
    NotLanded,
    /// Not enough thrust
    NotEnoughThrust,
    /// Not enough resources in module
    NotEnoughResources(StackId, ModuleId),
    /// Too much or not enough propellant burned
    IncorrectPropellantMass,

    /// Multiple naming orders for the same stack
    MultipleNamingOrders,

    /// Can't board - stack has no habitat to board with
    NoHab,
    /// Boarding contested by named stack - either by an existing habitat or another boarding attempt
    ContestedBoarding,

    /// Can't ISRU - not in the right situation to ISRU
    NoResourceAccess,

    /// Can't transfer
    ///
    /// Either invalid combination of source and destination
    ///
    /// Or transferred resources can't be put into that module
    InvalidTransfer,

    /// Module is not repairable because it's not in the damaged state
    ///
    /// Destroyed and intact modules can't be repaired
    NotDamaged,

    /// Can't build habitats while not in earth orbit
    NotInEarthOrbit,

    /// Can't shoot yourself
    InvalidTarget,
    /// No line of sight
    NoLineOfSight,

    /// Can't arm warheads on a stack with a hab
    HabOnStack,

    /// Burning while landed on a planet
    BurnWhileLanded,

    /// Destination after takeoff or orbit adjust is too far
    DestinationTooFar,

    /// Stack tried to board and do something else
    ///
    /// Something else might be another order or it might be another boarding attempt
    TooBusyToBoard,
    /// Stack was boarded; all other commands interrupted
    Boarded,
    /// Module was transferred, but was also interacted with
    ///
    /// Applies to both the module move and the other logistics order
    ModuleTransferConflict,
    /// New stack has conflicting position or velocity
    NewStackStateConflict,
    /// Not enough modules to support the wanted order
    NotEnoughModules,
    /// Residual (either positive or negative) left in the resource pool
    ResourcePoolResidual(StackId),
    /// Too many resources moved into module
    NotEnoughCapacity(StackId, ModuleId),
    /// Ordered to move multiple ways
    MultipleMoves,
}

impl OrderError {
    /// Display the order error, translating ids to names where appropriate
    #[cfg(feature = "client")]
    pub fn display(&self, game_state: &GameState) -> String {
        match self {
            OrderError::WrongPhase => "order issued in wrong phase".to_owned(),
            OrderError::InvalidStackId(stack_id) => format!("invalid stack id: {}", stack_id),
            OrderError::InvalidModuleId(stack_id, module_id) => format!(
                "invalid stack id and module id combination: {}.{}",
                stack_id, module_id
            ),
            OrderError::InvalidCelestialId(celestial_id) => {
                format!("invalid celestial body id: {}", celestial_id)
            }
            OrderError::InvalidModuleType(stack_id, module_id) => format!(
                "wrong type of module: {}",
                game_state.stacks[stack_id].modules[module_id]
            ),
            OrderError::BadOwnership(stack_id) => format!(
                "stack {} not owned by correct player",
                game_state.stacks[stack_id].name
            ),
            OrderError::NotRendezvoused(stack_id1, stack_id2) => format!(
                "stack {} and {} are not rendezvoused",
                game_state.stacks[stack_id1].name, game_state.stacks[stack_id2].name
            ),
            OrderError::NotInOrbit => "stack not in orbit".to_owned(),
            OrderError::NotLandable => "can't land on target".to_owned(),
            OrderError::NotLanded => "not landed on a celestial body".to_owned(),
            OrderError::NotEnoughThrust => "not enough thrust".to_owned(),
            OrderError::NotEnoughResources(stack_id, module_id) => format!(
                "not enough resources in {} in {}",
                game_state.stacks[stack_id].modules[module_id], game_state.stacks[stack_id].name
            ),
            OrderError::IncorrectPropellantMass => "incorrect propellant mass".to_owned(),
            OrderError::MultipleNamingOrders => "multiple naming orders".to_owned(),
            OrderError::NoHab => "stack doesn't contain a habitat".to_owned(),
            OrderError::ContestedBoarding => {
                "boarding action contested - boarding failed".to_owned()
            }
            OrderError::NoResourceAccess => "resource not available in this location".to_owned(),
            OrderError::InvalidTransfer => {
                "invalid transfer source-destination combination".to_owned()
            }
            OrderError::NotDamaged => "module not damaged".to_owned(),
            OrderError::NotInEarthOrbit => "stack not in Earth orbit".to_owned(),
            OrderError::InvalidTarget => "invalid target".to_owned(),
            OrderError::NoLineOfSight => "no line of sight to target".to_owned(),
            OrderError::HabOnStack => "stack has a habitat".to_owned(),
            OrderError::BurnWhileLanded => "cannot thrust while landed".to_owned(),
            OrderError::DestinationTooFar => "destination too far".to_owned(),
            OrderError::TooBusyToBoard => "can't board and carry out other orders".to_owned(),
            OrderError::Boarded => "stack boarded - order interrupted".to_owned(),
            OrderError::ModuleTransferConflict => {
                "module transferred involved in another order".to_owned()
            }
            OrderError::NewStackStateConflict => {
                "new stack created in multiple locations".to_owned()
            }
            OrderError::NotEnoughModules => {
                "not enough capacity for all orders of this type".to_owned()
            }
            OrderError::ResourcePoolResidual(stack_id) => format!(
                "resource pool for {} is not empty at end of turn",
                game_state.stacks[stack_id].name
            ),
            OrderError::NotEnoughCapacity(stack_id, module_id) => format!(
                "attempted to fill {} on {} beyond capacity",
                game_state.stacks[stack_id].modules[module_id], game_state.stacks[stack_id].name
            ),
            OrderError::MultipleMoves => "multiple movement orders for this stack".to_owned(),
        }
    }
}

impl Order {
    /// validates that orders can be carried out given some game state
    ///
    /// returns a token of said validity and the reasons why any invalid orders couldn't be carried out
    pub fn validate<'a>(
        game_state: &'a GameState,
        orders: &'a HashMap<PlayerId, Vec<Order>>,
    ) -> (
        ValidatedOrders<'a>,
        HashMap<PlayerId, Vec<Option<OrderError>>>,
    ) {
        let mut orders: HashMap<PlayerId, Vec<Result<&Order, OrderError>>> = orders
            .iter()
            .map(|(player, orders)| (*player, orders.iter().map(Ok).collect::<Vec<_>>()))
            .collect::<HashMap<_, _>>();

        // individual order validation
        for (&player, orders) in orders.iter_mut() {
            for order_result in orders.iter_mut() {
                let order = order_result.as_mut().unwrap();
                if let Err(e) = order.validate_single(game_state, player) {
                    *order_result = Err(e);
                }
            }
        }

        // aggregate check that only one naming order exists per stack
        let mut naming_orders_by_stack: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
            HashMap::new();
        for (_, orders) in orders.iter_mut() {
            for order in orders.iter_mut() {
                if let Ok(Order::NameStack { stack, .. }) = order {
                    naming_orders_by_stack
                        .entry(*stack)
                        .or_default()
                        .push(order);
                }
            }
        }
        for (_, orders) in naming_orders_by_stack {
            if orders.len() > 1 {
                for order in orders {
                    *order = Err(OrderError::MultipleNamingOrders);
                }
            }
        }

        // aggregate order validation
        match game_state.phase {
            Phase::Logistics => {
                // aggregate check that only one boarding attempt happens, and that it's the only order
                // aggregate check that successful boarding attempts interrupt other orders for target stack

                use std::collections::HashSet;
                let mut boarding_attempts_by_stack: HashMap<
                    StackId,
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                let mut other_orders_by_stack: HashMap<
                    StackId,
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(Order::Board { stack, .. }) => {
                                boarding_attempts_by_stack
                                    .entry(*stack)
                                    .or_default()
                                    .push(order);
                            }
                            Ok(
                                Order::ModuleTransfer { stack, .. }
                                | Order::Isru { stack, .. }
                                | Order::ResourceTransfer { stack, .. }
                                | Order::Repair { stack, .. }
                                | Order::Refine { stack, .. }
                                | Order::Build { stack, .. }
                                | Order::Salvage { stack, .. },
                            ) => {
                                other_orders_by_stack.entry(*stack).or_default().push(order);
                            }
                            _ => {
                                // no-op - should only be errors here
                            }
                        }
                    }
                }

                // check that boarding attempts are the only thing this stack does
                // also mark contested boarding attempts
                // note: don't need to deal with the case where a boarding target also does a boarding (hab presence rules already resolve this)
                for (stack, attempts) in boarding_attempts_by_stack.iter_mut() {
                    if attempts.len() > 1 || other_orders_by_stack.contains_key(stack) {
                        for attempt in attempts {
                            **attempt = Err(OrderError::TooBusyToBoard);
                        }
                    }
                }
                let mut boarded_by: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                for (_, mut attempts) in boarding_attempts_by_stack {
                    let Some(attempt @ Ok(_)) = attempts.pop() else {
                        continue;
                    };
                    let Ok(Order::Board { target, .. }) = attempt else {
                        unreachable!("pre-filtered");
                    };
                    boarded_by.entry(*target).or_default().push(attempt);
                }
                for (target, boarders) in boarded_by {
                    if boarders.len() > 1 {
                        for boarder in boarders {
                            *boarder = Err(OrderError::ContestedBoarding)
                        }
                    } else if let Some(other_orders) = other_orders_by_stack.get_mut(&target) {
                        for other_order in other_orders.iter_mut() {
                            **other_order = Err(OrderError::Boarded);
                        }
                    }
                }

                // aggregate check that no other order involves transferred module
                let mut module_moves: HashMap<
                    (StackId, ModuleId),
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                let mut other_orders: HashMap<
                    (StackId, ModuleId),
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(Order::ModuleTransfer { stack, module, .. }) => {
                                module_moves
                                    .entry((*stack, *module))
                                    .or_default()
                                    .push(order);
                            }
                            Ok(
                                Order::ResourceTransfer {
                                    stack,
                                    from: Some(module),
                                    ..
                                }
                                | Order::ResourceTransfer {
                                    stack,
                                    from: None,
                                    to: ResourceTransferTarget::Module(module),
                                    ..
                                }
                                | Order::Repair {
                                    target_stack: stack,
                                    target_module: module,
                                    ..
                                }
                                | Order::Salvage {
                                    stack,
                                    salvaged: module,
                                },
                            ) => {
                                other_orders
                                    .entry((*stack, *module))
                                    .or_default()
                                    .push(order);
                            }
                            _ => {
                                // no-op - should only be errors here
                            }
                        }
                    }
                }
                for (module, moves) in module_moves {
                    if moves.len() > 1 {
                        for move_order in moves {
                            *move_order = Err(OrderError::ModuleTransferConflict)
                        }
                    } else if let Some(other_orders) = other_orders.get_mut(&module) {
                        for other_order in other_orders.iter_mut() {
                            **other_order = Err(OrderError::ModuleTransferConflict);
                        }
                    }
                }

                // aggregate check that all transfers to new stack #n are in the same spot
                let mut new_stack_moves: HashMap<
                    (PlayerId, u32),
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                for (player, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        if let Ok(Order::ModuleTransfer {
                            to: ModuleTransferTarget::New(n),
                            ..
                        }) = order
                        {
                            new_stack_moves
                                .entry((*player, *n))
                                .or_default()
                                .push(order);
                        }
                    }
                }
                for (_, moves) in new_stack_moves {
                    let mut moves_iter = moves.iter();
                    let Some(Ok(Order::ModuleTransfer { stack, .. })) = moves_iter.next() else {
                        unreachable!("constructed with at least one")
                    };
                    let stack_ref = &game_state.stacks[stack];
                    let position = stack_ref.position;
                    let velocity = stack_ref.velocity;
                    if moves_iter.any(|move_order| {
                        let Ok(Order::ModuleTransfer { stack, .. }) = move_order else {
                            unreachable!("pre-filtered")
                        };
                        let stack_ref = &game_state.stacks[stack];
                        stack_ref.position != position || stack_ref.velocity != velocity
                    }) {
                        for move_order in moves {
                            *move_order = Err(OrderError::NewStackStateConflict);
                        }
                    }
                }

                // logistics aggregate checks

                // first, exclude moved modules from consideration
                let mut disabled_modules: HashSet<(StackId, ModuleId)> = HashSet::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(
                                Order::ModuleTransfer { stack, module, .. }
                                | Order::Salvage {
                                    stack,
                                    salvaged: module,
                                },
                            ) => {
                                disabled_modules.insert((*stack, *module));
                            }
                            _ => {
                                // no-op
                            }
                        }
                    }
                }

                // second, check individual stacks' orders for sufficient capacity
                // Isru, Repair, Refine, Build, Salvage
                let mut miner_capacity_used: HashMap<StackId, u32> = HashMap::new();
                let mut miner_orders: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                let mut skimmer_capacity_used: HashMap<StackId, u32> = HashMap::new();
                let mut skimmer_orders: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                let mut refinery_capacity_used: HashMap<StackId, u32> = HashMap::new();
                let mut refinery_orders: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                let mut hab_or_factory_orders: HashMap<
                    StackId,
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                let mut factory_only_orders: HashMap<
                    StackId,
                    Vec<&mut Result<&Order, OrderError>>,
                > = HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(Order::Isru {
                                stack,
                                ore,
                                water,
                                fuel,
                            }) => {
                                *miner_capacity_used.entry(*stack).or_default() += *ore + *water;
                                *skimmer_capacity_used.entry(*stack).or_default() += *fuel;

                                // note - both mining and skimming orders will not pass individual validation
                                if *ore > 0 || *water > 0 {
                                    miner_orders.entry(*stack).or_default().push(order);
                                } else if *fuel > 0 {
                                    skimmer_orders.entry(*stack).or_default().push(order);
                                }
                            }
                            Ok(Order::Refine {
                                stack,
                                materials,
                                fuel,
                            }) => {
                                *refinery_capacity_used.entry(*stack).or_default() +=
                                    *materials as u32 + *fuel as u32;
                                refinery_orders.entry(*stack).or_default().push(order);
                            }
                            Ok(Order::Repair { stack, .. }) => {
                                hab_or_factory_orders.entry(*stack).or_default().push(order);
                            }
                            Ok(Order::Build { stack, .. } | Order::Salvage { stack, .. }) => {
                                factory_only_orders.entry(*stack).or_default().push(order);
                            }
                            _ => {
                                // no-op
                            }
                        }
                    }
                }
                for (stack, orders) in miner_orders {
                    let stack_ref = &game_state.stacks[&stack];
                    let miner_count = stack_ref
                        .modules
                        .iter()
                        .filter(|&(&id, module)| {
                            !disabled_modules.contains(&(stack, id))
                                && matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Miner,
                                    }
                                )
                        })
                        .count();
                    let required_miners =
                        miner_capacity_used[&stack].div_ceil(ModuleDetails::MINER_PRODUCTION_RATE);
                    if required_miners as usize > miner_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in skimmer_orders {
                    let stack_ref = &game_state.stacks[&stack];
                    let skimmer_count = stack_ref
                        .modules
                        .iter()
                        .filter(|&(&id, module)| {
                            !disabled_modules.contains(&(stack, id))
                                && matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::FuelSkimmer,
                                    }
                                )
                        })
                        .count();
                    let required_skimmers = skimmer_capacity_used[&stack]
                        .div_ceil(ModuleDetails::FUEL_SKIMMER_PRODUCTION_RATE);
                    if required_skimmers as usize > skimmer_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in refinery_orders {
                    let stack_ref = &game_state.stacks[&stack];
                    let refinery_count = stack_ref
                        .modules
                        .iter()
                        .filter(|&(&id, module)| {
                            !disabled_modules.contains(&(stack, id))
                                && matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Refinery,
                                    }
                                )
                        })
                        .count();
                    let required_refineries =
                        refinery_capacity_used[&stack].div_ceil(ModuleDetails::REFINERY_CAPACITY);
                    if required_refineries as usize > refinery_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in hab_or_factory_orders {
                    let stack_ref = &game_state.stacks[&stack];
                    let hab_and_factory_count = stack_ref
                        .modules
                        .iter()
                        .filter(|&(&id, module)| {
                            !disabled_modules.contains(&(stack, id))
                                && matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Habitat { .. }
                                            | ModuleDetails::Factory
                                    }
                                )
                        })
                        .count();
                    if orders.len() + factory_only_orders.get(&stack).map(Vec::len).unwrap_or(0)
                        > hab_and_factory_count
                    {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in factory_only_orders {
                    let stack_ref = &game_state.stacks[&stack];
                    let factory_count = stack_ref
                        .modules
                        .iter()
                        .filter(|&(&id, module)| {
                            !disabled_modules.contains(&(stack, id))
                                && matches!(
                                    module,
                                    Module {
                                        health: Health::Intact,
                                        details: ModuleDetails::Factory
                                    }
                                )
                        })
                        .count();
                    if orders.len() > factory_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }

                // third, check for resource movement validity
                #[derive(Default)]
                struct Resources {
                    ore: i32,
                    materials: i32,
                    water: i32,
                    fuel: i32,
                }
                impl Resources {
                    fn empty(&self) -> bool {
                        self.ore == 0 && self.materials == 0 && self.water == 0 && self.fuel == 0
                    }
                }

                let mut resource_pools: HashMap<(PlayerId, StackId), Resources> = HashMap::new();
                let mut storage_deltas: HashMap<(PlayerId, StackId, ModuleId), Resources> =
                    HashMap::new();
                let mut orders_by_player: HashMap<PlayerId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                for (&player, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(Order::Isru {
                                stack,
                                ore,
                                water,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore += *ore as i32;
                                resource_pool.water += *water as i32;
                                resource_pool.fuel += *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::ResourceTransfer {
                                stack,
                                from: Some(from),
                                to: ResourceTransferTarget::FloatingPool,
                                ore,
                                materials,
                                water,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore += *ore as i32;
                                resource_pool.materials += *materials as i32;
                                resource_pool.water += *water as i32;
                                resource_pool.fuel += *fuel as i32;

                                let storage_delta =
                                    storage_deltas.entry((player, *stack, *from)).or_default();
                                storage_delta.ore -= *ore as i32;
                                storage_delta.materials -= *materials as i32;
                                storage_delta.water -= *water as i32;
                                storage_delta.fuel -= *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::ResourceTransfer {
                                stack,
                                from: None,
                                to: ResourceTransferTarget::Module(to),
                                ore,
                                materials,
                                water,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore -= *ore as i32;
                                resource_pool.materials -= *materials as i32;
                                resource_pool.water -= *water as i32;
                                resource_pool.fuel -= *fuel as i32;

                                let storage_delta =
                                    storage_deltas.entry((player, *stack, *to)).or_default();
                                storage_delta.ore += *ore as i32;
                                storage_delta.materials += *materials as i32;
                                storage_delta.water += *water as i32;
                                storage_delta.fuel += *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::ResourceTransfer {
                                stack,
                                from: None,
                                to: ResourceTransferTarget::Stack(to),
                                ore,
                                materials,
                                water,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore -= *ore as i32;
                                resource_pool.materials -= *materials as i32;
                                resource_pool.water -= *water as i32;
                                resource_pool.fuel -= *fuel as i32;

                                let storage_delta =
                                    resource_pools.entry((player, *to)).or_default();
                                storage_delta.ore += *ore as i32;
                                storage_delta.materials += *materials as i32;
                                storage_delta.water += *water as i32;
                                storage_delta.fuel += *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::ResourceTransfer {
                                stack,
                                from: None,
                                to: ResourceTransferTarget::Jettison,
                                ore,
                                materials,
                                water,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore -= *ore as i32;
                                resource_pool.materials -= *materials as i32;
                                resource_pool.water -= *water as i32;
                                resource_pool.fuel -= *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::Repair {
                                stack,
                                target_stack,
                                target_module,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.materials -=
                                    game_state.stacks[target_stack].modules[target_module]
                                        .dry_mass() as i32
                                        * 10
                                        / ModuleDetails::REPAIR_FRACTION;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::Refine {
                                stack,
                                materials,
                                fuel,
                            }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.ore -=
                                    *materials as i32 * ModuleDetails::REFINERY_ORE_PER_MATERIAL;
                                resource_pool.materials += *materials as i32;
                                resource_pool.water -=
                                    *fuel as i32 * ModuleDetails::REFINERY_WATER_PER_FUEL;
                                resource_pool.fuel += *fuel as i32;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::Build { stack, module }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.materials -= module.cost();

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            Ok(Order::Salvage { stack, salvaged }) => {
                                let resource_pool =
                                    resource_pools.entry((player, *stack)).or_default();
                                resource_pool.materials +=
                                    game_state.stacks[stack].modules[salvaged].dry_mass() as i32
                                        * 10
                                        / ModuleDetails::SALVAGE_FRACTION;

                                orders_by_player.entry(player).or_default().push(order);
                            }
                            _ => {
                                // no-op
                            }
                        }
                    }
                }
                for ((player, stack), delta) in resource_pools {
                    if !delta.empty() {
                        for order in orders_by_player.get_mut(&player).unwrap().iter_mut() {
                            **order = Err(OrderError::ResourcePoolResidual(stack))
                        }
                        break;
                    }
                }
                for ((player, stack, module), delta) in storage_deltas {
                    let module_ref = &game_state.stacks[&stack].modules[&module];
                    let problem = match module_ref {
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::CargoHold { ore, materials },
                        } => {
                            // note: delta only involves relevant resources due to individual validation
                            if *ore as i32 + *materials as i32 + delta.ore + delta.materials
                                > ModuleDetails::CARGO_HOLD_CAPACITY
                            {
                                Some(OrderError::NotEnoughCapacity(stack, module))
                            } else if *ore as i32 + delta.ore < 0
                                || *materials as i32 + delta.materials < 0
                            {
                                Some(OrderError::NotEnoughResources(stack, module))
                            } else {
                                None
                            }
                        }
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Tank { water, fuel },
                        } => {
                            // note: delta only involves relevant resources due to individual validation
                            if *water as i32 + *fuel as i32 + delta.water + delta.fuel
                                > ModuleDetails::TANK_CAPACITY
                            {
                                Some(OrderError::NotEnoughCapacity(stack, module))
                            } else if *water as i32 + delta.water < 0
                                || *fuel as i32 + delta.fuel < 0
                            {
                                Some(OrderError::NotEnoughResources(stack, module))
                            } else {
                                None
                            }
                        }
                        _ => unreachable!("order id validated"),
                    };
                    if let Some(problem) = problem {
                        for order in orders_by_player.get_mut(&player).unwrap().iter_mut() {
                            **order = Err(problem);
                        }
                    }
                }
            }
            Phase::Combat => {
                // aggregate check that there's enough guns to shoot all the targets
                let mut orders_by_stack: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        if let Ok(Order::Shoot { stack, .. }) = order {
                            orders_by_stack.entry(*stack).or_default().push(order);
                        }
                    }
                }
                for (stack, orders) in orders_by_stack {
                    let total_targets: u32 = orders
                        .iter()
                        .map(|order| {
                            let Ok(Order::Shoot { shots, .. }) = order else {
                                unreachable!("pre-filtered");
                            };
                            *shots
                        })
                        .sum();
                    let stack_ref = &game_state.stacks[&stack];
                    let total_guns = stack_ref
                        .modules
                        .values()
                        .filter(|module| {
                            matches!(
                                module,
                                Module {
                                    health: Health::Intact,
                                    details: ModuleDetails::Gun
                                }
                            )
                        })
                        .count();
                    if total_targets as usize > total_guns {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
            }
            Phase::Movement => {
                // aggregate check that there's only one move order per stack
                let mut orders_by_stack: HashMap<StackId, Vec<&mut Result<&Order, OrderError>>> =
                    HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(
                                Order::Burn { stack, .. }
                                | Order::OrbitAdjust { stack, .. }
                                | Order::Land { stack, .. }
                                | Order::TakeOff { stack, .. },
                            ) => {
                                orders_by_stack.entry(*stack).or_default().push(order);
                            }
                            _ => {
                                // no-op - should only be errors here
                            }
                        }
                    }
                }
                for (_, orders) in orders_by_stack {
                    if orders.len() > 1 {
                        for order in orders {
                            *order = Err(OrderError::MultipleMoves);
                        }
                    }
                }
            }
        }

        (
            ValidatedOrders {
                orders: orders
                    .iter()
                    .map(|(player, orders)| {
                        (
                            *player,
                            orders
                                .iter()
                                .filter_map(|order| match order {
                                    Ok(order) => Some(*order),
                                    Err(_) => None,
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
                game_state,
            },
            orders
                .into_iter()
                .map(|(player, orders)| {
                    (
                        player,
                        orders
                            .into_iter()
                            .map(|order| order.err())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<HashMap<_, _>>(),
        )
    }

    /// Validate all that can be validated for a single order
    fn validate_single(&self, game_state: &GameState, player: PlayerId) -> Result<(), OrderError> {
        fn validate_stack(
            stack: StackId,
            game_state: &GameState,
            player: PlayerId,
        ) -> Result<&Stack, OrderError> {
            let Some(stack_ref) = game_state.stacks.get(&stack) else {
                return Err(OrderError::InvalidStackId(stack));
            };

            if stack_ref.owner != player {
                return Err(OrderError::BadOwnership(stack));
            }

            Ok(stack_ref)
        }

        fn validate_module(
            stack: StackId,
            module: ModuleId,
            game_state: &GameState,
            player: PlayerId,
        ) -> Result<(&Stack, &Module), OrderError> {
            let stack_ref = validate_stack(stack, game_state, player)?;
            let Some(module_ref) = stack_ref.modules.get(&module) else {
                return Err(OrderError::InvalidModuleId(stack, module));
            };

            Ok((stack_ref, module_ref))
        }

        fn validate_phase(game_state: &GameState, phase: Phase) -> Result<(), OrderError> {
            if game_state.phase != phase {
                return Err(OrderError::WrongPhase);
            }

            Ok(())
        }

        fn validate_celestial(
            celestial: CelestialId,
            game_state: &GameState,
        ) -> Result<&Celestial, OrderError> {
            let Some(celestial_ref) = game_state.celestials.get(celestial) else {
                return Err(OrderError::InvalidCelestialId(celestial));
            };

            Ok(celestial_ref)
        }

        fn validate_burn(
            stack: StackId,
            stack_ref: &Stack,
            delta_v: u32,
            fuel_from: &Vec<(ModuleId, u8)>,
            gravity_min_accel: f32,
        ) -> Result<(), OrderError> {
            let min_accel = f32::max(delta_v as f32 * 2.0, gravity_min_accel);

            // F = ma; a = F/m
            // Units of m/s^2 or equivalently 0.5 hex/turn^2
            let mass = stack_ref.mass();

            if stack_ref.acceleration() < min_accel {
                return Err(OrderError::NotEnoughThrust);
            }

            let mut total_propellant_mass = 0;
            for &(module, amount) in fuel_from {
                let Some(module_ref) = stack_ref.modules.get(&module) else {
                    return Err(OrderError::InvalidModuleId(stack, module));
                };
                let Module {
                    health: Health::Intact,
                    details: ModuleDetails::Tank { fuel, .. },
                } = module_ref
                else {
                    return Err(OrderError::InvalidModuleType(stack, module));
                };
                if amount > *fuel {
                    return Err(OrderError::NotEnoughResources(stack, module));
                }
                total_propellant_mass += amount as u32;
            }

            // Δp = m·Δv
            // Units of tonne·hex/turn
            let delta_p = mass * delta_v as f32;
            let required_propellant_mass =
                (delta_p / ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32).ceil() as u32;
            if total_propellant_mass != required_propellant_mass {
                return Err(OrderError::IncorrectPropellantMass);
            }

            Ok(())
        }

        match self {
            Order::NameStack { stack, .. } => {
                validate_stack(*stack, game_state, player)?;
                // aggregate check that there are no other naming orders for this stack
            }
            Order::ModuleTransfer { stack, module, to } => {
                validate_phase(game_state, Phase::Logistics)?;
                let (stack_ref, _) = validate_module(*stack, *module, game_state, player)?;
                match to {
                    ModuleTransferTarget::Existing(target) => {
                        let target_ref = validate_stack(*target, game_state, player)?;
                        if *stack == *target {
                            return Err(OrderError::InvalidTarget);
                        }
                        if !stack_ref.rendezvoused_with(target_ref) {
                            return Err(OrderError::NotRendezvoused(*stack, *target));
                        }
                    }
                    ModuleTransferTarget::New(_) => {
                        // no-op check at the stack level
                        // aggregate check that all transfers to new stack #n are in the same spot
                    }
                }
                // aggregate check that no other order involves transferred module
            }
            Order::Board { stack, target } => {
                validate_phase(game_state, Phase::Logistics)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;

                if !stack_ref.modules.values().any(|module| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Habitat { .. }
                        }
                    )
                }) {
                    return Err(OrderError::NoHab);
                }

                let Some(target_ref) = game_state.stacks.get(target) else {
                    return Err(OrderError::InvalidStackId(*target));
                };
                if target_ref.owner == player {
                    return Err(OrderError::BadOwnership(*target));
                }
                if !stack_ref.rendezvoused_with(target_ref) {
                    return Err(OrderError::NotRendezvoused(*stack, *target));
                }

                if target_ref.modules.values().any(|module| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Habitat { .. }
                        }
                    )
                }) {
                    return Err(OrderError::ContestedBoarding);
                }

                // aggregate check that only one boarding attempt happens, and that it's the only order
                // aggregate check that successful boarding attempts interrupt other orders for target stack
            }
            Order::Isru {
                stack,
                ore,
                water,
                fuel,
            } => {
                validate_phase(game_state, Phase::Logistics)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;

                if *ore > 0
                    && !game_state
                        .celestials
                        .get_by_position(stack_ref.position)
                        .is_some_and(|(_, celestial)| {
                            stack_ref.landed(celestial)
                                && matches!(
                                    celestial.resources,
                                    Resources::MiningBoth | Resources::MiningOre
                                )
                        })
                {
                    return Err(OrderError::NoResourceAccess);
                }
                if *water > 0
                    && !game_state
                        .celestials
                        .get_by_position(stack_ref.position)
                        .is_some_and(|(_, celestial)| {
                            stack_ref.landed(celestial)
                                && matches!(
                                    celestial.resources,
                                    Resources::MiningBoth | Resources::MiningWater
                                )
                        })
                {
                    return Err(OrderError::NoResourceAccess);
                }
                if *fuel > 0
                    && !game_state.celestials.with_gravity().any(|celestial| {
                        stack_ref.orbiting(celestial)
                            && matches!(celestial.resources, Resources::Skimming)
                    })
                {
                    return Err(OrderError::NoResourceAccess);
                }

                // aggregate check that there's enough ISRU modules
            }
            Order::ResourceTransfer {
                stack,
                from,
                to,
                ore,
                materials,
                water,
                fuel,
            } => {
                validate_phase(game_state, Phase::Logistics)?;
                match (from, to) {
                    (Some(module), ResourceTransferTarget::FloatingPool)
                    | (None, ResourceTransferTarget::Module(module)) => {
                        let (_, module_ref) = validate_module(*stack, *module, game_state, player)?;
                        if (*ore != 0 || *materials != 0) && (*water != 0 || *fuel != 0) {
                            // has to be invalid - moving both solids and liquids
                            return Err(OrderError::InvalidModuleType(*stack, *module));
                        } else if *ore != 0 || *materials != 0 {
                            if !matches!(
                                module_ref,
                                Module {
                                    health: Health::Intact,
                                    details: ModuleDetails::CargoHold { .. },
                                }
                            ) {
                                return Err(OrderError::InvalidModuleType(*stack, *module));
                            }
                        } else if *water != 0 || *fuel != 0 {
                            if !matches!(
                                module_ref,
                                Module {
                                    health: Health::Intact,
                                    details: ModuleDetails::Tank { .. },
                                }
                            ) {
                                return Err(OrderError::InvalidModuleType(*stack, *module));
                            }
                        } else {
                            // still validate the case where no resources move
                            if !matches!(
                                module_ref,
                                Module {
                                    health: Health::Intact,
                                    details: ModuleDetails::CargoHold { .. }
                                        | ModuleDetails::Tank { .. }
                                }
                            ) {
                                return Err(OrderError::InvalidModuleType(*stack, *module));
                            }
                        }
                    }
                    (None, ResourceTransferTarget::Stack(to)) => {
                        let stack_ref = validate_stack(*stack, game_state, player)?;
                        let to_ref = validate_stack(*stack, game_state, player)?;
                        if *stack == *to {
                            return Err(OrderError::InvalidTarget);
                        }
                        if !stack_ref.rendezvoused_with(to_ref) {
                            return Err(OrderError::NotRendezvoused(*stack, *to));
                        }
                    }
                    (None, ResourceTransferTarget::Jettison) => {
                        validate_stack(*stack, game_state, player)?;
                    }
                    _ => {
                        return Err(OrderError::InvalidTransfer);
                    }
                }
                // aggregate check that all modules end up with a valid quantity of resources
            }
            Order::Repair {
                stack,
                target_stack,
                target_module,
            } => {
                validate_phase(game_state, Phase::Logistics)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                let (target_stack_ref, module_ref) =
                    validate_module(*target_stack, *target_module, game_state, player)?;
                if !stack_ref.rendezvoused_with(target_stack_ref) {
                    return Err(OrderError::NotRendezvoused(*stack, *target_stack));
                }
                if !matches!(module_ref.health, Health::Damaged) {
                    return Err(OrderError::NotDamaged);
                }
                // aggregate check that enough repair sources are present and not otherwise engaged, and that enough materials are in the floating pool
            }
            Order::Refine { stack, .. } => {
                validate_phase(game_state, Phase::Logistics)?;
                validate_stack(*stack, game_state, player)?;
                // aggregate logistics checks, refinery quantity check
            }
            Order::Build { stack, module } => {
                validate_phase(game_state, Phase::Logistics)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                if matches!(module, ModuleType::Habitat) {
                    let earth = game_state.celestials.get(game_state.earth).unwrap();
                    if !stack_ref.orbiting(earth) {
                        return Err(OrderError::NotInEarthOrbit);
                    }
                }
                // aggregate logistics checks, factory quantity check
            }
            Order::Salvage { stack, salvaged } => {
                validate_phase(game_state, Phase::Logistics)?;
                validate_module(*stack, *salvaged, game_state, player)?;
                // aggregate logistics check, factory quantity check (salvaged factories not included)
            }
            Order::Shoot { stack, target, .. } => {
                validate_phase(game_state, Phase::Combat)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                let Some(target_ref) = game_state.stacks.get(target) else {
                    return Err(OrderError::InvalidStackId(*target));
                };
                if *stack == *target {
                    return Err(OrderError::InvalidTarget);
                }
                if game_state.celestials.with_gravity().any(|celestial| {
                    celestial.blocks_weapons_effect(
                        stack_ref.position.cartesian(),
                        target_ref.position.cartesian(),
                    )
                }) {
                    return Err(OrderError::NoLineOfSight);
                }
            }
            Order::Arm { stack, warhead, .. } => {
                validate_phase(game_state, Phase::Combat)?;
                let (stack_ref, module_ref) =
                    validate_module(*stack, *warhead, game_state, player)?;
                if stack_ref.modules.values().any(|module| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact | Health::Damaged,
                            details: ModuleDetails::Habitat { .. }
                        }
                    )
                }) {
                    return Err(OrderError::HabOnStack);
                }
                if !matches!(
                    module_ref,
                    Module {
                        health: Health::Intact,
                        details: ModuleDetails::Warhead { .. }
                    }
                ) {
                    return Err(OrderError::InvalidModuleType(*stack, *warhead));
                }
            }
            Order::Burn {
                stack,
                delta_v,
                fuel_from,
            } => {
                validate_phase(game_state, Phase::Movement)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;

                validate_burn(
                    *stack,
                    stack_ref,
                    delta_v.norm().try_into().unwrap(),
                    fuel_from,
                    0.0,
                )?;

                if game_state
                    .celestials
                    .with_gravity()
                    .any(|celestial| stack_ref.landed_with_gravity(celestial))
                {
                    return Err(OrderError::BurnWhileLanded);
                }

                // aggregate check that there's only one move order per stack
            }
            Order::OrbitAdjust {
                stack,
                around,
                target_position,
                fuel_from,
                ..
            } => {
                validate_phase(game_state, Phase::Movement)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;

                validate_burn(*stack, stack_ref, 1, fuel_from, 0.0)?;

                let orbited = validate_celestial(*around, game_state)?;
                if !stack_ref.orbiting(orbited) {
                    return Err(OrderError::NotInOrbit);
                };

                // Check that target_position is a valid orbital position (distance 1 from orbited body)
                if (*target_position - orbited.position).norm() != 1 {
                    return Err(OrderError::DestinationTooFar);
                }

                // aggregate check that there's only one move order per stack
            }
            Order::Land {
                stack,
                on,
                fuel_from,
            } => {
                validate_phase(game_state, Phase::Movement)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                let celestial = validate_celestial(*on, game_state)?;
                if !celestial.orbit_gravity {
                    return Err(OrderError::NotLandable);
                }

                validate_burn(*stack, stack_ref, 1, fuel_from, celestial.surface_gravity)?;

                if !stack_ref.orbiting(celestial) {
                    return Err(OrderError::NotInOrbit);
                }

                if !celestial.can_land() {
                    return Err(OrderError::NotLandable);
                }

                // aggregate check that there's only one move order per stack
            }
            Order::TakeOff {
                stack,
                from,
                destination,
                fuel_from,
                ..
            } => {
                validate_phase(game_state, Phase::Movement)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                let celestial = validate_celestial(*from, game_state)?;

                validate_burn(*stack, stack_ref, 1, fuel_from, celestial.surface_gravity)?;

                if !stack_ref.landed_with_gravity(celestial) {
                    return Err(OrderError::NotLanded);
                }

                if (*destination - celestial.position).norm() != 1 {
                    return Err(OrderError::DestinationTooFar);
                }

                // aggregate check that there's only one move order per stack
            }
        }

        Ok(())
    }

    /// apply order to stacks state
    ///
    /// note: internal-only; call only on validated orders
    #[expect(clippy::too_many_arguments)]
    #[cfg(feature = "server")]
    fn apply(
        &self,
        stacks: &mut HashMap<StackId, Stack>,
        player: PlayerId,
        new_stack_ids: &mut HashMap<(PlayerId, u32), StackId>,
        game_state: &GameState,
        stack_id_generator: &mut impl Iterator<Item = StackId>,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
        rng: &mut impl Rng,
    ) {
        fn drain_fuel(stack: &mut Stack, fuel_from: &Vec<(ModuleId, u8)>) {
            for (module, quantity) in fuel_from {
                match stack.modules.get_mut(module).unwrap() {
                    Module {
                        details: ModuleDetails::Tank { fuel, .. },
                        ..
                    } => {
                        // note: not a wrapping sub since the tank never fills during the movement phase
                        *fuel -= quantity;
                    }
                    _ => {
                        panic!("order is validated, but was invalid upon application")
                    }
                }
            }
        }

        match self {
            Order::NameStack { stack, name } => {
                stacks.get_mut(stack).unwrap().name = name.clone();
            }
            Order::ModuleTransfer { stack, module, to } => {
                let stack = stacks.get_mut(stack).unwrap();
                let (module_id, module) = stack.modules.remove_entry(module).unwrap();
                match to {
                    ModuleTransferTarget::Existing(stack_id) => {
                        stacks
                            .get_mut(stack_id)
                            .unwrap()
                            .modules
                            .insert(module_id, module);
                    }
                    ModuleTransferTarget::New(new_count) => {
                        let new_id = *new_stack_ids
                            .entry((player, *new_count))
                            .or_insert_with(|| stack_id_generator.next().unwrap());
                        let position = stack.position;
                        let velocity = stack.velocity;
                        let owner = stack.owner;
                        let new_stack = stacks.entry(new_id).or_insert_with(|| {
                            Stack::new(position, velocity, owner, format!("New Stack #{new_count}"))
                        });
                        new_stack.modules.insert(module_id, module);
                    }
                }
            }
            Order::Board { stack, target } => {
                let transferred = std::mem::take(&mut stacks.get_mut(target).unwrap().modules);
                stacks.get_mut(stack).unwrap().modules.extend(transferred);
            }
            Order::Isru { .. } | Order::Refine { .. } => {
                // no-op - creates resources in pool
            }
            Order::ResourceTransfer {
                stack,
                from,
                to,
                ore,
                materials,
                water,
                fuel,
            } => {
                let stack = stacks.get_mut(stack).unwrap();
                match (from, to) {
                    (Some(from), ResourceTransferTarget::FloatingPool) => {
                        let from = stack.modules.get_mut(from).unwrap();
                        match from {
                            Module {
                                details:
                                    ModuleDetails::CargoHold {
                                        ore: hold_ore,
                                        materials: hold_materials,
                                    },
                                ..
                            } => {
                                *hold_ore = hold_ore.wrapping_sub(*ore);
                                *hold_materials = hold_materials.wrapping_sub(*materials);
                            }
                            Module {
                                details:
                                    ModuleDetails::Tank {
                                        water: tank_water,
                                        fuel: tank_fuel,
                                    },
                                ..
                            } => {
                                *tank_water = tank_water.wrapping_sub(*water);
                                *tank_fuel = tank_fuel.wrapping_sub(*fuel);
                            }
                            _ => unreachable!("order is validated"),
                        }
                    }
                    (None, ResourceTransferTarget::Module(to)) => {
                        let to = stack.modules.get_mut(to).unwrap();
                        match to {
                            Module {
                                details:
                                    ModuleDetails::CargoHold {
                                        ore: hold_ore,
                                        materials: hold_materials,
                                    },
                                ..
                            } => {
                                *hold_ore = hold_ore.wrapping_add(*ore);
                                *hold_materials = hold_materials.wrapping_add(*materials);
                            }
                            Module {
                                details:
                                    ModuleDetails::Tank {
                                        water: tank_water,
                                        fuel: tank_fuel,
                                    },
                                ..
                            } => {
                                *tank_water = tank_water.wrapping_add(*water);
                                *tank_fuel = tank_fuel.wrapping_add(*fuel);
                            }
                            _ => unreachable!("order is validated"),
                        }
                    }
                    (None, ResourceTransferTarget::Stack(_))
                    | (None, ResourceTransferTarget::Jettison) => {
                        // no-op - removes resources from pool
                    }
                    _ => unreachable!("order is validated"),
                }
            }
            Order::Repair {
                target_stack,
                target_module,
                ..
            } => {
                stacks
                    .get_mut(target_stack)
                    .unwrap()
                    .modules
                    .get_mut(target_module)
                    .unwrap()
                    .health = Health::Intact;
            }
            Order::Build { stack, module } => {
                let stack = stacks.get_mut(stack).unwrap();
                stack.modules.insert(
                    module_id_generator.next().unwrap(),
                    match module {
                        ModuleType::Miner => Module::new_miner(),
                        ModuleType::FuelSkimmer => Module::new_fuel_skimmer(),
                        ModuleType::CargoHold => Module::new_cargo_hold(),
                        ModuleType::Tank => Module::new_tank(),
                        ModuleType::Engine => Module::new_engine(),
                        ModuleType::Warhead => Module::new_warhead(),
                        ModuleType::Gun => Module::new_gun(),
                        ModuleType::Habitat => Module::new_habitat(stack.owner),
                        ModuleType::Refinery => Module::new_refinery(),
                        ModuleType::Factory => Module::new_factory(),
                        ModuleType::ArmourPlate => Module::new_armour_plate(),
                    },
                );
            }
            Order::Salvage { stack, salvaged } => {
                stacks
                    .get_mut(stack)
                    .unwrap()
                    .modules
                    .remove(salvaged)
                    .unwrap();
            }
            Order::Shoot {
                stack,
                target,
                shots,
            } => {
                let start_pos = stacks[stack].position;
                let target = stacks.get_mut(target).unwrap();
                let hit_probability = ModuleDetails::GUN_RANGE_ONE_HIT_CHANCE
                    .powi((target.position - start_pos).norm());

                let mut hits: u32 = 0;
                for _ in 0..*shots {
                    if rng.random::<f32>() < hit_probability {
                        hits += 1;
                    }
                }

                target.do_damage(hits, rng);
            }
            Order::Arm {
                stack,
                warhead,
                armed,
            } => {
                match stacks
                    .get_mut(stack)
                    .unwrap()
                    .modules
                    .get_mut(warhead)
                    .unwrap()
                {
                    Module {
                        details:
                            ModuleDetails::Warhead {
                                armed: warhead_armed,
                            },
                        ..
                    } => {
                        *warhead_armed = *armed;
                    }
                    _ => {
                        unreachable!("order is validated")
                    }
                }
            }
            Order::Burn {
                stack,
                delta_v,
                fuel_from,
            } => {
                let stack = stacks.get_mut(stack).unwrap();
                stack.velocity += *delta_v;

                drain_fuel(stack, fuel_from);
            }
            Order::OrbitAdjust {
                stack,
                around,
                target_position,
                clockwise,
                fuel_from,
            } => {
                let stack = stacks.get_mut(stack).unwrap();

                // Find the orbited body to get correct velocity for target position
                let orbited = game_state.celestials.get(*around).unwrap();

                // Get the correct velocity for the target position
                let orbit_params = orbited.orbit_parameters(*clockwise);
                let (_, target_velocity) = orbit_params
                    .into_iter()
                    .find(|(pos, _)| pos == target_position)
                    .unwrap();

                stack.position = *target_position;
                stack.velocity = target_velocity;

                drain_fuel(stack, fuel_from);
            }
            Order::Land {
                stack,
                on,
                fuel_from,
            } => {
                let stack = stacks.get_mut(stack).unwrap();
                let on = game_state.celestials.get(*on).unwrap();
                stack.position = on.position;
                stack.velocity = Vec2::zero();

                drain_fuel(stack, fuel_from);
            }
            Order::TakeOff {
                stack,
                from,
                destination,
                clockwise,
                fuel_from,
            } => {
                let stack = stacks.get_mut(stack).unwrap();
                let (position, velocity) = game_state
                    .celestials
                    .get(*from)
                    .unwrap()
                    .orbit_parameters(*clockwise)
                    .into_iter()
                    .find(|(position, _)| position == destination)
                    .unwrap();
                stack.position = position;
                stack.velocity = velocity;

                drain_fuel(stack, fuel_from);
            }
        }
    }

    /// Get the target of the order - which stack is carrying out the order
    #[cfg(feature = "client")]
    pub fn target(&self) -> StackId {
        match self {
            Order::NameStack { stack, .. }
            | Order::ModuleTransfer { stack, .. }
            | Order::Board { stack, .. }
            | Order::Isru { stack, .. }
            | Order::ResourceTransfer { stack, .. }
            | Order::Repair { stack, .. }
            | Order::Refine { stack, .. }
            | Order::Build { stack, .. }
            | Order::Salvage { stack, .. }
            | Order::Shoot { stack, .. }
            | Order::Arm { stack, .. }
            | Order::Burn { stack, .. }
            | Order::OrbitAdjust { stack, .. }
            | Order::Land { stack, .. }
            | Order::TakeOff { stack, .. } => *stack,
        }
    }

    #[cfg(feature = "client")]
    fn format_resources(ore: u32, materials: u32, water: u32, fuel: u32) -> String {
        match (ore, materials, water, fuel) {
            (0, 0, 0, 0) => "nothing".to_owned(),
            (ore, 0, 0, 0) => format!("{:.1}t ore", ore as f32 / 10.0),
            (0, materials, 0, 0) => format!("{:.1}t materials", materials as f32 / 10.0),
            (ore, materials, 0, 0) => format!(
                "{:.1}t ore and {:.1}t materials",
                ore as f32 / 10.0,
                materials as f32 / 10.0
            ),
            (0, 0, water, 0) => format!("{:.1}t water", water as f32 / 10.0),
            (ore, 0, water, 0) => format!(
                "{:.1}t ore and {:.1}t water",
                ore as f32 / 10.0,
                water as f32 / 10.0
            ),
            (0, materials, water, 0) => format!(
                "{:.1}t materials and {:.1}t water",
                materials as f32 / 10.0,
                water as f32 / 10.0
            ),
            (ore, materials, water, 0) => format!(
                "{:.1}t ore, {:.1}t materials, and {:.1}t water",
                ore as f32 / 10.0,
                materials as f32 / 10.0,
                water as f32 / 10.0
            ),
            (0, 0, 0, fuel) => format!("{:.1}t fuel", fuel as f32 / 10.0),
            (ore, 0, 0, fuel) => format!(
                "{:.1}t ore and {:.1}t fuel",
                ore as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (0, materials, 0, fuel) => format!(
                "{:.1}t materials and {:.1}t fuel",
                materials as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (ore, materials, 0, fuel) => format!(
                "{:.1}t ore, {:.1}t materials, and {:.1}t fuel",
                ore as f32 / 10.0,
                materials as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (0, 0, water, fuel) => format!(
                "{:.1}t water and {:.1}t fuel",
                water as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (ore, 0, water, fuel) => format!(
                "{:.1}t ore, {:.1}t water, and {:.1}t fuel",
                ore as f32 / 10.0,
                water as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (0, materials, water, fuel) => format!(
                "{:.1}t materials, {:.1}t water, and {:.1}t fuel",
                materials as f32 / 10.0,
                water as f32 / 10.0,
                fuel as f32 / 10.0
            ),
            (ore, materials, water, fuel) => format!(
                "{:.1}t ore, {:.1}t materials, {:.1}t water, and {:.1}t fuel",
                ore as f32 / 10.0,
                materials as f32 / 10.0,
                water as f32 / 10.0,
                fuel as f32 / 10.0
            ),
        }
    }

    /// Displays like
    ///
    /// Do thing
    #[cfg(feature = "client")]
    pub fn display_unattributed(&self, game_state: &GameState) -> String {
        match self {
            Order::NameStack { name, .. } => {
                format!("Rename to {name}")
            }
            Order::ModuleTransfer { stack, module, to } => {
                let stack = &game_state.stacks[stack];
                match to {
                    ModuleTransferTarget::Existing(stack_id) => format!(
                        "Transfer a {} to {}",
                        stack.modules[module], game_state.stacks[stack_id].name
                    ),
                    ModuleTransferTarget::New(n) => {
                        format!("Transfer a {} to new stack #{}", stack.modules[module], n)
                    }
                }
            }
            Order::Board { target, .. } => format!("Board {}", game_state.stacks[target].name),
            Order::Isru {
                ore, water, fuel, ..
            } => match (*ore, *water, *fuel) {
                (ore, 0, 0) => format!("Mine {:.1}t ore", ore as f32 / 10.0),
                (0, water, 0) => format!("Mine {:.1}t water", water as f32 / 10.0),
                (ore, water, 0) => format!(
                    "Mine {:.1}t ore and {:.1}t water",
                    ore as f32 / 10.0,
                    water as f32 / 10.0
                ),
                (0, 0, fuel) => format!("Skim {:.1}t fuel", fuel as f32 / 10.0,),
                _ => "Invalid ISRU order".to_owned(),
            },
            Order::ResourceTransfer {
                stack,
                from,
                to,
                ore,
                materials,
                water,
                fuel,
            } => match (from, to) {
                (None, ResourceTransferTarget::Jettison) => format!(
                    "Jettison {}",
                    Self::format_resources(
                        *ore as u32,
                        *materials as u32,
                        *water as u32,
                        *fuel as u32
                    )
                ),
                (None, ResourceTransferTarget::Module(module_id)) => {
                    format!(
                        "Transfer {} to {}",
                        Self::format_resources(
                            *ore as u32,
                            *materials as u32,
                            *water as u32,
                            *fuel as u32
                        ),
                        game_state.stacks[stack].modules[module_id]
                    )
                }
                (None, ResourceTransferTarget::Stack(stack_id)) => format!(
                    "Transfer {} to {}",
                    Self::format_resources(
                        *ore as u32,
                        *materials as u32,
                        *water as u32,
                        *fuel as u32
                    ),
                    game_state.stacks[stack_id].name
                ),
                (Some(module), ResourceTransferTarget::FloatingPool) => {
                    format!(
                        "Transfer {} from {}",
                        Self::format_resources(
                            *ore as u32,
                            *materials as u32,
                            *water as u32,
                            *fuel as u32
                        ),
                        game_state.stacks[stack].modules[module]
                    )
                }
                _ => "Invalid transfer order".to_owned(),
            },
            Order::Repair {
                stack,
                target_stack,
                target_module,
            } if stack == target_stack => format!(
                "Repair {}",
                game_state.stacks[target_stack].modules[target_module]
            ),
            Order::Repair {
                target_stack,
                target_module,
                ..
            } => format!(
                "Repair {} on {}",
                game_state.stacks[target_stack].modules[target_module],
                game_state.stacks[target_stack].name
            ),
            Order::Refine {
                materials, fuel, ..
            } => format!(
                "Produce {}",
                Self::format_resources(0, *materials as u32, 0, *fuel as u32)
            ),
            Order::Build { module, .. } => {
                format!("Build {}", module)
            }
            Order::Salvage { stack, salvaged } => {
                format!("Salvage {}", game_state.stacks[stack].modules[salvaged])
            }
            Order::Shoot { target, shots, .. } if *shots == 1 => {
                format!("Shoot {}", game_state.stacks[target].name)
            }
            Order::Shoot { target, shots, .. } => {
                format!("Shoot {} {} times", game_state.stacks[target].name, shots)
            }
            Order::Arm {
                stack,
                warhead,
                armed,
            } if *armed => format!("Arm {}", game_state.stacks[stack].modules[warhead]),
            Order::Arm { stack, warhead, .. } => {
                format!("Disarm {}", game_state.stacks[stack].modules[warhead])
            }
            Order::Burn { delta_v, .. } => format!("Make a {} hex/turn burn", delta_v.norm()),
            Order::OrbitAdjust { around, .. } => format!(
                "Adjust orbit around {}",
                game_state.celestials.get(*around).unwrap().name
            ),
            Order::Land { on, .. } => {
                format!("Land on {}", game_state.celestials.get(*on).unwrap().name)
            }
            Order::TakeOff { from, .. } => format!(
                "Take off from {}",
                game_state.celestials.get(*from).unwrap().name
            ),
        }
    }

    /// Displays like
    ///
    /// stack_name: do thing
    #[cfg(feature = "client")]
    pub fn display_attributed(&self, game_state: &GameState) -> String {
        let stack_name = &game_state.stacks[&self.target()].name;
        match self {
            Order::NameStack { name, .. } => {
                format!("{stack_name}: Rename to {name}")
            }
            Order::ModuleTransfer { stack, module, to } => {
                let stack = &game_state.stacks[stack];
                match to {
                    ModuleTransferTarget::Existing(stack_id) => format!(
                        "{stack_name}: Transfer a {} to {}",
                        stack.modules[module], game_state.stacks[stack_id].name
                    ),
                    ModuleTransferTarget::New(n) => {
                        format!(
                            "{stack_name}: Transfer a {} to new stack #{}",
                            stack.modules[module], n
                        )
                    }
                }
            }
            Order::Board { target, .. } => {
                format!("{stack_name}: Board {}", game_state.stacks[target].name)
            }
            Order::Isru {
                ore, water, fuel, ..
            } => match (*ore, *water, *fuel) {
                (ore, 0, 0) => format!("{stack_name}: Mine {:.1}t ore", ore as f32 / 10.0),
                (0, water, 0) => format!("{stack_name}: Mine {:.1}t water", water as f32 / 10.0),
                (ore, water, 0) => format!(
                    "{stack_name}: Mine {:.1}t ore and {:.1}t water",
                    ore as f32 / 10.0,
                    water as f32 / 10.0
                ),
                (0, 0, fuel) => format!("{stack_name}: Skim {:.1}t fuel", fuel as f32 / 10.0,),
                _ => "Invalid ISRU order".to_owned(),
            },
            Order::ResourceTransfer {
                stack,
                from,
                to,
                ore,
                materials,
                water,
                fuel,
            } => match (from, to) {
                (None, ResourceTransferTarget::Jettison) => format!(
                    "{stack_name}: Jettison {}",
                    Self::format_resources(
                        *ore as u32,
                        *materials as u32,
                        *water as u32,
                        *fuel as u32
                    )
                ),
                (None, ResourceTransferTarget::Module(module_id)) => {
                    format!(
                        "{stack_name}: Transfer {} to {}",
                        Self::format_resources(
                            *ore as u32,
                            *materials as u32,
                            *water as u32,
                            *fuel as u32
                        ),
                        game_state.stacks[stack].modules[module_id]
                    )
                }
                (None, ResourceTransferTarget::Stack(stack_id)) => format!(
                    "{stack_name}: Transfer {} to {}",
                    Self::format_resources(
                        *ore as u32,
                        *materials as u32,
                        *water as u32,
                        *fuel as u32
                    ),
                    game_state.stacks[stack_id].name
                ),
                (Some(module), ResourceTransferTarget::FloatingPool) => {
                    format!(
                        "{stack_name}: Transfer {} from {}",
                        Self::format_resources(
                            *ore as u32,
                            *materials as u32,
                            *water as u32,
                            *fuel as u32
                        ),
                        game_state.stacks[stack].modules[module]
                    )
                }
                _ => "Invalid transfer order".to_owned(),
            },
            Order::Repair {
                stack,
                target_stack,
                target_module,
            } if stack == target_stack => format!(
                "{stack_name}: Repair {}",
                game_state.stacks[target_stack].modules[target_module]
            ),
            Order::Repair {
                target_stack,
                target_module,
                ..
            } => format!(
                "{stack_name}: Repair {} on {}",
                game_state.stacks[target_stack].modules[target_module],
                game_state.stacks[target_stack].name
            ),
            Order::Refine {
                materials, fuel, ..
            } => format!(
                "{stack_name}: Produce {}",
                Self::format_resources(0, *materials as u32, 0, *fuel as u32)
            ),
            Order::Build { module, .. } => {
                format!("{stack_name}: Build {}", module)
            }
            Order::Salvage { stack, salvaged } => {
                format!(
                    "{stack_name}: Salvage {}",
                    game_state.stacks[stack].modules[salvaged]
                )
            }
            Order::Shoot { target, shots, .. } if *shots == 1 => {
                format!("{stack_name}: Shoot {}", game_state.stacks[target].name)
            }
            Order::Shoot { target, shots, .. } => {
                format!(
                    "{stack_name}: Shoot {} {} times",
                    game_state.stacks[target].name, shots
                )
            }
            Order::Arm {
                stack,
                warhead,
                armed,
            } if *armed => format!(
                "{stack_name}: Arm {}",
                game_state.stacks[stack].modules[warhead]
            ),
            Order::Arm { stack, warhead, .. } => {
                format!(
                    "{stack_name}: Disarm {}",
                    game_state.stacks[stack].modules[warhead]
                )
            }
            Order::Burn { delta_v, .. } => {
                format!("{stack_name}: Make a {} hex/turn burn", delta_v.norm())
            }
            Order::OrbitAdjust { around, .. } => format!(
                "{stack_name}: Adjust orbit around {}",
                game_state.celestials.get(*around).unwrap().name
            ),
            Order::Land { on, .. } => {
                format!(
                    "{stack_name}: Land on {}",
                    game_state.celestials.get(*on).unwrap().name
                )
            }
            Order::TakeOff { from, .. } => format!(
                "{stack_name}: Take off from {}",
                game_state.celestials.get(*from).unwrap().name
            ),
        }
    }
}

/// A set of orders that can be applied to the referenced game state
#[cfg_attr(all(feature = "client", not(feature = "server")), expect(dead_code))]
pub struct ValidatedOrders<'a> {
    orders: HashMap<PlayerId, Vec<&'a Order>>,
    game_state: &'a GameState,
}

impl ValidatedOrders<'_> {
    /// Apply orders
    #[cfg(feature = "server")]
    pub fn apply(
        &self,
        stack_id_generator: &mut impl Iterator<Item = StackId>,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
        rng: &mut impl Rng,
    ) -> HashMap<StackId, Stack> {
        let mut stacks = self.game_state.stacks.clone();
        let mut new_stack_ids = HashMap::new();
        for (&player, orders) in self.orders.iter() {
            for order in orders {
                order.apply(
                    &mut stacks,
                    player,
                    &mut new_stack_ids,
                    self.game_state,
                    stack_id_generator,
                    module_id_generator,
                    rng,
                );
            }
        }
        stacks
    }
}

#[cfg(all(test, feature = "server", feature = "client"))]
mod tests {
    use super::*;

    use rand::{SeedableRng, rng, rngs::StdRng};
    use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

    struct ShortIdGen<T: From<u8>> {
        next: u8,
        _t: PhantomData<T>,
    }
    impl<T: From<u8>> ShortIdGen<T> {
        pub fn new() -> Self {
            Self {
                next: 0,
                _t: PhantomData,
            }
        }
    }
    impl<T: From<u8>> Iterator for ShortIdGen<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let result = Some(self.next.into());
            self.next += 1;
            result
        }
    }
    struct LongIdGen<T: From<u32>> {
        next: u32,
        _t: PhantomData<T>,
    }
    impl<T: From<u32>> LongIdGen<T> {
        pub fn new() -> Self {
            Self {
                next: 0,
                _t: PhantomData,
            }
        }
    }
    impl<T: From<u32>> Iterator for LongIdGen<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let result = Some(self.next.into());
            self.next += 1;
            result
        }
    }

    #[test]
    fn test_rename_stack() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let player_2 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([
                (player_1, "player 1".to_owned()),
                (player_2, "player 2".to_owned()),
            ]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );

        let stack_1 = stack_id_generator.next().unwrap();
        let stack_2 = stack_id_generator.next().unwrap();
        let stack_1_data = Stack::new(
            Vec2::zero(),
            Vec2::zero(),
            player_1,
            "original name".to_owned(),
        );
        game_state.stacks.insert(stack_1, stack_1_data);
        game_state.stacks.insert(
            stack_2,
            Stack::new(Vec2::zero(), Vec2::zero(), player_2, "".to_owned()),
        );

        // Test 1: Valid rename order
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_1,
                name: "new name".to_owned(),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        // Test order application
        let mut rng = StdRng::seed_from_u64(42);
        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, "new name");

        // Test 2: Invalid stack id
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: 999_u32.into(),
                name: "new name".to_owned(),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::InvalidStackId(_)));

        // Test 3: Wrong ownership (trying to rename another player's stack)
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_2,
                name: "hacked name".to_owned(),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::BadOwnership(_)));

        // Test 4: Empty name is allowed
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_1,
                name: String::new(),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, "");

        // Test 5: Very long name is allowed
        let long_name = "a".repeat(1000);
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_1,
                name: long_name.clone(),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, long_name);

        // Test 6: Special characters in name are allowed
        let special_name = "Stack-1_TEST!@#$%^&*(){}[]|\\:;\"'<>,.?/`~";
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_1,
                name: special_name.to_owned(),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, special_name);

        // Test 7: Multiple rename orders in same turn
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::NameStack {
                    stack: stack_1,
                    name: "first name".to_owned(),
                },
                Order::NameStack {
                    stack: stack_1,
                    name: "second name".to_owned(),
                },
            ],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());
        assert!(errors[&player_1][1].is_none());

        // Both orders should be valid and the last one should win
        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, "second name");

        // Test 8: Rename order works in all phases
        for phase in [Phase::Logistics, Phase::Combat, Phase::Movement] {
            game_state.phase = phase;

            let orders = HashMap::from([(
                player_1,
                vec![Order::NameStack {
                    stack: stack_1,
                    name: format!("name in {phase:?}"),
                }],
            )]);

            let (_, errors) = Order::validate(&game_state, &orders);
            assert!(
                errors[&player_1][0].is_none(),
                "NameStack should be valid in {phase:?} phase"
            );
        }

        // Test 9: Unicode characters in name
        let unicode_name = "测试名称 🚀 ñoél ∑φμβοl";
        let orders = HashMap::from([(
            player_1,
            vec![Order::NameStack {
                stack: stack_1,
                name: unicode_name.to_owned(),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        assert_eq!(new_stacks[&stack_1].name, unicode_name);
    }

    #[test]
    fn test_module_transfer() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let player_2 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([
                (player_1, "player 1".to_owned()),
                (player_2, "player 2".to_owned()),
            ]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        // create stacks and modules
        let stack_1 = stack_id_generator.next().unwrap();
        let stack_2 = stack_id_generator.next().unwrap();
        let stack_3 = stack_id_generator.next().unwrap();
        let module_1 = module_id_generator.next().unwrap();
        let module_2 = module_id_generator.next().unwrap();

        let mut stack_1_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_1_data.modules.insert(module_1, Module::new_engine());
        stack_1_data.modules.insert(module_2, Module::new_gun());

        let stack_2_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned()); // same position for rendezvous
        let stack_3_data = Stack::new(Vec2 { q: 1, r: 0 }, Vec2::zero(), player_2, "".to_owned()); // different player

        game_state.stacks.insert(stack_1, stack_1_data);
        game_state.stacks.insert(stack_2, stack_2_data);
        game_state.stacks.insert(stack_3, stack_3_data);

        // Test 1: Valid transfer to existing rendezvoused stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        // Test 2: Invalid stack id
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: 999_u32.into(),
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::InvalidStackId(_)));

        // Test 3: Invalid module id
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: 999_u32.into(),
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::InvalidModuleId(_, _)));

        // Test 4: Wrong ownership of source stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_3,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::BadOwnership(_)));

        // Test 5: Wrong ownership of target stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_3),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::BadOwnership(_)));

        // Test 6: Not rendezvoused stacks
        let stack_4 = stack_id_generator.next().unwrap();
        let stack_4_data = Stack::new(Vec2 { q: 2, r: 0 }, Vec2::zero(), player_1, "".to_owned()); // different position
        game_state.stacks.insert(stack_4, stack_4_data);

        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_4),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::NotRendezvoused(_, _)));

        // Test 7: Valid transfer to new stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::New(0),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        // Test 8: Wrong phase
        game_state.phase = Phase::Combat;
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::WrongPhase));

        // Test 9: Multiple transfers of same module (conflict)
        game_state.phase = Phase::Logistics;
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ModuleTransfer {
                    stack: stack_1,
                    module: module_1,
                    to: ModuleTransferTarget::Existing(stack_2),
                },
                Order::ModuleTransfer {
                    stack: stack_1,
                    module: module_1,
                    to: ModuleTransferTarget::New(0),
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_some());
        assert!(errors[&player_1][1].is_some());
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::ModuleTransferConflict
        ));
        assert!(matches!(
            errors[&player_1][1].unwrap(),
            OrderError::ModuleTransferConflict
        ));

        // Test 10: Multiple transfers to same new stack with different positions (conflict)
        let stack_5 = stack_id_generator.next().unwrap();
        let mut stack_5_data =
            Stack::new(Vec2 { q: 3, r: 0 }, Vec2::zero(), player_1, "".to_owned()); // different position
        let module_3 = module_id_generator.next().unwrap();
        stack_5_data.modules.insert(module_3, Module::new_tank());
        game_state.stacks.insert(stack_5, stack_5_data);

        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ModuleTransfer {
                    stack: stack_1,
                    module: module_1,
                    to: ModuleTransferTarget::New(0),
                },
                Order::ModuleTransfer {
                    stack: stack_5,
                    module: module_3,
                    to: ModuleTransferTarget::New(0),
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_some());
        assert!(errors[&player_1][1].is_some());
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NewStackStateConflict
        ));
        assert!(matches!(
            errors[&player_1][1].unwrap(),
            OrderError::NewStackStateConflict
        ));
    }

    #[test]
    fn test_module_transfer_application() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        // create stacks and modules
        let stack_1 = stack_id_generator.next().unwrap();
        let stack_2 = stack_id_generator.next().unwrap();
        let module_1 = module_id_generator.next().unwrap();
        let module_2 = module_id_generator.next().unwrap();

        let mut stack_1_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_1_data.modules.insert(module_1, Module::new_engine());
        stack_1_data.modules.insert(module_2, Module::new_gun());

        let stack_2_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned()); // same position for rendezvous

        game_state.stacks.insert(stack_1, stack_1_data);
        game_state.stacks.insert(stack_2, stack_2_data);

        // Test 1: Transfer to existing stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_1,
                to: ModuleTransferTarget::Existing(stack_2),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let mut rng = StdRng::seed_from_u64(42);
        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);

        // Verify module was moved
        assert!(!new_stacks[&stack_1].modules.contains_key(&module_1));
        assert!(new_stacks[&stack_1].modules.contains_key(&module_2));
        assert!(new_stacks[&stack_2].modules.contains_key(&module_1));

        // Test 2: Transfer to new stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::ModuleTransfer {
                stack: stack_1,
                module: module_2,
                to: ModuleTransferTarget::New(0),
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none());

        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);

        // Verify module was moved to a new stack
        assert!(!new_stacks[&stack_1].modules.contains_key(&module_2));

        // Find the new stack that was created
        let new_stack_id = new_stacks
            .keys()
            .find(|&&id| {
                id != stack_1 && id != stack_2 && new_stacks[&id].modules.contains_key(&module_2)
            })
            .expect("New stack should exist");

        let new_stack = &new_stacks[new_stack_id];

        // Verify new stack has correct properties
        assert_eq!(new_stack.position, Vec2::zero());
        assert_eq!(new_stack.velocity, Vec2::zero());
        assert_eq!(new_stack.owner, player_1);
        assert!(new_stack.modules.contains_key(&module_2));
    }

    #[test]
    fn test_isru_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        // Create a mining world and set up a stack there
        let mining_world = celestial_id_generator.next().unwrap();
        let mut celestials: HashMap<CelestialId, Celestial> =
            Arc::into_inner(game_state.celestials).unwrap().into();
        celestials.insert(
            mining_world,
            Celestial {
                position: Vec2 { q: 5, r: 5 },
                name: "Mining World".to_owned(),
                orbit_gravity: true,
                surface_gravity: 3.0,
                resources: Resources::MiningBoth,
                radius: 0.3,
                colour: "#000000".to_owned(),
                is_minor: false,
            },
        );
        game_state.celestials = Arc::new(celestials.into());

        let stack_id = stack_id_generator.next().unwrap();
        let miner_module = module_id_generator.next().unwrap();
        let mut stack_data = Stack::new(Vec2 { q: 5, r: 5 }, Vec2::zero(), player_1, "".to_owned()); // Landed on mining world
        stack_data.modules.insert(miner_module, Module::new_miner());
        game_state.stacks.insert(stack_id, stack_data);

        // Test valid ISRU order for mining
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Isru {
                    stack: stack_id,
                    ore: 5,
                    water: 5,
                    fuel: 0,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 5,
                    materials: 0,
                    water: 5,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid ISRU order should succeed"
        );

        // Test ISRU order with wrong phase
        game_state.phase = Phase::Combat;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
        game_state.phase = Phase::Logistics;

        // Test ISRU order when not on resource body
        let mut wrong_stack =
            Stack::new(Vec2 { q: 0, r: 0 }, Vec2::zero(), player_1, "".to_owned()); // Not on mining world
        wrong_stack
            .modules
            .insert(module_id_generator.next().unwrap(), Module::new_miner());
        let wrong_stack_id = stack_id_generator.next().unwrap();
        game_state.stacks.insert(wrong_stack_id, wrong_stack);

        let orders = HashMap::from([(
            player_1,
            vec![Order::Isru {
                stack: wrong_stack_id,
                ore: 5,
                water: 0,
                fuel: 0,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NoResourceAccess
        ));

        // Test fuel skimming
        let gas_giant = celestial_id_generator.next().unwrap();
        let mut celestials: HashMap<CelestialId, Celestial> =
            Arc::into_inner(game_state.celestials).unwrap().into();
        celestials.insert(
            gas_giant,
            Celestial {
                position: Vec2 { q: 10, r: 10 },
                name: "Gas Giant".to_owned(),
                orbit_gravity: true,
                surface_gravity: 20.0,
                resources: Resources::Skimming,
                radius: 1.0,
                colour: "#000000".to_owned(),
                is_minor: false,
            },
        );
        game_state.celestials = Arc::new(celestials.into());

        let orbiting_stack_id = stack_id_generator.next().unwrap();
        let skimmer_module = module_id_generator.next().unwrap();
        let mut orbiting_stack = Stack::new(
            Vec2 { q: 10, r: 11 },
            Vec2 { q: -1, r: 0 },
            player_1,
            "".to_owned(),
        ); // Orbiting gas giant
        orbiting_stack
            .modules
            .insert(skimmer_module, Module::new_fuel_skimmer());
        game_state.stacks.insert(orbiting_stack_id, orbiting_stack);

        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Isru {
                    stack: orbiting_stack_id,
                    ore: 0,
                    water: 0,
                    fuel: 8,
                },
                Order::ResourceTransfer {
                    stack: orbiting_stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 0,
                    materials: 0,
                    water: 0,
                    fuel: 8,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid fuel skimming should succeed"
        );
    }

    #[test]
    fn test_resource_transfer_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        let stack_id = stack_id_generator.next().unwrap();
        let cargo_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();

        let mut cargo_hold = Module::new_cargo_hold();
        if let ModuleDetails::CargoHold { ore, materials } = &mut cargo_hold.details {
            *ore = 50;
            *materials = 30;
        }

        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { water, fuel } = &mut tank.details {
            *water = 40;
            *fuel = 60;
        }

        let mut stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_data.modules.insert(cargo_module, cargo_hold);
        stack_data.modules.insert(tank_module, tank);
        game_state.stacks.insert(stack_id, stack_data);

        // Test valid resource transfer from module to floating pool
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 10,
                    materials: 5,
                    water: 0,
                    fuel: 0,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 10,
                    materials: 5,
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid resource transfer should succeed"
        );

        // Test invalid transfer - trying to move both solids and liquids from same module
        let orders = HashMap::from([(
            player_1,
            vec![Order::ResourceTransfer {
                stack: stack_id,
                from: Some(cargo_module),
                to: ResourceTransferTarget::FloatingPool,
                ore: 10,
                materials: 5,
                water: 10, // Invalid - cargo holds don't hold water
                fuel: 0,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidModuleType(_, _)
        ));

        // Test transfer from floating pool to module
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Module(tank_module),
                    ore: 0,
                    materials: 0,
                    water: 5,
                    fuel: 10,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(tank_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: 0,
                    water: 5,
                    fuel: 10,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid transfer to module should succeed"
        );

        // Test jettison
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 5,
                    materials: 5,
                    water: 5,
                    fuel: 5,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(tank_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: 0,
                    water: 5,
                    fuel: 5,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 5,
                    materials: 5,
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid jettison should succeed"
        );

        // Test wrong phase
        game_state.phase = Phase::Combat;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
    }

    #[test]
    fn test_repair_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        let repair_stack_id = stack_id_generator.next().unwrap();
        let target_stack_id = stack_id_generator.next().unwrap();
        let habitat_module = module_id_generator.next().unwrap();
        let damaged_module = module_id_generator.next().unwrap();

        // Create repair stack with habitat and cargo hold with materials
        let mut repair_stack = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        repair_stack
            .modules
            .insert(habitat_module, Module::new_habitat(player_1));

        let cargo_module = module_id_generator.next().unwrap();
        let mut cargo_hold = Module::new_cargo_hold();
        if let ModuleDetails::CargoHold { materials, .. } = &mut cargo_hold.details {
            *materials = 10; // Add some materials for repair
        }
        repair_stack.modules.insert(cargo_module, cargo_hold);
        game_state.stacks.insert(repair_stack_id, repair_stack);

        // Create target stack with damaged module
        let mut target_stack = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned()); // Same position for rendezvous
        let mut damaged_engine = Module::new_engine();
        damaged_engine.health = Health::Damaged;
        target_stack.modules.insert(damaged_module, damaged_engine);
        game_state.stacks.insert(target_stack_id, target_stack);

        // Test valid repair order
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ResourceTransfer {
                    stack: repair_stack_id,
                    from: Some(cargo_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: 1, // Need materials to repair (1/10th of module mass)
                    water: 0,
                    fuel: 0,
                },
                Order::Repair {
                    stack: repair_stack_id,
                    target_stack: target_stack_id,
                    target_module: damaged_module,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid repair should succeed"
        );

        // Test repair on intact module (should fail)
        let intact_module = module_id_generator.next().unwrap();
        game_state
            .stacks
            .get_mut(&target_stack_id)
            .unwrap()
            .modules
            .insert(intact_module, Module::new_gun());

        let orders = HashMap::from([(
            player_1,
            vec![Order::Repair {
                stack: repair_stack_id,
                target_stack: target_stack_id,
                target_module: intact_module,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NotDamaged
        ));

        // Test repair when not rendezvoused
        let far_stack_id = stack_id_generator.next().unwrap();
        let far_damaged_module = module_id_generator.next().unwrap();
        let mut far_stack = Stack::new(Vec2 { q: 5, r: 5 }, Vec2::zero(), player_1, "".to_owned()); // Different position
        let mut far_damaged_engine = Module::new_engine();
        far_damaged_engine.health = Health::Damaged;
        far_stack
            .modules
            .insert(far_damaged_module, far_damaged_engine);
        game_state.stacks.insert(far_stack_id, far_stack);

        let orders = HashMap::from([(
            player_1,
            vec![Order::Repair {
                stack: repair_stack_id,
                target_stack: far_stack_id,
                target_module: far_damaged_module,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NotRendezvoused(_, _)
        ));
    }

    #[test]
    fn test_refine_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        let stack_id = stack_id_generator.next().unwrap();
        let refinery_module = module_id_generator.next().unwrap();
        let cargo_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();

        let mut stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_data
            .modules
            .insert(refinery_module, Module::new_refinery());

        // Add cargo hold with ore
        let mut cargo_hold = Module::new_cargo_hold();
        if let ModuleDetails::CargoHold { ore, .. } = &mut cargo_hold.details {
            *ore = 20; // Enough ore for refining
        }
        stack_data.modules.insert(cargo_module, cargo_hold);

        // Add tank with water
        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { water, .. } = &mut tank.details {
            *water = 10; // Enough water for refining
        }
        stack_data.modules.insert(tank_module, tank);

        game_state.stacks.insert(stack_id, stack_data);

        // Test valid refine order
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 20, // Need 2 ore per 1 material, so 20 ore for 10 materials
                    materials: 0,
                    water: 0,
                    fuel: 0,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(tank_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: 0,
                    water: 10, // Need 2 water per 1 fuel, so 10 water for 5 fuel
                    fuel: 0,
                },
                Order::Refine {
                    stack: stack_id,
                    materials: 10,
                    fuel: 5,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 0,
                    materials: 10, // Jettison the produced materials
                    water: 0,
                    fuel: 5, // Jettison the produced fuel
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid refine should succeed"
        );

        // Test wrong phase
        game_state.phase = Phase::Combat;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
        game_state.phase = Phase::Logistics;

        // Test with invalid stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::Refine {
                stack: 999_u32.into(),
                materials: 10,
                fuel: 5,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidStackId(_)
        ));
    }

    #[test]
    fn test_build_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        let stack_id = stack_id_generator.next().unwrap();
        let factory_module = module_id_generator.next().unwrap();
        let cargo_hold_module = module_id_generator.next().unwrap();

        // Set up Earth for habitat building
        let earth_celestial = game_state.celestials.get(game_state.earth).unwrap();
        let earth_position = earth_celestial.position;
        let earth_neighbors = earth_position.neighbours();

        let mut stack_data = Stack::new(
            earth_neighbors[0],
            Vec2 { q: 1, r: 0 },
            player_1,
            "".to_owned(),
        ); // Orbiting Earth
        stack_data
            .modules
            .insert(factory_module, Module::new_factory());
        let mut cargo_hold_actual = Module::new_cargo_hold();
        let ModuleDetails::CargoHold {
            ref mut materials, ..
        } = cargo_hold_actual.details
        else {
            unreachable!();
        };
        *materials = ModuleDetails::CARGO_HOLD_CAPACITY as u8;
        stack_data
            .modules
            .insert(cargo_hold_module, cargo_hold_actual);
        game_state.stacks.insert(stack_id, stack_data);

        // Test building various module types
        // No test for factories because they're too expensive to easily test
        for module_type in [
            ModuleType::Engine,
            ModuleType::Gun,
            ModuleType::Miner,
            ModuleType::FuelSkimmer,
            ModuleType::CargoHold,
            ModuleType::Tank,
            ModuleType::Warhead,
            ModuleType::Refinery,
            ModuleType::ArmourPlate,
        ] {
            let orders = HashMap::from([(
                player_1,
                vec![
                    Order::Build {
                        stack: stack_id,
                        module: module_type,
                    },
                    Order::ResourceTransfer {
                        stack: stack_id,
                        from: Some(cargo_hold_module),
                        to: ResourceTransferTarget::FloatingPool,
                        ore: 0,
                        materials: module_type.cost() as u8,
                        water: 0,
                        fuel: 0,
                    },
                ],
            )]);

            let (_, errors) = Order::validate(&game_state, &orders);
            assert!(
                errors[&player_1][0].is_none(),
                "Building module should succeed"
            );
        }

        // Test building habitat in Earth orbit (should succeed)
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Build {
                    stack: stack_id,
                    module: ModuleType::Habitat,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_hold_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: ModuleType::Habitat.cost() as u8,
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Building habitat in Earth orbit should succeed"
        );

        // Test building habitat NOT in Earth orbit (should fail)
        let away_stack_id = stack_id_generator.next().unwrap();
        let away_factory_module = module_id_generator.next().unwrap();
        let mut away_stack =
            Stack::new(Vec2 { q: 20, r: 20 }, Vec2::zero(), player_1, "".to_owned()); // Far from Earth
        away_stack
            .modules
            .insert(away_factory_module, Module::new_factory());
        game_state.stacks.insert(away_stack_id, away_stack);

        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Build {
                    stack: away_stack_id,
                    module: ModuleType::Habitat,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_hold_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: ModuleType::Habitat.cost() as u8,
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NotInEarthOrbit
        ));

        // Test wrong phase
        game_state.phase = Phase::Combat;
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Build {
                    stack: stack_id,
                    module: ModuleType::Engine,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(cargo_hold_module),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    materials: ModuleType::Engine.cost() as u8,
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
    }

    #[test]
    fn test_salvage_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        let stack_id = stack_id_generator.next().unwrap();
        let factory_module = module_id_generator.next().unwrap();
        let salvage_module = module_id_generator.next().unwrap();

        let mut stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_data
            .modules
            .insert(factory_module, Module::new_factory());
        stack_data.modules.insert(salvage_module, Module::new_gun()); // Module to salvage
        game_state.stacks.insert(stack_id, stack_data);

        // Test valid salvage order
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Salvage {
                    stack: stack_id,
                    salvaged: salvage_module,
                },
                Order::ResourceTransfer {
                    stack: stack_id,
                    from: None,
                    to: ResourceTransferTarget::Jettison,
                    ore: 0,
                    materials: 10, // Salvage produces gun_mass * 10 / 2 = 2 * 10 / 2 = 10 materials
                    water: 0,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid salvage should succeed"
        );

        // Test salvaging invalid module
        let orders = HashMap::from([(
            player_1,
            vec![Order::Salvage {
                stack: stack_id,
                salvaged: 999_u32.into(),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidModuleId(_, _)
        ));

        // Test wrong phase
        game_state.phase = Phase::Combat;
        let orders = HashMap::from([(
            player_1,
            vec![Order::Salvage {
                stack: stack_id,
                salvaged: salvage_module,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
    }

    #[test]
    fn test_shoot_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let player_2 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([
                (player_1, "player 1".to_owned()),
                (player_2, "player 2".to_owned()),
            ]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Combat;

        let shooter_stack = stack_id_generator.next().unwrap();
        let target_stack = stack_id_generator.next().unwrap();
        let gun_module = module_id_generator.next().unwrap();

        let mut shooter_data =
            Stack::new(Vec2 { q: 20, r: 20 }, Vec2::zero(), player_1, "".to_owned()); // Far from celestials
        shooter_data.modules.insert(gun_module, Module::new_gun());
        game_state.stacks.insert(shooter_stack, shooter_data);

        let target_data = Stack::new(Vec2 { q: 22, r: 20 }, Vec2::zero(), player_2, "".to_owned()); // 2 hexes away horizontally
        game_state.stacks.insert(target_stack, target_data);

        // Test valid shoot order (should have clear line of sight initially)
        let orders = HashMap::from([(
            player_1,
            vec![Order::Shoot {
                stack: shooter_stack,
                target: target_stack,
                shots: 1,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none(), "Valid shoot should succeed");

        // Test wrong phase
        game_state.phase = Phase::Logistics;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
        game_state.phase = Phase::Combat;

        // Test shooting self
        let orders = HashMap::from([(
            player_1,
            vec![Order::Shoot {
                stack: shooter_stack,
                target: shooter_stack,
                shots: 1,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidTarget
        ));

        // Test shooting invalid target
        let orders = HashMap::from([(
            player_1,
            vec![Order::Shoot {
                stack: shooter_stack,
                target: 999_u32.into(),
                shots: 1,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidStackId(_)
        ));

        // Test line of sight blocked by celestial
        let blocking_celestial = celestial_id_generator.next().unwrap();
        let mut celestials: HashMap<CelestialId, Celestial> =
            Arc::into_inner(game_state.celestials).unwrap().into();
        celestials.insert(
            blocking_celestial,
            Celestial {
                position: Vec2 { q: 21, r: 20 }, // Exactly between shooter at (20,20) and target at (22,20)
                name: "Blocker".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.0,
                resources: Resources::None,
                radius: 1.0, // Reasonable blocking radius
                colour: "#000000".to_owned(),
                is_minor: false,
            },
        );
        game_state.celestials = Arc::new(celestials.into());

        let orders = HashMap::from([(
            player_1,
            vec![Order::Shoot {
                stack: shooter_stack,
                target: target_stack,
                shots: 1,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NoLineOfSight
        ));
    }

    #[test]
    fn test_arm_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Combat;

        let stack_id = stack_id_generator.next().unwrap();
        let warhead_module = module_id_generator.next().unwrap();

        let mut stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        stack_data
            .modules
            .insert(warhead_module, Module::new_warhead());
        game_state.stacks.insert(stack_id, stack_data);

        // Test valid arm order
        let orders = HashMap::from([(
            player_1,
            vec![Order::Arm {
                stack: stack_id,
                warhead: warhead_module,
                armed: true,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none(), "Valid arm should succeed");

        // Test disarm order
        let orders = HashMap::from([(
            player_1,
            vec![Order::Arm {
                stack: stack_id,
                warhead: warhead_module,
                armed: false,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid disarm should succeed"
        );

        // Test wrong phase
        game_state.phase = Phase::Movement;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
        game_state.phase = Phase::Combat;

        // Test arming non-warhead module
        let gun_module = module_id_generator.next().unwrap();
        game_state
            .stacks
            .get_mut(&stack_id)
            .unwrap()
            .modules
            .insert(gun_module, Module::new_gun());

        let orders = HashMap::from([(
            player_1,
            vec![Order::Arm {
                stack: stack_id,
                warhead: gun_module, // Not a warhead
                armed: true,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidModuleType(_, _)
        ));

        // Test arming when habitat is present
        let habitat_module = module_id_generator.next().unwrap();
        game_state
            .stacks
            .get_mut(&stack_id)
            .unwrap()
            .modules
            .insert(habitat_module, Module::new_habitat(player_1));

        let orders = HashMap::from([(
            player_1,
            vec![Order::Arm {
                stack: stack_id,
                warhead: warhead_module,
                armed: true,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::HabOnStack
        ));
    }

    #[test]
    fn test_module_type_cost() {
        // Test that all module types have positive costs
        assert!(ModuleType::Miner.cost() > 0);
        assert!(ModuleType::FuelSkimmer.cost() > 0);
        assert!(ModuleType::CargoHold.cost() > 0);
        assert!(ModuleType::Tank.cost() > 0);
        assert!(ModuleType::Engine.cost() > 0);
        assert!(ModuleType::Warhead.cost() > 0);
        assert!(ModuleType::Gun.cost() > 0);
        assert!(ModuleType::Habitat.cost() > 0);
        assert!(ModuleType::Refinery.cost() > 0);
        assert!(ModuleType::Factory.cost() > 0);
        assert!(ModuleType::ArmourPlate.cost() > 0);

        // Test that costs are reasonable multiples of mass
        assert_eq!(
            ModuleType::Engine.cost(),
            ModuleDetails::ENGINE_MASS as i32 * 10
        );
        assert_eq!(
            ModuleType::Habitat.cost(),
            ModuleDetails::HABITAT_MASS as i32 * 10
        );
    }

    #[test]
    fn test_burn_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Movement;

        let stack_id = stack_id_generator.next().unwrap();
        let engine_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();

        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { fuel, .. } = &mut tank.details {
            *fuel = 100; // Plenty of fuel
        }

        let mut stack_data = Stack::new(Vec2 { q: 5, r: 5 }, Vec2::zero(), player_1, "".to_owned());
        stack_data
            .modules
            .insert(engine_module, Module::new_engine());
        stack_data.modules.insert(tank_module, tank);
        game_state.stacks.insert(stack_id, stack_data);

        // Test valid burn order (small delta-v)
        let orders = HashMap::from([(
            player_1,
            vec![Order::Burn {
                stack: stack_id,
                delta_v: Vec2 { q: 1, r: 0 },      // 1 hex/turn delta-v
                fuel_from: vec![(tank_module, 6)], // Use correct fuel amount: ceil(12 * 1 / 2) = 6
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none(), "Valid burn should succeed");

        // Test wrong phase
        game_state.phase = Phase::Logistics;
        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::WrongPhase
        ));
        game_state.phase = Phase::Movement;

        // Test burning while landed on a planet
        let planet = celestial_id_generator.next().unwrap();
        let mut celestials: HashMap<CelestialId, Celestial> =
            Arc::into_inner(game_state.celestials).unwrap().into();
        celestials.insert(
            planet,
            Celestial {
                position: Vec2 { q: 5, r: 5 }, // Same position as stack
                name: "Planet".to_owned(),
                orbit_gravity: true,
                surface_gravity: 9.8,
                resources: Resources::MiningBoth,
                radius: 0.3,
                colour: "#000000".to_owned(),
                is_minor: false,
            },
        );
        game_state.celestials = Arc::new(celestials.into());

        // Move stack to be landed on planet
        game_state.stacks.get_mut(&stack_id).unwrap().velocity = Vec2::zero();

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::BurnWhileLanded
        ));

        // Move stack back to space
        game_state.stacks.get_mut(&stack_id).unwrap().position = Vec2 { q: 10, r: 10 };

        // Test insufficient fuel
        let orders = HashMap::from([(
            player_1,
            vec![Order::Burn {
                stack: stack_id,
                delta_v: Vec2 { q: 1, r: 0 },
                fuel_from: vec![(tank_module, 200)], // More fuel than available
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::NotEnoughResources(_, _)
        ));

        // Test invalid fuel source module
        let orders = HashMap::from([(
            player_1,
            vec![Order::Burn {
                stack: stack_id,
                delta_v: Vec2 { q: 1, r: 0 },
                fuel_from: vec![(999_u32.into(), 10)], // Invalid module
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::InvalidModuleId(_, _)
        ));
    }

    #[test]
    fn test_movement_order_conflicts() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Movement;

        let stack_id = stack_id_generator.next().unwrap();
        let target_stack_id = stack_id_generator.next().unwrap();
        let engine_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();

        let mut fuel_tank = Module::new_tank();
        if let ModuleDetails::Tank { fuel, .. } = &mut fuel_tank.details {
            *fuel = 100;
        }

        // Get Earth's actual position and set up orbital parameters
        let earth_celestial = game_state.celestials.get(game_state.earth).unwrap();
        let earth_orbital_params = earth_celestial.orbit_parameters(true);
        let (orbital_pos_1, orbital_vel_1) = earth_orbital_params[0];
        let (orbital_pos_2, _orbital_vel_2) = earth_orbital_params[1];

        // Place both stacks in orbit around Earth using actual orbital parameters
        let mut stack_data = Stack::new(orbital_pos_1, orbital_vel_1, player_1, "".to_owned());
        stack_data
            .modules
            .insert(engine_module, Module::new_engine());
        stack_data.modules.insert(tank_module, fuel_tank);
        game_state.stacks.insert(stack_id, stack_data);

        // Target stack at a different orbital position (not needed for this test, but keeping for completeness)
        let target_engine = module_id_generator.next().unwrap();
        let mut target_data =
            Stack::new(orbital_pos_2, Vec2 { q: 0, r: 1 }, player_1, "".to_owned());
        target_data
            .modules
            .insert(target_engine, Module::new_engine());
        game_state.stacks.insert(target_stack_id, target_data);

        // Test multiple movement orders for same stack
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Burn {
                    stack: stack_id,
                    delta_v: Vec2 { q: 1, r: 0 },
                    fuel_from: vec![(tank_module, 6)], // Correct fuel calculation
                },
                Order::Burn {
                    stack: stack_id,
                    delta_v: Vec2 { q: 0, r: 1 },
                    fuel_from: vec![(tank_module, 6)], // Another burn order for same stack
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        // Both burn orders should fail with MultipleMoves
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::MultipleMoves
        ));
        assert!(matches!(
            errors[&player_1][1].unwrap(),
            OrderError::MultipleMoves
        ));

        // Test valid orbit adjustment
        let orders = HashMap::from([(
            player_1,
            vec![Order::OrbitAdjust {
                stack: stack_id,
                around: game_state.earth,
                target_position: orbital_pos_2,
                clockwise: true,
                fuel_from: vec![(tank_module, 6)],
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid orbit adjustment should succeed"
        );

        // Test invalid orbit adjustment - target too far (use a position not in orbit)
        let far_position = earth_celestial.position + Vec2 { q: 10, r: 0 };
        let orders = HashMap::from([(
            player_1,
            vec![Order::OrbitAdjust {
                stack: stack_id,
                around: game_state.earth,
                target_position: far_position, // Not an orbital position
                clockwise: true,
                fuel_from: vec![(tank_module, 6)],
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::DestinationTooFar
        ));
    }

    #[test]
    fn test_orbit_adjust_application() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([(player_1, "player 1".to_owned())]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Movement;

        let stack_id = stack_id_generator.next().unwrap();
        let engine_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();

        let mut fuel_tank = Module::new_tank();
        if let ModuleDetails::Tank { fuel, .. } = &mut fuel_tank.details {
            *fuel = 100;
        }

        // Get Earth's actual position and set up orbital parameters
        let earth_celestial = game_state.celestials.get(game_state.earth).unwrap();
        let earth_orbital_params = earth_celestial.orbit_parameters(true);
        let (orbital_pos_1, orbital_vel_1) = earth_orbital_params[0];
        let (orbital_pos_2, orbital_vel_2) = earth_orbital_params[1];

        // Place stack in orbit around Earth using actual orbital parameters
        let mut stack_data = Stack::new(orbital_pos_1, orbital_vel_1, player_1, "".to_owned());
        stack_data
            .modules
            .insert(engine_module, Module::new_engine());
        stack_data.modules.insert(tank_module, fuel_tank);
        game_state.stacks.insert(stack_id, stack_data);

        // Test orbit adjustment to a different position (clockwise)
        let orders = HashMap::from([(
            player_1,
            vec![Order::OrbitAdjust {
                stack: stack_id,
                around: game_state.earth,
                target_position: orbital_pos_2, // Another valid orbital position
                clockwise: true,
                fuel_from: vec![(tank_module, 6)],
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid orbit adjustment should succeed"
        );

        // Apply the orders and check result
        use rand::SeedableRng;
        use rand::rngs::StdRng;
        let mut rng = StdRng::seed_from_u64(12345);
        let updated_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);

        // Check that the stack moved to the correct position with correct velocity
        let updated_stack = updated_stacks.get(&stack_id).expect("Stack should exist");
        assert_eq!(
            updated_stack.position, orbital_pos_2,
            "Stack should be at target position"
        );
        assert_eq!(
            updated_stack.velocity, orbital_vel_2,
            "Stack should have correct orbital velocity"
        );

        // Test counterclockwise orbit - get counterclockwise parameters
        let earth_ccw_params = earth_celestial.orbit_parameters(false);
        let (ccw_pos_1, ccw_vel_1) = earth_ccw_params[0];
        let (ccw_pos_2, ccw_vel_2) = earth_ccw_params[1];

        game_state.stacks.get_mut(&stack_id).unwrap().position = ccw_pos_1;
        game_state.stacks.get_mut(&stack_id).unwrap().velocity = ccw_vel_1;

        let orders = HashMap::from([(
            player_1,
            vec![Order::OrbitAdjust {
                stack: stack_id,
                around: game_state.earth,
                target_position: ccw_pos_2,
                clockwise: false, // counterclockwise
                fuel_from: vec![(tank_module, 6)],
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid counterclockwise orbit adjustment should succeed"
        );

        let updated_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);
        let updated_stack = updated_stacks.get(&stack_id).expect("Stack should exist");
        assert_eq!(
            updated_stack.position, ccw_pos_2,
            "Stack should be at target position"
        );
        assert_eq!(
            updated_stack.velocity, ccw_vel_2,
            "Stack should have correct counterclockwise orbital velocity"
        );
    }

    #[test]
    fn test_board_orders() {
        // setup game state
        let mut player_id_generator = ShortIdGen::<PlayerId>::new();
        let mut celestial_id_generator = LongIdGen::<CelestialId>::new();
        let mut stack_id_generator = LongIdGen::<StackId>::new();
        let mut module_id_generator = LongIdGen::<ModuleId>::new();
        let player_1 = player_id_generator.next().unwrap();
        let player_2 = player_id_generator.next().unwrap();
        let mut game_state = (GameState::new("test").unwrap())(
            BTreeMap::from([
                (player_1, "player 1".to_owned()),
                (player_2, "player 2".to_owned()),
            ]),
            &mut celestial_id_generator,
            &mut stack_id_generator,
            &mut module_id_generator,
            &mut rng(),
        );
        game_state.phase = Phase::Logistics;

        // Create test stacks and modules
        let boarder_stack = stack_id_generator.next().unwrap();
        let target_stack = stack_id_generator.next().unwrap();
        let defended_stack = stack_id_generator.next().unwrap();
        let no_hab_stack = stack_id_generator.next().unwrap();

        let hab_module = module_id_generator.next().unwrap();
        let engine_module = module_id_generator.next().unwrap();
        let gun_module = module_id_generator.next().unwrap();
        let tank_module = module_id_generator.next().unwrap();
        let target_hab_module = module_id_generator.next().unwrap();

        // Boarder stack: has habitat (can board)
        let mut boarder_stack_data =
            Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        boarder_stack_data
            .modules
            .insert(hab_module, Module::new_habitat(player_1));
        boarder_stack_data
            .modules
            .insert(engine_module, Module::new_engine());

        // Target stack: no habitat (can be boarded)
        let mut target_stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_2, "".to_owned());
        target_stack_data
            .modules
            .insert(gun_module, Module::new_gun());
        target_stack_data
            .modules
            .insert(tank_module, Module::new_tank());

        // Defended stack: has habitat (cannot be boarded)
        let mut defended_stack_data =
            Stack::new(Vec2::zero(), Vec2::zero(), player_2, "".to_owned());
        defended_stack_data
            .modules
            .insert(target_hab_module, Module::new_habitat(player_2));

        // No hab stack: no habitat (cannot board others)
        let no_hab_stack_data = Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());

        game_state.stacks.insert(boarder_stack, boarder_stack_data);
        game_state.stacks.insert(target_stack, target_stack_data);
        game_state
            .stacks
            .insert(defended_stack, defended_stack_data);
        game_state.stacks.insert(no_hab_stack, no_hab_stack_data);

        // Test 1: Valid boarding order
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: boarder_stack,
                target: target_stack,
            }],
        )]);

        let (validated_orders, errors) = Order::validate(&game_state, &orders);
        assert!(
            errors[&player_1][0].is_none(),
            "Valid boarding should succeed"
        );

        // Test order application
        let mut rng = StdRng::seed_from_u64(42);
        let new_stacks =
            validated_orders.apply(&mut stack_id_generator, &mut module_id_generator, &mut rng);

        // Verify boarding worked: all modules from target moved to boarder
        assert!(new_stacks[&boarder_stack].modules.contains_key(&gun_module));
        assert!(
            new_stacks[&boarder_stack]
                .modules
                .contains_key(&tank_module)
        );
        assert!(new_stacks[&boarder_stack].modules.contains_key(&hab_module));
        assert!(
            new_stacks[&boarder_stack]
                .modules
                .contains_key(&engine_module)
        );
        assert!(new_stacks[&target_stack].modules.is_empty());

        // Test 2: Wrong phase
        game_state.phase = Phase::Combat;
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: boarder_stack,
                target: target_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::WrongPhase));

        // Reset to logistics phase
        game_state.phase = Phase::Logistics;

        // Test 3: Invalid boarder stack ID
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: 999_u32.into(),
                target: target_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::InvalidStackId(_)));

        // Test 4: Invalid target stack ID
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: boarder_stack,
                target: 999_u32.into(),
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::InvalidStackId(_)));

        // Test 5: Wrong ownership of boarder stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: defended_stack, // player_2's stack
                target: target_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::BadOwnership(_)));

        // Test 6: Trying to board own stack
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: boarder_stack,
                target: no_hab_stack, // player_1's stack
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::BadOwnership(_)));

        // Test 7: Boarder has no habitat
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: no_hab_stack,
                target: target_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::NoHab));

        // Test 8: Target has habitat (contested boarding)
        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: boarder_stack,
                target: defended_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::ContestedBoarding));

        // Test 9: Multiple boarding attempts on same stack (contested)
        let another_boarder = stack_id_generator.next().unwrap();
        let another_hab = module_id_generator.next().unwrap();
        let mut another_boarder_data =
            Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        another_boarder_data
            .modules
            .insert(another_hab, Module::new_habitat(player_1));
        game_state
            .stacks
            .insert(another_boarder, another_boarder_data);

        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Board {
                    stack: boarder_stack,
                    target: target_stack,
                },
                Order::Board {
                    stack: another_boarder,
                    target: target_stack,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_some());
        assert!(errors[&player_1][1].is_some());
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::ContestedBoarding
        ));
        assert!(matches!(
            errors[&player_1][1].unwrap(),
            OrderError::ContestedBoarding
        ));

        // Test 10: Boarding stack tries to do other orders (should fail)
        let orders = HashMap::from([(
            player_1,
            vec![
                Order::Board {
                    stack: boarder_stack,
                    target: target_stack,
                },
                Order::Refine {
                    stack: boarder_stack,
                    materials: 1,
                    fuel: 0,
                },
            ],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_some());
        assert!(errors[&player_1][1].is_some());
        assert!(matches!(
            errors[&player_1][0].unwrap(),
            OrderError::TooBusyToBoard
        ));

        // Test 11: Target stack has other orders but gets boarded (orders interrupted)
        let refinery_module = module_id_generator.next().unwrap();
        game_state
            .stacks
            .get_mut(&target_stack)
            .unwrap()
            .modules
            .insert(refinery_module, Module::new_refinery());

        let orders = HashMap::from([
            (
                player_1,
                vec![Order::Board {
                    stack: boarder_stack,
                    target: target_stack,
                }],
            ),
            (
                player_2,
                vec![Order::Refine {
                    stack: target_stack,
                    materials: 1,
                    fuel: 0,
                }],
            ),
        ]);

        let (_, errors) = Order::validate(&game_state, &orders);
        assert!(errors[&player_1][0].is_none(), "Boarding should succeed");
        assert!(
            errors[&player_2][0].is_some(),
            "Target's orders should be interrupted"
        );
        assert!(matches!(errors[&player_2][0].unwrap(), OrderError::Boarded));

        // Test 12: Boarding with damaged habitat should fail
        let damaged_hab_stack = stack_id_generator.next().unwrap();
        let damaged_hab_module = module_id_generator.next().unwrap();
        let mut damaged_hab_stack_data =
            Stack::new(Vec2::zero(), Vec2::zero(), player_1, "".to_owned());
        let mut damaged_habitat = Module::new_habitat(player_1);
        damaged_habitat.health = Health::Damaged;
        damaged_hab_stack_data
            .modules
            .insert(damaged_hab_module, damaged_habitat);
        game_state
            .stacks
            .insert(damaged_hab_stack, damaged_hab_stack_data);

        let orders = HashMap::from([(
            player_1,
            vec![Order::Board {
                stack: damaged_hab_stack,
                target: target_stack,
            }],
        )]);

        let (_, errors) = Order::validate(&game_state, &orders);
        let error = errors[&player_1][0].unwrap();
        assert!(matches!(error, OrderError::NoHab));
    }
}
