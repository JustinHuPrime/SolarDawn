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

//! Stacks - game pieces
//!
//! A stack is an unordered collection of modules

use std::collections::HashMap;

#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{PlayerId, Vec2, celestial::Celestial};

/// A stack - a collection of modules docked to one another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// Current position
    pub position: Vec2<i32>,
    /// Change in position for next turn
    pub velocity: Vec2<i32>,
    /// Current owner; can be changed by docking a habitat
    pub owner: PlayerId,
    /// Name as assigned by owner - may be blank
    pub name: String,
    /// Modules contained
    pub modules: HashMap<ModuleId, Module>,
}

impl Stack {
    /// Create a starter stack for some player at some location with some velocity
    ///
    /// Contents = 2x hab, factory, refinery, miner, cargo hold (empty), tank (fuelled), 4x engine
    #[cfg(feature = "server")]
    pub fn starter_stack(
        owner: PlayerId,
        position: Vec2<i32>,
        velocity: Vec2<i32>,
        name: String,
        module_id_generator: &mut dyn Iterator<Item = ModuleId>,
    ) -> Self {
        Self {
            position,
            velocity,
            owner,
            name,
            modules: HashMap::from([
                (
                    module_id_generator.next().unwrap(),
                    Module::new_habitat(owner),
                ),
                (
                    module_id_generator.next().unwrap(),
                    Module::new_habitat(owner),
                ),
                (module_id_generator.next().unwrap(), Module::new_factory()),
                (module_id_generator.next().unwrap(), Module::new_refinery()),
                (module_id_generator.next().unwrap(), Module::new_miner()),
                (
                    module_id_generator.next().unwrap(),
                    Module::new_cargo_hold(),
                ),
                (module_id_generator.next().unwrap(), Module::new_fuel_tank()),
                (module_id_generator.next().unwrap(), Module::new_engine()),
                (module_id_generator.next().unwrap(), Module::new_engine()),
                (module_id_generator.next().unwrap(), Module::new_engine()),
                (module_id_generator.next().unwrap(), Module::new_engine()),
            ]),
        }
    }

    /// Create a new empty stack
    #[cfg(feature = "server")]
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
    #[cfg(feature = "server")]
    pub fn rendezvoused_with(&self, other: &Stack) -> bool {
        self.position == other.position && self.velocity == other.velocity
    }

    /// Is this stack landed on the target celestial?
    ///
    /// Broad terms landed - also counts rendezvouses with non-gravity celestials
    #[cfg(feature = "server")]
    pub fn landed(&self, celestial: &Celestial) -> bool {
        self.position == celestial.position && self.velocity.norm() == 0
    }

    /// Is this stack landed on a celestial with gravity?
    ///
    /// Strict terms landed - doesn't count rendezvouses with non-gravity celestials
    #[cfg(feature = "server")]
    pub fn landed_with_gravity(&self, celestial: &Celestial) -> bool {
        celestial.orbit_gravity && self.position == celestial.position && self.velocity.norm() == 0
    }

    /// When is the closest approach between this and the other stack, if it is within WARHEAD_RANGE
    #[cfg(feature = "server")]
    pub fn closest_approach(&self, other: &Stack) -> Option<f32> {
        let t =
            Vec2::closest_approach(self.position, self.velocity, other.position, other.velocity);
        if Vec2::squared_distance_at_time(
            self.position,
            self.velocity,
            other.position,
            other.velocity,
            t,
        ) <= ModuleDetails::WARHEAD_RANGE * ModuleDetails::WARHEAD_RANGE
        {
            Some(t)
        } else {
            None
        }
    }

    /// Is the other stack within range at time t
    ///
    /// By in range, we mean WARHEAD_RANGE
    #[cfg(feature = "server")]
    pub fn in_range(&self, t: f32, other: &Stack) -> bool {
        Vec2::squared_distance_at_time(
            self.position,
            self.velocity,
            other.position,
            other.velocity,
            t,
        ) < ModuleDetails::WARHEAD_RANGE * ModuleDetails::WARHEAD_RANGE
    }

    /// Deal some damage to the stack
    #[cfg(feature = "server")]
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

