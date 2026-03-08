use crate::at_command::{verify_ok, AtRequest};
use crate::AtError;
use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;
use core::net::IpAddr;

/// Allows to query the DNS configuration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct QueryCDNSCFG;

/// Response for the query of the configured DNSs
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CDNSCFGResponse {
    pub primary_dns: IpAddr,
    pub secondary_dns: IpAddr,
}

impl AtRequest for QueryCDNSCFG {
    type Response = CDNSCFGResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        CommandBuilder::create_query(buffer, true)
            .named("+CDNSCFG")
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (primary, secondary) = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"PrimaryDns:")
            .trim_whitespace()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"SecondaryDns:")
            .trim_whitespace()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .trim_whitespace()
            .finish()?;

        let primary_dns = primary.parse().map_err(|_| AtError::AtParseError)?;
        let secondary_dns = secondary.parse().map_err(|_| AtError::AtParseError)?;

        Ok(CDNSCFGResponse {
            primary_dns,
            secondary_dns,
        })
    }
}

/// Configures the DNS
pub struct ConfigureDns<'a> {
    pub primary_dns: &'a str,
    pub secondary_dns: Option<&'a str>,
}

impl AtRequest for ConfigureDns<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        let mut builder = CommandBuilder::create_set(buffer, true)
            .named("+CDNSCFG")
            .with_string_parameter(self.primary_dns);

        if let Some(secondary_dns) = self.secondary_dns {
            builder = builder.with_string_parameter(secondary_dns);
        }

        builder.finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(&data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::net::IpAddr;
    use core::str::FromStr;

    // -------------------------
    // QueryCDNSCFG tests
    // -------------------------

    #[test]
    fn test_query_cdnscfg_command() {
        let cmd = QueryCDNSCFG;
        let mut buffer = [0u8; 32];

        let result = cmd.get_command(&mut buffer).unwrap();
        let command = core::str::from_utf8(result).unwrap();

        assert_eq!(command, "AT+CDNSCFG?\r\n");
    }

    #[test]
    fn test_query_cdnscfg_parse_valid() {
        let cmd = QueryCDNSCFG;

        let data = b"PrimaryDns: 8.8.8.8\r\nSecondaryDns: 8.8.4.4\r\nOK";

        let resp = cmd.parse_response_struct(data).unwrap();

        assert_eq!(resp.primary_dns, IpAddr::from_str("8.8.8.8").unwrap());
        assert_eq!(resp.secondary_dns, IpAddr::from_str("8.8.4.4").unwrap());
    }

    #[test]
    fn test_query_cdnscfg_invalid_ip() {
        let cmd = QueryCDNSCFG;

        let data = b"PrimaryDns: \"invalid\" SecondaryDns: \"8.8.4.4\"\r\nOK";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }

    #[test]
    fn test_query_cdnscfg_missing_secondary() {
        let cmd = QueryCDNSCFG;

        let data = b"PrimaryDns: \"8.8.8.8\"\r\nOK";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_real_response() {
        let response = b"\r\nPrimaryDns: 208.67.222.222\r\nSecondaryDns: 0.0.0.0\r\n\r\nOK\r";

        let cmd = QueryCDNSCFG;

        let resp = cmd.parse_response_struct(response).unwrap();

        assert_eq!(
            resp.primary_dns,
            IpAddr::from_str("208.67.222.222").unwrap()
        );
        assert_eq!(resp.secondary_dns, IpAddr::from_str("0.0.0.0").unwrap());
    }

    // -------------------------
    // ConfigureDns tests
    // -------------------------

    #[test]
    fn test_configure_dns_command_two_servers() {
        let cmd = ConfigureDns {
            primary_dns: "8.8.8.8",
            secondary_dns: Some("8.8.4.4"),
        };

        let mut buffer = [0u8; 64];

        let result = cmd.get_command(&mut buffer).unwrap();
        let command = core::str::from_utf8(result).unwrap();

        assert_eq!(command, "AT+CDNSCFG=\"8.8.8.8\",\"8.8.4.4\"\r\n");
    }

    #[test]
    fn test_configure_dns_command_single_server() {
        let cmd = ConfigureDns {
            primary_dns: "1.1.1.1",
            secondary_dns: None,
        };

        let mut buffer = [0u8; 64];

        let result = cmd.get_command(&mut buffer).unwrap();
        let command = core::str::from_utf8(result).unwrap();

        assert_eq!(command, "AT+CDNSCFG=\"1.1.1.1\"\r\n");
    }

    #[test]
    fn test_configure_dns_small_buffer() {
        let cmd = ConfigureDns {
            primary_dns: "8.8.8.8",
            secondary_dns: Some("8.8.4.4"),
        };

        let mut buffer = [0u8; 8];

        let result = cmd.get_command(&mut buffer);

        assert!(result.is_err());
    }

    #[test]
    fn test_configure_dns_parse_ok() {
        let cmd = ConfigureDns {
            primary_dns: "8.8.8.8",
            secondary_dns: None,
        };

        let data = b"\r\nOK\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_ok());
    }

    #[test]
    fn test_configure_dns_parse_error() {
        let cmd = ConfigureDns {
            primary_dns: "8.8.8.8",
            secondary_dns: None,
        };

        let data = b"\r\nERROR\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err());
    }
}
