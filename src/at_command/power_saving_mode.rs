use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerSavingModeState {
    Disabled,
    Enabled,
    Discard,
}

impl From<i32> for PowerSavingModeState {
    fn from(value: i32) -> Self {
        match value {
            0 => PowerSavingModeState::Disabled,
            1 => PowerSavingModeState::Enabled,
            2 => PowerSavingModeState::Discard,
            _ => {
                unreachable!()
            }
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetPowerSavingMode;

impl AtRequest for GetPowerSavingMode {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CPSMS")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (state,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMS: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PowerSavingModeState::from(state);
        Ok(AtResponse::PowerSavingMode(state))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetPowerSavingMode;

impl AtRequest for SetPowerSavingMode {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CPSMS")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (state, context) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMS: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PowerSavingModeState::from(state);
        Ok(AtResponse::PowerSavingMode(state))
    }
}
