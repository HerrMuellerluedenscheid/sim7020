use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkRegistration;

impl AtRequest for NetworkRegistration {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGREG")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (n, stat) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGREG: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r")
            .finish()?;
        let unsolicited = UnsolicitedResultCodes::from(n);
        let status = NetworkRegistrationStatus::from(stat);
        Ok(AtResponse::NetworkRegistrationStatus(unsolicited, status))
    }
}
