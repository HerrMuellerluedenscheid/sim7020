use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum EchoState {
    Disabled,
    Enabled,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum Echo {
    Disable = 0,
    Enable = 1,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtEchoState;

impl AtRequest for AtEchoState {
    type Response = ();

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let command = "ATE?\r\n";
        Ok(command.as_bytes())
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtEcho {
    pub status: Echo,
}

impl AtRequest for AtEcho {
    type Response = ();

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let command = match self.status {
            Echo::Disable => "ATE0\r\n",
            Echo::Enable => "ATE1\r\n",
        };
        Ok(command.as_bytes())
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}
