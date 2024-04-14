//! Mechanical Crab
//!
//! This is a simple program that allows you to control the Arduino board using a serial terminal.
//!
//! The following commands are supported:
//! | Command | Description |
//! | ------- | ----------- |
//! | help    | Print the list of commands |
//! | led on  | Turn on the built-in LED |
//! | led off | Turn off the built-in LED |
//! | get <pin> | Read the value of a digital pin |
//! | set <pin> high | Set a digital pin to high |
//! | set <pin> low  | Set a digital pin to low |
//! | pwm <0-255> | Set the duty cycle of the PWM output |
//! | adc <0-5>   | Read the value of an analog pin |
//! | temp        | Read the temperature sensor value |
//!
//! The following pins are available:
//! - Digital pins: 2, 3, 4, 6, 7, 8, 9, 10, 11, 12
//! - Analog pins: 0, 1, 2, 3, 4, 5
//! - Built-in LED: digital pin 13
//! - PWM output: digital pin 5

#![no_std]
#![no_main]

use core::str::FromStr;

use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::mode::{Floating, Input, Output};
use arduino_hal::port::Pin;
use arduino_hal::{hal::adc, simple_pwm::*};
use embedded_hal::serial::Read;
use heapless::String;
use nb::block;
use nom::sequence::preceded;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{all_consuming, map_res, recognize, value},
    IResult,
};
#[allow(unused_imports)]
use panic_halt as _;
use ufmt::{uwrite, uwriteln};

const HELP: &str =
    "commands: help, led on|off, get <pin>, set <pin> high|low, pwm <0-255>, adc <0-5>, temp";

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57_600);
    let mut led = pins.d13.into_output();
    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale1024);
    let mut pwm = pins.d5.into_output().into_pwm(&timer0);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let a0 = pins.a0.into_analog_input(&mut adc);
    let a1 = pins.a1.into_analog_input(&mut adc);
    let a2 = pins.a2.into_analog_input(&mut adc);
    let a3 = pins.a3.into_analog_input(&mut adc);
    let a4 = pins.a4.into_analog_input(&mut adc);
    let a5 = pins.a5.into_analog_input(&mut adc);

    let mut d2 = AnyPin::DigitalIn(pins.d2.downgrade());
    let mut d3 = AnyPin::DigitalIn(pins.d3.downgrade());
    let mut d4 = AnyPin::DigitalIn(pins.d4.downgrade());
    let mut d6 = AnyPin::DigitalIn(pins.d6.downgrade());
    let mut d7 = AnyPin::DigitalIn(pins.d7.downgrade());
    let mut d8 = AnyPin::DigitalIn(pins.d8.downgrade());
    let mut d9 = AnyPin::DigitalIn(pins.d9.downgrade());
    let mut d10 = AnyPin::DigitalIn(pins.d10.downgrade());
    let mut d11 = AnyPin::DigitalIn(pins.d11.downgrade());
    let mut d12 = AnyPin::DigitalIn(pins.d12.downgrade());

    loop {
        uwrite!(&mut serial, "> ").unwrap();
        let Ok(input) = read_line(&mut serial) else {
            continue;
        };

        match parse_command(&input) {
            Ok((_, Command::Help)) => {
                let _ = uwriteln!(&mut serial, "{}", HELP);
            }
            Ok((_, Command::Led(true))) => led.set_high(),
            Ok((_, Command::Led(false))) => led.set_low(),
            Ok((_, Command::GetPin { pin })) => {
                let value = match pin {
                    2 => d2.is_high(),
                    3 => d3.is_high(),
                    4 => d4.is_high(),
                    6 => d6.is_high(),
                    7 => d7.is_high(),
                    8 => d8.is_high(),
                    9 => d9.is_high(),
                    10 => d10.is_high(),
                    11 => d11.is_high(),
                    12 => d12.is_high(),
                    _ => {
                        let _ = uwriteln!(
                            &mut serial,
                            "unknown pin: {}, valid pins are 2-4, 6-12",
                            pin
                        );
                        continue;
                    }
                };
                let _ = uwriteln!(&mut serial, "d{}: {}", pin, value);
            }
            Ok((_, Command::SetPin { pin, value })) => {
                match (pin, value) {
                    (2, true) => d2.set_high(),
                    (2, false) => d2.set_low(),
                    (3, true) => d3.set_high(),
                    (3, false) => d3.set_low(),
                    (4, true) => d4.set_high(),
                    (4, false) => d4.set_low(),
                    (6, true) => d6.set_high(),
                    (6, false) => d6.set_low(),
                    (7, true) => d7.set_high(),
                    (7, false) => d7.set_low(),
                    (8, true) => d8.set_high(),
                    (8, false) => d8.set_low(),
                    (9, true) => d9.set_high(),
                    (9, false) => d9.set_low(),
                    (10, true) => d10.set_high(),
                    (10, false) => d10.set_low(),
                    (11, true) => d11.set_high(),
                    (11, false) => d11.set_low(),
                    (12, true) => d12.set_high(),
                    (12, false) => d12.set_low(),
                    _ => {
                        let _ = uwriteln!(
                            &mut serial,
                            "unknown pin: {}, valid pins are 2-4, 6-12",
                            pin
                        );
                        continue;
                    }
                };
            }
            Ok((_, Command::Pwm { duty_cycle })) => {
                pwm.set_duty(duty_cycle);
                pwm.enable();
            }
            Ok((_, Command::Adc { pin })) => {
                let value = match pin {
                    0 => a0.analog_read(&mut adc),
                    1 => a1.analog_read(&mut adc),
                    2 => a2.analog_read(&mut adc),
                    3 => a3.analog_read(&mut adc),
                    4 => a4.analog_read(&mut adc),
                    5 => a5.analog_read(&mut adc),
                    _ => {
                        let _ = uwriteln!(&mut serial, "unknown pin: {}, valid pins are 0-5", pin);
                        continue;
                    }
                };
                let _ = uwriteln!(&mut serial, "a{}: {}", pin, value);
            }
            Ok((_, Command::Temp)) => {
                let value = adc.read_blocking(&adc::channel::Temperature);
                let _ = uwriteln!(&mut serial, "temp: 0x{:04X}", value);
            }
            Err(_) => {
                let _ = uwriteln!(&mut serial, "invalid command: {}", input.as_str());
                let _ = uwriteln!(&mut serial, "{}", HELP);
            }
        }
    }
}

