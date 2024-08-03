use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct AtCreg;

impl AtRequest for AtCreg {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+CREG")
            .finish()
    }
}
