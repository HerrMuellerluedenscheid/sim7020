use crate::at_command::AtRequest;
use crate::AtError;
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
/// Test if a pin is required.
pub struct PINRequired;

impl AtRequest for PINRequired {
    type Response = Result<(), AtError>;

    fn send<T: embedded_io::Write>(&self, writer: &mut T) {
        writer.write("AT+CPIN?\r\n".as_bytes()).unwrap();
    }
}

#[derive(Format)]
/// Enter PIN.
pub struct EnterPIN {
    pin: u16,
}

impl AtRequest for EnterPIN {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        let pin = self.pin;
        writer.write("AT+CPIN=".as_bytes()).unwrap();
        writer.write(&pin.to_be_bytes()).expect("failed writing pin");
        writer.write("\r\n".as_bytes()).unwrap();
    }
}
