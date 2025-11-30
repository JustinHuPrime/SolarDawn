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

use std::collections::{HashMap, HashSet};

#[cfg(feature = "server")]
use rand::{
    Rng, RngCore,
    distr::{Distribution, weighted::WeightedIndex},
    seq::SliceRandom,
};
use serde::{Deserialize, Serialize};

use crate::{CartesianVec2, Vec2};

/// A celestial body
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(Clone))]
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
    /// - Mining(Both|Water|Ore) = must land, if a body with gravity, with a
    ///   landing manoeuver, or rendezvoused for bodies without gravity
    /// - Skimming = if in orbit, may use a skimming manoeuver
    pub resources: Resources,
    /// What is the radius of this body, in units of hex major radii
    ///
    /// Note: the minor radius is √3/2, or 0.866...
    pub radius: f32,
    /// Colour to draw the celestial with
    pub colour: String,
    /// Is this a minor body (asteroid, kuiper belt object)
    pub is_minor: bool,
}

/// Key to refer to celestial bodies
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CelestialId(u32);

#[cfg(feature = "server")]
impl From<u32> for CelestialId {
    fn from(value: u32) -> Self {
        CelestialId(value)
    }
}

#[cfg(feature = "server")]
impl From<CelestialId> for u32 {
    fn from(value: CelestialId) -> Self {
        value.0
    }
}

/// HashMap with additional features for storing Celestials
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "client", derive(Clone))]
#[cfg_attr(feature = "server", derive(Default))]
pub struct CelestialMap {
    /// All celestials
    all: HashMap<CelestialId, Celestial>,
    /// All by position
    by_position: HashMap<Vec2<i32>, CelestialId>,
    /// All celestials with gravity (orbitable, landable, collidable, etc)
    with_gravity: HashSet<CelestialId>,
    /// Major celestials, in creation order (significant)
    majors: Vec<CelestialId>,
}

impl CelestialMap {
    /// View this as all celestials
    pub fn all(&self) -> &HashMap<CelestialId, Celestial> {
        &self.all
    }
    /// View this as only celestials with gravity
    pub fn with_gravity(&self) -> impl Iterator<Item = &Celestial> {
        self.with_gravity.iter().map(|id| &self.all[id])
    }
    /// Lookup a specific one by position
    pub fn get_by_position(&self, position: Vec2<i32>) -> Option<(CelestialId, &Celestial)> {
        self.by_position
            .get(&position)
            .and_then(|&id| self.all.get(&id).map(|celestial| (id, celestial)))
    }
    /// Lookup a specific one by id
    pub fn get(&self, id: CelestialId) -> Option<&Celestial> {
        self.all.get(&id)
    }
    /// Get list of major celestials
    pub fn majors(&self) -> &[CelestialId] {
        &self.majors
    }
}

