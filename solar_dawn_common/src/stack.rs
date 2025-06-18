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

#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::celestial::Celestial;
use crate::{PlayerId, Vec2};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stack {
    pub position: Vec2<i32>,
    pub velocity: Vec2<i32>,
    /// Current owner; can be changed by docking a habitat
    pub owner: PlayerId,
    /// Name as assigned by owner - may be blank
    pub name: String,
    pub modules: HashMap<ModuleId, Module>,
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

    /// Create a new empty stack
    pub fn new(position: Vec2<i32>, velocity: Vec2<i32>, owner: PlayerId) -> Self {
        Self {
            position,
            velocity,
            owner,
            name: String::default(),
            modules: HashMap::new(),
        }
    }

    /// Is this stack rendezvoused with the other stack?
    pub fn rendezvoused_with(&self, other: &Stack) -> bool {
        self.position == other.position && self.velocity == other.velocity
    }

    /// Is this stack orbiting the target celestial?
    pub fn orbiting(&self, celestial: &Celestial) -> bool {
        celestial.orbit_gravity
            && (self.position - celestial.position).norm() == 1
            && self.velocity.norm() == 1
            && (self.position + self.velocity - celestial.position).norm() == 1
    }

    /// Is this stack landed on the target celestial?
    ///
    /// Broad terms landed - also counts rendezvouses with non-gravity celestials
    pub fn landed(&self, celestial: &Celestial) -> bool {
        self.position == celestial.position && self.velocity.norm() == 0
    }

    /// Is this stack landed on a celestial with gravity?
    ///
    /// Strict terms landed - doesn't count rendezvouses with non-gravity celestials
    pub fn landed_with_gravity(&self, celestial: &Celestial) -> bool {
        celestial.orbit_gravity && self.position == celestial.position && self.velocity.norm() == 0
    }

    /// Get the wet mass in tonnes
    pub fn mass(&self) -> f32 {
        self.modules.values().map(|module| module.mass()).sum()
    }

    /// How far through the turn does a close approach between this and the other stack happen, if any?
    ///
    /// By close, we mean WARHEAD_RANGE
    pub fn closest_approach(&self, other: &Stack) -> Option<f32> {
        // ||O_1 + V_1t - O_2 - V_2t|| <= R
        // ||O_1 - O_2 + (V_1 - V_2)t|| <= R
        // (O_1 - O_2 + (V_1 - V_2)t)^2 <= R^2
        // let dO = O_1 - O_2, dV = V_1 - V_2
        // (dO + dVt)^2 <= R^2
        // dV^2t^2 + 2dOdVt + dO^2 <= R^2
        // dV^2t^2 + 2dOdVt + dO^2 - R^2 <= 0
        let self_origin = self.position.cartesian();
        let self_velocity = self.velocity.cartesian();
        let other_origin = other.position.cartesian();
        let other_velocity = other.velocity.cartesian();

        let delta_origin = (
            self_origin.0 - other_origin.0,
            self_origin.1 - other_origin.1,
        );
        let delta_velocity = (
            self_velocity.0 - other_velocity.0,
            self_velocity.1 - other_velocity.1,
        );

        let a = delta_velocity.0 * delta_velocity.0 + delta_velocity.1 + delta_velocity.1;
        let b = 2.0 * (delta_origin.0 * delta_velocity.0 + delta_origin.1 * delta_velocity.1);
        let c = delta_origin.0 * delta_origin.0 + delta_origin.1 * delta_origin.1
            - ModuleDetails::WARHEAD_RANGE * ModuleDetails::WARHEAD_RANGE;

        // case 1: is already less than range at t = 0
        if c <= 0.0 {
            return Some(0.0);
        }

        // solve quadratic
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            // no approach
            return None;
        }
        let discriminant = discriminant.sqrt();

        // case 2: starts outside range, intersects one or two points at some time
        let t1 = (-b - discriminant) / (2.0 * a);
        let t2 = (-b + discriminant) / (2.0 * a);
        if (0.0..=1.0).contains(&t1) {
            Some(t1)
        } else if (0.0..=1.0).contains(&t2) {
            Some(t2)
        } else {
            None
        }
    }

    /// Is the other stack within range at time t
    ///
    /// By in range, we mean WARHEAD_RANGE
    pub fn in_range(&self, t: f32, other: &Stack) -> bool {
        let self_position = self.position.cartesian();
        let self_velocity = self.velocity.cartesian();
        let self_position = (
            self_position.0 + self_velocity.0 * t,
            self_position.1 + self_velocity.1 * t,
        );

        let other_position = other.position.cartesian();
        let other_velocity = other.velocity.cartesian();
        let other_position = (
            other_position.0 + other_velocity.0 * t,
            other_position.1 + other_velocity.1 * t,
        );

        let delta = (
            self_position.0 - other_position.0,
            self_position.1 - other_position.1,
        );

        delta.0 * delta.0 + delta.1 * delta.1
            < ModuleDetails::WARHEAD_RANGE * ModuleDetails::WARHEAD_RANGE
    }

    /// Deal some damage to the stack
    pub fn do_damage(&mut self, mut hits: u32, rng: &mut impl Rng) {
        let mut armour_plates = self
            .modules
            .values_mut()
            .filter_map(|module| {
                if let Module {
                    health: health @ Health::Intact,
                    details: ModuleDetails::ArmourPlate,
                } = module
                {
                    Some(health)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        while !armour_plates.is_empty() && hits > 0 {
            let damaged = rng.random_range(0..armour_plates.len());
            armour_plates[damaged].damage();

            armour_plates.swap_remove(damaged);

            hits -= 1;
        }

        let mut modules = self
            .modules
            .values_mut()
            .filter_map(|module| {
                if let Module {
                    health: health @ Health::Intact | health @ Health::Damaged,
                    ..
                } = module
                {
                    Some(health)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        while !modules.is_empty() && hits > 0 {
            let damaged = rng.random_range(0..modules.len());
            modules[damaged].damage();

            if matches!(modules[damaged], Health::Destroyed) {
                modules.swap_remove(damaged);
            }

            hits -= 1;
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackId(u32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    pub health: Health,
    pub details: ModuleDetails,
}

#[cfg(feature = "server")]
impl Module {
    fn new(details: ModuleDetails) -> Self {
        Self {
            health: Health::Intact,
            details,
        }
    }

    pub fn new_miner() -> Self {
        Self::new(ModuleDetails::Miner)
    }

    pub fn new_fuel_skimmer() -> Self {
        Self::new(ModuleDetails::FuelSkimmer)
    }

    pub fn new_cargo_hold() -> Self {
        Self::new(ModuleDetails::CargoHold {
            ore: 0,
            materials: 0,
        })
    }

    pub fn new_tank() -> Self {
        Self::new(ModuleDetails::Tank { water: 0, fuel: 0 })
    }

    fn new_fuel_tank() -> Self {
        Self::new(ModuleDetails::Tank {
            water: 0,
            fuel: ModuleDetails::TANK_CAPACITY as u8,
        })
    }

    pub fn new_engine() -> Self {
        Self::new(ModuleDetails::Engine)
    }

    pub fn new_warhead() -> Self {
        Self::new(ModuleDetails::Warhead { armed: false })
    }

    pub fn new_gun() -> Self {
        Self::new(ModuleDetails::Gun)
    }

    pub fn new_habitat(owner: PlayerId) -> Self {
        Self::new(ModuleDetails::Habitat { owner })
    }

    pub fn new_refinery() -> Self {
        Self::new(ModuleDetails::Refinery)
    }

    pub fn new_factory() -> Self {
        Self::new(ModuleDetails::Factory)
    }

    pub fn new_armour_plate() -> Self {
        Self::new(ModuleDetails::ArmourPlate)
    }

    fn mass(&self) -> f32 {
        self.details.mass()
    }

    pub fn dry_mass(&self) -> u32 {
        self.details.dry_mass()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Health {
    /// Operating normally
    Intact,
    /// Can't operate, but can be repaired
    Damaged,
    /// Can't operate or be repaired, but can be salvaged at a factory for 50% of materials
    Destroyed,
}

#[cfg(feature = "server")]
impl Health {
    pub fn damage(&mut self) {
        match self {
            Health::Intact => *self = Health::Damaged,
            Health::Damaged => *self = Health::Destroyed,
            Health::Destroyed => panic!("tried to damage destroyed module"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleDetails {
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
    /// Warhead - deals explosive attack to stack if intersecting (unless attached to a hab or disarmed)
    Warhead { armed: bool },
    /// Gun - 1/2 chance at 1 hex to deal 1 point of damage, 1/4 at 2 hexes, 1/8 at 3, and so on; guaranteed to hit at zero range
    Gun,
    /// Habitat - source of control and can effect repairs; stacks retain control until docked to by another habitat. Two habitats of different players can't dock.
    Habitat { owner: PlayerId },
    /// Refinery - turns water into fuel or ore into materials
    Refinery,
    /// Turns materials into modules, salvages modules, and effects repairs
    Factory,
    /// Absorbs damage first
    ArmourPlate,
}

impl ModuleDetails {
    /// How many resources are produced per turn, in 0.1 tonnes
    pub const MINER_PRODUCTION_RATE: u32 = 10;
    /// Mass of a miner, in tonnes
    pub const MINER_MASS: u32 = 10;

    /// How much fuel is produced per turn, in 0.1 tonnes
    pub const FUEL_SKIMMER_PRODUCTION_RATE: u32 = 10;
    /// Mass of a fuel skimmer, in tonnes
    pub const FUEL_SKIMMER_MASS: u32 = 10;

    /// How many resources a cargo hold can contain, in 0.1 tonnes
    pub const CARGO_HOLD_CAPACITY: i32 = 200;
    /// Mass of a cargo hold, in tonnes
    pub const CARGO_HOLD_MASS: u32 = 1;

    /// How many resources a tank can contain, in 0.1 tonnes
    pub const TANK_CAPACITY: i32 = 200;
    /// Mass of a tank, in tonnes
    pub const TANK_MASS: u32 = 1;

    /// How much impulse (in tonne·hex/turn aka kN·turn) does an engine produce per 0.1 tonne of fuel burned
    pub const ENGINE_SPECIFIC_IMPULSE: u32 = 2;
    /// How much thrust can an engine produce, in kN
    pub const ENGINE_THRUST: f32 = 40.0;
    /// Mass of an engine, in tonnes
    pub const ENGINE_MASS: u32 = 1;

    /// Mass of a warhead, in tonnes
    pub const WARHEAD_MASS: u32 = 1;
    /// Fraction of modules a warhead damages
    pub const WARHEAD_DAMAGE_FRACTION: u32 = 3;
    /// Range at which a warhead will hit, in hex major radii = √3/2
    pub const WARHEAD_RANGE: f32 = 0.866_025_4;

    /// Probability a gun will hit a target at one hex away (exponential falloff as distance increases)
    pub const GUN_RANGE_ONE_HIT_CHANCE: f32 = 0.5;
    /// Mass of a gun, in tonnes
    pub const GUN_MASS: u32 = 2;

    /// Mass of a habitat, in tonnes
    pub const HABITAT_MASS: u32 = 10;

    /// How much output can a refinery produce, in 0.1 tonnes
    pub const REFINERY_CAPACITY: u32 = 25;
    /// Conversion ratio of ore to materials
    pub const REFINERY_ORE_PER_MATERIAL: i32 = 2;
    /// Conversion ratio of water to fuel
    pub const REFINERY_WATER_PER_FUEL: i32 = 2;
    /// Mass of a refinery, in tonnes
    pub const REFINERY_MASS: u32 = 20;

    /// Mass of a factory, in tonnes
    pub const FACTORY_MASS: u32 = 50;

    /// Mass of an armour plate, in tonnes
    pub const ARMOUR_PLATE_MASS: u32 = 1;

    /// Fraction of mass to repair a module
    pub const REPAIR_FRACTION: i32 = 10;
    /// Fraction of mass returned when salvaging
    pub const SALVAGE_FRACTION: i32 = 2;

    fn mass(&self) -> f32 {
        match self {
            ModuleDetails::Miner => Self::MINER_MASS as f32,
            ModuleDetails::FuelSkimmer => Self::FUEL_SKIMMER_MASS as f32,
            ModuleDetails::CargoHold { ore, materials } => {
                Self::CARGO_HOLD_MASS as f32 + 0.1 * (*ore as f32 + *materials as f32)
            }
            ModuleDetails::Tank { water, fuel } => {
                Self::TANK_MASS as f32 + 0.1 * (*water as f32 + *fuel as f32)
            }
            ModuleDetails::Engine => Self::ENGINE_MASS as f32,
            ModuleDetails::Warhead { .. } => Self::WARHEAD_MASS as f32,
            ModuleDetails::Gun => Self::GUN_MASS as f32,
            ModuleDetails::Habitat { .. } => Self::HABITAT_MASS as f32,
            ModuleDetails::Refinery => Self::REFINERY_MASS as f32,
            ModuleDetails::Factory => Self::FACTORY_MASS as f32,
            ModuleDetails::ArmourPlate => Self::ARMOUR_PLATE_MASS as f32,
        }
    }

    fn dry_mass(&self) -> u32 {
        match self {
            ModuleDetails::Miner => Self::MINER_MASS,
            ModuleDetails::FuelSkimmer => Self::FUEL_SKIMMER_MASS,
            ModuleDetails::CargoHold { .. } => Self::CARGO_HOLD_MASS,
            ModuleDetails::Tank { .. } => Self::TANK_MASS,
            ModuleDetails::Engine => Self::ENGINE_MASS,
            ModuleDetails::Warhead { .. } => Self::WARHEAD_MASS,
            ModuleDetails::Gun => Self::GUN_MASS,
            ModuleDetails::Habitat { .. } => Self::HABITAT_MASS,
            ModuleDetails::Refinery => Self::REFINERY_MASS,
            ModuleDetails::Factory => Self::FACTORY_MASS,
            ModuleDetails::ArmourPlate => Self::ARMOUR_PLATE_MASS,
        }
    }
}

/// Id used to reference a module in a stack; unique across stacks
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u32);

#[cfg(test)]
mod tests {
    use super::*;
}
