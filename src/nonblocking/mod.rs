//! This module contains the implementation of [AsyncModule] which is a similar struct
//! to [Module] but with async methods.
//!
//! To use this module the feature nonblocking must be enabled

use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::{at_command, AtError, BUFFER_SIZE, ERROR_TERMINATOR, OK_TERMINATOR};
use core::cell::RefCell;
use embedded_io_async::{Read, Write};

use crate::at_command::at_cpin::{EnterPIN, PINRequired, PinStatus};
use crate::at_command::cmee::ReportMobileEquipmentErrorSetting;
use crate::at_command::csclk::CSCLKMode::HardwareControlled;
use crate::at_command::csclk::{CSCLKMode, SetCSCLKMode};
#[cfg(feature = "defmt")]
use defmt::*;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
#[cfg(feature = "defmt")]
use embedded_io::Error;
use embedded_io::ReadReady;
use log::error;

use core::debug_assert;

/// Time that we will await to ensure the system has turned ON
const AWAIT_TIME_FOR_POWER_UP: u32 = 1000 * 10;

/// Modem struct that will help controlling the SIM7020 module with async methods
pub struct AsyncModem<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs> {
    /// The writer where the AT Commands will be sent
    pub writer: T,
    /// The reader where the AT Commands will be received
    pub reader: U,
    /// The pin that controls the power of the module
    pub power_pin: P,
    /// The dtr pin which is used in the PSM mode
    pub dtr_pin: P,
    /// A delay implementation that will help controlling some await times
    pub delay: D,
    /// Current sleep mode that has been configured for the module
    sleep_mode: RefCell<CSCLKMode>,
}

