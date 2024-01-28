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

use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::Mul,
};

use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::vec2::{intercept_dynamic, intercept_static, AxialPosition};

use self::{
    celestial::{AsteroidField, CelestialBody},
    stack::{Ordnance, Positionable, Stack},
};

use super::order::Order;

mod celestial;
mod stack;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Id(u64);
impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0.to_string()
    }
}

#[derive(Serialize, Deserialize)]
struct IdGenerator {
    next: u64,
}
impl IdGenerator {
    fn generate(&mut self) -> Id {
        let id = Id(self.next);
        self.next += 1;
        id
    }
}
impl Default for IdGenerator {
    fn default() -> Self {
        Self { next: 1 }
    }
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Owner(u8);
impl TryFrom<u8> for Owner {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > GameState::MAX_PLAYERS {
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

#[derive(Serialize, Deserialize, Eq, PartialEq)]
pub enum TurnPhase {
    Economic,
    Ordnance,
    Combat,
    Movement,
}

#[derive(Serialize, Deserialize)]
struct Turn {
    number: u64,
    phase: TurnPhase,
}
impl Turn {
    pub fn next(&mut self) {
        match self.phase {
            TurnPhase::Economic => {
                self.phase = TurnPhase::Ordnance;
            }
            TurnPhase::Ordnance => {
                self.phase = TurnPhase::Combat;
            }
            TurnPhase::Combat => {
                self.phase = TurnPhase::Movement;
            }
            TurnPhase::Movement => {
                self.phase = TurnPhase::Economic;
                self.number += 1;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InventoryList {
    ore: u64,
    materials: u64,
    ice: u64,
    fuel: u64,
    mines: u64,
    torpedoes: u64,
    nukes: u64,
}
impl InventoryList {
    pub fn ore(ore: u64) -> Self {
        Self {
            ore,
            ..Self::default()
        }
    }
    pub fn materials(materials: u64) -> Self {
        Self {
            materials,
            ..Self::default()
        }
    }
    pub fn ice(ice: u64) -> Self {
        Self {
            ice,
            ..Self::default()
        }
    }
    pub fn fuel(fuel: u64) -> Self {
        Self {
            fuel,
            ..Self::default()
        }
    }
    pub fn mines(mines: u64) -> Self {
        Self {
            mines,
            ..Self::default()
        }
    }
    pub fn torpedoes(torpedoes: u64) -> Self {
        Self {
            torpedoes,
            ..Self::default()
        }
    }
    pub fn nukes(nukes: u64) -> Self {
        Self {
            nukes,
            ..Self::default()
        }
    }
}
impl Mul<u64> for InventoryList {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            ore: self.ore * rhs,
            materials: self.materials * rhs,
            ice: self.ice * rhs,
            fuel: self.fuel * rhs,
            mines: self.mines * rhs,
            torpedoes: self.torpedoes * rhs,
            nukes: self.nukes * rhs,
        }
    }
}

pub enum VictoryState {
    None,
    MutualLoss,
    Winner(Owner),
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    /// maps between player id and username
    players: HashMap<Owner, Option<String>>,
    turn: Turn,
    id_generator: IdGenerator,
    stacks: HashMap<Id, Stack>,
    ordnance: HashMap<Id, Ordnance>,
    celestials: HashMap<Id, CelestialBody>,
    asteroids: HashMap<Id, AsteroidField>,
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
        let mut celestials = HashMap::new();

        // generate non-asteroid celestial bodies
        let sol = CelestialBody::new(
            &mut id_generator,
            AxialPosition::new(0, 0),
            "#ffff00".to_owned(),
            0.8,
        );
        celestials.insert(sol.id, sol);

        // setup Earth bases

        // generate asteroids
        let mut asteroids = HashMap::new();
        // TODO

        Ok(GameState {
            players: (0..num_players)
                .map(|id| {
                    (
                        id.try_into()
                            .expect("num_players should be no greater than MAX_NUM_PLAYERS"),
                        None,
                    )
                })
                .collect(),
            turn: Turn {
                number: 0,
                phase: TurnPhase::Economic,
            },
            id_generator,
            stacks: HashMap::default(),
            ordnance: HashMap::default(),
            celestials,
            asteroids,
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
            eprintln!("warning: stopping the server is strongly recommended");
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

    /// Returns None if game is full
    pub fn assign_player(&mut self, username: &str) -> Option<Owner> {
        // if this username is already assigned, repeat assignment
        for entry in self.players.iter() {
            if entry
                .1
                .as_ref()
                .is_some_and(|entry_username| entry_username == username)
            {
                return Some(*entry.0);
            }
        }

        // else assign to the first open id
        for entry in self.players.iter() {
            if entry.1.is_none() {
                let owner = *entry.0;
                self.players.insert(owner, Some(username.to_owned()));
                return Some(owner);
            }
        }

        // username not already assigned and no open ids - game is full
        None
    }

    pub fn serialize_for_player(&self, player: Owner) -> String {
        todo!();
    }

    fn get_stack_with_owner_mut(&mut self, id: Id, owner: Owner) -> Option<&mut Stack> {
        let stack = self.stacks.get_mut(&id)?;
        if stack.owner != owner {
            None
        } else {
            Some(stack)
        }
    }

    fn get_stack_with_owner(&self, id: Id, owner: Owner) -> Option<&Stack> {
        let stack = self.stacks.get(&id)?;
        if stack.owner != owner {
            None
        } else {
            Some(stack)
        }
    }

    fn owner_to_username(&self, owner: Owner) -> &str {
        self.players
            .get(&owner)
            .expect("owner of orders should be a known player")
            .as_ref()
            .expect("owner of orders should be a known player")
    }

    fn display_invalid_phase_warning(&self, owner: Owner) {
        eprintln!(
            "warning: wrong-phase order from {} - ignoring this order",
            self.owner_to_username(owner)
        );
    }

    fn process_economic_orders(&mut self, orders: &HashMap<Owner, Vec<Order>>) {
        let mut foreign_cargo_deltas: HashMap<Owner, HashMap<(Id, Id), InventoryList>> =
            HashMap::new();
        let mut repaired_habitats: HashSet<Id> = HashSet::new();

        // run orders
        for (owner, orders) in orders.iter() {
            let mut new_stacks: HashMap<u64, Id> = HashMap::new();

            for order in orders.iter() {
                match order {
                    Order::Production(order) => {
                        todo!();
                    }
                    Order::CargoTransfer(order) => {
                        todo!();
                    }
                    Order::StackTransfer(order) => {
                        todo!();
                    }
                    Order::Reload(order) => {
                        todo!();
                    }
                    Order::HabitatRepair(order) => {
                        // stack must be valid
                        // habitat must be in stack and have not repaired before
                        // repaired component must be valid and must be damaged
                        // cargo hold must have one material
                        todo!();
                    }
                    Order::FactoryRepair(order) => {
                        // factory stack must be valid and contain at least one factory
                        // repaired stack must be valid, and component must be damaged
                        // repaired stack and factory stack must be rendezvoused
                        // cargo hold must have one material
                        todo!();
                    }
                    Order::Abort(order) => {
                        // order requires valid, owned ordnance
                        if let Some(ordnance) = self.ordnance.get(&order.ordnance) {
                            if ordnance.owner != *owner {
                                eprintln!(
                                    "warning: invalid abort order from {} - invalid owner",
                                    self.owner_to_username(*owner)
                                );
                                continue;
                            }

                            self.ordnance
                                .remove(&order.ordnance)
                                .expect("previously seen ordnance should still be in map");
                        } else {
                            eprintln!(
                                "warning: invalid abort order from {} - invalid ordnance",
                                self.owner_to_username(*owner)
                            );
                            continue;
                        }
                    }
                    _ => {
                        self.display_invalid_phase_warning(*owner);
                        continue;
                    }
                }
            }
        }

        // apply foreign cargo deltas
        for (_, foreign_cargo_deltas) in foreign_cargo_deltas.iter() {
            for ((from_stack, to_stack), delta) in foreign_cargo_deltas.iter() {
                match self
                    .stacks
                    .get_mut(to_stack)
                    .expect("previously seen stack should still be in map") // TODO: not true - might have been disassembled
                    .insert_cargo(delta)
                {
                    Ok(_) => continue,
                    Err(remainder) => {
                        let _ = self
                            .stacks
                            .get_mut(from_stack)
                            .expect("previously seen stack should still be in map") // TODO: not true - might have been disassembled
                            .insert_cargo(&remainder);
                    }
                }
            }
        }
    }

    fn process_ordnance_orders(&mut self, orders: &HashMap<Owner, Vec<Order>>) {
        for (owner, orders) in orders.iter() {
            for order in orders.iter() {
                match order {
                    Order::Launch(order) => {
                        // order requires valid, owned, stack and extant, non-damaged launch clamp
                        if let Some(stack) = self.get_stack_with_owner_mut(order.stack, *owner) {
                            if let Some(clamp) = stack.launch_clamps.get_mut(&order.launch_clamp) {
                                if clamp.damaged {
                                    eprintln!("warning: invalid launch order from {} - damaged launch clamp", self.owner_to_username(*owner));
                                    continue;
                                }

                                match &clamp.load {
                                    Some(ordnance_type) => {
                                        if order.boost.norm()
                                            <= ordnance_type
                                                .max_boost()
                                                .try_into()
                                                .expect("max boost should never be overly large")
                                        {
                                            let stack_position = stack.position.clone();
                                            let stack_velocity = stack.velocity.clone();
                                            let ordnance_type = *ordnance_type;
                                            let ordnance = Ordnance::new(
                                                &mut self.id_generator,
                                                *owner,
                                                ordnance_type,
                                                stack_position,
                                                &stack_velocity + &order.boost,
                                            );
                                            self.ordnance.insert(ordnance.id, ordnance);
                                        } else {
                                            eprintln!("warning: invalid launch order from {} - too large of a launch boost", self.owner_to_username(*owner));
                                            continue;
                                        }
                                    }
                                    None => {
                                        eprintln!("warning: invalid launch order from {} - unloaded launch clamp", self.owner_to_username(*owner));
                                        continue;
                                    }
                                }
                            } else {
                                eprintln!(
                                    "warning: invalid launch order from {} - invalid launch clamp",
                                    self.owner_to_username(*owner)
                                );
                                continue;
                            }
                        } else {
                            eprintln!(
                                "warning: invalid launch order from {} - invalid launching stack",
                                self.owner_to_username(*owner)
                            );
                            continue;
                        }
                    }
                    _ => {
                        self.display_invalid_phase_warning(*owner);
                        continue;
                    }
                }
            }
        }
    }

    fn shot_hit_check<T: Positionable>(&self, shooter: &Stack, target: &T) -> bool {
        if self.celestials.iter().any(|(_, celestial)| {
            intercept_static(
                shooter.position.cartesian(),
                target.get_position().cartesian(),
                celestial.position.cartesian(),
                celestial.radius,
            )
            .is_some()
        }) {
            return false;
        }

        let range = (shooter.get_position() - target.get_position()).norm();
        let hit_chance = 0.5_f64.powi(range.try_into().expect("range shouldn't be too large"));
        thread_rng().gen_bool(hit_chance)
    }

    fn apply_damage(&mut self, stack: Id, amount: u64) {
        for _ in 0..amount {
            let stack = self
                .stacks
                .get_mut(&stack)
                .expect("given stack should still be in map");
            for _ in 0..amount {
                let component = stack.get_random_component();
                if component.damage() {
                    let id = component.get_id();
                    stack
                        .remove_component(id)
                        .expect("stack's random component should be part of the stack");
                }

                if stack.is_empty() {
                    let id = stack.id;
                    self.stacks
                        .remove(&id)
                        .expect("previously seen stack should still be in map");
                    break;
                }
            }
        }
    }

    fn process_combat_orders(&mut self, orders: &HashMap<Owner, Vec<Order>>) {
        let mut pending_damage: HashMap<Id, u64> = HashMap::new();
        let mut shot_guns: HashSet<Id> = HashSet::new();

        // generate pending damage values
        for (owner, orders) in orders.iter() {
            for order in orders.iter() {
                match order {
                    Order::Shoot(order) => {
                        // order requires valid, owned stack, valid target, line of sight, and extant, non-damaged gun
                        if let Some(shooter) = self.get_stack_with_owner(order.shooter, *owner) {
                            if let Some(gun) = shooter.guns.get(&order.gun) {
                                if let Some(target) = self.stacks.get(&order.target) {
                                    if gun.damaged {
                                        eprintln!(
                                            "warning: invalid shoot order from {} - damaged gun",
                                            self.owner_to_username(*owner)
                                        );
                                        continue;
                                    } else if !shot_guns.insert(gun.id) {
                                        eprintln!("warning: invalid shoot order from {} - gun already shot this turn", self.owner_to_username(*owner));
                                        continue;
                                    }

                                    if self.shot_hit_check(shooter, target) {
                                        *(pending_damage.entry(target.id).or_insert(0)) += 1;
                                    }
                                } else if let Some(target) = self.ordnance.get(&order.target) {
                                    if !shot_guns.insert(gun.id) {
                                        eprintln!("warning: invalid shoot order from {} - gun already shot this turn", self.owner_to_username(*owner));
                                        continue;
                                    }

                                    if self.shot_hit_check(shooter, target) {
                                        self.ordnance.remove(&order.target);
                                    }
                                } else {
                                    eprintln!(
                                        "warning: invalid shoot order from {} - invalid target",
                                        self.owner_to_username(*owner)
                                    );
                                }
                            } else {
                                eprintln!(
                                    "warning: invalid shoot order from {} - invalid gun",
                                    self.owner_to_username(*owner)
                                );
                                continue;
                            }
                        } else {
                            eprintln!(
                                "warning: invalid shoot order from {} - invalid shooting stack",
                                self.owner_to_username(*owner)
                            );
                            continue;
                        }
                    }
                    _ => {
                        self.display_invalid_phase_warning(*owner);
                        continue;
                    }
                }
            }
        }

        // apply the damage
        for (stack, amount) in pending_damage.iter() {
            self.apply_damage(*stack, *amount);
        }
    }

    const HIT_CHECK_EPSILON: f64 = 1e-9;

    fn process_movement_orders(&mut self, orders: &HashMap<Owner, Vec<Order>>) {
        let mut burned_engines: HashSet<Id> = HashSet::new();

        for (owner, orders) in orders.iter() {
            for order in orders.iter() {
                match order {
                    Order::Burn(order) => {
                        // order requires valid, owned stack and extant, non-damaged engine
                        // overloads require overload-capable and ready engine
                        if let Some(stack) = self.get_stack_with_owner_mut(order.stack, *owner) {
                            if let Some(engine) = stack.engines.get_mut(&order.engine) {
                                if engine.damaged {
                                    eprintln!(
                                        "warning: invalid burn order from {} - damaged engine",
                                        self.owner_to_username(*owner)
                                    );
                                    continue;
                                }

                                if let Some(fuel_tank) = stack.fuel_tanks.get_mut(&order.fuel_tank)
                                {
                                    if fuel_tank.damaged {
                                        eprintln!("warning: invalid burn order from {} - damaged fuel tank", self.owner_to_username(*owner));
                                        continue;
                                    }

                                    match order.direction.norm() {
                                        1 => {
                                            if fuel_tank.fuel < 1 {
                                                eprintln!("warning: invalid burn order from {} - out of fuel", self.owner_to_username(*owner));
                                                continue;
                                            }

                                            if !burned_engines.insert(engine.id) {
                                                eprintln!("warning: invalid burn order from {} - engine already burned this turn", self.owner_to_username(*owner));
                                                continue;
                                            }
                                            fuel_tank.fuel -= 1;
                                        }
                                        2 => {
                                            if fuel_tank.fuel < 2 {
                                                eprintln!("warning: invalid burn order from {} - out of fuel", self.owner_to_username(*owner));
                                                continue;
                                            }

                                            if engine.overload_state.unwrap_or(false) {
                                                eprintln!("warning: invalid burn order from {} - engine can't overload", self.owner_to_username(*owner));
                                                continue;
                                            }

                                            if !burned_engines.insert(engine.id) {
                                                eprintln!("warning: invalid burn order from {} - engine already burned this turn", self.owner_to_username(*owner));
                                                continue;
                                            }
                                            fuel_tank.fuel -= 2;
                                            engine.overload_state =
                                                engine.overload_state.map(|_| false);
                                        }
                                        _ => {
                                            eprintln!(
                                            "warning: invalid burn order from {} - invalid delta-v",
                                            self.owner_to_username(*owner)
                                        );
                                            continue;
                                        }
                                    }

                                    stack.velocity += &order.direction;
                                } else {
                                    eprintln!(
                                        "warning: invalid burn order from {} - invalid fuel tank",
                                        self.owner_to_username(*owner)
                                    );
                                    continue;
                                }
                            } else {
                                eprintln!(
                                    "warning: invalid shoot order from {} - invalid engine",
                                    self.owner_to_username(*owner)
                                );
                                continue;
                            }
                        } else {
                            eprintln!(
                                "warning: invalid burn order from {} - invalid burning stack",
                                self.owner_to_username(*owner)
                            );
                            continue;
                        }
                    }
                    _ => {
                        self.display_invalid_phase_warning(*owner);
                        continue;
                    }
                }
            }
        }

        // ordnance hit check
        let mut to_remove = Vec::new();
        let mut hit_records = Vec::new();
        for (ordnance_id, ordnance) in self.ordnance.iter() {
            let ordnance_start = ordnance.position.cartesian();
            let ordnance_end = (&ordnance.position + &ordnance.velocity).cartesian();

            // when does this hit a celestial body?
            let mut celestial_impact = None;
            for (_, celestial) in self.celestials.iter() {
                let candidate_impact = intercept_static(
                    ordnance_start,
                    ordnance_end,
                    celestial.position.cartesian(),
                    celestial.radius,
                );
                if candidate_impact.is_some_and(|candidate_impact| match celestial_impact {
                    Some(celestial_impact) => candidate_impact < celestial_impact,
                    None => true,
                }) {
                    celestial_impact = candidate_impact;
                }
            }

            // check to see if it hits a stack first
            let mut stack_hit_distance: Option<f64> = None;
            let mut stacks_hit = Vec::new();
            for (stack_id, stack) in self.stacks.iter() {
                // no friendly fire
                if ordnance.owner == stack.owner {
                    continue;
                }

                let candidate_hit_distance = intercept_dynamic(
                    ordnance_start,
                    ordnance_end,
                    stack.position.cartesian(),
                    (&stack.position + &stack.velocity).cartesian(),
                    1.0,
                );
                match candidate_hit_distance {
                    Some(candidate_hit_distance_value)
                        if celestial_impact.is_none()
                            || celestial_impact.is_some_and(|celestial_impact| {
                                candidate_hit_distance_value <= celestial_impact
                            }) =>
                    {
                        match stack_hit_distance {
                            Some(stack_hit_distance_value)
                                if (stack_hit_distance_value - candidate_hit_distance_value)
                                    .abs()
                                    < Self::HIT_CHECK_EPSILON =>
                            {
                                stacks_hit.push(*stack_id);
                            }
                            Some(stack_hit_distance_value)
                                if stack_hit_distance_value > candidate_hit_distance_value =>
                            {
                                stack_hit_distance = candidate_hit_distance;
                                stacks_hit = vec![*stack_id];
                            }
                            None => {
                                stack_hit_distance = candidate_hit_distance;
                                stacks_hit = vec![*stack_id];
                            }
                            Some(_) => continue,
                        }
                    }
                    _ => continue,
                }
            }

            // apply hits, if relevant
            if celestial_impact.is_some() || stack_hit_distance.is_some() {
                to_remove.push(*ordnance_id);
            }
            if let Some(hit) = stacks_hit.choose(&mut thread_rng()) {
                hit_records.push((*hit, ordnance.ordnance_type));
            }
        }
        // apply hits
        for id in to_remove.iter() {
            self.ordnance
                .remove(id)
                .expect("previously seen ordnance should still be in map");
        }
        for (hit, ordnance_type) in hit_records.iter() {
            // stack may have been previously destroyed - punt
            if !self.stacks.contains_key(hit) {
                continue;
            }

            match ordnance_type {
                stack::OrdnanceType::Mine | stack::OrdnanceType::Torpedo => self.apply_damage(
                    *hit,
                    (self
                        .stacks
                        .get(hit)
                        .expect("previously seen stack should still be in map")
                        .num_components() as f64
                        * ordnance_type.damage_fraction())
                    .round() as u64,
                ),
                stack::OrdnanceType::Nuke => {
                    self.stacks
                        .remove(hit)
                        .expect("previously seen stack should still be in map");
                }
            }
        }

        // tick movement and miners
        for (_, ordnance) in self.ordnance.iter_mut() {
            // note: celestial body impact check already done above
            ordnance.position += &ordnance.velocity;
        }
        let mut to_remove = Vec::new();
        for (id, stack) in self.stacks.iter_mut() {
            if self.celestials.iter().any(|(_, celestial)| {
                intercept_static(
                    stack.position.cartesian(),
                    (&stack.position + &stack.velocity).cartesian(),
                    celestial.position.cartesian(),
                    celestial.radius,
                )
                .is_some()
            }) {
                to_remove.push(*id);
                continue;
            }

            stack.position += &stack.velocity;

            // miner tick
            if stack.velocity.is_zero() && !stack.miners.is_empty() {
                if let Some((_, asteroids)) = self
                    .asteroids
                    .iter()
                    .find(|(_, asteroids)| asteroids.position == stack.position)
                {
                    let to_add: InventoryList = asteroids.resource.into();
                    // don't care about overflow
                    let _ = stack.insert_cargo(&(to_add * stack.miners.len() as u64));
                }
            }
        }
        for id in to_remove.iter() {
            self.stacks
                .remove(id)
                .expect("previously seen stack should still be in map");
        }
    }

    pub fn process_orders(&mut self, orders: &HashMap<Owner, Vec<Order>>) -> VictoryState {
        match self.turn.phase {
            TurnPhase::Economic => self.process_economic_orders(orders),
            TurnPhase::Ordnance => self.process_ordnance_orders(orders),
            TurnPhase::Combat => self.process_combat_orders(orders),
            TurnPhase::Movement => self.process_movement_orders(orders),
        }

        // check for victory
        if self.stacks.is_empty() {
            return VictoryState::MutualLoss;
        }

        for (owner, _) in self.players.iter() {
            if self.stacks.iter().all(|(_, stack)| stack.owner == *owner) {
                return VictoryState::Winner(*owner);
            }
        }

        self.turn.next();
        VictoryState::None
    }
}
