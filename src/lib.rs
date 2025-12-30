// make `std` available when testing
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

pub mod at_command;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

use core::cell::RefCell;
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
use embedded_hal::digital::OutputPin;
use embedded_hal::delay::DelayNs;
#[cfg(feature = "defmt")]
use embedded_io::Error;
pub use embedded_io::{Read, Write};
use crate::at_command::csclk::{CSCLKMode, SetCSCLKMode};
use crate::at_command::csclk::CSCLKMode::HardwareControlled;

const BUFFER_SIZE: usize = 512;
const LF: u8 = 10; // n
const CR: u8 = 13; // r

const OK_TERMINATOR: &[u8] = &[CR, LF, b'O', b'K', CR, LF];
const ERROR_TERMINATOR: &[u8] = &[b'R', b'R', b'O', b'R', CR, LF];

pub struct Modem<'a, T: Write, U: Read, P, D> {
    pub writer: &'a mut T,
    pub reader: &'a mut U,
    /// The pin that controls the power of the module
    pub power_pin: P,
    /// The dtr pin which is used in the PSM mode
    pub dtr_pin: P,
    /// A delay implementation that will help controlling some await times
    pub delay: D,
    /// Current sleep mode that has been configured for the module
    sleep_mode: RefCell<CSCLKMode>
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
    HALError,
    IllegalModuleState
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


impl<'a, T: Write, U: Read,  P: OutputPin, D: DelayNs> Modem<'a, T, U, P, D> {
    /// Time that we will await to ensure the system has turned ON
    const AWAIT_TIME_FOR_POWER_UP: u32 = 1000 * 10;
    
    pub fn new(writer: &'a mut T, reader: &'a mut U, power_pin: P, dtr_pin: P, delay: D) -> Result<Self, AtError> {
        let mut modem = Self { writer, reader, power_pin, dtr_pin, delay, sleep_mode: RefCell::new(Default::default()) };
        #[cfg(feature = "defmt")]
        debug!("Ensuring the power pin is ON");
        modem.power_pin.set_high().map_err(|_| AtError::HALError)?;
        // We will set the DTR pin off so we are sure that the module does not go to sleep
        modem.turn_off_dtr()?;
        #[cfg(feature = "defmt")]
        debug!("Sleeping for {}ms to ensure SIM7020 has been turned on", Self::AWAIT_TIME_FOR_POWER_UP);
        modem.delay.delay_ms(Self::AWAIT_TIME_FOR_POWER_UP);
        modem.disable_echo()?;
        Ok(modem)
    }

    /// Turns off the module
    pub fn turn_off_module(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Turning off module");
        self.power_pin.set_low().map_err(|_| AtError::HALError)
    }

    /// Turns on the module
    pub fn turn_on_module(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Turning on module");

        self.power_pin.set_high().map_err(|_| AtError::HALError)?;

        #[cfg(feature = "defmt")]
        debug!("Sleeping for {}ms to ensure SIM7020 has been turned on", Self::AWAIT_TIME_FOR_POWER_UP);
        self.delay.delay_ms(Self::AWAIT_TIME_FOR_POWER_UP);
        Ok(())
    }

    /// Turns off the DTR pin
    #[inline]
    fn turn_off_dtr(&mut self) -> Result<(), AtError> {
        self.dtr_pin.set_low().map_err(|_| AtError::HALError)
    }

    /// Turns on the DTR pin
    #[inline]
    fn turn_on_dtr(&mut self) -> Result<(), AtError> {
        self.dtr_pin.set_high().map_err(|_| AtError::HALError)
    }

    /// Sets the sleep mode of the module to indicated with [mode]
    pub fn set_sleep_mode(&mut self, mode: CSCLKMode) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Setting sleep mode {}", mode);

        // First we will ensure that the DTR pin is off, so the module does not do goes to sleep
        self.turn_off_dtr()?;
        // First we will send the AT command to ensure the sleep mode is set
        self.send_and_wait_response(&SetCSCLKMode {
            mode
        })?;

        #[cfg(feature = "defmt")]
        debug!("Sleep mode enabled");

        self.sleep_mode.replace(mode);

        Ok(())
    }

    /// Starts the sleeping. This method is only valid if we have previously called
    /// [set_sleep_mode] with [CSCLKMode]::Hardware
    pub fn start_sleeping(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Starting sleeping");

        if *self.sleep_mode.borrow() != HardwareControlled {
            #[cfg(feature = "defmt")]
            warn!("The sleep mode has not been enabled");

            return Err(AtError::IllegalModuleState);
        }

        self.turn_on_dtr()
    }

    /// Wakes up the sim module depending on the configuration.
    /// If the module is not configured for sleep will do nothing (can be configured using [set_sleep_mode].
    /// If the module sleep is configured in software mode two AT commands will be sent to wake up.
    /// If the module sleep is configured in hardware mode the pin will be pulled off.
    pub fn wake_up(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Stopping sleeping");
        let sleep_mode = self.sleep_mode.borrow().clone();
        match sleep_mode {
            CSCLKMode::Disabled => {
                #[cfg(feature = "defmt")]
                debug!("The sleep mode is enabled, nothing to do");
                Ok(())
            }
            CSCLKMode::SoftwareControlled => {
                #[cfg(feature = "defmt")]
                debug!("Waking up from software");
                // According to the manual we need to send twice any AT command to wake up the module
                // when is configured in software mode
                const AT_COMMAND_TWICE: &[u8] = b"AT\r\nAT\r\n";

                self.writer.write_all(&AT_COMMAND_TWICE).map_err(|_| AtError::IOError)?;

                Ok(())

            }

            CSCLKMode::HardwareControlled => {
                #[cfg(feature = "defmt")]
                debug!("Waking up from hardware");

                // Just pull off the DTR pin
                self.turn_off_dtr()
            }

        }
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
