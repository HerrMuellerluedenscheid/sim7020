use crate::at_command::network_registration_status::{
    NetworkRegistrationStatus, UnsolicitedResultCodes,
};
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkRegistration;

pub struct NetworkRegistrationResponse {
    pub unsolicited_result: UnsolicitedResultCodes,
    pub status: NetworkRegistrationStatus,
}

impl NetworkRegistration {
    fn parse_response(
        data: &[u8],
    ) -> Result<(UnsolicitedResultCodes, NetworkRegistrationStatus), AtError> {
        let (n, stat) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CREG: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r")
            .finish()?;

        let unsolicited = UnsolicitedResultCodes::from(n);
        let status = NetworkRegistrationStatus::from(stat);

        return Ok((unsolicited, status));
    }
}

impl AtRequest for NetworkRegistration {
    type Response = NetworkRegistrationResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
            status: status,
            unsolicited_result: unsolicited,
        })
    }
}

// provokes an error for testing purposes
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AtCregError;

impl AtRequest for AtCregError {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CREG")
            .with_int_parameter(5)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}
