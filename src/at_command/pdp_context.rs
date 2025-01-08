use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PDPState {
    Deactivated,
    Activated,
}

impl From<i32> for PDPState {
    fn from(value: i32) -> Self {
        match value {
            0 => PDPState::Deactivated,
            1 => PDPState::Activated,
            _ => {
                unreachable!()
            }
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PDPContext;

impl AtRequest for PDPContext {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGACT")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (state, context) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGACT: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PDPState::from(state);
        Ok(AtResponse::PDPContext(state, context))
    }
}
