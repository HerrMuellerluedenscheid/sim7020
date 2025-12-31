//! Contains the common definitions for the socket contexts.
//!
//! This module contains some markers that helps identifying the different states a socket could be
//!
//!

/// Marker interface to indicate the state of [HttpContext] and [AsyncHttpContext]
pub struct PendingConnection;
/// Marker interface to indicate the state of [HttpContext] and [AsyncHttpContext]
pub struct Connected;
/// Marker interface to indicate the state of [HttpContext] and [AsyncHttpContext]
pub struct Closed;
