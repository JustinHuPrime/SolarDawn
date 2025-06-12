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

#[cfg(feature = "server")]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Vec2;

#[derive(Debug, Serialize, Deserialize)]
pub struct Celestial {
    pub position: Vec2<i32>,
    name: String,
    /// Are there gravity effects in the one-hex ring around this body (for orbits)
    orbit_gravity: bool,
    /// What is the surface gravity, in hex/turn^2 (aka 0.1 AU/day^2 aka m/s^2)
    surface_gravity: f32,
    /// What resources can be obtained?
    ///
    /// - Mining(Both|Ice|Ore) = must land, if a body with gravity, with a
    ///   landing manoeuver, or rendezvoused for bodies without gravity
    /// - Skimming = if in orbit, may use a skimming manoeuver
    resources: Resources,
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CelestialId(u8);

#[derive(Debug, Serialize, Deserialize)]
enum Resources {
    MiningBoth,
    MiningIce,
    MiningOre,
    Skimming,
    None,
}

#[cfg(feature = "server")]
impl Celestial {
    pub fn solar_system(
        celestial_id_generator: &mut impl Iterator<Item = CelestialId>,
    ) -> (HashMap<CelestialId, Celestial>, CelestialId) {
        let mut map = HashMap::new();
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2 { q: 0, r: 0 },
                name: String::from("The Sun"),
                orbit_gravity: true,
                surface_gravity: 274.0,
                resources: Resources::None,
            },
        );
        todo!()
    }

    /// Can only land on bodies you can get resources from via mining
    ///
    /// So you can't land on Earth, the Sun, or gas giants
    fn can_land(&self) -> bool {
        !matches!(self.resources, Resources::Skimming)
    }

    /// Generate orbital parameters; assumes body has gravity
    pub fn orbit_parameters(&self, clockwise: bool) -> [(Vec2<i32>, Vec2<i32>); 6] {
        self.position
            .neighbours()
            .iter()
            .cloned()
            .zip(
                Vec2::zero()
                    .neighbours()
                    .iter()
                    .cycle()
                    .skip(if clockwise { 2 } else { 4 })
                    .cloned(),
            )
            .collect::<Vec<_>>()
            .try_into()
            .expect("base vector has exact number of args")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_parameters() {
        let celestial = Celestial {
            position: Vec2 { q: 1, r: 1 },
            name: String::from("Test Body"),
            orbit_gravity: true,
            surface_gravity: f32::NAN,
            resources: Resources::None,
        };

        assert_eq!(
            celestial.orbit_parameters(true)[0],
            (celestial.position.up(), Vec2::unit_down_right())
        );
        assert_eq!(
            celestial.orbit_parameters(true)[1],
            (celestial.position.up_right(), Vec2::unit_down())
        );
        assert_eq!(
            celestial.orbit_parameters(false)[0],
            (celestial.position.up(), Vec2::unit_down_left())
        );
        assert_eq!(
            celestial.orbit_parameters(false)[1],
            (celestial.position.up_right(), Vec2::unit_up_left())
        );
    }
}
