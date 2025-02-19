use serde::Serialize;

use crate::plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

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
    pub charge: usize,
    pub output: PlantOutput,
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
    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput {
        let clipped_setpoint = setpoint
            .min(self.max_positive_power())
            .max(self.min_negative_power());
        self.setpoint = Some(clipped_setpoint);
        PlantOutput {
            setpoint: self.setpoint.unwrap_or(0),
            cost: self.cost(),
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Battery(BatteryPublicRepr {
            max_charge: self.settings.max_charge,
            charge: self.charge,
            output: PlantOutput {
                setpoint: self.setpoint.unwrap_or(0),
                cost: 0,
            },
        })
    }

    fn dispatch(&mut self) -> PlantOutput {
        let setpoint = self.setpoint.unwrap_or(0);
        let next_charge = usize::try_from(
            isize::try_from(self.charge)
                .unwrap_or(isize::MAX)
                .saturating_sub(self.setpoint.unwrap_or(0)),
        )
        .unwrap_or(0);
        let cost = self.cost();
        self.charge = next_charge;
        self.setpoint = None;
        PlantOutput { cost, setpoint }
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::{technologies::battery::Battery, PlantOutput, PowerPlant};

    #[test]
    fn test_battery() {
        let mut battery = Battery::new(1_000, 0);

        // Basic charge of the battery (power is negative in generator convention)
        assert_eq!(battery.charge, 0);
        assert_eq!(
            battery.program_setpoint(-100),
            PlantOutput {
                cost: 0,
                setpoint: -100
            }
        );

        let PlantOutput { cost, setpoint } = battery.dispatch();
        assert_eq!(cost, 0);
        assert_eq!(setpoint, -100);
        assert_eq!(battery.charge, 100);

        // Basic discharge of the battery (power is positive in generator convention)
        battery.program_setpoint(50);
        battery.dispatch();
        assert_eq!(battery.charge, 50);

        // Too much power is clipped in regard to max available discharge
        // Current charge is 50, and max is 1000 -> 50 should be clipped
        assert_eq!(
            battery.program_setpoint(-1000),
            PlantOutput {
                cost: 0,
                setpoint: -950
            }
        );
        battery.dispatch();
        assert_eq!(battery.charge, 1000);

        // Too much power is clipped in regard to max available charge
        // Current charge is 1000, 100 should be clipped
        assert_eq!(
            battery.program_setpoint(1100),
            PlantOutput {
                cost: 0,
                setpoint: 1000
            }
        );
        battery.dispatch();
        assert_eq!(battery.charge, 0);
    }
}
