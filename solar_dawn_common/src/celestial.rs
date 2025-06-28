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

//! Celestial bodies

#[cfg(feature = "server")]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Vec2;

#[derive(Debug, Serialize, Deserialize)]
pub struct Celestial {
    pub position: Vec2<i32>,
    pub name: String,
    /// Are there gravity effects in the one-hex ring around this body (for orbits)
    pub orbit_gravity: bool,
    /// What is the surface gravity, in m/s^2 (aka 0.05 AU/day^2 aka 0.5 hex/turn^2)
    pub surface_gravity: f32,
    /// What resources can be obtained?
    ///
    /// - Mining(Both|Ice|Ore) = must land, if a body with gravity, with a
    ///   landing manoeuver, or rendezvoused for bodies without gravity
    /// - Skimming = if in orbit, may use a skimming manoeuver
    pub resources: Resources,
    /// What is the radius of this body, in units of hex major radii
    ///
    /// Note: the minor radius is √3/2, or 0.866...
    pub radius: f32,
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CelestialId(u8);

#[cfg(feature = "server")]
impl From<u8> for CelestialId {
    fn from(value: u8) -> Self {
        CelestialId(value)
    }
}

#[cfg(feature = "server")]
impl From<CelestialId> for u8 {
    fn from(value: CelestialId) -> Self {
        value.0
    }
}

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Resources {
    MiningBoth,
    MiningIce,
    MiningOre,
    Skimming,
    None,
}

#[cfg(feature = "server")]
impl Celestial {
    pub fn solar_system_balanced_positions(
        celestial_id_generator: &mut impl Iterator<Item = CelestialId>,
    ) -> (HashMap<CelestialId, Celestial>, CelestialId) {
        let mut map = HashMap::new();
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::zero(),
                name: String::from("The Sun"),
                orbit_gravity: true,
                surface_gravity: 274.0,
                resources: Resources::None,
                radius: 0.85,
            },
        );
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(3.5, 170.0 * std::f32::consts::PI / 180.0),
                name: String::from("Mercury"),
                orbit_gravity: true,
                surface_gravity: 3.7,
                resources: Resources::MiningOre,
                radius: 0.15,
            },
        );
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(7.2, 70.0 * std::f32::consts::PI / 180.0),
                name: String::from("Venus"),
                orbit_gravity: true,
                surface_gravity: 8.9,
                resources: Resources::MiningOre,
                radius: 0.25,
            },
        );
        let earth_id = celestial_id_generator.next().expect("should be infinite");
        map.insert(
            earth_id,
            Celestial {
                position: Vec2::from_polar(10.0, 0.0 * std::f32::consts::PI / 180.0),
                name: String::from("Earth"),
                orbit_gravity: true,
                surface_gravity: 9.8,
                resources: Resources::None,
                radius: 0.25,
            },
        );
        // TODO: The Moon
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(14.8, 280.0 * std::f32::consts::PI / 180.0),
                name: String::from("Mars"),
                orbit_gravity: true,
                surface_gravity: 3.7,
                resources: Resources::MiningBoth,
                radius: 0.20,
            },
        );
        // TODO: Phobos, Deimos
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0),
                name: String::from("Jupiter"),
                orbit_gravity: true,
                surface_gravity: 24.8,
                resources: Resources::Skimming,
                radius: 0.75,
            },
        );
        // TODO: Io, Europa, Ganymede, Calliston
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0),
                name: String::from("Saturn"),
                orbit_gravity: true,
                surface_gravity: 10.4,
                resources: Resources::Skimming,
                radius: 0.70,
            },
        );
        // TODO: Titan, Rhea, Iapetus, Dione, Tethys
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0),
                name: String::from("Uranus"),
                orbit_gravity: true,
                surface_gravity: 8.9,
                resources: Resources::Skimming,
                radius: 0.45,
            },
        );
        // TODO: Titania, Oberon, Umbriel, Ariel
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(299.7, 235.0 * std::f32::consts::PI / 180.0),
                name: String::from("Neptune"),
                orbit_gravity: true,
                surface_gravity: 11.1,
                resources: Resources::Skimming,
                radius: 0.50,
            },
        );
        // TODO: Triton
        // TODO: asteroid belt
        // TODO: kuiper belt
        (map, earth_id)
    }

    /// Can only land on bodies you can get resources from via mining
    ///
    /// So you can't land on Earth, the Sun, or gas giants
    pub fn can_land(&self) -> bool {
        matches!(
            self.resources,
            Resources::MiningBoth | Resources::MiningIce | Resources::MiningOre
        )
    }

    /// Generate orbital parameters; assumes body has gravity
    pub fn orbit_parameters(&self, clockwise: bool) -> [(Vec2<i32>, Vec2<i32>); 6] {
        debug_assert!(self.orbit_gravity);
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

    /// Does the line from start to end collide with this celestial body
    ///
    /// Note bodies without gravity also have no collision
    pub fn collides(&self, start: Vec2<i32>, end: Vec2<i32>) -> bool {
        if !self.orbit_gravity {
            return false;
        }

        let start = start.cartesian();
        let end = end.cartesian();
        let direction = (end.0 - start.0, end.1 - start.1);
        // line is of the form start + t * direction
        let center = self.position.cartesian();
        let radius = self.radius;
        // circle is of the form (x - center)^2 + (y - center)^2 = radius^2
        let offset_start = (start.0 - center.0, start.1 - center.1);

        // construct quadratic equation of intersections according to https://stackoverflow.com/a/1084899

        let a = direction.0 * direction.0 + direction.1 + direction.1;
        let b = 2.0 * (direction.0 * offset_start.0 + direction.1 + offset_start.1);
        let c =
            (offset_start.0 * offset_start.0 + offset_start.1 * offset_start.1) - (radius * radius);

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            // no collision
            return false;
        }

        let intersect_1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let intersect_2 = (-b + discriminant.sqrt()) / (2.0 * a);

        (0.0..=1.0).contains(&intersect_1) || (0.0..=1.0).contains(&intersect_2)
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;

    struct IdGen<T: From<u8>> {
        next: u8,
        _t: PhantomData<T>,
    }
    impl<T: From<u8>> IdGen<T> {
        pub fn new() -> Self {
            Self {
                next: 0,
                _t: PhantomData,
            }
        }
    }
    impl<T: From<u8>> Iterator for IdGen<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let result = Some(self.next.into());
            self.next += 1;
            result
        }
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_orbit_parameters() {
        let celestial = Celestial {
            position: Vec2 { q: 1, r: 1 },
            name: String::from("Test Body"),
            orbit_gravity: true,
            surface_gravity: f32::NAN,
            resources: Resources::None,
            radius: f32::NAN,
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

    #[cfg(feature = "server")]
    #[test]
    fn test_custom_solar_system() {
        let mut id_gen = IdGen::<CelestialId>::new();
        let (celestials, earth_id) = Celestial::solar_system_balanced_positions(&mut id_gen);
        assert_eq!(celestials.get(&earth_id).unwrap().name, "Earth");
    }
}
