use serde::Serialize;

use super::{PlantOutput, PowerPlant, PowerPlantPublicRepr};

/// Plant with no dynamic constraints.
pub struct GasPlant {
    settings: GasPlantSettings,
    setpoint: isize,
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
            setpoint: 0,
        }
    }

    fn cost(&self) -> isize {
        self.setpoint * self.settings.energy_cost
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct GasPlantPublicRepr {
    pub settings: GasPlantSettings,
    pub output: PlantOutput,
}

impl PowerPlant for GasPlant {
    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput {
        self.setpoint = setpoint.max(0).min(self.settings.max_setpoint);
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
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
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

        // Program plant's setpoint
        assert_eq!(
            plant.program_setpoint(100),
            PlantOutput {
                setpoint: 100,
                cost: 100 * 47
            }
        );

        // Dispatch the plant, get previous setpoint
        assert_eq!(
            plant.dispatch(),
            PlantOutput {
                setpoint: 100,
                cost: 47 * 100
            }
        );
        // Setpoint should be kept after dispatching
        assert_eq!(plant.setpoint, 100);

        // No cost if no setpoint
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
