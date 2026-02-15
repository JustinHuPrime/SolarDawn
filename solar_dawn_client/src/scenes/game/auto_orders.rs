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

//! Automatic order generation for ISRU and refine operations

use solar_dawn_common::{
    GameState, Phase, PlayerId,
    celestial::Resources,
    order::{Order, ResourceTransferTarget},
    stack::{Health, Module, ModuleDetails, ModuleId, StackId},
};

use super::{AutoIsruSetting, AutoRefineSetting, ClientGameSettings};

/// Generate automatic orders based on client settings
pub fn generate_automatic_orders(
    game_state: &GameState,
    settings: &ClientGameSettings,
    me: PlayerId,
) -> Vec<(Order, bool)> {
    let mut auto_orders = Vec::new();

    // Only generate automatic orders during logistics phase
    if game_state.phase != Phase::Logistics {
        return auto_orders;
    }

    // Iterate through all player's stacks
    for (&stack_id, stack) in game_state.stacks.iter() {
        if stack.owner != me {
            continue;
        }

        // Generate ISRU orders
        if let Some(&isru_setting) = settings.auto_isru.get(&stack_id) {
            if let Some(orders) = generate_isru_orders(game_state, stack_id, stack, isru_setting) {
                auto_orders.extend(orders.into_iter().map(|order| (order, true)));
            }
        }

        // Generate Refine orders
        if let Some(&refine_setting) = settings.auto_refine.get(&stack_id) {
            if let Some(orders) =
                generate_refine_orders(game_state, stack_id, stack, refine_setting)
            {
                auto_orders.extend(orders.into_iter().map(|order| (order, true)));
            }
        }
    }

    auto_orders
}

fn generate_isru_orders(
    game_state: &GameState,
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
    setting: AutoIsruSetting,
) -> Option<Vec<Order>> {
    match setting {
        AutoIsruSetting::None => None,
        AutoIsruSetting::Water => generate_isru_water(game_state, stack_id, stack),
        AutoIsruSetting::Ore => generate_isru_ore(game_state, stack_id, stack),
    }
}

fn generate_isru_water(
    game_state: &GameState,
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
) -> Option<Vec<Order>> {
    // Check if stack is landed on a celestial with water
    let celestial = game_state
        .celestials
        .get_by_position(stack.position)
        .filter(|(_, c)| stack.landed(c))?
        .1;

    if !matches!(
        celestial.resources,
        Resources::MiningBoth | Resources::MiningWater
    ) {
        return None;
    }

    // Count functional miners
    let miner_count = stack
        .modules
        .values()
        .filter(|m| {
            matches!(
                m,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Miner
                }
            )
        })
        .count() as u32;

    if miner_count == 0 {
        return None;
    }

    // Calculate capacity: each miner can produce MINER_PRODUCTION_RATE (100) units per turn
    let production_capacity = miner_count * ModuleDetails::MINER_PRODUCTION_RATE;

    // Calculate storage capacity for water
    let storage_capacity = calculate_water_storage_capacity(&stack.modules);

    // Take the minimum of production and storage capacity
    let water_to_produce = production_capacity.min(storage_capacity);

    if water_to_produce == 0 {
        return None;
    }

    let mut orders = vec![Order::Isru {
        stack: stack_id,
        ore: 0,
        water: water_to_produce,
        fuel: 0,
    }];

    // Generate storage orders
    orders.extend(generate_storage_orders(
        stack_id,
        &stack.modules,
        0,
        water_to_produce,
        0,
        0,
    ));

    Some(orders)
}

fn generate_isru_ore(
    game_state: &GameState,
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
) -> Option<Vec<Order>> {
    // Check if stack is landed on a celestial with ore
    let celestial = game_state
        .celestials
        .get_by_position(stack.position)
        .filter(|(_, c)| stack.landed(c))?
        .1;

    if !matches!(
        celestial.resources,
        Resources::MiningBoth | Resources::MiningOre
    ) {
        return None;
    }

    // Count functional miners
    let miner_count = stack
        .modules
        .values()
        .filter(|m| {
            matches!(
                m,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Miner
                }
            )
        })
        .count() as u32;

    if miner_count == 0 {
        return None;
    }

    // Calculate capacity
    let production_capacity = miner_count * ModuleDetails::MINER_PRODUCTION_RATE;

    // Calculate storage capacity for ore
    let storage_capacity = calculate_ore_storage_capacity(&stack.modules);

    // Take the minimum
    let ore_to_produce = production_capacity.min(storage_capacity);

    if ore_to_produce == 0 {
        return None;
    }

    let mut orders = vec![Order::Isru {
        stack: stack_id,
        ore: ore_to_produce,
        water: 0,
        fuel: 0,
    }];

    // Generate storage orders
    orders.extend(generate_storage_orders(
        stack_id,
        &stack.modules,
        ore_to_produce,
        0,
        0,
        0,
    ));

    Some(orders)
}

