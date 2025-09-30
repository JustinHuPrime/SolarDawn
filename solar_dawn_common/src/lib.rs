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

//! Game state and game mechanics for Solar Dawn
//!
//! Should be platform agnostic (wasm32 vs x86_64)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "server")]
use std::time::SystemTime;
use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use celestial::{Celestial, CelestialId};
#[cfg(feature = "server")]
use rand::Rng;
use serde::{Deserialize, Serialize};
use stack::{ModuleId, Stack, StackId};

use crate::order::{Order, OrderError};
#[cfg(feature = "server")]
use crate::stack::{Health, Module, ModuleDetails};

pub mod celestial;
pub mod order;
pub mod stack;

/// The game state
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    /// The phase the game is in
    pub phase: Phase,
    /// Which turn is this - starts at one
    pub turn: u32,
    /// Map from player id to username
    pub players: HashMap<PlayerId, String>,
    /// Game board
    pub celestials: HashMap<CelestialId, Celestial>,
    /// Which celestial is Earth (starting planet and allows habitat builds in orbit)
    pub earth: CelestialId,
    /// Game pieces
    pub stacks: HashMap<StackId, Stack>,
    /// Unique game id
    ///
    /// Is the unix timestamp of when the game was created in milliseconds, modulo 32
    ///
    /// Used on the client side for indexing client-side-only settings storage
    pub game_id: u32,
}

/// Constructor function for some starting game state
pub type GameStateInitializer = fn(
    players: HashMap<PlayerId, String>,
    celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
    stack_id_generator: &mut dyn Iterator<Item = StackId>,
    module_id_generator: &mut dyn Iterator<Item = ModuleId>,
) -> GameState;

/// A game state delta
#[derive(Debug, Serialize, Deserialize)]
pub struct GameStateDelta {
    /// The next phase it should be in
    pub phase: Phase,
    /// The next turn count it should be in
    pub turn: u32,
    /// The new stacks
    pub stacks: HashMap<StackId, Stack>,
    /// All previous orders
    pub orders: HashMap<PlayerId, Vec<Order>>,
    /// The results of the previous orders
    pub errors: HashMap<PlayerId, Vec<Option<OrderError>>>,
}

#[cfg(feature = "server")]
impl GameState {
    /// Constructs an initializer for a new game given a scenario identifier
    ///
    /// Returns Err with the identifier if it does not correspond to a known scenario
    pub fn new(scenario: &str) -> Result<GameStateInitializer, &str> {
        match scenario {
            "campaign" => Ok(
                |players: HashMap<PlayerId, String>,
                 celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
                 stack_id_generator: &mut dyn Iterator<Item = StackId>,
                 module_id_generator: &mut dyn Iterator<Item = ModuleId>| {
                    {
                        let (celestials, earth) =
                            Celestial::solar_system_balanced_positions(celestial_id_generator);
                        let earth_ref = celestials.get(&earth).expect("earth id should be valid");
                        let mut stacks = HashMap::new();
                        for ((owner, _), (position, velocity)) in
                            players.iter().zip(earth_ref.orbit_parameters(true))
                        {
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
                            turn: 1,
                            players,
                            celestials,
                            earth,
                            stacks,
                            game_id: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .expect("date is well after epoch")
                                .as_millis() as u32,
                        }
                    }
                },
            ),
            #[cfg(test)]
            "test" => Ok(
                |players: HashMap<PlayerId, String>,
                 celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
                 _stack_id_generator: &mut dyn Iterator<Item = StackId>,
                 _module_id_generator: &mut dyn Iterator<Item = ModuleId>| {
                    {
                        let (celestials, earth) =
                            Celestial::solar_system_for_testing(celestial_id_generator);
                        let _ = celestials.get(&earth).expect("earth id should be valid");
                        Self {
                            phase: Phase::Logistics,
                            turn: 1,
                            players,
                            celestials,
                            earth,
                            stacks: HashMap::new(),
                            game_id: 0,
                        }
                    }
                },
            ),
            _ => Err(scenario),
        }
    }

