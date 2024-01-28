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

use std::collections::HashMap;

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::vec2::{AxialDisplacement, AxialPosition};

use super::{Id, IdGenerator, InventoryList, Owner};

pub trait Positionable {
    fn get_position(&self) -> &AxialPosition;
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum OrdnanceType {
    Mine,
    Torpedo,
    Nuke,
}
impl OrdnanceType {
    pub fn max_boost(&self) -> u64 {
        match self {
            OrdnanceType::Mine => 0,
            OrdnanceType::Torpedo => 2,
            OrdnanceType::Nuke => 0,
        }
    }

    /// returns the fraction of a ship's total components that should be damaged
    pub fn damage_fraction(&self) -> f64 {
        match self {
            OrdnanceType::Mine => 1.0 / 3.0,
            OrdnanceType::Torpedo => 2.0 / 3.0,
            OrdnanceType::Nuke => panic!("shouldn't ask for damage from nukes"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ordnance {
    pub id: Id,
    pub owner: Owner,
    pub ordnance_type: OrdnanceType,
    pub position: AxialPosition,
    pub velocity: AxialDisplacement,
}
impl Ordnance {
    pub fn new(
        id_generator: &mut IdGenerator,
        owner: Owner,
        ordnance_type: OrdnanceType,
        position: AxialPosition,
        velocity: AxialDisplacement,
    ) -> Self {
        Self {
            id: id_generator.generate(),
            owner,
            ordnance_type,
            position,
            velocity,
        }
    }
}
impl Positionable for Ordnance {
    fn get_position(&self) -> &AxialPosition {
        &self.position
    }
}

pub trait Component: IdAble {
    fn damage(&mut self) -> bool;
    fn repair(&mut self);
}
pub trait IdAble {
    fn get_id(&self) -> Id;
}

#[derive(Serialize, Deserialize)]
pub struct Stack {
    pub id: Id,
    pub owner: Owner,
    pub name: String,
    pub position: AxialPosition,
    pub velocity: AxialDisplacement,
    pub fuel_tanks: HashMap<Id, FuelTank>,
    pub cargo_holds: HashMap<Id, CargoHold>,
    pub engines: HashMap<Id, Engine>,
    pub guns: HashMap<Id, Gun>,
    pub launch_clamps: HashMap<Id, LaunchClamp>,
    pub habitats: HashMap<Id, Habitat>,
    pub miners: HashMap<Id, Miner>,
    pub factories: HashMap<Id, Factory>,
    pub armour_plates: HashMap<Id, ArmourPlate>,
}
impl Stack {
    pub fn num_components(&self) -> usize {
        self.fuel_tanks.len()
            + self.cargo_holds.len()
            + self.engines.len()
            + self.guns.len()
            + self.launch_clamps.len()
            + self.habitats.len()
            + self.miners.len()
            + self.factories.len()
            + self.armour_plates.len()
    }

    pub fn get_random_component(&mut self) -> &mut dyn Component {
        let num_components = self.num_components();
        if num_components == 0 {
            panic!("should not have empty stack")
        }
        let mut selected_component_index = thread_rng().gen_range(0..num_components);

        if selected_component_index < self.cargo_holds.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.cargo_holds.len();
        }

        if selected_component_index < self.cargo_holds.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.cargo_holds.len();
        }

        if selected_component_index < self.engines.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.engines.len();
        }

        if selected_component_index < self.guns.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.guns.len();
        }

        if selected_component_index < self.launch_clamps.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.launch_clamps.len();
        }

        if selected_component_index < self.habitats.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.habitats.len();
        }

        if selected_component_index < self.miners.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.miners.len();
        }

        if selected_component_index < self.factories.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.factories.len();
        }

        if selected_component_index < self.armour_plates.len() {
            return self
                .cargo_holds
                .iter_mut()
                .nth(selected_component_index)
                .expect("index should be in range")
                .1;
        } else {
            selected_component_index -= self.armour_plates.len();
        }

        panic!(
            "selected component index was out of range - it ended up as {}",
            selected_component_index
        );
    }

    pub fn is_empty(&self) -> bool {
        self.fuel_tanks.is_empty()
            && self.cargo_holds.is_empty()
            && self.engines.is_empty()
            && self.guns.is_empty()
            && self.launch_clamps.is_empty()
            && self.habitats.is_empty()
            && self.miners.is_empty()
            && self.factories.is_empty()
            && self.armour_plates.is_empty()
    }

    pub fn remove_component(&mut self, component: Id) -> Result<(), ()> {
        if self.fuel_tanks.remove(&component).is_some()
            || self.cargo_holds.remove(&component).is_some()
            || self.engines.remove(&component).is_some()
            || self.guns.remove(&component).is_some()
            || self.launch_clamps.remove(&component).is_some()
            || self.habitats.remove(&component).is_some()
            || self.miners.remove(&component).is_some()
            || self.factories.remove(&component).is_some()
            || self.armour_plates.remove(&component).is_some()
        {
            Ok(())
        } else {
            Err(())
        }
    }

    /// try to insert as much cargo from the source list as possible, reporting leftover amount if failed
    /// TODO: how to prioritize leftovers
    pub fn insert_cargo(&mut self, cargo: &InventoryList) -> Result<(), InventoryList> {
        for (_, cargo_hold) in self.cargo_holds.iter_mut() {
            todo!();
        }
        todo!();
    }
}
impl Positionable for Stack {
    fn get_position(&self) -> &AxialPosition {
        &self.position
    }
}

#[derive(Serialize, Deserialize)]
pub struct FuelTank {
    id: Id,
    pub fuel: u64,
    pub damaged: bool,
}
impl Component for FuelTank {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for FuelTank {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct CargoHold {
    id: Id,
    inventory: InventoryList,
    damaged: bool,
}
impl Component for CargoHold {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for CargoHold {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Engine {
    pub id: Id,
    /// Has this engine overloaded? None = can't, Some(true) = ready to overload, Some(false) = not ready - needs overhaul
    pub overload_state: Option<bool>,
    pub damaged: bool,
}
impl Component for Engine {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for Engine {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Gun {
    pub id: Id,
    pub damaged: bool,
}
impl Component for Gun {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for Gun {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct LaunchClamp {
    id: Id,
    pub load: Option<OrdnanceType>,
    pub damaged: bool,
}
impl Component for LaunchClamp {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for LaunchClamp {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Habitat {
    id: Id,
    owner: Owner,
    damaged: bool,
}
impl Component for Habitat {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for Habitat {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Miner {
    id: Id,
    damaged: bool,
}
impl Component for Miner {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for Miner {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Factory {
    id: Id,
    damaged: bool,
}
impl Component for Factory {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for Factory {
    fn get_id(&self) -> Id {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct ArmourPlate {
    id: Id,
    damaged: bool,
}
impl Component for ArmourPlate {
    fn damage(&mut self) -> bool {
        if !self.damaged {
            self.damaged = true;
            false
        } else {
            true
        }
    }

    fn repair(&mut self) {
        self.damaged = false;
    }
}
impl IdAble for ArmourPlate {
    fn get_id(&self) -> Id {
        self.id
    }
}
