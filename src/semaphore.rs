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

use std::sync::{Condvar, LockResult, Mutex, PoisonError};

pub struct Semaphore {
    value: Mutex<u64>,
    changed: Condvar,
}
impl Semaphore {
    pub fn new(value: u64) -> Self {
        Self {
            value: Mutex::new(value),
            changed: Condvar::new(),
        }
    }

    pub fn get(&self) -> LockResult<u64> {
        let mut value = self
            .value
            .lock()
            .map_err(|inner| PoisonError::new(*inner.into_inner()))?;
        Ok(*value)
    }

    pub fn up(&self) -> LockResult<()> {
        self.up_n(1)
    }

    pub fn up_n(&self, n: u64) -> LockResult<()> {
        if n == 0 {
            return Ok(());
        }

        let mut value = self.value.lock().map_err(|_| PoisonError::new(()))?;
        *value += n;
        if n > 1 {
            self.changed.notify_all();
        } else {
            self.changed.notify_one();
        }
        Ok(())
    }

    pub fn down(&self) -> LockResult<()> {
        self.down_n(1)
    }

    pub fn down_n(&self, n: u64) -> LockResult<()> {
        if n == 0 {
            return Ok(());
        }

        let mut value = self.value.lock().map_err(|_| PoisonError::new(()))?;
        while *value < n {
            value = self.changed.wait(value).map_err(|_| PoisonError::new(()))?;
        }
        *value -= n;
        Ok(())
    }
}
