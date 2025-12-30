use crate::at_command::AtRequest;

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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct SetPSDSettings<'a> {
    pdp_type: PdpType,
    apn: Option<&'a str>,
    username: Option<&'a str>,
    password: Option<&'a str>,
}

impl AtRequest for SetPSDSettings<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
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
