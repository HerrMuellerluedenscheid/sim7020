//! Module containing the structs that are necessary to interact with the APN configuration

use crate::at_command::{verify_ok, AtRequest, BufferType};
use crate::AtError;

const CSTT_SIZE_MAX: usize = 32; // AT Datasheet page 172

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Command to get the current APN configuration
pub struct GetAPNUserPassword;

/// Contains the current configuration of the APN
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct APNUserPassword {
    /// Configured APN if there is any
    pub apn: Option<heapless::String<CSTT_SIZE_MAX>>,
    /// Configured username if there is any
    pub username: Option<heapless::String<CSTT_SIZE_MAX>>,
    /// Configured password if there is any
    pub password: Option<heapless::String<CSTT_SIZE_MAX>>,
}

impl AtRequest for GetAPNUserPassword {
    type Response = APNUserPassword;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("+CSTT")
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (apn, user, password) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CSTT: ")
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .expect_optional_string_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        let apn = apn.map(|x| x.try_into()).transpose()?;
        let username = user.map(|x| x.try_into()).transpose()?;
        let password = password.map(|x| x.try_into()).transpose()?;

        let output = APNUserPassword {
            apn,
            username,
            password,
        };

        Ok(output)
    }
}

/// Command to set the APN configuration
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default, PartialEq, Clone)]
pub struct SetAPNUserPassword<'a> {
    /// The APN to be set if any
    pub(crate) apn: Option<&'a str>,
    /// The user to be set if any
    pub(crate) user: Option<&'a str>,
    /// The password to be set if any
    pub(crate) password: Option<&'a str>,
}

impl<'a> SetAPNUserPassword<'a> {
    /// Creates a new [SetAPNUserPassword] with the provided data
    pub fn new(apn: Option<&'a str>, user: Option<&'a str>, password: Option<&'a str>) -> Self {
        Self {
            apn,
            user,
            password,
        }
    }
    /// Sets the APN
    pub fn with_apn(&mut self, apn: &'a str) {
        self.apn = Some(apn);
    }

    /// Set the user
    pub fn with_user(&mut self, user: &'a str) {
        self.user = Some(user);
    }

    /// Sets the password
    pub fn with_password(&mut self, pass: &'a str) {
        self.password = Some(pass);
    }
}

impl AtRequest for SetAPNUserPassword<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSTT")
            .with_optional_string_parameter(self.apn)
            .with_optional_string_parameter(self.user)
            .with_optional_string_parameter(self.password)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_apn_password_request() {
        let mut buffer = [0u8; 512];

        let request = GetAPNUserPassword.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CSTT?\r\n");
    }

    #[test]
    fn test_get_apn_password_response() {
        let buffer = b"\r\n+CSTT: \r\n\r\nOK\r\n";

        let result = GetAPNUserPassword.parse_response_struct(buffer).unwrap();

        assert_eq!(result.apn, None);
        assert_eq!(result.username, None);
        assert_eq!(result.password, None);
    }

    #[test]
    fn test_get_apn_password_response_with_data() {
        let buffer = b"\r\n+CSTT: \"APN\"\r\n\r\nOK\r\n";

        let result = GetAPNUserPassword.parse_response_struct(buffer).unwrap();

        assert_eq!(result.apn.unwrap(), "APN");
        assert_eq!(result.username, None);
        assert_eq!(result.password, None);

        let buffer = b"\r\n+CSTT: \"APN\",\"user\"\r\n\r\nOK\r\n";

        let result = GetAPNUserPassword.parse_response_struct(buffer).unwrap();

        assert_eq!(result.apn.unwrap(), "APN");
        assert_eq!(result.username.unwrap(), "user");
        assert_eq!(result.password, None);

        let buffer = b"\r\n+CSTT: \"APN\",\"user\",\"pass\"\r\n\r\nOK\r\n";

        let result = GetAPNUserPassword.parse_response_struct(buffer).unwrap();

        assert_eq!(result.apn.unwrap(), "APN");
        assert_eq!(result.username.unwrap(), "user");
        assert_eq!(result.password.unwrap(), "pass");
    }

    #[test]
    fn test_set_apn_password_empty_request() {
        let mut buffer = [0u8; 512];

        let set_apn_password = SetAPNUserPassword::default();

        let request = set_apn_password.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CSTT=,,\r\n");
    }

    #[test]
    fn test_set_apn_password_request() {
        let mut buffer = [0u8; 512];

        let mut set_apn_password = SetAPNUserPassword::default();
        set_apn_password.with_apn("APN");

        let request = set_apn_password.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CSTT=\"APN\",,\r\n");

        set_apn_password.with_user("user");

        let request = set_apn_password.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CSTT=\"APN\",\"user\",\r\n");

        set_apn_password.with_password("pass");

        let request = set_apn_password.get_command(&mut buffer).unwrap();

        assert_eq!(request, b"AT+CSTT=\"APN\",\"user\",\"pass\"\r\n");
    }

    #[test]
    fn test_set_apn_password_response() {
        let response = b"\r\nOK\r";

        let set_apn_password = SetAPNUserPassword::default();

        set_apn_password.parse_response_struct(response).unwrap();
    }
}
