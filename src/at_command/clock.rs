//! Module to handle clock commands

use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;
use chrono::NaiveDateTime;

/// Request the current clock
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct Clock;

impl Clock {
    fn parse_clock_response(data: &[u8]) -> Result<NaiveDateTime, AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CCLK: ")
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        // 00/01/01,00:07:50+32  // +32 means east according to datasheet. Need to understand how to interpret
        // these zone infos
        let timestamp = NaiveDateTime::parse_from_str(&parsed[..17], "%y/%m/%d,%H:%M:%S")?;

        Ok(timestamp)
    }
}

impl AtRequest for Clock {
    type Response = NaiveDateTime;
    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CCLK")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let timestamp = Self::parse_clock_response(data)?;
        Ok(AtResponse::NTPTimestamp(timestamp.and_utc().timestamp()))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let timestamp = Self::parse_clock_response(data)?;
        Ok(timestamp)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn clock_get_command() {
        let cmd = Clock;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CCLK?\r\n");
    }
    #[test]
    fn clock_parse_valid_response() {
        let cmd = Clock;

        let data = b"\r\n+CCLK: 24/01/02,13:45:59+32\r\n\r\nOK";

        let timestamp = cmd.parse_response_struct(data).unwrap();

        let expected = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveTime::from_hms_opt(13, 45, 59).unwrap(),
        );

        assert_eq!(timestamp, expected);
    }
}
