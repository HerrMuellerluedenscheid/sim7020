use crate::at_command::socket::{
    CloseSocket, ConnectSocketToRemote, CreateSocket, Domain, Protocol, SendSocketMessage,
    SendSocketString, Type,
};
use crate::contexts::common_socket_context::{Connected, PendingConnection};
use crate::nonblocking::AsyncModem;
use crate::AtError;
use core::marker::PhantomData;
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_io::ReadReady;
use embedded_io_async::{Read, Write};

pub struct AsyncSocketContext<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs, S> {
    socket_id: u8,
    modem: &'a mut AsyncModem<W, R, P, D>,
    _state: PhantomData<S>,
}


pub async fn new_async_socket_context<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>(
    modem: &'a mut AsyncModem<W, R, P, D>,
    domain: Domain,
    connection_type: Type,
    protocol: Protocol,
    cid: Option<i32>,
) -> Result<AsyncSocketContext<'a, W, R, P, D, PendingConnection>, AtError> {
    #[cfg(feature = "defmt")]
    debug!("Creating a new HTTP Context");

    let socket_id = modem
        .send_and_wait_response(CreateSocket {
            cid,
            domain,
            connection_type,
            protocol,
        })
        .await?;

    Ok(AsyncSocketContext {
        socket_id: socket_id.socket_id,
        modem,
        _state: Default::default(),
    })
}

async fn close_socket_context<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs, S>(
    context: AsyncSocketContext<'a, W, R, P, D, S>,
) -> Result<(), AtError> {
    context
        .modem
        .send_and_wait_response(CloseSocket {
            socket_id: context.socket_id,
        })
        .await?;

    Ok(())
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>
    AsyncSocketContext<'a, W, R, P, D, PendingConnection>
{
    /// Connects the socket session to the remote server
    pub async fn connect_to_remote(
        self,
        port: u16,
        address: &str,
    ) -> Result<AsyncSocketContext<'a, W, R, P, D, Connected>, AtError> {
        #[cfg(feature = "defmt")]
        debug!("Connecting socket to {}:{}", address, port);

        self.modem
            .send_and_wait_response(ConnectSocketToRemote {
                port,
                socket_id: self.socket_id,
                remote_address: address,
            })
            .await?;

        #[cfg(feature = "defmt")]
        debug!("Socket connected to remote peer OK");

        Ok(AsyncSocketContext {
            socket_id: self.socket_id,
            modem: self.modem,
            _state: Default::default(),
        })
    }

    pub async fn close(self) -> Result<(), AtError> {
        close_socket_context(self).await
    }
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>
    AsyncSocketContext<'a, W, R, P, D, Connected>
{
    /// Sends the given string to the remote connection
    pub async fn send_string(&mut self, data: &str) -> Result<(), AtError> {
        self.modem
            .send_and_wait_response(SendSocketString {
                data,
                socket_id: self.socket_id,
            })
            .await?;

        Ok(())
    }

    /// Send the given data to the remote connection
    pub async fn send_data(&mut self, data: &[u8]) -> Result<(), AtError> {
        self.modem
            .send_and_wait_response(SendSocketMessage {
                data,
                socket_id: self.socket_id,
            })
            .await?;

        Ok(())
    }

    pub async fn close(self) -> Result<(), AtError> {
        close_socket_context(self).await
    }
}
