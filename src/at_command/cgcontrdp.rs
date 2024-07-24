use crate::at_command::{AtRequest, AtResponse};
use crate::AtError;
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct PDPContextReadDynamicsParameters;

impl AtRequest for PDPContextReadDynamicsParameters {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+CGCONTRDP\r\n".as_bytes()).unwrap();
    }
}
