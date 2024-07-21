use crate::at_command::{AtRequest, AtResponse};
use crate::ModemWriter;
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
pub struct AtI;

pub struct ProductInformation {
    dummy: u8,
}

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("ATI\r\n").unwrap();
    }
}
