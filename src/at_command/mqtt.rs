//! Model to handle the MQTT request
use crate::at_command::mqtt::MQTTSessionWrapper::Disconnected;
use crate::at_command::AtRequest;
#[allow(deprecated)]
use crate::at_command::AtResponse;
use crate::{AtError, Modem};
use at_commands::builder::CommandBuilder;
#[cfg(feature = "defmt")]
use defmt::{error, info};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, ReadReady, Write};

/// Maximum server length
const MAX_SERVER_LEN: usize = 50;

/// MQTT errors
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub enum MQTTError {
    ConnectionFailed,
    Disconnected,
    Publish,
}

/// The mqtt session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct Mqtt<'a> {
    session_settings: &'a MQTTSessionSettings<'a>,
    session_wrapper: MQTTSessionWrapper,
}

impl<'a> Mqtt<'a> {
    /// Creates a new MQTT session
    pub fn new(session_settings: &'a MQTTSessionSettings<'a>) -> Self {
        let session_wrapper = Disconnected(MQTTSession::new());
        Self {
            session_settings,
            session_wrapper,
        }
    }
    /// Creates the MQTT session
    pub fn create_session<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<Self, MQTTError> {
        let session_wrapper = self
            .session_wrapper
            .create_session(modem, self.session_settings)?;
        Ok(Self {
            session_settings: self.session_settings,
            session_wrapper,
        })
    }

    /// Connects the MQTT session
    pub fn connect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        connection_settings: MQTTConnectionSettings,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<Self, MQTTError> {
        let session_wrapper = self.session_wrapper.connect(modem, connection_settings)?;
        Ok(Self {
            session_settings: self.session_settings,
            session_wrapper,
        })
    }

    /// Disconnects the MQTT session
    pub fn disconnect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<Self, MQTTError> {
        match self.session_wrapper {
            Disconnected(_) => Err(MQTTError::Disconnected),
            MQTTSessionWrapper::Connected(session) => {
                let session = session.disconnect(modem).expect("connect failed");
                let session_wrapper = Disconnected(session);
                Ok(Self {
                    session_settings: self.session_settings,
                    session_wrapper,
                })
            }
            MQTTSessionWrapper::ConnectedGood(session) => {
                let session = session.disconnect(modem).expect("connect failed");
                let session_wrapper = Disconnected(session);
                Ok(Self {
                    session_settings: self.session_settings,
                    session_wrapper,
                })
            }
        }
    }

    /// Publish on a MQTT session
    pub fn publish<T, U, P, D>(
        &self,
        message: &MQTTMessage,
        p1: &mut Modem<T, U, P, D>,
    ) -> Result<(), MQTTError>
    where
        T: Write,
        U: Read + ReadReady,
        P: OutputPin,
        D: DelayNs,
    {
        self.session_wrapper.publish(message, p1)
    }
}

/// Wrapper around the MQTT sessions with the possible states
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
enum MQTTSessionWrapper {
    Disconnected(MQTTSession<StateDisconnected>),
    Connected(MQTTSession<StateConnected>),
    ConnectedGood(MQTTSession<StateConnectedGood>),
}

impl MQTTSessionWrapper {
    /// Create a new MQTT session
    fn create_session<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
        session_settings: &MQTTSessionSettings,
    ) -> Result<MQTTSessionWrapper, MQTTError> {
        match self {
            Disconnected(session) => match session.create_session(modem, session_settings) {
                Ok(session) => Ok(Self::Connected(session)),
                Err(_e) => {
                    #[cfg(feature = "defmt")]
                    error!("{:?}", _e);
                    Err(MQTTError::Disconnected)
                }
            },
            _ => {
                #[cfg(feature = "defmt")]
                info!("already connected");
                Ok(self)
            }
        }
    }

    /// Connects the MQTT session
    fn connect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
        connection_settings: MQTTConnectionSettings,
    ) -> Result<MQTTSessionWrapper, MQTTError> {
        match self {
            Disconnected(_) => Err(MQTTError::Disconnected),
            MQTTSessionWrapper::Connected(session) => {
                match session.connect(modem, connection_settings) {
                    Ok(session) => Ok(Self::ConnectedGood(session)),
                    Err(_e) => {
                        #[cfg(feature = "defmt")]
                        error!("{:?}", _e);
                        Err(MQTTError::Disconnected)
                    }
                }
            }
            _ => {
                #[cfg(feature = "defmt")]
                info!("already connected");
                Ok(self)
            }
        }
    }

    /// Publish on the MQTT session
    pub(crate) fn publish<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        &self,
        p0: &MQTTMessage,
        p1: &mut Modem<'_, T, U, P, D>,
    ) -> Result<(), MQTTError> {
        match self {
            Disconnected(_) => Err(MQTTError::Disconnected),
            MQTTSessionWrapper::Connected(_) => Err(MQTTError::Disconnected), // should be state where session established but not connected
            MQTTSessionWrapper::ConnectedGood(session) => {
                session.publish(p0, p1)?;
                Ok(())
            }
        }
    }
}

/// MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTSession<S> {
    state: S,
}

/// Disconnected MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
struct StateDisconnected {}

/// Connected MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
struct StateConnected {
    mqtt_id: u8,
}

/// Connected MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
struct StateConnectedGood {
    mqtt_id: u8,
}

impl Default for MQTTSession<StateDisconnected> {
    fn default() -> Self {
        Self::new()
    }
}

impl MQTTSession<StateDisconnected> {
    pub fn new() -> MQTTSession<StateDisconnected> {
        Self {
            state: StateDisconnected {},
        }
    }

    /// Creates a new MQTT session
    pub fn create_session<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
        session_settings: &MQTTSessionSettings,
    ) -> Result<MQTTSession<StateConnected>, AtError> {
        #[cfg(feature = "defmt")]
        info!("Creating new session");
        let mqtt_id = modem.send_and_wait_response(session_settings)?.mqtt_id;
        Ok(MQTTSession {
            state: StateConnected { mqtt_id },
        })
    }
}

impl MQTTSession<StateConnected> {
    /// Disconnects the MQTT session
    pub fn disconnect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        &self,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<MQTTSession<StateDisconnected>, AtError> {
        modem.send_and_wait_response(&CloseMQTTConnection {
            mqtt_id: self.state.mqtt_id,
        })?;
        Ok(MQTTSession {
            state: StateDisconnected {},
        })
    }

    /// Connects the MQTT session
    pub fn connect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        self,
        modem: &mut Modem<'_, T, U, P, D>,
        connection_settings: MQTTConnectionSettings,
    ) -> Result<MQTTSession<StateConnectedGood>, AtError> {
        let mqtt_id = self.state.mqtt_id;
        let connection_settings = connection_settings.with_mqtt_id(mqtt_id);
        modem.send_and_wait_response(&connection_settings)?;
        Ok(MQTTSession {
            state: StateConnectedGood { mqtt_id },
        })
    }
}

impl MQTTSession<StateConnectedGood> {
    /// Disconnects the MQTT session
    fn disconnect<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        &self,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<MQTTSession<StateDisconnected>, AtError> {
        modem.send_and_wait_response(&CloseMQTTConnection {
            mqtt_id: self.state.mqtt_id,
        })?;
        Ok(MQTTSession {
            state: StateDisconnected {},
        })
    }

    /// Publish on the MQTT
    fn publish<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        &self,
        message: &MQTTMessage,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<(), MQTTError> {
        modem
            .send_and_wait_response(&MQTTPublish {
                mqtt_id: self.state.mqtt_id,
                topic: message.topic,
                qos: message.qos,
                retained: message.retained,
                dup: message.dup,
                message: message.message,
            })
            .map_err(|_| MQTTError::Publish)?;
        Ok(())
    }
}

/// The MQTT connection states
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub enum MQTTConnection {
    Connected(u8),
    Disconnected,
}

impl MQTTConnection {
    pub fn publish<T: Write, U: Read + ReadReady, P: OutputPin, D: DelayNs>(
        &self,
        message: &MQTTMessage,
        modem: &mut Modem<'_, T, U, P, D>,
    ) -> Result<(), MQTTError> {
        match self {
            MQTTConnection::Disconnected => Err(MQTTError::Disconnected),
            MQTTConnection::Connected(mqtt_id) => {
                modem
                    .send_and_wait_response(&MQTTPublish {
                        mqtt_id: *mqtt_id,
                        topic: message.topic,
                        qos: message.qos,
                        retained: message.retained,
                        dup: message.dup,
                        message: message.message,
                    })
                    .map_err(|_| MQTTError::Publish)?;
                Ok(())
            }
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
/// Create a new MQTT connection
pub struct MQTTSessionSettings<'a> {
    pub server: &'a str,
    pub port: u16,               // 0 - 65535
    pub timeout_ms: u16,         // 0 - 60.000
    pub buffer_size: u16,        // 20 - 1132
    pub context_id: Option<u16>, // PDP context, AT+CGAT response
}

impl MQTTSessionSettings<'_> {
    pub fn new(server: &str, port: u16) -> MQTTSessionSettings<'_> {
        let timeout_ms = 5000;
        let buffer_size = 600;
        MQTTSessionSettings {
            server,
            port,
            timeout_ms,
            buffer_size,
            context_id: None,
        }
    }

