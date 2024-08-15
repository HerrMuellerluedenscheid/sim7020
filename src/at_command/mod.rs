use crate::{AtError, BUFFER_SIZE};
use defmt::{error, info};
use crate::at_command::http::HttpClient;

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

#[derive(defmt::Format)]
pub enum AtResponse {
    Ok,
    ModelIdentifier([u8; 8]),
    PDPContextDynamicParameters(u8, u8, *const u8, *const u8),
    HTTPSessionCreated(HttpClient),
}


pub trait AtRequest {

    type Response;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize>;

    fn get_command_no_error<'a>(&'a self, buffer: &'a mut BufferType) -> &'a [u8] {
        self.get_command(buffer)
            .expect("buffer war not large enough")
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        info!("parsing: {=[u8]:a}", data);
        Ok(AtResponse::Ok)
    }
}
