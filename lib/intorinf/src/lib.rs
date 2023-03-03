//! A specialized, integer-or-inf type

#![no_std]

use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntOrInf {
    Int(i32),
    Inf,
}

macro_rules! from_ixx {
    ($ixx: ty) => {
        impl From<$ixx> for IntOrInf {
            fn from(value: $ixx) -> Self {
                if value < 0 {
                    Self::Inf
                } else {
                    Self::Int(value.into())
                }
            }
        }
    }
}

macro_rules! from_uxx {
    ($uxx: ty) => {
        impl From<$uxx> for IntOrInf {
            fn from(value: $uxx) -> Self {
                Self::Int(value.into())
            }
        }
    }
}

from_ixx!(i32);
from_ixx!(i16);
from_ixx!(i8);
from_uxx!(u16);
from_uxx!(u8);

impl PartialOrd for IntOrInf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntOrInf {
    fn cmp(&self, other: &Self) -> Ordering {
        match (*self, *other) {
            (Self::Int(v1), Self::Int(v2)) => v1.cmp(&v2),
            (Self::Int(_), Self::Inf) => Ordering::Less,
            (Self::Inf, Self::Int(_)) => Ordering::Greater,
            (Self::Inf, Self::Inf) => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod test {
    use super::IntOrInf;

    #[test]
    fn test_from_i32() {
        assert_eq!(IntOrInf::from(0), IntOrInf::Int(0));
        assert_eq!(IntOrInf::from(-1), IntOrInf::Inf);
        assert_eq!(IntOrInf::from(-10000), IntOrInf::Inf);
        assert_eq!(IntOrInf::from(1), IntOrInf::Int(1));
        assert_eq!(IntOrInf::from(100), IntOrInf::Int(100));
    }

    #[test]
    fn test_cmp() {
        let v1 = IntOrInf::Int(0);
        let v1_eq = IntOrInf::Int(0);
        let v2 = IntOrInf::Int(100);
        let v2_eq = IntOrInf::Int(100);
        let v3 = IntOrInf::Inf;
        let v3_eq = IntOrInf::Inf;

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
        assert!(v3 > v2);
        assert!(v2 > v1);
        assert!(v3 > v1);
        assert_eq!(v1, v1_eq);
        assert_eq!(v2, v2_eq);
        assert_eq!(v3, v3_eq);
    }
}
