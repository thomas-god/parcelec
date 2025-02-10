use serde::Serialize;

use super::{PowerPlant, PowerPlantPublicRepr};

/// Store energy accros delivery periods
pub struct Battery {
    settings: BatterySettings,
    charge: usize,
    setpoint: Option<isize>,
}
pub struct BatterySettings {
    max_charge: usize,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct BatteryPublicRepr {
    pub max_charge: usize,
    pub current_setpoint: isize,
    pub charge: usize,
}

impl Battery {
    pub fn new(max_charge: usize, start_charge: usize) -> Battery {
        Battery {
            settings: BatterySettings { max_charge },
            charge: start_charge,
            setpoint: None,
        }
    }

    fn cost(&self) -> isize {
        0
    }

    fn max_positive_power(&self) -> isize {
        isize::try_from(self.charge).unwrap_or(isize::MAX)
    }
    fn min_negative_power(&self) -> isize {
        -isize::try_from(self.settings.max_charge.saturating_sub(self.charge)).unwrap_or(isize::MAX)
    }
}
impl PowerPlant for Battery {
    /// For a battery in generator convention:
    /// - **positive** setpoint will **discharge** the battery (energy provided to the grid)
    /// - **negative** setpoint will **charge the** battery (energy taken from the grid)
    fn program_setpoint(&mut self, setpoint: isize) -> isize {
        let clipped_setpoint = setpoint
            .min(self.max_positive_power())
            .max(self.min_negative_power());
        self.setpoint = Some(clipped_setpoint);
        self.cost()
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Battery(BatteryPublicRepr {
            max_charge: self.settings.max_charge,
            current_setpoint: self.setpoint.unwrap_or(0),
            charge: self.charge,
        })
    }

    fn dispatch(&mut self) -> isize {
        let next_charge = usize::try_from(
            isize::try_from(self.charge)
                .unwrap_or(isize::MAX)
                .saturating_sub(self.setpoint.unwrap_or(0)),
        )
        .unwrap_or(0);
        let cost = self.cost();
        self.charge = next_charge;
        self.setpoint = None;
        cost
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::battery::{Battery, PowerPlant};

    #[test]
    fn test_battery() {
        let mut battery = Battery::new(1_000, 0);

        // Basic charge of the battery (power is negative in generator convention)
        assert_eq!(battery.charge, 0);
        assert_eq!(battery.program_setpoint(-100), 0);
        assert_eq!(battery.setpoint, Some(-100));

        let dispatch_cost = battery.dispatch();
        assert_eq!(dispatch_cost, 0);

        assert_eq!(battery.charge, 100);

        // Basic discharge of the battery (power is positive in generator convention)
        battery.program_setpoint(50);
        battery.dispatch();
        assert_eq!(battery.charge, 50);

        // Too much power is clipped in regard to max available discharge
        // Current charge is 50, and max is 1000 -> 50 should be clipped
        battery.program_setpoint(-1000);
        assert_eq!(battery.setpoint, Some(-950));
        battery.dispatch();
        assert_eq!(battery.charge, 1000);

        // Too much power is clipped in regard to max available charge
        // Current charge is 1000, 100 should be clipped
        battery.program_setpoint(1100);
        assert_eq!(battery.setpoint, Some(1000));
        battery.dispatch();
        assert_eq!(battery.charge, 0);
    }
}
