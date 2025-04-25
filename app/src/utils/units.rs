use std::ops::{Div, Mul};

use derive_more::{Add, Display, Div, From, Into, Mul, Neg, Sub, SubAssign};
use serde::{Deserialize, Serialize};

/// Represent a arbitrary unit of Power (like watt).
#[derive(
    Debug, From, Into, PartialEq, PartialOrd, Ord, Eq, Mul, Add, Serialize, Deserialize, Clone, Copy,
)]
pub struct Power(isize);

impl Mul<Time> for Power {
    type Output = Energy;

    fn mul(self, rhs: Time) -> Self::Output {
        Energy(self.0 * rhs.0)
    }
}

impl Default for Power {
    fn default() -> Self {
        NO_POWER
    }
}

pub const NO_POWER: Power = Power(0);

/// Represent a arbitrary unit of Time (like hour/second).
#[derive(
    Debug, From, Into, PartialEq, PartialOrd, Ord, Eq, Mul, Add, Serialize, Deserialize, Clone, Copy,
)]
pub struct Time(isize);

pub const TIMESTEP: Time = Time(1);

/// Represent a arbitrary unit of Energy (like joule).
#[derive(
    Debug,
    From,
    Into,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Mul,
    Neg,
    Add,
    Sub,
    SubAssign,
    Serialize,
    Deserialize,
    Clone,
    Copy,
)]
pub struct Energy(isize);

impl Div<Time> for Energy {
    type Output = Power;

    fn div(self, rhs: Time) -> Self::Output {
        Power(self.0 / rhs.0)
    }
}

/// Represent an energy cost in arbitrary unit, i.e. how much cost a unit of energy (like €/joule).
#[derive(
    Debug,
    From,
    Into,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Mul,
    Div,
    Add,
    Neg,
    Serialize,
    Deserialize,
    Clone,
    Copy,
)]
pub struct EnergyCost(isize);

impl Mul<EnergyCost> for Energy {
    type Output = Money;

    fn mul(self, rhs: EnergyCost) -> Self::Output {
        Money(self.0 * rhs.0)
    }
}

impl Mul<Energy> for EnergyCost {
    type Output = Money;

    fn mul(self, rhs: Energy) -> Self::Output {
        Money(self.0 * rhs.0)
    }
}

/// Represent an arbitrary unit of money (like €).
#[derive(
    Debug,
    Display,
    From,
    Into,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Mul,
    Div,
    Add,
    Sub,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Default,
)]
pub struct Money(isize);

#[cfg(test)]
mod test {

    use super::{Energy, EnergyCost, Money, Power, Time};

    #[test]
    fn test_multiply_power_by_time_into_energy() {
        assert_eq!(Power(10) * Time(1), Energy(10));
        assert_eq!(Power(-10) * Time(1), Energy(-10));
    }

    #[test]
    fn test_multiply_energy_by_energy_cost_into_money() {
        assert_eq!(Energy(10) * EnergyCost(10), Money(100));
        assert_eq!(Energy(-10) * EnergyCost(10), Money(-100));
        assert_eq!(Energy(10) * EnergyCost(-10), Money(-100));
        assert_eq!(Energy(-10) * EnergyCost(-10), Money(100));
    }
}
