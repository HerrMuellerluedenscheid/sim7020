use crate::at_command::{AtRequest, BufferType};
use crate::AtError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Echo {
    Disable = 0,
    Enable = 1,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy)]
pub struct AtEcho {
    pub status: Echo,
}

impl AtRequest for AtEcho {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, _buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let command = match self.status {
            Echo::Disable => "ATE0\r\n",
            Echo::Enable => "ATE1\r\n",
        };
        Ok(command.as_bytes())
    }
}
