//! Module to get the model identification information
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::error;

/// Command to get the model identification
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct ModelIdentification;

/// Size of the model
pub const MODEL_IDENTIFIER_SIZE: usize = 8;

/// Response containing the model identification
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct ModelIdentificationResponse {
    pub model: [u8; MODEL_IDENTIFIER_SIZE],
}

impl ModelIdentification {
    fn get_model(data: &[u8]) -> Result<[u8; MODEL_IDENTIFIER_SIZE], AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"OK")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_identification_get_command() {
        let cmd = ModelIdentification;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CGMM\r\n");
    }

    #[test]
    fn model_identification_parse_exact_size() {
        let cmd = ModelIdentification;

        let data = b"MODEL123\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.model, *b"MODEL123");
    }

    #[test]
    fn model_identification_parse_short_model() {
        let cmd = ModelIdentification;

        let data = b"ABC\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        let mut expected = [0u8; MODEL_IDENTIFIER_SIZE];
        expected[..3].copy_from_slice(b"ABC");

        assert_eq!(response.model, expected);
    }

    #[test]
    fn model_identification_parse_fails_on_empty_response() {
        let cmd = ModelIdentification;

        let data = b"\r\nOK\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }
}
