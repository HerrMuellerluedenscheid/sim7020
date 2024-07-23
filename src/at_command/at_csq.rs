use crate::at_command::AtRequest;
use crate::{AtError};
use embedded_io::Write;
use defmt::Format;

#[derive(Format)]
pub struct AtCsq;

impl AtRequest for AtCsq {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CSQ\r\n".as_bytes()).unwrap();
    }
}
