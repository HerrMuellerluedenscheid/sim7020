use core::net::IpAddr;
use at_commands::parser::CommandParser;
use crate::at_command::unsolicited_at_responses::AtUnsolicitedResponse;
use crate::AtError;

/// Enum containing the DNSErrors
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum DNSErrors {
    DnsCommonError,
    NetworkError,
    Unknown
}
///Error code for common DNS errors
const DNS_COMMON_ERROR: u8 = 8;

/// Error code for network errors on DNS
const DNS_NETWORK_ERROR: u8 = 3;

impl From<i32> for DNSErrors {
    fn from(value: i32) -> Self {
        let value = value as u8;
        match value {
            DNS_COMMON_ERROR => Self::DnsCommonError,
            DNS_NETWORK_ERROR => Self::NetworkError,
            _ => Self::Unknown
        }
    }
}

/// Contains the information of a DNS query response.
/// The size of the domain will default to 64 and can be configured
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CDNSIPResponse<const N: usize = 64> {
    pub domain: heapless::String<N>,
    pub ip1: IpAddr,
    pub ip2: Option<IpAddr>,
}


impl<const N: usize> AtUnsolicitedResponse for CDNSIPResponse<N> {
    fn parse_response_struct(data: &[u8]) -> Result<Self, AtError> {
        let result = CommandParser::parse(data)
            .trim_whitespace()
            // Check we have the 1 indicating the OK
            .expect_identifier(b"+CDNSGIP: 1,")
            .trim_whitespace()
            .expect_string_parameter()
            .expect_string_parameter()
            .expect_optional_string_parameter()
            .finish();

        if let Ok((domain, ip1, ip2)) = result {
            let domain : heapless::String<N> = domain.try_into()?;
            let ip1: IpAddr = ip1.parse().map_err(|_| AtError::AtParseError)?;
            let ip2: Option<IpAddr> = ip2.map(|addr| addr.parse()).transpose().map_err(|_| AtError::AtParseError)?;

            Ok(Self {
                domain,
                ip1,
                ip2
            })
        } else {
            let (dns_error_code,) = CommandParser::parse(data)
                .trim_whitespace()
                // Check we have the 1 indicating the OK
                .expect_identifier(b"+CDNSGIP: 0,")
                .trim_whitespace()
                .expect_int_parameter().finish()?;

            let dns_error_code = dns_error_code.into();

            Err(AtError::DNSError(dns_error_code))
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use core::net::IpAddr;

    #[test]
    fn test_dns_error_from_common() {
        let err = DNSErrors::from(8);
        assert_eq!(err, DNSErrors::DnsCommonError);
    }

    #[test]
    fn test_dns_error_from_network() {
        let err = DNSErrors::from(3);
        assert_eq!(err, DNSErrors::NetworkError);
    }

    #[test]
    fn test_dns_error_from_unknown() {
        let err = DNSErrors::from(42);
        assert_eq!(err, DNSErrors::Unknown);
    }

    #[test]
    fn test_parse_response_single_ip() {
        let data = b"+CDNSGIP: 1,\"example.com\",\"127.0.0.1\"\r\n";

        let result = CDNSIPResponse::<64>::parse_response_struct(data).unwrap();

        let domain = result.domain.as_str();
        assert_eq!(domain, "example.com");
        assert_eq!(result.ip1, IpAddr::from_str("127.0.0.1").unwrap());
        assert_eq!(result.ip2, None);
    }

    #[test]
    fn test_parse_response_two_ips() {
        let data = b"+CDNSGIP: 1,\"example.com\",\"127.0.0.1\",\"127.0.0.1\"\r\n";

        let result = CDNSIPResponse::<64>::parse_response_struct(data).unwrap();

        assert_eq!(result.domain.as_str(), "example.com");
        assert_eq!(result.ip1, IpAddr::from_str("127.0.0.1").unwrap());
        assert_eq!(
            result.ip2,
            Some(IpAddr::from_str("127.0.0.1").unwrap())
        );
    }

    #[test]
    fn test_parse_dns_error_common() {
        let data = b"+CDNSGIP: 0,8\r\n";

        let result = CDNSIPResponse::<64>::parse_response_struct(data);

        match result {
            Err(AtError::DNSError(err)) => {
                assert_eq!(err, DNSErrors::DnsCommonError);
            }
            _ => panic!("Expected DNS common error"),
        }
    }

    #[test]
    fn test_parse_dns_error_network() {
        let data = b"+CDNSGIP: 0,3\r\n";

        let result = CDNSIPResponse::<64>::parse_response_struct(data);

        match result {
            Err(AtError::DNSError(err)) => {
                assert_eq!(err, DNSErrors::NetworkError);
            }
            _ => panic!("Expected DNS network error"),
        }
    }

    #[test]
    fn test_parse_invalid_ip() {
        let data = b"+CDNSGIP: 1,\"example.com\",\"not_an_ip\"\r\n";

        let result = CDNSIPResponse::<64>::parse_response_struct(data);

        assert!(matches!(result, Err(AtError::AtParseError)));
    }

    #[test]
    fn test_parse_invalid_format() {
        let data = b"garbage data";

        let result = CDNSIPResponse::<64>::parse_response_struct(data);

        assert!(result.is_err());
    }
}