fn generate_refine_orders(
    _game_state: &GameState,
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
    setting: AutoRefineSetting,
) -> Option<Vec<Order>> {
    match setting {
        AutoRefineSetting::None => None,
        AutoRefineSetting::Fuel => generate_refine_fuel(stack_id, stack),
        AutoRefineSetting::Materials => generate_refine_materials(stack_id, stack),
    }
}

fn generate_refine_fuel(
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
) -> Option<Vec<Order>> {
    // Count functional refineries
    let refinery_count = stack
        .modules
        .values()
        .filter(|m| {
            matches!(
                m,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Refinery
                }
            )
        })
        .count() as u32;

    if refinery_count == 0 {
        return None;
    }

    // Calculate refinery capacity
    let refinery_capacity = refinery_count * ModuleDetails::REFINERY_CAPACITY;

    // Calculate how much water is available in modules
    let available_water = calculate_available_water(&stack.modules);

    // Calculate fuel storage capacity
    let fuel_storage = calculate_fuel_storage_capacity(&stack.modules);

    // Fuel production is limited by:
    // 1. Refinery capacity
    // 2. Available water (need REFINERY_WATER_PER_FUEL water per fuel)
    // 3. Storage capacity for fuel
    let max_fuel_from_water = available_water / ModuleDetails::REFINERY_WATER_PER_FUEL as u32;
    let fuel_to_produce = refinery_capacity
        .min(max_fuel_from_water)
        .min(fuel_storage) as u8;

    if fuel_to_produce == 0 {
        return None;
    }

    let mut orders = Vec::new();

    // Calculate water needed for refining
    let water_needed = fuel_to_produce as u32 * ModuleDetails::REFINERY_WATER_PER_FUEL as u32;

    // Generate ResourceTransfer orders to move water from modules to floating pool
    orders.extend(transfer_water_to_floating_pool(
        stack_id,
        &stack.modules,
        water_needed,
    ));

    // Generate the Refine order
    orders.push(Order::Refine {
        stack: stack_id,
        materials: 0,
        fuel: fuel_to_produce,
    });

    // Generate storage orders to move fuel from floating pool to modules
    orders.extend(generate_storage_orders(
        stack_id,
        &stack.modules,
        0,
        0,
        0,
        fuel_to_produce as u32,
    ));

    Some(orders)
}

fn generate_refine_materials(
    stack_id: StackId,
    stack: &solar_dawn_common::stack::Stack,
) -> Option<Vec<Order>> {
    // Count functional refineries
    let refinery_count = stack
        .modules
        .values()
        .filter(|m| {
            matches!(
                m,
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Refinery
                }
            )
        })
        .count() as u32;

    if refinery_count == 0 {
        return None;
    }

    // Calculate refinery capacity
    let refinery_capacity = refinery_count * ModuleDetails::REFINERY_CAPACITY;

    // Calculate how much ore is available in modules
    let available_ore = calculate_available_ore(&stack.modules);

    // Calculate materials storage capacity
    let materials_storage = calculate_materials_storage_capacity(&stack.modules);

    // Materials production is limited by:
    // 1. Refinery capacity
    // 2. Available ore (need REFINERY_ORE_PER_MATERIAL ore per material)
    // 3. Storage capacity for materials
    let max_materials_from_ore = available_ore / ModuleDetails::REFINERY_ORE_PER_MATERIAL as u32;
    let materials_to_produce = refinery_capacity
        .min(max_materials_from_ore)
        .min(materials_storage) as u8;

    if materials_to_produce == 0 {
        return None;
    }

    let mut orders = Vec::new();

    // Calculate ore needed for refining
    let ore_needed = materials_to_produce as u32 * ModuleDetails::REFINERY_ORE_PER_MATERIAL as u32;

    // Generate ResourceTransfer orders to move ore from modules to floating pool
    orders.extend(transfer_ore_to_floating_pool(
        stack_id,
        &stack.modules,
        ore_needed,
    ));

    // Generate the Refine order
    orders.push(Order::Refine {
        stack: stack_id,
        materials: materials_to_produce,
        fuel: 0,
    });

    // Generate storage orders to move materials from floating pool to modules
    orders.extend(generate_storage_orders(
        stack_id,
        &stack.modules,
        0,
        0,
        materials_to_produce as u32,
        0,
    ));

    Some(orders)
}

