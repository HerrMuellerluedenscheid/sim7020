//! This module contains the required implementations to get the signal report
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;

/// Queries the signal report
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SignalQualityReport;

/// Contains the response from the signal report
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct SignalQualityResponse {
    pub rx_signal_strength: i32,
    pub rx_quality: i32,
}

impl SignalQualityReport {
    fn get_signal_response(data: &[u8]) -> Result<(i32, i32), AtError> {
        // \r\n+CSQ: 24,0\r\n\r\nOK\r\n
        // c.f.GSM 05.08, section 8.2.4
        let tuple = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CSQ: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        Ok(tuple)
    }
}

impl AtRequest for SignalQualityReport {
    type Response = SignalQualityResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CSQ")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (rx_signal_strength, rx_quality) = Self::get_signal_response(data)?;
        Ok(AtResponse::SignalQuality(rx_signal_strength, rx_quality))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (rx_signal_strength, rx_quality) = Self::get_signal_response(data)?;
        Ok(SignalQualityResponse {
            rx_quality,
            rx_signal_strength,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_signal_quality_request() {
        let mut buffer = [0u8; 512];

        let data = SignalQualityReport.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT+CSQ\r\n");
    }

    #[test]
    fn test_signal_quality_response() {
        let buffer = b"\r\n+CSQ: 0,0\r\n\r\nOK\r\n";

        let data = SignalQualityReport.parse_response_struct(buffer).unwrap();

        assert_eq!(
            data,
            SignalQualityResponse {
                rx_quality: 0,
                rx_signal_strength: 0,
            }
        )
    }
}
