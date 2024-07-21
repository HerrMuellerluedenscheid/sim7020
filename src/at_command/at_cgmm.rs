use crate::at_command::AtRequest;
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
pub struct AtCgmm;

impl AtRequest for AtCgmm {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CGMM\r\n").unwrap();
    }
}
