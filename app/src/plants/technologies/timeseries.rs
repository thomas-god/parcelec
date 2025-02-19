pub struct Timeseries {
    index: usize,
    values: Vec<isize>,
}

impl From<&[isize]> for Timeseries {
    fn from(values: &[isize]) -> Self {
        Timeseries {
            values: Vec::from(values),
            index: 0
        }
    }
}

impl Timeseries {
    pub fn next(&mut self) -> isize {
        let val = self.values.get(self.index).unwrap().clone();
        self.index = (self.index + 1) % self.values.len();
        val
    }
}

#[cfg(test)]
mod test {
    use crate::plants::technologies::timeseries::Timeseries;



    #[test]
    fn test_timeseries_loop_at_the_end() {
        let mut timeseries = Timeseries::from([1,2,3].as_slice());

        assert_eq!(timeseries.next(), 1);
        assert_eq!(timeseries.next(), 2);
        assert_eq!(timeseries.next(), 3);
        assert_eq!(timeseries.next(), 1);
        assert_eq!(timeseries.next(), 2);
        assert_eq!(timeseries.next(), 3);
        assert_eq!(timeseries.next(), 1);
    }
}