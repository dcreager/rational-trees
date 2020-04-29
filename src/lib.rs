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

/// Each rational number represents _two_ continued fractions: one that ends with a 1, and one that
/// does not.  That means that we can't translate path vectors into continued fractions as-is — we
/// wouldn't be able to distinguish `[3,5,1]` from `[3,6]`, for example, since both paths would be
/// represented by the same rational number.  To get around this, we sneakily add 2 to every
/// element of a path vector as we translate it into a rational number, and subtract it back when
/// regenerating a path.  That ensures that our path vectors can use 0-based indexes, and that
/// we'll _never_ have any 1s in the continued fractions that we create.
const FUDGE: u64 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathIdentifier(u64, u64, u64, u64);

impl PathIdentifier {
    pub fn root() -> PathIdentifier {
        PathIdentifier(1, 0, 0, 1)
    }

    pub fn is_root(&self) -> bool {
        self.1 == 0
    }

    fn from_path_element(element: u64) -> PathIdentifier {
        PathIdentifier(element, 1, 1, 0)
    }
}

impl std::ops::Mul for PathIdentifier {
    type Output = PathIdentifier;

    fn mul(self, other: PathIdentifier) -> PathIdentifier {
        // | s0 s1 |   | o0 o1 |   | s0o0 + s1o2  s0o1 + s1o3 |
        // | s2 s3 | x | o2 o3 | = | s2o0 + s3o2  s2o1 + s3o3 |
        PathIdentifier(
            self.0 * other.0 + self.1 * other.2,
            self.0 * other.1 + self.1 * other.3,
            self.2 * other.0 + self.3 * other.2,
            self.2 * other.1 + self.3 * other.3,
        )
    }
}

impl std::ops::MulAssign for PathIdentifier {
    fn mul_assign(&mut self, other: PathIdentifier) {
        let s0 = self.0;
        let s1 = self.1;
        let s2 = self.2;
        let s3 = self.3;
        self.0 = s0 * other.0 + s1 * other.2;
        self.1 = s0 * other.1 + s1 * other.3;
        self.2 = s2 * other.0 + s3 * other.2;
        self.3 = s2 * other.1 + s3 * other.3;
    }
}

impl std::iter::FromIterator<u64> for PathIdentifier {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u64>,
    {
        let mut id = PathIdentifier::root();
        for piece in iter.into_iter() {
            id *= PathIdentifier::from_path_element(piece + FUDGE);
        }
        id
    }
}

impl std::str::FromStr for PathIdentifier {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(PathIdentifier::root())
        } else {
            s.split('.').map(u64::from_str).collect()
        }
    }
}

impl From<Vec<u64>> for PathIdentifier {
    fn from(pieces: Vec<u64>) -> PathIdentifier {
        pieces.into_iter().collect()
    }
}

impl PartialEq<(u64, u64, u64, u64)> for PathIdentifier {
    fn eq(&self, other: &(u64, u64, u64, u64)) -> bool {
        *self == PathIdentifier(other.0, other.1, other.2, other.3)
    }
}

struct PathIterator {
    current: PathIdentifier,
}

impl PathIdentifier {
    fn path_iter(&self) -> PathIterator {
        PathIterator {
            current: self.clone(),
        }
    }
}

impl Iterator for PathIterator {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.current.is_root() {
            return None;
        }

        let result = self.current.0 / self.current.2;
        let s0 = self.current.0;
        let s1 = self.current.1;
        let s2 = self.current.2;
        let s3 = self.current.3;
        self.current.0 = s2;
        self.current.1 = s3;
        self.current.2 = s0 - s2 * result;
        self.current.3 = s1 - s3 * result;
        Some(result)
    }
}

impl PathIdentifier {
    pub fn path(&self) -> impl Iterator<Item = u64> {
        self.path_iter().map(|element| element - FUDGE)
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
        assert_eq!(parse_id(""), (1, 0, 0, 1));
        assert_eq!(parse_id("3"), (5, 1, 1, 0));
        assert_eq!(parse_id("3.12"), (71, 5, 14, 1));
        assert_eq!(parse_id("3.12.5"), (502, 71, 99, 14));
        assert_eq!(parse_id("3.12.5.1"), (1577, 502, 311, 99));
        assert_eq!(parse_id("3.12.5.1.21"), (36773, 1577, 7252, 311));
    }

    #[test]
    fn can_parse_path_vecs() {
        assert_eq!(PathIdentifier::from(vec![]), (1, 0, 0, 1));
        assert_eq!(PathIdentifier::from(vec![3]), (5, 1, 1, 0));
        assert_eq!(PathIdentifier::from(vec![3, 12]), (71, 5, 14, 1));
        assert_eq!(PathIdentifier::from(vec![3, 12, 5]), (502, 71, 99, 14));
        assert_eq!(
            PathIdentifier::from(vec![3, 12, 5, 1]),
            (1577, 502, 311, 99)
        );
        assert_eq!(
            PathIdentifier::from(vec![3, 12, 5, 1, 21]),
            (36773, 1577, 7252, 311)
        );
    }

    fn generate_path(s: &str) -> Vec<u64> {
        parse_id(s).path().collect()
    }

    #[test]
    fn can_generate_paths() {
        assert_eq!(generate_path(""), vec![]);
        assert_eq!(generate_path("3"), vec![3]);
        assert_eq!(generate_path("3.12"), vec![3, 12]);
        assert_eq!(generate_path("3.12.5"), vec![3, 12, 5]);
        assert_eq!(generate_path("3.12.5.1"), vec![3, 12, 5, 1]);
        assert_eq!(generate_path("3.12.5.1.21"), vec![3, 12, 5, 1, 21]);
    }
}
