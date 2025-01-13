use crate::at_command::mqtt::MQTTSessionWrapper::Disconnected;
use crate::at_command::{AtRequest, AtResponse, BufferType};
use crate::{AtError, Modem};
use at_commands::builder::CommandBuilder;
#[cfg(feature = "defmt")]
use defmt::{error, info};
use embedded_io::{Read, Write};

const MAX_SERVER_LEN: usize = 50;

#[cfg_attr(feature = "defmt", derive(defmt::Format, Debug))]
pub enum MQTTError {
    ConnectionFailed,
    Disconnected,
    Publish,
}

pub struct Mqtt<'a> {
    session_settings: &'a MQTTSessionSettings<'a>,
    session_wrapper: MQTTSessionWrapper,
}

impl<'a> Mqtt<'a> {
    pub fn new(session_settings: &'a MQTTSessionSettings<'a>) -> Self {
        let session_wrapper = Disconnected(MQTTSession::new());
        Self {
            session_settings,
            session_wrapper,
        }
    }
    pub fn create_session<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<Self, MQTTError> {
        let session_wrapper = self
            .session_wrapper
            .create_session(modem, self.session_settings)?;
        Ok(Self {
            session_settings: self.session_settings,
            session_wrapper,
        })
    }

    pub fn connect<T: Write, U: Read>(
        self,
        connection_settings: MQTTConnectionSettings,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<Self, MQTTError> {
        let session_wrapper = self.session_wrapper.connect(modem, connection_settings)?;
        Ok(Self {
            session_settings: self.session_settings,
            session_wrapper,
        })
    }

    pub fn disconnect<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
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

    pub fn publish<T, U>(
        &self,
        message: &MQTTMessage,
        p1: &mut Modem<T, U>,
    ) -> Result<(), MQTTError>
    where
        T: Write,
        U: Read,
    {
        self.session_wrapper.publish(message, p1)
    }
}

enum MQTTSessionWrapper {
    Disconnected(MQTTSession<StateDisconnected>),
    Connected(MQTTSession<StateConnected>),
    ConnectedGood(MQTTSession<StateConnectedGood>),
}

impl MQTTSessionWrapper {
    fn create_session<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
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

    fn connect<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
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

    pub(crate) fn publish<T: Write, U: Read>(
        &self,
        p0: &MQTTMessage,
        p1: &mut Modem<'_, T, U>,
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

pub struct MQTTSession<S> {
    state: S,
}

struct StateDisconnected {}

struct StateConnected {
    mqtt_id: u8,
}

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

    pub fn create_session<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
        session_settings: &MQTTSessionSettings,
    ) -> Result<MQTTSession<StateConnected>, AtError> {
        #[cfg(feature = "defmt")]
        info!("Creating new session");
        match modem.send_and_wait_reply(session_settings) {
            Ok(AtResponse::MQTTSessionCreated(mqtt_id)) => Ok(MQTTSession {
                state: StateConnected { mqtt_id },
            }),
            Ok(_response) => {
                #[cfg(feature = "defmt")]
                error!("unexpected response from mqtt modem: {:?}", _response);
                Err(AtError::ErrorReply(0))
            }
            Err(e) => Err(e),
        }
    }
}

impl MQTTSession<StateConnected> {
    pub fn disconnect<T: Write, U: Read>(
        &self,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<MQTTSession<StateDisconnected>, AtError> {
        modem.send_and_wait_reply(&CloseMQTTConnection {
            mqtt_id: self.state.mqtt_id,
        })?;
        Ok(MQTTSession {
            state: StateDisconnected {},
        })
    }

    pub fn connect<T: Write, U: Read>(
        self,
        modem: &mut Modem<'_, T, U>,
        connection_settings: MQTTConnectionSettings,
    ) -> Result<MQTTSession<StateConnectedGood>, AtError> {
        let mqtt_id = self.state.mqtt_id;
        let connection_settings = connection_settings.with_mqtt_id(mqtt_id);
        match modem.send_and_wait_reply(&connection_settings) {
            Ok(response) => match response {
                AtResponse::Ok => Ok(MQTTSession {
                    state: StateConnectedGood { mqtt_id },
                }),
                _ => {
                    #[cfg(feature = "defmt")]
                    error!("unexpected response from mqtt modem: {:?}", response);
                    Err(AtError::ErrorReply(0))
                }
            },
            Err(e) => Err(e),
        }
    }
}

impl MQTTSession<StateConnectedGood> {
    fn disconnect<T: Write, U: Read>(
        &self,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<MQTTSession<StateDisconnected>, AtError> {
        modem
            .send_and_wait_reply(&CloseMQTTConnection {
                mqtt_id: self.state.mqtt_id,
            })
            .expect("TODO: panic message");
        Ok(MQTTSession {
            state: StateDisconnected {},
        })
    }

    fn publish<T: Write, U: Read>(
        &self,
        message: &MQTTMessage,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<(), MQTTError> {
        modem
            .send_and_wait_reply(&MQTTPublish {
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MQTTConnection {
    Connected(u8),
    Disconnected,
}

impl MQTTConnection {
    pub fn publish<T: Write, U: Read>(
        &self,
        message: &MQTTMessage,
        modem: &mut Modem<'_, T, U>,
    ) -> Result<(), MQTTError> {
        match self {
            MQTTConnection::Disconnected => Err(MQTTError::Disconnected),
            MQTTConnection::Connected(mqtt_id) => {
                modem
                    .send_and_wait_reply(&MQTTPublish {
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
/// Create a new MQTT connection
pub struct MQTTSessionSettings<'a> {
    pub server: &'a str,
    pub port: u16,               // 0 - 65535
    pub timeout_ms: u16,         // 0 - 60.000
    pub buffer_size: u16,        // 20 - 1132
    pub context_id: Option<u16>, // PDP context, AT+CGAT response
}

impl MQTTSessionSettings<'_> {
    pub fn new(server: &str, port: u16) -> MQTTSessionSettings {
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
        if buffer_size < 20 || buffer_size > 1132 {
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

impl AtRequest for MQTTSessionSettings<'_> {
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
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;

        Ok(AtResponse::MQTTSessionCreated(mqtt_id as u8))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format, Debug))]
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

pub struct GetMQTTSession {}

impl AtRequest for GetMQTTSession {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
        CommandBuilder::create_query(buffer, true)
            .named("+CMQNEW")
            .finish()
    }

    fn parse_response(&self, data: &[u8]) -> Result<AtResponse, AtError> {
        let (mqtt_id, used_state, server) = at_commands::parser::CommandParser::parse(data)
            .expect_identifier(b"\r\n+CMQNEW: ")
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .expect_identifier(b"\r\n\r\nOK")
            .finish()?;
        let mut server_str: [u8; MAX_SERVER_LEN] = [0; MAX_SERVER_LEN];
        let chars = server.len().min(MAX_SERVER_LEN);
        server_str[..chars].copy_from_slice(&server.as_bytes()[..chars]);
        Ok(AtResponse::MQTTSession(
            mqtt_id as u8,
            UsedState::from(used_state),
            server_str,
        ))
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CloseMQTTConnection {
    pub mqtt_id: u8,
}

impl AtRequest for CloseMQTTConnection {
    type Response = ();

    fn get_command<'a>(&'a self, buffer: &'a mut BufferType) -> Result<&'a [u8], usize> {
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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
