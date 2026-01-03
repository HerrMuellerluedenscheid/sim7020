//! Module for IP address
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;
#[cfg(feature = "defmt")]
use defmt::info;

/// Command to request the IP address
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct LocalIPAddress;

/// Max size of an IP address (including v4 and v6) in string format
const MAX_IP_SIZE: usize = 39;

/// The response containing the local ip address
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct LocalIpAddressResponse {
    pub ip: heapless::String<MAX_IP_SIZE>,
}

impl AtRequest for LocalIPAddress {
    type Response = LocalIpAddressResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CIFSR")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (local_ip,) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CIFSR: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        #[cfg(feature = "defmt")]
        info!("localip: {}", local_ip);
        Ok(AtResponse::LocalIPAddress(local_ip))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (local_ip,) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CIFSR: ")
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        #[cfg(feature = "defmt")]
        info!("localip: {}", local_ip);
        let ip: heapless::String<MAX_IP_SIZE> = local_ip.try_into()?;

        Ok(LocalIpAddressResponse { ip })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn local_ip_address_get_command() {
        let cmd = LocalIPAddress;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CIFSR\r\n");
    }

    #[test]
    fn local_ip_address_parse_ipv4() {
        let cmd = LocalIPAddress;

        let data = b"\r\n+CIFSR: 192.168.1.100\r\n\r\nOK";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.ip.as_str(), "192.168.1.100");
    }

    #[test]
    fn local_ip_address_parse_ipv6() {
        let cmd = LocalIPAddress;

        let data = b"\r\n+CIFSR: fe80::1ff:fe23:4567:890a\r\n\r\nOK";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.ip.as_str(), "fe80::1ff:fe23:4567:890a");
    }

    #[test]
    fn local_ip_address_parse_fails_without_ok() {
        let cmd = LocalIPAddress;

        let data = b"ERROR";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }
}