    pub fn with_timeout_ms(mut self, timeout_ms: u16) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_buffer_size(mut self, buffer_size: u16) -> Self {
        if !(20..=1132).contains(&buffer_size) {
            panic!("buffer size must be between 20 and 1132");
        }
        self.buffer_size = buffer_size;
        self
    }

    pub fn with_context_id(mut self, context_id: Option<u16>) -> Self {
        self.context_id = context_id;
        self
    }
}

/// The MQTT session id
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MqttSessionId {
    pub mqtt_id: u8,
}

impl MQTTSessionSettings<'_> {
    fn get_session_id(data: &[u8]) -> Result<u8, AtError> {
        let (mqtt_id,) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CMQNEW: ")
            .expect_int_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        Ok(mqtt_id as u8)
    }
}

impl AtRequest for MQTTSessionSettings<'_> {
    type Response = MqttSessionId;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        CommandBuilder::create_set(buffer, true)
            .named("+CMQNEW")
            .with_string_parameter(self.server)
            .with_int_parameter(self.port)
            .with_int_parameter(self.timeout_ms)
            .with_int_parameter(self.buffer_size)
            // .with_optional_int_parameter(self.context_id)
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let mqtt_id = Self::get_session_id(data)?;
        Ok(AtResponse::MQTTSessionCreated(mqtt_id))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let mqtt_id = Self::get_session_id(data)?;
        Ok(MqttSessionId { mqtt_id })
    }
}

/// The used state
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone, Debug)]
pub enum UsedState {
    NotUsed,
    Used,
}

impl From<i32> for UsedState {
    fn from(value: i32) -> Self {
        match value {
            0 => UsedState::NotUsed,
            1 => UsedState::Used,
            _ => unreachable!(),
        }
    }
}

/// Command to get the MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GetMQTTSession;

/// Response with the MQTT session information
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct GetMQTTSessionResponse {
    pub mqtt_id: u8,
    pub used_state: UsedState,
    pub server: heapless::String<MAX_SERVER_LEN>,
}

impl GetMQTTSession {
    fn get_data(data: &[u8]) -> Result<(i32, i32, &str), AtError> {
        let tuple = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CMQNEW: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_string_parameter()
            .trim_whitespace()
            .expect_identifier(b"OK")
            .finish()?;

        Ok(tuple)
    }
}

impl AtRequest for GetMQTTSession {
    type Response = GetMQTTSessionResponse;

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        CommandBuilder::create_query(buffer, true)
            .named("+CMQNEW")
            .finish()
    }

    #[allow(deprecated)]
    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (mqtt_id, used_state, server) = Self::get_data(data)?;
        let mut server_str: [u8; MAX_SERVER_LEN] = [0; MAX_SERVER_LEN];
        let chars = server.len().min(MAX_SERVER_LEN);
        server_str[..chars].copy_from_slice(&server.as_bytes()[..chars]);
        Ok(AtResponse::MQTTSession(
            mqtt_id as u8,
            UsedState::from(used_state),
            server_str,
        ))
    }

    fn parse_response_struct(&self, data: &[u8]) -> Result<Self::Response, AtError> {
        let (mqtt_id, used_state, server) = Self::get_data(data)?;
        let server: heapless::String<MAX_SERVER_LEN> = server.try_into()?;
        let used_state: UsedState = used_state.into();
        Ok(GetMQTTSessionResponse {
            mqtt_id: mqtt_id as u8,
            used_state,
            server,
        })
    }
}

/// Request to close the MQTT session
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct CloseMQTTConnection {
    pub mqtt_id: u8,
}

