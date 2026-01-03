//! Module to interact with the PSD configuration
use crate::at_command::AtRequest;

/// The types of PDP that can be configured
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub enum PdpType {
    IP,
    IPV6,
    IPV4V6,
    NonIp,
}

impl PdpType {
    fn as_str(&self) -> &'static str {
        match self {
            PdpType::IP => "IP",
            PdpType::IPV6 => "IPV6",
            PdpType::IPV4V6 => "IPV4V6",
            PdpType::NonIp => "Non-IP",
        }
    }
}

/// Struct that can be used to set the PSD settings
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SetPSDSettings<'a> {
    /// Type of PDP to be used
    pub pdp_type: PdpType,
    /// The APN if there is any
    pub apn: Option<&'a str>,
    /// The username if there is any
    pub username: Option<&'a str>,
    /// The password if there is any
    pub password: Option<&'a str>,
}

impl AtRequest for SetPSDSettings<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("*MCGDEFCONT")
            .with_string_parameter(self.pdp_type.as_str())
            .with_optional_string_parameter(self.apn)
            .with_optional_string_parameter(self.username)
            .with_optional_string_parameter(self.password)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, crate::AtError> {
        super::verify_ok(data)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pdp_as_str() {
        assert_eq!(PdpType::IP.as_str(), "IP");
        assert_eq!(PdpType::IPV6.as_str(), "IPV6");
        assert_eq!(PdpType::NonIp.as_str(), "Non-IP");
        assert_eq!(PdpType::IP.as_str(), "IP");
    }

    #[test]
    fn test_set_psd_settings_request() {
        let mut buffer = [0u8; 512];

        let command = SetPSDSettings {
            pdp_type: PdpType::IP,
            apn: None,
            username: None,
            password: None,
        };

        let data = command.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT*MCGDEFCONT=\"IP\",,,\r\n");

        let command = SetPSDSettings {
            pdp_type: PdpType::IP,
            apn: Some("APN"),
            username: None,
            password: None,
        };

        let data = command.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT*MCGDEFCONT=\"IP\",\"APN\",,\r\n");

        let command = SetPSDSettings {
            pdp_type: PdpType::IP,
            apn: Some("APN"),
            username: Some("USERNAME"),
            password: None,
        };

        let data = command.get_command(&mut buffer).unwrap();

        assert_eq!(data, b"AT*MCGDEFCONT=\"IP\",\"APN\",\"USERNAME\",\r\n");

        let command = SetPSDSettings {
            pdp_type: PdpType::IP,
            apn: Some("APN"),
            username: Some("USERNAME"),
            password: Some("PASSWORD"),
        };

        let data = command.get_command(&mut buffer).unwrap();

        assert_eq!(
            data,
            b"AT*MCGDEFCONT=\"IP\",\"APN\",\"USERNAME\",\"PASSWORD\"\r\n"
        );
    }

    #[test]
    fn test_set_psd_settings_response() {
        let data = b"\r\nOK\r\n";

        let command = SetPSDSettings {
            pdp_type: PdpType::IP,
            apn: None,
            username: None,
            password: None,
        };

        command.parse_response_struct(data).unwrap();
    }
}
