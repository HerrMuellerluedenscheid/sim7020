use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModelIdentification {}

impl AtRequest for ModelIdentification {
    type Response = Result<AtResponse, AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CGMM")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (parsed,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()
            .inspect(|_e| {
                #[cfg(feature = "defmt")]
                error!("Failed to parse response: {=[u8]:a}", data);
            })?;

        let mut id: [u8; 8] = [0; 8];
        for (i, b) in parsed.as_bytes().iter().enumerate() {
            id[i] = *b;
        }
        Ok(AtResponse::ModelIdentifier(id))
    }
}
