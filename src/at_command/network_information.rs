#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
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

const OPERATOR_MAX_SIZE: usize = 16;

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

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
