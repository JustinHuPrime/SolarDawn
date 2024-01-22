// Copyright 2023 Justin Hu
//
// This file is part of the Solar Dawn Server.
//
// The Solar Dawn Server is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// The Solar Dawn Server is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero
// General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with the Solar Dawn Server. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::vec2::Position;

use super::{stack::Stack, Id, IdGenerator};

type Colour = String;

#[derive(Serialize, Deserialize)]
pub enum CelestialObject {
    AsteroidField(AsteroidField),
    MinorBody(MinorBody),
    OrbitableBody(OrbitableBody),
    UnorbitableBody(UnorbitableBody),
}

#[derive(Serialize, Deserialize)]
pub struct CelestialBodyAppearance {
    #[serde(deserialize_with = "CelestialBodyAppearance::deserialize_hex_colour")]
    colour: Colour,
    #[serde(deserialize_with = "CelestialBodyAppearance::deserialize_zero_to_one")]
    radius: f64,
}
impl CelestialBodyAppearance {
    pub fn sol() -> CelestialBodyAppearance {
        CelestialBodyAppearance {
            colour: "#ffff00".to_owned(),
            radius: 0.8,
        }
    }

    fn deserialize_hex_colour<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        if Regex::new(r"^#[0-9a-f]{6}$")
            .expect("Hardcoded regex should be valid")
            .is_match(&s)
        {
            Ok(s)
        } else {
            Err(D::Error::custom(
                "the field 'colour' must be a valid hex colour",
            ))
        }
    }

    fn deserialize_zero_to_one<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f: f64 = Deserialize::deserialize(deserializer)?;

        if (0.0..=1.0).contains(&f) {
            Ok(f)
        } else {
            Err(D::Error::custom(
                "the field 'radius' must be between 0 and 1",
            ))
        }
    }
}

#[derive(Serialize, Deserialize)]
enum AsteroidResource {
    Ice,
    Ore,
    None,
}
impl Distribution<AsteroidResource> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AsteroidResource {
        match rng.gen_range(1..=6) {
            1 => AsteroidResource::Ice,
            6 => AsteroidResource::Ore,
            _ => AsteroidResource::None,
        }
    }
}
impl TryFrom<&str> for AsteroidResource {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ice" => Ok(AsteroidResource::Ice),
            "ore" => Ok(AsteroidResource::Ore),
            "none" => Ok(AsteroidResource::None),
            _ => Err("invalid resource type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AsteroidField {
    id: Id,
    position: Position,
    resource: AsteroidResource,
}
impl AsteroidField {
    pub fn new(id_generator: &mut IdGenerator, position: Position) -> Self {
        Self {
            id: id_generator.generate(Some("asteroid_field")),
            position,
            resource: thread_rng().gen(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Base {
    id: Id,
    stack: Stack,
    docked: Vec<Id>,
}
impl Base {
    pub fn new(stack: Stack) -> Self {
        Self {
            id: stack.id.clone(),
            stack,
            docked: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MinorBody {
    id: Id,
    position: Position,
    base: Option<Box<Base>>,
    appearance: CelestialBodyAppearance,
}
impl MinorBody {
    fn new(
        id_generator: &mut IdGenerator,
        position: Position,
        appearance: CelestialBodyAppearance,
    ) -> Self {
        Self {
            id: id_generator.generate(Some("minor_body")),
            position,
            base: None,
            appearance,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OrbitableBody {
    id: Id,
    position: Position,
    bases: [Option<Box<Base>>; 6],
    appearance: CelestialBodyAppearance,
}
impl OrbitableBody {
    pub fn new(
        id_generator: &mut IdGenerator,
        position: Position,
        appearance: CelestialBodyAppearance,
    ) -> Self {
        Self {
            id: id_generator.generate(Some("orbitable_body")),
            position,
            bases: Default::default(),
            appearance,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UnorbitableBody {
    pub id: Id,
    position: Position,
    appearance: CelestialBodyAppearance,
}
impl UnorbitableBody {
    pub fn new(
        id_generator: &mut IdGenerator,
        position: Position,
        appearance: CelestialBodyAppearance,
    ) -> Self {
        Self {
            id: id_generator.generate(Some("unorbitable_body")),
            position,
            appearance,
        }
    }
}
