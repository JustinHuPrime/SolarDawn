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

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Game state and game mechanics for Solar Dawn
//!
//! Should be platform agnostic (wasm32 vs x86_64)

use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Sub, SubAssign},
    rc::Rc,
};

use celestial::{Celestial, CelestialId};
#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};
use stack::{ModuleId, Stack, StackId};

#[cfg(feature = "server")]
use crate::{
    order::{Order, OrderError},
    stack::{Health, Module, ModuleDetails},
};

mod celestial;
pub mod order;
mod stack;

/// The game state
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    /// The phase the game is in
    phase: Phase,
    /// Map from player id to username
    players: Rc<HashMap<PlayerId, String>>,
    /// Game board
    celestials: Rc<HashMap<CelestialId, Celestial>>,
    /// Which celestial is Earth (starting planet and allows habitat builds in orbit)
    earth: CelestialId,
    /// Game pieces
    stacks: HashMap<StackId, Stack>,
}

#[cfg(feature = "server")]
impl GameState {
    /// Construct a new game state
    ///
    /// Player map is expected to be determined by netcode or config
    ///
    /// Celestials are expected to be supplied from a constant
    ///
    /// Generates starter stacks from [`Stack::starter_stack`]
    pub fn new(
        players: HashMap<PlayerId, String>,
        celestials: HashMap<CelestialId, Celestial>,
        earth_id: CelestialId,
        stack_id_generator: &mut impl Iterator<Item = StackId>,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
    ) -> Self {
        use std::rc::Rc;

        let earth = celestials.get(&earth_id).expect("earth id should be valid");
        let mut stacks = HashMap::new();
        for ((owner, _), (position, velocity)) in players.iter().zip(earth.orbit_parameters(true)) {
            stacks.insert(
                stack_id_generator.next().expect("should be infinite"),
                Stack::starter_stack(
                    *owner,
                    position,
                    velocity,
                    owner.starting_stack_name(),
                    module_id_generator,
                ),
            );
        }
        Self {
            phase: Phase::Logistics,
            players: Rc::new(players),
            celestials: Rc::new(celestials),
            earth: earth_id,
            stacks,
        }
    }

    /// Produce the next game state after resolving orders
    pub fn next(
        &self,
        orders: &HashMap<PlayerId, Vec<Order>>,
        stack_id_generator: &mut impl Iterator<Item = StackId>,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
        rng: &mut impl Rng,
    ) -> (Self, HashMap<PlayerId, Vec<Option<OrderError>>>) {
        // validate orders
        let (validated, errors) = Order::validate(self, orders);

        // apply orders
        let mut stacks = validated.apply(stack_id_generator, module_id_generator, rng);

        if matches!(self.phase, Phase::Movement) {
            // crash stacks
            stacks.retain(|_, stack| {
                !self.celestials.values().any(|celestial| {
                    celestial.collides(stack.position, stack.position + stack.velocity)
                })
            });

            // detonate warheads
            let mut detonated = Vec::new();
            let mut damaged: HashMap<StackId, u32> = HashMap::new();
            for (&missile, missile_ref) in stacks.iter() {
                // for each stack with an armed warhead
                if missile_ref.modules.values().any(|module| {
                    matches!(
                        module,
                        Module {
                            health: Health::Intact,
                            details: ModuleDetails::Warhead { armed: true }
                        }
                    )
                }) {
                    // find the first point at which a non-owned stack comes into range, if any
                    if let Some(intercept) = stacks
                        .values()
                        .filter(|stack| stack.owner != missile_ref.owner)
                        .filter_map(|stack| missile_ref.closest_approach(stack))
                        .min_by(|a, b| a.total_cmp(b))
                    {
                        // mark all stacks in range at that point in time as damaged
                        for thing in stacks.iter().filter_map(|(stack, stack_ref)| {
                            if missile_ref.in_range(intercept, stack_ref) {
                                Some(*stack)
                            } else {
                                None
                            }
                        }) {
                            *damaged.entry(thing).or_default() += 1;
                        }
                        // mark the warhead as detonated
                        detonated.push(missile);
                    }
                }
            }
            // actually do damage
            for (stack, hits) in damaged {
                let stack_ref = stacks.get_mut(&stack).expect("saved key");

                let module_count = stack_ref.modules.len() as u32;
                let damaged_count = module_count.div_ceil(ModuleDetails::WARHEAD_DAMAGE_FRACTION) * hits;
                stack_ref.do_damage(damaged_count, rng);
            }
            // remove detonated stacks
            for detonated in detonated {
                stacks.remove(&detonated);
            }

            // apply movement
            for (_, stack) in stacks.iter_mut() {
                stack.position += stack.velocity;
            }
        }

        // apply state-based action - delete empty stacks
        stacks.retain(|_, stack| !stack.modules.is_empty());

        // apply state-based actions
        // set stack ownership based on habitats
        // disarm warheads on stack with habitats
        // disarm damaged and destroyed warheads
        for (_, stack) in stacks.iter_mut() {
            if let Some(owner) = stack.modules.values().find_map(|module| match module {
                Module {
                    health: Health::Intact,
                    details: ModuleDetails::Habitat { owner },
                } => Some(*owner),
                _ => None,
            }) {
                stack.owner = owner;
                for (_, module) in stack.modules.iter_mut() {
                    if let Module {
                        details: ModuleDetails::Warhead { armed },
                        ..
                    } = module
                    {
                        *armed = false;
                    }
                }
            } else {
                for (_, module) in stack.modules.iter_mut() {
                    if let Module {
                        details: ModuleDetails::Warhead { armed },
                        health: Health::Damaged | Health::Destroyed,
                    } = module
                    {
                        *armed = false;
                    }
                }
            }
        }

        (
            Self {
                phase: self.phase.next(),
                players: self.players.clone(),
                celestials: self.celestials.clone(),
                earth: self.earth,
                stacks,
            },
            errors,
        )
    }
}

