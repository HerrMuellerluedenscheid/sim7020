use crate::at_command::AtRequest;
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

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

    fn send(&self, writer: &mut ModemWriter) {
        let status = self.status as u8;
        writer.write_str("ATE").unwrap();
        writer.write_full_blocking(&[status]);
        writer.write_str("\r\n").unwrap();
    }
}
