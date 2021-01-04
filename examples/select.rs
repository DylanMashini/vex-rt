#![no_std]
#![no_main]

use core::time::Duration;

use vex_rt::prelude::*;
use vex_rt::rtos::Loop;
use vex_rt::select;

struct SelectRobot;

impl Robot for SelectRobot {
    fn initialize() -> Self {
        Self
    }
    fn autonomous(&self, ctx: Context) {
        println!("autonomous");
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        loop {
            println!("{}", x);
            x += 1;
            select! {
                _ = l.select() => {},
                _ = ctx.done() => break,
            }
        }
        println!("auto done")
    }
    fn opcontrol(&self, _: Context) {
        println!("opcontrol");
    }
    fn disabled(&self, _: Context) {
        println!("disabled");
    }
}

entry!(SelectRobot);