#[cfg(feature = "server")]
impl From<HashMap<CelestialId, Celestial>> for CelestialMap {
    fn from(value: HashMap<CelestialId, Celestial>) -> Self {
        Self {
            by_position: value
                .iter()
                .map(|(&id, celestial)| (celestial.position, id))
                .collect::<HashMap<_, _>>(),
            with_gravity: value
                .iter()
                .filter_map(|(&id, celestial)| {
                    if celestial.orbit_gravity {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>(),
            majors: {
                let mut majors = value
                    .iter()
                    .filter_map(
                        |(&id, celestial)| {
                            if !celestial.is_minor { Some(id) } else { None }
                        },
                    )
                    .collect::<Vec<_>>();
                majors.sort();
                majors
            },
            all: value,
        }
    }
}

#[cfg(feature = "server")]
impl From<CelestialMap> for HashMap<CelestialId, Celestial> {
    fn from(value: CelestialMap) -> Self {
        value.all
    }
}

/// What resources are present on a celestial body
#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Resources {
    /// Can mine for both water and ore if landed
    MiningBoth,
    /// Can mine for only water if landed
    MiningWater,
    /// Can mine for only ore if landed
    MiningOre,
    /// Can't land, but can skim for fuel if in orbit
    Skimming,
    /// Can't land and can't get any resources
    None,
}

impl Celestial {
    /// Create a map of the solar system but with curated phase angles for the planets
    #[cfg(feature = "server")]
    pub fn solar_system_balanced_positions(
        celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
        rng: &mut dyn RngCore,
    ) -> (HashMap<CelestialId, Celestial>, CelestialId) {
        let mut map = HashMap::new();
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::zero(),
                name: "The Sun".to_owned(),
                orbit_gravity: true,
                surface_gravity: 274.0,
                resources: Resources::None,
                radius: 0.85,
                colour: "#ffff00".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(3.5, 170.0 * std::f32::consts::PI / 180.0),
                name: "Mercury".to_owned(),
                orbit_gravity: true,
                surface_gravity: 3.7,
                resources: Resources::MiningOre,
                radius: 0.15,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(7.2, 70.0 * std::f32::consts::PI / 180.0),
                name: "Venus".to_owned(),
                orbit_gravity: true,
                surface_gravity: 8.9,
                resources: Resources::MiningOre,
                radius: 0.25,
                colour: "#ffee99".to_owned(),
                is_minor: false,
            },
        );
        let earth_id = celestial_id_generator.next().unwrap();
        map.insert(
            earth_id,
            Celestial {
                position: Vec2::from_polar(10.0, 0.0 * std::f32::consts::PI / 180.0),
                name: "Earth".to_owned(),
                orbit_gravity: true,
                surface_gravity: 9.8,
                resources: Resources::None,
                radius: 0.25,
                colour: "#0000ff".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0 * std::f32::consts::PI / 180.0)
                    .up()
                    .up_right()
                    .up_right(),
                name: "The Moon".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.6,
                resources: Resources::MiningBoth,
                radius: 0.15,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(14.8, 280.0 * std::f32::consts::PI / 180.0),
                name: "Mars".to_owned(),
                orbit_gravity: true,
                surface_gravity: 3.7,
                resources: Resources::MiningBoth,
                radius: 0.20,
                colour: "#cc5151".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(14.8, 280.0 * std::f32::consts::PI / 180.0)
                    .up_left()
                    .up_left(),
                name: "Phobos".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.0,
                resources: Resources::MiningOre,
                radius: 0.10,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(14.8, 280.0 * std::f32::consts::PI / 180.0)
                    .down()
                    .down()
                    .down()
                    .down(),
                name: "Deimos".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.0,
                resources: Resources::MiningOre,
                radius: 0.10,
                colour: "#666666".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0),
                name: "Jupiter".to_owned(),
                orbit_gravity: true,
                surface_gravity: 24.8,
                resources: Resources::Skimming,
                radius: 0.75,
                colour: "#ffee99".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0)
                    .down_left()
                    .down_left()
                    .down_left(),
                name: "Io".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.8,
                resources: Resources::MiningOre,
                radius: 0.15,
                colour: "#ffff00".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0)
                    .up_right()
                    .up_right()
                    .up_right()
                    .up_right(),
                name: "Europa".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.3,
                resources: Resources::MiningWater,
                radius: 0.15,
                colour: "#88bbdd".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0)
                    .up()
                    .up()
                    .up()
                    .up()
                    .up(),
                name: "Ganymede".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.4,
                resources: Resources::MiningBoth,
                radius: 0.15,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(54.3, 45.0 * std::f32::consts::PI / 180.0)
                    .down()
                    .down()
                    .down()
                    .down()
                    .down()
                    .down(),
                name: "Callisto".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.2,
                resources: Resources::MiningBoth,
                radius: 0.15,
                colour: "#666666".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0),
                name: "Saturn".to_owned(),
                orbit_gravity: true,
                surface_gravity: 10.4,
                resources: Resources::Skimming,
                radius: 0.70,
                colour: "#ddcc77".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0)
                    .down_left()
                    .down_left(),
                name: "Tethys".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.1,
                resources: Resources::MiningWater,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0)
                    .down_right()
                    .down()
                    .down(),
                name: "Dione".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.2,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0)
                    .up()
                    .up()
                    .up()
                    .up(),
                name: "Rhea".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.3,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0)
                    .up()
                    .up()
                    .up()
                    .up()
                    .up_right()
                    .up_right(),
                name: "Titan".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.4,
                resources: Resources::MiningBoth,
                radius: 0.15,
                colour: "#ddcc77".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(100.4, 125.0 * std::f32::consts::PI / 180.0)
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left(),
                name: "Iapetus".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.2,
                resources: Resources::MiningWater,
                radius: 0.1,
                colour: "#444444".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0),
                name: "Uranus".to_owned(),
                orbit_gravity: true,
                surface_gravity: 8.9,
                resources: Resources::Skimming,
                radius: 0.45,
                colour: "#00cccc".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0)
                    .up()
                    .up(),
                name: "Miranda".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.1,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#aaaaaa".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0)
                    .down()
                    .down(),
                name: "Ariel".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.2,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#666666".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0)
                    .down_right()
                    .down_right()
                    .down_right(),
                name: "Umbriel".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.2,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#666666".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0)
                    .up_left()
                    .up_left()
                    .up_left()
                    .up_left(),
                name: "Titania".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.4,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(194.7, 305.0 * std::f32::consts::PI / 180.0)
                    .up_right()
                    .up_right()
                    .up_right()
                    .up_right()
                    .up_right(),
                name: "Oberon".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.4,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(299.7, 235.0 * std::f32::consts::PI / 180.0),
                name: "Neptune".to_owned(),
                orbit_gravity: true,
                surface_gravity: 11.1,
                resources: Resources::Skimming,
                radius: 0.50,
                colour: "#0055cc".to_owned(),
                is_minor: false,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(299.7, 235.0 * std::f32::consts::PI / 180.0)
                    .up_right()
                    .up_right()
                    .up_right()
                    .up_right()
                    .up_right(),
                name: "Triton".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.8,
                resources: Resources::MiningBoth,
                radius: 0.1,
                colour: "#888888".to_owned(),
                is_minor: false,
            },
        );

        // minor bodies
        let mut minor_body_numbers = (1_u32..=875150).collect::<Vec<_>>();
        minor_body_numbers.shuffle(rng);

        // generate asteroid belt
        let asteroid_belt_start = 21;
        let asteroid_belt_end = 32;
        let asteroid_belt_resources = [
            (0.8, Resources::MiningOre),
            (0.1, Resources::MiningBoth),
            (0.1, Resources::MiningWater),
        ];
        let asteroid_belt_distribution =
            WeightedIndex::new(asteroid_belt_resources.iter().map(|&(weight, _)| weight)).unwrap();
        let mut considered_positions = HashSet::new();
        for r in asteroid_belt_start..=asteroid_belt_end {
            for theta in 0..(asteroid_belt_end as f32 * 2.0 * std::f32::consts::PI).ceil() as i32 {
                let candidate_position =
                    Vec2::from_polar(r as f32, theta as f32 / 201.0 * 2.0 * std::f32::consts::PI);
                if !considered_positions.insert(candidate_position) {
                    continue;
                }

                // Calculate probability using semicircle distribution
                // Normalize r to range [0, 1] where 0.5 is the middle of the belt
                let belt_width = asteroid_belt_end - asteroid_belt_start + 1;
                let normalized_r = (r - asteroid_belt_start) as f32 / belt_width as f32;
                // Semicircle formula: y = sqrt(1 - (x - 1)^2) where x is in [0, 1]
                // This gives maximum value of 1 at x = 0.5 and 0 at x = 0 and x = 1
                let semicircle_value = (1.0 - (normalized_r - 1.0).powi(2)).max(0.0).sqrt();
                // Scale to maximum probability of 0.5
                let probability = semicircle_value * 0.5;

                if !rng.random_bool(probability as f64) {
                    continue;
                }

                let resources = asteroid_belt_resources[asteroid_belt_distribution.sample(rng)].1;
                map.insert(
                    celestial_id_generator.next().unwrap(),
                    Celestial {
                        position: candidate_position,
                        name: format!("MBO {}", minor_body_numbers.pop().unwrap()),
                        orbit_gravity: false,
                        surface_gravity: 0.0,
                        resources,
                        radius: 0.1,
                        colour: match resources {
                            Resources::MiningBoth => "#888888".to_owned(),
                            Resources::MiningWater => "#aaaaaa".to_owned(),
                            Resources::MiningOre => "#666666".to_owned(),
                            _ => unreachable!(),
                        },
                        is_minor: true,
                    },
                );
            }
        }

        // generate kuiper belt
        let kuiper_belt_start = 395;
        let kuiper_belt_end = 487;
        let kuiper_belt_resources = [
            (0.9, Resources::MiningWater),
            (0.05, Resources::MiningBoth),
            (0.05, Resources::MiningOre),
        ];
        let kuiper_belt_distribution =
            WeightedIndex::new(kuiper_belt_resources.iter().map(|&(weight, _)| weight)).unwrap();
        let mut considered_positions = HashSet::new();
        for r in kuiper_belt_start..=kuiper_belt_end {
            for theta in 0..(kuiper_belt_end as f32 * 2.0 * std::f32::consts::PI).ceil() as i32 {
                let candidate_position =
                    Vec2::from_polar(r as f32, theta as f32 / 3047.0 * 2.0 * std::f32::consts::PI);
                if !considered_positions.insert(candidate_position) {
                    continue;
                }

                // Calculate probability using semicircle distribution
                // Normalize r to range [0, 1] where 0.5 is the middle of the belt
                let belt_width = kuiper_belt_end - kuiper_belt_start + 1;
                let normalized_r = (r - kuiper_belt_start) as f32 / belt_width as f32;
                // Semicircle formula: y = sqrt(1 - (x - 1)^2) where x is in [0, 1]
                // This gives maximum value of 1 at x = 0.5 and 0 at x = 0 and x = 1
                let semicircle_value = (1.0 - (normalized_r - 1.0).powi(2)).max(0.0).sqrt();
                // Scale to maximum probability of 0.5
                let probability = semicircle_value * 0.5;

                if !rng.random_bool(probability as f64) {
                    continue;
                }

                let resources = kuiper_belt_resources[kuiper_belt_distribution.sample(rng)].1;
                map.insert(
                    celestial_id_generator.next().unwrap(),
                    Celestial {
                        position: candidate_position,
                        name: format!("KBO {}", minor_body_numbers.pop().unwrap()),
                        orbit_gravity: false,
                        surface_gravity: 0.0,
                        resources,
                        radius: 0.1,
                        colour: match resources {
                            Resources::MiningBoth => "#888888".to_owned(),
                            Resources::MiningWater => "#aaaaaa".to_owned(),
                            Resources::MiningOre => "#666666".to_owned(),
                            _ => unreachable!(),
                        },
                        is_minor: true,
                    },
                );
            }
        }
        (map, earth_id)
    }

    #[cfg(test)]
    /// Creates a simple solar system for testing purposes with a Sun, Earth, Moon, and asteroids.
    #[cfg(feature = "server")]
    pub fn solar_system_for_testing(
        celestial_id_generator: &mut dyn Iterator<Item = CelestialId>,
    ) -> (HashMap<CelestialId, Celestial>, CelestialId) {
        let mut map = HashMap::new();
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::zero(),
                name: "The Sun".to_owned(),
                orbit_gravity: true,
                surface_gravity: 274.0,
                resources: Resources::None,
                radius: 0.85,
                colour: "#ffff00".to_owned(),
                is_minor: false,
            },
        );

        let earth_id = celestial_id_generator.next().unwrap();
        map.insert(
            earth_id,
            Celestial {
                position: Vec2::from_polar(10.0, 0.0),
                name: "Earth".to_owned(),
                orbit_gravity: true,
                surface_gravity: 9.8,
                resources: Resources::None,
                radius: 0.25,
                colour: "#0000ff".to_owned(),
                is_minor: false,
            },
        );

        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).up_right().up_right(),
                name: "Moon".to_owned(),
                orbit_gravity: true,
                surface_gravity: 1.6,
                resources: Resources::MiningWater,
                radius: 0.1,
                colour: "#aaaaaa".to_owned(),
                is_minor: false,
            },
        );

        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).down_left().down_left(),
                name: "1 Ceres".to_owned(),
                orbit_gravity: false,
                surface_gravity: 0.0,
                resources: Resources::MiningOre,
                radius: 0.05,
                colour: "#888888".to_owned(),
                is_minor: true,
            },
        );

        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(10.0, 0.0).down_left().down_left().down(),
                name: "2 Vesta".to_owned(),
                orbit_gravity: false,
                surface_gravity: 24.8,
                resources: Resources::MiningBoth,
                radius: 0.05,
                colour: "#999999".to_owned(),
                is_minor: true,
            },
        );
        map.insert(
            celestial_id_generator.next().unwrap(),
            Celestial {
                position: Vec2::from_polar(15.0, 0.0),
                name: "Jupiter".to_owned(),
                orbit_gravity: true,
                surface_gravity: 24.8,
                resources: Resources::Skimming,
                radius: 0.75,
                colour: "#ffee99".to_owned(),
                is_minor: false,
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
            Resources::MiningBoth | Resources::MiningWater | Resources::MiningOre
        )
    }

    fn intersects(&self, start: CartesianVec2, end: CartesianVec2) -> Option<(f32, f32)> {
        // find t such that self.radius^2 = (start - self.position + t*(end - start))^2
        let dp = start - self.position.cartesian();
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
        Some((intersect_1, intersect_2))
    }

    /// Would a stack moving from start to end collide with this celestial body, and if so, when?
    ///
    /// Note bodies without gravity also have no collision
    #[cfg(feature = "server")]
    pub fn stack_movement_collides(&self, start: CartesianVec2, end: CartesianVec2) -> Option<f32> {
        if !self.orbit_gravity {
            return None;
        }

        let (intersect_1, intersect_2) = self.intersects(start, end)?;

        // note - if both are in range, intersect_1 is the first collision
        if (0.0..=1.0).contains(&intersect_1) {
            Some(intersect_1)
        } else if (0.0..=1.0).contains(&intersect_2) {
            Some(intersect_2)
        } else {
            None
        }
    }

    /// Is a weapons effect originating from start blocked for a stack at end?
    ///
    /// Note bodies without gravity don't block
    pub fn blocks_weapons_effect(&self, start: CartesianVec2, end: CartesianVec2) -> bool {
        if !self.orbit_gravity {
            return false;
        }

        self.intersects(start, end)
            .is_some_and(|(intersect_1, intersect_2)| {
                // only blocks if BOTH intersection points are within the segment
                (0.0..=1.0).contains(&intersect_1)
                    && (0.0..=1.0).contains(&intersect_2)
                    && intersect_1 != intersect_2
            })
    }

    /// Given a starting position and velocity, what's the effect of this body's gravity
    #[cfg(feature = "server")]
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

                // this hex is effectively traversed if, at the point of closest approach to this hex, the distance is less than:
                // - the distance to any neighbouring hex, except the planet's own hex
                // - the distance to any other gravity hex

                let closest_approach =
                    Vec2::closest_approach(position, velocity, gravity_hex, Vec2::zero());
                let closest_distance = Vec2::squared_distance_at_time(
                    position,
                    velocity,
                    gravity_hex,
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
            .unwrap()
    }
}

