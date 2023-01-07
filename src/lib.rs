//! This is a `no_std` driver for the [TB6612FNG motor driver](https://www.sparkfun.com/datasheets/Robotics/TB6612FNG.pdf) as can e.g. be found on the corresponding [SparkFun module](https://www.sparkfun.com/products/14450).
//!
//! The motor driver itself supports two motors and has a standby pin which controls both motors at the same time.
//! The crate can be either used to control a single motor (using the [`Motor`] struct directly) or
//! to control both motors (using the [`Tb6612fng`] struct) - the latter also supports using the standby functionality.
//!
//! ## When to use what
//! * You plan on using both motors and the standby feature: use [`Tb6612fng`]
//! * You plan on using both motors without the standby feature: use two separate [`Motor`]s
//! * You plan on using a single motor with the standby feature: use [`Motor`] and control the standby pin manually
//! * You plan on using a single motor without the standby feature: use [`Motor`]
//!
//! ## Optional features
//! * `defmt`: you can enable the `defmt` feature to get a `defmt::Format` implementation for all structs & enums in this crate and a `defmt::debug` call for every speed change.

#![forbid(unsafe_code)]
#![forbid(warnings)]
#![forbid(missing_docs)]
#![forbid(missing_debug_implementations)]
#![forbid(unused)]
#![no_std]

#[cfg(feature = "defmt")]
use defmt::Format;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;

/// Defines errors which can happen while trying to set a speed.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum DriveError {
    /// An invalid speed has been defined. The speed must be given as a percentage value between 0 and 100 to be valid.
    InvalidSpeed,
}

/// Defines the possible drive commands.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum DriveCommand {
    /// Drive forward with the defined speed (in percentage)
    Forward(u8),
    /// Drive backwards with the defined speed (in percentage)
    Backwards(u8),
    /// Actively brake
    Brake,
    /// Coast, i.e. stop but don't actively brake.
    Stop,
}

/// Represents a TB6612FNG controller.
///
/// Use the [`Motor`] struct directly if you only have one motor.
/// See the crate-level comment for further details on when to use what.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct Tb6612fng<MAIN1, MAIN2, MAPWM, MBIN1, MBIN2, MBPWM, STBY> {
    /// The first motor, labelled as 'A' on the chip
    pub motor_a: Motor<MAIN1, MAIN2, MAPWM>,
    /// The second motor, labelled as 'B' on the chip
    pub motor_b: Motor<MBIN1, MBIN2, MBPWM>,
    /// The standby pin used to put both motors on standby
    standby: STBY,
}

impl<MAIN1, MAIN2, MAPWM, MBIN1, MBIN2, MBPWM, STBY>
    Tb6612fng<MAIN1, MAIN2, MAPWM, MBIN1, MBIN2, MBPWM, STBY>
where
    MAIN1: OutputPin,
    MAIN2: OutputPin,
    MAPWM: PwmPin<Duty = u16>,
    MBIN1: OutputPin,
    MBIN2: OutputPin,
    MBPWM: PwmPin<Duty = u16>,
    STBY: OutputPin,
{
    /// Instantiate a new [`Tb6612fng`] with the defined pins.
    /// This also automatically enables the two PWM pins.
    /// The initial state of the motors will be [stopped](DriveCommand::Stop).
    ///
    /// Usage example:
    /// ```
    /// # use embedded_hal_mock::pin::Mock as PinMock;
    /// # use embedded_hal_mock::pin::Transaction as PinTransaction;
    /// # let motor_a_in1 = PinMock::new([]);
    /// # let motor_a_in2 = PinMock::new([]);
    /// # let motor_a_pwm_expectations = [PinTransaction::enable()];
    /// # let motor_a_pwm = PinMock::new(&motor_a_pwm_expectations);
    /// # let motor_b_in1 = PinMock::new([]);
    /// # let motor_b_in2 = PinMock::new([]);
    /// # let motor_b_pwm_expectations = [PinTransaction::enable()];
    /// # let motor_b_pwm = PinMock::new(&motor_a_pwm_expectations);
    /// # let standby = PinMock::new([]);
    /// use tb6612fng::Tb6612fng;
    ///
    /// let controller = Tb6612fng::new(
    ///     motor_a_in1,
    ///     motor_a_in2,
    ///     motor_a_pwm,
    ///     motor_b_in1,
    ///     motor_b_in2,
    ///     motor_b_pwm,
    ///     standby,
    /// );
    /// ```
    pub fn new(
        motor_a_in1: MAIN1,
        motor_a_in2: MAIN2,
        motor_a_pwm: MAPWM,
        motor_b_in1: MBIN1,
        motor_b_in2: MBIN2,
        motor_b_pwm: MBPWM,
        standby: STBY,
    ) -> Tb6612fng<MAIN1, MAIN2, MAPWM, MBIN1, MBIN2, MBPWM, STBY> {
        Tb6612fng {
            motor_a: Motor::new(motor_a_in1, motor_a_in2, motor_a_pwm),
            motor_b: Motor::new(motor_b_in1, motor_b_in2, motor_b_pwm),
            standby,
        }
    }

    /// Enable standby. This ignores any other setting currently done on the motors and puts them into standby.
    ///
    /// Note that this does not change any commands on the motors, i.e. the PWM signal will continue
    /// and once [`Tb6612fng::disable_standby`] is called the motor will pick up where it left off (unless the command was changed in-between).
    pub fn enable_standby(&mut self) {
        self.standby.set_low().ok();
    }

    /// Disable standby. Note that the last active commands on the motors will resume.
    pub fn disable_standby(&mut self) {
        self.standby.set_high().ok();
    }
}

