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
