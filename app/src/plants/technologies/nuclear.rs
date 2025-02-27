use serde::Serialize;

use crate::{
    forecast::ForecastLevel,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
};

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct NuclearPublicRepr {
    pub output: PlantOutput,
    pub max_setpoint: isize,
    pub previous_setpoint: isize,
    pub energy_cost: isize,
    pub locked: bool,
    pub touched: bool,
}

pub struct NuclearPlant {
    setpoint: isize,
    previous_setpoint: isize,
    max_setpoint: isize,
    touched: bool,
    locked: bool,
    energy_cost: isize,
}

impl NuclearPlant {
    pub fn new(max_setpoint: isize, energy_cost: isize) -> NuclearPlant {
        NuclearPlant {
            setpoint: 0,
            previous_setpoint: 0,
            max_setpoint,
            energy_cost,
            touched: false,
            locked: false,
        }
    }

    fn cost(&self) -> isize {
        self.setpoint * self.energy_cost
    }
}

impl PowerPlant for NuclearPlant {
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Nuclear(NuclearPublicRepr {
            output: PlantOutput {
                setpoint: self.setpoint,
                cost: self.cost(),
            },
            max_setpoint: self.max_setpoint,
            previous_setpoint: self.previous_setpoint,
            energy_cost: self.energy_cost,
            locked: self.locked,
            touched: self.touched,
        })
    }

    fn program_setpoint(&mut self, setpoint: isize) -> PlantOutput {
        if !self.locked {
            self.setpoint = setpoint.min(self.max_setpoint).max(0);
            self.touched = setpoint != self.previous_setpoint;
        }
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        self.locked = self.touched;
        self.touched = false;
        self.previous_setpoint = self.setpoint;
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
    }

    fn get_forecast(&self) -> Option<ForecastLevel> {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::plants::{
        technologies::nuclear::NuclearPlant, PlantOutput, PowerPlant, PowerPlantPublicRepr,
    };

    use super::NuclearPublicRepr;

    fn extract_state(plant: &NuclearPlant) -> NuclearPublicRepr {
        let PowerPlantPublicRepr::Nuclear(state) = plant.current_state() else {
            unreachable!("Should be a nuclear plant state");
        };
        state
    }

    #[test]
    fn nuclear_has_no_forecast() {
        let plant = NuclearPlant::new(1200, 35);

        assert!(plant.get_forecast().is_none());
    }

    #[test]
    fn nuclear_cannot_be_programmed_2_periods_in_a_row() {
        let mut plant = NuclearPlant::new(1200, 35);

        // First period, plant can be programmed
        let output_program = plant.program_setpoint(500);
        assert_eq!(output_program.setpoint, 500);
        assert!(extract_state(&plant).touched);
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 500);

        // Second period, plant is locked
        let second_output_program = plant.program_setpoint(700);
        assert_eq!(second_output_program.setpoint, 500);
        assert!(extract_state(&plant).locked);
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 500);

        // Third period, plant can be programmed again
        assert!(!extract_state(&plant).locked);
        assert!(!extract_state(&plant).touched);
        let third_output_program = plant.program_setpoint(600);
        assert_eq!(third_output_program.setpoint, 600);
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 600);
    }

    #[test]
    fn nuclear_programming_the_same_setpoint_as_previous_period_does_not_lock_the_plant() {
        let mut plant = NuclearPlant::new(1200, 35);

        // First period, program the plant and dispatch
        plant.program_setpoint(500);
        plant.dispatch();

        // Second period, plant is locked, dispatch
        plant.dispatch();

        // Third period, program a setpoint and go back to initial setpoint
        plant.program_setpoint(700);
        plant.program_setpoint(500);
        let output = plant.dispatch();
        assert_eq!(output.setpoint, 500);

        // Fourth period, plant can be programmed
        let output = plant.program_setpoint(600);
        assert_eq!(output.setpoint, 600);
        let output = plant.dispatch();
        assert_eq!(output.setpoint, 600);
    }

    #[test]
    fn nuclear_setpoint_limits() {
        let mut plant = NuclearPlant::new(1200, 35);

        assert_eq!(plant.program_setpoint(0).setpoint, 0);
        assert_eq!(plant.program_setpoint(-1).setpoint, 0);

        assert_eq!(plant.program_setpoint(1200).setpoint, 1200);
        assert_eq!(plant.program_setpoint(1201).setpoint, 1200);
    }

    #[test]
    fn nuclear_public_repr() {
        let mut plant = NuclearPlant::new(1200, 35);

        assert_eq!(
            extract_state(&plant),
            NuclearPublicRepr {
                output: PlantOutput {
                    cost: 0,
                    setpoint: 0
                },
                max_setpoint: 1200,
                previous_setpoint: 0,
                energy_cost: 35,
                locked: false,
                touched: false
            }
        );

        plant.program_setpoint(600);
        plant.dispatch();

        assert_eq!(
            extract_state(&plant),
            NuclearPublicRepr {
                output: PlantOutput {
                    cost: 600 * 35,
                    setpoint: 600
                },
                max_setpoint: 1200,
                previous_setpoint: 600,
                energy_cost: 35,
                locked: true,
                touched: false
            }
        );
    }
}
