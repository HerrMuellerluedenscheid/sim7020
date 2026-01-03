//! Module to configure and check the UART control flow

#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::info;

/// Possible configuration of the control flow
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum ControlFlowStatus {
    No,
    Software,
    Hardware,
}

/// Command to set the control flow configuration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SetFlowControl {
    pub(crate) ta_to_te: ControlFlowStatus,
    pub(crate) te_to_ta: ControlFlowStatus,
}

const NO_FLOW_CONTROL: i32 = 0;
const SOFTWARE_FLOW_CONTROL: i32 = 1;
const HARDWARE_FLOW_CONTROL: i32 = 2;

impl ControlFlowStatus {
    fn to_int(&self) -> i32 {
        match self {
            ControlFlowStatus::No => NO_FLOW_CONTROL,
            ControlFlowStatus::Software => SOFTWARE_FLOW_CONTROL,
            ControlFlowStatus::Hardware => HARDWARE_FLOW_CONTROL,
        }
    }
}

impl From<ControlFlowStatus> for i32 {
    #[inline]
    fn from(value: ControlFlowStatus) -> Self {
        value.to_int()
    }
}

impl From<i32> for ControlFlowStatus {
    fn from(value: i32) -> Self {
        match value {
            NO_FLOW_CONTROL => ControlFlowStatus::No,
            SOFTWARE_FLOW_CONTROL => ControlFlowStatus::Software,
            HARDWARE_FLOW_CONTROL => ControlFlowStatus::Hardware,
            _ => unreachable!(),
        }
    }
}

impl AtRequest for SetFlowControl {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+IFC")
            .with_int_parameter(self.ta_to_te.to_int())
            .with_int_parameter(self.te_to_ta.to_int())
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Command to get the flow control configuration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GetFlowControl;

/// Response of [GetFlowControl]
#[allow(unused)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GetFlowControlResponse {
    pub dce_by_dte: ControlFlowStatus,
    pub dte_by_dce: ControlFlowStatus,
}

impl GetFlowControl {
    fn parse_data(data: &[u8]) -> Result<(ControlFlowStatus, ControlFlowStatus), AtError> {
        let (dce_by_dte, dte_by_dce, _) = CommandParser::parse(data)
            .expect_optional_identifier(b"AT+IFC?\r")
            .expect_identifier(b"\r\n+IFC: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .finish()?;

        Ok((dce_by_dte.into(), dte_by_dce.into()))
    }
}

impl AtRequest for GetFlowControl {
    type Response = GetFlowControlResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+IFC")
            .finish()
    }

    /// dce_by_dte: method that will be used by TE at receive of data
    ///             from TA
    /// dte_by_dce: Specifies the method will be used by TA at receive of data
    ///             from TE
    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (dce_by_dte, dte_by_dce) = Self::parse_data(data)?;
        #[cfg(feature = "defmt")]
        info!("parity {}, {}", dce_by_dte, dte_by_dce);
        Ok(AtResponse::ControlFlow(dce_by_dte, dte_by_dce))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (dce_by_dte, dte_by_dce) = Self::parse_data(data)?;
        Ok(GetFlowControlResponse {
            dce_by_dte,
            dte_by_dce,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn control_flow_status_to_int() {
        assert_eq!(ControlFlowStatus::No.to_int(), 0);
        assert_eq!(ControlFlowStatus::Software.to_int(), 1);
        assert_eq!(ControlFlowStatus::Hardware.to_int(), 2);
    }

    #[test]
    fn control_flow_status_from_int() {
        assert_eq!(ControlFlowStatus::from(0), ControlFlowStatus::No);
        assert_eq!(ControlFlowStatus::from(1), ControlFlowStatus::Software);
        assert_eq!(ControlFlowStatus::from(2), ControlFlowStatus::Hardware);
    }

    #[test]
    #[should_panic]
    fn control_flow_status_from_invalid_int_panics() {
        let _ = ControlFlowStatus::from(99);
    }

    #[test]
    fn set_flow_control_no_no() {
        let cmd = SetFlowControl {
            ta_to_te: ControlFlowStatus::No,
            te_to_ta: ControlFlowStatus::No,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+IFC=0,0\r\n");
    }

    #[test]
    fn set_flow_control_sw_hw() {
        let cmd = SetFlowControl {
            ta_to_te: ControlFlowStatus::Software,
            te_to_ta: ControlFlowStatus::Hardware,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+IFC=1,2\r\n");
    }

    #[test]
    fn set_flow_control_hw_sw() {
        let cmd = SetFlowControl {
            ta_to_te: ControlFlowStatus::Hardware,
            te_to_ta: ControlFlowStatus::Software,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+IFC=2,1\r\n");
    }

    #[test]
    fn set_flow_control_parse_ok() {
        let cmd = SetFlowControl {
            ta_to_te: ControlFlowStatus::No,
            te_to_ta: ControlFlowStatus::Software,
        };

        let result = cmd.parse_response_struct(b"\r\nOK\r\n");

        assert!(result.is_ok());
    }

    #[test]
    fn get_flow_control_command() {
        let cmd = GetFlowControl;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+IFC?\r\n");
    }

    #[test]
    fn get_flow_control_parse_valid_response() {
        let cmd = GetFlowControl;

        let data = b"\r\n+IFC: 1,2\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.dce_by_dte, ControlFlowStatus::Software);
        assert_eq!(response.dte_by_dce, ControlFlowStatus::Hardware);
    }

    #[test]
    fn get_flow_control_parse_with_echoed_command() {
        let cmd = GetFlowControl;

        let data = b"AT+IFC?\r\r\n+IFC: 2,1\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.dce_by_dte, ControlFlowStatus::Hardware);
        assert_eq!(response.dte_by_dce, ControlFlowStatus::Software);
    }
}
