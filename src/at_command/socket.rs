use crate::{
    at_command::{AtRequest, AtResponse},
    AtError,
};

/// Domain for the socket connection
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Domain {
    IPv4 = 1,
    IPv6 = 2,
}

/// Indicates the type of connection for the socket
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Type {
    TCP = 1,
    UPD = 2,
    RAW = 3,
}

/// Indicates the underlaying protocol using for the socket
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Protocol {
    IP = 1,
    ICMP = 2,
    UDPLITE = 3,
}

/// AT command to create a socket
pub struct CreateSocket {
    /// Type of IP connection that will be used
    pub domain: Domain,
    /// Communication type that will be used
    pub connection_type: Type,
    /// Underlaying communication protocol that will be used
    pub protocol: Protocol,
    /// PDP context, check [PDPContext](crate::at_command::pdp_context::PDPContext)
    pub cid: Option<i32>,
}

impl AtRequest for CreateSocket {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let mut builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOC")
            .with_int_parameter(self.domain as u8)
            .with_int_parameter(self.connection_type as u8)
            .with_int_parameter(self.protocol as u8);

        if let Some(cid) = self.cid {
            builder = builder.with_int_parameter(cid);
        }

        builder.finish()
    }

    fn parse_response(&self, _data: &[u8]) -> Result<super::AtResponse, AtError> {
        let socket_id = at_commands::parser::CommandParser::parse(_data)
            .expect_identifier(b"+CSOC: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()?;

        Ok(AtResponse::SocketCreated(socket_id.0 as u8))
    }
}

/// Command to connect the socket to a remote address
pub struct ConnectSocketToRemote<'a> {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
    /// Port to be used in the communication
    pub port: u16,
    /// Address of the server which we want to connect to
    pub remote_address: &'a str,
    /// Communication type that will be used
    pub connection_type: Type,
}

impl AtRequest for ConnectSocketToRemote<'_> {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        assert!(self.port > 0);
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOCON")
            .with_int_parameter(self.socket_id)
            .with_int_parameter(self.port as i32)
            .with_string_parameter(self.remote_address)
            .with_int_parameter(self.connection_type as u8);

        builder.finish()
    }

    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        at_commands::parser::CommandParser::parse(_data)
            .expect_identifier(b"OK\r\n")
            .finish()?;

        Ok(AtResponse::Ok)
    }
}

/// Struct used to send data through the socket
pub struct SendSocketMessage<'a> {
    /// Socket ID obtained by using [CreateSocket]
    socket_id: u8,
    /// Length of the data we want to send
    data_len: u16,
    /// Data to be send
    data: &'a [u8],
}

impl AtRequest for SendSocketMessage<'_> {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOSEND")
            .with_int_parameter(self.socket_id)
            .with_int_parameter(self.data_len)
            .with_raw_parameter(self.data);

        builder.finish()
    }

    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        at_commands::parser::CommandParser::parse(_data)
            .expect_identifier(b"OK\r\n")
            .finish()?;

        Ok(AtResponse::Ok)
    }
}

/// Closes the opened TCP socket
pub struct CloseSocket {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
}

impl AtRequest for CloseSocket {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOCL")
            .with_int_parameter(self.socket_id);

        builder.finish()
    }
}

#[cfg(test)]
mod test {
    use crate::at_command::{
        socket::{CloseSocket, ConnectSocketToRemote, CreateSocket, Domain, Protocol, Type},
        AtRequest, AtResponse,
    };

    #[test]
    fn test_create_socket_command() {
        let mut buffer = [0; 512];

        let create_socket = CreateSocket {
            domain: Domain::IPv4,
            connection_type: Type::TCP,
            protocol: Protocol::IP,
            cid: Some(3),
        };

        let result = create_socket.get_command(&mut buffer).unwrap();

        assert_eq!(core::str::from_utf8(result).unwrap(), "AT+CSOC=1,1,1,3\r\n");
    }

    #[test]
    fn test_create_socket_command_without_cid() {
        let mut buffer = [0; 512];

        let create_socket = CreateSocket {
            domain: Domain::IPv6,
            connection_type: Type::RAW,
            protocol: Protocol::ICMP,
            cid: None,
        };

        let result = create_socket.get_command(&mut buffer).unwrap();

        assert_eq!(core::str::from_utf8(result).unwrap(), "AT+CSOC=2,3,2\r\n");
    }

    #[test]
    fn test_parse_create_socket_response() {
        let create_socket = CreateSocket {
            domain: Domain::IPv4,
            connection_type: Type::TCP,
            protocol: Protocol::IP,
            cid: None,
        };

        // Response example: +CSOC 5\r\n\r\nOK\r\n
        let response = b"+CSOC: 5\r\n\r\nOK\r\n";

        let parsed = create_socket.parse_response(response).unwrap();

        match parsed {
            AtResponse::SocketCreated(id) => assert_eq!(id, 5),
            _ => panic!("Expected AtResponse::SocketCreated"),
        }
    }

    #[test]
    fn test_connect_remote_socket_command() {
        let mut buffer = [0; 512];

        let at_connect_request = ConnectSocketToRemote {
            connection_type: super::Type::TCP,
            port: 1111,
            socket_id: 1,
            remote_address: "127.0.0.1",
        };

        let result = at_connect_request.get_command(&mut buffer).unwrap();

        assert_eq!(
            core::str::from_utf8(result).unwrap(),
            "AT+CSOCON=1,1111,\"127.0.0.1\",1\r\n"
        );
    }

    #[test]
    #[should_panic]
    fn test_connect_remote_socket_command_with_invalid_port() {
        let mut buffer = [0; 512];

        let at_connect_request = ConnectSocketToRemote {
            connection_type: super::Type::TCP,
            port: 0,
            socket_id: 1,
            remote_address: "127.0.0.1",
        };

        at_connect_request.get_command(&mut buffer).unwrap();
    }

    #[test]
    fn test_close_socket() {
        let mut buffer = [0; 512];

        let at_connect_request = CloseSocket { socket_id: 0 };

        let result = at_connect_request.get_command(&mut buffer).unwrap();

        assert_eq!(core::str::from_utf8(result).unwrap(), "AT+CSOCL=0\r\n");
    }
}
