use crate::at_command::AtRequest;
use crate::AtError;
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct AtCreg;

impl AtRequest for AtCreg {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CREG?\r\n".as_bytes()).unwrap();
    }
}
