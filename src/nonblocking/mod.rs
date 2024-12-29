use crate::at_command::{AtRequest, AtResponse};
use crate::{at_command, Modem, BUFFER_SIZE, ERROR_TERMINATOR, OK_TERMINATOR};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::Mode;
use embedded_io_async::{ErrorType, Read, Write};

#[cfg(feature = "defmt")]
use defmt::*;
use crate::at_command::cmee::ReportMobileEquipmentErrorSetting;

pub struct AsyncModem<T: Write, U: Read> {
    pub writer: T,
    pub reader: U,
}

impl<'a, T: Write, U: Read> AsyncModem<T, U> {
    pub async fn new(writer: T, reader: U) -> Self {
        let mut modem = Self { writer, reader };
        modem.disable_echo().await;
        modem
    }

    pub async fn disable_echo(&mut self) {
        self.send_and_wait_reply(at_command::ate::AtEcho {
            status: at_command::ate::Echo::Disable,
        })
        .await
        .unwrap();
    }

    pub async fn verbosity(&mut self, verbosity: ReportMobileEquipmentErrorSetting) {
        self.send_and_wait_reply(at_command::cmee::WriteReportMobileEquipmentError {
            setting: verbosity,
        })
            .await
            .unwrap();
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
        let response_size = self.read_response(&mut buffer).await?;
        #[cfg(feature = "defmt")]
        debug!("received response: {=[u8]:a}", buffer[..response_size]);
        let response = payload.parse_response(&buffer);
        #[cfg(feature = "defmt")]
        debug!("parsed response: {}", response);
        response
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

                Err(_e) => {
                    #[cfg(feature = "defmt")]
                    error!("no data")
                }
            }
        }
    }
}
