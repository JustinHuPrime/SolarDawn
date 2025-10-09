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

#[cfg(feature = "server")]
use crate::CartesianVec2;
use crate::Vec2;

/// A celestial body
#[derive(Debug, Serialize, Deserialize)]
pub struct Celestial {
    /// Where this celestial body is positioned
    pub position: Vec2<i32>,
    /// What its name is (English)
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

/// Key to refer to celestial bodies
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

/// What resources are present on a celestial body
#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Resources {
    /// Can mine for both ice and ore if landed
    MiningBoth,
    /// Can mine for only ice if landed
    MiningIce,
    /// Can mine for only ore if landed
    MiningOre,
    /// Can't land, but can skim for fuel if in orbit
    Skimming,
    /// Can't land and can't get any resources
    None,
}

#[cfg(feature = "server")]
impl Celestial {
    /// Create a map of the solar system but with curated phase angles for the planets
    pub fn solar_system_balanced_positions(
        celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
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
        // TODO: Io, Europa, Ganymede, Callisto
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

    #[cfg(test)]
    /// Creates a simple solar system for testing purposes with a Sun, Earth, Moon, and asteroids.
    pub fn solar_system_for_testing(
        celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
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

        let earth_id = celestial_id_generator.next().expect("should be infinite");
        map.insert(
            earth_id,
            Celestial {
                position: Vec2::from_polar(10.0, 0.0),
                name: String::from("Earth"),
                orbit_gravity: true,
                surface_gravity: 9.8,
                resources: Resources::None,
                radius: 0.25,
            },
        );

        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).up_right().up_right(),
                name: String::from("Moon"),
                orbit_gravity: true,
                surface_gravity: 1.6,
                resources: Resources::MiningIce,
                radius: 0.1,
            },
        );

        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).down_left().down_left(),
                name: String::from("1 Ceres"),
                orbit_gravity: false,
                surface_gravity: 0.0,
                resources: Resources::MiningOre,
                radius: 0.05,
            },
        );

        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).down_left().down_left().down(),
                name: String::from("2 Vesta"),
                orbit_gravity: false,
                surface_gravity: 24.8,
                resources: Resources::MiningBoth,
                radius: 0.05,
            },
        );
        map.insert(
            celestial_id_generator.next().expect("should be infinite"),
            Celestial {
                position: Vec2::from_polar(15.0, 0.0),
                name: String::from("Jupiter"),
                orbit_gravity: true,
                surface_gravity: 24.8,
                resources: Resources::Skimming,
                radius: 0.75,
            },
        );

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

    /// When does the line from start to end collide with this celestial body, if at all
    ///
    /// Note bodies without gravity also have no collision
    pub fn collides(&self, start: CartesianVec2, end: CartesianVec2) -> Option<f32> {
        if !self.orbit_gravity {
            return None;
        }

        // find t such that self.radius^2 = (start - self.position + t*(end - start))^2
        let dp = start - self.position.cartesian();

        // special case - if already in range, collide immediately
        if dp.dot(dp) <= self.radius * self.radius {
            return Some(0.0);
        }

        let dv = end - start;

        // find t such that 0 = (dp + t*dv)^2 = dv⋅dv*t^2 + 2*dp⋅dv*t + dp⋅dp - self.radius^2
        let c = dp.dot(dp) - self.radius * self.radius;
        let b = 2.0 * dp.dot(dv);
        let a = dv.dot(dv);

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            // no collision
            return None;
        }

        let intersect_1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let intersect_2 = (-b + discriminant.sqrt()) / (2.0 * a);

        // note - if both are in range, intersect_1 is the first collision
        if (0.0..=1.0).contains(&intersect_1) {
            Some(intersect_1)
        } else if (0.0..=1.0).contains(&intersect_2) {
            Some(intersect_2)
        } else {
            None
        }
    }

    /// Given a starting position and velocity, what's the effect of this body's gravity
    pub fn gravity_to(&self, position: Vec2<i32>, velocity: Vec2<i32>) -> Vec2<i32> {
        if !self.orbit_gravity {
            return Vec2::zero();
        }

        // special case - if velocity is zero, apply the gravity on the occupied hex
        if velocity == Vec2::zero() {
            if let Some(gravity_hex) = self
                .position
                .neighbours()
                .into_iter()
                .find(|&neighbour| neighbour == position)
            {
                return self.position - gravity_hex;
            } else {
                return Vec2::zero();
            }
        }

        // each gravity hex defines a hex and a triangular slice of the planet's hex
        self.position
            .neighbours()
            .into_iter()
            .filter_map(|gravity_hex| {
                // ignore the starting hex
                if gravity_hex == position {
                    return None;
                }

                // this hex is effectively traversed if, at the point of closest approach, the distance is less than:
                // - the distance to any neighbouring hex, except the planet's own hex
                // - the distance to any other gravity hex

                let closest_approach =
                    Vec2::closest_approach(position, velocity, self.position, Vec2::zero());
                let closest_distance = Vec2::squared_distance_at_time(
                    position,
                    velocity,
                    self.position,
                    Vec2::zero(),
                    closest_approach,
                );

                gravity_hex
                    .neighbours()
                    .into_iter()
                    .filter(|&neighbour| neighbour != self.position)
                    .chain(
                        self.position
                            .neighbours()
                            .into_iter()
                            .filter(|&neighbour| neighbour != gravity_hex),
                    )
                    .map(|neighbour| {
                        Vec2::squared_distance_at_time(
                            position,
                            velocity,
                            neighbour,
                            Vec2::zero(),
                            closest_approach,
                        )
                    })
                    .all(|neighbour_distance| neighbour_distance >= closest_distance)
                    .then(|| self.position - gravity_hex)
            })
            .sum()
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
    fn test_solar_system_for_testing() {
        let mut id_gen = IdGen::<CelestialId>::new();
        let (celestials, earth_id) = Celestial::solar_system_for_testing(&mut id_gen);

        // Should have 6 celestials: Sun, Earth, Moon, 2 asteroids, Jupiter
        assert_eq!(celestials.len(), 6);

        // Earth should be found
        let earth = celestials.get(&earth_id).expect("Earth should exist");
        assert_eq!(earth.name, "Earth");
        assert!(earth.orbit_gravity);
        assert_eq!(earth.surface_gravity, 9.8);
        assert!(matches!(earth.resources, Resources::None));

        // Sun should be at origin
        let sun = celestials
            .values()
            .find(|c| c.name == "The Sun")
            .expect("Sun should exist");
        assert_eq!(sun.position, Vec2::zero());
        assert!(!sun.can_land()); // Sun has Resources::None

        // Moon should have ice mining
        let moon = celestials
            .values()
            .find(|c| c.name == "Moon")
            .expect("Moon should exist");
        assert!(matches!(moon.resources, Resources::MiningIce));
        assert!(moon.can_land());

        // Check asteroids
        let ceres = celestials
            .values()
            .find(|c| c.name == "1 Ceres")
            .expect("Ceres should exist");
        assert!(!ceres.orbit_gravity); // Asteroids don't have orbit gravity
        assert!(matches!(ceres.resources, Resources::MiningOre));

        let vesta = celestials
            .values()
            .find(|c| c.name == "2 Vesta")
            .expect("Vesta should exist");
        assert!(matches!(vesta.resources, Resources::MiningBoth));

        // Jupiter should be a gas giant
        let jupiter = celestials
            .values()
            .find(|c| c.name == "Jupiter")
            .expect("Jupiter should exist");
        assert!(matches!(jupiter.resources, Resources::Skimming));
        assert!(!jupiter.can_land()); // Gas giants can't be landed on
    }

    #[test]
    fn test_resources_enum() {
        // Test all resource types
        let mining_both = Resources::MiningBoth;
        let mining_ice = Resources::MiningIce;
        let mining_ore = Resources::MiningOre;
        let skimming = Resources::Skimming;
        let none = Resources::None;

        // Create celestials with each resource type to test can_land
        let mining_both_body = Celestial {
            position: Vec2::zero(),
            name: "Mining Both".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_both,
            radius: 0.5,
        };
        assert!(mining_both_body.can_land());

        let mining_ice_body = Celestial {
            position: Vec2::zero(),
            name: "Ice World".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_ice,
            radius: 0.5,
        };
        assert!(mining_ice_body.can_land());

        let mining_ore_body = Celestial {
            position: Vec2::zero(),
            name: "Ore World".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_ore,
            radius: 0.5,
        };
        assert!(mining_ore_body.can_land());

        let gas_giant = Celestial {
            position: Vec2::zero(),
            name: "Gas Giant".to_string(),
            orbit_gravity: true,
            surface_gravity: 20.0,
            resources: skimming,
            radius: 1.0,
        };
        assert!(!gas_giant.can_land());

        let no_resources_body = Celestial {
            position: Vec2::zero(),
            name: "Barren".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: none,
            radius: 0.5,
        };
        assert!(!no_resources_body.can_land());
    }

    #[test]
    fn test_celestial_id_conversions() {
        let id_val = 42u8;
        let celestial_id = CelestialId::from(id_val);
        let back_to_u8: u8 = celestial_id.into();
        assert_eq!(id_val, back_to_u8);
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_orbit_parameters_clockwise_counterclockwise() {
        let celestial = Celestial {
            position: Vec2 { q: 2, r: 3 },
            name: "Test Body".to_string(),
            orbit_gravity: true,
            surface_gravity: 5.0,
            resources: Resources::None,
            radius: 0.4,
        };

        let clockwise_params = celestial.orbit_parameters(true);
        let counterclockwise_params = celestial.orbit_parameters(false);

        // Both should return 6 orbital positions
        assert_eq!(clockwise_params.len(), 6);
        assert_eq!(counterclockwise_params.len(), 6);

        // All positions should be neighbors of the celestial
        let neighbors = celestial.position.neighbours();
        for (position, _velocity) in &clockwise_params {
            assert!(neighbors.contains(position));
        }
        for (position, _velocity) in &counterclockwise_params {
            assert!(neighbors.contains(position));
        }

        // Velocities should have norm 1
        for (_position, velocity) in &clockwise_params {
            assert_eq!(velocity.norm(), 1);
        }
        for (_position, velocity) in &counterclockwise_params {
            assert_eq!(velocity.norm(), 1);
        }

        // Clockwise and counterclockwise should give different velocity patterns
        assert_ne!(clockwise_params, counterclockwise_params);
    }

    #[test]
    fn test_collision_edge_cases() {
        let body = Celestial {
            position: Vec2 { q: 5, r: 5 },
            name: "Test Body".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 1.0,
        };

        // Line that starts and ends at the same point (zero length)
        assert!(
            body.collides(
                Vec2 { q: 10, r: 10 }.cartesian(),
                Vec2 { q: 10, r: 10 }.cartesian()
            )
            .is_none()
        );

        // Line that just touches the edge of the circle
        // This is harder to test precisely due to floating point, so test approximate collision
        let close_start = Vec2 { q: 5, r: 4 }; // Just outside
        let close_end = Vec2 { q: 5, r: 6 }; // Just outside on other side
        // This line passes very close to center and should collide
        assert!(
            body.collides(close_start.cartesian(), close_end.cartesian())
                .is_some()
        );

        // Line that starts inside (should collide)
        assert!(
            body.collides(body.position.cartesian(), Vec2 { q: 10, r: 10 }.cartesian())
                .is_some()
        );
    }

    #[test]
    fn test_collision_with_zero_radius() {
        let point_body = Celestial {
            position: Vec2::zero(),
            name: "Point".to_string(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.0,
        };

        // Only lines that pass exactly through the center should collide
        assert!(
            point_body
                .collides(
                    Vec2 { q: -1, r: 0 }.cartesian(),
                    Vec2 { q: 1, r: 0 }.cartesian()
                )
                .is_some()
        );
        assert!(
            point_body
                .collides(
                    Vec2 { q: -1, r: 1 }.cartesian(),
                    Vec2 { q: 1, r: 1 }.cartesian()
                )
                .is_none()
        );
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_balanced_solar_system_properties() {
        let mut id_gen = IdGen::<CelestialId>::new();
        let (celestials, earth_id) = Celestial::solar_system_balanced_positions(&mut id_gen);

        // Should have at least 8 celestials (Sun through Neptune)
        assert!(celestials.len() >= 8);

        // All celestials should have positive radius and reasonable properties
        for celestial in celestials.values() {
            assert!(celestial.radius > 0.0);
            assert!(celestial.surface_gravity >= 0.0);

            // Gas giants should have skimming resources
            if celestial.name.contains("Jupiter")
                || celestial.name.contains("Saturn")
                || celestial.name.contains("Uranus")
                || celestial.name.contains("Neptune")
            {
                assert!(matches!(celestial.resources, Resources::Skimming));
                assert!(!celestial.can_land());
            }

            // Rocky planets should have mining or no resources
            if celestial.name.contains("Mercury")
                || celestial.name.contains("Venus")
                || celestial.name.contains("Mars")
            {
                assert!(matches!(
                    celestial.resources,
                    Resources::MiningOre | Resources::MiningBoth
                ));
                assert!(celestial.can_land());
            }
        }

        // Earth should exist and have expected properties
        let earth = celestials.get(&earth_id).expect("Earth should exist");
        assert_eq!(earth.name, "Earth");
        assert!(matches!(earth.resources, Resources::None));
        assert!(!earth.can_land()); // Earth has no resources so can't land
    }

    #[test]
    fn test_can_land() {
        // Bodies with mining resources should allow landing
        let mining_both_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Mining Both"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningBoth,
            radius: 0.5,
        };
        assert!(mining_both_body.can_land());

        let mining_ice_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Mining Ice"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningIce,
            radius: 0.5,
        };
        assert!(mining_ice_body.can_land());

        let mining_ore_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Mining Ore"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningOre,
            radius: 0.5,
        };
        assert!(mining_ore_body.can_land());

        // Bodies with skimming or no resources should not allow landing
        let skimming_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Gas Giant"),
            orbit_gravity: true,
            surface_gravity: 20.0,
            resources: Resources::Skimming,
            radius: 1.0,
        };
        assert!(!skimming_body.can_land());

        let no_resources_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Earth"),
            orbit_gravity: true,
            surface_gravity: 9.8,
            resources: Resources::None,
            radius: 0.25,
        };
        assert!(!no_resources_body.can_land());
    }

    #[test]
    fn test_collides() {
        // Body with gravity at origin with radius 1.0
        let body = Celestial {
            position: Vec2::zero(),
            name: String::from("Test Body"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 1.0,
        };

        // Line passing through center should collide
        assert!(
            body.collides(
                Vec2 { q: -3, r: 0 }.cartesian(),
                Vec2 { q: 3, r: 0 }.cartesian()
            )
            .is_some()
        );
        // Line passing through center hex but offset from exact center should collide
        assert!(
            body.collides(
                Vec2 { q: -2, r: 1 }.cartesian(),
                Vec2 { q: 1, r: 0 }.cartesian()
            )
            .is_some()
        );
        // Line far from center should not collide
        assert!(
            body.collides(
                Vec2 { q: -2, r: 3 }.cartesian(),
                Vec2 { q: 2, r: 3 }.cartesian()
            )
            .is_none()
        );
        // Line segment starting at center should collide
        assert!(
            body.collides(Vec2::zero().cartesian(), Vec2 { q: 2, r: 0 }.cartesian())
                .is_some()
        );
        // Line segment entirely far away should not collide
        assert!(
            body.collides(
                Vec2 { q: 5, r: 0 }.cartesian(),
                Vec2 { q: 6, r: 0 }.cartesian()
            )
            .is_none()
        );

        // Body without gravity should never collide
        let no_gravity_body = Celestial {
            position: Vec2::zero(),
            name: String::from("No Gravity"),
            orbit_gravity: false,
            surface_gravity: 0.0,
            resources: Resources::MiningIce,
            radius: 1.0,
        };

        // Even a line passing through should not collide if no gravity
        assert!(
            no_gravity_body
                .collides(
                    Vec2 { q: -2, r: 0 }.cartesian(),
                    Vec2 { q: 2, r: 0 }.cartesian()
                )
                .is_none()
        );

        // Test with body at different position
        let offset_body = Celestial {
            position: Vec2 { q: 3, r: 2 },
            name: String::from("Offset Body"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.5,
        };

        // Line passing through offset body center should collide
        assert!(
            offset_body
                .collides(
                    Vec2 { q: 2, r: 2 }.cartesian(),
                    Vec2 { q: 4, r: 2 }.cartesian()
                )
                .is_some()
        );
        // Line missing offset body should not collide
        assert!(
            offset_body
                .collides(
                    Vec2 { q: 2, r: 5 }.cartesian(),
                    Vec2 { q: 4, r: 5 }.cartesian()
                )
                .is_none()
        );

        // Test edge case with very small body
        let small_body = Celestial {
            position: Vec2::zero(),
            name: String::from("Small Body"),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.1,
        };

        // Line passing very close to center should collide
        assert!(
            small_body
                .collides(
                    Vec2 { q: -1, r: 0 }.cartesian(),
                    Vec2 { q: 1, r: 0 }.cartesian()
                )
                .is_some()
        );
        // Line passing farther away should not collide
        assert!(
            small_body
                .collides(
                    Vec2 { q: -1, r: 1 }.cartesian(),
                    Vec2 { q: 1, r: 1 }.cartesian()
                )
                .is_none()
        );
    }
}
