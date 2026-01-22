//! Implementation of embedded async NAL for a AsyncModem
//!
//! This module contains the [TcpClientStack]

use super::MAX_IP_SIZE;
use crate::at_command::receive::socket_reception::SocketDataReception;
use crate::at_command::socket::{
    CloseSocket, ConnectSocketToRemote, CreateSocket, Domain, Protocol, SendSocketMessage, Type,
};
use crate::nal::SocketStatus;
use crate::nonblocking::shared_async_modem::SharedAsyncModem;
use crate::nonblocking::AsyncModem;
use crate::AtError;
use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::net::SocketAddr;
use core::pin::pin;
use core::task::Poll;
#[cfg(feature = "defmt")]
use defmt::{debug, info, warn};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_io::ErrorType;
use embedded_io_async::{Read, ReadReady, Write};
use embedded_nal_async::TcpConnect;
use heapless::format;

/// TCP context that can be used to send/recv TPC content
pub struct AsyncTcpContext<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> {
    socket_id: u8,
    modem: &'a SharedAsyncModem<W, R, P, D>,
    status: RefCell<SocketStatus>,
}

impl<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> SharedAsyncModem<W, R, P, D> {
    fn with_mut<T>(&self, f: impl FnOnce(&mut AsyncModem<W, R, P, D>) -> T) -> T {
        f(&mut self.borrow_mut())
    }
}

impl<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> TcpConnect
    for SharedAsyncModem<W, R, P, D>
{
    type Error = AtError;
    type Connection<'a>
        = AsyncTcpContext<'a, W, R, P, D>
    where
        Self: 'a;

    async fn connect<'a>(
        &'a self,
        remote: SocketAddr,
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let connection_type: Domain = remote.ip().into();
        let remote_address = remote.ip();
        let remote_address: heapless::String<MAX_IP_SIZE> =
            format!("{}", remote_address).map_err(|_| AtError::CapacityError)?;

        let socket_id = poll_fn(|cx| {
            self.with_mut(|modem| {
                let mut modem = pin!(modem);
                let future = modem.send_and_wait_response(CreateSocket {
                    cid: None,
                    domain: connection_type,
                    protocol: Protocol::IP,
                    connection_type: Type::TCP,
                });

                let future = pin!(future);

                let result = future.poll(cx);

                match result {
                    Poll::Ready(result) => Poll::Ready(result),
                    _ => Poll::Pending,
                }
            })
        })
        .await?
        .socket_id;

        #[cfg(feature = "defmt")]
        debug!("Connecting to {}", remote_address);

        let remote_ip = remote_address.as_str();

        poll_fn(|cx| {
            self.with_mut(|modem| {
                let mut modem = pin!(modem);

                let future = modem.send_and_wait_response(ConnectSocketToRemote {
                    socket_id,
                    remote_address: remote_ip,
                    port: remote.port(),
                });

                let future = pin!(future);

                let result = future.poll(cx);

                match result {
                    Poll::Ready(result) => Poll::Ready(result),
                    _ => Poll::Pending,
                }
            })
        })
        .await?;

        let context = AsyncTcpContext {
            socket_id,
            modem: self,
            status: RefCell::new(SocketStatus::Connected),
        };

        Ok(context)
    }
}

impl<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> ErrorType
    for AsyncTcpContext<'_, W, R, P, D>
{
    type Error = AtError;
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> Read
    for AsyncTcpContext<'a, W, R, P, D>
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.ensure_connected()?;
        let message: SocketDataReception = poll_fn(|cx| {
            self.modem.with_mut(|modem| {
                let mut modem = pin!(modem);

                let future = modem.get_unsolicited_message();

                let future = pin!(future);

                if let Poll::Ready(result) = future.poll(cx) {
                    Poll::Ready(result)
                } else {
                    Poll::Pending
                }
            })
        })
        .await?;

        buf.copy_from_slice(message.data.as_slice());

        Ok(message.data.len())
    }
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> Write
    for AsyncTcpContext<'a, W, R, P, D>
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.ensure_connected()?;

        poll_fn(|cx| {
            self.modem.with_mut(|modem| {
                let mut modem = pin!(modem);

                let future = modem.send_and_wait_response(SendSocketMessage {
                    socket_id: self.socket_id,
                    data: buf,
                });

                let future = pin!(future);

                if let Poll::Ready(result) = future.poll(cx) {
                    Poll::Ready(result)
                } else {
                    Poll::Pending
                }
            })
        })
        .await?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        #[cfg(feature = "defmt")]
        info!("Called flush on async TCP connection which has no real effect");
        Ok(())
    }
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> Drop
    for AsyncTcpContext<'a, W, R, P, D>
{
    fn drop(&mut self) {
        if *self.status.borrow() == SocketStatus::Connected {
            #[cfg(feature = "defmt")]
            warn!("The AsyncTcpContext is dropped while is already connected. As this struct is async we will try to call async to close the socket, but we can not ensure the competition of it. Calling disconnect is highly advised");
            // We can not await the future. We need to await for some kind of AsyncDrop
            let mut future = pin!(self.disconnect_internal());

            let _poll_result = poll_fn(move |cx| future.as_mut().poll(cx));
        }
    }
}
impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> AsyncTcpContext<'a, W, R, P, D> {
    fn ensure_connected(&mut self) -> Result<(), AtError> {
        if *self.status.borrow() != SocketStatus::Connected {
            #[cfg(feature = "defmt")]
            info!("The socket context is not already connected");

            Err(AtError::IllegalModuleState)
        } else {
            Ok(())
        }
    }

    /// Internal method to perform the socket closing
    async fn disconnect_internal(&mut self) -> Result<(), AtError> {
        poll_fn(|cx| {
            self.modem.with_mut(|modem| {
                let mut modem = pin!(modem);

                let future = modem.send_and_wait_response(CloseSocket {
                    socket_id: self.socket_id,
                });

                let future = pin!(future);

                if let Poll::Ready(result) = future.poll(cx) {
                    Poll::Ready(result)
                } else {
                    Poll::Pending
                }
            })
        })
        .await?;

        self.status.replace(SocketStatus::Disconnected);

        Ok(())
    }

    /// Disconnects the socket.
    pub async fn disconnect(mut self) -> Result<(), AtError> {
        // We take the ownership as the future socket will not have any real effect
        self.disconnect_internal().await
    }
}