/// The phase of the turn; determines which orders are allowed
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Phase {
    /// Logistics orders only
    Logistics,
    /// Shoot and arm orders only
    Combat,
    /// Burn orders only
    Movement,
}

impl Phase {
    fn next(self) -> Self {
        use Phase::*;

        match self {
            Logistics => Combat,
            Combat => Movement,
            Movement => Logistics,
        }
    }
}

/// Refers to a player
///
/// 0 is special - refers to unowned stacks (shouldn't be possible)
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(u8);

#[cfg(feature = "server")]
impl PlayerId {
    fn starting_stack_name(&self) -> String {
        match self.0 {
            1 => "Washington Station".to_string(),
            2 => "Moscow Orbital".to_string(),
            3 => "Beijing Highport".to_string(),
            4 => "Paris Terminal".to_string(),
            5 => "London Spacedock".to_string(),
            6 => "New Delhi Platform".to_string(),
            _ => panic!("Invalid player id"),
        }
    }
}

/// A hex-grid axial vector
///
/// Flat-topped hexes
///
/// +q = down-right, +r = down
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Vec2<T: Copy> {
    q: T,
    r: T,
}

impl Add for Vec2<i32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl AddAssign for Vec2<i32> {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.r += rhs.r;
    }
}
impl Sub for Vec2<i32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl SubAssign for Vec2<i32> {
    fn sub_assign(&mut self, rhs: Self) {
        self.q -= rhs.q;
        self.r -= rhs.r;
    }
}
impl Vec2<i32> {
    fn zero() -> Self {
        Self { q: 0, r: 0 }
    }

    fn unit_up() -> Self {
        Self { q: 0, r: -1 }
    }
    fn unit_up_right() -> Self {
        Self { q: 1, r: -1 }
    }
    fn unit_down_right() -> Self {
        Self { q: 1, r: 0 }
    }
    fn unit_down() -> Self {
        Self { q: 0, r: 1 }
    }
    fn unit_down_left() -> Self {
        Self { q: -1, r: 1 }
    }
    fn unit_up_left() -> Self {
        Self { q: -1, r: 0 }
    }

    fn up(&self) -> Self {
        *self + Self::unit_up()
    }
    fn up_right(&self) -> Self {
        *self + Self::unit_up_right()
    }
    fn down_right(&self) -> Self {
        *self + Self::unit_down_right()
    }
    fn down(&self) -> Self {
        *self + Self::unit_down()
    }
    fn down_left(&self) -> Self {
        *self + Self::unit_down_left()
    }
    fn up_left(&self) -> Self {
        *self + Self::unit_up_left()
    }

    fn neighbours(&self) -> [Self; 6] {
        [
            self.up(),
            self.up_right(),
            self.down_right(),
            self.down(),
            self.down_left(),
            self.up_left(),
        ]
    }

    fn norm(&self) -> i32 {
        (self.q.abs() + (self.q + self.r).abs() + self.r.abs()) / 2
    }

    fn cartesian(&self) -> (f32, f32) {
        let x: f32 = 1.5 * self.q as f32;
        let y: f32 = 3.0_f32.sqrt() / 2.0 * self.q as f32 + 3.0_f32.sqrt() * self.r as f32;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
