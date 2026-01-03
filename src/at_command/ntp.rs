/// Commands for the NTP protocol
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

/// Starts a NTP query
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct StartQueryNTP<'a> {
    pub url: &'a str,
    pub tzinfo: Option<&'a str>, // currently not implemented
}

impl AtRequest for StartQueryNTP<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        match &self.tzinfo {
            None => at_commands::builder::CommandBuilder::create_set(buffer, true)
                .named("+CSNTPSTART")
                .with_string_parameter(self.url)
                .finish(),
            Some(tzinfo) => at_commands::builder::CommandBuilder::create_set(buffer, true)
                .named("+CSNTPSTART")
                .with_string_parameter(self.url)
                .with_string_parameter(tzinfo)
                .finish(),
        }
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Stops the NTP query
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct StopQueryNTP;

impl AtRequest for StopQueryNTP {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CSNTPSTOP")
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_query_ntp_without_tzinfo() {
        let cmd = StartQueryNTP {
            url: "pool.ntp.org",
            tzinfo: None,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSNTPSTART=\"pool.ntp.org\"\r\n");
    }

    #[test]
    fn start_query_ntp_with_tzinfo() {
        let cmd = StartQueryNTP {
            url: "time.google.com",
            tzinfo: Some("UTC"),
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSNTPSTART=\"time.google.com\",\"UTC\"\r\n");
    }

    #[test]
    fn start_query_ntp_parse_response_ok() {
        let cmd = StartQueryNTP {
            url: "pool.ntp.org",
            tzinfo: None,
        };

        let data = b"\r\nOK\r\n";

        assert!(cmd.parse_response_struct(data).is_ok());
    }

    #[test]
    fn stop_query_ntp_get_command() {
        let cmd = StopQueryNTP;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSNTPSTOP?\r\n");
    }

    #[test]
    fn stop_query_ntp_parse_response_ok() {
        let cmd = StopQueryNTP;

        let data = b"\r\nOK\r\n";

        assert!(cmd.parse_response_struct(data).is_ok());
    }
}
