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

use serde::{Deserialize, Serialize};

use crate::{PlayerId, Vec2};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    position: Vec2<i32>,
    velocity: Vec2<i32>,
    /// Current owner; can be changed by docking a habitat
    owner: PlayerId,
    /// Name as assigned by owner - may be blank
    name: String,
    modules: HashMap<ModuleId, Module>,
}

#[cfg(feature = "server")]
impl Stack {
    /// Create a starter stack for some player at some location with some velocity
    ///
    /// Contents = 2x hab, factory, refinery, miner, cargo hold (empty), tank (fuelled), 4x engine
    pub fn starter_stack(
        owner: PlayerId,
        position: Vec2<i32>,
        velocity: Vec2<i32>,
        name: String,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
    ) -> Self {
        Self {
            position,
            velocity,
            owner,
            name,
            modules: HashMap::from([
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_habitat(owner),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_habitat(owner),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_factory(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_refinery(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_miner(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_cargo_hold(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_fuel_tank(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_engine(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_engine(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_engine(),
                ),
                (
                    module_id_generator.next().expect("should be infinite"),
                    Module::new_engine(),
                ),
            ]),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackId(u32);

#[derive(Debug, Serialize, Deserialize)]
struct Module {
    health: Health,
    details: ModuleDetails,
}

#[cfg(feature = "server")]
impl Module {
    fn new(details: ModuleDetails) -> Self {
        Self {
            health: Health::Intact,
            details,
        }
    }

    fn new_miner() -> Self {
        Self::new(ModuleDetails::Miner)
    }

    fn new_fuel_skimmer() -> Self {
        Self::new(ModuleDetails::FuelSkimmer)
    }

    fn new_cargo_hold() -> Self {
        Self::new(ModuleDetails::CargoHold {
            ore: 0,
            materials: 0,
        })
    }

    fn new_tank() -> Self {
        Self::new(ModuleDetails::Tank { water: 0, fuel: 0 })
    }

    fn new_fuel_tank() -> Self {
        Self::new(ModuleDetails::Tank {
            water: 0,
            fuel: ModuleDetails::TANK_CAPACITY,
        })
    }

    fn new_engine() -> Self {
        Self::new(ModuleDetails::Engine)
    }

    fn new_warhead() -> Self {
        Self::new(ModuleDetails::Warhead { armed: false })
    }

    fn new_gun() -> Self {
        Self::new(ModuleDetails::Gun)
    }

    fn new_habitat(owner: PlayerId) -> Self {
        Self::new(ModuleDetails::Habitat { owner })
    }

    fn new_refinery() -> Self {
        Self::new(ModuleDetails::Refinery)
    }

    fn new_factory() -> Self {
        Self::new(ModuleDetails::Factory)
    }

    fn new_armour_plate() -> Self {
        Self::new(ModuleDetails::ArmourPlate)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Health {
    /// Operating normally
    Intact,
    /// Can't operate, but can be repaired
    Damaged,
    /// Can't operate or be repaired, but can be salvaged at a factory for 50% of materials
    Destroyed,
}

#[derive(Debug, Serialize, Deserialize)]
enum ModuleDetails {
    /// Miner - produces resources if landed on a body
    Miner,
    /// Fuel skimmer - produces fuel if orbiting a skimmable body
    FuelSkimmer,
    /// Cargo hold - holds ore and materials
    ///
    /// One unit = 0.1 tonnes
    CargoHold { ore: u8, materials: u8 },
    /// Tank - holds water and fuel
    ///
    /// One unit = 0.1 tonnes
    Tank { water: u8, fuel: u8 },
    /// Engine - produces 20 kN of thrust using 1 tonne of fuel; 1 m/s^2 = 1 hex/turn/turn
    ///
    /// Can burn fractional points of fuel, down to the 0.1 tonne
    Engine,
    /// Warhead - deals explosive attack to stack if intersecting (unless attached to a ship or disarmed)
    Warhead { armed: bool },
    /// Gun - 1/2 chance at 1 hex to deal 1 point of damage, 1/4 at 2 hexes, 1/8 at 3, and so on; guaranteed to hit at zero range
    Gun,
    /// Habitat - source of control and can effect repairs; stacks retain control until docked to by another habitat. Two habitats of different players can't dock.
    Habitat { owner: PlayerId },
    /// Refinery - turns water into fuel or ore into materials (must choose one)
    Refinery,
    /// Turns materials into modules and effects repairs
    Factory,
    /// Absorbs damage first
    ArmourPlate,
}

impl ModuleDetails {
    /// How many resources are produced per turn, in 0.1 tonnes
    const MINER_PRODUCTION_RATE: u8 = 10;
    /// Mass of a miner, in tonnes
    const MINER_MASS: u32 = 10;

    /// How much fuel is produced per turn, in 0.1 tonnes
    const FUEL_SKIMMER_PRODUCTION_RATE: u8 = 10;
    /// Mass of a fuel skimmer, in tonnes
    const FUEL_SKIMMER_MASS: u32 = 10;

    /// How many resources a cargo hold can contain, in 0.1 tonnes
    const CARGO_HOLD_CAPACITY: u8 = 200;
    /// Mass of a cargo hold, in tonnes
    const CARGO_HOLD_MASS: u32 = 1;

    /// How many resources a tank can contain, in 0.1 tonnes
    const TANK_CAPACITY: u8 = 200;
    /// Mass of a tank, in tonnes
    const TANK_MASS: u32 = 1;

    /// How much thrust does an engine produce per 0.1 tonne of fuel burned
    const ENGINE_THRUST: u32 = 2;
    /// How much fuel can an engine burn per turn, in 0.1 tonnes
    const ENGINE_MDOT: u32 = 10;
    /// Mass of an engine, in tonnes
    const ENGINE_MASS: u32 = 1;

    /// Mass of a warhead, in tonnes
    const WARHEAD_MASS: u32 = 1;

    /// Probability a gun will hit a target at one hex away (exponential falloff as distance increases)
    const GUN_RANGE_ONE_HIT_CHANCE: f32 = 0.5;
    /// Mass of a gun, in tonnes
    const GUN_MASS: u32 = 2;

    /// Mass of a habitat, in tonnes
    const HABITAT_MASS: u32 = 10;

    /// How much input can a refinery process, in 0.1 tonnes
    const REFINERY_CAPACITY: u8 = 50;
    /// Conversion ratio of ore to materials
    const REFINERY_ORE_PER_MATERIAL: u8 = 2;
    /// Conversion ratio of water to fuel
    const REFINERY_WATER_PER_FUEL: u8 = 2;
    /// Mass of a refinery, in tonnes
    const REFINERY_MASS: u32 = 20;

    /// Mass of a factory, in tonnes
    const FACTORY_MASS: u32 = 50;

    /// Mass of an armour plate, in tonnes
    const ARMOUR_PLATE_MASS: u32 = 1;
}

/// Id used to reference a module in a stack; unique across stacks
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u32);

#[cfg(test)]
mod tests {
    use super::*;
}
