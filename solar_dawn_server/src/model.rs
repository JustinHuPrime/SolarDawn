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

use std::collections::HashMap;
use std::marker::PhantomData;

use anyhow::{bail, Result};
use num_traits::PrimInt;
use rand::{rng, Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use solar_dawn_common::order::Order;
use solar_dawn_common::stack::{ModuleId, StackId};
use solar_dawn_common::{
    celestial::{Celestial, CelestialId},
    GameState, PlayerId,
};

/// Game state plus server information
#[derive(Serialize, Deserialize)]
pub struct GameServerState {
    game_state: GameState,
    #[serde(skip)]
    orders: HashMap<PlayerId, Vec<Order>>,
    celestial_id_generator: IdGenerator<CelestialId, u8>,
    stack_id_generator: IdGenerator<StackId, u32>,
    module_id_generator: IdGenerator<ModuleId, u32>,
    #[serde(skip, default = "GameServerState::new_rng")]
    rng: Pcg64,
}
impl GameServerState {
    pub fn new(players: HashMap<PlayerId, String>, map: &str) -> Result<Self> {
        let mut celestial_id_generator = IdGenerator::default();
        let mut stack_id_generator = IdGenerator::default();
        let mut module_id_generator = IdGenerator::default();
        let (celestials, earth_id) = match map {
            "balanced" => Celestial::solar_system_balanced_positions(&mut celestial_id_generator),
            _ => {
                bail!("map name {} not recognized", map);
            }
        };
        Ok(Self {
            game_state: GameState::new(
                players,
                celestials,
                earth_id,
                &mut stack_id_generator,
                &mut module_id_generator,
            ),
            orders: Default::default(),
            celestial_id_generator,
            stack_id_generator,
            module_id_generator,
            rng: Self::new_rng(),
        })
    }

    fn new_rng() -> Pcg64 {
        Pcg64::from_seed(rng().random())
    }
}

#[derive(Serialize, Deserialize)]
struct IdGenerator<T: From<U>, U: PrimInt> {
    next: U,
    _t: PhantomData<T>,
}
impl<T: From<U>, U: PrimInt> Default for IdGenerator<T, U> {
    fn default() -> Self {
        Self {
            next: U::zero(),
            _t: PhantomData,
        }
    }
}
impl<T: From<U>, U: PrimInt> Iterator for IdGenerator<T, U> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let result: Option<T> = Some(self.next.into());
        self.next = self.next + U::one();
        result
    }
}
