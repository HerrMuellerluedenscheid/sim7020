//! This module contains the structs to get the battery information

#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

/// Current status of the battery
#[allow(dead_code)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryChargeStatus {
    /// Capacity of the battery in percent
    capacity_percent: i32,
    /// Voltage of the battery in mV
    voltage_millivolt: i32,
}

/// Command to get the battery information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BatteryCharge;

impl BatteryCharge {
    fn get_battery_charge_status(data: &[u8]) -> Result<BatteryChargeStatus, AtError> {
        let (capacity_percent, voltage_millivolt) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CBC: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        let status = BatteryChargeStatus {
            capacity_percent,
            voltage_millivolt,
        };

        Ok(status)
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
        Self::get_battery_charge_status(data)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_battery_charge_request() {
        let mut buffer = [0u8; 512];

        let data = BatteryCharge.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT+CBC\r\n");
    }

    #[test]
    fn test_battery_charge_response() {
        let data = b"\r\n+CBC: 10,10\r\n\r\nOK\r\n";

        let response = BatteryCharge.parse_response_struct(data).unwrap();

        assert_eq!(response.capacity_percent, 10);
        assert_eq!(response.voltage_millivolt, 10);
    }
}
