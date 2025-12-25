#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControlFlowStatus {
    No,
    Software,
    Hardware,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetFlowControl;

#[allow(unused)]
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

        return Ok((dce_by_dte.into(), dte_by_dce.into()));
    }
}

impl AtRequest for GetFlowControl {
    type Response = GetFlowControlResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
            dce_by_dte: dce_by_dte,
            dte_by_dce: dte_by_dce,
        })
    }
}
