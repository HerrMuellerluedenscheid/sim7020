use crate::at_command::{AtRequest, BufferType};
use crate::BUFFER_SIZE;
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct AtI;

pub struct ProductInformation {}

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("I")
            .finish()
    }
}
