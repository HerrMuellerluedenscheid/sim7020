//! This module contains the definitions of the unsolicited AT responses.

use crate::AtError;

pub mod at_cdnsip_response;

/// Trait that defines a common behavior for all the structs that are
/// unsolicited at messages and need to be deserialized
pub trait AtUnsolicitedResponse: Sized {
    /// Parses the given data into the Self type
    fn parse_response_struct(data: &[u8]) -> Result<Self, AtError>;
}
