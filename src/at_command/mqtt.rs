use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::AtError;
use at_commands::builder::CommandBuilder;
#[cfg(feature = "defmt")]
use defmt::error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Create a new MQTT connection
pub struct NewMQTTConnection<'a> {
    pub server: &'a str,
    pub port: u16,               // 0 - 65535
    pub timeout_ms: u16,         // 0 - 60.000
    pub buffer_size: u16,        // 20 - 1132
    pub context_id: Option<u16>, // PDP context, AT+CGAT response
}

impl NewMQTTConnection<'_> {
    pub fn new<'b>(
        server: &'b str,
        port: u16,
        timeout_ms: u16,
        buffer_size: u16,
        context_id: Option<u16>,
    ) -> NewMQTTConnection<'b> {
        // TODO: move into new
        if timeout_ms > 60000 {
            #[cfg(feature = "defmt")]
            error!("timeout is out of range")
        }
        if (buffer_size > 1132) | (buffer_size < 20) {
            #[cfg(feature = "defmt")]
            error!("buffer_size is out of range")
        }
        NewMQTTConnection {
            server,
            port,
            timeout_ms,
            buffer_size,
            context_id,
        }
    }
}

impl AtRequest for NewMQTTConnection<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        CommandBuilder::create_set(buffer, true)
            .named("+CMQNEW")
            .with_string_parameter(self.server)
            .with_int_parameter(self.port)
            .with_int_parameter(self.timeout_ms)
            .with_int_parameter(self.buffer_size)
            // .with_optional_int_parameter(self.context_id)
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (mqtt_id,) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CMQNEW: ")
            .expect_int_parameter()
            .expect_identifier(b"\r\n\r\nOK\r\n")
            .finish()
            .unwrap();

        Ok(AtResponse::MQTTSessionCreated(mqtt_id as u8))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CloseMQTTConnection {
    pub mqtt_id: u8,
}

impl AtRequest for CloseMQTTConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        // todo fix hard coded client id
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CMQDISCON")
            .with_int_parameter(self.mqtt_id)
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MQTTVersion {
    MQTT31,
    MQTT311,
}

pub struct WillOptions<'a> {
    pub topic: &'a str,
    pub quality_of_service: u8,
    pub retained: bool,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
            .with_string_parameter(self.client_id)
            .with_int_parameter(self.keepalive_interval)
            .with_int_parameter(self.clean_session as u8)
            .with_int_parameter(self.will_flag as u8)
            .with_string_parameter(self.username)
            .with_string_parameter(self.password)
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MQTTDataFormat {
    Bytes,
    Hex,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Publish a message via mqtt
///
/// The message length has to be between 2 and 1000 byte.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MQTTPublish<'a> {
    pub mqtt_id: u8,      // AT+CMQNEW response
    pub topic: &'a str,   // length max 128b
    pub qos: u8,          // 0 | 1 | 2
    pub retained: bool,   // 0 | 1
    pub dup: bool,        // 0 | 1
    pub message: &'a [u8], // as hex
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
            .with_string_parameter(self.message)
            .finish()
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MQTTSubscribe<'a> {
    pub mqtt_id: u8,    // AT+CMQNEW response
    pub topic: &'a str, // length max 128b
    pub qos: u8,        // 0 | 1 | 2
}

impl AtRequest for MQTTSubscribe<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        CommandBuilder::create_set(buffer, true)
            .named("+CMQSUB")
            .with_int_parameter(self.mqtt_id)
            .with_string_parameter(self.topic)
            .with_int_parameter(self.qos)
            .finish()
    }
}
