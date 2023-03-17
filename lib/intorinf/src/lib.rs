//! A specialized, integer-or-inf type

#![no_std]

use core::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign},
};

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

impl Display for IntOrInf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Int(i) => i.fmt(f),
            Self::Inf => write!(f, "inf"),
        }
    }
}

impl Add<Self> for IntOrInf {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => Self::Int(a + b),
            _ => Self::Inf,
        }
    }
}

impl AddAssign<Self> for IntOrInf {
    fn add_assign(&mut self, rhs: Self) {
        match (*self, rhs) {
            (Self::Int(a), Self::Int(b)) => *self = Self::Int(a + b),
            _ => *self = Self::Inf,
        }
    }
}

macro_rules! add_impl {
    ($t: ty) => {
        impl Add<$t> for IntOrInf {
            type Output = Self;

            fn add(self, other: $t) -> Self::Output {
                Add::<IntOrInf>::add(self, other.into())
            }
        }

        impl AddAssign<$t> for IntOrInf {
            fn add_assign(&mut self, rhs: $t) {
                match *self {
                    Self::Int(a) => *self = Self::Int(a + rhs as i32),
                    Self::Inf => {},
                }
            }
        }
    }
}

add_impl!(i32);
add_impl!(i16);
add_impl!(i8);
add_impl!(u16);
add_impl!(u8);

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
