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

use crate::vec2::AxialDisplacement;

use super::state::{Id, InventoryList};

pub enum Order {
    Production(Production),
    CargoTransfer(CargoTransfer),
    StackTransfer(StackTransfer),
    Reload(Reload),
    HabitatRepair(HabitatRepair),
    FactoryRepair(FactoryRepair),
    Abort(Abort),
    Launch(Launch),
    Shoot(Shoot),
    Burn(Burn),
}

pub enum ProductionRecipe {
    OreToMaterials,
    IceToFuel,
    Mine,
    Torpedo,
    Nuke,
    FuelTank,
    CargoHold,
    CivilianEngine,
    MilitaryEngine,
    Gun,
    LaunchClamp,
    HabitatModule,
    Miner,
    Factory,
    ArmourPlate,
}
impl ProductionRecipe {
    fn cost(&self) -> InventoryList<u64> {
        match self {
            ProductionRecipe::OreToMaterials => InventoryList::ore(1),
            ProductionRecipe::IceToFuel => InventoryList::ice(2),
            ProductionRecipe::Mine => InventoryList::materials(1),
            ProductionRecipe::Torpedo => InventoryList::materials(1),
            ProductionRecipe::Nuke => InventoryList::materials(2),
            ProductionRecipe::FuelTank => InventoryList::materials(2),
            ProductionRecipe::CargoHold => InventoryList::materials(2),
            ProductionRecipe::CivilianEngine => InventoryList::materials(3),
            ProductionRecipe::MilitaryEngine => InventoryList::materials(5),
            ProductionRecipe::Gun => InventoryList::materials(4),
            ProductionRecipe::LaunchClamp => InventoryList::materials(2),
            ProductionRecipe::HabitatModule => InventoryList::materials(3),
            ProductionRecipe::Miner => InventoryList::materials(10),
            ProductionRecipe::Factory => InventoryList::materials(100),
            ProductionRecipe::ArmourPlate => InventoryList::materials(1),
        }
    }
}

pub struct Production {
    stack: Id,
    recipe: ProductionRecipe,
    amount: u64,
}

pub struct CargoTransfer {
    from_stack: Id,
    from_cargo_hold: Option<Id>,
    to_stack: Id,
    to_cargo_hold: Option<Id>,
    delta: InventoryList<u64>,
}

pub enum StackTransferTarget {
    Existing(Id),
    New(u64),
}
pub struct StackTransfer {
    from_stack: Id,
    components: Vec<Id>,
    to_stack: StackTransferTarget,
}

pub struct Reload {
    from_stack: Id,
    from_cargo_hold: Option<Id>,
    to_stack: Id,
    to_launch_clamp: Id,
}

pub struct HabitatRepair {
    stack: Id,
    habitat: Id,
    cargo_hold: Option<Id>,
    component: Id,
}

pub struct FactoryRepair {
    factory_stack: Id,
    cargo_hold: Option<Id>,
    repaired_stack: Id,
    component: Id,
}

pub struct Abort {
    pub ordnance: Id,
}

pub struct Launch {
    pub stack: Id,
    pub launch_clamp: Id,
    pub boost: AxialDisplacement,
}

pub struct Shoot {
    pub shooter: Id,
    pub gun: Id,
    pub target: Id,
}

pub struct Burn {
    pub stack: Id,
    pub engine: Id,
    pub fuel_tank: Id,
    pub direction: AxialDisplacement,
}
