use crate::at_command::socket::{ConnectSocketToRemote, Domain};
use core::net::IpAddr;

pub mod tcp_socket;

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
