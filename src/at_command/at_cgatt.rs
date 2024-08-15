use crate::at_command::{AtRequest, BufferType};
use crate::{AtError};
use defmt::Format;

#[derive(Format)]
pub struct GPRSServiceStatus;

impl AtRequest for GPRSServiceStatus {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGATT")
            .finish()
    }
}
