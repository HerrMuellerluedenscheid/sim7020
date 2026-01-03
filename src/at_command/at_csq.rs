#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SignalQualityReport;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SignalQualityResponse {
    pub rx_signal_strength: i32,
    pub rx_quality: i32,
}

impl SignalQualityReport {
    fn get_signal_response(data: &[u8]) -> Result<(i32, i32), AtError> {
        // \r\n+CSQ: 24,0\r\n\r\nOK\r\n
        // c.f.GSM 05.08, section 8.2.4
        let tuple = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CSQ: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        Ok(tuple)
    }
}

impl AtRequest for SignalQualityReport {
    type Response = SignalQualityResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
