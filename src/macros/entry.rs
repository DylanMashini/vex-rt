#[macro_export]
/// Specifies the entrypoint for the robot.
///
/// # Examples
///
/// ```
/// #![no_std]
/// #![no_main]
///
/// use vex_rt::prelude::*;
///
/// struct FooBot;
///
/// impl Robot for FooBot {
///     fn new(_p: Peripherals) -> Self {
///         FooBot
///     }
/// }
///
/// entry!(FooBot);
/// ```
macro_rules! entry {
    ($robot_type:ty) => {
        static ROBOT: $crate::once::Once<($robot_type, Competition)> = $crate::once::Once::new();

        #[no_mangle]
        unsafe extern "C" fn initialize() {
            $crate::rtos::Task::spawn(|| {
                ROBOT.call_once(|| {
                    (
                        $crate::robot::Robot::new(unsafe {
                            $crate::peripherals::Peripherals::new()
                        }),
                        Competition::new(),
                    )
                });
            })
            .expect("failed to launch task for initialize()");
        }

        #[no_mangle]
        extern "C" fn opcontrol() {
            ROBOT.wait().1.opcontrol();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            ROBOT.wait().1.autonomous();
        }

        #[no_mangle]
        extern "C" fn disabled() {
            ROBOT.wait().1.disabled();
        }

        $crate::state_machine! {
            /// State machine for the competition modes.
            pub Competition = initialize();

            #[inline]
            /// Initialization stage of the robot.
            initialize(ctx) {
                $crate::robot::Robot::initialize(&ROBOT.wait().0, ctx);
            }

            #[inline]
            /// Driver control period.
            opcontrol(ctx) {
                $crate::robot::Robot::opcontrol(&ROBOT.wait().0, ctx);
            }

            #[inline]
            /// Autonomous period.
            autonomous(ctx) {
                $crate::robot::Robot::autonomous(&ROBOT.wait().0, ctx);
            }

            #[inline]
            /// Disabled period.
            disabled(ctx) {
                $crate::robot::Robot::disabled(&ROBOT.wait().0, ctx);
            }
        }
    };
}
