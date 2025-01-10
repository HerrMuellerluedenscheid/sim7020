use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryChargeStatus {
    capacity_percent: i32,
    voltage_millivolt: i32,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryCharge;

impl AtRequest for BatteryCharge {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CBC")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (capacity_percent, voltage_millivolt) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CBC: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let status = BatteryChargeStatus {
            capacity_percent,
            voltage_millivolt,
        };
        Ok(AtResponse::BatteryCharge(status))
    }
}