impl AtRequest for CloseMQTTConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CMQDISCON")
            .with_int_parameter(self.mqtt_id)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// MQTT versions
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
#[repr(u8)]
pub enum MQTTVersion {
    MQTT31,
    MQTT311,
}

/// Options for MQTT will
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct WillOptions<'a> {
    pub topic: &'a str,
    pub quality_of_service: u8,
    pub retained: bool,
}

/// Command to connect to MQTT with different options
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
struct MQTTConnectionSettingsWithID<'a> {
    mqtt_id: u8,
    version: MQTTVersion,
    client_id: &'a str,
    keepalive_interval: u16, // 0 - 64800
    clean_session: bool,
    will_flag: bool,
    // pub will_options: Option<WillOptions>,
    username: &'a str,
    password: &'a str,
}

/// Command to connect to MQTT with different options
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTConnectionSettings<'a> {
    pub version: MQTTVersion,
    pub client_id: &'a str,
    pub keepalive_interval: u16, // 0 - 64800
    pub clean_session: bool,
    pub will_flag: bool,
    // pub will_options: Option<WillOptions>,
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> MQTTConnectionSettings<'a> {
    fn with_mqtt_id(self, mqtt_id: u8) -> MQTTConnectionSettingsWithID<'a> {
        MQTTConnectionSettingsWithID {
            mqtt_id,
            version: self.version,
            client_id: self.client_id,
            keepalive_interval: self.keepalive_interval,
            clean_session: self.clean_session,
            will_flag: self.will_flag,
            username: self.username,
            password: self.password,
        }
    }
}

impl AtRequest for MQTTConnectionSettingsWithID<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// MQTT format of the data
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub enum MQTTDataFormat {
    Bytes,
    Hex,
}

/// Request to set MQTT data format
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTRawData {
    pub data_format: MQTTDataFormat,
}

impl AtRequest for MQTTRawData {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        let format = match self.data_format {
            MQTTDataFormat::Bytes => 0,
            MQTTDataFormat::Hex => 1,
        };

        at_commands::builder::CommandBuilder::create_set(buffer, true)
            .named("+CREVHEX")
            .with_int_parameter(format)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

/// Publish a message via mqtt
///
/// The message length has to be between 2 and 1000 byte.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTMessage<'a> {
    pub topic: &'a str,    // length max 128b
    pub qos: u8,           // 0 | 1 | 2
    pub retained: bool,    // 0 | 1
    pub dup: bool,         // 0 | 1
    pub message: &'a [u8], // as hex
}

/// Publish a message via mqtt
///
/// The message length has to be between 2 and 1000 byte.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTPublish<'a> {
    pub mqtt_id: u8,       // AT+CMQNEW response
    pub topic: &'a str,    // length max 128b
    pub qos: u8,           // 0 | 1 | 2
    pub retained: bool,    // 0 | 1
    pub dup: bool,         // 0 | 1
    pub message: &'a [u8], // as hex
}

impl AtRequest for MQTTPublish<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
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

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq, Clone)]
pub struct MQTTSubscribe<'a> {
    pub mqtt_id: u8,    // AT+CMQNEW response
    pub topic: &'a str, // length max 128b
    pub qos: u8,        // 0 | 1 | 2
}

