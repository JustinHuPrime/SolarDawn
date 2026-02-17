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

use std::collections::{BTreeMap, HashMap};
use std::fs::read;
use std::marker::PhantomData;
use std::path::Path;

use anyhow::{Context, Result};
use num_traits::PrimInt;
use rand::{RngExt, SeedableRng, rng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use serde_cbor::from_slice;
use solar_dawn_common::{
    GameState, GameStateInitializer, PlayerId,
    celestial::CelestialId,
    order::Order,
    stack::{ModuleId, StackId},
};

/// Game state plus server information
#[derive(Serialize, Deserialize)]
pub struct GameServerState {
    pub game_state: GameState,
    #[serde(skip)]
    pub orders: HashMap<PlayerId, Vec<Order>>,
    pub celestial_id_generator: IdGenerator<CelestialId, u32>,
    pub stack_id_generator: IdGenerator<StackId, u32>,
    pub module_id_generator: IdGenerator<ModuleId, u32>,
    #[serde(skip, default = "GameServerState::new_rng")]
    pub rng: Pcg64,
}
impl GameServerState {
    pub fn new(players: BTreeMap<PlayerId, String>, initializer: GameStateInitializer) -> Self {
        let mut celestial_id_generator = IdGenerator::default();
        let mut stack_id_generator = IdGenerator::default();
        let mut module_id_generator = IdGenerator::default();
        let mut rng = Self::new_rng();
        Self {
            game_state: initializer(
                players,
                &mut celestial_id_generator,
                &mut stack_id_generator,
                &mut module_id_generator,
                &mut rng,
            ),
            orders: Default::default(),
            celestial_id_generator,
            stack_id_generator,
            module_id_generator,
            rng,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let file =
            read(path).with_context(|| format!("while reading {}", path.to_string_lossy()))?;

        from_slice(&file).with_context(|| format!("while deserializing {}", path.to_string_lossy()))
    }

    fn new_rng() -> Pcg64 {
        Pcg64::from_seed(rng().random())
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdGenerator<T: From<U>, U: PrimInt> {
    next: U,
    _t: PhantomData<T>,
}
impl<T: From<U>, U: PrimInt> Default for IdGenerator<T, U> {
    fn default() -> Self {
        Self {
            next: U::one(),
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
