use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct PDPContextReadDynamicsParameters;

impl AtRequest for PDPContextReadDynamicsParameters {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGCONTRD")
.finish()
    }
}
