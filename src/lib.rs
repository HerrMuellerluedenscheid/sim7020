#![no_std]
#![no_main]

use defmt_rtt as _;
pub mod at_command;

mod utils;

use at_command::AtRequest;
use core::fmt::Debug;
use defmt::*;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io::{ErrorType, Read, Write};

use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use crate::at_command::at;

use cortex_m::asm::delay;
use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_serial_Read};

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
const BUFFER_SIZE: usize = 128;
const CR: u8 = 13;
const LF: u8 = 10;


pub struct Modem<'a, T: Write, U: Read> {
    pub writer: &'a mut T,
    pub reader: &'a mut U,
}

#[derive(Debug)]
pub enum AtError {
    TooManyReturnedLines,
    ErrorReply,
}

impl <T: Write, U: Read> Modem<'_, T, U> {
    pub fn send_and_wait_reply<V: AtRequest + Format>(
        &mut self,
        payload: V,
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
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 10] = [0; 10];
        loop {
            match self.reader.read(&mut read_buffer) {
                Ok(num_bytes) => {
                    for i in 0..num_bytes{
                        buffer[offset] = read_buffer[i];
                        match buffer[offset] {
                            LF => return Ok(buffer),
                            _ => {}
                        }
                        offset += 1;
                    }

                },

                Err(e) => {
                    error!("no data")
                }
            }
        }
    }
}
