use serde::Serialize;

use crate::{
    forecast::Forecast,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
    utils::units::{Energy, Money, Power, TIMESTEP},
};

/// Store energy accros delivery periods
pub struct Battery {
    settings: BatterySettings,
    charge: Energy,
    setpoint: Power,
    history: Vec<PlantOutput>,
}
pub struct BatterySettings {
    max_charge: Energy,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct BatteryPublicRepr {
    pub max_charge: Energy,
    pub charge: Energy,
    pub output: PlantOutput,
}

impl Battery {
    pub fn new(max_charge: Energy, start_charge: Energy) -> Battery {
        Battery {
            settings: BatterySettings { max_charge },
            charge: start_charge,
            setpoint: Power::from(0),
            history: Vec::new(),
        }
    }

    fn cost(&self) -> Money {
        Money::from(0)
    }

    fn max_positive_power(&self) -> Power {
        self.charge / TIMESTEP
    }

    fn min_negative_power(&self) -> Power {
        -(self.settings.max_charge - self.charge) / TIMESTEP
    }
}

impl PowerPlant for Battery {
    /// For a battery in generator convention:
    /// - **positive** setpoint will **discharge** the battery (energy provided to the grid)
    /// - **negative** setpoint will **charge the** battery (energy taken from the grid)
    fn program_setpoint(&mut self, setpoint: Power) -> PlantOutput {
        self.setpoint = setpoint
            .min(self.max_positive_power())
            .max(self.min_negative_power());
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Battery(BatteryPublicRepr {
            max_charge: self.settings.max_charge,
            charge: self.charge,
            output: PlantOutput {
                setpoint: self.setpoint,
                cost: self.cost(),
            },
        })
    }

    fn dispatch(&mut self) -> PlantOutput {
        let setpoint = self.setpoint;
        let next_charge = self.charge - self.setpoint * TIMESTEP;
        let cost = self.cost();
        self.charge = next_charge;
        self.setpoint = Power::from(0);
        let output = PlantOutput { cost, setpoint };

        self.history.push(output.clone());

        output
    }

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        None
    }

    fn get_history(&self) -> Vec<PlantOutput> {
        self.history.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        plants::{PlantOutput, PowerPlant, technologies::battery::Battery},
        utils::units::{Energy, Money, Power},
    };

    #[test]
    fn test_battery() {
        let mut battery = Battery::new(Energy::from(1_000), Energy::from(0));

        // Initial history is empty
        assert!(battery.get_history().is_empty());

        // Basic charge of the battery (power is negative in generator convention)
        assert_eq!(battery.charge, Energy::from(0));
        assert_eq!(
            battery.program_setpoint(Power::from(-100)),
            PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(-100)
            }
        );

        let PlantOutput { cost, setpoint } = battery.dispatch();
        assert_eq!(cost, Money::from(0));
        assert_eq!(setpoint, Power::from(-100));
        assert_eq!(battery.charge, Energy::from(100));
        assert_eq!(
            battery.get_history(),
            vec![PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(-100)
            }]
        );

        // Basic discharge of the battery (power is positive in generator convention)
        battery.program_setpoint(Power::from(50));
        battery.dispatch();
        assert_eq!(battery.charge, Energy::from(50));
        assert_eq!(
            battery.get_history(),
            vec![
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(-100)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(50)
                }
            ]
        );

        // Too much power is clipped in regard to max available discharge
        // Current charge is 50, and max is 1000 -> 50 should be clipped
        assert_eq!(
            battery.program_setpoint(Power::from(-1000)),
            PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(-950)
            }
        );
        battery.dispatch();
        assert_eq!(battery.charge, Energy::from(1000));
        assert_eq!(
            battery.get_history(),
            vec![
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(-100)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(50)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(-950)
                }
            ]
        );

        // Too much power is clipped in regard to max available charge
        // Current charge is 1000, 100 should be clipped
        assert_eq!(
            battery.program_setpoint(Power::from(1100)),
            PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(1000)
            }
        );
        battery.dispatch();
        assert_eq!(battery.charge, Energy::from(0));
        assert_eq!(
            battery.get_history(),
            vec![
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(-100)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(50)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(-950)
                },
                PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(1000)
                }
            ]
        );
    }

    #[test]
    fn test_battery_has_no_forecast() {
        let battery = Battery::new(Energy::from(1000), Energy::from(0));
        assert!(battery.get_forecast().is_none());
    }
}
