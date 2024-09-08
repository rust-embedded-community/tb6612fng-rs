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
//! * `defmt-03`: you can enable this feature to get a `defmt::Format` implementation for all structs & enums in this crate and a `defmt::debug` call for every speed change.

#![forbid(unsafe_code)]
#![deny(warnings)]
#![forbid(missing_docs)]
#![forbid(missing_debug_implementations)]
#![deny(unused)]
#![no_std]

#[cfg(feature = "defmt-03")]
use defmt::Format;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use embedded_hal::pwm::SetDutyCycle;

/// Defines errors which can happen when calling [`Motor::drive()`].
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt-03", derive(Format))]
pub enum MotorError<IN1Error, IN2Error, PWMError> {
    /// An invalid speed has been defined. The speed must be given as a percentage value between 0 and 100 to be valid.
    InvalidSpeed,
    /// An error in setting the output of the IN1 pin
    In1Error(IN1Error),
    /// An error in setting the output of the IN2 pin
    In2Error(IN2Error),
    /// An error in setting the output of the PWM pin
    PwmError(PWMError),
}

/// Defines errors which can happen when calling [`Tb6612fng::new()`].
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt-03", derive(Format))]
pub enum Tb6612fngError<STBYError> {
    /// An error in setting the initial output of the standby pin
    Standby(STBYError),
}

