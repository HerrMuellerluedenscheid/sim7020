use core::net::IpAddr;
use defmt::debug;
use crate::at_command::{verify_ok, AtRequest};
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

/// Contains the information of a DNS query response.
/// The size of the domain will default to 64 and can be configured
pub struct CDNSIPResponse<const N: usize = 64> {
    pub domain: heapless::String<N>,
    pub ip1: IpAddr,
    pub ip2: Option<IpAddr>,
}


impl AtRequest for CDNSGIP<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CDNSGIP")
            .with_string_parameter(self.domain)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        debug!("The received data is: {:?}", data);
        verify_ok(data)
    }
}
