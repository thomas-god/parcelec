use rand::Rng;

pub trait Timeseries {
    fn next_value(&mut self) -> isize;
}

pub struct LoopingTimeseries {
    index: usize,
    values: Vec<isize>,
}

impl From<&[isize]> for LoopingTimeseries {
    fn from(values: &[isize]) -> Self {
        LoopingTimeseries {
            values: Vec::from(values),
            index: 0,
        }
    }
}

impl Timeseries for LoopingTimeseries {
    fn next_value(&mut self) -> isize {
        let val = self.values.get(self.index).unwrap();
        self.index = (self.index + 1) % self.values.len();
        *val
    }
}

pub struct RngTimeseries {
    min: i64,
    max: i64,
}

impl RngTimeseries {
    pub fn new(min: i64, max: i64) -> RngTimeseries {
        if min > max {
            return RngTimeseries { min: max, max: min };
        }
        RngTimeseries { min, max }
    }
}

impl Timeseries for RngTimeseries {
    fn next_value(&mut self) -> isize {
        rand::rng().random_range(self.min..=self.max) as isize
    }
}

#[cfg(test)]
mod test_looping_timeseries {
    use crate::plants::technologies::timeseries::{LoopingTimeseries, Timeseries};

    #[test]
    fn test_timeseries_loop_at_the_end() {
        let mut timeseries = LoopingTimeseries::from([1, 2, 3].as_slice());

        assert_eq!(timeseries.next_value(), 1);
        assert_eq!(timeseries.next_value(), 2);
        assert_eq!(timeseries.next_value(), 3);
        assert_eq!(timeseries.next_value(), 1);
        assert_eq!(timeseries.next_value(), 2);
        assert_eq!(timeseries.next_value(), 3);
        assert_eq!(timeseries.next_value(), 1);
    }
}

#[cfg(test)]
mod test_rng_timeseries {

    use super::{RngTimeseries, Timeseries};

    #[test]
    fn test_new_rng_timeseries_min_greater_than_max() {
        let mut timeseries = RngTimeseries::new(10, 5);
        timeseries.next_value();
    }

    #[test]
    fn test_rng_timeseries_bounds() {
        let mut timeseries = RngTimeseries::new(0, 10);

        for _ in 0..1000 {
            let val = timeseries.next_value();
            assert!(val >= 0);
            assert!(val <= 10);
        }
    }
}
