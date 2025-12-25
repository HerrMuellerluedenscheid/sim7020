#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg(feature = "defmt")]
use defmt::warn;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PDPContextReadDynamicsParameters;

const APN_MAX_SIZE: usize = 255;
const LOCAL_ADDRESS_AND_SUBNET_MASK_MAX_SIZE: usize = 255;
const GATEWAY_ADDRESS_MAX_SIZE: usize = 255;
const DNS_MAX_SIZE: usize = 128;

pub struct PDPContextReadDynamicsParametersResponse {
    pub cid: i32,
    pub bearer_id: i32,
    pub apn: heapless::String<APN_MAX_SIZE>,
    pub local_address_and_subnet_mask:
        Option<heapless::String<LOCAL_ADDRESS_AND_SUBNET_MASK_MAX_SIZE>>,
    pub gateway_address: Option<heapless::String<GATEWAY_ADDRESS_MAX_SIZE>>,
    pub primary_dns_address: Option<heapless::String<DNS_MAX_SIZE>>,
    pub secondary_dns_address: Option<heapless::String<DNS_MAX_SIZE>>,
    pub ipv4_mtu: Option<i32>,
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
            .expect_identifier(b"\r\nOK\r")
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
            .expect_identifier(b"\r\n+CGCONTRDP: ")
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
