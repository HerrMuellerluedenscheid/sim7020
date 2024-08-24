use crate::at_command::{AtRequest, BufferType};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AtI;

pub struct ProductInformation {}

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("I")
            .finish()
    }
}
