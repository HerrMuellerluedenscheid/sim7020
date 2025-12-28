use crate::at_command::socket::{
    CloseSocket, ConnectSocketToRemote, CreateSocket, Domain, Protocol, SendSocketMessage,
    SendSocketString, Type,
};
use crate::contexts::common_socket_context::{Connected, PendingConnection};
use crate::nonblocking::AsyncModem;
use crate::AtError;
use core::marker::PhantomData;
use defmt::debug;
use embedded_io_async::{Read, Write};

pub struct AsyncSocketContext<'a, W: Write, R: Read, S> {
    socket_id: u8,
    modem: &'a mut AsyncModem<W, R>,
    _state: PhantomData<S>,
}

pub async fn new_async_http_session<'a, W: Write, R: Read>(
    modem: &'a mut AsyncModem<W, R>,
    domain: Domain,
    connection_type: Type,
    protocol: Protocol,
    cid: Option<i32>,
) -> Result<AsyncSocketContext<'a, W, R, PendingConnection>, AtError> {
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

async fn close_socket_context<'a, W: Write, R: Read, S>(
    context: AsyncSocketContext<'a, W, R, S>,
) -> Result<(), AtError> {
    context
        .modem
        .send_and_wait_response(CloseSocket {
            socket_id: context.socket_id,
        })
        .await?;

    Ok(())
}

impl<'a, W: Write, R: Read> AsyncSocketContext<'a, W, R, PendingConnection> {
    /// Connects the socket session to the remote server
    pub async fn connect_to_remote(
        self,
        port: u16,
        address: &str,
    ) -> Result<AsyncSocketContext<'a, W, R, Connected>, AtError> {
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

impl<'a, W: Write, R: Read> AsyncSocketContext<'a, W, R, Connected> {
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
