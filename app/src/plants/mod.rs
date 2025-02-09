pub mod battery;
pub mod gas_plant;
pub mod stack;

pub trait PowerPlant {
    /// Program the setpoint for the next delivery period.
    fn program_setpoint(&mut self, setpoint: isize) -> isize;

    /// Apply the programmed setpoint, and update the state of the plant.
    fn dispatch(&mut self) -> isize;

    /// Retrieve a string representation of the plant's state
    fn current_state(&self) -> String;
}
