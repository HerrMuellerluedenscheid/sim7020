use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use defmt::Format;

#[derive(Format)]
pub struct SignalQualityReport;

impl AtRequest for SignalQualityReport {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CSQ")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        // \r\n+CSQ: 24,0\r\n\r\nOK\r\n
        // c.f.GSM 05.08, section 8.2.4
        let (rx_signal_strength, rx_quality) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CSQ: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()
            .unwrap();
        Ok(AtResponse::SignalQuality(rx_signal_strength, rx_quality))
    }
}
