use crate::at_command::at_cgatt::GPRSServiceState;
use crate::at_command::at_cpin::PinStatus;
use crate::at_command::battery::BatteryChargeStatus;
use crate::at_command::flow_control::ControlFlowStatus;
use crate::at_command::mqtt::UsedState;
use crate::at_command::network_information::{NetworkFormat, NetworkMode, NetworkOperator};
use crate::at_command::network_registration_status::{
    NetworkRegistrationStatus, UnsolicitedResultCodes,
};
use crate::at_command::pdp_context::PDPState;
use crate::at_command::power_saving_mode::PowerSavingModeState;
use crate::at_command::sleep_indication::SleepIndication;
use crate::{AtError, BUFFER_SIZE};
#[cfg(feature = "defmt")]
use defmt::debug;

pub mod at;
pub mod at_cgatt;
pub mod at_cpin;
pub mod at_creg;
pub mod at_csq;
pub mod at_cstt;
pub mod ate;
pub mod ati;
pub mod battery;
pub mod ceer;
pub mod cgcontrdp;
pub mod clock;
pub mod cmee;
pub(crate) mod flow_control;
pub mod http;
pub mod ip_address;
pub mod model_identification;
pub mod mqtt;
pub mod network_information;
pub mod network_registration_status;
pub mod ntp;
pub mod pdp_context;
pub mod power_saving_mode;
pub mod sleep_indication;
pub mod socket;
pub mod wireless;

type BufferType = [u8; BUFFER_SIZE];

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[deprecated(since = "3.0.0", note = "Now each type has it's own response type.")]
pub enum AtResponse {
    Ok,
    ModelIdentifier([u8; 8]),
    NTPTimestamp(i64),
    PDPContextDynamicParameters(u8, u8, *const u8, *const u8),
    MQTTSessionCreated(u8), // client_id
    MQTTSession(u8, UsedState, [u8; 50]),
    HTTPSessionCreated(u8),                               // client_id
    HttpSessions(u8, bool, u8, bool, u8, bool, u8, bool), // id0, state0, id1, state1 ...
    PacketDomainAttachmentState(GPRSServiceState),
    NetworkInformationState(NetworkMode, NetworkFormat, Option<NetworkOperator>),
    SignalQuality(i32, i32),
    ReportMobileEquipmentErrorSetting(i32),
    NetworkRegistration(UnsolicitedResultCodes, NetworkRegistrationStatus),
    NetworkRegistrationStatus(UnsolicitedResultCodes, NetworkRegistrationStatus),
    PDPContext(Option<(PDPState, i32)>),
    SleepIndication(SleepIndication),
    PowerSavingMode(PowerSavingModeState),
    BatteryCharge(BatteryChargeStatus),
    ControlFlow(ControlFlowStatus, ControlFlowStatus),
    LocalIPAddress(i32),
    SocketCreated(u8),
    SocketConnected,
    PinStatus(PinStatus),
}

pub trait AtRequest {
    type Response;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize>;

    fn get_command_no_error<'a>(&'a self, buffer: &'a mut BufferType) -> &'a [u8] {
        self.get_command(buffer).expect("buffer too small")
    }

    #[deprecated(since = "3.0.0", note = "Migrate to parse_response_struct")]
    #[allow(deprecated)]
    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        debug!("default parsing: {=[u8]:a}", _data);
        Ok(AtResponse::Ok)
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        todo!("Not implemented yet for this type")
    }
}
