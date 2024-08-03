use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct StartNTPConnection;

impl AtRequest for StartNTPConnection {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        // todo fix hard coded ip
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSNTPSTART")
            .with_string_parameter("202.112.29.82")
.finish()
    }
}

#[derive(Format)]
pub struct StopNTPConnection;

impl AtRequest for StopNTPConnection {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CSNTPSTOP")
.finish()
    }
}

#[derive(Format)]
pub struct NTPTime {}

impl AtRequest for NTPTime {
    type Response = Result<(), AtError>;
    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CCLK").finish()
    }
}
