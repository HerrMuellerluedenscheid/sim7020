use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::{error, info};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct At {}

impl AtRequest for At {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        Ok("AT\r\n".as_bytes())
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("parse_response {=[u8]:a}", data);
        let (_matready, _cfun) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"AT\r\r\n")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\n")
            .expect_raw_string()
            .expect_identifier(b"\r\nAT\r\r\nOK\r")
            .finish()
            .inspect_err(|_e| {
                #[cfg(feature = "defmt")]
                error!("Failed to parse response: {=[u8]:a}", data);
            })?;
        #[cfg(feature = "defmt")]
        info!("matready: {} | cfun: {}", _matready, _cfun);
        Ok(AtResponse::Ok)
    }
}
