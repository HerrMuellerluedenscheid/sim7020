use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
/// TA returns a list of quadruplets, each representing an operator present in
/// the network. Any of the formats may be unavailable and should then be an
/// empty field. The list of operators shall be in order: home network,
/// networks referenced in SIM, and other networks.
pub struct NetworkInformationAvailable;

impl AtRequest for NetworkInformationAvailable {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+COPS")
.finish()
    }
}

#[derive(Format)]
/// Current mode and the currently selected operator
pub struct NetworkInformation;

impl AtRequest for NetworkInformation {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a[u8], usize> {        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+COPS")
.finish()
    }
}
