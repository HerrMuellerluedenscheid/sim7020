#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SleepIndication {
    Disabled,
    Enabled,
}

impl From<i32> for SleepIndication {
    fn from(value: i32) -> Self {
        match value {
            0 => SleepIndication::Disabled,
            1 => SleepIndication::Enabled,
            _ => {
                unreachable!()
            }
        }
    }
}

pub struct SleepIndicationStatus;

impl SleepIndicationStatus {
    fn get_status(data: &[u8]) -> Result<SleepIndication, AtError> {
        let (state,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMSTATUS: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = SleepIndication::from(state);
        return Ok(state);
    }
}

impl AtRequest for SleepIndicationStatus {
    type Response = SleepIndication;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CPSMSTATUS")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let state = Self::get_status(data)?;
        Ok(AtResponse::SleepIndication(state))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let state = Self::get_status(data)?;
        return Ok(state);
    }
}
