//! Module with convenience structs to help with context messages such as MQTT, Socket or HTTP
//!
//! Under [nonblocking] there are the async implementations
//! Under [blocking] are the blocking implementations
//!
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

pub mod blocking;

pub mod common_socket_context;
