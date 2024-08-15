#![no_std]
#![no_main]

pub mod at_command;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

use crate::at_command::at_cpin::PINRequired;
use at_command::AtRequest;
use at_command::AtResponse;
use defmt::*;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io::{ErrorType, Read, Write};
use crate::at_command::http::{HttpClient, HttpSession};

const BUFFER_SIZE: usize = 128;
const LF: u8 = 10; // n
const CR: u8 = 13; // r

const OK_TERMINATOR: [u8; 4] = [b'O', b'K', CR, LF];

pub struct Modem<'a, T: Write, U: Read> {
    pub writer: &'a mut T,
    pub reader: &'a mut U,
}

#[derive(Debug, defmt::Format)]
pub enum AtError {
    TooManyReturnedLines,
    ErrorReply,
    CreateHTTPSessionFailed(HttpClient),
}

impl<T: Write, U: Read> Modem<'_, T, U> {
    pub fn send_and_wait_reply<'a, V: AtRequest + Format + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<AtResponse, AtError> {

        let mut at_buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut at_buffer);
        self.writer.write(data).unwrap();
        let mut response_out: [u8; BUFFER_SIZE] = [b'\0'; BUFFER_SIZE];

        let n_bytes = self.read_response(&mut response_out);
        let response = payload.parse_response(&response_out);
        response
    }

    fn read_response(&mut self, response_out: &mut [u8; 128]) -> Result<usize, AtError> {
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 10] = [0; 10];

        loop {
            match self.reader.read(&mut read_buffer) {
                Ok(num_bytes) => {
                    for i in 0..num_bytes {
                        response_out[offset + i] = read_buffer[i];

                        // why is the index with + 1 and - 3?
                        if offset + i > 4 {
                            let start = offset + i - 3;
                            let stop = offset + i + 1;
                            if response_out[start..stop] == OK_TERMINATOR {
                                return Ok(offset + i);
                            }
                        }
                    }
                    offset += num_bytes;
                }

                Err(e) => {
                    error!("no data")
                }
            }
        }
    }
}
