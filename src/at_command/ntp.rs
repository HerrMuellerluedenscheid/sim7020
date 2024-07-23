use crate::at_command::{AtRequest, AtResponse};
use crate::{AtError};
use embedded_io::Write;
use defmt::Format;

#[derive(Format)]
pub struct StartNTPConnection;

impl AtRequest for StartNTPConnection {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CSNTPSTART=202.112.29.82\r\n".as_bytes()).unwrap();
    }
}

#[derive(Format)]
pub struct StopNTPConnection;

impl AtRequest for StopNTPConnection {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CSNTPSTOP\r\n".as_bytes()).unwrap();
    }
}

#[derive(Format)]
pub struct NTPTime {}

impl AtRequest for NTPTime {
    type Response = Result<(), AtError>;
    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CCLK?\r\n".as_bytes()).unwrap();
    }
}