/// Represents a single motor (either motor A or motor B) hooked up to a TB6612FNG controller.
/// This is unaware of the standby pin. If you plan on using both motors and the standby feature then use the [`Tb6612fng`] struct instead.
/// See the crate-level comment for further details on when to use what.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct Motor<IN1, IN2, PWM> {
    in1: IN1,
    in2: IN2,
    pwm: PWM,
    current_drive_command: DriveCommand,
}

impl<IN1, IN2, PWM> Motor<IN1, IN2, PWM>
where
    IN1: OutputPin,
    IN2: OutputPin,
    PWM: PwmPin<Duty = u16>,
{
    /// Instantiate a new [`Motor`] with the defined pins.
    /// This also automatically enables the PWM pin.
    /// The initial state of the motor will be [stopped](DriveCommand::Stop).
    ///
    /// Usage example:
    /// ```
    /// # use embedded_hal_mock::pin::Mock as PinMock;
    /// # use embedded_hal_mock::pin::Transaction as PinTransaction;
    /// # let motor_in1 = PinMock::new([]);
    /// # let motor_in2 = PinMock::new([]);
    /// # let motor_pwm_expectations = [PinTransaction::enable()];
    /// # let motor_pwm = PinMock::new(&motor_pwm_expectations);
    /// use tb6612fng::Motor;
    ///
    /// let motor = Motor::new(
    ///     motor_in1,
    ///     motor_in2,
    ///     motor_pwm,
    /// );
    /// ```
    pub fn new(in1: IN1, in2: IN2, mut pwm: PWM) -> Motor<IN1, IN2, PWM> {
        pwm.enable();
        Motor {
            in1,
            in2,
            pwm,
            current_drive_command: DriveCommand::Stop,
        }
    }

    /// Drive forward with the defined speed. Note that the speed is a percentage between 0 and 100!
    pub fn drive_forward(&mut self, speed: u8) -> Result<(), DriveError> {
        self.drive(DriveCommand::Forward(speed))
    }

    /// Drive backwards with the defined speed. Note that the speed is a percentage between 0 and 100!
    pub fn drive_backwards(&mut self, speed: u8) -> Result<(), DriveError> {
        self.drive(DriveCommand::Backwards(speed))
    }

    /// Actively brake.
    pub fn brake(&mut self) {
        self.drive(DriveCommand::Brake)
            .expect("could set speed to brake");
    }

    /// Stop the motor but don't brake (let it coast).
    pub fn stop(&mut self) {
        self.drive(DriveCommand::Stop)
            .expect("could set speed to stop");
    }

    /// Drive with the defined speed (or brake or stop the motor).
    pub fn drive(&mut self, drive_command: DriveCommand) -> Result<(), DriveError> {
        let speed = match drive_command {
            DriveCommand::Forward(s) | DriveCommand::Backwards(s) => s,
            _ => 0,
        };

        if speed > 100 {
            return Err(DriveError::InvalidSpeed);
        }

        match drive_command {
            DriveCommand::Forward(_) => {
                self.in1.set_high().ok();
                self.in2.set_low().ok();
            }
            DriveCommand::Backwards(_) => {
                self.in1.set_low().ok();
                self.in2.set_high().ok();
            }
            DriveCommand::Brake => {
                self.in1.set_high().ok();
                self.in2.set_high().ok();
            }
            DriveCommand::Stop => {
                self.in1.set_low().ok();
                self.in2.set_low().ok();
            }
        }

        let max_duty = self.pwm.get_max_duty();

        let duty = (speed as f32 * (max_duty as f32 / 100.0)) as u16; // speed given in percentage

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "driving {} with duty {} (max duty: {})",
            drive_command,
            duty,
            max_duty
        );

        self.pwm.set_duty(duty);

        self.current_drive_command = drive_command;

        Ok(())
    }

    /// Get the currently active drive command.
    ///
    /// If you only want to know the speed consider calling [`Motor::current_speed`] instead.
    pub fn current_drive_command(&self) -> &DriveCommand {
        &self.current_drive_command
    }

    /// Return the current speed of the motor (in percentage). Note that driving forward returns a positive number
    /// while driving backwards returns a negative number and both [`DriveCommand::Brake`] and [`DriveCommand::Stop`] return 0.
    ///
    /// If you need to know in more details what the current status is consider calling [`Motor::current_drive_command`] instead.
    pub fn current_speed(&self) -> i8 {
        match self.current_drive_command() {
            DriveCommand::Forward(s) => *s as i8,
            DriveCommand::Backwards(s) => -(*s as i8),
            DriveCommand::Brake => 0,
            DriveCommand::Stop => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{DriveCommand, DriveError, Motor};
    use embedded_hal_mock::pin::State::{High, Low};
    use embedded_hal_mock::pin::Transaction as PinTransaction;
    use embedded_hal_mock::pin::{Mock as PinMock, PwmDuty};

    #[test]
    fn test_motor_stop() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(Low)];
        let motor_pwm_expectations = [
            PinTransaction::enable(),
            PinTransaction::get_max_duty(max_duty),
            PinTransaction::set_duty(0),
        ];
        let motor_in1 = PinMock::new(&motor_in1_expectations);
        let motor_in2 = PinMock::new(&motor_in2_expectations);
        let motor_pwm = PinMock::new(&motor_pwm_expectations);

        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm);

        motor.stop();

        assert_eq!(*motor.current_drive_command(), DriveCommand::Stop);
        assert_eq!(motor.current_speed(), 0);
    }

    #[test]
    fn test_motor_brake() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(High)];
        let motor_in2_expectations = [PinTransaction::set(High)];
        let motor_pwm_expectations = [
            PinTransaction::enable(),
            PinTransaction::get_max_duty(max_duty),
            PinTransaction::set_duty(0),
        ];
        let motor_in1 = PinMock::new(&motor_in1_expectations);
        let motor_in2 = PinMock::new(&motor_in2_expectations);
        let motor_pwm = PinMock::new(&motor_pwm_expectations);

        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm);

        motor.brake();

        assert_eq!(*motor.current_drive_command(), DriveCommand::Brake);
        assert_eq!(motor.current_speed(), 0);
    }

    #[test]
    fn test_motor_drive_forward() {
        let max_duty = 100;
        let speed: u8 = 100;
        let motor_in1_expectations = [PinTransaction::set(High)];
        let motor_in2_expectations = [PinTransaction::set(Low)];
        let motor_pwm_expectations = [
            PinTransaction::enable(),
            PinTransaction::get_max_duty(max_duty),
            PinTransaction::set_duty(speed as PwmDuty),
        ];
        let motor_in1 = PinMock::new(&motor_in1_expectations);
        let motor_in2 = PinMock::new(&motor_in2_expectations);
        let motor_pwm = PinMock::new(&motor_pwm_expectations);

        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm);

        motor.drive_forward(speed).expect("speed can be set");

        assert_eq!(*motor.current_drive_command(), DriveCommand::Forward(100));
        assert_eq!(motor.current_speed(), speed as i8);
    }

    #[test]
    fn test_motor_drive_backwards() {
        let max_duty = 100;
        let speed: u8 = 100;
        let motor_in1_expectations = [PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(High)];
        let motor_pwm_expectations = [
            PinTransaction::enable(),
            PinTransaction::get_max_duty(max_duty),
            PinTransaction::set_duty(speed as PwmDuty),
        ];
        let motor_in1 = PinMock::new(&motor_in1_expectations);
        let motor_in2 = PinMock::new(&motor_in2_expectations);
        let motor_pwm = PinMock::new(&motor_pwm_expectations);

        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm);

        motor.drive_backwards(speed).expect("speed can be set");

        assert_eq!(*motor.current_drive_command(), DriveCommand::Backwards(100));
        assert_eq!(motor.current_speed(), -(speed as i8));
    }

    #[test]
    fn test_motor_drive_invalid_speed() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(High)];
        let motor_pwm_expectations = [
            PinTransaction::enable(),
            PinTransaction::get_max_duty(max_duty),
            PinTransaction::set_duty(100),
        ];
        let motor_in1 = PinMock::new(&motor_in1_expectations);
        let motor_in2 = PinMock::new(&motor_in2_expectations);
        let motor_pwm = PinMock::new(&motor_pwm_expectations);

        let mut motor = Motor::new(motor_in1, motor_in2, motor_pwm);

        let current_drive_command = motor.current_drive_command().clone();
        let current_speed = motor.current_speed();

        assert_eq!(
            motor
                .drive_forward(101)
                .expect_err("Invalid speed must result in an exception"),
            DriveError::InvalidSpeed
        );

        // this should still be what was set before the invalid command
        assert_eq!(*motor.current_drive_command(), current_drive_command);
        assert_eq!(motor.current_speed(), current_speed);
    }
}
