use crate::at_command::{AtRequest, BufferType};
use crate::{AtError, BUFFER_SIZE};
use at_commands::builder::CommandBuilder;
use defmt::export::write;
use defmt::{error, info, Format};
use embedded_io::Write;
use hex;

#[derive(Format)]
/// Create a new MQTT connection
pub struct NewMQTTConnection<'a> {
    pub server: &'a str,
    pub port: u16,               // 0 - 65535
    pub timeout_ms: u16,         // 0 - 60.000
    pub buffer_size: u16,        // 20 - 1132
    pub context_id: Option<u16>, // PDP context, AT+CGAT response
}

impl AtRequest for NewMQTTConnection<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        // TODO: move into new
        if self.timeout_ms > 60000 {
            error!("timeout is out of range")
        }
        if (self.buffer_size > 1132) | (self.buffer_size < 20) {
            error!("buffer_size is out of range")
        }

        CommandBuilder::create_set(buffer, true)
            .named("+CMQNEW")
            .with_string_parameter(self.server)
            .with_int_parameter(self.port)
            .with_int_parameter(self.timeout_ms)
            .with_int_parameter(self.buffer_size)
            // .with_optional_int_parameter(self.context_id)
            .finish()
    }
}

#[derive(Format)]
pub struct CloseMQTTConnection {}

impl AtRequest for CloseMQTTConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        // todo fix hard coded client id
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CMQDISCON")
            .with_int_parameter(0)
            .finish()
    }
}

#[derive(Format)]
#[repr(u8)]
pub enum MQTTVersion {
    MQTT31,
    MQTT311,
}

pub struct WillOptions<'a> {
    pub topic: &'a str,
    pub QoS: u8,
    pub retained: bool,
}

#[derive(Format)]
pub struct MQTTConnect<'a> {
    pub mqtt_id: u8,
    pub version: MQTTVersion,
    pub client_id: &'a str,
    pub keepalive_interval: u16, // 0 - 64800
    pub clean_session: bool,
    pub will_flag: bool,
    // pub will_options: Option<WillOptions>,
    pub username: &'a str,
    pub password: &'a str,
}

impl AtRequest for MQTTConnect<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let version: u8 = match self.version {
            MQTTVersion::MQTT31 => 3,
            MQTTVersion::MQTT311 => 4,
        };
        CommandBuilder::create_set(buffer, true)
            .named("+CMQCON")
            .with_int_parameter(self.mqtt_id)
            .with_int_parameter(version)
            .with_string_parameter(&self.client_id)
            .with_int_parameter(self.keepalive_interval)
            .with_int_parameter(self.clean_session as u8)
            .with_int_parameter(self.will_flag as u8)
            .with_string_parameter(&self.username)
            .with_string_parameter(&self.password)
            .finish()
    }
}

#[derive(Format)]
pub enum MQTTDataFormat {
    Bytes,
    Hex,
}

#[derive(Format)]
pub struct MQTTRawData {
    pub data_format: MQTTDataFormat,
}

impl AtRequest for MQTTRawData {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        let format = match self.data_format {
            MQTTDataFormat::Bytes => "0",
            MQTTDataFormat::Hex => "1",
        };

        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CREVHEX")
            .with_string_parameter(format)
            .finish()
    }
}

#[derive(Format)]
pub struct MQTTPublish<'a> {
    pub mqtt_id: u8,      // AT+CMQNEW response
    pub topic: &'a str,   // length max 128b
    pub qos: u8,          // 0 | 1 | 2
    pub retained: bool,   // 0 | 1
    pub dup: bool,        // 0 | 1
    pub message: &'a str, // as hex
}

impl AtRequest for MQTTPublish<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        CommandBuilder::create_set(buffer, true)
            .named("+CMQPUB")
            .with_int_parameter(self.mqtt_id)
            .with_string_parameter(self.topic)
            .with_int_parameter(self.qos)
            .with_int_parameter(self.retained as u8)
            .with_int_parameter(self.dup as u8)
            .with_int_parameter(self.message.len() as i32)
            .with_string_parameter(self.message.as_bytes())
            .finish()
    }
}
