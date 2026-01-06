//! Commands to check the PDP context
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;
use at_commands::parser::CommandParser;
#[cfg(feature = "defmt")]
use defmt::debug;

/// The states of the PDP
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum PDPState {
    Deactivated,
    Activated,
}

impl From<i32> for PDPState {
    fn from(value: i32) -> Self {
        match value {
            0 => PDPState::Deactivated,
            1 => PDPState::Activated,
            _ => {
                unreachable!()
            }
        }
    }
}

/// Requests the current PDP context
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct PDPContext;

/// Response containing the PDP state
pub struct PDPContextResponse {
    pub state: PDPState,
    pub context: i32,
}

impl PDPContext {
    fn get_status(data: &[u8]) -> Result<(PDPState, i32), AtError> {
        let (state, context) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CGACT: ")
            .trim_whitespace()
            .expect_int_parameter()
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;
        let state = PDPState::from(state);

        Ok((state, context))
    }
}

impl AtRequest for PDPContext {
    type Response = PDPContextResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CGACT")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        if data.starts_with(b"\r\nOK\r") {
            #[cfg(feature = "defmt")]
            debug!("waiting for PDPContext");
            return Ok(AtResponse::PDPContext(None));
        };
        let (state, context) = Self::get_status(data)?;
        Ok(AtResponse::PDPContext(Some((state, context))))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (state, context) = Self::get_status(data)?;
        Ok(PDPContextResponse { state, context })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pdp_state_from_int() {
        assert_eq!(PDPState::from(0), PDPState::Deactivated);
        assert_eq!(PDPState::from(1), PDPState::Activated);
    }

    #[test]
    fn pdp_context_get_command() {
        let req = PDPContext;
        let mut buffer: [u8; 512] = [0; 512];

        let cmd = req.get_command(&mut buffer).unwrap();

        assert_eq!(cmd, b"AT+CGACT?\r\n");
    }
    #[test]
    fn parse_pdp_context_activated() {
        let data = b"\r\n+CGACT: 1,1\r\n\r\nOK";

        let response = PDPContext.parse_response_struct(data).unwrap();

        assert_eq!(response.state, PDPState::Activated);
        assert_eq!(response.context, 1);
    }

    #[test]
    fn parse_pdp_context_deactivated() {
        let data = b"\r\n+CGACT: 0,3\r\n\r\nOK";

        let response = PDPContext.parse_response_struct(data).unwrap();

        assert_eq!(response.state, PDPState::Deactivated);
        assert_eq!(response.context, 3);
    }

    #[test]
    fn parse_pdp_context_invalid_numbers() {
        let data = b"\r\n+CGACT: a,b\r\n\r\nOK";

        assert!(PDPContext.parse_response_struct(data).is_err());
    }
}
