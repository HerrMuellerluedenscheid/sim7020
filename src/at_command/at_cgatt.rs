use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GPRSServiceState{
    Detached, // 0
    Attached, // 1
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GPRSServiceStatus;

impl AtRequest for GPRSServiceStatus {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGATT")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (state,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGATT: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        let state = match state {
            0 => GPRSServiceState::Attached,
            1 => GPRSServiceState::Detached,
            _ => {panic!("invalid GPRSServiceStatus")}
        };
        Ok(AtResponse::PacketDomainAttachmentState(state))
    }
}
