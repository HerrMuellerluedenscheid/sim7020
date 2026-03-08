#[cfg(feature = "defmt")]
use defmt::debug;
use crate::at_command::{verify_ok, AtRequest};
use crate::AtError;

/// Implementation of the command AT+CDNSGIP which allows performing a DNS query
/// The query response will arrive by an unsolicited message
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CDNSGIP<'a> {
    pub domain: &'a str,
}

impl AtRequest for CDNSGIP<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CDNSGIP")
            .with_string_parameter(self.domain)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        debug!("The received data is: {:?}", data);
        verify_ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command_basic() {
        let cmd = CDNSGIP { domain: "example.com" };
        let mut buffer = [0u8; 64];

        let result = cmd.get_command(&mut buffer).unwrap();
        let command_str = core::str::from_utf8(result).unwrap();

        assert_eq!(command_str, "AT+CDNSGIP=\"example.com\"\r\n");
    }

    #[test]
    fn test_get_command_small_buffer() {
        let cmd = CDNSGIP { domain: "example.com" };
        let mut buffer = [0u8; 8];

        let result = cmd.get_command(&mut buffer);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_ok() {
        let cmd = CDNSGIP { domain: "example.com" };

        let data = b"\r\nOK\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn test_parse_response_error() {
        let cmd = CDNSGIP { domain: "example.com" };

        let data = b"\r\nERROR\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_with_extra_whitespace() {
        let cmd = CDNSGIP { domain: "example.com" };

        let data = b"\r\nOK\r\n\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_ok());
    }
}