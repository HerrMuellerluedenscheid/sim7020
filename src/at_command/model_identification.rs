#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModelIdentification {}

pub const MODEL_IDENTIFIER_SIZE: usize = 8;

pub struct ModelIdentificationResponse {
    pub model: [u8; MODEL_IDENTIFIER_SIZE],
}

impl ModelIdentification {
    fn get_model(data: &[u8]) -> Result<[u8; MODEL_IDENTIFIER_SIZE], AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()
            .inspect(|_e| {
                #[cfg(feature = "defmt")]
                error!("Failed to parse response: {=[u8]:a}", data);
            })?;

        let mut id: [u8; MODEL_IDENTIFIER_SIZE] = [0; MODEL_IDENTIFIER_SIZE];
        for (i, b) in parsed.as_bytes().iter().enumerate() {
            id[i] = *b;
        }
        Ok(id)
    }
}

impl AtRequest for ModelIdentification {
    type Response = ModelIdentificationResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CGMM")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let id = Self::get_model(data)?;
        Ok(AtResponse::ModelIdentifier(id))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let id = Self::get_model(data)?;
        Ok(ModelIdentificationResponse { model: id })
    }
}