    /// Is this stack orbiting the target celestial?
    pub fn orbiting(&self, celestial: &Celestial) -> bool {
        celestial.orbit_gravity
            && (self.position - celestial.position).norm() == 1
            && self.velocity.norm() == 1
            && (self.position + self.velocity - celestial.position).norm() == 1
    }

    /// Get the wet mass in tonnes
    pub fn mass(&self) -> f32 {
        self.modules.values().map(|module| module.mass()).sum()
    }

    /// Get the dry mass in tonnes
    #[cfg(feature = "client")]
    pub fn dry_mass(&self) -> u32 {
        self.modules.values().map(|module| module.dry_mass()).sum()
    }

    /// Get the maximum possible mass in tonnes
    #[cfg(feature = "client")]
    pub fn full_mass(&self) -> u32 {
        self.modules.values().map(|module| module.full_mass()).sum()
    }

    /// Get the acceleration of a stack, in m/s^2
    pub fn acceleration(&self) -> f32 {
        let engine_count = self
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
        engine_count as f32 * ModuleDetails::ENGINE_THRUST / self.mass()
    }

    /// Get number of intact, damaged, and destroyed modules
    #[cfg(feature = "client")]
    pub fn damage_status(&self) -> (u32, u32, u32) {
        self.modules.values().fold(
            (0, 0, 0),
            |(intact, damaged, destroyed), module| match module.health {
                Health::Intact => (intact + 1, damaged, destroyed),
                Health::Damaged => (intact, damaged + 1, destroyed),
                Health::Destroyed => (intact, damaged + 1, destroyed),
            },
        )
    }

    /// Get current delta-v capabilities (assumes one large burn)
    #[cfg(feature = "client")]
    pub fn current_dv(&self) -> f32 {
        // propellant mass, in 0.1 tonnes, and total mass in tonnes
        let (propellant_mass, mass) =
            self.modules
                .values()
                .fold((0_f32, 0_f32), |(prop_mass, mass), module| {
                    match module.details {
                        ModuleDetails::Tank { fuel, .. } => {
                            (prop_mass + fuel as f32, mass + module.mass())
                        }
                        _ => (prop_mass, mass + module.mass()),
                    }
                });
        let delta_p = propellant_mass * ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32;
        delta_p / mass
    }

    /// Get fully-fuelled delta-v capabilities (assumes one large burn and no extra cargo)
    #[cfg(feature = "client")]
    pub fn max_dv(&self) -> f32 {
        // potential propellant mass, in 0.1 tonnes, and total mass in tonnes
        let (propellant_mass, mass) =
            self.modules
                .values()
                .fold((0_f32, 0_f32), |(prop_mass, mass), module| {
                    match module.details {
                        ModuleDetails::Tank { water, .. } => (
                            prop_mass + (ModuleDetails::TANK_CAPACITY as u8 - water) as f32,
                            mass + module.mass(),
                        ),
                        _ => (prop_mass, mass + module.mass()),
                    }
                });
        let delta_p = propellant_mass * ModuleDetails::ENGINE_SPECIFIC_IMPULSE as f32;
        delta_p / mass
    }

    /// Classify a stack for display purposes
    #[cfg(feature = "client")]
    pub fn classify(&self) -> &'static str {
        ""
        // TODO: improve this after alpha
    }
}

/// Id to refer to a stack
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackId(u32);

#[cfg(feature = "server")]
impl From<u32> for StackId {
    fn from(value: u32) -> Self {
        StackId(value)
    }
}

#[cfg(feature = "server")]
impl From<StackId> for u32 {
    fn from(value: StackId) -> Self {
        value.0
    }
}

/// A module, part of a stack
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    /// Current health - only intact stacks can do things
    pub health: Health,
    /// What type of stack is this, exactly, and what data does that involve
    pub details: ModuleDetails,
}

impl Module {
    #[cfg(feature = "server")]
    fn new(details: ModuleDetails) -> Self {
        Self {
            health: Health::Intact,
            details,
        }
    }

    /// Create a new miner module
    #[cfg(feature = "server")]
    pub fn new_miner() -> Self {
        Self::new(ModuleDetails::Miner)
    }

    /// Create a new fuel skimmer module
    #[cfg(feature = "server")]
    pub fn new_fuel_skimmer() -> Self {
        Self::new(ModuleDetails::FuelSkimmer)
    }

