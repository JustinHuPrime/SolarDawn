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

use serde::{Deserialize, Serialize};

use crate::{
    stack::{ModuleId, StackId},
    Vec2,
};

/// An order that can be given
#[derive(Serialize, Deserialize)]
pub enum Order {
    NameStack(NameStack),
    ModuleTransfer(ModuleTransfer),
    Board(Board),
    Isru(Isru),
    ResourceTransfer(ResourceTransfer),
    Repair(Repair),
    Refine(Refine),
    Build(Build),
    Salvage(Salvage),
    Shoot(Shoot),
    Arm(Arm),
    Burn(Burn),
}

/// Name a stack
///
/// Always valid
#[derive(Serialize, Deserialize)]
struct NameStack {
    name: String,
}

/// Transfer modules from this stack to another
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct ModuleTransfer {
    module: ModuleId,
    to: ModuleTransferTarget,
}

/// Where a transferred module should go
#[derive(Serialize, Deserialize)]
enum ModuleTransferTarget {
    /// An existing stack
    Existing(StackId),
    /// To the nth new stack this player is creating
    New(u32),
}

/// Forcefully dock another stack to this stack
///
/// Stack must have no functioning habitats
///
/// Interrupts any orders the target might have been given
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Board {
    target: StackId,
}

/// Miner/skimmer activation
///
/// Aggregate order
///
/// Puts resources into floating resource pool
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Isru {
    ore: u8,
    water: u8,
    fuel: u8,
}

/// Transfer resources
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct ResourceTransfer {
    /// If None, indicates that this a transfer from the floating pool
    from: Option<ModuleId>,
    to_module: ResourceTransferTarget,
    ore: u8,
    materials: u8,
    water: u8,
    fuel: u8,
}

/// Where a resource transfer should go
#[derive(Serialize, Deserialize)]
enum ResourceTransferTarget {
    /// This stack's floating pool
    FloatingPool,
    /// Jettison
    Jettison,
    /// A module in this stack
    Module(ModuleId),
    /// That stack's floating pool
    Stack(StackId),
}

/// Repair another module
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Repair {
    repairer: ModuleId,
    target_stack: StackId,
    target_module: ModuleId,
}

/// Refine some resources
///
/// Aggregate order
///
/// A stack may not refine more resources than it has refinery capacity
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Refine {
    materials: u8,
    fuel: u8,
}

/// Build a module
///
/// Aggregate order
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Build {
    module: ModuleType,
}

#[derive(Serialize, Deserialize)]
enum ModuleType {
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

/// Salvage a module
///
/// Logistics phase
#[derive(Serialize, Deserialize)]
struct Salvage {
    salvaged: ModuleId,
}

/// Shoot some target
///
/// Combat phase
#[derive(Serialize, Deserialize)]
struct Shoot {
    gun: ModuleId,
    target: StackId,
}

/// Set warhead arming status
///
/// Fails if the warhead is on a stack with a habitat
///
/// Combat phase
#[derive(Serialize, Deserialize)]
struct Arm {
    warhead: ModuleId,
    armed: bool,
}

/// Change velocity
///
/// Aggregate order
///
/// May not have more than one per stack in a turn
///
/// Stack must have enough thrust to generate the requested delta-v
///
/// Fuel tanks must have enough fuel to cover the fuel consumption
///
/// Movement phase
#[derive(Serialize, Deserialize)]
struct Burn {
    delta_v: Vec2<i32>,
    fuel_from: Vec<(ModuleId, u8)>,
}

#[cfg(test)]
mod tests {
    use super::*;
}
