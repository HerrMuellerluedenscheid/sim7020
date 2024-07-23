use crate::at_command::AtRequest;
use crate::{AtError};
use defmt::{Format, info};

#[derive(Format)]
pub struct At;

impl AtRequest for At {
    type Response = Result<(), AtError>;
    fn send<T>(&self, writer: &mut T) {
        // writer.write(b"AT\r\n").expect("TODO: panic message");
        info!("skiooing write");
    }
}
