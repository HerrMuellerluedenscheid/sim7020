#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
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

impl BatteryCharge {
    fn get_battery_charge_status(data: &[u8]) -> Result<BatteryChargeStatus, AtError> {
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

        return Ok(status);
    }
}

impl AtRequest for BatteryCharge {
    type Response = BatteryChargeStatus;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CBC")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let status = Self::get_battery_charge_status(data)?;
        Ok(AtResponse::BatteryCharge(status))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        return Self::get_battery_charge_status(data);
    }
}
