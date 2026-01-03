#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use chrono::NaiveDateTime;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct Clock;

impl Clock {
    fn parse_clock_response(data: &[u8]) -> Result<NaiveDateTime, AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CCLK: ")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        // 00/01/01,00:07:50+32  // +32 means east according to datasheet. Need to understand how to interpret
        // these zone infos
        let timestamp = NaiveDateTime::parse_from_str(&parsed[..17], "%y/%m/%d,%H:%M:%S")?;

        Ok(timestamp)
    }
}

impl AtRequest for Clock {
    type Response = NaiveDateTime;
    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
