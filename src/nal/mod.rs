use crate::at_command::socket::{ConnectSocketToRemote, Domain};
use crate::AtError;
use core::cell::{OnceCell, RefCell};
use core::error::Error;
use core::fmt::Display;
use core::fmt::Formatter;
use core::net::IpAddr;
use embedded_nal::{TcpError, TcpErrorKind};

#[cfg(all(feature = "async-nal", feature = "experimental"))]
pub mod nonblocking;
#[cfg(feature = "nal")]
pub mod tcp_socket;

/// Max size an IP (v4 or v6) can have
const MAX_IP_SIZE: usize = 45;

impl From<IpAddr> for Domain {
    fn from(value: IpAddr) -> Self {
        if value.is_ipv4() {
            Domain::IPv4
        } else if value.is_ipv6() {
            Domain::IPv6
        } else {
            unreachable!()
        }
    }
}

/// Contains the current socket status
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Debug)]
enum SocketStatus {
    Connected,
    Disconnected,
}

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

impl Display for AtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("AtError")
    }
}

impl Error for AtError {}

impl embedded_io::Error for AtError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}
