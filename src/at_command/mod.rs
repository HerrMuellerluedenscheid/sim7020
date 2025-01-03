use crate::{AtError, BUFFER_SIZE};

pub mod at_cgatt;
pub mod at_cpin;
pub mod at_creg;
pub mod at_csq;
pub mod at_cstt;
pub mod ate;
pub mod ati;
pub mod ceer;
pub mod cgcontrdp;
pub mod cmee;
pub mod http;
pub mod model_identification;
pub mod mqtt;
pub mod network_information;
pub mod ntp;

type BufferType = [u8; BUFFER_SIZE];

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AtResponse {
    Ok,
    ModelIdentifier([u8; 8]),
    NTPTimestamp(i64),
    PDPContextDynamicParameters(u8, u8, *const u8, *const u8),
    MQTTSessionCreated(u8),                               // client_id
    HTTPSessionCreated(u8),                               // client_id
    HttpSessions(u8, bool, u8, bool, u8, bool, u8, bool), // id0, state0, id1, state1 ...
    PacketDomainAttachmentState(bool),                    // true: attached, false: detached
    SignalQuality(i32, i32),
    ReportMobileEquipmentErrorSetting(i32),
}

pub trait AtRequest {
    type Response;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize>;

    fn get_command_no_error<'a>(&'a self, buffer: &'a mut BufferType) -> &'a [u8] {
        self.get_command(buffer).expect("buffer too small")
    }

    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        Ok(AtResponse::Ok)
    }
}
