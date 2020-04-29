// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2020, Douglas Creager.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License.  You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied.  See the License for the specific language governing permissions and
// limitations under the License.
// ------------------------------------------------------------------------------------------------

use num_rational::Ratio;

/// Each rational number represents _two_ continued fractions: one that ends with a 1, and one that
/// does not.  That means that we can't translate path vectors into continued fractions as-is — we
/// wouldn't be able to distinguish `[3,5,1]` from `[3,6]`, for example, since both paths would be
/// represented by the same rational number.  To get around this, we sneakily add 2 to every
/// element of a path vector as we translate it into a rational number, and subtract it back when
/// regenerating a path.  That ensures that our path vectors can use 0-based indexes, and that
/// we'll _never_ have any 1s in the continued fractions that we create.
const FUDGE: u64 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathIdentifier(Ratio<u64>);

impl std::iter::FromIterator<u64> for PathIdentifier {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u64>,
    {
        let mut ratio = Ratio::from_integer(0);
        for piece in iter.into_iter() {
            ratio = (ratio + piece + FUDGE).recip();
        }
        PathIdentifier(ratio.recip())
    }
}

impl std::str::FromStr for PathIdentifier {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('.').map(u64::from_str).rev().collect()
    }
}

impl From<Vec<u64>> for PathIdentifier {
    fn from(pieces: Vec<u64>) -> PathIdentifier {
        pieces.into_iter().rev().collect()
    }
}

impl PartialEq<(u64, u64)> for PathIdentifier {
    fn eq(&self, other: &(u64, u64)) -> bool {
        self.0 == Ratio::from(*other)
    }
}

#[derive(Clone, Debug)]
struct Euclidean {
    a: u64,
    b: u64,
    q: u64,
    r: u64,
}

impl PathIdentifier {
    fn euclidean(&self) -> Euclidean {
        Euclidean {
            a: 0,
            b: *self.0.numer(),
            q: 0,
            r: *self.0.denom(),
        }
    }
}

impl Euclidean {
    fn advance(&mut self) {
        assert!(self.r != 0);
        self.a = self.b;
        self.b = self.r;
        self.q = self.a / self.b;
        self.r = self.a % self.b;
    }
}

struct EuclideanIterator {
    euclidean: Option<Euclidean>,
}

impl PathIdentifier {
    fn euclidean_iter(&self) -> EuclideanIterator {
        EuclideanIterator {
            euclidean: Some(self.euclidean()),
        }
    }
}

impl Iterator for EuclideanIterator {
    type Item = Euclidean;

    fn next(&mut self) -> Option<Euclidean> {
        if let Some(euclidean) = self.euclidean.as_mut() {
            if euclidean.r == 0 {
                self.euclidean = None;
            } else {
                euclidean.advance();
            }
        }
        self.euclidean.clone()
    }
}

impl PathIdentifier {
    pub fn path(&self) -> impl Iterator<Item = u64> {
        self.euclidean_iter().map(|euclidean| euclidean.q - FUDGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_id(s: &str) -> PathIdentifier {
        s.parse().expect("Cannot parse identifier")
    }

    #[test]
    fn can_parse_paths() {
        assert_eq!(parse_id("3"), (5, 1));
        assert_eq!(parse_id("3.12"), (71, 14));
        assert_eq!(parse_id("3.12.5"), (502, 99));
        assert_eq!(parse_id("3.12.5.1"), (1577, 311));
        assert_eq!(parse_id("3.12.5.1.21"), (36773, 7252));
    }

    #[test]
    fn can_parse_path_vecs() {
        assert_eq!(PathIdentifier::from(vec![3]), (5, 1));
        assert_eq!(PathIdentifier::from(vec![3, 12]), (71, 14));
        assert_eq!(PathIdentifier::from(vec![3, 12, 5]), (502, 99));
        assert_eq!(PathIdentifier::from(vec![3, 12, 5, 1]), (1577, 311));
        assert_eq!(PathIdentifier::from(vec![3, 12, 5, 1, 21]), (36773, 7252));
    }

    fn generate_path(s: &str) -> Vec<u64> {
        parse_id(s).path().collect()
    }

    #[test]
    fn can_generate_paths() {
        assert_eq!(generate_path("3"), vec![3]);
        assert_eq!(generate_path("3.12"), vec![3, 12]);
        assert_eq!(generate_path("3.12.5"), vec![3, 12, 5]);
        assert_eq!(generate_path("3.12.5.1"), vec![3, 12, 5, 1]);
        assert_eq!(generate_path("3.12.5.1.21"), vec![3, 12, 5, 1, 21]);
    }
}
