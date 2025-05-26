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

use serde::{Deserialize, Serialize};
use solar_dawn_common::order::Order;
use solar_dawn_common::{GameState, PlayerId};

/// Game state plus server information
#[derive(Serialize, Deserialize)]
struct ServerState {
    game_state: GameState,
    orders: HashMap<PlayerId, Vec<Order>>,
}

impl ServerState {
    pub fn new() -> Self {
        todo!()
    }
}
