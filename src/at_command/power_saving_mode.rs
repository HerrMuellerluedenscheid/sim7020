#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
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

impl GetPowerSavingMode {
    fn parse_state(data: &[u8]) -> Result<PowerSavingModeState, AtError> {
        let (state,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMS: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PowerSavingModeState::from(state);

        Ok(state)
    }
}

impl AtRequest for GetPowerSavingMode {
    type Response = PowerSavingModeState;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CPSMS")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let state = Self::parse_state(data)?;
        Ok(AtResponse::PowerSavingMode(state))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let state = Self::parse_state(data)?;
        Ok(state)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetPowerSavingMode;

impl SetPowerSavingMode {
    fn parse_state(data: &[u8]) -> Result<PowerSavingModeState, AtError> {
        let (state, _context) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CPSMS: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PowerSavingModeState::from(state);

        Ok(state)
    }
}

impl AtRequest for SetPowerSavingMode {
    type Response = PowerSavingModeState;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CPSMS")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let state = Self::parse_state(data)?;
        Ok(AtResponse::PowerSavingMode(state))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let state = Self::parse_state(data)?;
        Ok(state)
    }
}