    /// Produce the next game state after resolving orders
    pub fn next(
        &self,
        orders: HashMap<PlayerId, Vec<Order>>,
        stack_id_generator: &mut impl Iterator<Item = StackId>,
        module_id_generator: &mut impl Iterator<Item = ModuleId>,
        rng: &mut impl Rng,
    ) -> GameStateDelta {
        // validate orders
        let (validated, errors) = Order::validate(self, &orders);

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
                let warhead_count = missile_ref
                    .modules
                    .values()
                    .filter(|module| {
                        matches!(
                            module,
                            Module {
                                health: Health::Intact,
                                details: ModuleDetails::Warhead { armed: true }
                            }
                        )
                    })
                    .count() as u32;
                if warhead_count > 0 {
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
                            *damaged.entry(thing).or_default() += warhead_count;
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
                let damaged_count =
                    module_count.div_ceil(ModuleDetails::WARHEAD_DAMAGE_FRACTION) * hits;
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

        GameStateDelta {
            phase: self.phase.next(),
            turn: if matches!(self.phase, Phase::Movement) {
                self.turn + 1
            } else {
                self.turn
            },
            stacks,
            orders,
            errors,
        }
    }
}

impl GameState {
    /// Apply a delta
    pub fn apply(&mut self, delta: GameStateDelta) {
        self.phase = delta.phase;
        self.turn = delta.turn;
        self.stacks = delta.stacks;
    }
}

/// The phase of the turn; determines which orders are allowed
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    /// Logistics orders only
    Logistics,
    /// Shoot and arm orders only
    Combat,
    /// Burn orders only
    Movement,
}

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
impl From<u8> for PlayerId {
    fn from(value: u8) -> Self {
        PlayerId(value)
    }
}

