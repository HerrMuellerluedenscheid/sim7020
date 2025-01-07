use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FlowControl{
    No,
    Software,
    Hardware
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetFlowControl{
    pub(crate) ta_to_te: FlowControl,
    pub(crate) te_to_ta: FlowControl
}

impl FlowControl{
    fn to_int(&self) -> i32{
        match self {
            FlowControl::No => {0},
            FlowControl::Software => {1},
            FlowControl::Hardware => {2},
        }
    }
}

impl AtRequest for SetFlowControl {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+IFC")
            .with_int_parameter(self.ta_to_te.to_int())
            .with_int_parameter(self.te_to_ta.to_int())
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetFlowControl;

impl AtRequest for GetFlowControl {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+IFC")
            .finish()
    }

    /// dce_by_dte: method that will be used by TE at receive of data
    ///             from TA
    /// dte_by_dce: Specifies the method will be used by TA at receive of data
    ///             from TE
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (dce_by_dte, dte_by_dce, ra) = CommandParser::parse(data)
            .expect_optional_identifier(b"AT+IFC?\r")
            .expect_identifier(b"\r\n+IFC: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .finish()?;
        let dce_by_dte = match dce_by_dte {
            0 => FlowControl::No,
            1 => FlowControl::Software,
            2 => FlowControl::Hardware,
            _ => panic!("Invalid dce-by-dte parameter returned"),
        };
        let dte_by_dce = match dte_by_dce {
            0 => FlowControl::No,
            1 => FlowControl::Software,
            2 => FlowControl::Hardware,
            _ => panic!("Invalid dce-by-dte parameter returned"),
        };
        info!("parity {}, {}", dce_by_dte, dte_by_dce);
        Ok(AtResponse::Ok {})
    }
}