/// Defines the possible drive commands.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt-03", derive(Format))]
pub enum DriveCommand {
    /// Drive forward with the defined speed (in percentage)
    Forward(u8),
    /// Drive backward with the defined speed (in percentage)
    Backward(u8),
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
#[cfg_attr(feature = "defmt-03", derive(Format))]
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
    MAPWM: SetDutyCycle,
    MBIN1: OutputPin,
    MBIN2: OutputPin,
    MBPWM: SetDutyCycle,
    STBY: OutputPin,
{
    /// Instantiate a new [`Tb6612fng`] with the defined pins.
    /// This also automatically enables the two PWM pins.
    /// The initial state of the motors will be set to [stopped](DriveCommand::Stop).
    /// The initial state of standby will be *disabled*.
    ///
    /// # Errors
    /// If any of the underlying pin interactions fail these errors will be propagated up.
    /// The errors are specific to your HAL.
    ///
    /// # Usage example
    /// ```
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # use embedded_hal_mock::eh1::pwm::Mock as PwmMock;
    /// # use embedded_hal_mock::eh1::pwm::Transaction as PwmTransaction;
    /// # use embedded_hal_mock::eh1::digital::Transaction as PinTransaction;
    /// # use embedded_hal_mock::eh1::digital::State::{High, Low};
    ///
    /// # let motor_a_in1 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_a_in1_ = motor_a_in1.clone();
    /// # let motor_a_in2 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_a_in2_ = motor_a_in2.clone();
    /// # let motor_a_pwm = PwmMock::new(&[PwmTransaction::max_duty_cycle(100), PwmTransaction::set_duty_cycle(0)]);
    /// # let mut motor_a_pwm_ = motor_a_pwm.clone();
    ///
    /// # let motor_b_in1 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_b_in1_ = motor_b_in1.clone();
    /// # let motor_b_in2 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_b_in2_ = motor_b_in2.clone();
    /// # let motor_b_pwm = PwmMock::new(&[PwmTransaction::max_duty_cycle(100), PwmTransaction::set_duty_cycle(0)]);
    /// # let mut motor_b_pwm_ = motor_b_pwm.clone();
    ///
    /// # let standby = PinMock::new(&[PinTransaction::set(High)]);
    /// # let mut standby_ = standby.clone();
    ///
    /// use tb6612fng::{Motor, Tb6612fng};
    ///
    /// let controller = Tb6612fng::new(
    ///     Motor::new(motor_a_in1, motor_a_in2, motor_a_pwm).unwrap(),
    ///     Motor::new(motor_b_in1, motor_b_in2, motor_b_pwm).unwrap(),
    ///     standby,
    /// );
    ///
    /// # motor_a_in1_.done();
    /// # motor_a_in2_.done();
    /// # motor_a_pwm_.done();
    /// # motor_b_in1_.done();
    /// # motor_b_in2_.done();
    /// # motor_b_pwm_.done();
    /// # standby_.done();
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn new(
        motor_a: Motor<MAIN1, MAIN2, MAPWM>,
        motor_b: Motor<MBIN1, MBIN2, MBPWM>,
        standby: STBY,
    ) -> Result<
        Tb6612fng<MAIN1, MAIN2, MAPWM, MBIN1, MBIN2, MBPWM, STBY>,
        Tb6612fngError<STBY::Error>,
    > {
        let mut controller = Tb6612fng {
            motor_a,
            motor_b,
            standby,
        };

        controller
            .disable_standby()
            .map_err(Tb6612fngError::Standby)?;

        Ok(controller)
    }

    /// Enable standby. This ignores any other setting currently done on the motors and puts them into standby.
    ///
    /// Note that this does not change any commands on the motors, i.e. the PWM signal will continue
    /// and once [`Tb6612fng::disable_standby`] is called the motor will pick up where it left off (unless the command was changed in-between).
    ///
    /// # Errors
    /// If the underlying pin interaction fails this error will be propagated up.
    /// The error is specific to your HAL.
    pub fn enable_standby(&mut self) -> Result<(), STBY::Error> {
        self.standby.set_low()
    }

    /// Disable standby. Note that the last active commands on the motors will resume.
    ///
    /// # Errors
    /// If the underlying pin interaction fails this error will be propagated up.
    /// The error is specific to your HAL.
    pub fn disable_standby(&mut self) -> Result<(), STBY::Error> {
        self.standby.set_high()
    }

    /// Returns whether the standby mode is enabled.
    ///
    /// *NOTE* this does *not* read the electrical state of the pin, see [`StatefulOutputPin`]
    ///
    /// # Errors
    /// If the underlying pin interaction fails this error will be propagated up.
    /// The error is specific to your HAL.
    pub fn current_standby(&mut self) -> Result<bool, STBY::Error>
    where
        STBY: StatefulOutputPin,
    {
        self.standby.is_set_high()
    }
}

/// Represents a single motor (either motor A or motor B) hooked up to a TB6612FNG controller.
///
/// This is unaware of the standby pin. If you plan on using both motors and the standby feature then use the [`Tb6612fng`] struct instead.
/// See the crate-level comment for further details on when to use what.
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(Format))]
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
    PWM: SetDutyCycle,
{
    /// Instantiate a new [`Motor`] with the defined pins.
    /// This also automatically enables the PWM pin.
    /// The initial state of the motor will be set to [stopped](DriveCommand::Stop).
    ///
    /// # Errors
    /// If any of the underlying pin interactions fail these errors will be propagated up.
    /// The errors are specific to your HAL.
    ///
    /// # Usage example
    /// ```
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # use embedded_hal_mock::eh1::pwm::Mock as PwmMock;
    /// # use embedded_hal_mock::eh1::pwm::Transaction as PwmTransaction;
    /// # use embedded_hal_mock::eh1::digital::Transaction as PinTransaction;
    /// # use embedded_hal_mock::eh1::digital::State::{Low};
    /// # let motor_in1 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_in1_ = motor_in1.clone();
    /// # let motor_in2 = PinMock::new(&[PinTransaction::set(Low)]);
    /// # let mut motor_in2_ = motor_in2.clone();
    /// # let motor_pwm = PwmMock::new(&[PwmTransaction::max_duty_cycle(100), PwmTransaction::set_duty_cycle(0)]);
    /// # let mut motor_pwm_ = motor_pwm.clone();
    /// use tb6612fng::Motor;
    ///
    /// let motor = Motor::new(
    ///     motor_in1,
    ///     motor_in2,
    ///     motor_pwm,
    /// );
    ///
    /// # motor_in1_.done();
    /// # motor_in2_.done();
    /// # motor_pwm_.done();
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn new(
        in1: IN1,
        in2: IN2,
        pwm: PWM,
    ) -> Result<Motor<IN1, IN2, PWM>, MotorError<IN1::Error, IN2::Error, PWM::Error>> {
        let mut motor = Motor {
            in1,
            in2,
            pwm,
            current_drive_command: DriveCommand::Stop,
        };

        motor.drive(motor.current_drive_command)?;

        Ok(motor)
    }

    /// Drive with the defined speed (or brake or stop the motor).
    ///
    /// # Errors
    /// If the underlying pin interaction fails this error will be propagated up.
    /// The error is specific to your HAL.
    ///
    /// The specified speed must be between 0 and 100 (inclusive), otherwise you will get a
    /// [`MotorError::InvalidSpeed`] error.
    #[allow(clippy::type_complexity)]
    pub fn drive(
        &mut self,
        drive_command: DriveCommand,
    ) -> Result<(), MotorError<IN1::Error, IN2::Error, PWM::Error>> {
        let speed = match drive_command {
            DriveCommand::Forward(s) | DriveCommand::Backward(s) => s,
            _ => 0,
        };

        if speed > 100 {
            return Err(MotorError::InvalidSpeed);
        }

        match drive_command {
            DriveCommand::Forward(_) => {
                self.in1.set_high().map_err(MotorError::In1Error)?;
                self.in2.set_low().map_err(MotorError::In2Error)?;
            }
            DriveCommand::Backward(_) => {
                self.in1.set_low().map_err(MotorError::In1Error)?;
                self.in2.set_high().map_err(MotorError::In2Error)?;
            }
            DriveCommand::Brake => {
                self.in1.set_high().map_err(MotorError::In1Error)?;
                self.in2.set_high().map_err(MotorError::In2Error)?;
            }
            DriveCommand::Stop => {
                self.in1.set_low().map_err(MotorError::In1Error)?;
                self.in2.set_low().map_err(MotorError::In2Error)?;
            }
        }

        #[cfg(feature = "defmt-03")]
        defmt::debug!("driving {} with speed {}", drive_command, speed);

        self.pwm
            .set_duty_cycle_percent(speed)
            .map_err(MotorError::PwmError)?;

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
    /// while driving backward returns a negative number and both [`DriveCommand::Brake`] and [`DriveCommand::Stop`] return 0.
    ///
    /// If you need to know in more details what the current status is, consider calling [`Motor::current_drive_command`] instead.
    pub fn current_speed(&self) -> i8 {
        match self.current_drive_command() {
            DriveCommand::Forward(s) => *s as i8,
            DriveCommand::Backward(s) => -(*s as i8),
            DriveCommand::Brake => 0,
            DriveCommand::Stop => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{DriveCommand, Motor, MotorError};
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::digital::State::{High, Low};
    use embedded_hal_mock::eh1::digital::Transaction as PinTransaction;
    use embedded_hal_mock::eh1::pwm::Mock as PwmMock;
    use embedded_hal_mock::eh1::pwm::Transaction as PwmTransaction;

    #[test]
    fn test_motor_stop() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(Low), PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(Low), PinTransaction::set(Low)];
        let motor_pwm_expectations = [
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
        ];
        let mut motor_in1 = PinMock::new(&motor_in1_expectations);
        let mut motor_in2 = PinMock::new(&motor_in2_expectations);
        let mut motor_pwm = PwmMock::new(&motor_pwm_expectations);

        let mut motor =
            Motor::new(motor_in1.clone(), motor_in2.clone(), motor_pwm.clone()).unwrap();

        motor.drive(DriveCommand::Stop).unwrap();

        assert_eq!(*motor.current_drive_command(), DriveCommand::Stop);
        assert_eq!(motor.current_speed(), 0);

        motor_in1.done();
        motor_in2.done();
        motor_pwm.done();
    }

    #[test]
    fn test_motor_brake() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(Low), PinTransaction::set(High)];
        let motor_in2_expectations = [PinTransaction::set(Low), PinTransaction::set(High)];
        let motor_pwm_expectations = [
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
        ];
        let mut motor_in1 = PinMock::new(&motor_in1_expectations);
        let mut motor_in2 = PinMock::new(&motor_in2_expectations);
        let mut motor_pwm = PwmMock::new(&motor_pwm_expectations);

        let mut motor =
            Motor::new(motor_in1.clone(), motor_in2.clone(), motor_pwm.clone()).unwrap();

        motor.drive(DriveCommand::Brake).unwrap();

        assert_eq!(*motor.current_drive_command(), DriveCommand::Brake);
        assert_eq!(motor.current_speed(), 0);

        motor_in1.done();
        motor_in2.done();
        motor_pwm.done();
    }

    #[test]
    fn test_motor_drive_forward() {
        let max_duty = 100;
        let speed: u8 = 100;
        let motor_in1_expectations = [PinTransaction::set(Low), PinTransaction::set(High)];
        let motor_in2_expectations = [PinTransaction::set(Low), PinTransaction::set(Low)];
        let motor_pwm_expectations = [
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(speed as u16),
        ];
        let mut motor_in1 = PinMock::new(&motor_in1_expectations);
        let mut motor_in2 = PinMock::new(&motor_in2_expectations);
        let mut motor_pwm = PwmMock::new(&motor_pwm_expectations);

        let mut motor =
            Motor::new(motor_in1.clone(), motor_in2.clone(), motor_pwm.clone()).unwrap();

        motor
            .drive(DriveCommand::Forward(speed))
            .expect("speed can be set");

        assert_eq!(*motor.current_drive_command(), DriveCommand::Forward(100));
        assert_eq!(motor.current_speed(), speed as i8);

        motor_in1.done();
        motor_in2.done();
        motor_pwm.done();
    }

    #[test]
    fn test_motor_drive_backward() {
        let max_duty = 100;
        let speed = 100;
        let motor_in1_expectations = [PinTransaction::set(Low), PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(Low), PinTransaction::set(High)];
        let motor_pwm_expectations = [
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(speed as u16),
        ];
        let mut motor_in1 = PinMock::new(&motor_in1_expectations);
        let mut motor_in2 = PinMock::new(&motor_in2_expectations);
        let mut motor_pwm = PwmMock::new(&motor_pwm_expectations);

        let mut motor =
            Motor::new(motor_in1.clone(), motor_in2.clone(), motor_pwm.clone()).unwrap();

        motor
            .drive(DriveCommand::Backward(speed))
            .expect("speed can be set");

        assert_eq!(*motor.current_drive_command(), DriveCommand::Backward(100));
        assert_eq!(motor.current_speed(), -(speed as i8));

        motor_in1.done();
        motor_in2.done();
        motor_pwm.done();
    }

    #[test]
    fn test_motor_drive_invalid_speed() {
        let max_duty = 100;
        let motor_in1_expectations = [PinTransaction::set(Low)];
        let motor_in2_expectations = [PinTransaction::set(Low)];
        let motor_pwm_expectations = [
            PwmTransaction::max_duty_cycle(max_duty),
            PwmTransaction::set_duty_cycle(0),
        ];
        let mut motor_in1 = PinMock::new(&motor_in1_expectations);
        let mut motor_in2 = PinMock::new(&motor_in2_expectations);
        let mut motor_pwm = PwmMock::new(&motor_pwm_expectations);

        let mut motor =
            Motor::new(motor_in1.clone(), motor_in2.clone(), motor_pwm.clone()).unwrap();

        let current_drive_command = *motor.current_drive_command();
        let current_speed = motor.current_speed();

        assert_eq!(
            motor
                .drive(DriveCommand::Forward(101))
                .expect_err("Invalid speed must result in an exception"),
            MotorError::InvalidSpeed
        );

        // this should still be what was set before the invalid command
        assert_eq!(*motor.current_drive_command(), current_drive_command);
        assert_eq!(motor.current_speed(), current_speed);

        motor_in1.done();
        motor_in2.done();
        motor_pwm.done();
    }
}
