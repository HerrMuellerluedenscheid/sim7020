//! Module to handle the wireless commands
use crate::at_command::{AtRequest, BufferType};

/// Command to start the wireless connection
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct StartWirelessConnection;

impl AtRequest for StartWirelessConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CIICR")
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, crate::AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_wireless_connection_command() {
        let cmd = StartWirelessConnection;

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CIICR\r\n");
    }

    #[test]
    fn start_wireless_connection_parse_response_struct() {
        let cmd = StartWirelessConnection;

        // Typical OK response, but parser ignores content
        let data = b"\r\nOK\r\n";

        assert!(cmd.parse_response_struct(data).is_ok());
    }
}
