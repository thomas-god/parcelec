use serde::Serialize;

use crate::{
    forecast::Forecast,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
    utils::units::{EnergyCost, Money, NO_POWER, Power, TIMESTEP},
};

/// Plant with no dynamic constraints.
pub struct GasPlant {
    settings: GasPlantSettings,
    setpoint: Power,
    history: Vec<PlantOutput>,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct GasPlantSettings {
    energy_cost: EnergyCost,
    max_setpoint: Power,
}

impl GasPlant {
    pub fn new(energy_cost: EnergyCost, max_setpoint: Power) -> GasPlant {
        GasPlant {
            settings: GasPlantSettings {
                energy_cost,
                max_setpoint,
            },
            setpoint: Power::from(0),
            history: Vec::new(),
        }
    }

    fn cost(&self) -> Money {
        self.setpoint * TIMESTEP * self.settings.energy_cost
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct GasPlantPublicRepr {
    pub settings: GasPlantSettings,
    pub output: PlantOutput,
}

impl PowerPlant for GasPlant {
    fn program_setpoint(&mut self, setpoint: Power) -> PlantOutput {
        self.setpoint = setpoint.max(NO_POWER).min(self.settings.max_setpoint);
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::GasPlant(GasPlantPublicRepr {
            settings: self.settings,
            output: PlantOutput {
                setpoint: self.setpoint,
                cost: self.cost(),
            },
        })
    }

    fn dispatch(&mut self) -> PlantOutput {
        let output = PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        };
        self.history.push(output);
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
        plants::{PlantOutput, PowerPlant, technologies::gas_plant::GasPlant},
        utils::units::{EnergyCost, Money, Power},
    };

    #[test]
    fn test_gas_plant() {
        let mut plant = GasPlant::new(EnergyCost::from(47), Power::from(1000));

        assert!(plant.get_history().is_empty());

        // Program plant's setpoint
        assert_eq!(
            plant.program_setpoint(Power::from(100)),
            PlantOutput {
                setpoint: Power::from(100),
                cost: Money::from(100 * 47)
            }
        );

        // Dispatch the plant, get previous setpoint
        assert_eq!(
            plant.dispatch(),
            PlantOutput {
                setpoint: Power::from(100),
                cost: Money::from(47 * 100)
            }
        );
        assert_eq!(
            plant.get_history(),
            vec![PlantOutput {
                cost: Money::from(100 * 47),
                setpoint: Power::from(100)
            }]
        );

        // Setpoint should be kept after dispatching
        assert_eq!(plant.setpoint, Power::from(100));

        // No cost if no setpoint
        assert_eq!(
            plant.program_setpoint(Power::from(0)),
            PlantOutput {
                setpoint: Power::from(0),
                cost: Money::from(0)
            }
        );

        // Setpoint cannot be negative
        assert_eq!(
            plant.program_setpoint(Power::from(-100)),
            PlantOutput {
                setpoint: Power::from(0),
                cost: Money::from(0)
            }
        );

        // Setpoint will be clipped if above P_max
        assert_eq!(
            plant.program_setpoint(Power::from(1100)),
            PlantOutput {
                setpoint: Power::from(1000),
                cost: Money::from(1000 * 47)
            }
        );
    }

    #[test]
    fn test_gas_plant_has_no_forecast() {
        let plant = GasPlant::new(EnergyCost::from(70), Power::from(1000));
        assert!(plant.get_forecast().is_none());
    }
}
