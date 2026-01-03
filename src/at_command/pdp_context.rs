#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;
#[cfg(feature = "defmt")]
use defmt::debug;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
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
#[derive(PartialEq, Clone)]
pub struct PDPContext;

pub struct PDPContextResponse {
    pub state: PDPState,
    pub context: i32,
}

impl PDPContext {
    fn get_status(data: &[u8]) -> Result<(PDPState, i32), AtError> {
        let (state, context) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGACT: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let state = PDPState::from(state);

        Ok((state, context))
    }
}

impl AtRequest for PDPContext {
    type Response = PDPContextResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGACT")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        if data.starts_with(b"\r\nOK\r") {
            #[cfg(feature = "defmt")]
            debug!("waiting for PDPContext");
            return Ok(AtResponse::PDPContext(None));
        };
        let (state, context) = Self::get_status(data)?;
        Ok(AtResponse::PDPContext(Some((state, context))))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (state, context) = Self::get_status(data)?;
        Ok(PDPContextResponse { state, context })
    }
}
