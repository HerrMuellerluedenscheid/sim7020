//! This module contains the code related to at_commands that will be used to communicate with
//! the SIM7020 module.
//! All this commands are defined in the AT commands guide of the module
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
use crate::AtError;
#[cfg(feature = "defmt")]
use defmt::debug;

pub mod at;
pub mod at_cgatt;
pub mod at_cpin;
pub mod at_creg;
pub mod at_csq;
pub mod at_cstt;
pub mod at_psd;
pub mod ate;
pub mod ati;
pub mod battery;
pub mod ceer;
pub mod cgcontrdp;
pub mod clock;
pub mod cmee;
pub mod csclk;
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

// We have to do this workaround because the derive causes deprecation warnings.
// The workaround allows deprecations in the deprecated module and then we
mod deprecated {
    #![allow(deprecated)]
    use super::*;
    #[deprecated(since = "3.0.0", note = "Now each type has it's own response type.")]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
}

// We re-export with deprectation the AtResponse so we do not break any current implementation
#[allow(deprecated)]
#[deprecated(since = "3.0.0", note = "Now each type has it's own response type.")]
pub type AtResponse = deprecated::AtResponse;

/// Defines a generic trait for the defined AT commands
pub trait AtRequest {
    /// Type response of the command
    type Response;

    /// Gets the command to be sent to the module
    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize>;

    /// Gets the command to be sent to the module unwrapping any possible error
    fn get_command_no_error<'a>(&'a self, buffer: &'a mut [u8]) -> &'a [u8] {
        self.get_command(buffer).expect("buffer too small")
    }

    /// Parses the response and get the [AtResponse] this method is deprecated and
    /// [parse_response_struct] should be used instead
    #[deprecated(since = "3.0.0", note = "Migrate to parse_response_struct")]
    #[allow(deprecated)]
    fn parse_response(&self, _data: &[u8]) -> Result<AtResponse, AtError> {
        #[cfg(feature = "defmt")]
        debug!("default parsing: {=[u8]:a}", _data);
        Ok(AtResponse::Ok)
    }

    /// Parses the given data and returns a [Result] which contains the [Response] type defined
    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError>;
}

/// Verifies if the data contains an OK ignoring any leading whitespaces
pub(crate) fn verify_ok(data: &[u8]) -> Result<(), AtError> {
    at_commands::parser::CommandParser::parse(data)
        .trim_whitespace()
        .expect_identifier(b"OK")
        .finish()?;

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_very_ok() {
        const OK_1: &[u8] = b"\r\nOK";
        verify_ok(OK_1).unwrap();
        const OK_2: &[u8] = b"\r\nOK\r\n";
        verify_ok(OK_2).unwrap();
        const OK_3: &[u8] = b"OK\r\n";
        verify_ok(OK_3).unwrap();
        const OK_4: &[u8] = b"OK";
        verify_ok(OK_4).unwrap();
    }
}
