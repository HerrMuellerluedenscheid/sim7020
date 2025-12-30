#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::parser::CommandParser;
#[cfg(feature = "defmt")]
use defmt::info;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct LocalIPAddress;

const MAX_IP_SIZE: usize = 39;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct LocalIpAddressResponse {
    pub ip: heapless::String<MAX_IP_SIZE>,
}

impl AtRequest for LocalIPAddress {
    type Response = LocalIpAddressResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CIFSR")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (local_ip,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CIFSR: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        #[cfg(feature = "defmt")]
        info!("localip: {}", local_ip);
        Ok(AtResponse::LocalIPAddress(local_ip))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (local_ip,) = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CIFSR: ")
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        #[cfg(feature = "defmt")]
        info!("localip: {}", local_ip);
        let ip: heapless::String<MAX_IP_SIZE> = local_ip.try_into()?;

        Ok(LocalIpAddressResponse { ip })
    }
}
