use core::net::IpAddr;
use at_commands::parser::CommandParser;
use defmt::debug;
use crate::at_command::AtRequest;
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub enum DNSErrors {
    DnsCommonError,
    NetworkError
}

/// Implementation of the command AT+CDNSGIP which allows performing a DNS query
/// The query response will arrive by an unsolicited message
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CDNSGIP<'a> {
    pub domain: &'a str,
}

pub struct CDNSIPResponse {
    pub ip1: IpAddr,
    pub ip2: Option<IpAddr>,
}


impl AtRequest for CDNSGIP<'_> {
    type Response = CDNSIPResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CDNSGIP")
            .with_string_parameter(self.domain)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        debug!("The received data is: {:?}", data);
        todo!()
    }
}
