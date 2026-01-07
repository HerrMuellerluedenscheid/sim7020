//! Implementation of embedded NAL for a Modem
//!
//! This module contains the [TcpClientStack]
use super::*;
use crate::at_command::receive::socket_reception::SocketDataReception;
use crate::at_command::socket::{
    CloseSocket, CreateSocket, Protocol, SendSocketMessage, SendSocketString, Type,
};
use crate::{AtError, Modem};
use core::cell::{OnceCell, RefCell};
use core::net::SocketAddr;
use defmt::{debug, warn};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, ReadReady, Write};
use embedded_nal::{TcpClientStack, TcpError, TcpErrorKind};
use heapless::format;

impl TcpError for AtError {
    fn kind(&self) -> TcpErrorKind {
        TcpErrorKind::Other
    }
}

/// Socket context containing the information of the TCP connection
pub struct TcpSocketContext {
    socket_id: OnceCell<u8>,
    connected: RefCell<SocketStatus>,
}

/// Max size an IP (v4 or v6) can have
const MAX_IP_SIZE: usize = 45;

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> TcpClientStack
    for Modem<'a, W, R, P, D>
{
    type TcpSocket = TcpSocketContext;
    type Error = AtError;

    fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
        Ok(TcpSocketContext {
            connected: RefCell::new(SocketStatus::Disconnected),
            socket_id: OnceCell::new(),
        })
    }

    fn connect(
        &mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr,
    ) -> embedded_nal::nb::Result<(), Self::Error> {
        if *socket.connected.borrow() != SocketStatus::Disconnected {
            #[cfg(feature = "defmt")]
            warn!("The socket is already connected");
            return Ok(());
        }
        let connection_type: Domain = remote.ip().into();
        let remote_address = remote.ip();
        let remote_address: heapless::String<MAX_IP_SIZE> =
            format!("{}", remote_address).map_err(|_| AtError::CapacityError)?;

        let socket_id = self
            .send_and_wait_response(&CreateSocket {
                cid: None,
                domain: connection_type,
                protocol: Protocol::IP,
                connection_type: Type::TCP,
            })?
            .socket_id;

        socket
            .socket_id
            .set(socket_id)
            .map_err(|_| AtError::IllegalModuleState)?;

        #[cfg(feature = "defmt")]
        debug!("Connecting to {}", remote_address);

        let remote_ip = remote_address.as_str();

        self.send_and_wait_response(&ConnectSocketToRemote {
            port: remote.port(),
            socket_id,
            remote_address: remote_ip,
        })?;

        socket.connected.replace(SocketStatus::Connected);

        Ok(())
    }

    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> embedded_nal::nb::Result<usize, Self::Error> {
        socket.ensure_connected()?;
        let socket_id = *socket.socket_id.get().ok_or(AtError::IllegalModuleState)?;
        self.send_and_wait_response(&SendSocketMessage {
            socket_id,
            data: buffer,
        })?;

        Ok(buffer.len())
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> embedded_nal::nb::Result<usize, Self::Error> {
        socket.ensure_connected()?;
        let socket_data: SocketDataReception = self.get_unsolicited_message()?;
        let socket_id = *socket.socket_id.get().ok_or(AtError::IllegalModuleState)?;
        if socket_data.socket_id != socket_id {
            return Err(AtError::IOError.into());
        }

        buffer.copy_from_slice(socket_data.data.as_slice());

        Ok(socket_data.data.len())
    }

    fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let socket_id = *socket.socket_id.get().ok_or(AtError::IllegalModuleState)?;
        self.send_and_wait_response(&CloseSocket { socket_id })?;

        Ok(())
    }
}

pub trait TcpClientStackSendStr: TcpClientStack {
    fn send_str(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &str,
    ) -> embedded_nal::nb::Result<(), Self::Error>;
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> TcpClientStackSendStr
    for Modem<'a, W, R, P, D>
{
    fn send_str(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &str,
    ) -> embedded_nal::nb::Result<(), Self::Error> {
        socket.ensure_connected()?;
        let socket_id = *socket.socket_id.get().ok_or(AtError::IllegalModuleState)?;
        self.send_and_wait_response(&SendSocketString {
            socket_id,
            data: buffer,
        })?;

        Ok(())
    }
}

impl TcpSocketContext {
    #[inline]
    fn ensure_connected(&self) -> Result<(), AtError> {
        if *self.connected.borrow() != SocketStatus::Connected {
            #[cfg(feature = "defmt")]
            warn!("The socket is not connected");

            return Err(AtError::IllegalModuleState);
        }
        Ok(())
    }
}
