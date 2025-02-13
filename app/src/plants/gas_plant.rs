use serde::Serialize;

use super::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

/// Plant with no dynamic constraints.
pub struct GasPlant {
    settings: GasPlantSettings,
    setpoint: Option<isize>,
}
#[derive(Debug, Serialize, Clone, Copy)]
pub struct GasPlantSettings {
    energy_cost: isize,
    max_setpoint: isize,
}

impl GasPlant {
    pub fn new(energy_cost: isize, max_setpoint: isize) -> GasPlant {
        GasPlant {
            settings: GasPlantSettings {
                energy_cost,
                max_setpoint,
            },
            setpoint: None,
        }
    }

    fn cost(&self) -> isize {
        self.setpoint
            .map(|p| p * self.settings.energy_cost)
            .unwrap_or(0)
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct GasPlantPublicRepr {
    pub settings: GasPlantSettings,
    pub cost: isize,
    pub setpoint: isize,
}

impl PowerPlant for GasPlant {
    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput {
        self.setpoint = Some(setpoint.max(0).min(self.settings.max_setpoint));
        PlantOutput {
            setpoint: self.setpoint.unwrap_or(0),
            cost: self.cost(),
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::GasPlant(GasPlantPublicRepr {
            settings: self.settings,
            cost: self.cost(),
            setpoint: self.setpoint.unwrap_or(0),
        })
    }

    fn dispatch(&mut self) -> PlantOutput {
        let output = PlantOutput {
            setpoint: self.setpoint.unwrap_or(0),
            cost: self.cost(),
        };
        self.setpoint = None;
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::{
        gas_plant::{GasPlant, PowerPlant},
        PlantOutput,
    };

    #[test]
    fn test_gas_plant() {
        let mut plant = GasPlant::new(47, 1000);

        assert_eq!(
            plant.program_setpoint(100),
            PlantOutput {
                setpoint: 100,
                cost: 100 * 47
            }
        );

        assert_eq!(
            plant.dispatch(),
            PlantOutput {
                setpoint: 100,
                cost: 47 * 100
            }
        );

        assert_eq!(
            plant.program_setpoint(0),
            PlantOutput {
                setpoint: 0,
                cost: 0
            }
        );

        // Setpoint cannot be negative
        assert_eq!(
            plant.program_setpoint(-100),
            PlantOutput {
                setpoint: 0,
                cost: 0
            }
        );

        // Setpoint will be clipped if above P_max
        assert_eq!(
            plant.program_setpoint(1100),
            PlantOutput {
                setpoint: 1000,
                cost: 1000 * 47
            }
        );
    }
}
