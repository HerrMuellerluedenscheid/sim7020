#![no_std]
#![no_main]

pub mod at_command;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

use crate::at_command::http::HttpClient;
use at_command::AtRequest;
use at_command::AtResponse;
use at_commands::parser::ParseError;
use core::ptr::read;
#[cfg(feature = "defmt")]
use defmt::*;
pub use embedded_io::{ErrorType, Read, Write};

const BUFFER_SIZE: usize = 512;
const LF: u8 = 10; // n
const CR: u8 = 13; // r

const OK_TERMINATOR: &[u8] = &[CR, LF, b'O', b'K', CR, LF];
const ERROR_TERMINATOR: &[u8] = &[b'R', b'R', b'O', b'R', CR, LF];

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

impl<'a, T: Write, U: Read> Modem<'a, T, U> {
    pub fn new(writer: &'a mut T, reader: &'a mut U) -> Self {
        let mut modem = Self { writer, reader };
        // let modem = modem.disable_echo();  // todo
        modem
    }

    // fn disable_echo(mut self) -> Self{
    //     self
    //         .send_and_wait_reply(at_command::ate::AtEcho {
    //             status: at_command::ate::Echo::Disable,
    //         })
    //         .unwrap();
    //     self
    // }

    pub fn send_and_wait_reply<'b, V: AtRequest + 'b>(
        &'b mut self,
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
                        if offset + i < 5 {
                            continue;
                        }

                        let start = offset + i - 5;
                        let stop = offset + i + 1;

                        match &response_out[start..stop] {
                            OK_TERMINATOR => return Ok(offset + i),
                            ERROR_TERMINATOR => return Err(crate::AtError::ErrorReply(offset + i)),
                            _ => continue,
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
