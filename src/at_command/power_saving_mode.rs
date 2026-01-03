//! Commands to handle the power saving modes
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

/// The power saving modes
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
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

/// Command to get the current power saving mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GetPowerSavingMode;

impl GetPowerSavingMode {
    fn parse_state(data: &[u8]) -> Result<PowerSavingModeState, AtError> {
        let (state,) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CPSMS: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        let state = PowerSavingModeState::from(state);

        Ok(state)
    }
}

impl AtRequest for GetPowerSavingMode {
    type Response = PowerSavingModeState;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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
#[derive(PartialEq, Clone)]
pub struct SetPowerSavingMode;

impl SetPowerSavingMode {
    fn parse_state(data: &[u8]) -> Result<PowerSavingModeState, AtError> {
        let (state, _context) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CPSMS: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        let state = PowerSavingModeState::from(state);

        Ok(state)
    }
}

impl AtRequest for SetPowerSavingMode {
    type Response = PowerSavingModeState;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn power_saving_mode_state_from_int() {
        assert_eq!(
            PowerSavingModeState::from(0),
            PowerSavingModeState::Disabled
        );
        assert_eq!(PowerSavingModeState::from(1), PowerSavingModeState::Enabled);
        assert_eq!(PowerSavingModeState::from(2), PowerSavingModeState::Discard);
    }

    #[test]
    fn get_power_saving_mode_get_command() {
        let cmd = GetPowerSavingMode;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CPSMS?\r\n");
    }

    #[test]
    fn get_power_saving_mode_parse_disabled() {
        let data = b"\r\n+CPSMS: 0\r\nOK\r\n";

        let state = GetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Disabled);
    }

    #[test]
    fn get_power_saving_mode_parse_enabled() {
        let data = b"\r\n+CPSMS: 1\r\nOK\r\n";

        let state = GetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Enabled);
    }

    #[test]
    fn get_power_saving_mode_parse_discard() {
        let data = b"\r\n+CPSMS: 2\r\nOK\r\n";

        let state = GetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Discard);
    }

    #[test]
    fn set_power_saving_mode_get_command() {
        let cmd = SetPowerSavingMode;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CPSMS=\r\n");
    }

    #[test]
    fn set_power_saving_mode_parse_disabled() {
        let data = b"\r\n+CPSMS: 0,1\r\nOK\r\n";

        let state = SetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Disabled);
    }

    #[test]
    fn set_power_saving_mode_parse_enabled() {
        let data = b"\r\n+CPSMS: 1,0\r\nOK\r\n";

        let state = SetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Enabled);
    }

    #[test]
    fn set_power_saving_mode_parse_discard() {
        let data = b"\r\n+CPSMS: 2,5\r\nOK\r\n";

        let state = SetPowerSavingMode.parse_response_struct(data).unwrap();

        assert_eq!(state, PowerSavingModeState::Discard);
    }

    #[test]
    #[should_panic]
    fn power_saving_mode_parse_invalid_state() {
        let data = b"\r\n+CPSMS: 9\r\nOK\r\n";

        GetPowerSavingMode.parse_response_struct(data).unwrap();
    }
}
