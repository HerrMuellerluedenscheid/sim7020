use crate::at_command::{AtRequest, AtResponse};
use crate::{Modem, BUFFER_SIZE, ERROR_TERMINATOR, OK_TERMINATOR};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::Mode;
use embedded_io_async::{ErrorType, Read, Write};

#[cfg(feature = "defmt")]
use defmt::*;

pub struct AsyncModem<T: Write, U: Read> {
    pub writer: T,
    pub reader: U,
}

impl<T: Write, U: Read> AsyncModem<T, U> {
    pub async fn send_and_wait_reply<'a, V: AtRequest + 'a>(
        &'a mut self,
        payload: V,
    ) -> Result<AtResponse, crate::AtError> {
        let mut buffer = [0; BUFFER_SIZE];
        let data = payload.get_command_no_error(&mut buffer);
        self.writer.write(data).await.unwrap();
        let response = self.read_response(&mut buffer).await;
        if let Err(crate::AtError::ErrorReply(isize)) = response {
            return Err(crate::AtError::ErrorReply(isize));
        }

        let response = payload.parse_response(&buffer);
        #[cfg(feature = "defmt")]
        info!("received response: {}", response);
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
                        // info!("{=[u8]:a}, {}", *response_out, offset + i );

                        // why is the index with + 1 and - 5?
                        if offset + i >= 5 {
                            let start = offset + i - 5;
                            let stop = offset + i + 1;
                            if response_out[start..stop] == OK_TERMINATOR {
                                return Ok(offset + i);
                            }
                            if response_out[start..stop] == ERROR_TERMINATOR {
                                return Err(crate::AtError::ErrorReply(offset + i));
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
