use crate::at_command::{AtRequest, AtResponse};
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
/// Test if a pin is required.
pub struct PINRequired;

impl AtRequest for PINRequired {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CPIN?\r\n").unwrap();
    }
}

#[derive(Format)]
/// Enter PIN.
pub struct EnterPIN {
    pin: u8,
}

impl AtRequest for EnterPIN {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        let pin = self.pin;
        writer.write_str("AT+CPIN=").unwrap();
        writer.write_full_blocking(&[pin]);
        writer.write_str("\r\n").unwrap();
    }
}
