use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StartQueryNTP<'a> {
    pub url: &'a str,
    pub tzinfo: Option<&'a str>, // currently not implemented
}

impl AtRequest for StartQueryNTP<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        match &self.tzinfo {
            None => at_commands::builder::CommandBuilder::create_set(buffer, true)
                .named("+CSNTPSTART")
                .with_string_parameter(self.url)
                .finish(),
            Some(tzinfo) => at_commands::builder::CommandBuilder::create_set(buffer, true)
                .named("+CSNTPSTART")
                .with_string_parameter(self.url)
                .with_string_parameter(tzinfo)
                .finish(),
        }
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StopQueryNTP;

impl AtRequest for StopQueryNTP {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CSNTPSTOP")
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}
