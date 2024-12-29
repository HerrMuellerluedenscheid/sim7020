use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ExtendedErrorReport;

impl AtRequest for ExtendedErrorReport {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CEER")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("error report response: {=[u8]:a}", data);
        Ok(AtResponse::Ok)
    }
}