#[cfg(all(test, feature = "server", feature = "client"))]
mod tests {
    use super::*;

    use std::marker::PhantomData;

    use rand::rng;

    struct IdGen<T: From<u32>> {
        next: u32,
        _t: PhantomData<T>,
    }
    impl<T: From<u32>> IdGen<T> {
        pub fn new() -> Self {
            Self {
                next: 0,
                _t: PhantomData,
            }
        }
    }
    impl<T: From<u32>> Iterator for IdGen<T> {
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
            name: "Test Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: f32::NAN,
            resources: Resources::None,
            radius: f32::NAN,
            colour: "#ffffff".to_owned(),
            is_minor: false,
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

        // Moon should have water mining
        let moon = celestials
            .values()
            .find(|c| c.name == "Moon")
            .expect("Moon should exist");
        assert!(matches!(moon.resources, Resources::MiningWater));
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
        let mining_ice = Resources::MiningWater;
        let mining_ore = Resources::MiningOre;
        let skimming = Resources::Skimming;
        let none = Resources::None;

        // Create celestials with each resource type to test can_land
        let mining_both_body = Celestial {
            position: Vec2::zero(),
            name: "Mining Both".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_both,
            radius: 0.5,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };
        assert!(mining_both_body.can_land());

        let mining_water_body = Celestial {
            position: Vec2::zero(),
            name: "Water World".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_ice,
            radius: 0.5,
            colour: "#aaddff".to_owned(),
            is_minor: false,
        };
        assert!(mining_water_body.can_land());

