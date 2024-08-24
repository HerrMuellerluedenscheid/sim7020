#![no_std]
#![no_main]

pub mod at_command;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

use crate::at_command::http::HttpClient;
use at_command::AtRequest;
use at_command::AtResponse;
#[cfg(feature = "defmt")]
use defmt::*;
pub use embedded_io::{ErrorType, Read, Write};

const BUFFER_SIZE: usize = 512;
const LF: u8 = 10; // n
const CR: u8 = 13; // r

const OK_TERMINATOR: [u8; 6] = [CR, LF, b'O', b'K', CR, LF];
const ERROR_TERMINATOR: [u8; 6] = [b'R', b'R', b'O', b'R', CR, LF];

pub struct Modem<'a, T: Write, U: Read> {
    pub writer: &'a mut T,
    pub reader: &'a mut U,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum AtError {
    TooManyReturnedLines,
    ErrorReply(usize),
    CreateHTTPSessionFailed(HttpClient),
}

impl<T: Write, U: Read> Modem<'_, T, U> {
    pub fn send_and_wait_reply<'a, V: AtRequest + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<AtResponse, AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);
        self.writer.write(data).unwrap();

        let response = self.read_response(&mut buffer);
        if let Err(AtError::ErrorReply(isize)) = response {
            #[cfg(feature = "defmt")]
            error!("error message: {=[u8]:a}", &buffer[..isize]);
            return Err(AtError::ErrorReply(isize));
        }

        let response = payload.parse_response(&buffer);

        #[cfg(feature = "defmt")]
        info!("received response: {}", response);

        response
    }

    fn read_response(&mut self, response_out: &mut [u8; BUFFER_SIZE]) -> Result<usize, AtError> {
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 10] = [0; 10];

        loop {
            match self.reader.read(&mut read_buffer) {
                Ok(num_bytes) => {
                    for i in 0..num_bytes {
                        response_out[offset + i] = read_buffer[i];
                        // info!("{=[u8]:a}, {}", *response_out, offset + i );

                        // why is the index with + 1 and - 5?
                        if offset + i >= 5 {
                            let start = offset + i - 5;
                            let stop = offset + i + 1;
                            if response_out[start..stop] == OK_TERMINATOR {
                                return Ok(offset + i);
                            }
                            if response_out[start..stop] == ERROR_TERMINATOR {
                                return Err(AtError::ErrorReply(offset + i));
                            }
                        }
                    }
                    offset += num_bytes;
                }

                Err(_e) => {
                    #[cfg(feature = "defmt")]
                    error!("no data")
                }
            }
        }
    }
}
