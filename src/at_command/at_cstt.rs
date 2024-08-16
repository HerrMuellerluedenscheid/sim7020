use crate::at_command::{AtRequest, BufferType};
use crate::AtError;
use defmt::Format;

const CSTT_SIZE_MAX: usize = 32; // AT Datasheet page 172

#[derive(Format)]
/// Enter PIN.
pub struct GetAPNUserPassword {}

impl AtRequest for GetAPNUserPassword {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_test(buffer, true)
            .named("+CSTT")
            .finish()
    }
}

#[derive(Format)]
/// Enter PIN.
pub struct SetAPNUserPassword {
    pub(crate) apn: Option<[u8; CSTT_SIZE_MAX]>,
    pub(crate) user: Option<[u8; CSTT_SIZE_MAX]>,
    pub(crate) password: Option<[u8; CSTT_SIZE_MAX]>,
}

impl SetAPNUserPassword {
    pub fn new() -> Self {
        Self {
            apn: None,
            user: None,
            password: None,
        }
    }
    pub fn with_apn(mut self, apn: &str) -> Self {
        let mut apn_b = [b'\0'; CSTT_SIZE_MAX];
        for (i, b) in apn.as_bytes().iter().enumerate() {
            apn_b[i] = *b;
        }
        self.apn = Some(apn_b);
        self
    }
}

impl AtRequest for SetAPNUserPassword {
    type Response = Result<(), AtError>;

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("CSTT")
            .with_optional_string_parameter(self.apn)
            .with_optional_string_parameter(self.user)
            .with_optional_string_parameter(self.password)
            .finish()
    }
}
