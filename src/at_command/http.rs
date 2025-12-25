#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::*;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct HttpClient {
    pub client_id: u8,
}

pub struct HttpSession<'a> {
    pub client_id: u8,
    pub successful: bool,
    pub host: &'a str,
}

const HTTP_SESSION_SUCCESSFULLY: i32 = 1;
const HTTP_SESSION_FAILED: i32 = 0;
const DEFAULT_N_SESSIONS: usize = 4;

/// create a HTTP or HTTPS session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetHttpSessions<
    const N_SESSIONS: usize = DEFAULT_N_SESSIONS,
    const HOST_MAX_SIZE: usize = DEFAULT_HOST_MAX_SIZE,
> {}

pub enum HttpSessionState {
    Sucessfully,
    Failed,
}

impl From<i32> for HttpSessionState {
    fn from(value: i32) -> Self {
        match value {
            HTTP_SESSION_SUCCESSFULLY => HttpSessionState::Sucessfully,
            HTTP_SESSION_FAILED => HttpSessionState::Failed,
            _ => unreachable!(),
        }
    }
}

const DEFAULT_HOST_MAX_SIZE: usize = 255;

pub struct HttpSessionInformation<const HOST_MAX_SIZE: usize = DEFAULT_HOST_MAX_SIZE> {
    pub http_client_id: i32,
    pub state: HttpSessionState,
    pub host: heapless::String<HOST_MAX_SIZE>,
}

// TODO: We will add a simple implementation for 4 items. We should find a way to implement a way to get N items
impl<const HOST_MAX_SIZE: usize> AtRequest for GetHttpSessions<DEFAULT_N_SESSIONS, HOST_MAX_SIZE> {
    type Response = [HttpSessionInformation<HOST_MAX_SIZE>; DEFAULT_N_SESSIONS];

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
        debug!("Parsing {} http responses");
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
pub struct CreateHttpSession<'a> {
    pub host: &'a str,
    pub user: Option<&'a str>,
    pub password: Option<&'a str>,
}

pub struct CreateHttpSessionResponse {
    pub client_id: u8,
}

impl CreateHttpSession<'_> {
    fn get_client_id(data: &[u8]) -> Result<u8, AtError> {
        let (client_id,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;

        Ok(client_id as u8)
    }
}

impl AtRequest for CreateHttpSession<'_> {
    type Response = CreateHttpSessionResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
pub struct HttpConnect {
    pub client_id: u8,
}

impl AtRequest for HttpConnect {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
pub struct HttpDisconnect {
    pub client_id: u8,
}

impl AtRequest for HttpDisconnect {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
pub struct HttpDestroy {
    pub client_id: u8,
}

impl AtRequest for HttpDestroy {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
pub struct HttpSend<'a> {
    pub client_id: u8,
    pub method: HttpMethod,
    pub path: &'a str,
}

impl AtRequest for HttpSend<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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
