use crate::at_command::receive::UnsolicitedMessage;
use crate::nonblocking::AsyncModem;
use crate::{AtError, BUFFER_SIZE};
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_io_async::{Read, ReadReady, Write};

impl<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs> AsyncModem<T, U, P, D> {
    /// If there are data available in the reception buffer it will be read and parsed.
    /// If there is no data a None will be returned.
    /// A blocking implementation that will wait until there is data in the buffer is [get_unsolicited_message]
    /// *Warning* If there is data related to another command in the buffer this method will fail.
    pub async fn try_get_unsolicited_message<M: UnsolicitedMessage>(
        &mut self,
    ) -> Result<Option<M>, AtError> {
        if !self.reader.read_ready().map_err(|_| AtError::IOError)? {
            #[cfg(feature = "defmt")]
            debug!("There is not data available in the reader");
            return Ok(None);
        }

        let message = self.get_unsolicited_message().await?;

        Ok(Some(message))
    }

    /// Reads from the reception buffer. Will block until there is data.
    /// If you need a non-blocking implementation [try_get_unsolicited_message]
    /// *Warning* If there is data related to another command in the buffer this method will fail.
    pub async fn get_unsolicited_message<M: UnsolicitedMessage>(&mut self) -> Result<M, AtError> {
        let mut buffer = [0u8; BUFFER_SIZE];

        let bytes = self
            .reader
            .read(&mut buffer)
            .await
            .map_err(|_| AtError::IOError)?;

        let result = M::decode(&buffer[..bytes])?;

        Ok(result)
    }
}
