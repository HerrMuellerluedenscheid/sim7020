use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::{AtError};
use defmt::{Format, info};

#[derive(Format)]
pub struct PDPContextReadDynamicsParameters;

impl AtRequest for PDPContextReadDynamicsParameters {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CGCONTRDP")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        info!("to parse: {=[u8]:a}", data);
        let (cid, bearer_id, apn, local_address) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CGCONTRDP: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_string_parameter()
            .expect_string_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()
            .unwrap();
        info!("done{} {} {} {} ", cid, bearer_id, apn, local_address);
        Ok(AtResponse::PDPContextDynamicParameters(cid as u8, bearer_id as u8, apn.as_ptr(), local_address.as_ptr()))
    }

}