enum AnyPin {
    DigitalIn(Pin<Input<Floating>, Dynamic>),
    DigitalOut(Pin<Output, Dynamic>),
}

impl AnyPin {
    fn as_input(&mut self) {
        *self = match self {
            AnyPin::DigitalIn(_) => return,
            AnyPin::DigitalOut(pin) => {
                let fake_pin = unsafe { core::mem::zeroed() };
                AnyPin::DigitalIn(
                    core::mem::replace(pin, fake_pin)
                        .into_floating_input()
                        .downgrade(),
                )
            }
        }
    }

    fn as_output(&mut self) {
        *self = match self {
            AnyPin::DigitalIn(ref mut pin) => {
                let fake_pin = unsafe { core::mem::zeroed() };
                AnyPin::DigitalOut(core::mem::replace(pin, fake_pin).into_output().downgrade())
            }
            AnyPin::DigitalOut(_) => return,
        };
    }

    fn is_high(&self) -> bool {
        match self {
            AnyPin::DigitalIn(pin) => pin.is_high(),
            AnyPin::DigitalOut(pin) => pin.is_set_high(),
        }
    }

    fn set_high(&mut self) {
        self.as_output();
        if let AnyPin::DigitalOut(pin) = self {
            pin.set_high();
        } else {
            unreachable!("pin is not an output");
        }
    }

    fn set_low(&mut self) {
        self.as_output();
        if let AnyPin::DigitalOut(pin) = self {
            pin.set_low();
        } else {
            unreachable!("pin is not an output");
        }
    }
}

enum Command {
    Help,
    Led(bool),
    GetPin { pin: u8 },
    SetPin { pin: u8, value: bool },
    Pwm { duty_cycle: u8 },
    Adc { pin: u8 },
    Temp,
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    let (input, cmd) = alt((
        all_consuming(tag("help")),
        tag("led"),
        tag("get"),
        tag("set"),
        tag("pwm"),
        tag("adc"),
        all_consuming(tag("temp")),
    ))(input)?;
    match cmd {
        "help" => Ok((input, Command::Help)),
        "led" => all_consuming(parse_led_command)(input),
        "get" => all_consuming(parse_get_pin_command)(input),
        "set" => all_consuming(parse_set_pin_command)(input),
        "pwm" => all_consuming(parse_pwm_command)(input),
        "adc" => all_consuming(parse_adc_command)(input),
        "temp" => Ok((input, Command::Temp)),
        _ => unreachable!(),
    }
}

fn parse_led_command(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag(" ")(input)?;
    let (input, value) = alt((value(true, tag("on")), value(false, tag("off"))))(input)?;
    Ok((input, Command::Led(value)))
}

fn parse_get_pin_command(input: &str) -> IResult<&str, Command> {
    let (input, pin) = preceded(tag(" "), parse_number)(input)?;
    Ok((input, Command::GetPin { pin }))
}

fn parse_set_pin_command(input: &str) -> IResult<&str, Command> {
    let (input, pin) = preceded(tag(" "), parse_number)(input)?;
    let (input, value) = preceded(
        tag(" "),
        alt((value(true, tag("high")), value(false, tag("low")))),
    )(input)?;
    Ok((input, Command::SetPin { pin, value }))
}

fn parse_pwm_command(input: &str) -> IResult<&str, Command> {
    let (input, duty_cycle) = preceded(tag(" "), parse_number)(input)?;
    Ok((input, Command::Pwm { duty_cycle }))
}

fn parse_adc_command(input: &str) -> IResult<&str, Command> {
    let (input, pin) = preceded(tag(" "), parse_number)(input)?;
    Ok((input, Command::Adc { pin }))
}

/// Parses a number from the input string.
fn parse_number<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
{
    map_res(
        recognize(take_while1(|c: char| c.is_ascii_digit())),
        FromStr::from_str,
    )(input)
}

/// Reads a line of up to 32 characters from the serial port, returning it.
///
/// The terminating newline character is not included in the returned string.
fn read_line<R: Read<u8>>(serial: &mut R) -> Result<String<32>, ()> {
    let mut buf = String::new();
    loop {
        let byte = block!(serial.read()).map_err(|_| ())?;
        if byte == b'\n' {
            break;
        }
        buf.push(byte as char)?;
    }
    Ok(buf)
}
