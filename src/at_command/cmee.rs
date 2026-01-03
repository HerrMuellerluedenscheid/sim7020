//! Module for the report of mobile equipment error

#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{verify_ok, AtRequest, BufferType};
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::info;

/// Command to request the error report
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct ReportMobileEquipmentError;

impl ReportMobileEquipmentError {
    fn get_setting(data: &[u8]) -> Result<i32, AtError> {
        let (setting,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CMEE: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;

        Ok(setting)
    }
}

impl AtRequest for ReportMobileEquipmentError {
    type Response = ReportMobileEquipmentErrorSetting;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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

        Ok(setting)
    }
}

/// The settings for the mobile equipment error
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum ReportMobileEquipmentErrorSetting {
    Disabled = 0,
    Numeric = 1,
    EnabledVerbose = 2,
}

const REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_DISABLED: i32 = 0;
const REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_NUMERIC: i32 = 1;
const REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_ENABLED_VERBOSE: i32 = 2;

impl From<i32> for ReportMobileEquipmentErrorSetting {
    fn from(value: i32) -> Self {
        match value {
            REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_DISABLED => {
                ReportMobileEquipmentErrorSetting::Disabled
            }
            REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_NUMERIC => {
                ReportMobileEquipmentErrorSetting::Numeric
            }
            REPORT_MOBILE_EQUIPMENT_ERROR_SETTING_ENABLED_VERBOSE => {
                ReportMobileEquipmentErrorSetting::EnabledVerbose
            }
            _ => unreachable!(),
        }
    }
}

/// Command to set the configuration of the report for mobile equipment error
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SetReportMobileEquipmentError {
    pub setting: ReportMobileEquipmentErrorSetting,
}

impl AtRequest for SetReportMobileEquipmentError {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cmee_get_command() {
        let cmd = ReportMobileEquipmentError;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CMEE?\r\n");
    }

    #[test]
    fn cmee_parse_disabled() {
        let cmd = ReportMobileEquipmentError;

        let data = b"\r\n+CMEE: 0\r\n\r\nOK\r\n";

        let setting = cmd.parse_response_struct(data).unwrap();

        assert_eq!(setting, ReportMobileEquipmentErrorSetting::Disabled);
    }

    #[test]
    fn cmee_parse_numeric() {
        let cmd = ReportMobileEquipmentError;

        let data = b"\r\n+CMEE: 1\r\n\r\nOK\r\n";

        let setting = cmd.parse_response_struct(data).unwrap();

        assert_eq!(setting, ReportMobileEquipmentErrorSetting::Numeric);
    }

    #[test]
    fn cmee_parse_enabled_verbose() {
        let cmd = ReportMobileEquipmentError;

        let data = b"\r\n+CMEE: 2\r\n\r\nOK\r\n";

        let setting = cmd.parse_response_struct(data).unwrap();

        assert_eq!(setting, ReportMobileEquipmentErrorSetting::EnabledVerbose);
    }

    #[test]
    fn cmee_parse_fails_without_ok() {
        let cmd = ReportMobileEquipmentError;

        let data = b"\r\n+CMEE: 1\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }

    #[test]
    #[should_panic]
    fn cmee_parse_panics_on_invalid_setting() {
        let cmd = ReportMobileEquipmentError;

        // get_setting() returns i32, but From<i32> uses unreachable!()
        let data = b"\r\n+CMEE: 99\r\n\r\nOK\r\n";

        let _ = cmd.parse_response_struct(data).unwrap();
    }

    #[test]
    fn cmee_setting_from_i32() {
        assert_eq!(
            ReportMobileEquipmentErrorSetting::from(0),
            ReportMobileEquipmentErrorSetting::Disabled
        );
        assert_eq!(
            ReportMobileEquipmentErrorSetting::from(1),
            ReportMobileEquipmentErrorSetting::Numeric
        );
        assert_eq!(
            ReportMobileEquipmentErrorSetting::from(2),
            ReportMobileEquipmentErrorSetting::EnabledVerbose
        );
    }

    #[test]
    fn set_cmee_disabled_command() {
        let cmd = SetReportMobileEquipmentError {
            setting: ReportMobileEquipmentErrorSetting::Disabled,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CMEE=0\r\n");
    }

    #[test]
    fn set_cmee_numeric_command() {
        let cmd = SetReportMobileEquipmentError {
            setting: ReportMobileEquipmentErrorSetting::Numeric,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CMEE=1\r\n");
    }

    #[test]
    fn set_cmee_enabled_verbose_command() {
        let cmd = SetReportMobileEquipmentError {
            setting: ReportMobileEquipmentErrorSetting::EnabledVerbose,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CMEE=2\r\n");
    }

    #[test]
    fn set_cmee_parse_response_ok() {
        let cmd = SetReportMobileEquipmentError {
            setting: ReportMobileEquipmentErrorSetting::Numeric,
        };

        let result = cmd.parse_response_struct(b"OK\r\n");

        assert!(result.is_ok());
    }
}
