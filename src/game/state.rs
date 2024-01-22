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

use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

use crate::vec2::Position;

use self::{
    celestial::{CelestialBodyAppearance, CelestialObject, UnorbitableBody},
    stack::{Ordnance, Stack},
};

mod celestial;
mod stack;

#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
struct Id(String);
impl From<String> for Id {
    fn from(value: String) -> Self {
        Id(value)
    }
}
impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

#[derive(Serialize, Deserialize)]
struct IdGenerator {
    next: u64,
}
impl IdGenerator {
    fn generate(&mut self, prefix: Option<&str>) -> Id {
        let postfix = self.next.to_string();
        self.next += 1;
        if let Some(prefix) = prefix {
            prefix.to_owned() + "_" + &postfix
        } else {
            "object_".to_owned() + &postfix
        }
        .into()
    }
}
impl Default for IdGenerator {
    fn default() -> Self {
        Self { next: 1 }
    }
}

#[derive(Serialize, Deserialize)]
struct Owner(u8);
impl TryFrom<u8> for Owner {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 6 {
            Err("value too high")
        } else {
            Ok(Owner(value))
        }
    }
}
impl From<Owner> for u8 {
    fn from(value: Owner) -> Self {
        value.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub num_players: u8,
    id_generator: IdGenerator,
    stacks: HashMap<Id, Stack>,
    ordnance: HashMap<Id, Ordnance>,
    celestials: HashMap<Id, CelestialObject>,
}
impl GameState {
    const MIN_PLAYERS: u8 = 2;
    const MAX_PLAYERS: u8 = 6;

    pub fn new(num_players: u8) -> Result<Self, &'static str> {
        if num_players > Self::MAX_PLAYERS {
            return Err("too many players");
        } else if num_players < Self::MIN_PLAYERS {
            return Err("not enough players");
        }

        let mut id_generator = IdGenerator::default();
        let mut celestials = HashMap::default();

        // generate non-asteroid celestial bodies
        let sol = UnorbitableBody::new(
            &mut id_generator,
            Position::new(0, 0),
            CelestialBodyAppearance::sol(),
        );
        celestials.insert(sol.id.clone(), CelestialObject::UnorbitableBody(sol));

        // setup Earth bases

        // generate asteroids
        // TODO

        Ok(GameState {
            num_players,
            id_generator,
            stacks: HashMap::default(),
            ordnance: HashMap::default(),
            celestials,
        })
    }

    pub fn load_from_file(filename: &str) -> Result<Self, &'static str> {
        if let Ok(file) = fs::read_to_string(filename) {
            serde_json::from_str(&file).map_err(|_| "could not parse save file")
        } else {
            Err("could not read file")
        }
    }

    pub fn save_to_file(&self, filename: &str) {
        fn display_warning(filename: &str) {
            eprintln!("warning: unable to write to {filename} - your game will not be saved");
            eprintln!("stopping the server is strongly recommended");
        }

        if fs::write(
            filename,
            if let Ok(stringified) = serde_json::to_string(self) {
                stringified
            } else {
                display_warning(filename);
                return;
            },
        )
        .is_err()
        {
            display_warning(filename);
        }
    }
}