impl<'a, T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs> AsyncModem<T, U, P, D> {
    pub async fn new(
        writer: T,
        reader: U,
        power_pin: P,
        dtr_pin: P,
        delay: D,
    ) -> Result<Self, AtError> {
        let mut modem = Self {
            writer,
            reader,
            power_pin,
            dtr_pin,
            delay,
            sleep_mode: RefCell::new(Default::default()),
        };
        #[cfg(feature = "defmt")]
        debug!("Ensuring the power pin is ON");
        modem.power_pin.set_high().map_err(|_| AtError::HALError)?;
        // We will set the DTR pin off so we are sure that the module does not go to sleep
        modem.turn_off_dtr()?;
        #[cfg(feature = "defmt")]
        debug!(
            "Sleeping for {}ms to ensure SIM7020 has been turned on",
            AWAIT_TIME_FOR_POWER_UP
        );
        modem.delay.delay_ms(AWAIT_TIME_FOR_POWER_UP).await;
        modem.disable_echo().await?;
        Ok(modem)
    }

    /// Turns off the module
    pub fn turn_off_module(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Turning off module");
        self.power_pin.set_low().map_err(|_| AtError::HALError)
    }

    /// Turns on the module
    pub async fn turn_on_module(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Turning on module");

        self.power_pin.set_high().map_err(|_| AtError::HALError)?;

        #[cfg(feature = "defmt")]
        debug!(
            "Sleeping for {}ms to ensure SIM7020 has been turned on",
            AWAIT_TIME_FOR_POWER_UP
        );
        self.delay.delay_ms(AWAIT_TIME_FOR_POWER_UP).await;
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
    pub async fn set_sleep_mode(&mut self, mode: CSCLKMode) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Setting sleep mode {}", mode);

        // First we will ensure that the DTR pin is off, so the module does not do goes to sleep
        self.turn_off_dtr()?;
        // First we will send the AT command to ensure the sleep mode is set
        self.send_and_wait_response(SetCSCLKMode { mode }).await?;

        #[cfg(feature = "defmt")]
        debug!("Sleep mode enabled");

        self.sleep_mode.replace(mode);

        Ok(())
    }

    /// Starts the sleeping. This method is only valid if we have previously called
    /// [set_sleep_mode] with [CSCLKMode]::Hardware
    pub async fn start_sleeping(&mut self) -> Result<(), AtError> {
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
    pub async fn wake_up(&mut self) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Stopping sleeping");
        let sleep_mode = *self.sleep_mode.borrow();
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

                self.writer
                    .write_all(AT_COMMAND_TWICE)
                    .await
                    .map_err(|_| AtError::IOError)?;

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

    async fn disable_echo(&mut self) -> Result<(), AtError> {
        self.send_and_wait_response(at_command::ate::AtEcho {
            status: at_command::ate::Echo::Disable,
        })
        .await?;
        Ok(())
    }

    pub async fn verbosity(
        &mut self,
        verbosity: ReportMobileEquipmentErrorSetting,
    ) -> Result<(), AtError> {
        self.send_and_wait_response(at_command::cmee::SetReportMobileEquipmentError {
            setting: verbosity,
        })
        .await?;
        Ok(())
    }

    pub async fn send_and_wait_response<V: AtRequest + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<V::Response, crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];

        // Before we try to read we will ensure that the read buffer is empty
        #[cfg(feature = "defmt")]
        debug!("Checking if are pending bytes to read before performing the read operation");
        if self.reader.read_ready().map_err(|_e| AtError::IOError)? {
            #[cfg(feature = "defmt")]
            info!("There are some bytes pending to be read from the read, reading them before continuing");
            let _flush_read_size = self
                .reader
                .read(&mut buffer)
                .await
                .map_err(|_e| AtError::IOError)?;

            #[cfg(feature = "defmt")]
            debug!(
                "The flush read has read {} bytes. The content was: {}",
                _flush_read_size,
                buffer[.._flush_read_size]
            );
        }

        let data = payload.get_command_no_error(&mut buffer);
        #[cfg(feature = "defmt")]
        debug!("payload: {=[u8]:a}", &data);
        self.writer
            .write_all(data)
            .await
            .map_err(|_| AtError::IOError)?;
        let response_size = self.read_response(&mut buffer).await?;

        #[cfg(feature = "defmt")]
        debug!("received response: {=[u8]:a}", buffer[..response_size]);
        let response = payload.parse_response_struct(&buffer);
        #[cfg(feature = "defmt")]
        debug!("parsed response: {}", response);
        response
    }

    #[deprecated(since = "3.0.0", note = "Use the send_and_wait_response")]
    #[allow(deprecated)]
    pub async fn send_and_wait_reply<V: AtRequest + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<AtResponse, crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);
        #[cfg(feature = "defmt")]
        debug!("payload: {=[u8]:a}", &data);
        self.writer.write_all(data).await.unwrap();
        match self.read_response(&mut buffer).await {
            Ok(response_size) => {
                #[cfg(feature = "defmt")]
                debug!("received response: {=[u8]:a}", buffer[..response_size]);
                let response = payload.parse_response(&buffer);
                #[cfg(feature = "defmt")]
                debug!("parsed response: {}", response);
                response
            }
            Err(at_error) => {
                match at_error {
                    AtError::ErrorReply(response_size) => {
                        #[cfg(feature = "defmt")]
                        debug!("received response: {=[u8]:a}", buffer[..response_size]);
                    }
                    _ => {
                        #[cfg(feature = "defmt")]
                        debug!("error: {:?}", at_error);
                    }
                }

                Err(at_error)
            }
        }
    }

    pub async fn read_next_response(&mut self) -> Result<(), crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        #[cfg(feature = "defmt")]
        let response_size = self.read_response(&mut buffer).await?;
        #[cfg(feature = "defmt")]
        debug!("received response: {=[u8]:a}", buffer[..response_size]);
        Ok(())
    }

    async fn read_response(
        &mut self,
        response_out: &mut [u8; BUFFER_SIZE],
    ) -> Result<usize, crate::AtError> {
        let mut offset = 0_usize;
        let mut read_buffer: [u8; 10] = [0; 10];
        loop {
            match self.reader.read(&mut read_buffer).await {
                Ok(num_bytes) => {
                    for i in 0..num_bytes {
                        response_out[offset + i] = read_buffer[i];
                        // debug!("{=[u8]:a}, {}", *response_out, offset + i );

                        // why is the index with + 1 and - 5?
                        if offset + i < 5 {
                            continue;
                        }

                        let start = offset + i - 5;
                        let stop = offset + i + 1;

                        match &response_out[start..stop] {
                            OK_TERMINATOR => return Ok(offset + i),
                            ERROR_TERMINATOR => return Err(AtError::ErrorReply(offset + i)),
                            _ => continue,
                        }
                    }
                    offset += num_bytes;
                }

                Err(e) => {
                    #[cfg(feature = "defmt")]
                    error!("no data: {:?}", e.kind());
                }
            }
        }
    }

    /// Default max unlock tries
    const MAX_UNLOCK_TRIES: usize = 1;

    /// Try to unlock the sim card. If the sim card is already unlocked nothing will happen.
    /// If the sim card needs a PIN the provided [pin] will be used to unlock.
    /// If the status of the SIM is nor unlocked nor required PIN an [AtError]::IllegalPinStatus
    /// with the current [PinStatus] will be returned.
    /// This method will return OK only if the SIM is ready to be used
    ///
    /// You can use [max_unlock_tries] to indicate how many times you want to try to unlock
    /// the SIM. If None is passed them [MAX_UNLOCK_TRIES] will be used as max tries.
    pub async fn try_to_unlock_sim(
        &mut self,
        pin: u16,
        max_unlock_tries: Option<usize>,
    ) -> Result<(), AtError> {
        #[cfg(feature = "defmt")]
        info!("Trying to unlock SIM");

        let max_unlock_tries = max_unlock_tries.unwrap_or(Self::MAX_UNLOCK_TRIES);

        debug_assert!(max_unlock_tries > 0, "We need at least one try to unlock");

        // First we need to check for the PIN status
        let current_pin_status = self.send_and_wait_response(PINRequired).await?;

        #[cfg(feature = "defmt")]
        debug!("current pin status: {}", current_pin_status);

        let mut unlock_tries = 0;

        loop {
            // First we need to check the status of the pin
            match current_pin_status {
                PinStatus::Ready => {
                    #[cfg(feature = "defmt")]
                    info!("SIM is already unlocked returning");

                    return Ok(());
                }
                PinStatus::SimPin => {
                    #[cfg(feature = "defmt")]
                    debug!("SIM pin is required")
                }
                _ => {
                    // This is a state the method can not handle
                    #[cfg(feature = "defmt")]
                    warn!("SIM status can no be handled. {}", current_pin_status);
                    return Err(AtError::IllegalPinStatus(current_pin_status));
                }
            }

            // If we already do the maximum unlock tries return an error with the current
            // status
            if unlock_tries >= Self::MAX_UNLOCK_TRIES {
                #[cfg(feature = "defmt")]
                return Err(AtError::IllegalPinStatus(current_pin_status));
            }

            // We have at least 1 try to unlock the pin
            #[cfg(feature = "defmt")]
            debug!("Trying to unlock SIM with the provided pin");

            self.send_and_wait_response(EnterPIN { pin }).await?;

            unlock_tries += 1;
        }
    }
}
