//! Module to handle the AT echo
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

/// Echo status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum EchoState {
    Disabled,
    Enabled,
}

/// The echo state
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum Echo {
    Disable = 0,
    Enable = 1,
}

/// Struct to query the echo state
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtEchoState;

impl AtRequest for AtEchoState {
    type Response = ();

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let command = "ATE?\r\n";
        Ok(command.as_bytes())
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Struct to set the echo state
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtEcho {
    pub status: Echo,
}

impl AtRequest for AtEcho {
    type Response = ();

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let command = match self.status {
            Echo::Disable => "ATE0\r\n",
            Echo::Enable => "ATE1\r\n",
        };
        Ok(command.as_bytes())
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn echo_enum_values() {
        assert_eq!(Echo::Disable as u8, 0);
        assert_eq!(Echo::Enable as u8, 1);
    }

    #[test]
    fn echo_state_enum_values() {
        assert_eq!(EchoState::Disabled as u8, 0);
        assert_eq!(EchoState::Enabled as u8, 1);
    }

    #[test]
    fn at_echo_state_get_command() {
        let mut buffer = [0u8; 512];

        let result = AtEchoState.get_command(&mut buffer).unwrap();

        assert_eq!(result, b"ATE?\r\n");
    }

    #[test]
    fn at_echo_state_parse_response() {
        let response = AtEchoState.parse_response_struct(b"OK\r\n");

        assert!(response.is_ok());
    }

    #[test]
    fn at_echo_disable_command() {
        let cmd = AtEcho {
            status: Echo::Disable,
        };
        let mut buffer = [0u8; 512];

        let result = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(result, b"ATE0\r\n");
    }

    #[test]
    fn at_echo_enable_command() {
        let cmd = AtEcho {
            status: Echo::Enable,
        };
        let mut buffer = [0u8; 512];

        let result = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(result, b"ATE1\r\n");
    }

    #[test]
    fn at_echo_parse_response() {
        let cmd = AtEcho {
            status: Echo::Enable,
        };

        let response = cmd.parse_response_struct(b"OK\r\n");

        assert!(response.is_ok());
    }
}
