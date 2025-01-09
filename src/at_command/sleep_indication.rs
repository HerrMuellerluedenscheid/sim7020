use crate::at_command::{AtRequest, AtResponse, BufferType};
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

impl AtRequest for SleepIndicationStatus {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CPSMSTATUS")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (state,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMSTATUS: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = SleepIndication::from(state);
        Ok(AtResponse::SleepIndication(state))
    }
}
