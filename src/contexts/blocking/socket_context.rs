//! Contains the definitions for the socket contexts

use core::marker::PhantomData;

use crate::at_command::socket::*;
use crate::contexts::common_socket_context::{Connected, PendingConnection};
use crate::{AtError, Modem};
#[cfg(feature = "defmt")]
use defmt::{debug, error, warn};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, ReadReady, Write};
#[cfg(test)]
use mockall::automock;

/// Helper trait for sending and receiving data from
#[cfg_attr(test, automock)]
pub trait SocketContextReceiver {
    /// Sends the given data and returns a [Result]
    fn send_data<'a>(&mut self, send_socket_message: SendSocketMessage<'a>) -> Result<(), AtError>;

    /// Sends the given data and returns a [Result]
    fn send_string<'a>(&mut self, send_socket_string: SendSocketString<'a>) -> Result<(), AtError>;

    /// Connects the socket and returns a [Result]
    fn connect<'a>(
        &mut self,
        connect_socket_to_remote: ConnectSocketToRemote<'a>,
    ) -> Result<(), AtError>;

    /// Creates a new socket
    fn create(&mut self, create_socket: CreateSocket) -> Result<u8, AtError>;

    /// Closes the socket
    fn close(&mut self, close_socket: CloseSocket) -> Result<(), AtError>;

    // TODO: Add a receive data method
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> SocketContextReceiver
    for Modem<'a, W, R, P, D>
{
    #[inline]
    fn send_data(&mut self, send_socket_message: SendSocketMessage) -> Result<(), AtError> {
        self.send_and_wait_response(&send_socket_message)?;

        Ok(())
    }

    #[inline]
    fn send_string(&mut self, send_socket_string: SendSocketString) -> Result<(), AtError> {
        self.send_and_wait_response(&send_socket_string)?;

        Ok(())
    }

    #[inline]
    fn connect(&mut self, connect_socket_to_remote: ConnectSocketToRemote) -> Result<(), AtError> {
        self.send_and_wait_response(&connect_socket_to_remote)?;

        Ok(())
    }

    #[inline]
    fn create(&mut self, create_socket: CreateSocket) -> Result<u8, AtError> {
        let socket_id = self.send_and_wait_response(&create_socket)?;

        Ok(socket_id.socket_id)
    }

    #[inline]
    fn close(&mut self, close_socket: CloseSocket) -> Result<(), AtError> {
        self.send_and_wait_response(&close_socket)?;

        Ok(())
    }
}

/// Defines a socket context, which is associated with one socket id.
/// The socket context will be attached to a [Modem] through a lifecycle
pub struct SocketContext<'a, T: SocketContextReceiver, S> {
    socket_id: u8,
    modem: &'a mut T,
    _state: PhantomData<S>,
}

/// Creates a new [SocketContext] using the given modem
pub fn new_socket_context<T: SocketContextReceiver>(
    modem: &mut T,
    domain: Domain,
    connection_type: Type,
    protocol: Protocol,
    cid: Option<i32>,
) -> Result<SocketContext<'_, T, PendingConnection>, AtError> {
    #[cfg(feature = "defmt")]
    debug!("Creating a new HTTP Context");

    let socket_id = modem.create(CreateSocket {
        cid,
        domain,
        connection_type,
        protocol,
    })?;

    Ok(SocketContext {
        socket_id,
        modem,
        _state: Default::default(),
    })
}

fn close_socket_context<T: SocketContextReceiver, S>(
    context: SocketContext<T, S>,
) -> Result<(), AtError> {
    context.modem.close(CloseSocket {
        socket_id: context.socket_id,
    })?;

    Ok(())
}

impl<'a, T: SocketContextReceiver> SocketContext<'a, T, PendingConnection> {
    /// Connects the socket session to the remote server
    pub fn connect_to_remote(
        self,
        port: u16,
        address: &'a str,
    ) -> Result<SocketContext<'a, T, Connected>, AtError> {
        #[cfg(feature = "defmt")]
        debug!("Connecting socket to {}:{}", address, port);

        self.modem.connect(ConnectSocketToRemote {
            port,
            socket_id: self.socket_id,
            remote_address: address,
        })?;

        #[cfg(feature = "defmt")]
        debug!("Socket connected to remote peer OK");

        Ok(SocketContext {
            socket_id: self.socket_id,
            modem: self.modem,
            _state: Default::default(),
        })
    }

    pub fn close(self) -> Result<(), AtError> {
        close_socket_context(self)
    }
}

impl<'a, T: SocketContextReceiver> SocketContext<'a, T, Connected> {
    /// Sends the given string to the remote connection
    pub fn send_string(&mut self, data: &str) -> Result<(), AtError> {
        self.modem.send_string(SendSocketString {
            data,
            socket_id: self.socket_id,
        })?;

        Ok(())
    }

    /// Send the given data to the remote connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<(), AtError> {
        self.modem.send_data(SendSocketMessage {
            data,
            socket_id: self.socket_id,
        })?;

        Ok(())
    }

    pub fn close(self) -> Result<(), AtError> {
        close_socket_context(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::AtError;

    use mockall::{predicate, Sequence};

    #[test]
    fn test_socket_context() -> Result<(), AtError> {
        let mut mock = MockSocketContextReceiver::new();

        let create = CreateSocket {
            cid: None,
            domain: Domain::IPv4,
            connection_type: Type::TCP,
            protocol: Protocol::IP,
        };
        let connect = ConnectSocketToRemote {
            socket_id: 1,
            port: 9999,
            remote_address: "127.0.0.1",
        };
        let send_string = SendSocketString {
            socket_id: 1,
            data: "Hello world!",
        };

        let send_data = SendSocketMessage {
            socket_id: 1,
            data: b"Hello world!",
        };

        let mut seq = Sequence::new();

        mock.expect_create()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(create.clone()))
            .return_const(Ok(1));

        mock.expect_connect()
            .once()
            .in_sequence(&mut seq)
            .withf(move |x| *x == connect)
            .return_const(Ok(()));

        mock.expect_send_data()
            .once()
            .in_sequence(&mut seq)
            .withf(move |x| *x == send_data)
            .return_const(Ok(()));

        mock.expect_send_string()
            .once()
            .in_sequence(&mut seq)
            .withf(move |x| *x == send_string)
            .return_const(Ok(()));

        mock.expect_close()
            .once()
            .in_sequence(&mut seq)
            .withf(|x| *x == CloseSocket { socket_id: 1 })
            .return_const(Ok(()));

        let socket_context =
            new_socket_context(&mut mock, Domain::IPv4, Type::TCP, Protocol::IP, None)?;

        let mut socket_context = socket_context.connect_to_remote(9999, "127.0.0.1")?;

        socket_context.send_data(b"Hello world!")?;

        socket_context.send_string("Hello world!")?;

        socket_context.close()?;

        Ok(())
    }
}
