//! Module for the sockets
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::{
    at_command::{verify_ok, AtRequest},
    AtError,
};

/// Domain for the socket connection
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum Domain {
    IPv4 = 1,
    IPv6 = 2,
}

/// Indicates the type of connection for the socket
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    TCP = 1,
    UPD = 2,
    RAW = 3,
}

/// Indicates the underlaying protocol using for the socket
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Copy)]
pub enum Protocol {
    IP = 1,
    ICMP = 2,
    UDPLITE = 3,
}

/// AT command to create a socket
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Debug)]
pub struct SocketCreated {
    pub socket_id: u8,
}

impl CreateSocket {
    fn get_socket_id(data: &[u8]) -> Result<u8, AtError> {
        let (socket_id,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CSOC: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        Ok(socket_id as u8)
    }
}

impl AtRequest for CreateSocket {
    type Response = SocketCreated;

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

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<super::AtResponse, AtError> {
        let socket_id = Self::get_socket_id(data)?;

        Ok(AtResponse::SocketCreated(socket_id))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let socket_id = Self::get_socket_id(data)?;

        Ok(SocketCreated { socket_id })
    }
}

/// Command to connect the socket to a remote address
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct ConnectSocketToRemote<'a> {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
    /// Port to be used in the communication
    pub port: u16,
    /// Address of the server which we want to connect to
    pub remote_address: &'a str,
}

impl AtRequest for ConnectSocketToRemote<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        assert!(self.port > 0);
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOCON")
            .with_int_parameter(self.socket_id)
            .with_int_parameter(self.port as i32)
            .with_string_parameter(self.remote_address);

        builder.finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        verify_ok(data)?;
        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)?;
        Ok(())
    }
}

/// Struct used to send data through the socket
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq)]
pub struct SendSocketMessage<'a> {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
    /// Data to be sent.
    pub data: &'a [u8],
}

impl AtRequest for SendSocketMessage<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOSEND")
            .with_int_parameter(self.socket_id)
            // The data size must be multiplied by 2, we need to indicate the hex length
            // for each byte that we will write there will be 2 hex bytes
            .with_int_parameter((self.data.len() as u16) * 2)
            .with_rax_hex_parameter(self.data);

        builder.finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        verify_ok(data)?;

        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)?;

        Ok(())
    }
}

/// Struct used to send data through the socket
pub struct SendSocketString<'a> {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
    /// Data to be send. Must be in hex format
    pub data: &'a str,
}

impl AtRequest for SendSocketString<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOSEND")
            .with_int_parameter(self.socket_id)
            .with_int_parameter(0)
            .with_string_parameter(self.data);

        builder.finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        verify_ok(data)?;

        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)?;

        Ok(())
    }
}

/// Closes the opened TCP socket
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CloseSocket {
    /// Socket ID obtained by using [CreateSocket]
    pub socket_id: u8,
}

impl AtRequest for CloseSocket {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut super::BufferType) -> Result<&'a [u8], usize> {
        let builder = at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CSOCL")
            .with_int_parameter(self.socket_id);

        builder.finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #![allow(deprecated)]
    use super::*;

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
        let response = b"\r\n+CSOC: 5\r\n\r\nOK\r";

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
            port: 1111,
            socket_id: 1,
            remote_address: "127.0.0.1",
        };

        let result = at_connect_request.get_command(&mut buffer).unwrap();

        assert_eq!(
            core::str::from_utf8(result).unwrap(),
            "AT+CSOCON=1,1111,\"127.0.0.1\"\r\n"
        );
    }

    #[test]
    #[should_panic]
    fn test_connect_remote_socket_command_with_invalid_port() {
        let mut buffer = [0; 512];

        let at_connect_request = ConnectSocketToRemote {
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

    #[test]
    fn domain_enum_values() {
        assert_eq!(Domain::IPv4 as u8, 1);
        assert_eq!(Domain::IPv6 as u8, 2);
    }

    #[test]
    fn type_enum_values() {
        assert_eq!(Type::TCP as u8, 1);
        assert_eq!(Type::UPD as u8, 2);
        assert_eq!(Type::RAW as u8, 3);
    }

    #[test]
    fn protocol_enum_values() {
        assert_eq!(Protocol::IP as u8, 1);
        assert_eq!(Protocol::ICMP as u8, 2);
        assert_eq!(Protocol::UDPLITE as u8, 3);
    }

    #[test]
    fn create_socket_command_without_cid() {
        let cmd = CreateSocket {
            domain: Domain::IPv4,
            connection_type: Type::TCP,
            protocol: Protocol::IP,
            cid: None,
        };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSOC=1,1,1\r\n");
    }

    #[test]
    fn create_socket_command_with_cid() {
        let cmd = CreateSocket {
            domain: Domain::IPv6,
            connection_type: Type::RAW,
            protocol: Protocol::ICMP,
            cid: Some(3),
        };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSOC=2,3,2,3\r\n");
    }

    #[test]
    fn create_socket_parse_response_struct() {
        let data = b"\r\n+CSOC: 5\r\n\r\nOK\r";

        let cmd = CreateSocket {
            domain: Domain::IPv6,
            connection_type: Type::RAW,
            protocol: Protocol::ICMP,
            cid: Some(3),
        };

        let resp = cmd.parse_response_struct(data).unwrap();

        assert_eq!(resp, SocketCreated { socket_id: 5 });
    }

    #[test]
    fn connect_socket_command() {
        let cmd = ConnectSocketToRemote {
            socket_id: 2,
            port: 1883,
            remote_address: "127.0.0.1",
        };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSOCON=2,1883,\"127.0.0.1\"\r\n");
    }

    #[test]
    fn connect_socket_parse_ok() {
        let data = b"\r\nOK\r\n";

        assert!(ConnectSocketToRemote {
            socket_id: 1,
            port: 80,
            remote_address: "127.0.0.1"
        }
        .parse_response_struct(data)
        .is_ok());
    }

    #[test]
    fn send_socket_message_command() {
        let payload = [0xDE, 0xAD, 0xBE, 0xEF];

        let cmd = SendSocketMessage {
            socket_id: 1,
            data: &payload,
        };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        // length = 4 bytes * 2 hex chars = 8
        assert_eq!(bytes, b"AT+CSOSEND=1,8,deadbeef\r\n");
    }

    #[test]
    fn send_socket_message_parse_ok() {
        let data = b"\r\nOK\r\n";

        assert!(SendSocketMessage {
            socket_id: 0,
            data: &[0x00, 0x01],
        }
        .parse_response_struct(data)
        .is_ok());
    }

    #[test]
    fn send_socket_string_command() {
        let cmd = SendSocketString {
            socket_id: 3,
            data: "48656C6C6F",
        };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSOSEND=3,0,\"48656C6C6F\"\r\n");
    }

    #[test]
    fn close_socket_command() {
        let cmd = CloseSocket { socket_id: 9 };

        let mut buffer = [0u8; 512];
        let bytes = cmd.get_command(&mut buffer).unwrap();

        assert_eq!(bytes, b"AT+CSOCL=9\r\n");
    }

    #[test]
    fn close_socket_parse_response_struct() {
        let data = b"\r\nOK\r\n";

        assert!(CloseSocket { socket_id: 1 }
            .parse_response_struct(data)
            .is_ok());
    }
}
