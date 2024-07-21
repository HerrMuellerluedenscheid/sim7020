use crate::at_command::AtRequest;
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
pub struct GPRSServiceStatus;

impl AtRequest for GPRSServiceStatus {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CGATT?\r\n").unwrap();
    }
}
