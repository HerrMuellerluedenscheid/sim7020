//! Contains the source code that allows controlling AT commands for CSCLK

use crate::at_command::{verify_ok, AtRequest, BufferType};
use crate::AtError;

/// The modes CSLK can be controlled
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Eq, Copy, Clone, Default)]
#[repr(u8)]
pub enum CSCLKMode {
    /// The module does not go to sleep
    #[default]
    Disabled = 0,
    /// The module sleep is controlled with the DTR pin
    HardwareControlled = 1,
    /// The module sleep is controlled by the own module in idle periods
    SoftwareControlled = 2,
}

/// Struct that can be used to control the module CSCLK mode
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Eq)]
pub struct SetCSCLKMode {
    pub mode: CSCLKMode,
}

impl AtRequest for SetCSCLKMode {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("CSCLK")
            .with_int_parameter(self.mode as u8)
            .finish()
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        verify_ok(data)
    }
}
