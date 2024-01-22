use serde::{Deserialize, Serialize};

use crate::vec2::{Displacement, Position};

use super::{Id, Owner};

#[derive(Serialize, Deserialize)]
pub enum OrdnanceType {
    Mine,
    Torpedo,
    Nuke,
}
impl TryFrom<&str> for OrdnanceType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "mine" => Ok(OrdnanceType::Mine),
            "torpedo" => Ok(OrdnanceType::Torpedo),
            "nuke" => Ok(OrdnanceType::Nuke),
            _ => Err("invalid ordnance type"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ordnance {
    id: Id,
    owner: Owner,
    ordnance_type: OrdnanceType,
    position: Position,
    velocity: Displacement,
}

#[derive(Serialize, Deserialize)]
pub struct Stack {
    pub id: Id,
    owner: Owner,
    name: String,
    position: Position,
    velocity: Displacement,
    fuel_tanks: Vec<FuelTank>,
    cargo_holds: Vec<CargoHold>,
    engines: Vec<Engine>,
    guns: Vec<Gun>,
    launch_tubes: Vec<LaunchClamp>,
    habitat: Vec<Habitat>,
    miners: Vec<Miner>,
    factories: Vec<Factory>,
    armour_plates: Vec<ArmourPlate>,
}

#[derive(Serialize, Deserialize)]
struct FuelTank {
    id: Id,
    fuel: u64,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct CargoHold {
    id: Id,
    ore_amount: u64,
    materials_amount: u64,
    ice_amount: u64,
    mines: u64,
    torpedoes: u64,
    nukes: u64,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct Engine {
    id: Id,
    /// Has this engine overloaded? None = can't, Some(true) = ready to overload, Some(false) = not ready - needs overhaul
    overload_state: Option<bool>,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct Gun {
    id: Id,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct LaunchClamp {
    id: Id,
    load: Option<OrdnanceType>,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct Habitat {
    id: Id,
    owner: Owner,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct Miner {
    id: Id,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct Factory {
    id: Id,
    damaged: bool,
}

#[derive(Serialize, Deserialize)]
struct ArmourPlate {
    id: Id,
    damaged: bool,
}
