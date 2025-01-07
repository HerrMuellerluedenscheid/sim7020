use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;
#[cfg(feature = "defmt")]
use defmt::info;

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

impl AtRequest for NetworkInformation {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+COPS")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
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
        let operator = match operator {
            None => None,
            Some(o) => Some(NetworkOperator::from(o)),
        };
        #[cfg(feature = "defmt")]
        info!("network information: {:?} | operator: {}", mode, operator);
        Ok(AtResponse::NetworkInformationState(mode, format, operator))
    }
}
