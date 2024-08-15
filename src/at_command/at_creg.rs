use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::{AtError, BUFFER_SIZE};
use defmt::{Format, info};
use strum_macros::FromRepr;

#[derive(FromRepr)]
#[repr(u8)]
enum UnsolicitedCode {
    Disabled=0,
    Enabled=1,
    EnabledWithLocation=2,
}

#[derive(Format)]
pub struct AtCreg;

impl AtRequest for AtCreg {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+CREG")
            .finish()
    }

    // fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
    //     info!("to parse: {=[u8]:a}", data);
    //     // let (unsolicited_code, status) = at_commands::parser::CommandParser::parse(b"\r\n+CREG: (0-2)\r\n\r\nOK\r\n")
    //     //     .expect_identifier(b"\r\n+CREG: (")
    //     //     .expect_int_parameter()
    //     //     .expect_identifier(b"-")
    //     //     .expect_int_parameter()
    //     //     .expect_identifier(b")\r\n\r\nOK\r\n")
    //     //     .finish()
    //     //     .unwrap();
    //
    //
    //     Ok(AtResponse::Ok)
    // }
}
