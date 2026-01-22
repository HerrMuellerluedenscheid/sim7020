//! This module contains the implementations to have a shared [AsyncModem]

use crate::nonblocking::AsyncModem;
use core::cell::RefCell;
use core::ops::Deref;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_io::ReadReady;
use embedded_io_async::{Read, Write};

/// Shared version of [AsyncModem]
///
/// *Warning* This struct is not concurrent in any way. Any method from this struct or structs depending
/// on this struct may fail due to [RefCell] rules
///
/// ***This struct is experimental***
pub struct SharedAsyncModem<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> {
    inner: RefCell<AsyncModem<W, R, P, D>>,
}

impl<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> SharedAsyncModem<W, R, P, D> {
    /// Creates a new [SharedAsyncModem]
    pub fn new(inner: AsyncModem<W, R, P, D>) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }

    /// Takes ownership of the internal [AsyncModem]
    pub fn into_inner(self) -> AsyncModem<W, R, P, D> {
        self.inner.into_inner()
    }
}

impl<W: Write, R: Read + ReadReady, P: OutputPin, D: DelayNs> Deref
    for SharedAsyncModem<W, R, P, D>
{
    type Target = RefCell<AsyncModem<W, R, P, D>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
