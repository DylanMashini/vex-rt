//! SmartPort.

use crate::{
    motor::{EncoderUnits, Gearset, Motor},
    serial::Serial,
};
/// A struct which represents an unconfigured smart port.
pub struct SmartPort {
    port: u8,
}

impl SmartPort {
    /// Constructs a new smart port.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows the user to create multiple
    /// mutable references to a V5 smart port. You likely want to implement
    /// [`Robot::new()`](crate::robot::Robot::new()) instead.
    pub unsafe fn new(port: u8) -> Self {
        assert!(
            (1..22).contains(&port),
            "Cannot construct a smart port on port {}",
            port
        );
        Self { port }
    }

    /// Converts a `SmartPort` into a [`Motor`](crate::motor::Motor).
    pub fn into_motor(self, gearset: Gearset, encoder_units: EncoderUnits, reverse: bool) -> Motor {
        unsafe { Motor::new(self.port, gearset, encoder_units, reverse) }
    }

    /// Converts a `SmartPort` into a [`Serial`].
    pub fn into_serial(self, baudrate: i32) -> Serial {
        unsafe { Serial::new(self.port, baudrate) }
    }
}
