use crate::BUFFER_SIZE;
use defmt::error;

pub mod at_cgatt;
pub mod at_cpin;
pub mod at_creg;
pub mod at_csq;
pub mod at_cstt;
pub mod ate;
pub mod ati;
pub mod cgcontrdp;
pub mod http;
pub mod model_identification;
pub mod mqtt;
pub mod network_information;
pub mod ntp;

type BufferType = [u8; BUFFER_SIZE];

pub trait AtResponse {
    fn parse(response: &str) {}
}

pub trait AtRequest {
    type Response;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize>;

    fn get_command_no_error<'a>(&'a self, buffer: &'a mut BufferType) -> &'a [u8] {
        self.get_command(buffer)
            .expect("buffer war not large enough")
    }
}