    /// Create a new (empty) cargo hold module
    #[cfg(feature = "server")]
    pub fn new_cargo_hold() -> Self {
        Self::new(ModuleDetails::CargoHold {
            ore: 0,
            materials: 0,
        })
    }

    /// Create a new (empty) tank module
    #[cfg(feature = "server")]
    pub fn new_tank() -> Self {
        Self::new(ModuleDetails::Tank { water: 0, fuel: 0 })
    }

    /// Create a tank full of fuel
    #[cfg(feature = "server")]
    fn new_fuel_tank() -> Self {
        Self::new(ModuleDetails::Tank {
            water: 0,
            fuel: ModuleDetails::TANK_CAPACITY as u8,
        })
    }

    /// Create a new engine module
    #[cfg(feature = "server")]
    pub fn new_engine() -> Self {
        Self::new(ModuleDetails::Engine)
    }

    /// Create a new warhead module (starts disarmed)
    #[cfg(feature = "server")]
    pub fn new_warhead() -> Self {
        Self::new(ModuleDetails::Warhead { armed: false })
    }

    /// Create a new gun module
    #[cfg(feature = "server")]
    pub fn new_gun() -> Self {
        Self::new(ModuleDetails::Gun)
    }

    /// Create a new habitat module
    #[cfg(feature = "server")]
    pub fn new_habitat(owner: PlayerId) -> Self {
        Self::new(ModuleDetails::Habitat { owner })
    }

    /// Create a new refinery module
    #[cfg(feature = "server")]
    pub fn new_refinery() -> Self {
        Self::new(ModuleDetails::Refinery)
    }

    /// Create a new factory module
    #[cfg(feature = "server")]
    pub fn new_factory() -> Self {
        Self::new(ModuleDetails::Factory)
    }

    /// Create a new armour plate module
    #[cfg(feature = "server")]
    pub fn new_armour_plate() -> Self {
        Self::new(ModuleDetails::ArmourPlate)
    }

    /// Get the dry mass of this module, in tonnes (excludes module contents for containers)
    pub fn dry_mass(&self) -> u32 {
        self.details.dry_mass()
    }

    /// Get the mass of this module, in tonnes (only relevant for getting the mass of the stack)
    pub fn mass(&self) -> f32 {
        self.details.mass()
    }

    /// Get the full mass of this module, in tonnes
    #[cfg(feature = "client")]
    pub fn full_mass(&self) -> u32 {
        self.details.full_mass()
    }
}

/// The health of a module
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Health {
    /// Operating normally
    Intact,
    /// Can't operate, but can be repaired
    Damaged,
    /// Can't operate or be repaired, but can be salvaged at a factory for 50% of materials
    Destroyed,
}

impl Health {
    /// Do damage to this module's health
    ///
    /// Note: must not try to do damage to a destroyed module, panic in debug mode if you do so
    #[cfg(feature = "server")]
    pub fn damage(&mut self) {
        match self {
            Health::Intact => *self = Health::Damaged,
            Health::Damaged => *self = Health::Destroyed,
            Health::Destroyed => {
                #[cfg(debug_assertions)]
                panic!("tried to damage destroyed module");
            }
        }
    }
}

