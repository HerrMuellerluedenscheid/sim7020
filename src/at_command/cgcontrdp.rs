//! This module contains the resources to handle the PDP context parameters

#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::warn;

/// Struct to request the PDP context parameters
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PDPContextReadDynamicsParameters;

/// The max size allowed by the APN
const APN_MAX_SIZE: usize = 255;
/// The maximum size that the local address and its mask can have
const LOCAL_ADDRESS_AND_SUBNET_MASK_MAX_SIZE: usize = 255;
/// The maximum size that the gateway can have
const GATEWAY_ADDRESS_MAX_SIZE: usize = 255;
/// The max size of the DNS
const DNS_MAX_SIZE: usize = 128;

/// Struct containig the PDP dynamic parameters
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug,PartialEq)]
pub struct PDPContextReadDynamicsParametersResponse {
    /// The CID
    pub cid: i32,
    /// The bearer ID
    pub bearer_id: i32,
    /// The configured APN
    pub apn: heapless::String<APN_MAX_SIZE>,
    /// The local IP address and the corresponding mask
    pub local_address_and_subnet_mask:
        Option<heapless::String<LOCAL_ADDRESS_AND_SUBNET_MASK_MAX_SIZE>>,
    /// The IP of the gateway
    pub gateway_address: Option<heapless::String<GATEWAY_ADDRESS_MAX_SIZE>>,
    /// The primary DNS address
    pub primary_dns_address: Option<heapless::String<DNS_MAX_SIZE>>,
    /// The secondary DNS address
    pub secondary_dns_address: Option<heapless::String<DNS_MAX_SIZE>>,
    /// The IPv4 max MTU
    pub ipv4_mtu: Option<i32>,
    /// The MTU for non IP
    pub non_ip_mtu: Option<i32>,
    pub serving_plmn_rate_control_value: Option<i32>,
}

impl AtRequest for PDPContextReadDynamicsParameters {
    type Response = Option<PDPContextReadDynamicsParametersResponse>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CGCONTRDP")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        if at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\nOK\r")
            .finish()
            .is_ok()
        {
            #[cfg(feature = "defmt")]
            warn!("return plain ok. No data available, yet");
            return Ok(AtResponse::Ok);
        }

        let (cid, bearer_id, apn, local_address) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGCONTRDP: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_string_parameter()
            .expect_string_parameter()
            .expect_identifier(b"\r\n\r\nOK\r")
            .finish()?;
        Ok(AtResponse::PDPContextDynamicParameters(
            cid as u8,
            bearer_id as u8,
            apn.as_ptr(),
            local_address.as_ptr(),
        ))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        if at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()
            .is_ok()
        {
            #[cfg(feature = "defmt")]
            warn!("return plain ok. No data available, yet");
            return Ok(None);
        }
        let (
            cid,
            bearer_id,
            apn,
            local_address_and_subnet_mask,
            gateway_address,
            primary_dns_address,
            secondary_dns_address,
            ipv4_mtu,
            non_ip_mtu,
            serving_plmn_rate_control_value,
        ) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CGCONTRDP: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .expect_optional_int_parameter()
            .finish()?;

        let apn: heapless::String<APN_MAX_SIZE> = apn.try_into()?;
        let local_address_and_subnet_mask: Option<
            heapless::String<LOCAL_ADDRESS_AND_SUBNET_MASK_MAX_SIZE>,
        > = local_address_and_subnet_mask
            .map(|x| x.try_into())
            .transpose()?;
        let gateway_address: Option<heapless::String<GATEWAY_ADDRESS_MAX_SIZE>> =
            gateway_address.map(|x| x.try_into()).transpose()?;

        let primary_dns_address: Option<heapless::String<DNS_MAX_SIZE>> =
            primary_dns_address.map(|x| x.try_into()).transpose()?;

        let secondary_dns_address: Option<heapless::String<DNS_MAX_SIZE>> =
            secondary_dns_address.map(|x| x.try_into()).transpose()?;

        let response = PDPContextReadDynamicsParametersResponse {
            cid,
            bearer_id,
            apn,
            local_address_and_subnet_mask,
            gateway_address,
            primary_dns_address,
            secondary_dns_address,
            ipv4_mtu,
            non_ip_mtu,
            serving_plmn_rate_control_value,
        };

        Ok(Some(response))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_pdpcontext_read_dynamics_parameters_request() {
        let mut buffer = [0u8;512];

        let request = PDPContextReadDynamicsParameters.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CGCONTRDP=\r\n");
    }

    #[test]
    fn test_pdpcontext_read_dynamic_parameters_response() {
        let data = b"+CGCONTRDP: 1,1,\"APN\",\"127.0.0.1.255.255.255.0\",\"127.0.0.1\",\"127.0.0.1\",\"127.0.0.1\",1,1,\r\n\r\nOK\r\n";

        let response = PDPContextReadDynamicsParameters.parse_response_struct(data).unwrap().unwrap();


        assert_eq!(response,PDPContextReadDynamicsParametersResponse {
            cid: 1,
            bearer_id: 1,
            apn: "APN".try_into().unwrap(),
            local_address_and_subnet_mask: Some("127.0.0.1.255.255.255.0".parse().unwrap()),
            gateway_address: Some("127.0.0.1".parse().unwrap()),
            primary_dns_address: Some("127.0.0.1".parse().unwrap()),
            secondary_dns_address: Some("127.0.0.1".parse().unwrap()),
            ipv4_mtu: Some(1),
            non_ip_mtu: Some(1),
            serving_plmn_rate_control_value: None,
        } );

        let data = b"OK\r\n";

        let response = PDPContextReadDynamicsParameters.parse_response_struct(data).unwrap();

        assert!(response.is_none());
    }
}