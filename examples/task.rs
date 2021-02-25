#![no_std]
#![no_main]

use core::time::Duration;
use vex_rt::prelude::*;

struct TaskBot;

impl Robot for TaskBot {
    fn new(_peripherals: Peripherals) -> Self {
        let mut x = 0;
        let mut l = Loop::new(Duration::from_secs(1));
        Task::spawn_ext(
            "test",
            Task::DEFAULT_PRIORITY,
            Task::DEFAULT_STACK_DEPTH,
            move || {
                println!("Task name: {}", Task::current().name());
                loop {
                    println!("{}", x);
                    x += 1;
                    l.delay()
                }
            },
        )
        .unwrap();
        TaskBot
    }
}

entry!(TaskBot);
