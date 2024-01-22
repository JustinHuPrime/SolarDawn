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

use std::ops::*;

use serde::{Deserialize, Serialize};

pub type Position = AxialPosition<i64>;
pub type Displacement = AxialDisplacement<i64>;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AxialPosition<T> {
    pub q: T,
    pub r: T,
}
impl<T> AxialPosition<T> {
    pub fn new(q: T, r: T) -> Self {
        Self { q, r }
    }
}
impl<T: Copy + Add<Output = T>> AddAssign<AxialDisplacement<T>> for AxialPosition<T> {
    fn add_assign(&mut self, rhs: AxialDisplacement<T>) {
        *self = Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl<T: Copy + Add<Output = T>> Add<AxialDisplacement<T>> for AxialPosition<T> {
    type Output = Self;

    fn add(self, rhs: AxialDisplacement<T>) -> Self::Output {
        let mut copy = self;
        copy += rhs;
        copy
    }
}
impl<T: Copy + Sub<Output = T>> Sub<Self> for AxialPosition<T> {
    type Output = AxialDisplacement<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        AxialDisplacement {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl<T: Copy + Sub<Output = T>> SubAssign<AxialDisplacement<T>> for AxialPosition<T> {
    fn sub_assign(&mut self, rhs: AxialDisplacement<T>) {
        *self = Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl<T: Copy + Sub<Output = T>> Sub<AxialDisplacement<T>> for AxialPosition<T> {
    type Output = Self;

    fn sub(self, rhs: AxialDisplacement<T>) -> Self::Output {
        let mut copy = self;
        copy -= rhs;
        copy
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AxialDisplacement<T> {
    pub q: T,
    pub r: T,
}
impl<T> AxialDisplacement<T> {
    pub fn new(q: T, r: T) -> Self {
        Self { q, r }
    }
}
impl<T: Copy + Add<Output = T>> AddAssign<AxialDisplacement<T>> for AxialDisplacement<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl<T: Copy + Add<Output = T>> Add<AxialDisplacement<T>> for AxialDisplacement<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut copy = self;
        copy += rhs;
        copy
    }
}
impl<T: Copy + Add<Output = T>> Add<AxialPosition<T>> for AxialDisplacement<T> {
    type Output = AxialPosition<T>;

    fn add(self, rhs: AxialPosition<T>) -> AxialPosition<T> {
        AxialPosition {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl<T: Copy + Div<Output = T>> DivAssign<T> for AxialDisplacement<T> {
    fn div_assign(&mut self, rhs: T) {
        *self = Self {
            q: self.q / rhs,
            r: self.r / rhs,
        }
    }
}
impl<T: Copy + Div<Output = T>> Div<T> for AxialDisplacement<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        let mut copy = self;
        copy /= rhs;
        copy
    }
}
impl<T: Copy + Mul<Output = T>> MulAssign<T> for AxialDisplacement<T> {
    fn mul_assign(&mut self, rhs: T) {
        *self = Self {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}
impl<T: Copy + Mul<Output = T>> Mul<T> for AxialDisplacement<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        let mut copy = self;
        copy *= rhs;
        copy
    }
}
impl<T: Copy + Rem<Output = T>> RemAssign<T> for AxialDisplacement<T> {
    fn rem_assign(&mut self, rhs: T) {
        *self = Self {
            q: self.q % rhs,
            r: self.r % rhs,
        }
    }
}
impl<T: Copy + Rem<Output = T>> Rem<T> for AxialDisplacement<T> {
    type Output = Self;

    fn rem(self, rhs: T) -> Self {
        let mut copy = self;
        copy %= rhs;
        copy
    }
}
impl<T: Copy + Sub<Output = T>> SubAssign<Self> for AxialDisplacement<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl<T: Copy + Sub<Output = T>> Sub<Self> for AxialDisplacement<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut copy = self;
        copy -= rhs;
        copy
    }
}
