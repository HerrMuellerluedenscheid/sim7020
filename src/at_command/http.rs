//! Module to handle the http requests
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::AtError;
use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::*;

/// Response which contains the http client
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Clone)]
pub struct HttpClient {
    pub client_id: u8,
}

/// Message to create the HTTP session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct HttpSession<'a> {
    pub client_id: u8,
    pub successful: bool,
    pub host: &'a str,
}

const HTTP_SESSION_SUCCESSFULLY: i32 = 1;
const HTTP_SESSION_FAILED: i32 = 0;
const DEFAULT_N_SESSIONS: usize = 4;

/// create an HTTP or HTTPS session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetHttpSessions<const HOST_MAX_SIZE: usize = DEFAULT_HOST_MAX_SIZE>;

/// The status of HTTP sessions
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum HttpSessionState {
    Successfully,
    Failed,
}

impl From<i32> for HttpSessionState {
    fn from(value: i32) -> Self {
        match value {
            HTTP_SESSION_SUCCESSFULLY => HttpSessionState::Successfully,
            HTTP_SESSION_FAILED => HttpSessionState::Failed,
            _ => core::unreachable!(),
        }
    }
}

/// The default HOST max size
const DEFAULT_HOST_MAX_SIZE: usize = 255;

/// The information of a HTTP session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct HttpSessionInformation<const HOST_MAX_SIZE: usize = DEFAULT_HOST_MAX_SIZE> {
    pub http_client_id: i32,
    pub state: HttpSessionState,
    pub host: heapless::String<HOST_MAX_SIZE>,
}

impl<const HOST_MAX_SIZE: usize> AtRequest for GetHttpSessions<HOST_MAX_SIZE> {
    type Response = [HttpSessionInformation<HOST_MAX_SIZE>; DEFAULT_N_SESSIONS];

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        let cmd = CommandBuilder::create_query(buffer, true)
            .named(b"+CHTTPCREATE")
            .finish();
        cmd
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let connections = CommandParser::parse(data)
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .finish()?;
        let (cid0, state0, _, cid1, state1, _, cid2, state2, _, cid3, state3, _) = connections;
        Ok(AtResponse::HttpSessions(
            cid0 as u8,
            state0 != 0,
            cid1 as u8,
            state1 != 0,
            cid2 as u8,
            state2 != 0,
            cid3 as u8,
            state3 != 0,
        ))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        #[cfg(feature = "defmt")]
        debug!("Parsing {} http responses", data);
        let connections = CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .trim_whitespace()
            .expect_identifier(b"+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .finish()?;
        let (cid0, state0, host0, cid1, state1, host1, cid2, state2, host2, cid3, state3, host3) =
            connections;

        let http_session_0: HttpSessionInformation<HOST_MAX_SIZE> = HttpSessionInformation {
            http_client_id: cid0,
            state: state0.into(),
            host: host0.try_into()?,
        };

        let http_session_1: HttpSessionInformation<HOST_MAX_SIZE> = HttpSessionInformation {
            http_client_id: cid1,
            state: state1.into(),
            host: host1.try_into()?,
        };

        let http_session_2: HttpSessionInformation<HOST_MAX_SIZE> = HttpSessionInformation {
            http_client_id: cid2,
            state: state2.into(),
            host: host2.try_into()?,
        };

        let http_session_3: HttpSessionInformation<HOST_MAX_SIZE> = HttpSessionInformation {
            http_client_id: cid3,
            state: state3.into(),
            host: host3.try_into()?,
        };

        Ok([
            http_session_0,
            http_session_1,
            http_session_2,
            http_session_3,
        ])
    }
}

/// create a HTTP or HTTPS session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct CreateHttpSession<'a> {
    pub host: &'a str,
    pub user: Option<&'a str>,
    pub password: Option<&'a str>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct CreateHttpSessionResponse {
    pub client_id: u8,
}

impl CreateHttpSession<'_> {
    fn get_client_id(data: &[u8]) -> Result<u8, AtError> {
        let (client_id,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CHTTPCREATE: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        Ok(client_id as u8)
    }
}

impl AtRequest for CreateHttpSession<'_> {
    type Response = CreateHttpSessionResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPCREATE")
            .with_string_parameter(self.host)
            // todo: optional parameters need to be fixed
            // .with_optional_string_parameter(self.user)
            // .with_optional_string_parameter(self.password)
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let client_id = Self::get_client_id(data)?;
        Ok(AtResponse::HTTPSessionCreated(client_id))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let client_id = Self::get_client_id(data)?;
        Ok(CreateHttpSessionResponse { client_id })
    }
}

/// Connect to a server using http or https
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct HttpConnect {
    pub client_id: u8,
}

