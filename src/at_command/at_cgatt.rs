//! Module to handle the GPRS service status

use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;
use at_commands::parser::CommandParser;

/// Current status of the GPRS
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum GPRSServiceState {
    Detached, // 0
    Attached, // 1
}

impl From<i32> for GPRSServiceState {
    fn from(v: i32) -> Self {
        match v {
            0 => GPRSServiceState::Detached,
            1 => GPRSServiceState::Attached,
            _ => {
                unreachable!("invalid GPRSServiceStatus")
            }
        }
    }
}

/// Command to request the GPRS status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GPRSServiceStatus;

/// Current state of the GPRS service
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct PacketDomainAttachmentState {
    pub state: GPRSServiceState,
}

impl GPRSServiceStatus {
    fn parse_state(data: &[u8]) -> Result<GPRSServiceState, AtError> {
        let (state,) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CGATT: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        let state: GPRSServiceState = state.into();

        Ok(state)
    }
}

impl AtRequest for GPRSServiceStatus {
    type Response = PacketDomainAttachmentState;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGATT")
            .finish()
    }

    #[allow(deprecated_in_future)]
    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let state = Self::parse_state(data)?;
        Ok(AtResponse::PacketDomainAttachmentState(state))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let state = Self::parse_state(data)?;

        Ok(PacketDomainAttachmentState { state })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_request_gprs_service_status() {
        let mut buffer = [0u8; 512];
        let response = GPRSServiceStatus.get_command(&mut buffer).unwrap();

        assert_eq!(response, b"AT+CGATT?\r\n");
    }

    #[test]
    fn test_response_gprs_service_status() {
        let response = b"\r\n+CGATT: 0\r\n\r\nOK\r\n";

        let status = GPRSServiceStatus.parse_response_struct(response).unwrap();

        assert_eq!(status.state, GPRSServiceState::Detached);
    }

    #[test]
    fn test_gprs_service_status_parse() {
        let deattached: GPRSServiceState = 0i32.into();

        assert_eq!(deattached, GPRSServiceState::Detached);

        let attached: GPRSServiceState = 1i32.into();

        assert_eq!(attached, GPRSServiceState::Attached);
    }

    #[test]
    #[should_panic]
    fn test_gprs_service_status_parse_invalid() {
        let _: GPRSServiceState = 1000i32.into();
    }
}
