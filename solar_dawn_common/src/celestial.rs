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

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Vec2;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Celestial {
    position: Vec2<i32>,
    name: String,
    gravity: Gravity,
    /// What resources can be obtained?
    ///
    /// - Mining(Both|Ice|Ore) = must land, if a body with gravity, with a
    ///   landing manoeuver, or rendezvoused for bodies without gravity
    /// - Skimming = if in orbit, may use a skimming manoeuver
    resources: Resources,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CelestialId(u8);

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum Gravity {
    High,
    Normal,
    None,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum Resources {
    MiningBoth,
    MiningIce,
    MiningOre,
    Skimming,
    None,
}

impl Celestial {
    pub fn can_land(&self) -> bool {
        !matches!(self.gravity, Gravity::High) && !matches!(self.resources, Resources::Skimming)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
