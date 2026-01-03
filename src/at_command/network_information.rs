//! Module for the network information
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

/// The formats allowed for network
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum NetworkFormat {
    LongAlphanumeric,
    ShortAlphanumeric,
    Numeric,
    Unknown,
}

impl From<i32> for NetworkFormat {
    fn from(value: i32) -> Self {
        match value {
            0 => NetworkFormat::LongAlphanumeric,
            1 => NetworkFormat::ShortAlphanumeric,
            2 => NetworkFormat::Numeric,
            _ => unreachable!(),
        }
    }
}

/// The network mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum NetworkMode {
    Automatic,
    Manual,
}

impl From<i32> for NetworkMode {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Automatic,
            1 => Self::Manual,
            _ => unreachable!(),
        }
    }
}

/// TA returns a list of quadruplets, each representing an operator present in
/// the network. Any of the formats may be unavailable and should then be an
/// empty field. The list of operators shall be in order: home network,
/// networks referenced in SIM, and other networks.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkInformation;

/// Max size of the oeprator
const OPERATOR_MAX_SIZE: usize = 16;

/// Network operator definition
pub type NetworkOperator = heapless::String<OPERATOR_MAX_SIZE>;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct NetworkInformationState {
    pub mode: NetworkMode,
    pub format: NetworkFormat,
    pub operator: Option<NetworkOperator>,
}

impl NetworkInformation {
    fn get_network_info(data: &[u8]) -> Result<NetworkInformationState, AtError> {
        let (mode, format, operator, _access_technology) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+COPS: ")
            .expect_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        let mode = NetworkMode::from(mode);

        let format = match format {
            Some(form) => NetworkFormat::from(form),
            None => NetworkFormat::Unknown,
        };

        let operator: Option<NetworkOperator> = operator.map(|x| x.try_into()).transpose()?;

        Ok(NetworkInformationState {
            format,
            mode,
            operator,
        })
    }
}

impl AtRequest for NetworkInformation {
    type Response = NetworkInformationState;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+COPS")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let network = Self::get_network_info(data)?;
        Ok(AtResponse::NetworkInformationState(
            network.mode,
            network.format,
            network.operator,
        ))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let network = Self::get_network_info(data)?;
        Ok(network)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn network_format_from_int() {
        assert_eq!(NetworkFormat::from(0), NetworkFormat::LongAlphanumeric);
        assert_eq!(NetworkFormat::from(1), NetworkFormat::ShortAlphanumeric);
        assert_eq!(NetworkFormat::from(2), NetworkFormat::Numeric);
    }

    #[test]
    fn network_mode_from_int() {
        assert_eq!(NetworkMode::from(0), NetworkMode::Automatic);
        assert_eq!(NetworkMode::from(1), NetworkMode::Manual);
    }

    #[test]
    fn network_information_get_command() {
        let req = NetworkInformation;
        let mut buffer: [u8; 512] = [0; 512];

        let cmd = req.get_command(&mut buffer).unwrap();

        assert_eq!(cmd, b"AT+COPS?\r\n");
    }

    #[test]
    fn parse_network_information_full_response() {
        let data = b"\r\n+COPS: 0,2,\"26201\",7\r\n\r\nOK";

        let info = NetworkInformation.parse_response_struct(data).unwrap();

        assert_eq!(info.mode, NetworkMode::Automatic);
        assert_eq!(info.format, NetworkFormat::Numeric);
        assert_eq!(info.operator.as_ref().map(|s| s.as_str()), Some("26201"));
    }

    #[test]
    fn parse_network_information_without_operator() {
        let data = b"\r\n+COPS: 0\r\n\r\nOK";

        let info = NetworkInformation.parse_response_struct(data).unwrap();

        assert_eq!(info.mode, NetworkMode::Automatic);
        assert_eq!(info.format, NetworkFormat::Unknown);
        assert!(info.operator.is_none());
    }

    #[test]
    fn parse_network_information_with_format_only() {
        let data = b"\r\n+COPS: 1,1\r\n\r\nOK";

        let info = NetworkInformation.parse_response_struct(data).unwrap();

        assert_eq!(info.mode, NetworkMode::Manual);
        assert_eq!(info.format, NetworkFormat::ShortAlphanumeric);
        assert!(info.operator.is_none());
    }

    #[test]
    fn parse_network_information_invalid_response() {
        let data = b"\r\n+COPS: ,,,\r\nERROR";

        assert!(NetworkInformation.parse_response_struct(data).is_err());
    }
}
