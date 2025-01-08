use crate::at_command::network_registration_status::{
    NetworkRegistrationStatus, UnsolicitedResultCodes,
};
use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkRegistration;

impl AtRequest for NetworkRegistration {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CREG")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (n, stat) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CREG: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r")
            .finish()?;
        let unsolicited = UnsolicitedResultCodes::from(n);
        let status = NetworkRegistrationStatus::from(stat);
        Ok(AtResponse::NetworkRegistration(unsolicited, status))
    }
}

// provokes an error for testing purposes
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AtCregError;
impl AtRequest for AtCregError {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CREG")
            .with_int_parameter(5)
            .finish()
    }
}
