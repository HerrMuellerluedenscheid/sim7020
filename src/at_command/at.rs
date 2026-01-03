//! Module to handle the basic AT commands

use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::{error, info};

/// Struct to send the AT request and verify AT is OK
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct At;

impl At {
    fn get_command_response(data: &[u8]) -> Result<(&str, &str), AtError> {
        let tuple = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
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

        Ok(tuple)
    }
}

impl AtRequest for At {
    type Response = ();

    fn get_command<'a>(&'a self, _buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        Ok("AT\r\n".as_bytes())
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("parse_response {=[u8]:a}", data);
        let (_matready, _cfun) = Self::get_command_response(data)?;
        #[cfg(feature = "defmt")]
        info!("matready: {} | cfun: {}", _matready, _cfun);
        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        info!("parse_response {=[u8]:a}", data);
        let (_matready, _cfun) = Self::get_command_response(data)?;
        #[cfg(feature = "defmt")]
        info!("matready: {} | cfun: {}", _matready, _cfun);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_command_request() {
        let mut buffer = [0u8; 512];

        let data = At.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT\r\n");
    }
}
