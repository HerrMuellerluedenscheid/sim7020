use at_commands::parser::CommandParser;
use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetworkFormat{
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
pub enum NetworkStatus{
    Unknown,
    OperatorAvailable,
    OperatorCurrent,
    OperatorForbidden
}

impl From<i32> for NetworkStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => {Self::Unknown},
            1 => Self::OperatorAvailable,
            2 => Self::OperatorCurrent,
            3 => Self::OperatorForbidden,
            _ => unreachable!()
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

        let (state, format, operator, _access_technology ) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+COPS: ")
            .expect_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state= NetworkStatus::from(state);

        let format = match format {
            Some(form) => { NetworkFormat::from(form)}
            None => {NetworkFormat::Unknown}
        };
        #[cfg(feature = "defmt")]
        info!("network information: {:?} | operator: {}", state, operator);
        Ok(AtResponse::NetworkInformationState(state, format))
    }
}
