use crate::at_command::{AtRequest, BufferType};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct StartWirelessConnection;

impl AtRequest for StartWirelessConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_execute(buffer, true)
            .named("+CIICR")
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, crate::AtError> {
        Ok(())
    }
}
