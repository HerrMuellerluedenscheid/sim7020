//! Contains the source code that allows controlling AT commands for CSCLK

use crate::at_command::{verify_ok, AtRequest, BufferType};
use crate::AtError;

/// The modes CSLK can be controlled
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Eq, Copy, Clone, Default)]
#[repr(u8)]
pub enum CSCLKMode {
    /// The module does not go to sleep
    #[default]
    Disabled = 0,
    /// The module sleep is controlled with the DTR pin
    HardwareControlled = 1,
    /// The module sleep is controlled by the own module in idle periods
    SoftwareControlled = 2,
}

/// Struct that can be used to control the module CSCLK mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Eq)]
pub struct SetCSCLKMode {
    pub mode: CSCLKMode,
}

impl AtRequest for SetCSCLKMode {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSCLK")
            .with_int_parameter(self.mode as u8)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn csclk_mode_repr_values() {
        assert_eq!(CSCLKMode::Disabled as u8, 0);
        assert_eq!(CSCLKMode::HardwareControlled as u8, 1);
        assert_eq!(CSCLKMode::SoftwareControlled as u8, 2);
    }

    #[test]
    fn set_csclk_disabled_command() {
        let cmd = SetCSCLKMode {
            mode: CSCLKMode::Disabled,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSCLK=0\r\n");
    }

    #[test]
    fn set_csclk_hardware_controlled_command() {
        let cmd = SetCSCLKMode {
            mode: CSCLKMode::HardwareControlled,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSCLK=1\r\n");
    }

    #[test]
    fn set_csclk_software_controlled_command() {
        let cmd = SetCSCLKMode {
            mode: CSCLKMode::SoftwareControlled,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSCLK=2\r\n");
    }

    #[test]
    fn set_csclk_parse_ok() {
        let cmd = SetCSCLKMode {
            mode: CSCLKMode::SoftwareControlled,
        };

        let result = cmd.parse_response_struct(b"\r\nOK\r\n");

        assert!(result.is_ok());
    }

    #[test]
    fn set_csclk_parse_fails_on_error() {
        let cmd = SetCSCLKMode {
            mode: CSCLKMode::Disabled,
        };

        let result = cmd.parse_response_struct(b"\r\nERROR\r\n");

        assert!(result.is_err());
    }
}
