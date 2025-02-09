use std::fmt::Debug;

use serde::Serialize;

pub trait PowerPlant {
    type Output: PowerPlant;
    type PublicState: Serialize + Clone + Debug;

    /// Program the setpoint for the next delivery period.
    fn program_setpoint(&mut self, setpoint: isize) -> isize;

    /// Retrieve the current state of the plant.
    fn current_state(&self) -> Box<Self::PublicState>;

    /// Apply the programmed setpoint, and update the state of the plant.
    fn dispatch(self) -> (Box<Self::Output>, isize);
}
