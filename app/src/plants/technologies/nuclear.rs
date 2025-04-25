use serde::Serialize;

use crate::{
    forecast::Forecast,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
    utils::units::{EnergyCost, Money, NO_POWER, Power, TIMESTEP},
};

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct NuclearPublicRepr {
    pub output: PlantOutput,
    pub max_setpoint: Power,
    pub previous_setpoint: Power,
    pub energy_cost: EnergyCost,
    pub locked: bool,
    pub touched: bool,
}

pub struct NuclearPlant {
    setpoint: Power,
    previous_setpoint: Power,
    max_setpoint: Power,
    touched: bool,
    locked: bool,
    energy_cost: EnergyCost,
}

impl NuclearPlant {
    pub fn new(max_setpoint: Power, energy_cost: EnergyCost) -> NuclearPlant {
        NuclearPlant {
            setpoint: Power::from(0),
            previous_setpoint: Power::from(0),
            max_setpoint,
            energy_cost,
            touched: false,
            locked: false,
        }
    }

    fn cost(&self) -> Money {
        self.setpoint * TIMESTEP * self.energy_cost
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

    fn program_setpoint(&mut self, setpoint: Power) -> PlantOutput {
        if !self.locked {
            self.setpoint = setpoint.min(self.max_setpoint).max(NO_POWER);
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

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        None
    }
}

#[cfg(test)]
mod test {

    use crate::{
        plants::{
            PlantOutput, PowerPlant, PowerPlantPublicRepr, technologies::nuclear::NuclearPlant,
        },
        utils::units::{EnergyCost, Money, Power},
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
        let plant = NuclearPlant::new(Power::from(1200), EnergyCost::from(35));

        assert!(plant.get_forecast().is_none());
    }

    #[test]
    fn nuclear_cannot_be_programmed_2_periods_in_a_row() {
        let mut plant = NuclearPlant::new(Power::from(1200), EnergyCost::from(35));

        // First period, plant can be programmed
        let output_program = plant.program_setpoint(500.into());
        assert_eq!(output_program.setpoint, 500.into());
        assert!(extract_state(&plant).touched);
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 500.into());

        // Second period, plant is locked
        let second_output_program = plant.program_setpoint(700.into());
        assert_eq!(second_output_program.setpoint, 500.into());
        assert!(extract_state(&plant).locked);
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 500.into());

        // Third period, plant can be programmed again
        assert!(!extract_state(&plant).locked);
        assert!(!extract_state(&plant).touched);
        let third_output_program = plant.program_setpoint(600.into());
        assert_eq!(third_output_program.setpoint, 600.into());
        let output_dispatch = plant.dispatch();
        assert_eq!(output_dispatch.setpoint, 600.into());
    }

    #[test]
    fn nuclear_programming_the_same_setpoint_as_previous_period_does_not_lock_the_plant() {
        let mut plant = NuclearPlant::new(Power::from(1200), EnergyCost::from(35));

        // First period, program the plant and dispatch
        plant.program_setpoint(500.into());
        plant.dispatch();

        // Second period, plant is locked, dispatch
        plant.dispatch();

        // Third period, program a setpoint and go back to initial setpoint
        plant.program_setpoint(700.into());
        plant.program_setpoint(500.into());
        let output = plant.dispatch();
        assert_eq!(output.setpoint, 500.into());

        // Fourth period, plant can be programmed
        let output = plant.program_setpoint(600.into());
        assert_eq!(output.setpoint, 600.into());
        let output = plant.dispatch();
        assert_eq!(output.setpoint, 600.into());
    }

    #[test]
    fn nuclear_setpoint_limits() {
        let mut plant = NuclearPlant::new(Power::from(1200), EnergyCost::from(35));

        assert_eq!(plant.program_setpoint(0.into()).setpoint, 0.into());
        assert_eq!(plant.program_setpoint((-1).into()).setpoint, 0.into());

        assert_eq!(plant.program_setpoint(1200.into()).setpoint, 1200.into());
        assert_eq!(plant.program_setpoint(1201.into()).setpoint, 1200.into());
    }

    #[test]
    fn nuclear_public_repr() {
        let mut plant = NuclearPlant::new(Power::from(1200), EnergyCost::from(35));

        assert_eq!(
            extract_state(&plant),
            NuclearPublicRepr {
                output: PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(0)
                },
                max_setpoint: Power::from(1200),
                previous_setpoint: Power::from(0),
                energy_cost: EnergyCost::from(35),
                locked: false,
                touched: false
            }
        );

        plant.program_setpoint(600.into());
        plant.dispatch();

        assert_eq!(
            extract_state(&plant),
            NuclearPublicRepr {
                output: PlantOutput {
                    cost: Money::from(600 * 35),
                    setpoint: Power::from(600)
                },
                max_setpoint: Power::from(1200),
                previous_setpoint: Power::from(600),
                energy_cost: EnergyCost::from(35),
                locked: true,
                touched: false
            }
        );
    }
}
