use crate::at_command::{AtRequest, AtResponse};
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
pub struct StartNTPConnection;

impl AtRequest for StartNTPConnection {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CSNTPSTART=202.112.29.82\r\n").unwrap();
    }
}

#[derive(Format)]
pub struct StopNTPConnection;

impl AtRequest for StopNTPConnection {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CSNTPSTOP\r\n").unwrap();
    }
}

#[derive(Format)]
pub struct NTPTime {}

impl AtRequest for NTPTime {
    type Response = Result<(), AtError>;
    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CCLK?\r\n").unwrap();
    }
}
