//! Commands to handle the sleep indication
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

/// States of the sleep indication
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
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

/// Request to get the sleep indication status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SleepIndicationStatus;

impl SleepIndicationStatus {
    fn get_status(data: &[u8]) -> Result<SleepIndication, AtError> {
        let (state,) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CPSMSTATUS: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        let state = SleepIndication::from(state);
        Ok(state)
    }
}

impl AtRequest for SleepIndicationStatus {
    type Response = SleepIndication;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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
        Ok(state)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sleep_indication_from_int() {
        assert_eq!(SleepIndication::from(0), SleepIndication::Disabled);
        assert_eq!(SleepIndication::from(1), SleepIndication::Enabled);
    }

    #[test]
    fn sleep_indication_status_get_command() {
        let cmd = SleepIndicationStatus;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CPSMSTATUS?\r\n");
    }

    #[test]
    fn sleep_indication_status_parse_disabled() {
        let data = b"\r\n+CPSMSTATUS: 0\r\nOK\r\n";

        let state = SleepIndicationStatus.parse_response_struct(data).unwrap();

        assert_eq!(state, SleepIndication::Disabled);
    }

    #[test]
    fn sleep_indication_status_parse_enabled() {
        let data = b"\r\n+CPSMSTATUS: 1\r\nOK\r\n";

        let state = SleepIndicationStatus.parse_response_struct(data).unwrap();

        assert_eq!(state, SleepIndication::Enabled);
    }

    #[test]
    #[should_panic]
    fn sleep_indication_status_parse_invalid_state() {
        let data = b"\r\n+CPSMSTATUS: 9\r\nOK\r\n";
        SleepIndicationStatus.parse_response_struct(data).unwrap();
    }
}