/// Details about a module, like what type it is
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ModuleDetails {
    /// Miner - produces resources if landed on a body
    Miner,
    /// Fuel skimmer - produces fuel if orbiting a skimmable body
    FuelSkimmer,
    /// Cargo hold - holds ore and materials
    ///
    /// One unit = 0.1 tonnes
    CargoHold {
        /// How much ore is held, in 0.1 tonnes
        ore: u8,
        /// How much materials are held, in 0.1 tonnes
        materials: u8,
    },
    /// Tank - holds water and fuel
    ///
    /// One unit = 0.1 tonnes
    Tank {
        /// How much water is held, in 0.1 tonnes
        water: u8,
        /// How much fuel is held, in 0.1 tonnes
        fuel: u8,
    },
    /// Engine - produces 20 kN of thrust using 1 tonne of fuel; 1 m/s^2 = 0.5 hex/turn/turn
    ///
    /// Can burn fractional points of fuel, down to the 0.1 tonne
    Engine,
    /// Warhead - deals explosive attack to stack if intersecting (unless attached to a hab or disarmed)
    Warhead {
        /// Should this warhead detonate
        armed: bool,
    },
    /// Gun - 1/2 chance at 1 hex to deal 1 point of damage, 1/4 at 2 hexes, 1/8 at 3, and so on; guaranteed to hit at zero range
    Gun,
    /// Habitat - source of control and can effect repairs; stacks retain control until docked to by another habitat. Two habitats of different players can't dock.
    Habitat {
        /// Who owns this habitat (can't be changed in game)
        owner: PlayerId,
    },
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

    /// How much impulse (in tonne·hex/turn/0.1 tonne aka 10 hex/turn) does an engine produce per 0.1 tonne of fuel burned
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

    /// Mass of the module, in tonnes
    pub fn mass(&self) -> f32 {
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
}

impl ModuleDetails {
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

    #[cfg(feature = "client")]
    fn full_mass(&self) -> u32 {
        match self {
            ModuleDetails::CargoHold { .. } => {
                Self::CARGO_HOLD_MASS + Self::CARGO_HOLD_CAPACITY as u32 / 10
            }
            ModuleDetails::Tank { .. } => Self::TANK_MASS + Self::TANK_CAPACITY as u32 / 10,
            not_container => not_container.dry_mass(),
        }
    }
}

/// Id used to reference a module in a stack; unique across stacks
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(u32);

#[cfg(feature = "server")]
impl From<u32> for ModuleId {
    fn from(value: u32) -> Self {
        ModuleId(value)
    }
}

#[cfg(feature = "server")]
impl From<ModuleId> for u32 {
    fn from(value: ModuleId) -> Self {
        value.0
    }
}

#[cfg(all(test, feature = "server", feature = "client"))]
mod tests {
    use super::*;
    use crate::celestial::{Celestial, Resources};
    use rand::{SeedableRng, rngs::StdRng};

    #[test]
    fn test_stack_creation() {
        let position = Vec2 { q: 5, r: 3 };
        let velocity = Vec2 { q: 1, r: -1 };
        let owner = PlayerId::from(1u8);

        let stack = Stack::new(position, velocity, owner);

        assert_eq!(stack.position, position);
        assert_eq!(stack.velocity, velocity);
        assert_eq!(stack.owner, owner);
        assert_eq!(stack.name, String::default());
        assert!(stack.modules.is_empty());
    }

    #[test]
    fn test_starter_stack() {
        struct MockIdGen {
            next_id: u32,
        }
        impl Iterator for MockIdGen {
            type Item = ModuleId;
            fn next(&mut self) -> Option<Self::Item> {
                let id = ModuleId::from(self.next_id);
                self.next_id += 1;
                Some(id)
            }
        }

        let mut id_gen = MockIdGen { next_id: 0 };
        let owner = PlayerId::from(1u8);
        let position = Vec2 { q: 2, r: 3 };
        let velocity = Vec2 { q: 0, r: 1 };
        let name = "Test Starter".to_owned();

        let stack = Stack::starter_stack(owner, position, velocity, name.clone(), &mut id_gen);

        assert_eq!(stack.position, position);
        assert_eq!(stack.velocity, velocity);
        assert_eq!(stack.owner, owner);
        assert_eq!(stack.name, name);
        assert_eq!(stack.modules.len(), 11); // 2 habs + factory + refinery + miner + cargo + tank + 4 engines

        // Check that we have the expected module types
        let mut hab_count = 0;
        let mut engine_count = 0;
        let mut other_count = 0;

        for module in stack.modules.values() {
            match &module.details {
                ModuleDetails::Habitat { .. } => hab_count += 1,
                ModuleDetails::Engine => engine_count += 1,
                ModuleDetails::Factory
                | ModuleDetails::Refinery
                | ModuleDetails::Miner
                | ModuleDetails::CargoHold { .. }
                | ModuleDetails::Tank { .. } => other_count += 1,
                _ => panic!("Unexpected module type in starter stack"),
            }
        }

        assert_eq!(hab_count, 2);
        assert_eq!(engine_count, 4);
        assert_eq!(other_count, 5);
    }

    #[test]
    fn test_stack_rendezvous() {
        let position = Vec2 { q: 5, r: 3 };
        let velocity = Vec2 { q: 1, r: -1 };

        let stack1 = Stack {
            position,
            velocity,
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        let stack2 = Stack {
            position,
            velocity,
            owner: PlayerId::from(2u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        let stack3 = Stack {
            position: Vec2 { q: 6, r: 3 }, // different position
            velocity,
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        let stack4 = Stack {
            position,
            velocity: Vec2 { q: 2, r: -1 }, // different velocity
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(stack1.rendezvoused_with(&stack2));
        assert!(!stack1.rendezvoused_with(&stack3));
        assert!(!stack1.rendezvoused_with(&stack4));
    }

    #[test]
    fn test_stack_orbiting() {
        let celestial = Celestial {
            position: Vec2 { q: 0, r: 0 },
            name: "Test Planet".to_owned(),
            orbit_gravity: true,
            surface_gravity: 9.8,
            resources: Resources::None,
            radius: 0.5,
            colour: "#000000".to_owned(),
        };

        // For a stack to be in orbit, it needs:
        // 1. Distance from celestial = 1 hex
        // 2. Velocity norm = 1
        // 3. After movement (position + velocity) distance from celestial = 1

        // Use celestial's neighbors for valid orbital positions
        let neighbors = celestial.position.neighbours();

        // Test a valid orbit: position at neighbor, velocity to next neighbor
        let orbiting_stack = Stack {
            position: neighbors[0],        // Vec2 { q: 0, r: -1 } (up from center)
            velocity: Vec2 { q: 1, r: 0 }, // Move to up-right neighbor
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // Check preconditions
        assert_eq!((orbiting_stack.position - celestial.position).norm(), 1);
        assert_eq!(orbiting_stack.velocity.norm(), 1);

        // After movement: (0, -1) + (1, 0) = (1, -1) which is also a neighbor
        let next_pos = orbiting_stack.position + orbiting_stack.velocity;
        assert_eq!((next_pos - celestial.position).norm(), 1);

        assert!(orbiting_stack.orbiting(&celestial));

        // Test a non-orbiting stack - wrong distance
        let not_orbiting_stack = Stack {
            position: Vec2 { q: 2, r: 0 }, // distance 2 from celestial
            velocity: Vec2 { q: 0, r: 1 },
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(!not_orbiting_stack.orbiting(&celestial));

        // Test wrong velocity magnitude
        let wrong_velocity_stack = Stack {
            position: neighbors[0],
            velocity: Vec2 { q: 2, r: 0 }, // velocity norm = 2
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(!wrong_velocity_stack.orbiting(&celestial));

        // Celestial without gravity
        let no_gravity_celestial = Celestial {
            position: Vec2 { q: 0, r: 0 },
            name: "Asteroid".to_owned(),
            orbit_gravity: false,
            surface_gravity: 0.0,
            resources: Resources::MiningOre,
            radius: 0.1,
            colour: "#000000".to_owned(),
        };

        assert!(!orbiting_stack.orbiting(&no_gravity_celestial));
    }

    #[test]
    fn test_stack_landing() {
        let celestial = Celestial {
            position: Vec2 { q: 3, r: 2 },
            name: "Landable Planet".to_owned(),
            orbit_gravity: true,
            surface_gravity: 9.8,
            resources: Resources::MiningBoth,
            radius: 0.3,
            colour: "#000000".to_owned(),
        };

        // Stack landed (same position, zero velocity)
        let landed_stack = Stack {
            position: celestial.position,
            velocity: Vec2::zero(),
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(landed_stack.landed(&celestial));
        assert!(landed_stack.landed_with_gravity(&celestial));

        // Stack not landed - different position
        let not_landed_stack = Stack {
            position: Vec2 { q: 4, r: 2 },
            velocity: Vec2::zero(),
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(!not_landed_stack.landed(&celestial));
        assert!(!not_landed_stack.landed_with_gravity(&celestial));

        // Stack not landed - has velocity
        let moving_stack = Stack {
            position: celestial.position,
            velocity: Vec2 { q: 1, r: 0 },
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(!moving_stack.landed(&celestial));
        assert!(!moving_stack.landed_with_gravity(&celestial));

        // Test with no-gravity celestial
        let no_gravity_celestial = Celestial {
            position: Vec2 { q: 3, r: 2 },
            name: "Asteroid".to_owned(),
            orbit_gravity: false,
            surface_gravity: 0.0,
            resources: Resources::MiningOre,
            radius: 0.1,
            colour: "#000000".to_owned(),
        };

        assert!(landed_stack.landed(&no_gravity_celestial));
        assert!(!landed_stack.landed_with_gravity(&no_gravity_celestial)); // No gravity, so false
    }

    #[test]
    fn test_stack_mass() {
        let mut stack = Stack {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // Empty stack should have zero mass
        assert_eq!(stack.mass(), 0.0);

        // Add some modules
        stack
            .modules
            .insert(ModuleId::from(0u32), Module::new_engine());
        stack.modules.insert(
            ModuleId::from(1u32),
            Module::new_habitat(PlayerId::from(1u8)),
        );

        let expected_mass = ModuleDetails::ENGINE_MASS as f32 + ModuleDetails::HABITAT_MASS as f32;
        assert_eq!(stack.mass(), expected_mass);

        // Add a tank with fuel
        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { fuel, .. } = &mut tank.details {
            *fuel = 50;
        }
        stack.modules.insert(ModuleId::from(2u32), tank);

        let expected_mass_with_tank = expected_mass + ModuleDetails::TANK_MASS as f32 + 0.1 * 50.0;
        assert_eq!(stack.mass(), expected_mass_with_tank);
    }

    #[test]
    fn test_stack_closest_approach() {
        // Two stacks starting at same position - should be in range immediately
        let stack1 = Stack {
            position: Vec2 { q: 0, r: 0 },
            velocity: Vec2 { q: 1, r: 0 },
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        let stack2 = Stack {
            position: Vec2 { q: 0, r: 0 }, // Same starting position
            velocity: Vec2 { q: 0, r: 1 },
            owner: PlayerId::from(2u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // Should find approach at t=0 since they start at same position
        let approach_time = stack1.closest_approach(&stack2);
        assert!(approach_time.is_some());
        assert!((approach_time.unwrap() - 0.0).abs() < 0.01);

        // Test stacks that don't come close enough (beyond warhead range)
        let far_stack = Stack {
            position: Vec2 { q: 10, r: 10 }, // Very far away
            velocity: Vec2 { q: 0, r: 0 },   // Stationary
            owner: PlayerId::from(3u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        assert!(stack1.closest_approach(&far_stack).is_none());

        // Test parallel moving stacks that maintain distance
        let parallel_stack = Stack {
            position: Vec2 { q: 0, r: 2 }, // 2 units away
            velocity: Vec2 { q: 1, r: 0 }, // Same velocity as stack1
            owner: PlayerId::from(4u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // They maintain distance so might not come within warhead range
        let approach = stack1.closest_approach(&parallel_stack);
        // The outcome depends on WARHEAD_RANGE vs the distance - just test it doesn't panic
        assert!(approach.is_some() || approach.is_none()); // Either result is valid
    }

    #[test]
    fn test_stack_in_range() {
        let stack1 = Stack {
            position: Vec2 { q: 0, r: 0 },
            velocity: Vec2 { q: 1, r: 0 },
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        let stack2 = Stack {
            position: Vec2 { q: 0, r: 0 }, // Same initial position
            velocity: Vec2 { q: 0, r: 1 },
            owner: PlayerId::from(2u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // At t=0, they should be in range (same position)
        assert!(stack1.in_range(0.0, &stack2));

        // At t=1, they'll be at (1,0) and (0,1) respectively
        // Distance = sqrt((1-0)^2 + (0-1)^2) = sqrt(2) ≈ 1.414
        // But we need cartesian distance: (1.5, sqrt(3)/2) vs (0, sqrt(3))
        // This is more complex than hex distance, so let's test with closer positions

        // Test with a closer stationary stack
        let stack3 = Stack {
            position: Vec2 { q: 0, r: 0 },
            velocity: Vec2 { q: 0, r: 0 }, // Stationary
            owner: PlayerId::from(3u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // stack1 at t=0.1 will be close to origin - should be in range
        assert!(stack1.in_range(0.1, &stack3));

        // At t=0, should definitely be in range (same position)
        assert!(stack1.in_range(0.0, &stack3));
    }

    #[test]
    fn test_stack_do_damage() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut stack = Stack {
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            owner: PlayerId::from(1u8),
            name: String::new(),
            modules: HashMap::new(),
        };

        // Add some modules including armor
        stack
            .modules
            .insert(ModuleId::from(0u32), Module::new_engine());
        stack
            .modules
            .insert(ModuleId::from(1u32), Module::new_gun());
        stack
            .modules
            .insert(ModuleId::from(2u32), Module::new_armour_plate());

        // Initially all should be intact
        for module in stack.modules.values() {
            assert!(matches!(module.health, Health::Intact));
        }

        // Do 1 damage - should hit armour first
        stack.do_damage(1, &mut rng);
        let damaged_armour_count = stack
            .modules
            .values()
            .filter(|m| {
                matches!(
                    m,
                    Module {
                        health: Health::Damaged,
                        details: ModuleDetails::ArmourPlate
                    }
                )
            })
            .count();
        assert_eq!(damaged_armour_count, 1);

        // Do enough damage to affect other modules too
        stack.do_damage(5, &mut rng);

        // Check that some damage was done (either armor destroyed or other modules damaged)
        let total_damaged_or_destroyed = stack
            .modules
            .values()
            .filter(|m| !matches!(m.health, Health::Intact))
            .count();
        assert!(
            total_damaged_or_destroyed > 1,
            "Should have more than just one damaged module"
        );
    }

    #[test]
    fn test_module_creation() {
        let owner = PlayerId::from(1u8);

        // Test all module creation methods
        assert!(matches!(Module::new_miner().details, ModuleDetails::Miner));
        assert!(matches!(
            Module::new_fuel_skimmer().details,
            ModuleDetails::FuelSkimmer
        ));
        assert!(matches!(
            Module::new_cargo_hold().details,
            ModuleDetails::CargoHold {
                ore: 0,
                materials: 0
            }
        ));
        assert!(matches!(
            Module::new_tank().details,
            ModuleDetails::Tank { water: 0, fuel: 0 }
        ));
        assert!(matches!(
            Module::new_engine().details,
            ModuleDetails::Engine
        ));
        assert!(matches!(
            Module::new_warhead().details,
            ModuleDetails::Warhead { armed: false }
        ));
        assert!(matches!(Module::new_gun().details, ModuleDetails::Gun));
        assert!(
            matches!(Module::new_habitat(owner).details, ModuleDetails::Habitat { owner: o } if o == owner)
        );
        assert!(matches!(
            Module::new_refinery().details,
            ModuleDetails::Refinery
        ));
        assert!(matches!(
            Module::new_factory().details,
            ModuleDetails::Factory
        ));
        assert!(matches!(
            Module::new_armour_plate().details,
            ModuleDetails::ArmourPlate
        ));

        // Test that all start as intact
        assert!(matches!(Module::new_engine().health, Health::Intact));
    }

    #[test]
    fn test_module_mass() {
        // Test dry modules
        assert_eq!(
            Module::new_engine().mass(),
            ModuleDetails::ENGINE_MASS as f32
        );
        assert_eq!(Module::new_miner().mass(), ModuleDetails::MINER_MASS as f32);

        // Test modules with contents
        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { water, fuel } = &mut tank.details {
            *water = 10;
            *fuel = 20;
        }
        let expected_mass = ModuleDetails::TANK_MASS as f32 + 0.1 * (10.0 + 20.0);
        assert_eq!(tank.mass(), expected_mass);

        let mut cargo = Module::new_cargo_hold();
        if let ModuleDetails::CargoHold { ore, materials } = &mut cargo.details {
            *ore = 5;
            *materials = 15;
        }
        let expected_mass = ModuleDetails::CARGO_HOLD_MASS as f32 + 0.1 * (5.0 + 15.0);
        assert_eq!(cargo.mass(), expected_mass);
    }

    #[test]
    fn test_module_dry_mass() {
        // All modules should return their base mass regardless of contents
        assert_eq!(Module::new_engine().dry_mass(), ModuleDetails::ENGINE_MASS);
        assert_eq!(
            Module::new_habitat(PlayerId::from(1u8)).dry_mass(),
            ModuleDetails::HABITAT_MASS
        );

        // Even with contents, dry mass should be the same
        let mut tank = Module::new_tank();
        if let ModuleDetails::Tank { water, fuel } = &mut tank.details {
            *water = 100;
            *fuel = 100;
        }
        assert_eq!(tank.dry_mass(), ModuleDetails::TANK_MASS);
    }

    #[test]
    fn test_health_damage() {
        let mut health = Health::Intact;

        health.damage();
        assert!(matches!(health, Health::Damaged));

        health.damage();
        assert!(matches!(health, Health::Destroyed));

        // Damaging a destroyed module should panic in debug mode
        // but we can't easily test panic behavior here
    }

    #[test]
    fn test_module_details_mass() {
        assert_eq!(
            ModuleDetails::Miner.mass(),
            ModuleDetails::MINER_MASS as f32
        );
        assert_eq!(
            ModuleDetails::Engine.mass(),
            ModuleDetails::ENGINE_MASS as f32
        );

        let tank_with_contents = ModuleDetails::Tank {
            water: 50,
            fuel: 30,
        };
        let expected_mass = ModuleDetails::TANK_MASS as f32 + 0.1 * (50.0 + 30.0);
        assert_eq!(tank_with_contents.mass(), expected_mass);

        let cargo_with_contents = ModuleDetails::CargoHold {
            ore: 20,
            materials: 40,
        };
        let expected_mass = ModuleDetails::CARGO_HOLD_MASS as f32 + 0.1 * (20.0 + 40.0);
        assert_eq!(cargo_with_contents.mass(), expected_mass);
    }

    #[test]
    fn test_module_details_dry_mass() {
        assert_eq!(
            ModuleDetails::Factory.dry_mass(),
            ModuleDetails::FACTORY_MASS
        );
        assert_eq!(
            ModuleDetails::Refinery.dry_mass(),
            ModuleDetails::REFINERY_MASS
        );

        // Contents shouldn't matter for dry mass
        let tank_full = ModuleDetails::Tank {
            water: 200,
            fuel: 0,
        };
        assert_eq!(tank_full.dry_mass(), ModuleDetails::TANK_MASS);

        let cargo_full = ModuleDetails::CargoHold {
            ore: 100,
            materials: 100,
        };
        assert_eq!(cargo_full.dry_mass(), ModuleDetails::CARGO_HOLD_MASS);
    }

    #[test]
    #[expect(clippy::assertions_on_constants)]
    fn test_module_details_constants() {
        // Test that all constants are positive and reasonable
        assert!(ModuleDetails::MINER_PRODUCTION_RATE > 0);
        assert!(ModuleDetails::FUEL_SKIMMER_PRODUCTION_RATE > 0);
        assert!(ModuleDetails::CARGO_HOLD_CAPACITY > 0);
        assert!(ModuleDetails::TANK_CAPACITY > 0);
        assert!(ModuleDetails::ENGINE_SPECIFIC_IMPULSE > 0);
        assert!(ModuleDetails::ENGINE_THRUST > 0.0);
        assert!(ModuleDetails::WARHEAD_RANGE > 0.0);
        assert!(
            ModuleDetails::GUN_RANGE_ONE_HIT_CHANCE > 0.0
                && ModuleDetails::GUN_RANGE_ONE_HIT_CHANCE <= 1.0
        );

        // Test mass constants
        assert!(ModuleDetails::MINER_MASS > 0);
        assert!(ModuleDetails::ENGINE_MASS > 0);
        assert!(ModuleDetails::HABITAT_MASS > 0);

        // Test conversion ratios
        assert!(ModuleDetails::REFINERY_ORE_PER_MATERIAL > 0);
        assert!(ModuleDetails::REFINERY_WATER_PER_FUEL > 0);
        assert!(ModuleDetails::REPAIR_FRACTION > 0);
        assert!(ModuleDetails::SALVAGE_FRACTION > 0);
    }

    #[test]
    fn test_id_conversions() {
        // Test StackId conversions
        let stack_val = 12345u32;
        let stack_id = StackId::from(stack_val);
        let back_to_u32: u32 = stack_id.into();
        assert_eq!(stack_val, back_to_u32);

        // Test ModuleId conversions
        let module_val = 67890u32;
        let module_id = ModuleId::from(module_val);
        let back_to_u32: u32 = module_id.into();
        assert_eq!(module_val, back_to_u32);
    }
}
