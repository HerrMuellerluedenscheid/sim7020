//! Module to handle the Extended Report functionality
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::info;

/// Command to execute the extended report
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct ExtendedErrorReport;

impl AtRequest for ExtendedErrorReport {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CEER")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("error report response: {=[u8]:a}", _data);
        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        info!("error report response: {=[u8]:a}", _data);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extended_report_request() {
        let mut buffer = [0u8; 512];

        let data = ExtendedErrorReport.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT+CEER\r\n");
    }

    #[test]
    fn test_extended_report_response() {
        let mut buffer = [0u8; 512];

        ExtendedErrorReport.parse_response_struct(&buffer).unwrap();
    }
}
