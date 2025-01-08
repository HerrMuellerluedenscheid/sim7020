use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use chrono::NaiveDateTime;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StartQueryNTP<'a> {
    pub url: &'a str,
    pub tzinfo: Option<i8>,
}

impl AtRequest for StartQueryNTP<'_> {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSNTPSTART")
            .with_string_parameter(self.url)
            .with_optional_int_parameter(self.tzinfo)
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StopQueryNTP;

impl AtRequest for StopQueryNTP {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CSNTPSTOP")
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NTPTime {}

impl AtRequest for NTPTime {
    type Response = Result<(), AtError>;
    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CCLK")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CCLK: ")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        // 00/01/01,00:07:50+32  // +32 means east according to datasheet. Need to understand how to interpret
        // these zone infos
        let timestamp = NaiveDateTime::parse_from_str(&parsed[..17], "%y/%m/%d,%H:%M:%S").unwrap();
        Ok(AtResponse::NTPTimestamp(timestamp.and_utc().timestamp()))
    }
}
