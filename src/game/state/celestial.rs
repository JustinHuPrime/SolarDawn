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

use num_traits::Num;
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};
use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::vec2::AxialPosition;

use super::{stack::Stack, Id, IdGenerator, InventoryList};

type Colour = String;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum AsteroidResource {
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
impl<T: Num + From<u8>> From<AsteroidResource> for InventoryList<T> {
    fn from(value: AsteroidResource) -> Self {
        match value {
            AsteroidResource::Ice => InventoryList::ice(2.into()),
            AsteroidResource::Ore => InventoryList::ore(2.into()),
            AsteroidResource::None => InventoryList::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AsteroidField {
    id: Id,
    pub position: AxialPosition,
    pub resource: AsteroidResource,
}
impl AsteroidField {
    pub fn new(id_generator: &mut IdGenerator, position: AxialPosition) -> Self {
        Self {
            id: id_generator.generate(),
            position,
            resource: thread_rng().gen(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CelestialBody {
    pub id: Id,
    pub position: AxialPosition,
    #[serde(deserialize_with = "CelestialBody::deserialize_hex_colour")]
    colour: Colour,
    #[serde(deserialize_with = "CelestialBody::deserialize_zero_to_one")]
    pub radius: f64,
}
impl CelestialBody {
    pub fn new(
        id_generator: &mut IdGenerator,
        position: AxialPosition,
        colour: Colour,
        radius: f64,
    ) -> Self {
        Self {
            id: id_generator.generate(),
            position,
            colour,
            radius,
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
