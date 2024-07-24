use crate::at_command::AtRequest;
use crate::AtError;
use defmt::Format;
use embedded_io::Write;

#[derive(Format)]
/// TA returns a list of quadruplets, each representing an operator present in
/// the network. Any of the formats may be unavailable and should then be an
/// empty field. The list of operators shall be in order: home network,
/// networks referenced in SIM, and other networks.
pub struct NetworkInformationAvailable;

impl AtRequest for NetworkInformationAvailable {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+COPS=?\r\n".as_bytes()).unwrap();
    }
}

#[derive(Format)]
/// Current mode and the currently selected operator
pub struct NetworkInformation;

impl AtRequest for NetworkInformation {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        writer.write("AT+COPS?\r\n".as_bytes()).unwrap();
    }
}