#[cfg(feature = "server")]
impl From<PlayerId> for u8 {
    fn from(value: PlayerId) -> Self {
        value.0
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
    /// Create the zero vector
    pub fn zero() -> Self {
        Self { q: 0, r: 0 }
    }

    /// Create the unit vector going up
    pub fn unit_up() -> Self {
        Self { q: 0, r: -1 }
    }
    /// Create the unit vector going up and right
    pub fn unit_up_right() -> Self {
        Self { q: 1, r: -1 }
    }
    /// Create the unit vector going down and right
    pub fn unit_down_right() -> Self {
        Self { q: 1, r: 0 }
    }
    /// Create the unit vector going down
    pub fn unit_down() -> Self {
        Self { q: 0, r: 1 }
    }
    /// Create the unit vector going down and left
    pub fn unit_down_left() -> Self {
        Self { q: -1, r: 1 }
    }
    /// Create the unit vector going up and left
    pub fn unit_up_left() -> Self {
        Self { q: -1, r: 0 }
    }

    /// What's this vector but one step up
    pub fn up(&self) -> Self {
        *self + Self::unit_up()
    }
    /// What's this vector but one step up and right
    pub fn up_right(&self) -> Self {
        *self + Self::unit_up_right()
    }
    /// What's this vector but one step down and right
    pub fn down_right(&self) -> Self {
        *self + Self::unit_down_right()
    }
    /// What's this vector but one step down
    pub fn down(&self) -> Self {
        *self + Self::unit_down()
    }
    /// What's this vector but one step down and left
    pub fn down_left(&self) -> Self {
        *self + Self::unit_down_left()
    }
    /// What's this vector but one step up and left
    pub fn up_left(&self) -> Self {
        *self + Self::unit_up_left()
    }

    /// Get the neighbours of this position
    pub fn neighbours(&self) -> [Self; 6] {
        [
            self.up(),
            self.up_right(),
            self.down_right(),
            self.down(),
            self.down_left(),
            self.up_left(),
        ]
    }

    /// Get the length of this vector, in cells
    ///
    /// Note: not the cartesian length
    pub fn norm(&self) -> i32 {
        (self.q.abs() + (self.q + self.r).abs() + self.r.abs()) / 2
    }

    /// Convert this vector to cartesian
    pub fn cartesian(&self) -> (f32, f32) {
        let x: f32 = 1.5 * self.q as f32;
        let y: f32 = 3.0_f32.sqrt() / 2.0 * self.q as f32 + 3.0_f32.sqrt() * self.r as f32;
        (x, y)
    }

    /// Construct a vector from fractional q and r
    pub fn round(q: f32, r: f32) -> Self {
        let s = -q - r;

        let mut q_int = q.round_ties_even() as i32;
        let mut r_int = r.round_ties_even() as i32;
        let s_int = s.round_ties_even() as i32;

        let dq = (q - q_int as f32).abs();
        let dr = (r - r_int as f32).abs();
        let ds = (s - s_int as f32).abs();

        if dq > dr && dq > ds {
            q_int = -r_int - s_int;
        } else if dr > ds {
            r_int = -q_int - s_int;
        }
        // don't care about s coordinate
        Self { q: q_int, r: r_int }
    }

    /// Construct a vector from cartesian coordinates
    pub fn from_cartesian(x: f32, y: f32) -> Self {
        let q = x * 2.0 / 3.0;
        #[expect(clippy::neg_multiply)]
        let r = x * -1.0 / 3.0 + y * 3.0_f32.sqrt() / 3.0;
        Self::round(q, r)
    }

    /// Construct a vector from polar coordinates
    pub fn from_polar(r: f32, theta: f32) -> Self {
        let x = r * theta.cos();
        let y = r * -theta.sin();
        Self::from_cartesian(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_arithmetic() {
        let v1 = Vec2 { q: 3, r: 4 };
        let v2 = Vec2 { q: 1, r: 2 };

        // Test addition
        let sum = v1 + v2;
        assert_eq!(sum.q, 4);
        assert_eq!(sum.r, 6);

        // Test subtraction
        let diff = v1 - v2;
        assert_eq!(diff.q, 2);
        assert_eq!(diff.r, 2);

        // Test add_assign
        let mut v3 = v1;
        v3 += v2;
        assert_eq!(v3.q, 4);
        assert_eq!(v3.r, 6);

        // Test sub_assign
        let mut v4 = v1;
        v4 -= v2;
        assert_eq!(v4.q, 2);
        assert_eq!(v4.r, 2);
    }

    #[test]
    fn test_vec2_unit_vectors() {
        assert_eq!(Vec2::zero(), Vec2 { q: 0, r: 0 });
        assert_eq!(Vec2::unit_up(), Vec2 { q: 0, r: -1 });
        assert_eq!(Vec2::unit_up_right(), Vec2 { q: 1, r: -1 });
        assert_eq!(Vec2::unit_down_right(), Vec2 { q: 1, r: 0 });
        assert_eq!(Vec2::unit_down(), Vec2 { q: 0, r: 1 });
        assert_eq!(Vec2::unit_down_left(), Vec2 { q: -1, r: 1 });
        assert_eq!(Vec2::unit_up_left(), Vec2 { q: -1, r: 0 });
    }

    #[test]
    fn test_vec2_movement_methods() {
        let origin = Vec2::zero();

        assert_eq!(origin.up(), Vec2 { q: 0, r: -1 });
        assert_eq!(origin.up_right(), Vec2 { q: 1, r: -1 });
        assert_eq!(origin.down_right(), Vec2 { q: 1, r: 0 });
        assert_eq!(origin.down(), Vec2 { q: 0, r: 1 });
        assert_eq!(origin.down_left(), Vec2 { q: -1, r: 1 });
        assert_eq!(origin.up_left(), Vec2 { q: -1, r: 0 });

        // Test from non-origin position
        let pos = Vec2 { q: 2, r: 3 };
        assert_eq!(pos.up(), Vec2 { q: 2, r: 2 });
        assert_eq!(pos.down_right(), Vec2 { q: 3, r: 3 });
    }

    #[test]
    fn test_vec2_neighbours() {
        let origin = Vec2::zero();
        let neighbours = origin.neighbours();

        assert_eq!(neighbours.len(), 6);
        assert_eq!(neighbours[0], Vec2 { q: 0, r: -1 }); // up
        assert_eq!(neighbours[1], Vec2 { q: 1, r: -1 }); // up_right
        assert_eq!(neighbours[2], Vec2 { q: 1, r: 0 }); // down_right
        assert_eq!(neighbours[3], Vec2 { q: 0, r: 1 }); // down
        assert_eq!(neighbours[4], Vec2 { q: -1, r: 1 }); // down_left
        assert_eq!(neighbours[5], Vec2 { q: -1, r: 0 }); // up_left
    }

    #[test]
    fn test_vec2_norm() {
        assert_eq!(Vec2::zero().norm(), 0);
        assert_eq!(Vec2 { q: 1, r: 0 }.norm(), 1);
        assert_eq!(Vec2 { q: 0, r: 1 }.norm(), 1);
        assert_eq!(Vec2 { q: 1, r: 1 }.norm(), 2);
        assert_eq!(Vec2 { q: 2, r: -1 }.norm(), 2);
        assert_eq!(Vec2 { q: -2, r: 1 }.norm(), 2);
        assert_eq!(Vec2 { q: 3, r: 0 }.norm(), 3);
        assert_eq!(Vec2 { q: -3, r: 3 }.norm(), 3);
    }

    #[test]
    fn test_vec2_cartesian() {
        let origin = Vec2::zero();
        let (x, y) = origin.cartesian();
        assert!((x - 0.0).abs() < f32::EPSILON);
        assert!((y - 0.0).abs() < f32::EPSILON);

        let v = Vec2 { q: 1, r: 0 };
        let (x, y) = v.cartesian();
        assert!((x - 1.5).abs() < f32::EPSILON);
        assert!((y - 3.0_f32.sqrt() / 2.0).abs() < f32::EPSILON);

        let v = Vec2 { q: 0, r: 1 };
        let (x, y) = v.cartesian();
        assert!((x - 0.0).abs() < f32::EPSILON);
        assert!((y - 3.0_f32.sqrt()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec2_from_cartesian() {
        // Test conversion from cartesian back to hex coordinates
        let origin_cart = (0.0, 0.0);
        assert_eq!(
            Vec2::from_cartesian(origin_cart.0, origin_cart.1),
            Vec2::zero()
        );

        let v1 = Vec2 { q: 1, r: 0 };
        let (x, y) = v1.cartesian();
        assert_eq!(Vec2::from_cartesian(x, y), v1);

        let v2 = Vec2 { q: 0, r: 1 };
        let (x, y) = v2.cartesian();
        assert_eq!(Vec2::from_cartesian(x, y), v2);

        let v3 = Vec2 { q: 2, r: -1 };
        let (x, y) = v3.cartesian();
        assert_eq!(Vec2::from_cartesian(x, y), v3);
    }

    #[test]
    fn test_vec2_from_polar() {
        // Test simple polar coordinate conversion
        assert_eq!(Vec2::from_polar(0.0, 0.0), Vec2::zero());

        // Let's test by converting back and forth to verify consistency
        let test_points = vec![
            Vec2 { q: 1, r: 0 },
            Vec2 { q: 0, r: 1 },
            Vec2 { q: 2, r: -1 },
        ];

        for point in test_points {
            let (x, y) = point.cartesian();
            let r = (x * x + y * y).sqrt();
            let theta = (-y).atan2(x); // Note: y is negated in from_polar
            let converted_back = Vec2::from_polar(r, theta);
            assert_eq!(
                converted_back, point,
                "Round-trip conversion failed for {:?}",
                point
            );
        }
    }

    #[test]
    fn test_vec2_round() {
        // Test basic rounding of fractional coordinates
        assert_eq!(Vec2::round(0.0, 0.0), Vec2::zero());
        assert_eq!(Vec2::round(1.0, 0.0), Vec2 { q: 1, r: 0 });
        assert_eq!(Vec2::round(0.0, 1.0), Vec2 { q: 0, r: 1 });

        // Test that the rounding function works correctly for round-trip conversions
        let test_points = vec![
            Vec2 { q: 1, r: 0 },
            Vec2 { q: 0, r: 1 },
            Vec2 { q: 2, r: -1 },
            Vec2 { q: -1, r: 2 },
        ];

        for point in test_points {
            let (x, y) = point.cartesian();
            let q = x * (2.0 / 3.0);
            let r = x * (-1.0 / 3.0) + y * (3.0_f32.sqrt() / 3.0);
            let rounded = Vec2::round(q, r);
            assert_eq!(
                rounded, point,
                "Round failed for {:?} -> ({}, {})",
                point, q, r
            );
        }
    }

    #[test]
    fn test_phase_transitions() {
        assert_eq!(Phase::Logistics.next(), Phase::Combat);
        assert_eq!(Phase::Combat.next(), Phase::Movement);
        assert_eq!(Phase::Movement.next(), Phase::Logistics);

        // Test complete cycle
        let mut phase = Phase::Logistics;
        phase = phase.next();
        assert_eq!(phase, Phase::Combat);
        phase = phase.next();
        assert_eq!(phase, Phase::Movement);
        phase = phase.next();
        assert_eq!(phase, Phase::Logistics);
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_player_id_conversions() {
        let id_val = 5u8;
        let player_id = PlayerId::from(id_val);
        let back_to_u8: u8 = player_id.into();
        assert_eq!(id_val, back_to_u8);
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_player_id_starting_stack_names() {
        assert_eq!(
            PlayerId::from(1u8).starting_stack_name(),
            "Washington Station"
        );
        assert_eq!(PlayerId::from(2u8).starting_stack_name(), "Moscow Orbital");
        assert_eq!(
            PlayerId::from(3u8).starting_stack_name(),
            "Beijing Highport"
        );
        assert_eq!(PlayerId::from(4u8).starting_stack_name(), "Paris Terminal");
        assert_eq!(
            PlayerId::from(5u8).starting_stack_name(),
            "London Spacedock"
        );
        assert_eq!(
            PlayerId::from(6u8).starting_stack_name(),
            "New Delhi Platform"
        );
    }

    #[cfg(feature = "server")]
    #[test]
    #[should_panic(expected = "Invalid player id")]
    fn test_player_id_invalid_starting_stack_name() {
        PlayerId::from(7u8).starting_stack_name();
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_game_state_new_scenarios() {
        // Test valid scenario
        let campaign_init = GameState::new("campaign");
        assert!(campaign_init.is_ok());

        // Test test scenario (only available in cfg(test))
        let test_init = GameState::new("test");
        assert!(test_init.is_ok());

        // Test invalid scenario
        let invalid_result = GameState::new("invalid_scenario");
        assert!(invalid_result.is_err());
        assert_eq!(invalid_result.err().unwrap(), "invalid_scenario");
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_game_state_apply() {
        use std::collections::HashMap;

        // Create a simple game state
        let mut game_state = GameState {
            phase: Phase::Logistics,
            turn: 1,
            players: HashMap::new(),
            celestials: HashMap::new(),
            earth: CelestialId::from(0u8),
            stacks: HashMap::new(),
            game_id: 0,
        };

        // Create a delta to apply
        let mut new_stacks = HashMap::new();
        new_stacks.insert(
            StackId::from(1u32),
            Stack {
                position: Vec2 { q: 1, r: 1 },
                velocity: Vec2::zero(),
                owner: PlayerId::from(1u8),
                name: "Test Stack".to_string(),
                modules: HashMap::new(),
            },
        );

        let delta = GameStateDelta {
            phase: Phase::Combat,
            turn: 1,
            stacks: new_stacks.clone(),
            orders: HashMap::new(),
            errors: HashMap::new(),
        };

        // Apply the delta
        game_state.apply(delta);

        // Verify the changes
        assert_eq!(game_state.phase, Phase::Combat);
        assert_eq!(game_state.stacks.len(), 1);
        assert!(game_state.stacks.contains_key(&StackId::from(1u32)));
    }
}