        let mining_ore_body = Celestial {
            position: Vec2::zero(),
            name: "Ore World".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: mining_ore,
            radius: 0.5,
            colour: "#996633".to_owned(),
            is_minor: false,
        };
        assert!(mining_ore_body.can_land());

        let gas_giant = Celestial {
            position: Vec2::zero(),
            name: "Gas Giant".to_owned(),
            orbit_gravity: true,
            surface_gravity: 20.0,
            resources: skimming,
            radius: 1.0,
            colour: "#ffaa77".to_owned(),
            is_minor: false,
        };
        assert!(!gas_giant.can_land());

        let no_resources_body = Celestial {
            position: Vec2::zero(),
            name: "Barren".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: none,
            radius: 0.5,
            colour: "#666666".to_owned(),
            is_minor: false,
        };
        assert!(!no_resources_body.can_land());
    }

    #[test]
    fn test_celestial_id_conversions() {
        let id_val = 42u32;
        let celestial_id = CelestialId::from(id_val);
        let back_to_u32: u32 = celestial_id.into();
        assert_eq!(id_val, back_to_u32);
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_orbit_parameters_clockwise_counterclockwise() {
        let celestial = Celestial {
            position: Vec2 { q: 2, r: 3 },
            name: "Test Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 5.0,
            resources: Resources::None,
            radius: 0.4,
            colour: "#ffffff".to_owned(),
            is_minor: false,
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
            name: "Test Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 1.0,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line that starts and ends at the same point (zero length)
        assert!(
            body.stack_movement_collides(
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
            body.stack_movement_collides(close_start.cartesian(), close_end.cartesian())
                .is_some()
        );

        // Line that starts inside (should collide)
        assert!(
            body.stack_movement_collides(
                body.position.cartesian(),
                Vec2 { q: 10, r: 10 }.cartesian()
            )
            .is_some()
        );
    }

    #[test]
    fn test_collision_with_zero_radius() {
        let point_body = Celestial {
            position: Vec2::zero(),
            name: "Point".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.0,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Only lines that pass exactly through the center should collide
        assert!(
            point_body
                .stack_movement_collides(
                    Vec2 { q: -1, r: 0 }.cartesian(),
                    Vec2 { q: 1, r: 0 }.cartesian()
                )
                .is_some()
        );
        assert!(
            point_body
                .stack_movement_collides(
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
        let (celestials, earth_id) =
            Celestial::solar_system_balanced_positions(&mut id_gen, &mut rng());

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
            name: "Mining Both".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningBoth,
            radius: 0.5,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };
        assert!(mining_both_body.can_land());

        let mining_water_body = Celestial {
            position: Vec2::zero(),
            name: "Mining Water".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningWater,
            radius: 0.5,
            colour: "#aaddff".to_owned(),
            is_minor: false,
        };
        assert!(mining_water_body.can_land());

        let mining_ore_body = Celestial {
            position: Vec2::zero(),
            name: "Mining Ore".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::MiningOre,
            radius: 0.5,
            colour: "#996633".to_owned(),
            is_minor: false,
        };
        assert!(mining_ore_body.can_land());

        // Bodies with skimming or no resources should not allow landing
        let skimming_body = Celestial {
            position: Vec2::zero(),
            name: "Gas Giant".to_owned(),
            orbit_gravity: true,
            surface_gravity: 20.0,
            resources: Resources::Skimming,
            radius: 1.0,
            colour: "#ffaa77".to_owned(),
            is_minor: false,
        };
        assert!(!skimming_body.can_land());

        let no_resources_body = Celestial {
            position: Vec2::zero(),
            name: "Earth".to_owned(),
            orbit_gravity: true,
            surface_gravity: 9.8,
            resources: Resources::None,
            radius: 0.25,
            colour: "#0000ff".to_owned(),
            is_minor: false,
        };
        assert!(!no_resources_body.can_land());
    }

    #[test]
    fn test_collides() {
        // Body with gravity at origin with radius 1.0
        let body = Celestial {
            position: Vec2::zero(),
            name: "Test Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 1.0,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing through center should collide
        assert!(
            body.stack_movement_collides(
                Vec2 { q: -3, r: 0 }.cartesian(),
                Vec2 { q: 3, r: 0 }.cartesian()
            )
            .is_some()
        );
        // Line passing through center hex but offset from exact center should collide
        assert!(
            body.stack_movement_collides(
                Vec2 { q: -2, r: 1 }.cartesian(),
                Vec2 { q: 1, r: 0 }.cartesian()
            )
            .is_some()
        );
        // Line far from center should not collide
        assert!(
            body.stack_movement_collides(
                Vec2 { q: -2, r: 3 }.cartesian(),
                Vec2 { q: 2, r: 3 }.cartesian()
            )
            .is_none()
        );
        // Line segment starting at center should collide
        assert!(
            body.stack_movement_collides(Vec2::zero().cartesian(), Vec2 { q: 2, r: 0 }.cartesian())
                .is_some()
        );
        // Line segment entirely far away should not collide
        assert!(
            body.stack_movement_collides(
                Vec2 { q: 5, r: 0 }.cartesian(),
                Vec2 { q: 6, r: 0 }.cartesian()
            )
            .is_none()
        );

        // Body without gravity should never collide
        let no_gravity_body = Celestial {
            position: Vec2::zero(),
            name: "No Gravity".to_owned(),
            orbit_gravity: false,
            surface_gravity: 0.0,
            resources: Resources::MiningWater,
            radius: 1.0,
            colour: "#aaddff".to_owned(),
            is_minor: false,
        };

        // Even a line passing through should not collide if no gravity
        assert!(
            no_gravity_body
                .stack_movement_collides(
                    Vec2 { q: -2, r: 0 }.cartesian(),
                    Vec2 { q: 2, r: 0 }.cartesian()
                )
                .is_none()
        );

        // Test with body at different position
        let offset_body = Celestial {
            position: Vec2 { q: 3, r: 2 },
            name: "Offset Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.5,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing through offset body center should collide
        assert!(
            offset_body
                .stack_movement_collides(
                    Vec2 { q: 2, r: 2 }.cartesian(),
                    Vec2 { q: 4, r: 2 }.cartesian()
                )
                .is_some()
        );
        // Line missing offset body should not collide
        assert!(
            offset_body
                .stack_movement_collides(
                    Vec2 { q: 2, r: 5 }.cartesian(),
                    Vec2 { q: 4, r: 5 }.cartesian()
                )
                .is_none()
        );

        // Test edge case with very small body
        let small_body = Celestial {
            position: Vec2::zero(),
            name: "Small Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.1,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing very close to center should collide
        assert!(
            small_body
                .stack_movement_collides(
                    Vec2 { q: -1, r: 0 }.cartesian(),
                    Vec2 { q: 1, r: 0 }.cartesian()
                )
                .is_some()
        );
        // Line passing farther away should not collide
        assert!(
            small_body
                .stack_movement_collides(
                    Vec2 { q: -1, r: 1 }.cartesian(),
                    Vec2 { q: 1, r: 1 }.cartesian()
                )
                .is_none()
        );
    }

    #[test]
    fn test_passes_through() {
        // Body with gravity at origin with radius 1.0
        let body = Celestial {
            position: Vec2::zero(),
            name: "Test Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 1.0,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing completely through center should pass through
        assert!(body.blocks_weapons_effect(
            Vec2 { q: -3, r: 0 }.cartesian(),
            Vec2 { q: 3, r: 0 }.cartesian()
        ));

        // Line passing through but offset should still pass through
        assert!(body.blocks_weapons_effect(
            Vec2 { q: -2, r: 1 }.cartesian(),
            Vec2 { q: 1, r: 0 }.cartesian()
        ));

        // Line starting at center (inside) going out should NOT pass through
        // because only one intersection point is in range
        assert!(
            !body.blocks_weapons_effect(Vec2::zero().cartesian(), Vec2 { q: 2, r: 0 }.cartesian())
        );

        // Line ending at center (inside) should NOT pass through
        assert!(
            !body.blocks_weapons_effect(Vec2 { q: -2, r: 0 }.cartesian(), Vec2::zero().cartesian())
        );

        // Line entirely far away should not pass through
        assert!(!body.blocks_weapons_effect(
            Vec2 { q: 5, r: 0 }.cartesian(),
            Vec2 { q: 6, r: 0 }.cartesian()
        ));

        // Line that grazes the surface (tangent) - should not pass through
        // because discriminant would be zero (or very small)
        // This is a subtle edge case

        // Body without gravity should never block
        let no_gravity_body = Celestial {
            position: Vec2::zero(),
            name: "No Gravity".to_owned(),
            orbit_gravity: false,
            surface_gravity: 0.0,
            resources: Resources::MiningWater,
            radius: 1.0,
            colour: "#aaddff".to_owned(),
            is_minor: false,
        };

        // Even a line passing through should not block if no gravity
        assert!(!no_gravity_body.blocks_weapons_effect(
            Vec2 { q: -2, r: 0 }.cartesian(),
            Vec2 { q: 2, r: 0 }.cartesian()
        ));

        // Test with body at different position
        let offset_body = Celestial {
            position: Vec2 { q: 3, r: 2 },
            name: "Offset Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.5,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing completely through offset body should pass through
        assert!(offset_body.blocks_weapons_effect(
            Vec2 { q: 2, r: 2 }.cartesian(),
            Vec2 { q: 4, r: 2 }.cartesian()
        ));

        // Line missing offset body should not pass through
        assert!(!offset_body.blocks_weapons_effect(
            Vec2 { q: 2, r: 5 }.cartesian(),
            Vec2 { q: 4, r: 5 }.cartesian()
        ));

        // Test with very small body - line passing through should still pass through
        let small_body = Celestial {
            position: Vec2::zero(),
            name: "Small Body".to_owned(),
            orbit_gravity: true,
            surface_gravity: 1.0,
            resources: Resources::None,
            radius: 0.1,
            colour: "#ffffff".to_owned(),
            is_minor: false,
        };

        // Line passing completely through small body
        assert!(small_body.blocks_weapons_effect(
            Vec2 { q: -1, r: 0 }.cartesian(),
            Vec2 { q: 1, r: 0 }.cartesian()
        ));

        // Line passing farther away should not pass through
        assert!(!small_body.blocks_weapons_effect(
            Vec2 { q: -1, r: 1 }.cartesian(),
            Vec2 { q: 1, r: 1 }.cartesian()
        ));
    }
}
