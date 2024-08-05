use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use defmt::Format;

/// create a HTTP or HTTPS session
#[derive(Format)]
pub struct HttpSession<'a> {
    pub host: &'a str,
    pub user: Option<&'a str>,
    pub password: Option<&'a str>,
}

impl AtRequest for HttpSession<'_> {
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
}

/// Connect to a server using http or https
#[derive(Format)]
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
}

/// Disconnect from a server
#[derive(Format)]
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
#[derive(Format)]
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

#[derive(Format)]
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
#[derive(Format)]
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
