use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::Format;
use embedded_io::Write;

#[derive(Format, Clone, Copy)]
#[repr(u8)]
pub enum Echo {
    Disable = 0,
    Enable = 1,
}

#[derive(Format, Clone, Copy)]
pub struct AtEcho {
    pub status: Echo,
}

impl AtRequest for AtEcho {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("E")
            .with_int_parameter(self.status as u8)
.finish()
    }
}
