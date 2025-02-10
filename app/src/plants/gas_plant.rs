use serde::Serialize;

use super::{PowerPlant, PowerPlantPublicRepr};

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
    fn program_setpoint(&mut self, setpoint: isize) -> isize {
        self.setpoint = Some(setpoint);
        self.cost()
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::GasPlant(GasPlantPublicRepr {
            settings: self.settings,
            cost: self.cost(),
            setpoint: self.setpoint.unwrap_or(0),
        })
    }

    fn dispatch(&mut self) -> isize {
        let cost = self.cost();
        self.setpoint = None;
        cost
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::gas_plant::{GasPlant, PowerPlant};

    #[test]
    fn test_gas_plant() {
        let mut plant = GasPlant::new(47, 1000);

        assert_eq!(plant.program_setpoint(100), 47 * 100);

        let dispatch_cost = plant.dispatch();
        assert_eq!(dispatch_cost, 47 * 100);

        assert_eq!(plant.program_setpoint(0), 0);
    }
}
