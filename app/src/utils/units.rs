use std::ops::{Div, Mul};

use derive_more::{Add, Display, Div, From, Into, Mul, Neg, Sub, SubAssign};
use serde::{Deserialize, Serialize};

/// Represent a arbitrary unit of Power (like watt) in generator convetion, i.e.
/// power < 0 : means power is leaving the system (consumers, charge of storage)
/// power > 0 : means power is entering the system (power plant output, discharge of storage)
#[derive(
    Debug,
    From,
    Into,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Mul,
    Add,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Neg,
)]
pub struct Power(i32);

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

impl Power {
    pub fn abs(&self) -> Self {
        Self(self.0.abs())
    }
}

pub const NO_POWER: Power = Power(0);

/// Represent a arbitrary unit of Time (like hour/second).
#[derive(
    Debug, From, Into, PartialEq, PartialOrd, Ord, Eq, Mul, Add, Serialize, Deserialize, Clone, Copy,
)]
pub struct Time(i32);

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
pub struct Energy(i32);

pub const ZERO_ENERGY: Energy = Energy(0);

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
pub struct EnergyCost(i32);

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

/// Constant used when converting [`Power`] or [`Energy`] in generator convention into cost from an
/// utility perspective. Assuming a positive value of [`EnergyCost`] :
/// energy > 0 : means energy is entering the system (produced), so we have to pay for it (money < 0)
/// energy < 0 : means energy is leaving the system (consummed), so we are payed for it (money > 0)
pub const GENERATOR_CONVENTION_TO_MONEY: i32 = -1;

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
pub struct Money(i32);

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
