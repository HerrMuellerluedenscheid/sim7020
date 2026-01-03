use crate::at_command::{AtRequest, BufferType};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct AtI;

pub struct ProductInformation;

impl AtRequest for AtI {
    type Response = ProductInformation;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_query(buffer, true)
            .named("I")
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, crate::AtError> {
        Ok(ProductInformation {})
    }
}
