use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::{
    forecast::Forecast,
    plants::{
        PlantOutput, PowerPlant, PowerPlantPublicRepr,
        technologies::{PowerShape, ShapeBasedPlant},
    },
    utils::units::{Money, Power},
};

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenewablePlantPublicRepr {
    pub output: PlantOutput,
}
pub struct RenewablePlant {
    state: ShapeBasedPlant,
    history: Vec<PlantOutput>,
}

impl RenewablePlant {
    pub fn new(shape: PowerShape, capacity: Power, forecasts_range: usize) -> RenewablePlant {
        let plant = ShapeBasedPlant::new(shape, capacity, forecasts_range);
        let history = Vec::new();

        RenewablePlant {
            state: plant,
            history,
        }
    }

    fn cost(&self) -> Money {
        Money::from(0)
    }

    fn setpoint(&self) -> Power {
        self.state.setpoint()
    }
}

impl PowerPlant for RenewablePlant {
    fn program_setpoint(&mut self, _setpoint: Power) -> PlantOutput {
        PlantOutput {
            setpoint: self.setpoint(),
            cost: self.cost(),
        }
    }

    fn dispatch(&mut self) -> PlantOutput {
        let output = PlantOutput {
            setpoint: self.setpoint(),
            cost: self.cost(),
        };
        self.state.dispatch();
        self.history.push(output);
        output
    }
    fn current_state(&self) -> PowerPlantPublicRepr {
        PowerPlantPublicRepr::RenewablePlant(RenewablePlantPublicRepr {
            output: PlantOutput {
                setpoint: self.setpoint(),
                cost: Money::from(0),
            },
        })
    }
    fn get_forecast(&self) -> Option<Vec<Forecast>> {
        Some(self.state.forecasts.clone())
    }

    fn get_history(&self) -> Vec<PlantOutput> {
        self.history.clone()
    }

    fn category(&self) -> crate::plants::PlantCategory {
        crate::plants::PlantCategory::RenewablePlant
    }
}

const RENEWABLES_POWER_SHAPES: [[f32; 6]; 3] = [
    [0., 0.1, 0.6, 0.8, 0.6, 0.1],    // Summer sun
    [0., 0., 0.3, 0.45, 0.3, 0.],     // Winter sun
    [0.15, 0.3, 0.55, 0.7, 0.9, 0.1], // Wind ramping
];

pub fn random_renewables_power_shape() -> PowerShape {
    let mut rng = rand::rng();
    match RENEWABLES_POWER_SHAPES.choose(&mut rng) {
        None => PowerShape::random(6),
        Some(values) => PowerShape::from(values.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        game::delivery_period::DeliveryPeriodId,
        plants::{PlantOutput, PowerPlant, technologies::PowerShape},
        utils::units::{Money, Power},
    };

    use super::RenewablePlant;

    fn shape() -> PowerShape {
        PowerShape(vec![0.1, 0.5, 0.9])
    }

    #[test]
    fn test_renewable_plant() {
        let mut plant = RenewablePlant::new(shape(), Power::from(1000), 2);

        // Initial history is empty
        assert!(plant.get_history().is_empty());

        // Plant has no associated cost
        let PlantOutput { cost, .. } = plant.program_setpoint(100.into());
        assert_eq!(cost, Money::from(0));

        // The plant cannot be programed
        let initial_setpoint: Power = plant.setpoint();
        let PlantOutput { setpoint, .. } = plant.program_setpoint(initial_setpoint + 1.into());
        assert_eq!(setpoint, initial_setpoint);

        // Dispatching should return the previous setpoint
        let previous_value: Power = plant.setpoint();
        let returned_value = plant.dispatch();
        assert_eq!(previous_value, returned_value.setpoint);
        assert_eq!(plant.get_history(), vec![returned_value]);
    }

    #[test]
    fn test_renewable_forecasts_periods() {
        let mut plant = RenewablePlant::new(shape(), Power::from(1000), 2);

        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(2), DeliveryPeriodId::from(3)]
        );

        plant.dispatch();
        let forecasts = plant.get_forecast().unwrap();
        assert_eq!(
            forecasts.iter().map(|f| f.period).collect::<Vec<_>>(),
            vec![DeliveryPeriodId::from(3), DeliveryPeriodId::from(4)]
        );
    }
}
