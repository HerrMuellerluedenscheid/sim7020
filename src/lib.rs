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
use embedded_io::{Error, ErrorKind};
use crate::at_command::cmee::ReportMobileEquipmentErrorSetting;
use crate::at_command::flow_control::FlowControl;

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
    MqttFailure,
    NotReady,
    IOError,
    AtParseError,
}

impl From<ParseError> for AtError {
    fn from(_: ParseError) -> AtError {
        AtError::AtParseError
    }
}

impl<'a, T: Write, U: Read> Modem<'a, T, U> {
    pub fn new(writer: &'a mut T, reader: &'a mut U) -> Result<Self, AtError> {
        let mut modem = Self { writer, reader };
        modem.disable_echo()?;
        // modem.get_flow_control().expect("failed to get flow control");
        Ok(modem)
    }

    /// disable echo if echo is enabled
    pub fn disable_echo(&mut self) -> Result<(), AtError>{
        // info!("check echo enabled");
        // self
        //     .send_and_wait_reply(&at_command::ate::AtEchoState {})?;
        info!("Disable echo");
        self
            .send_and_wait_reply(&at_command::ate::AtEcho {
                status: at_command::ate::Echo::Disable,
            })?;
        Ok(())
    }

    pub fn enable_numeric_errors(&mut self) -> Result<(), AtError> {
        info!("enable numeric errors");
        self
            .send_and_wait_reply(&at_command::cmee::SetReportMobileEquipmentError{setting: ReportMobileEquipmentErrorSetting::EnabledVerbose})?;
        Ok(())
    }

    pub fn get_flow_control(&mut self) -> Result<(), AtError>{
        info!("Get flow control");
        self.send_and_wait_reply(&at_command::flow_control::GetFlowControl {}).expect("TODO: panic message");
        Ok(())
    }

    pub fn set_flow_control(&mut self) -> Result<(), AtError>{
        info!("Set flow control to software");
        self.send_and_wait_reply(
            &at_command::flow_control::SetFlowControl{
                ta_to_te: FlowControl::Software,
                te_to_ta: FlowControl::Software }).expect("TODO: panic message");
        Ok(())
    }

        /// probe the modem's readiness by sending 'AT'. Errors if not ready.
    pub fn ready(&mut self) -> Result<(), AtError>{
        #[cfg(feature = "defmt")]
        info!("probing modem readiness");
        self
            .send_and_wait_reply(&at_command::at::At {})?;
        Ok(())
    }

    pub fn send_and_wait_reply<'b, V: AtRequest + 'b>(
        &'b mut self,
        payload: &V,
    ) -> Result<AtResponse, AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);

        #[cfg(feature = "defmt")]
        debug!("sending command: {=[u8]:a}", data);
        self.writer.write(&data).map_err(|e| AtError::IOError)?;

        let mut read_buffer = [0; BUFFER_SIZE];
        let response_size = self.read_response(&mut read_buffer)?;
        let response = payload.parse_response(&read_buffer[..response_size]);
        match response {
            Ok(response) => {Ok(response)},
            Err(e) => {
                #[cfg(feature = "defmt")]
                error!("{}\nparse response failed on request: {=[u8]:a}\n response: {=[u8]:a}", e, &data,  &read_buffer[..response_size]);
                // Err(AtError::AtParseError)
                Ok(AtResponse::Ok{})
            }
        }
    }

    pub fn read_response(&mut self, response_out: &mut [u8; BUFFER_SIZE]) -> Result<usize, AtError> {
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 100] = [0; 100];
        loop{
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
                            OK_TERMINATOR => return {
                                debug!("OK terminated: {=[u8]:a}", response_out[..offset + i + 5]);
                                Ok(offset + i)

                            },
                            ERROR_TERMINATOR => {
                                #[cfg(feature = "defmt")]
                                error!("received ERROR response: {=[u8]:a}", response_out[..offset + i + 5]);
                                return Err(AtError::ErrorReply(offset + i))
                            },
                            _ => {
                                #[cfg(feature = "defmt")]
                                continue
                            },
                        }
                        info!("..");
                    }

                    offset += num_bytes;
                }

                Err(e) => {
                    #[cfg(feature = "defmt")]
                    error!("uart error {}", e.kind());
                    return Err(AtError::NotReady)
                }
            }
        }
    }
}