impl AtRequest for MQTTSubscribe<'_> {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut [u8]) -> Result<&'a [u8], usize> {
        CommandBuilder::create_set(buffer, true)
            .named("+CMQSUB")
            .with_int_parameter(self.mqtt_id)
            .with_string_parameter(self.topic)
            .with_int_parameter(self.qos)
            .finish()
    }

    fn parse_response_struct(&self, _data: &[u8]) -> Result<Self::Response, AtError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_SERVER: &str = "mqtt.example.com";

    #[test]
    fn mqtt_session_settings_new_defaults() {
        let settings = MQTTSessionSettings::new(TEST_SERVER, 1883);

        assert_eq!(settings.server, TEST_SERVER);
        assert_eq!(settings.port, 1883);
        assert_eq!(settings.timeout_ms, 5000);
        assert_eq!(settings.buffer_size, 600);
        assert!(settings.context_id.is_none());
    }

    #[test]
    fn mqtt_session_settings_with_timeout_and_buffer() {
        let settings = MQTTSessionSettings::new(TEST_SERVER, 1883)
            .with_timeout_ms(10000)
            .with_buffer_size(1024);

        assert_eq!(settings.timeout_ms, 10000);
        assert_eq!(settings.buffer_size, 1024);
    }

    #[test]
    fn mqtt_session_settings_with_context() {
        let settings = MQTTSessionSettings::new(TEST_SERVER, 1883).with_context_id(Some(1));

        assert_eq!(settings.context_id, Some(1));
    }

    #[test]
    fn mqtt_session_settings_get_command() {
        let settings = MQTTSessionSettings::new(TEST_SERVER, 1883);
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = settings.get_command(&mut buffer).unwrap();

        assert!(cmd.starts_with(b"AT+CMQNEW"));
        assert!(cmd
            .windows(TEST_SERVER.len())
            .any(|w| w == TEST_SERVER.as_bytes()));
    }

    #[test]
    fn mqtt_session_settings_parse_session_id_success() {
        let data = b"+CMQNEW: 3\r\nOK";
        let id = MQTTSessionSettings::get_session_id(data).unwrap();
        assert_eq!(id, 3);
    }

    #[test]
    fn mqtt_session_settings_parse_session_id_failure() {
        let data = b"+CMQNEW: \r\nOK";
        assert!(MQTTSessionSettings::get_session_id(data).is_err());
    }

    #[test]
    fn get_mqtt_session_parse_response_struct() {
        const MAX_SERVER_LEN: usize = 32;
        let data = b"+CMQNEW: 1,0,\"mqtt.example.com\"\r\nOK";

        let response = GetMQTTSession::parse_response_struct(&GetMQTTSession, data).unwrap();

        assert_eq!(response.mqtt_id, 1);
        assert_eq!(response.used_state, UsedState::NotUsed);
        assert_eq!(response.server.as_str(), "mqtt.example.com");
    }

    #[test]
    fn close_mqtt_connection_get_command() {
        let close = CloseMQTTConnection { mqtt_id: 5 };
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = close.get_command(&mut buffer).unwrap();

        assert!(cmd.starts_with(b"AT+CMQDISCON=5"));
    }

    #[test]
    fn mqtt_connection_settings_with_id_get_command() {
        let base = MQTTConnectionSettings {
            version: MQTTVersion::MQTT311,
            client_id: "client",
            keepalive_interval: 60,
            clean_session: true,
            will_flag: false,
            username: "user",
            password: "pass",
        };
        let settings = base.with_mqtt_id(2);
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = settings.get_command(&mut buffer).unwrap();

        assert!(cmd.windows(b"+CMQCON".len()).any(|w| w == b"+CMQCON"));
        assert!(cmd.windows(b"client".len()).any(|w| w == b"client"));
        assert!(cmd.windows(b"user".len()).any(|w| w == b"user"));
    }

    #[test]
    fn mqtt_raw_data_bytes() {
        let raw = MQTTRawData {
            data_format: MQTTDataFormat::Bytes,
        };
        let mut buffer: [u8; 512] = [0; 512];
        let _cmd = raw.get_command(&mut buffer).unwrap();
    }

    #[test]
    fn mqtt_raw_data_hex() {
        let raw = MQTTRawData {
            data_format: MQTTDataFormat::Hex,
        };
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = raw.get_command(&mut buffer).unwrap();
        assert!(cmd
            .windows(b"AT+CREVHEX=1".len())
            .any(|w| w == b"AT+CREVHEX=1"));
    }

    #[test]
    fn mqtt_publish_get_command() {
        let msg = MQTTPublish {
            mqtt_id: 1,
            topic: "topic",
            qos: 1,
            retained: true,
            dup: false,
            message: b"hello",
        };
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = msg.get_command(&mut buffer).unwrap();

        assert!(cmd.windows(b"+CMQPUB".len()).any(|w| w == b"+CMQPUB"));
        assert!(cmd.windows(b"topic".len()).any(|w| w == b"topic"));
    }

    #[test]
    fn mqtt_subscribe_get_command() {
        let sub = MQTTSubscribe {
            mqtt_id: 1,
            topic: "topic",
            qos: 0,
        };
        let mut buffer: [u8; 512] = [0; 512];
        let cmd = sub.get_command(&mut buffer).unwrap();

        assert!(cmd.windows(b"+CMQSUB".len()).any(|w| w == b"+CMQSUB"));
        assert!(cmd.windows(b"topic".len()).any(|w| w == b"topic"));
    }
}
