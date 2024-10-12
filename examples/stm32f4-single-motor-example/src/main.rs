//! Control a DC motor via the TB6612FNG motor driver.

#![forbid(unsafe_code)]
#![forbid(missing_debug_implementations)]
#![forbid(unused)]
#![no_std]
#![no_main]

use panic_probe as _;

use defmt_rtt as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [EXTI1])]
mod app {
    use stm32f4xx_hal::gpio::{Edge, Input, Output, PB4, PB5, PC13};
    use stm32f4xx_hal::timer::{MonoTimerUs, PwmChannel};
    use stm32f4xx_hal::{pac, pac::TIM2, prelude::*, watchdog::IndependentWatchdog};
    use tb6612fng::{DriveCommand, Motor};

    #[monotonic(binds = TIM5, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM5>;

    #[shared]
    struct Shared {
        motor: Motor<PB5<Output>, PB4<Output>, PwmChannel<TIM2, 2>>,
    }

    #[local]
    struct Local {
        watchdog: IndependentWatchdog,
        button: PC13<Input>,
        motor_ramp_direction: i8,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut syscfg = ctx.device.SYSCFG.constrain();

        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(84.MHz()).freeze();
        let mono = ctx.device.TIM5.monotonic_us(&clocks);

        let gpiob = ctx.device.GPIOB.split();
        let gpioc = ctx.device.GPIOC.split();

        // set up the motor
        let motor_in1 = gpiob.pb5.into_push_pull_output();
        let motor_in2 = gpiob.pb4.into_push_pull_output();
        let (_, (_, _, motor_pwm, ..)) = ctx.device.TIM2.pwm_hz(100.kHz(), &clocks);
        let mut motor_pwm = motor_pwm.with(gpiob.pb10);
        motor_pwm.enable();
        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm).unwrap();
        motor.drive(DriveCommand::Backward(0)).unwrap();

        // set up the button
        let mut button = gpioc.pc13.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);

        // set up the watchdog
        let mut watchdog = IndependentWatchdog::new(ctx.device.IWDG);
        watchdog.start(500u32.millis());
        watchdog.feed();
        periodic::spawn().ok();

        defmt::info!("init done!");

        update_motor_speed::spawn().ok();

        (
            Shared { motor },
            Local {
                watchdog,
                button,
                motor_ramp_direction: 1,
            },
            init::Monotonics(mono),
        )
    }

    /// Feed the watchdog to avoid hardware reset.
    #[task(priority = 1, local = [watchdog])]
    fn periodic(ctx: periodic::Context) {
        defmt::trace!("feeding the watchdog!");
        ctx.local.watchdog.feed();
        periodic::spawn_after(100.millis()).ok();
    }

    /// Increase/decrease the motor speed every 100ms by 1% (iterates from 100% forward to 100% backwards)
    #[task(priority = 1, local = [motor_ramp_direction], shared = [motor])]
    fn update_motor_speed(mut ctx: update_motor_speed::Context) {
        ctx.shared.motor.lock(|motor| {
            let motor_ramp_direction = ctx.local.motor_ramp_direction;
            let new_drive_direction = match motor.current_drive_command() {
                DriveCommand::Forward(speed) => match speed {
                    100 => {
                        *motor_ramp_direction = -1;
                        DriveCommand::Forward(99)
                    }
                    0 => {
                        *motor_ramp_direction = 1;
                        DriveCommand::Backward(1)
                    }
                    _ => DriveCommand::Forward((*speed as i8 + *motor_ramp_direction) as u8),
                },
                DriveCommand::Backward(speed) => match speed {
                    100 => {
                        *motor_ramp_direction = -1;
                        DriveCommand::Backward(99)
                    }
                    0 => {
                        *motor_ramp_direction = 1;
                        DriveCommand::Forward(1)
                    }
                    _ => DriveCommand::Backward((*speed as i8 + *motor_ramp_direction) as u8),
                },
                DriveCommand::Stop | DriveCommand::Brake => {
                    return;
                }
            };
            motor
                .drive(new_drive_direction)
                .expect("could set drive speed");
        });
        update_motor_speed::spawn_after(100.millis()).ok();
    }

    // see here for why this is EXTI15_10: https://github.com/stm32-rs/stm32f4xx-hal/blob/6d0c29233a4cd1f780b2fef3e47ef091ead6cf4a/src/gpio/exti.rs#L8-L23
    /// Start/stop/brake on button press
    #[task(binds = EXTI15_10, local = [button], shared = [motor])]
    fn button_click(mut ctx: button_click::Context) {
        ctx.local.button.clear_interrupt_pending_bit();

        ctx.shared
            .motor
            .lock(|motor| match motor.current_drive_command() {
                DriveCommand::Stop => {
                    defmt::info!("motor stopped => applying brake");
                    motor.drive(DriveCommand::Brake).unwrap();
                }
                DriveCommand::Brake => {
                    defmt::info!("brake was on => starting the motor again");
                    motor.drive(DriveCommand::Backward(0)).unwrap();
                    update_motor_speed::spawn_after(100.millis()).ok();
                }
                _ => {
                    defmt::info!("was driving so far => stopping the motor");
                    motor.drive(DriveCommand::Stop).unwrap();
                }
            });
    }
}
