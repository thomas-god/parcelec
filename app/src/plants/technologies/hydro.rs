use serde::Serialize;

use crate::{
    forecast::Forecast,
    plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr},
    utils::units::{Energy, Money, NO_POWER, Power, TIMESTEP},
};

/// Reservoir hydro plant.
///
/// It holds a finite `stock` of energy (MWh) that can only be drained to produce power. Unlike a
/// battery, it cannot be recharged by consuming energy from the grid: this model is
/// **drain-only**. The reservoir starts the game full and there is no natural inflow, so its
/// stock can only decrease over time.
///
/// Power is limited by two independent settings: the reservoir `max_stock` (MWh) bounds the total
/// energy available, and `max_setpoint` (MW) represents the turbine's maximum output power.
pub struct HydroPlant {
    settings: HydroSettings,
    stock: Energy,
    setpoint: Power,
    history: Vec<PlantOutput>,
}

#[derive(Debug, Clone, Copy)]
pub struct HydroSettings {
    max_stock: Energy,
    max_setpoint: Power,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct HydroPublicRepr {
    pub max_stock: Energy,
    pub max_setpoint: Power,
    pub stock: Energy,
    pub output: PlantOutput,
}

impl HydroPlant {
    /// Create a hydro plant that starts with a full reservoir.
    pub fn new(max_stock: Energy, max_setpoint: Power) -> HydroPlant {
        HydroPlant {
            settings: HydroSettings {
                max_stock,
                max_setpoint,
            },
            stock: max_stock,
            setpoint: Power::from(0),
            history: Vec::new(),
        }
    }

    fn cost(&self) -> Money {
        // Generating is free: finding the opportunity cost of the stored water is the player's job.
        Money::from(0)
    }

    fn max_remaining_stock_power(&self) -> Power {
        self.stock / TIMESTEP
    }
}

impl PowerPlant for HydroPlant {
    /// For a hydro plant in generator convention:
    /// - **positive** setpoint will **drain** the reservoir (energy provided to the grid)
    /// - **negative** setpoints are not allowed, as the reservoir cannot be refilled from the grid
    fn program_setpoint(&mut self, setpoint: Power) -> PlantOutput {
        self.setpoint = setpoint
            .max(NO_POWER)
            .min(self.settings.max_setpoint)
            .min(self.max_remaining_stock_power());
        PlantOutput {
            setpoint: self.setpoint,
            cost: self.cost(),
        }
    }

    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::Hydro(HydroPublicRepr {
            max_stock: self.settings.max_stock,
            max_setpoint: self.settings.max_setpoint,
            stock: self.stock,
            output: PlantOutput {
                setpoint: self.setpoint,
                cost: self.cost(),
            },
        })
    }

    fn dispatch(&mut self) -> PlantOutput {
        let setpoint = self.setpoint;
        self.stock = (self.stock - setpoint * TIMESTEP).max(Energy::from(0));
        let cost = self.cost();
        self.setpoint = Power::from(0);
        let output = PlantOutput { cost, setpoint };

        self.history.push(output);

        output
    }

    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        None
    }

    fn get_history(&self) -> Vec<PlantOutput> {
        self.history.clone()
    }

    fn category(&self) -> crate::plants::PlantCategory {
        crate::plants::PlantCategory::Hydro
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        plants::{PlantOutput, PowerPlant, PowerPlantPublicRepr, technologies::hydro::HydroPlant},
        utils::units::{Energy, Money, NO_POWER, Power},
    };

    fn extract_state(plant: &HydroPlant) -> super::HydroPublicRepr {
        let PowerPlantPublicRepr::Hydro(state) = plant.current_state() else {
            unreachable!("Should be a hydro plant state");
        };
        state
    }

    #[test]
    fn test_hydro_starts_full() {
        let plant = HydroPlant::new(Energy::from(1000), Power::from(100));

        assert_eq!(plant.stock, Energy::from(1000));
        assert_eq!(
            extract_state(&plant).stock,
            Energy::from(1000),
            "Public representation should expose the full reservoir"
        );
    }

    #[test]
    fn test_hydro_drain() {
        let mut plant = HydroPlant::new(Energy::from(1000), Power::from(100));

        assert!(plant.get_history().is_empty());

        // Drain the reservoir (power is positive in generator convention)
        assert_eq!(
            plant.program_setpoint(Power::from(100)),
            PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(100)
            }
        );

        let PlantOutput { cost, setpoint } = plant.dispatch();
        assert_eq!(cost, Money::from(0));
        assert_eq!(setpoint, Power::from(100));
        assert_eq!(plant.stock, Energy::from(900));
        assert_eq!(
            plant.get_history(),
            vec![PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(100)
            }]
        );
    }

    #[test]
    fn test_hydro_cannot_be_recharged() {
        let mut plant = HydroPlant::new(Energy::from(1000), Power::from(100));

        // A negative setpoint is clipped to zero: no grid recharge
        assert_eq!(
            plant.program_setpoint(Power::from(-100)),
            PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(0)
            }
        );

        plant.dispatch();
        assert_eq!(plant.stock, Energy::from(1000));
        assert_eq!(
            plant.get_history(),
            vec![PlantOutput {
                cost: Money::from(0),
                setpoint: Power::from(0)
            }]
        );
    }

    #[test]
    fn test_hydro_setpoint_limited_by_turbine_and_stock() {
        let mut plant = HydroPlant::new(Energy::from(50), Power::from(100));

        // Stock limit is tighter than the turbine limit here
        assert_eq!(
            plant.program_setpoint(Power::from(1000)).setpoint,
            Power::from(50)
        );

        // Stock limit: cannot drain more than what is stored
        plant.dispatch();
        assert_eq!(plant.stock, Energy::from(0));
        assert_eq!(plant.program_setpoint(Power::from(10)).setpoint, NO_POWER);
    }

    #[test]
    fn test_hydro_never_drains_below_zero() {
        let mut plant = HydroPlant::new(Energy::from(30), Power::from(30));

        // Program the stock in one shot, then try to keep draining
        plant.program_setpoint(Power::from(30));
        plant.dispatch();
        assert_eq!(plant.stock, Energy::from(0));

        plant.program_setpoint(Power::from(30));
        plant.dispatch();
        assert_eq!(plant.stock, Energy::from(0));
    }

    #[test]
    fn test_hydro_has_no_forecast() {
        let plant = HydroPlant::new(Energy::from(1000), Power::from(100));
        assert!(plant.get_forecast().is_none());
    }

    #[test]
    fn test_hydro_public_repr() {
        let mut plant = HydroPlant::new(Energy::from(1000), Power::from(100));

        assert_eq!(
            extract_state(&plant),
            super::HydroPublicRepr {
                max_stock: Energy::from(1000),
                max_setpoint: Power::from(100),
                stock: Energy::from(1000),
                output: PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(0)
                },
            }
        );

        plant.program_setpoint(Power::from(100));
        plant.dispatch();

        assert_eq!(
            extract_state(&plant),
            super::HydroPublicRepr {
                max_stock: Energy::from(1000),
                max_setpoint: Power::from(100),
                stock: Energy::from(900),
                output: PlantOutput {
                    cost: Money::from(0),
                    setpoint: Power::from(0)
                },
            }
        );
    }
}
