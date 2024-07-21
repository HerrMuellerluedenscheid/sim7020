#![no_std]
#![no_main]

use defmt_rtt as _;
pub mod at_command;

mod utils;

use at_command::AtRequest;
use bsp::entry;
use core::fmt::Debug;
use defmt::*;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use crate::at_command::at;
use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionUart, Pins},
    pac,
    sio::Sio,
    uart::{self, DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Clock,
};
use cortex_m::asm::delay;
use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_serial_Read};
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{Pin, PullDown};
use rp_pico::hal::uart::{Reader, Writer};
use rp_pico::pac::UART0;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
const BUFFER_SIZE: usize = 128;
const CR: u8 = 13;
const LF: u8 = 10;

type ModemWriter = Writer<
    UART0,
    (
        Pin<Gpio0, FunctionUart, PullDown>,
        Pin<Gpio1, FunctionUart, PullDown>,
    ),
>;

type ModemReader = Reader<
    UART0,
    (
        Pin<Gpio0, FunctionUart, PullDown>,
        Pin<Gpio1, FunctionUart, PullDown>,
    ),
>;

pub struct Modem {
    pub writer: ModemWriter,
    pub reader: ModemReader,
}

#[derive(Debug)]
pub enum AtError {
    TooManyReturnedLines,
    ErrorReply,
}

impl Modem {
    pub fn send_and_wait_reply<T: AtRequest + Format>(
        &mut self,
        payload: T,
    ) -> Result<[u8; BUFFER_SIZE], AtError> {
        info!("========>    sending data: {:?}", payload);
        payload.send(&mut self.writer);
        let mut previous_line: [u8; BUFFER_SIZE] = [b'\0'; BUFFER_SIZE];

        // Assuming there will always max 1 line containing a response followed by one 'OK' line
        for iline in 0..10_usize {
            let response = self.read_line_from_modem()?;
            debug!("line {}: {=[u8]:a}", iline, response);
            if response.starts_with(b"\x00") {
                // debug!("skipping empty line: {}", response);
                continue;
            }
            if response.starts_with(b"AT+") {
                // debug!("skipping echo line");
                continue;
            }
            if response.starts_with(b"OK") {
                // debug!("found OK");
                return Ok(previous_line);
            }
            if response.starts_with(b"ERROR") {
                error!("response data: {=[u8]:a}", response);
                return Err(AtError::ErrorReply);
            };
            info!("response data: {=[u8]:a}", response);
            previous_line = response;
        }
        info!("returning response data: {=[u8]:a}", previous_line);
        Err(AtError::TooManyReturnedLines)
    }

    fn read_line_from_modem(&mut self) -> Result<[u8; BUFFER_SIZE], AtError> {
        // muss wieder in die main loop
        // try parse unsolicited message
        // try parse other response
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut index = 0;

        loop {
            match self.reader.read() {
                Ok(CR) => return Ok(buffer),
                Ok(LF) => continue,
                Ok(byte) => {
                    buffer[index] = byte;
                    index += 1;
                    if index == BUFFER_SIZE - 1 {
                        // flush_cli(buffer);
                        error!("out of bounds index")
                        // will panic after here
                    }
                }
                Err(e) => {
                    // error!("no data")
                }
            }
        }
    }
}
