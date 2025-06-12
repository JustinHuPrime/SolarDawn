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
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
    rc::Rc,
};

use celestial::{Celestial, CelestialId};
use num::Num;
use serde::{Deserialize, Serialize};
use stack::{ModuleId, Stack, StackId};

mod celestial;
pub mod order;
mod stack;

/// The game state
#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
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
}

/// The phase of the turn; determines which orders are allowed
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
/// +q = down-right, +r = down
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Vec2<T: Num + Copy> {
    q: T,
    r: T,
}

impl<T: Num + Copy> Vec2<T> {
    fn zero() -> Self {
        Self {
            q: T::zero(),
            r: T::zero(),
        }
    }
}
impl<T: Num + Copy> Add for Vec2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl<T: Num + Copy + AddAssign> AddAssign for Vec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.r += rhs.r;
    }
}
impl<T: Num + Copy> Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl<T: Num + Copy + SubAssign> SubAssign for Vec2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.q -= rhs.q;
        self.r -= rhs.r;
    }
}
impl<T: Num + Copy + Neg<Output = T>> Vec2<T> {
    fn unit_up() -> Self {
        Self {
            q: T::zero(),
            r: -T::one(),
        }
    }
    fn unit_up_right() -> Self {
        Self {
            q: T::one(),
            r: -T::one(),
        }
    }
    fn unit_down_right() -> Self {
        Self {
            q: T::one(),
            r: T::zero(),
        }
    }
    fn unit_down() -> Self {
        Self {
            q: T::zero(),
            r: T::one(),
        }
    }
    fn unit_down_left() -> Self {
        Self {
            q: -T::one(),
            r: T::one(),
        }
    }
    fn unit_up_left() -> Self {
        Self {
            q: -T::one(),
            r: T::zero(),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
}