/// Calculate available water in modules (in 0.1 tonnes)
fn calculate_available_water(
    modules: &std::collections::BTreeMap<ModuleId, Module>,
) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::Tank { water, .. } => Some(water as u32),
            _ => None,
        })
        .sum()
}

/// Calculate available ore in modules (in 0.1 tonnes)
fn calculate_available_ore(modules: &std::collections::BTreeMap<ModuleId, Module>) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::CargoHold { ore, .. } => Some(ore as u32),
            _ => None,
        })
        .sum()
}

/// Calculate water storage capacity (in 0.1 tonnes)
fn calculate_water_storage_capacity(
    modules: &std::collections::BTreeMap<ModuleId, Module>,
) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::Tank { water, fuel } => {
                let used = water as i32 + fuel as i32;
                if used < ModuleDetails::TANK_CAPACITY {
                    Some((ModuleDetails::TANK_CAPACITY - used) as u32)
                } else {
                    Some(0)
                }
            }
            _ => None,
        })
        .sum()
}

/// Calculate ore storage capacity (in 0.1 tonnes)
fn calculate_ore_storage_capacity(
    modules: &std::collections::BTreeMap<ModuleId, Module>,
) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::CargoHold { ore, materials } => {
                let used = ore as i32 + materials as i32;
                if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    Some((ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32)
                } else {
                    Some(0)
                }
            }
            _ => None,
        })
        .sum()
}

/// Calculate fuel storage capacity (in 0.1 tonnes)
fn calculate_fuel_storage_capacity(
    modules: &std::collections::BTreeMap<ModuleId, Module>,
) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::Tank { water, fuel } => {
                let used = water as i32 + fuel as i32;
                if used < ModuleDetails::TANK_CAPACITY {
                    Some((ModuleDetails::TANK_CAPACITY - used) as u32)
                } else {
                    Some(0)
                }
            }
            _ => None,
        })
        .sum()
}

/// Calculate materials storage capacity (in 0.1 tonnes)
fn calculate_materials_storage_capacity(
    modules: &std::collections::BTreeMap<ModuleId, Module>,
) -> u32 {
    modules
        .values()
        .filter_map(|m| match m.details {
            ModuleDetails::CargoHold { ore, materials } => {
                let used = ore as i32 + materials as i32;
                if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    Some((ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32)
                } else {
                    Some(0)
                }
            }
            _ => None,
        })
        .sum()
}

/// Generate ResourceTransfer orders to store produced resources
///
/// Storage priority:
/// 1. Containers that already hold some of the resource
/// 2. Empty containers
/// 3. Containers with other resources
fn generate_storage_orders(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    ore: u32,
    water: u32,
    materials: u32,
    fuel: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    if ore > 0 {
        orders.extend(store_ore(stack_id, modules, ore));
    }
    if water > 0 {
        orders.extend(store_water(stack_id, modules, water));
    }
    if materials > 0 {
        orders.extend(store_materials(stack_id, modules, materials));
    }
    if fuel > 0 {
        orders.extend(store_fuel(stack_id, modules, fuel));
    }

    orders
}

/// Transfer ore from modules to floating pool
fn transfer_ore_to_floating_pool(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, .. } = module.details {
            let to_transfer = (ore as u32).min(amount).min(255);
            if to_transfer > 0 {
                orders.push(Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(module_id),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: to_transfer as u8,
                    water: 0,
                    materials: 0,
                    fuel: 0,
                });
                amount -= to_transfer;
            }
        }
    }

    orders
}