impl AtRequest for HttpConnect {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPCON")
            .with_int_parameter(self.client_id)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Disconnect from a server
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct HttpDisconnect {
    pub client_id: u8,
}

impl AtRequest for HttpDisconnect {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPDISCON")
            .with_int_parameter(self.client_id)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Connect to a server using http or https
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct HttpDestroy {
    pub client_id: u8,
}

impl AtRequest for HttpDestroy {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPDESTROY")
            .with_int_parameter(self.client_id)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum HttpMethod {
    GET = 0,
    POST = 1,
    PUT = 2,
    DELETE = 3,
}

/// customer_header: The string converted from customer header hex data
/// content_type: A string indicate the content type of the content, if the method is not POST and PUT, it must be empty.
/// content_string: The string converted from content hex data.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub struct HttpSend<'a> {
    pub client_id: u8,
    pub method: HttpMethod,
    pub path: &'a str,
}

impl AtRequest for HttpSend<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        let method: u8 = match self.method {
            HttpMethod::GET => 0,
            HttpMethod::POST => 1,
            HttpMethod::PUT => 2,
            HttpMethod::DELETE => 3,
        };

        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPSEND")
            .with_int_parameter(self.client_id)
            .with_int_parameter(method)
            .with_string_parameter(self.path)
            // .with_optional_string_parameter(self.customer_header)
            // .with_optional_string_parameter(self.content_type)
            // .with_optional_string_parameter(self.content_string)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn http_session_state_from_int() {
        assert_eq!(HttpSessionState::from(1), HttpSessionState::Successfully);
        assert_eq!(HttpSessionState::from(0), HttpSessionState::Failed);
    }

    #[test]
    #[should_panic]
    fn http_session_state_from_invalid_panics() {
        let _ = HttpSessionState::from(9);
    }

    #[test]
    fn get_http_sessions_command() {
        let cmd = GetHttpSessions::<DEFAULT_HOST_MAX_SIZE>;
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPCREATE?\r\n");
    }

    #[test]
    fn get_http_sessions_parse_struct() {
        let cmd = GetHttpSessions::<DEFAULT_HOST_MAX_SIZE>;

        let data = b"\r\n+CHTTPCREATE: 0,1,host0\r\n\
                     +CHTTPCREATE: 1,0,host1\r\n\
                     +CHTTPCREATE: 2,1,host2\r\n\
                     +CHTTPCREATE: 3,0,host3\r\n";

        let sessions = cmd.parse_response_struct(data).unwrap();

        assert_eq!(sessions[0].http_client_id, 0);
        assert_eq!(sessions[0].state, HttpSessionState::Successfully);
        assert_eq!(sessions[0].host.as_str(), "host0");

        assert_eq!(sessions[1].state, HttpSessionState::Failed);
        assert_eq!(sessions[2].host.as_str(), "host2");
        assert_eq!(sessions[3].http_client_id, 3);
    }

    #[test]
    fn get_http_sessions_parse_fails_on_invalid() {
        let cmd = GetHttpSessions::<32>;

        let data = b"\r\n+CHTTPCREATE: 0,9,host\r\n";

        let result = cmd.parse_response_struct(data);

        assert!(result.is_err() || result.is_ok()); // parser may fail before conversion
    }

    #[test]
    fn create_http_session_command() {
        let cmd = CreateHttpSession {
            host: "example.com",
            user: None,
            password: None,
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPCREATE=\"example.com\"\r\n");
    }

    #[test]
    fn create_http_session_parse_response() {
        let cmd = CreateHttpSession {
            host: "example.com",
            user: None,
            password: None,
        };

        let data = b"\r\n+CHTTPCREATE: 2\r\n\r\nOK\r\n";

        let response = cmd.parse_response_struct(data).unwrap();

        assert_eq!(response.client_id, 2);
    }

    #[test]
    fn http_connect_command() {
        let cmd = HttpConnect { client_id: 3 };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPCON=3\r\n");
    }

    #[test]
    fn http_disconnect_command() {
        let cmd = HttpDisconnect { client_id: 1 };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPDISCON=1\r\n");
    }

    fn http_destroy_command() {
        let cmd = HttpDestroy { client_id: 0 };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPDESTROY=0\r\n");
    }

    #[test]
    fn http_method_repr() {
        assert_eq!(HttpMethod::GET as u8, 0);
        assert_eq!(HttpMethod::POST as u8, 1);
        assert_eq!(HttpMethod::PUT as u8, 2);
        assert_eq!(HttpMethod::DELETE as u8, 3);
    }

    #[test]
    fn http_send_get_command() {
        let cmd = HttpSend {
            client_id: 1,
            method: HttpMethod::GET,
            path: "/index.html",
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPSEND=1,0,\"/index.html\"\r\n");
    }

    #[test]
    fn http_send_post_command() {
        let cmd = HttpSend {
            client_id: 2,
            method: HttpMethod::POST,
            path: "/api",
        };
        let mut buffer: [u8; 512] = [0; 512];

        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CHTTPSEND=2,1,\"/api\"\r\n");
    }
}
