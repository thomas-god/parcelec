use crate::game::delivery_period::DeliveryPeriodId;

pub struct Timeseries {
    values: Vec<isize>,
}

impl From<&[isize]> for Timeseries {
    fn from(values: &[isize]) -> Self {
        Timeseries {
            values: Vec::from(values),
        }
    }
}

impl Timeseries {
    pub fn value_at(&self, period: DeliveryPeriodId) -> isize {
        let period: usize = period.into();
        let index = if period < 1 {
            0
        } else {
            (period - 1) % self.values.len()
        };
        *self.values.get(index).unwrap()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod test_looping_timeseries {
    use crate::{
        game::delivery_period::DeliveryPeriodId, plants::technologies::timeseries::Timeseries,
    };

    #[test]
    fn test_timeseries_loop_at_the_end() {
        let timeseries = Timeseries::from([1, 2, 3].as_slice());

        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(0)), 1);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(1)), 1);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(2)), 2);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(3)), 3);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(4)), 1);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(5)), 2);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(6)), 3);
        assert_eq!(timeseries.value_at(DeliveryPeriodId::from(7)), 1);
    }
}