/// Transfer water from modules to floating pool
fn transfer_water_to_floating_pool(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, .. } = module.details {
            let to_transfer = (water as u32).min(amount).min(255);
            if to_transfer > 0 {
                orders.push(Order::ResourceTransfer {
                    stack: stack_id,
                    from: Some(module_id),
                    to: ResourceTransferTarget::FloatingPool,
                    ore: 0,
                    water: to_transfer as u8,
                    materials: 0,
                    fuel: 0,
                });
                amount -= to_transfer;
            }
        }
    }

    orders
}

fn store_ore(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    // Priority 1: Cargo holds that already have ore
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if ore > 0 {
                let used = ore as i32 + materials as i32;
                let capacity = if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    (ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: to_store as u8,
                        water: 0,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 2: Empty cargo holds
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if ore == 0 && materials == 0 {
                let capacity = ModuleDetails::CARGO_HOLD_CAPACITY as u32;
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: to_store as u8,
                        water: 0,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 3: Cargo holds with other resources (materials)
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if ore == 0 && materials > 0 {
                let used = ore as i32 + materials as i32;
                let capacity = if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    (ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: to_store as u8,
                        water: 0,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    orders
}

fn store_water(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    // Priority 1: Tanks that already have water
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if water > 0 {
                let used = water as i32 + fuel as i32;
                let capacity = if used < ModuleDetails::TANK_CAPACITY {
                    (ModuleDetails::TANK_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: to_store as u8,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 2: Empty tanks
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if water == 0 && fuel == 0 {
                let capacity = ModuleDetails::TANK_CAPACITY as u32;
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: to_store as u8,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 3: Tanks with fuel
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if water == 0 && fuel > 0 {
                let used = water as i32 + fuel as i32;
                let capacity = if used < ModuleDetails::TANK_CAPACITY {
                    (ModuleDetails::TANK_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: to_store as u8,
                        materials: 0,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    orders
}

fn store_materials(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    // Priority 1: Cargo holds that already have materials
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if materials > 0 {
                let used = ore as i32 + materials as i32;
                let capacity = if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    (ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: to_store as u8,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 2: Empty cargo holds
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if ore == 0 && materials == 0 {
                let capacity = ModuleDetails::CARGO_HOLD_CAPACITY as u32;
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: to_store as u8,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 3: Cargo holds with ore
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::CargoHold { ore, materials } = module.details {
            if materials == 0 && ore > 0 {
                let used = ore as i32 + materials as i32;
                let capacity = if used < ModuleDetails::CARGO_HOLD_CAPACITY {
                    (ModuleDetails::CARGO_HOLD_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: to_store as u8,
                        fuel: 0,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    orders
}

fn store_fuel(
    stack_id: StackId,
    modules: &std::collections::BTreeMap<ModuleId, Module>,
    mut amount: u32,
) -> Vec<Order> {
    let mut orders = Vec::new();

    // Priority 1: Tanks that already have fuel
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if fuel > 0 {
                let used = water as i32 + fuel as i32;
                let capacity = if used < ModuleDetails::TANK_CAPACITY {
                    (ModuleDetails::TANK_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: 0,
                        fuel: to_store as u8,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 2: Empty tanks
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if water == 0 && fuel == 0 {
                let capacity = ModuleDetails::TANK_CAPACITY as u32;
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: 0,
                        fuel: to_store as u8,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    // Priority 3: Tanks with water
    for (&module_id, module) in modules {
        if amount == 0 {
            break;
        }
        if let ModuleDetails::Tank { water, fuel } = module.details {
            if fuel == 0 && water > 0 {
                let used = water as i32 + fuel as i32;
                let capacity = if used < ModuleDetails::TANK_CAPACITY {
                    (ModuleDetails::TANK_CAPACITY - used) as u32
                } else {
                    0
                };
                let to_store = capacity.min(amount).min(255);
                if to_store > 0 {
                    orders.push(Order::ResourceTransfer {
                        stack: stack_id,
                        from: None,
                        to: ResourceTransferTarget::Module(module_id),
                        ore: 0,
                        water: 0,
                        materials: 0,
                        fuel: to_store as u8,
                    });
                    amount -= to_store;
                }
            }
        }
    }

    orders
}
