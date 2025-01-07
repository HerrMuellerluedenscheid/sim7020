use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::builder::CommandBuilder;
use at_commands::parser::CommandParser;

#[cfg(feature = "defmt")]
use defmt::info;

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

/// create a HTTP or HTTPS session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GetHttpSessions {}

impl AtRequest for GetHttpSessions {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let cmd = CommandBuilder::create_query(buffer, true)
            .named(b"+CHTTPCREATE")
            .finish();
        cmd
    }

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
}

/// create a HTTP or HTTPS session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CreateHttpSession<'a> {
    pub host: &'a str,
    pub user: Option<&'a str>,
    pub password: Option<&'a str>,
}

impl AtRequest for CreateHttpSession<'_> {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPCREATE")
            .with_string_parameter(self.host)
            // todo: optional parameters need to be fixed
            // .with_optional_string_parameter(self.user)
            // .with_optional_string_parameter(self.password)
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (client_id,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CHTTPCREATE: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;
        Ok(AtResponse::HTTPSessionCreated(client_id as u8))
    }
}

/// Connect to a server using http or https
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HttpConnect {
    pub client_id: u8,
}

impl AtRequest for HttpConnect {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPCON")
            .with_int_parameter(self.client_id)
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        info!("parsing {=[u8]:a}", data);
        Ok(AtResponse::Ok)
    }
}

/// Disconnect from a server
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HttpDisconnect {
    pub client_id: u8,
}

impl AtRequest for HttpDisconnect {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPDISCON")
            .with_int_parameter(self.client_id)
            .finish()
    }
}

/// Connect to a server using http or https
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HttpDestroy {
    pub client_id: u8,
}

impl AtRequest for HttpDestroy {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CHTTPDESTROY")
            .with_int_parameter(self.client_id)
            .finish()
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
    type Response = Result<(), AtError>;

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
}
