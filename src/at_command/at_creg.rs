//! Module to check the network registration status

use crate::at_command::network_registration_status::{
    NetworkRegistrationStatus, UnsolicitedResultCodes,
};
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;

/// Struct to query the network registration status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkRegistration;

/// Current status of the network registration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkRegistrationResponse {
    /// Current state of the unsolicited messages
    pub unsolicited_result: UnsolicitedResultCodes,
    /// Current state of the network registration
    pub status: NetworkRegistrationStatus,
}

impl NetworkRegistration {
    fn parse_response(
        data: &[u8],
    ) -> Result<(UnsolicitedResultCodes, NetworkRegistrationStatus), AtError> {
        let (n, stat) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CREG: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        let unsolicited = UnsolicitedResultCodes::from(n);
        let status = NetworkRegistrationStatus::from(stat);

        Ok((unsolicited, status))
    }
}

impl AtRequest for NetworkRegistration {
    type Response = NetworkRegistrationResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CREG")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (unsolicited, status) = Self::parse_response(data)?;
        Ok(AtResponse::NetworkRegistration(unsolicited, status))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (unsolicited, status) = Self::parse_response(data)?;
        Ok(NetworkRegistrationResponse {
            status,
            unsolicited_result: unsolicited,
        })
    }
}

/// provokes an error for testing purposes
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtCregError;

impl AtRequest for AtCregError {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CREG")
            .with_int_parameter(5)
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
    fn test_network_registration_command() {
        let mut buffer = [0u8; 512];
        let command = NetworkRegistration.get_command(&mut buffer).unwrap();

        assert_eq!(command, b"AT+CREG?\r\n");
    }

    #[test]
    fn test_network_registration_response() {
        let data = b"\r\n+CREG: 0,0\r\n\r\nOK\r\n";

        let result = NetworkRegistration.parse_response_struct(data).unwrap();
        assert_eq!(result.unsolicited_result, UnsolicitedResultCodes::Disabled);
        assert_eq!(result.status, NetworkRegistrationStatus::NotRegistered)
    }
}
