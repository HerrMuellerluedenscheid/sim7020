#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReportMobileEquipmentError;

impl ReportMobileEquipmentError {
    fn get_setting(data: &[u8]) -> Result<i32, AtError> {
        let (setting,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CMEE: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;

        return Ok(setting);
    }
}

impl AtRequest for ReportMobileEquipmentError {
    type Response = ReportMobileEquipmentErrorSetting;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CMEE")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("error report response: {=[u8]:a}", data);
        let setting = Self::get_setting(data)?;
        // let setting = match setting {
        //     0 => ReportMobileEquipmentErrorSetting::Disabled,
        //     1 => ReportMobileEquipmentErrorSetting::Enabled,
        //     2 => ReportMobileEquipmentErrorSetting::EnabledVerbose,
        //     _ => return Err(AtError::InvalidResponse),
        // };
        Ok(AtResponse::ReportMobileEquipmentErrorSetting(setting))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let setting = Self::get_setting(data)?;
        let setting: ReportMobileEquipmentErrorSetting = setting.into();

        return Ok(setting);
    }
}

#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportMobileEquipmentErrorSetting {
    Disabled = 0,
    Numeric = 1,
    EnabledVerbose = 2,
}

const REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_DISABLED: i32 = 0;
const REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_NUMERIC: i32 = 1;
const REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_ENABLED_VERBOSE: i32 = 2;

impl From<i32> for ReportMobileEquipmentErrorSetting {
    fn from(value: i32) -> Self {
        match value {
            REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_DISABLED => {
                ReportMobileEquipmentErrorSetting::Disabled
            }
            REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_NUMERIC => {
                ReportMobileEquipmentErrorSetting::Numeric
            }
            REPORT_MOBILE_EQUIPEMENT_ERROR_SETTING_ENABLED_VERBOSE => {
                ReportMobileEquipmentErrorSetting::EnabledVerbose
            }
            _ => unreachable!(),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SetReportMobileEquipmentError {
    pub setting: ReportMobileEquipmentErrorSetting,
}

impl AtRequest for SetReportMobileEquipmentError {
    type Response = ();

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

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}
