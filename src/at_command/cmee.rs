use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReportMobileEquipmentError;

impl AtRequest for ReportMobileEquipmentError {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CMEE")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("error report response: {=[u8]:a}", data);
        let (setting,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CMEE: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;
        // let setting = match setting {
        //     0 => ReportMobileEquipmentErrorSetting::Disabled,
        //     1 => ReportMobileEquipmentErrorSetting::Enabled,
        //     2 => ReportMobileEquipmentErrorSetting::EnabledVerbose,
        //     _ => return Err(AtError::InvalidResponse),
        // };
        Ok(AtResponse::ReportMobileEquipmentErrorSetting(setting))
    }
}

#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportMobileEquipmentErrorSetting {
    Disabled,
    Numeric,
    EnabledVerbose,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetReportMobileEquipmentError {
    pub setting: ReportMobileEquipmentErrorSetting,
}

impl AtRequest for SetReportMobileEquipmentError {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let setting = match self.setting {
            ReportMobileEquipmentErrorSetting::Disabled => 0,
            ReportMobileEquipmentErrorSetting::Numeric => 1,
            ReportMobileEquipmentErrorSetting::EnabledVerbose => 2,
        };

        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CMEE")
            .with_int_parameter(setting)
            .finish()
    }
}
