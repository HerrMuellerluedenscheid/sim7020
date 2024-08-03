use crate::at_command::{AtRequest, BufferType};
use crate::{AtError};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
/// Test if a pin is required.
pub struct PINRequired;

impl AtRequest for PINRequired {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+CPIN")
.finish()
    }
}

#[derive(Format)]
/// Enter PIN.
pub struct EnterPIN {
    pin: u16,
}

impl AtRequest for EnterPIN {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CPIN")
            .with_int_parameter(self.pin)
            .finish()
    }
}
