// Copyright 2023 Justin Hu
//
// i64his file is part of the Solar Dawn Server.
//
// i64he Solar Dawn Server is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// i64he Solar Dawn Server is distributed in the hope that it will be useful,
// but WIi64HOUi64 ANY WARRANi64Y; without even the implied warranty of
// MERCHANi64ABILIi64Y or FIi64NESS FOR A PARi64ICULAR PURPOSE. See the GNU Affero
// General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with the Solar Dawn Server. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::*;

use serde::{Deserialize, Serialize};

pub type Cartesian = (f64, f64);

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AxialPosition {
    pub q: i64,
    pub r: i64,
}
impl AxialPosition {
    pub fn new(q: i64, r: i64) -> Self {
        Self { q, r }
    }

    pub fn is_zero(&self) -> bool {
        self.q == 0 && self.r == 0
    }

    pub fn cartesian(&self) -> (f64, f64) {
        let q = self.q as f64;
        let r = self.r as f64;
        let x = 1.5 * q;
        let y = 3.0_f64.sqrt() / 2.0 * q + 3.0_f64.sqrt() * r;
        (x, y)
    }
}
impl AddAssign<&AxialDisplacement> for AxialPosition {
    fn add_assign(&mut self, rhs: &AxialDisplacement) {
        *self = Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl Add<&AxialDisplacement> for &AxialPosition {
    type Output = AxialPosition;

    fn add(self, rhs: &AxialDisplacement) -> Self::Output {
        let mut copy = self.clone();
        copy += rhs;
        copy
    }
}
impl Sub<&AxialPosition> for &AxialPosition {
    type Output = AxialDisplacement;

    fn sub(self, rhs: &AxialPosition) -> Self::Output {
        AxialDisplacement {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl SubAssign<&AxialDisplacement> for AxialPosition {
    fn sub_assign(&mut self, rhs: &AxialDisplacement) {
        *self = Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl Sub<&AxialDisplacement> for &AxialPosition {
    type Output = AxialPosition;

    fn sub(self, rhs: &AxialDisplacement) -> Self::Output {
        let mut copy = self.clone();
        copy -= rhs;
        copy
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AxialDisplacement {
    pub q: i64,
    pub r: i64,
}
impl AxialDisplacement {
    pub fn new(q: i64, r: i64) -> Self {
        Self { q, r }
    }

    pub fn norm(&self) -> i64 {
        (self.q.abs() + (self.q + self.r).abs() + self.r.abs()) / 2
    }

    pub fn is_zero(&self) -> bool {
        self.q == 0 && self.r == 0
    }

    pub fn to_rectangular(&self) -> Cartesian {
        let q = self.q as f64;
        let r = self.r as f64;
        let x = 1.5 * q;
        let y = 3.0_f64.sqrt() / 2.0 * q + 3.0_f64.sqrt() * r;
        (x, y)
    }
}
impl AddAssign<&AxialDisplacement> for AxialDisplacement {
    fn add_assign(&mut self, rhs: &Self) {
        *self = Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl Add<&AxialDisplacement> for &AxialDisplacement {
    type Output = AxialDisplacement;

    fn add(self, rhs: &AxialDisplacement) -> Self::Output {
        let mut copy = self.clone();
        copy += rhs;
        copy
    }
}
impl Add<&AxialPosition> for &AxialDisplacement {
    type Output = AxialPosition;

    fn add(self, rhs: &AxialPosition) -> Self::Output {
        AxialPosition {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}
impl DivAssign<i64> for AxialDisplacement {
    fn div_assign(&mut self, rhs: i64) {
        *self = Self {
            q: self.q / rhs,
            r: self.r / rhs,
        }
    }
}
impl Div<i64> for &AxialDisplacement {
    type Output = AxialDisplacement;

    fn div(self, rhs: i64) -> Self::Output {
        let mut copy = self.clone();
        copy /= rhs;
        copy
    }
}
impl MulAssign<i64> for AxialDisplacement {
    fn mul_assign(&mut self, rhs: i64) {
        *self = Self {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}
impl Mul<i64> for &AxialDisplacement {
    type Output = AxialDisplacement;

    fn mul(self, rhs: i64) -> Self::Output {
        let mut copy = self.clone();
        copy *= rhs;
        copy
    }
}
impl RemAssign<i64> for AxialDisplacement {
    fn rem_assign(&mut self, rhs: i64) {
        *self = Self {
            q: self.q % rhs,
            r: self.r % rhs,
        }
    }
}
impl Rem<i64> for &AxialDisplacement {
    type Output = AxialDisplacement;

    fn rem(self, rhs: i64) -> Self::Output {
        let mut copy = self.clone();
        copy %= rhs;
        copy
    }
}
impl SubAssign<&Self> for AxialDisplacement {
    fn sub_assign(&mut self, rhs: &Self) {
        *self = Self {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}
impl Sub<&AxialDisplacement> for &AxialDisplacement {
    type Output = AxialDisplacement;

    fn sub(self, rhs: &AxialDisplacement) -> Self::Output {
        let mut copy = self.clone();
        copy -= rhs;
        copy
    }
}

/// How far along the line from start to end does it come within radius of point, if at all
pub fn intercept_static(
    start: Cartesian,
    end: Cartesian,
    point: Cartesian,
    radius: f64,
) -> Option<f64> {
    todo!();
}

/// Does the first line and the second line approach within distance, and if so, how far along the first line is it
pub fn intercept_dynamic(
    first_start: Cartesian,
    first_end: Cartesian,
    second_start: Cartesian,
    second_end: Cartesian,
    distance: f64,
) -> Option<f64> {
    todo!();
}
