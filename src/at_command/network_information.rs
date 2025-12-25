#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkOperator([u8; 10]);

impl From<&str> for NetworkOperator {
    fn from(value: &str) -> Self {
        let mut data = [0; 10];
        let vab = value.as_bytes();
        let len = vab.len().min(data.len());
        data[..len].copy_from_slice(&vab[..len]);
        Self(data)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
#[derive(PartialEq, Eq)]
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
pub struct NetworkInformation;

const OPERATOR_MAX_SIZE: usize = 16;

pub type OperatorName = heapless::String<OPERATOR_MAX_SIZE>;

pub struct NetworkInformationState {
    pub mode: NetworkMode,
    pub format: NetworkFormat,
    pub operator: Option<OperatorName>,
}

impl NetworkInformation {
    fn get_network_info(
        data: &[u8],
    ) -> Result<(i32, Option<i32>, Option<&str>, Option<i32>), AtError> {
        let tuple = CommandParser::parse(data)
            .expect_identifier(b"\r\n+COPS: ")
            .expect_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        return Ok(tuple);
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
        let (mode, format, operator, _access_technology) = Self::get_network_info(data)?;
        let mode = NetworkMode::from(mode);

        let format = match format {
            Some(form) => NetworkFormat::from(form),
            None => NetworkFormat::Unknown,
        };
        let operator = operator.map(NetworkOperator::from);
        Ok(AtResponse::NetworkInformationState(mode, format, operator))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (mode, format, operator, _access_technology) = Self::get_network_info(data)?;
        let mode = NetworkMode::from(mode);

        let format = match format {
            Some(form) => NetworkFormat::from(form),
            None => NetworkFormat::Unknown,
        };

        let operator: Option<OperatorName> = operator.map(|x| x.try_into()).transpose()?;

        return Ok(Self::Response {
            format: format,
            mode: mode,
            operator: operator,
        });
    }
}
