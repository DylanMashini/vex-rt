use core::time::Duration;

use crate::{
    bindings,
    rtos::{GenericSleep, Instant, Selectable},
};

use super::time_since_start;

/// Provides a constant-period looping construct.
pub struct Loop {
    last_time: u32,
    delta: u32,
}

impl Loop {
    #[inline]
    /// Creates a new loop object with a given period.
    pub fn new(delta: Duration) -> Self {
        Loop {
            last_time: time_since_start().as_millis(),
            delta: delta.as_millis() as u32,
        }
    }

    #[inline]
    /// Delays until the next loop cycle.
    pub fn delay(&mut self) {
        unsafe { bindings::task_delay_until(&mut self.last_time, self.delta) }
    }

    #[inline]
    /// A [`Selectable`] event which occurs at the next loop cycle.
    pub fn select(&'_ mut self) -> impl Selectable + '_ {
        struct LoopSelect<'a>(&'a mut Loop);

        impl<'a> Selectable for LoopSelect<'a> {
            fn poll(self) -> Result<(), Self> {
                if unsafe { bindings::millis() } >= self.0.last_time + self.0.delta {
                    self.0.last_time += self.0.delta;
                    Ok(())
                } else {
                    Err(self)
                }
            }
            fn sleep(&self) -> GenericSleep {
                GenericSleep::Timestamp(Instant::from_millis(self.0.last_time + self.0.delta))
            }
        }

        LoopSelect(self)
    }
}
