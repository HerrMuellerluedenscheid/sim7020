use crate::at_command::{AtRequest, AtResponse};
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use defmt::Format;

#[derive(Format)]
/// TA returns a list of quadruplets, each representing an operator present in
/// the network. Any of the formats may be unavailable and should then be an
/// empty field. The list of operators shall be in order: home network,
/// networks referenced in SIM, and other networks.
pub struct NetworkInformationAvailable;

impl AtRequest for NetworkInformationAvailable {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+COPS=?\r\n").unwrap();
    }
}

#[derive(Format)]
/// Current mode and the currently selected operator
pub struct NetworkInformation;

impl AtRequest for NetworkInformation {
    type Response = Result<(), AtError>;

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+COPS?\r\n").unwrap();
    }
}
