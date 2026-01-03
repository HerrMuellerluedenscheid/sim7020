//! Module for the operations related to the network registration
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;

/// The status of unsolicited messages
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum UnsolicitedResultCodes {
    Disabled,
    Enabled,
    EnabledVerbose,
}

impl From<i32> for UnsolicitedResultCodes {
    fn from(code: i32) -> Self {
        match code {
            0 => UnsolicitedResultCodes::Disabled,
            1 => UnsolicitedResultCodes::Enabled,
            2 => UnsolicitedResultCodes::EnabledVerbose,
            _ => {
                unreachable!()
            }
        }
    }
}

/// Possible status allowed by the network registration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum NetworkRegistrationStatus {
    NotRegistered,
    RegisteredHomeNetwork,
    NotRegisteredSearching,
    RegistrationDenied,
    Unknown,
    RegisteredRoaming,
    SMSOnlyHome,
    SMSOnlyRoaming,
}

impl From<i32> for NetworkRegistrationStatus {
    fn from(code: i32) -> Self {
        match code {
            0 => Self::NotRegistered,
            1 => Self::RegisteredHomeNetwork,
            2 => Self::NotRegisteredSearching,
            3 => Self::RegistrationDenied,
            4 => Self::Unknown,
            5 => Self::RegisteredRoaming,
            6 => Self::SMSOnlyHome,
            7 => Self::SMSOnlyRoaming,
            _ => {
                unreachable!()
            }
        }
    }
}

/// Command to query the network registration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkRegistration;

/// Response with the network registration status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkRegistrationResponse {
    pub unsolicited: UnsolicitedResultCodes,
    pub status: NetworkRegistrationStatus,
}

impl NetworkRegistration {
    fn get_data(
        data: &[u8],
    ) -> Result<(UnsolicitedResultCodes, NetworkRegistrationStatus), AtError> {
        let (n, stat) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGREG: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r")
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
            .named("+CGREG")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (unsolicited, status) = Self::get_data(data)?;
        Ok(AtResponse::NetworkRegistrationStatus(unsolicited, status))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (unsolicited, status) = Self::get_data(data)?;
        Ok(NetworkRegistrationResponse {
            status,
            unsolicited,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unsolicited_result_codes_from_int() {
        assert_eq!(
            UnsolicitedResultCodes::from(0),
            UnsolicitedResultCodes::Disabled
        );
        assert_eq!(
            UnsolicitedResultCodes::from(1),
            UnsolicitedResultCodes::Enabled
        );
        assert_eq!(
            UnsolicitedResultCodes::from(2),
            UnsolicitedResultCodes::EnabledVerbose
        );
    }

    #[test]
    fn network_registration_status_from_int() {
        assert_eq!(
            NetworkRegistrationStatus::from(0),
            NetworkRegistrationStatus::NotRegistered
        );
        assert_eq!(
            NetworkRegistrationStatus::from(1),
            NetworkRegistrationStatus::RegisteredHomeNetwork
        );
        assert_eq!(
            NetworkRegistrationStatus::from(2),
            NetworkRegistrationStatus::NotRegisteredSearching
        );
        assert_eq!(
            NetworkRegistrationStatus::from(3),
            NetworkRegistrationStatus::RegistrationDenied
        );
        assert_eq!(
            NetworkRegistrationStatus::from(4),
            NetworkRegistrationStatus::Unknown
        );
        assert_eq!(
            NetworkRegistrationStatus::from(5),
            NetworkRegistrationStatus::RegisteredRoaming
        );
        assert_eq!(
            NetworkRegistrationStatus::from(6),
            NetworkRegistrationStatus::SMSOnlyHome
        );
        assert_eq!(
            NetworkRegistrationStatus::from(7),
            NetworkRegistrationStatus::SMSOnlyRoaming
        );
    }

    #[test]
    fn network_registration_get_command() {
        let req = NetworkRegistration;
        let mut buffer: [u8; 512] = [0; 512];

        let cmd = req.get_command(&mut buffer).unwrap();

        assert_eq!(cmd, b"AT+CGREG?\r\n");
    }

    #[test]
    fn parse_network_registration_registered_home() {
        let data = b"\r\n+CGREG: 1,1\r\n\r\nOK\r";

        let response = NetworkRegistration.parse_response_struct(data).unwrap();

        assert_eq!(response.unsolicited, UnsolicitedResultCodes::Enabled);
        assert_eq!(
            response.status,
            NetworkRegistrationStatus::RegisteredHomeNetwork
        );
    }

    #[test]
    fn parse_network_registration_not_registered() {
        let data = b"\r\n+CGREG: 0,0\r\n\r\nOK\r";

        let response = NetworkRegistration.parse_response_struct(data).unwrap();

        assert_eq!(response.unsolicited, UnsolicitedResultCodes::Disabled);
        assert_eq!(response.status, NetworkRegistrationStatus::NotRegistered);
    }

    #[test]
    fn parse_network_registration_roaming() {
        let data = b"\r\n+CGREG: 2,5\r\n\r\nOK\r";

        let response = NetworkRegistration.parse_response_struct(data).unwrap();

        assert_eq!(response.unsolicited, UnsolicitedResultCodes::EnabledVerbose);
        assert_eq!(
            response.status,
            NetworkRegistrationStatus::RegisteredRoaming
        );
    }

    #[test]
    fn parse_network_registration_invalid_format() {
        let data = b"\r\n+CGREG: a,b\r\n\r\nOK\r";

        assert!(NetworkRegistration.parse_response_struct(data).is_err());
    }
}
