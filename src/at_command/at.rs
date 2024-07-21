use crate::at_command::AtRequest;
use crate::{AtError, ModemWriter};
use defmt::Format;

#[derive(Format)]
pub struct At;

impl AtRequest for At {
    type Response = Result<(), AtError>;
    fn send(&self, writer: &mut ModemWriter) {
        writer.write_full_blocking(b"AT\r\n");
    }
}
