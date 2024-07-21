use crate::at_command::{AtRequest, AtResponse};
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

const CSTT_SIZE_MAX: usize = 32; // AT Datasheet page 172

#[derive(Format)]
/// Enter PIN.
pub struct GetAPNUserPassword {}

impl AtRequest for GetAPNUserPassword {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CSTT?").unwrap();
    }
}

#[derive(Format)]
/// Enter PIN.
pub struct SetAPNUserPassword {
    pub(crate) apn: Option<[u8; CSTT_SIZE_MAX]>,
    pub(crate) user: Option<[u8; CSTT_SIZE_MAX]>,
    pub(crate) password: Option<[u8; CSTT_SIZE_MAX]>,
}

impl SetAPNUserPassword {
    pub fn new() -> Self {
        Self {
            apn: None,
            user: None,
            password: None,
        }
    }
    pub fn with_apn(mut self, apn: &str) -> Self {
        let mut apn_b = [b'\0'; CSTT_SIZE_MAX];
        for (i, b) in apn.as_bytes().iter().enumerate() {
            apn_b[i] = *b;
        }
        self.apn = Some(apn_b);
        self
    }
}

impl AtRequest for SetAPNUserPassword {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CSTT=").unwrap();
        if Option::is_some(&self.apn) {
            writer.write_full_blocking(&self.apn.unwrap());
        }
        writer.write_char(',').unwrap();
        if Option::is_some(&self.user) {
            writer.write_full_blocking(&self.user.unwrap());
        }
        writer.write_char(',').unwrap();
        if Option::is_some(&self.password) {
            writer.write_full_blocking(&self.password.unwrap());
        }
        writer.write_str("\r\n").unwrap();
    }
}
