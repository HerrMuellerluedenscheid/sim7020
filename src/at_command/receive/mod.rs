//! This module contains the AT-Commands which are unsolicited.

#[cfg(feature = "nonblocking")]
pub mod nonblocking;

pub mod socket_reception;

use crate::{AtError, Modem, BUFFER_SIZE};
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, ReadReady, Write};

/// Type of AT command which is unsolicited
pub trait UnsolicitedMessage: Sized {
    /// Decodes the data and returns the result
    fn decode(data: &[u8]) -> Result<Self, AtError>;
}

impl<'a, T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs> Modem<'a, T, U, P, D> {
    /// If there are data available in the reception buffer it will be read and parsed.
    /// If there is no data a None will be returned.
    /// A blocking implementation that will wait until there is data in the buffer is [get_unsolicited_message]
    /// *Warning* If there is data related to another command in the buffer this method will fail.
    pub fn try_get_unsolicited_message<M: UnsolicitedMessage>(
        &mut self,
    ) -> Result<Option<M>, AtError> {
        if !self.reader.read_ready().map_err(|_| AtError::IOError)? {
            #[cfg(feature = "defmt")]
            debug!("There is not data available in the reader");
            return Ok(None);
        }

        let message = self.get_unsolicited_message()?;

        Ok(Some(message))
    }

    /// Reads from the reception buffer. Will block until there is data.
    /// If you need a non-blocking implementation [try_get_unsolicited_message]
    /// *Warning* If there is data related to another command in the buffer this method will fail.
    pub fn get_unsolicited_message<M: UnsolicitedMessage>(&mut self) -> Result<M, AtError> {
        let mut buffer = [0u8; BUFFER_SIZE];

        let bytes = self
            .reader
            .read(&mut buffer)
            .map_err(|_| AtError::IOError)?;

        let result = M::decode(&buffer[..bytes])?;

        Ok(result)
    }
}
