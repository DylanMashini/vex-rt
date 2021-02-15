#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct DriveTrain {
    left_motor: Motor,
    right_motor: Motor,
}

impl DriveTrain {
    fn spin(&mut self, velocity: i8) {
        self.left_motor.move_i8(velocity).unwrap();
        self.right_motor.move_i8(velocity).unwrap();
    }
}

struct ClawBot {
    controller: Controller,
    drive_train: Mutex<DriveTrain>,
}

impl Robot for ClawBot {
    fn new(p: Peripherals) -> Self {
        ClawBot {
            controller: p.master_controller,
            drive_train: Mutex::new(DriveTrain {
                left_motor: p.port01.into_motor(
                    Gearset::EighteenToOne,
                    EncoderUnits::Degrees,
                    false,
                ),
                right_motor: p.port02.into_motor(
                    Gearset::EighteenToOne,
                    EncoderUnits::Degrees,
                    true,
                ),
            }),
        }
    }

    fn opcontrol(&self, ctx: Context) {
        let mut l = Loop::new(Duration::from_millis(10));
        let mut drive_train = self.drive_train.lock();

        loop {
            select! {
                _ = ctx.done() => break,
                _ = l.select() => {
                    let velocity = self.controller.left_stick.get_x().unwrap();
                    drive_train.spin(velocity);
                },
            }
        }
    }

    fn disabled(&self, _ctx: Context) {
        self.drive_train.lock().spin(0);
    }
}

entry!(ClawBot);
