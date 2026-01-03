//! Module for the product information

use crate::at_command::{AtRequest, BufferType};

/// Request for the module information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtI;

/// The max size allowed by a product information
const PRODUCT_INFORMATION_MAX_SIZE: usize = 64;

/// Struct containing the product information
pub struct ProductInformation {
    name: heapless::String<PRODUCT_INFORMATION_MAX_SIZE>,
}

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("I")
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, crate::AtError> {
        let (name,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        let name: heapless::String<PRODUCT_INFORMATION_MAX_SIZE> = name.try_into()?;

        Ok(ProductInformation { name })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn ati_get_command() {
        let cmd = AtI;
        let mut buffer = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"ATI?\r\n");
    }

    #[test]
    fn ati_parse_valid_response() {
        let cmd = AtI;

        let data = b"SIM7020 R1752\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.name.as_str(), "SIM7020 R1752");
    }

    #[test]
    fn ati_parse_fails_on_empty_name() {
        let cmd = AtI;

        let data = b"\r\nOK\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }

    #[test]
    fn ati_parse_fails_when_name_too_long() {
        let cmd = AtI;

        // 65 bytes (PRODUCT_INFORMATION_MAX_SIZE is 64)
        let long_name = [b'A'; PRODUCT_INFORMATION_MAX_SIZE + 1];
        let mut data = heapless::Vec::<u8, 128>::new();

        data.extend_from_slice(&long_name).unwrap();
        data.extend_from_slice(b"\r\nOK\r\n").unwrap();

        let result = cmd.parse_response_struct(&data);

        assert!(result.is_err());
    }
}
