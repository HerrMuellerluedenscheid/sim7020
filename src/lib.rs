// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

pub mod at_command;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

pub mod contexts;

use crate::at_command::flow_control::ControlFlowStatus;
use crate::at_command::http::HttpClient;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{
    cmee::ReportMobileEquipmentErrorSetting, flow_control::GetFlowControlResponse,
};
use at_command::AtRequest;
use at_commands::parser::ParseError;
#[cfg(feature = "defmt")]
use defmt::*;
#[cfg(feature = "defmt")]
use embedded_io::Error;
pub use embedded_io::{Read, Write};

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
    ConnectSocketError,
    CapacityError,
    ParseClockError,
}

impl From<ParseError> for AtError {
    fn from(_: ParseError) -> AtError {
        AtError::AtParseError
    }
}

impl From<heapless::CapacityError> for AtError {
    fn from(_: heapless::CapacityError) -> Self {
        AtError::CapacityError
    }
}

impl From<chrono::format::ParseError> for AtError {
    fn from(_: chrono::format::ParseError) -> Self {
        AtError::ParseClockError
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
    pub fn disable_echo(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Disable echo");
        self.send_and_wait_response(&at_command::ate::AtEcho {
            status: at_command::ate::Echo::Disable,
        })?;
        Ok(())
    }

    pub fn enable_numeric_errors(&mut self) -> Result<(), AtError> {
        self.send_and_wait_response(&at_command::cmee::SetReportMobileEquipmentError {
            setting: ReportMobileEquipmentErrorSetting::EnabledVerbose,
        })?;
        Ok(())
    }

    pub fn get_flow_control(&mut self) -> Result<GetFlowControlResponse, AtError> {
        let result = self.send_and_wait_response(&at_command::flow_control::GetFlowControl {})?;
        Ok(result)
    }

    pub fn set_flow_control(&mut self) -> Result<(), AtError> {
        self.send_and_wait_response(&at_command::flow_control::SetFlowControl {
            ta_to_te: ControlFlowStatus::Software,
            te_to_ta: ControlFlowStatus::Software,
        })?;
        Ok(())
    }

    /// probe the modem's readiness by sending 'AT'. Errors if not ready.
    pub fn ready(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("probing modem readiness");
        self.send_and_wait_response(&at_command::at::At {})?;
        Ok(())
    }

    pub fn send_and_wait_response<'b, V: AtRequest + 'b>(
        &'b mut self,
        payload: &V,
    ) -> Result<V::Response, AtError> {
        #[cfg(feature = "defmt")]
        info!("Sending command to the modem");

        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);

        #[cfg(feature = "defmt")]
        debug!("sending command: {=[u8]:a}", data);

        self.writer.write_all(data).map_err(|_e| AtError::IOError)?;

        let mut read_buffer = [0; BUFFER_SIZE];
        let response_size = self.read_response(&mut read_buffer)?;
        let response = payload.parse_response_struct(&read_buffer[..response_size])?;

        Ok(response)
    }

    #[deprecated(since = "3.0.0", note = "Use the send_and_wait_response")]
    #[allow(deprecated)]
    pub fn send_and_wait_reply<'b, V: AtRequest + 'b>(
        &'b mut self,
        payload: &V,
    ) -> Result<AtResponse, AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);

        #[cfg(feature = "defmt")]
        debug!("sending command: {=[u8]:a}", data);
        self.writer.write_all(data).map_err(|_e| AtError::IOError)?;

        let mut read_buffer = [0; BUFFER_SIZE];
        let response_size = self.read_response(&mut read_buffer)?;
        let response = payload.parse_response(&read_buffer[..response_size]);
        match response {
            Ok(response) => Ok(response),
            Err(_e) => {
                #[cfg(feature = "defmt")]
                error!(
                    "{}\nparse response failed on request: {=[u8]:a}\n response: {=[u8]:a}",
                    _e,
                    &data,
                    &read_buffer[..response_size]
                );
                Ok(AtResponse::Ok {})
            }
        }
    }

    pub fn read_response(
        &mut self,
        response_out: &mut [u8; BUFFER_SIZE],
    ) -> Result<usize, AtError> {
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 100] = [0; 100];
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
                            OK_TERMINATOR => {
                                return {
                                    #[cfg(feature = "defmt")]
                                    trace!(
                                        "OK terminated: {=[u8]:a}",
                                        response_out[..offset + i + 5]
                                    );
                                    Ok(offset + i)
                                }
                            }
                            ERROR_TERMINATOR => {
                                #[cfg(feature = "defmt")]
                                error!(
                                    "received ERROR response: {=[u8]:a}",
                                    response_out[..offset + i + 5]
                                );
                                return Err(AtError::ErrorReply(offset + i));
                            }
                            _ =>
                            {
                                #[cfg(feature = "defmt")]
                                continue
                            }
                        }
                    }

                    offset += num_bytes;
                }

                Err(_e) => {
                    #[cfg(feature = "defmt")]
                    error!("uart error {}", _e.kind());
                    return Err(AtError::NotReady);
                }
            }
        }
    }
}
