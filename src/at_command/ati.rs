use crate::at_command::{AtRequest, AtResponse};
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
pub struct AtI;

pub struct ProductInformation {
    dummy: u8,
}

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("ATI\r\n".as_bytes()).unwrap();
    }
}
