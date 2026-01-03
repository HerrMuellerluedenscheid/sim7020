//! Contains the definitions for the socket contexts

use core::marker::PhantomData;

#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, ReadReady, Write};

use crate::at_command::socket::*;
use crate::contexts::common_socket_context::{Connected, PendingConnection};
use crate::{AtError, Modem};

/// Defines a socket context, which is associated with one socket id.
/// The socket context will be attached to a [Modem] thorugh a lifecycle
pub struct SocketContext<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs, S> {
    socket_id: u8,
    modem: &'a mut Modem<'a, W, R, P, D>,
    _state: PhantomData<S>,
}

/// Creates a new [SocketContext] using the given modem
pub fn new_socket_context<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>(
    modem: &'a mut Modem<'a, W, R, P, D>,
    domain: Domain,
    connection_type: Type,
    protocol: Protocol,
    cid: Option<i32>,
) -> Result<SocketContext<'a, W, R, P, D, PendingConnection>, AtError> {
    #[cfg(feature = "defmt")]
    debug!("Creating a new HTTP Context");

    let socket_id = modem.send_and_wait_response(&CreateSocket {
        cid,
        domain,
        connection_type,
        protocol,
    })?;

    Ok(SocketContext {
        socket_id: socket_id.socket_id,
        modem,
        _state: Default::default(),
    })
}

fn close_socket_context<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs, S>(
    context: SocketContext<W, R, P, D, S>,
) -> Result<(), AtError> {
    context.modem.send_and_wait_response(&CloseSocket {
        socket_id: context.socket_id,
    })?;

    Ok(())
}

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>
    SocketContext<'a, W, R, P, D, PendingConnection>
{
    /// Connects the socket session to the remote server
    pub fn connect_to_remote(
        self,
        port: u16,
        address: &str,
    ) -> Result<SocketContext<'a, W, R, P, D, Connected>, AtError> {
        #[cfg(feature = "defmt")]
        debug!("Connecting socket to {}:{}", address, port);

        self.modem.send_and_wait_response(&ConnectSocketToRemote {
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

impl<'a, W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs>
    SocketContext<'a, W, R, P, D, Connected>
{
    /// Sends the given string to the remote connection
    pub fn send_string(&mut self, data: &str) -> Result<(), AtError> {
        self.modem.send_and_wait_response(&SendSocketString {
            data,
            socket_id: self.socket_id,
        })?;

        Ok(())
    }

    /// Send the given data to the remote connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<(), AtError> {
        self.modem.send_and_wait_response(&SendSocketMessage {
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
    use std::{cell::RefCell, sync::Mutex};

    use crate::{AtError, Modem};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{State as PinState, Transaction as PinTransaction};
    use embedded_io::{ErrorType, Read, ReadReady, Write};
    use mockall::{mock, predicate};
    use std::sync::Arc;

    mock! {
        TestReader{}
        impl ErrorType for TestReader {
            type Error = embedded_io::ErrorKind;
        }
        impl Read for TestReader {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize, embedded_io::ErrorKind>;
        }

        impl ReadReady for TestReader {
            fn read_ready(&mut self) -> Result<bool, embedded_io::ErrorKind> {
                Ok(true)
            }
        }
    }

    mock! {
        TestWriter{}

        impl ErrorType for TestWriter {
            type Error = embedded_io::ErrorKind;
        }

        impl Write for TestWriter {
            fn write(&mut self, buf: &[u8]) -> Result<usize, embedded_io::ErrorKind>;
            fn flush(&mut self) -> Result<(), embedded_io::ErrorKind>;
        }

    }

    #[test]
    #[ignore = "Refactor will be made"]
    fn test_socket_context() -> Result<(), AtError> {
        let mut mock_writer = MockTestWriter::new();
        let mut mock_reader = MockTestReader::new();

        let echo_off = [65, 84, 69, 48, 13, 10];
        let create = b"AT+CSOC=1,1,1,3\r\n";
        let connect = b"AT+CSOCON=1,1111,\"127.0.0.1\"\r\n";
        let send_string = b"AT+CSOSEND=1,0,\"HELLO TEST\"\r\n";
        let close = b"AT+CSOCL=1\r\n";

        mock_writer
            .expect_write()
            .with(predicate::eq(Vec::from(echo_off)))
            .return_const(Ok(echo_off.len()));

        mock_writer
            .expect_write()
            .with(predicate::eq(Vec::from(create)))
            .return_const(Ok(create.len()));

        mock_writer
            .expect_write()
            .with(predicate::eq(Vec::from(connect)))
            .return_const(Ok(connect.len()));

        mock_writer
            .expect_write()
            .with(predicate::eq(Vec::from(send_string)))
            .return_const(Ok(send_string.len()));

        mock_writer
            .expect_write()
            .with(predicate::eq(Vec::from(close)))
            .return_const(Ok(close.len()));

        let create_response = b"+CSOC: 1\r\n\r\nOK\r\n";
        let ok_response = b"\r\nOK\r\n";

        let once = Arc::new(Mutex::new(TimesMatcher::new(2)));

        let cloned = once.clone();

        mock_reader
            .expect_read()
            .with(predicate::function(move |_| {
                let data = cloned.lock().unwrap();
                data.get()
            }))
            .returning(|mut a| {
                a.write(create_response).expect("Failed writing");
                Ok(create_response.len())
            });

        mock_reader.expect_read().times(3).returning(|mut a| {
            a.write(ok_response).expect("Failed writing");
            Ok(ok_response.len())
        });

        let power_pin_expectation = [PinTransaction::get(PinState::High)];

        let dtr_pin_expectation = [PinTransaction::get(PinState::Low)];

        let power_pin = embedded_hal_mock::eh1::digital::Mock::new(&power_pin_expectation);
        let dtr_pin = embedded_hal_mock::eh1::digital::Mock::new(&dtr_pin_expectation);

        let mut modem = Modem::new(
            &mut mock_writer,
            &mut mock_reader,
            power_pin,
            dtr_pin,
            NoopDelay,
        )
        .unwrap();

        let context = super::new_socket_context(
            &mut modem,
            crate::at_command::socket::Domain::IPv4,
            crate::at_command::socket::Type::TCP,
            crate::at_command::socket::Protocol::IP,
            Some(3),
        )?;

        let mut connected_socket = context.connect_to_remote(1111, "127.0.0.1").unwrap();

        connected_socket.send_string("HELLO TEST").unwrap();

        connected_socket.close()?;

        Ok(())
    }

    struct TimesMatcher {
        matched: RefCell<i64>,
    }

    impl TimesMatcher {
        fn new(times: i64) -> Self {
            Self {
                matched: RefCell::new(times),
            }
        }

        fn get(&self) -> bool {
            let old = self.matched.replace_with(|&mut old| old - 1);
            old > 0
        }
    }
}
