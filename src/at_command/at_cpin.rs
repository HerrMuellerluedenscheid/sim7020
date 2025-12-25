#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Test if a pin is required.
pub struct PINRequired;

/// Indicates the response of [PinRequired]
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PinStatus {
    Ready,
    SimPin,
    SimPuk,
    PhSimPin,
    PhSimPuk,
    SimPin2,
    SimPuk2,
    PhNetPin,
    PhNetsubPin,
    PhSpPin,
    PhCorpPin,
    Unkown,
}

/// Associated string to [PinStatus]::Ready
const PIN_STATUS_READY: &str = "READY";

/// Associated string to [PinStatus]::SIM_PIN
const PIN_STATUS_SIM_PIN: &str = "SIM PIN";

/// Associated string to [PinStatus]::SIM_PUK
const PIN_STATUS_SIM_PUK: &str = "SIM PUK";

/// Associated string to [PinStatus]::PH_SIM_PIN
const PIN_STATUS_PH_SIM_PIN: &str = "PH_SIM PIN";

/// Associated string to [PinStatus]::PH_SIM_PUK
const PIN_STATUS_PH_SIM_PUK: &str = "PH_SIM PUK";

/// Associated string to [PinStatus]::SIM_PIN2
const PIN_STATUS_SIM_PIN2: &str = "SIM PIN2";

/// Associated string to [PinStatus]::SIM_PUK2
const PIN_STATUS_SIM_PUK2: &str = "SIM PUK2";

/// Associated string to [PinStatus]::PH_NET_PIN
const PIN_STATUS_PH_NET_PIN: &str = "PH-NET PIN";

/// Associated string to [PinStatus]::PH_NETSUB_PIN
const PIN_STATUS_PH_NETSUB_PIN: &str = "PH-NETSUB PIN";

/// Associated string to [PinStatus]::PH_SP_PIN
const PIN_STATUS_PH_SP_PIN: &str = "PH-SP PIN";

/// Associated string to [PinStatus]::PH_CORP_PIN
const PIN_STATUS_PH_CORP_PIN: &str = "PH-CORP PIN";

impl From<&str> for PinStatus {
    fn from(value: &str) -> Self {
        match value {
            PIN_STATUS_READY => PinStatus::Ready,
            PIN_STATUS_SIM_PIN => PinStatus::SimPin,
            PIN_STATUS_SIM_PUK => PinStatus::SimPuk,
            PIN_STATUS_PH_SIM_PIN => PinStatus::PhSimPin,
            PIN_STATUS_PH_SIM_PUK => PinStatus::PhSimPuk,
            PIN_STATUS_SIM_PIN2 => PinStatus::SimPin2,
            PIN_STATUS_SIM_PUK2 => PinStatus::SimPuk2,
            PIN_STATUS_PH_NET_PIN => PinStatus::PhNetPin,
            PIN_STATUS_PH_NETSUB_PIN => PinStatus::PhNetsubPin,
            PIN_STATUS_PH_SP_PIN => PinStatus::PhSpPin,
            PIN_STATUS_PH_CORP_PIN => PinStatus::PhCorpPin,
            _ => Self::Unkown,
        }
    }
}

impl PINRequired {
    fn get_pin_response(data: &[u8]) -> Result<PinStatus, AtError> {
        let response_code = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"+CPIN: ")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        let pin_status: PinStatus = response_code.0.into();

        return Ok(pin_status);
    }
}

impl AtRequest for PINRequired {
    type Response = PinStatus;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CPIN")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<super::AtResponse, AtError> {
        let pin_status = Self::get_pin_response(&data)?;
        Ok(AtResponse::PinStatus(pin_status))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        Self::get_pin_response(&data)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Enter PIN.
pub struct EnterPIN {
    pub pin: u16,
}

impl AtRequest for EnterPIN {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CPIN")
            .with_int_parameter(self.pin)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        // TODO: Check that the command is an OK
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #![allow(deprecated)]
    use super::*;

    #[test]
    fn test_parse_pin_ready() -> Result<(), AtError> {
        let req = PINRequired;

        let data = b"+CPIN: READY\r\n\r\nOK\r\n";

        let response = req.parse_response(data)?;

        match response {
            AtResponse::PinStatus(status) => {
                assert_eq!(status, PinStatus::Ready);
            }
            _ => panic!("Unexpected response type"),
        }

        return Ok(());
    }

    #[test]
    fn test_parse_pin_sim_pin() -> Result<(), AtError> {
        let req = PINRequired;

        let data = b"+CPIN: SIM PIN\r\n\r\nOK\r\n";

        let response = req.parse_response(data)?;

        match response {
            AtResponse::PinStatus(status) => {
                assert_eq!(status, PinStatus::SimPin);
            }
            _ => panic!("Unexpected response type"),
        }

        return Ok(());
    }

    #[test]
    fn test_parse_pin_ph_net_pin() -> Result<(), AtError> {
        let req = PINRequired;

        let data = b"+CPIN: PH-NET PIN\r\n\r\nOK\r\n";

        let response = req.parse_response(data)?;

        match response {
            AtResponse::PinStatus(status) => {
                assert_eq!(status, PinStatus::PhNetPin);
            }
            _ => panic!("Unexpected response type"),
        }

        return Ok(());
    }

    #[test]
    fn test_parse_pin_unknown_status() -> Result<(), AtError> {
        let req = PINRequired;

        let data = b"+CPIN: FOO\r\n\r\nOK\r\n";

        let response = req.parse_response(data)?;

        match response {
            AtResponse::PinStatus(status) => {
                assert_eq!(status, PinStatus::Unkown);
            }
            _ => panic!("Unexpected response type"),
        }

        return Ok(());
    }

    #[test]
    fn test_parse_pin_malformed_response() {
        let req = PINRequired;

        // No parameter after +CPIN
        let data = b"+CPIN\r\n\r\nOK\r\n";

        let result = req.parse_response(data);

        assert!(result.is_err());
    }
}
