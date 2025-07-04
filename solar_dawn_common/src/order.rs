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

#[cfg(feature = "server")]
use std::collections::HashMap;

#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::celestial::CelestialId;
#[cfg(feature = "server")]
use crate::{
    GameState, Phase, PlayerId, Vec2,
    celestial::{Celestial, Resources},
    stack::{Health, Module, ModuleDetails, ModuleId, Stack, StackId},
};

/// An order that can be given
#[derive(Debug, Serialize, Deserialize)]
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
        /// How much ore to produce
        ore: u8,
        /// How much water to produce
        water: u8,
        /// How much fuel to produce
        fuel: u8,
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
        /// How much ore to transfer
        ore: u8,
        /// How much materials to transfer
        materials: u8,
        /// How much water to transfer
        water: u8,
        /// How much fuel to transfer
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
        /// How many materials to produce
        materials: u8,
        /// How much fuel to produce
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
    /// Rendezvous with another stack orbiting the same body
    ///
    /// Requires functional engine and one hex/turn of delta-v
    ///
    /// Target must not have any move order
    Rendezvous {
        /// Which stack
        stack: StackId,
        /// Who to rendezvous with
        target: StackId,
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
pub enum ModuleTransferTarget {
    /// An existing stack
    Existing(StackId),
    /// To the nth new stack this player is creating
    New(u32),
}

/// Where a resource transfer should go
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[expect(missing_docs)]
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
    fn cost(&self) -> i32 {
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

/// Why an order could not be applied
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

    /// Destination after takeoff is too far
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
    /// Residual left in the resource pool
    ResourcePoolResidual(StackId),
    /// Too many resources moved into module
    NotEnoughCapacity(StackId, ModuleId),
    /// Ordered to move multiple ways
    MultipleMoves,
    /// Rendezvous target moved
    TargetMoved,
}

#[cfg(feature = "server")]
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
                let Ok(order) = order_result else {
                    unreachable!("initialized with Ok");
                };
                if let Err(e) = order.validate_single(game_state, player) {
                    *order_result = Err(e);
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
                    let stack_ref = game_state.stacks.get(stack).expect("order is validated");
                    let position = stack_ref.position;
                    let velocity = stack_ref.velocity;
                    if moves_iter.any(|move_order| {
                        let Ok(Order::ModuleTransfer { stack, .. }) = move_order else {
                            unreachable!("pre-filtered")
                        };
                        let stack_ref = game_state.stacks.get(stack).expect("order is validated");
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
                                *miner_capacity_used.entry(*stack).or_default() +=
                                    *ore as u32 + *water as u32;
                                *skimmer_capacity_used.entry(*stack).or_default() += *fuel as u32;

                                // mote - both mining and skimming orders will not pass individual validation
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
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                    let required_miners = miner_capacity_used
                        .get(&stack)
                        .expect("constructed with same key")
                        .div_ceil(ModuleDetails::MINER_PRODUCTION_RATE);
                    if required_miners as usize > miner_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in skimmer_orders {
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                    let required_skimmers = skimmer_capacity_used
                        .get(&stack)
                        .expect("constructed with same key")
                        .div_ceil(ModuleDetails::FUEL_SKIMMER_PRODUCTION_RATE);
                    if required_skimmers as usize > skimmer_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in refinery_orders {
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                    let required_refineries = refinery_capacity_used
                        .get(&stack)
                        .expect("constructed with same key")
                        .div_ceil(ModuleDetails::REFINERY_CAPACITY);
                    if required_refineries as usize > refinery_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in hab_or_factory_orders {
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                    if orders.len() > hab_and_factory_count {
                        for order in orders {
                            *order = Err(OrderError::NotEnoughModules);
                        }
                    }
                }
                for (stack, orders) in factory_only_orders {
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                                resource_pool.materials -= game_state
                                    .stacks
                                    .get(target_stack)
                                    .expect("order is validated")
                                    .modules
                                    .get(target_module)
                                    .expect("order is validated")
                                    .dry_mass()
                                    as i32
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
                                resource_pool.materials += game_state
                                    .stacks
                                    .get(stack)
                                    .expect("order is validated")
                                    .modules
                                    .get(salvaged)
                                    .expect("order is validated")
                                    .dry_mass()
                                    as i32
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
                        for order in orders_by_player
                            .get_mut(&player)
                            .expect("constructed with same key")
                            .iter_mut()
                        {
                            **order = Err(OrderError::ResourcePoolResidual(stack))
                        }
                        break;
                    }
                }
                for ((player, stack, module), delta) in storage_deltas {
                    let module_ref = game_state
                        .stacks
                        .get(&stack)
                        .expect("order is validated")
                        .modules
                        .get(&module)
                        .expect("order is validated");
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
                        for order in orders_by_player
                            .get_mut(&player)
                            .expect("constructed with same key")
                            .iter_mut()
                        {
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
                    let stack_ref = game_state.stacks.get(&stack).expect("order is validated");
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
                                | Order::Rendezvous { stack, .. }
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

                // aggregate check that rendezvous target has no move order
                let mut order_by_stack: HashMap<StackId, &mut Result<&Order, OrderError>> =
                    HashMap::new();
                for (_, orders) in orders.iter_mut() {
                    for order in orders.iter_mut() {
                        match order {
                            Ok(
                                Order::Burn { stack, .. }
                                | Order::Rendezvous { stack, .. }
                                | Order::Land { stack, .. }
                                | Order::TakeOff { stack, .. },
                            ) => {
                                order_by_stack.insert(*stack, order);
                            }
                            _ => {
                                // no-op - should only be errors here
                            }
                        }
                    }
                }
                let mut conflicted = Vec::new();
                for (stack, order) in order_by_stack.iter() {
                    if let Ok(Order::Rendezvous { target, .. }) = order {
                        if order_by_stack.contains_key(target) {
                            conflicted.push(*stack);
                        }
                    }
                }
                for conflicted in conflicted {
                    **order_by_stack.get_mut(&conflicted).expect("saved key") =
                        Err(OrderError::TargetMoved);
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
            let Some(celestial_ref) = game_state.celestials.get(&celestial) else {
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
            let engine_count = stack_ref
                .modules
                .values()
                .filter(|module| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Engine
                        }
                    )
                })
                .count();

            if engine_count as f32 * ModuleDetails::ENGINE_THRUST / mass < min_accel {
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
            }
            Order::ModuleTransfer { stack, module, to } => {
                validate_phase(game_state, Phase::Logistics)?;
                let (stack_ref, _) = validate_module(*stack, *module, game_state, player)?;
                match to {
                    ModuleTransferTarget::Existing(target) => {
                        let target_ref = validate_stack(*target, game_state, player)?;
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
                    && !game_state.celestials.values().any(|celestial| {
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
                    && !game_state.celestials.values().any(|celestial| {
                        stack_ref.landed(celestial)
                            && matches!(
                                celestial.resources,
                                Resources::MiningBoth | Resources::MiningIce
                            )
                    })
                {
                    return Err(OrderError::NoResourceAccess);
                }
                if *fuel > 0
                    && !game_state.celestials.values().any(|celestial| {
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
                    let earth = game_state
                        .celestials
                        .get(&game_state.earth)
                        .expect("earth id should be valid");
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
                if game_state
                    .celestials
                    .values()
                    .any(|celestial| celestial.collides(stack_ref.position, target_ref.position))
                {
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
                    delta_v.norm().try_into().expect("absolute value applied"),
                    fuel_from,
                    0.0,
                )?;

                if game_state
                    .celestials
                    .values()
                    .any(|celestial| stack_ref.landed_with_gravity(celestial))
                {
                    return Err(OrderError::BurnWhileLanded);
                }

                // aggregate check that there's only one move order per stack
            }
            Order::Rendezvous {
                stack,
                target,
                fuel_from,
            } => {
                validate_phase(game_state, Phase::Movement)?;
                let stack_ref = validate_stack(*stack, game_state, player)?;
                let Some(target_ref) = game_state.stacks.get(target) else {
                    return Err(OrderError::InvalidStackId(*target));
                };

                validate_burn(*stack, stack_ref, 1, fuel_from, 0.0)?;

                let Some(orbited) = game_state
                    .celestials
                    .values()
                    .find(|celestial| stack_ref.orbiting(celestial))
                else {
                    return Err(OrderError::NotInOrbit);
                };

                if !target_ref.orbiting(orbited) {
                    return Err(OrderError::NotInOrbit);
                }

                // aggregate check that there's only one move order per stack
                // aggregate check that target has no move order
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
                match stack.modules.get_mut(module).expect("order is validated") {
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
                stacks.get_mut(stack).expect("order is validated").name = name.clone();
            }
            Order::ModuleTransfer { stack, module, to } => {
                let stack = stacks.get_mut(stack).expect("order is validated");
                let (module_id, module) = stack
                    .modules
                    .remove_entry(module)
                    .expect("order is validated");
                match to {
                    ModuleTransferTarget::Existing(stack_id) => {
                        stacks
                            .get_mut(stack_id)
                            .expect("order is validated")
                            .modules
                            .insert(module_id, module);
                    }
                    ModuleTransferTarget::New(new_count) => {
                        let new_id =
                            *new_stack_ids
                                .entry((player, *new_count))
                                .or_insert_with(|| {
                                    stack_id_generator.next().expect("should be infinite")
                                });
                        let position = stack.position;
                        let velocity = stack.velocity;
                        let owner = stack.owner;
                        let new_stack = stacks
                            .entry(new_id)
                            .or_insert_with(|| Stack::new(position, velocity, owner));
                        new_stack.modules.insert(module_id, module);
                    }
                }
            }
            Order::Board { stack, target } => {
                let transferred = std::mem::take(
                    &mut stacks.get_mut(target).expect("order is validated").modules,
                );
                stacks
                    .get_mut(stack)
                    .expect("order is validated")
                    .modules
                    .extend(transferred);
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
                let stack = stacks.get_mut(stack).expect("order is validated");
                match (from, to) {
                    (Some(from), ResourceTransferTarget::FloatingPool) => {
                        let from = stack.modules.get_mut(from).expect("order is validated");
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
                        let to = stack.modules.get_mut(to).expect("order is validated");
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
                    .expect("order is validated")
                    .modules
                    .get_mut(target_module)
                    .expect("order is validated")
                    .health = Health::Intact;
            }
            Order::Build { stack, module } => {
                let stack = stacks.get_mut(stack).expect("order is validated");
                stack.modules.insert(
                    module_id_generator.next().expect("should be infinite"),
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
                    .expect("order is validated")
                    .modules
                    .remove(salvaged)
                    .expect("order is validated");
            }
            Order::Shoot {
                stack,
                target,
                shots,
            } => {
                let start_pos = stacks.get(stack).expect("order is validated").position;
                let target = stacks.get_mut(target).expect("order is validated");
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
                    .expect("order is validated")
                    .modules
                    .get_mut(warhead)
                    .expect("order is validated")
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
                let stack = stacks.get_mut(stack).expect("order is validated");
                stack.velocity += *delta_v;

                drain_fuel(stack, fuel_from);
            }
            Order::Rendezvous {
                stack,
                target,
                fuel_from,
            } => {
                let target = stacks.get(target).expect("order is validated");
                let target_position = target.position;
                let target_velocity = target.velocity;

                let stack = stacks.get_mut(stack).expect("order is validated");
                stack.position = target_position;
                stack.velocity = target_velocity;

                drain_fuel(stack, fuel_from);
            }
            Order::Land {
                stack,
                on,
                fuel_from,
            } => {
                let stack = stacks.get_mut(stack).expect("order is validated");
                let on = game_state.celestials.get(on).expect("order is validated");
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
                let stack = stacks.get_mut(stack).expect("order is validated");
                let (position, velocity) = game_state
                    .celestials
                    .get(from)
                    .expect("order is validated")
                    .orbit_parameters(*clockwise)
                    .into_iter()
                    .find(|(position, _)| position == destination)
                    .expect("order is validated");
                stack.position = position;
                stack.velocity = velocity;

                drain_fuel(stack, fuel_from);
            }
        }
    }
}

#[cfg(feature = "server")]
/// A set of orders that can be applied to the referenced game state
pub struct ValidatedOrders<'a> {
    orders: HashMap<PlayerId, Vec<&'a Order>>,
    game_state: &'a GameState,
}

#[cfg(feature = "server")]
impl ValidatedOrders<'_> {
    /// Apply orders
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

#[cfg(test)]
mod tests {
    use super::*;
}
