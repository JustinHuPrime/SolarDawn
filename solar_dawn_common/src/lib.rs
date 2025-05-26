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

//! Game state and game mechanics for Solar Dawn
//!
//! Should be platform agnostic (wasm32 vs x86_64)

use std::collections::HashMap;

use celestial::{Celestial, CelestialId};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use stack::{Stack, StackId};

mod celestial;
pub mod order;
mod stack;

/// The game state
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameState {
    phase: Phase,
    players: HashMap<PlayerId, String>,
    celestials: HashMap<CelestialId, Celestial>,
    stacks: HashMap<StackId, Stack>,
}

/// The phase of the turn; determines which orders are allowed
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Phase {
    /// Logistics orders only
    Logistics,
    /// Shoot and arm orders only
    Combat,
    /// Burn orders only
    Movement,
}

impl GameState {}

/// Refers to a player
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PlayerId(u8);

/// A hex-grid axial vector
///
/// +q = down-right, +r = down
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Vec2<T> {
    q: T,
    r: T,
}

#[cfg(test)]
mod tests {
    use super::*;
}
