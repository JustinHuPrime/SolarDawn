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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    stack::{ModuleId, StackId},
    Vec2,
};

/// An order that can be given
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Order {
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

/// Transfer modules from this stack to another
///
/// Logistics phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModuleTransfer {
    module: ModuleId,
    to: ModuleTransferTarget,
}

/// Where a transferred module should go
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Board {
    target: StackId,
}

/// Miner/skimmer activation
///
/// Aggregate order
///
/// Puts resources into floating resource pool
///
/// Logistics phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Isru {
    ore: u8,
    water: u8,
    fuel: u8,
}

/// Transfer resources
///
/// Logistics phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ResourceTransfer {
    /// If None, indicates that this a transfer from the floating pool
    from: Option<ModuleId>,
    to_module: ResourceTransferTarget,
    ore: u8,
    materials: u8,
    water: u8,
    fuel: u8,
}

/// Where a resource transfer should go
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Repair {
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Refine {
    materials: u8,
    fuel: u8,
}

/// Build a module
///
/// Aggregate order
///
/// Logistics phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Build {
    module: ModuleType,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Salvage {
    salvaged: ModuleId,
}

/// Shoot some target
///
/// Combat phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Shoot {
    gun: ModuleId,
    target: StackId,
}

/// Set warhead arming status
///
/// Fails if the warhead is on a stack with a habitat
///
/// Combat phase
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Arm {
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Burn {
    delta_v: Vec2<i32>,
    fuel_from: Vec<(ModuleId, u8)>,
}

#[cfg(test)]
mod tests {
    use super::*;
}
