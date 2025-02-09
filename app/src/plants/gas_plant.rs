use serde::Serialize;

use super::PowerPlant;

/// Plant with no dynamic constraints.
pub struct GasPlant {
    settings: GasPlantSettings,
    setpoint: Option<isize>,
}
struct GasPlantSettings {
    energy_cost: isize,
}

impl GasPlant {
    pub fn new(energy_cost: isize) -> GasPlant {
        GasPlant {
            settings: GasPlantSettings { energy_cost },
            setpoint: None,
        }
    }

    fn cost(&self) -> isize {
        self.setpoint
            .map(|p| p * self.settings.energy_cost)
            .unwrap_or(0)
    }
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GasPlantPublicState {
    cost: isize,
    setpoint: isize,
}
impl PowerPlant for GasPlant {
    type Output = GasPlant;
    type PublicState = GasPlantPublicState;

    fn program_setpoint(&mut self, setpoint: isize) -> isize {
        self.setpoint = Some(setpoint);
        self.cost()
    }

    fn current_state(&self) -> Box<Self::PublicState> {
        Box::new(GasPlantPublicState {
            cost: self.cost(),
            setpoint: self.setpoint.unwrap_or(0),
        })
    }

    fn dispatch(self) -> (Box<GasPlant>, isize) {
        let cost = self.cost();
        (
            Box::new(GasPlant {
                settings: self.settings,
                setpoint: None,
            }),
            cost,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::plants::gas_plant::{GasPlant, GasPlantPublicState, PowerPlant};

    #[test]
    fn test_gas_plant() {
        let mut plant = GasPlant::new(47);

        assert_eq!(plant.program_setpoint(100), 47 * 100);
        assert_eq!(
            *plant.current_state(),
            GasPlantPublicState {
                cost: 47 * 100,
                setpoint: 100
            }
        );

        let (mut plant, dispatch_cost) = plant.dispatch();
        assert_eq!(dispatch_cost, 47 * 100);

        assert_eq!(plant.program_setpoint(0), 0);
    }
}
