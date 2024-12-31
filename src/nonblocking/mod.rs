use crate::at_command::{AtRequest, AtResponse};
use crate::{at_command, AtError, Modem, BUFFER_SIZE, ERROR_TERMINATOR, OK_TERMINATOR};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::Mode;
use embedded_io_async::{ErrorType, Read, Write};

use crate::at_command::at_creg::AtCregError;
use crate::at_command::cmee::ReportMobileEquipmentErrorSetting;
#[cfg(feature = "defmt")]
use defmt::*;
use embedded_io::{Error, ErrorKind};
use log::error;

pub struct AsyncModem<T: Write, U: Read> {
    pub writer: T,
    pub reader: U,
}

impl<'a, T: Write, U: Read> AsyncModem<T, U> {
    pub async fn new(writer: T, reader: U) -> Result<Self, AtError> {
        let mut modem = Self { writer, reader };
        modem.disable_echo().await?;
        Ok(modem)
    }

    async fn disable_echo(&mut self) -> Result<AtResponse, AtError> {
        self.send_and_wait_reply(at_command::ate::AtEcho {
            status: at_command::ate::Echo::Disable,
        })
        .await
    }

    pub async fn verbosity(
        &mut self,
        verbosity: ReportMobileEquipmentErrorSetting,
    ) -> Result<AtResponse, AtError> {
        self.send_and_wait_reply(at_command::cmee::WriteReportMobileEquipmentError {
            setting: verbosity,
        })
        .await
    }

    pub async fn send_and_wait_reply<V: AtRequest + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<AtResponse, crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);
        #[cfg(feature = "defmt")]
        debug!("payload: {=[u8]:a}", &data);
        self.writer.write(data).await.unwrap();
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

    pub async fn read_next_response(&mut self) -> Result<AtResponse, crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        #[cfg(feature = "defmt")]
        let response_size = self.read_response(&mut buffer).await?;
        #[cfg(feature = "defmt")]
        debug!("received response: {=[u8]:a}", buffer[..response_size]);
        Ok(AtResponse::Ok)
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
                            ERROR_TERMINATOR => return Err(crate::AtError::ErrorReply(offset + i)),
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
}
