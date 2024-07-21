use crate::at_command::{AtRequest, AtResponse};
use crate::utils::split_u16_to_u8;
use crate::{AtError, ModemWriter};
use core::fmt::Write;
use cortex_m::prelude::_embedded_hal_serial_Write;
use defmt::export::write;
use defmt::{error, info, Format};
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

    fn send(&self, writer: &mut ModemWriter) {
        // TODO: move into new
        if self.port > 65535 {
            error!("port is out of range")
        }
        if self.timeout_ms > 60000 {
            error!("timeout is out of range")
        }
        if (self.buffer_size > 1132) | (self.buffer_size < 20) {
            error!("buffer_size is out of range")
        }

        let port = split_u16_to_u8(self.port);
        let timeout = split_u16_to_u8(self.timeout_ms);
        let buffer_size = split_u16_to_u8(self.buffer_size);
        writer.write_str("AT+CMQNEW=").unwrap();
        writer.write_str("88.198.226.54,1883,5000,600").unwrap();
        // writer.write_str(self.server).unwrap();
        // writer.write_char(',').unwrap();
        // writer.write_full_blocking(&port);
        // writer.write_char(',').unwrap();
        // writer.write_full_blocking(&timeout);
        // writer.write_char(',').unwrap();
        // writer.write_full_blocking(&buffer_size);
        // // hier muss noch die cid hin falls nicht none
        writer.write_str("\r\n").unwrap();
    }
}

#[derive(Format)]
pub struct CloseMQTTConnection {}

impl AtRequest for CloseMQTTConnection {
    type Response = ();

    fn send(&self, writer: &mut ModemWriter) {
        // todo fix hard coded client id
        writer.write_str("AT+CMQDISCON=0\r\n").unwrap();
    }
}

#[derive(Format)]
pub struct MQTTConnect {}

impl AtRequest for MQTTConnect {
    type Response = ();

    fn send(&self, writer: &mut ModemWriter) {
        //
        writer
            .write_str("AT+CMQCON=0,4,234343493,120,0,0,marius,Haufenhistory\r\n")
            .unwrap()
    }
}

#[derive(Format)]
pub struct MQTTPublish {
    // mqtt_id: u8,  // AT+CMQNEW response
    // topic: str  // length max 128b
    // qos:  // 0 | 1 | 2
    // retained: u8  // 0 | 1
    // dup: u8  // 0 | 1
    // message_len: u8  | 2 - 1000
    // message: str as hex
}

fn byte_to_hex(byte: u8) -> (char, char) {
    let hex_chars = b"0123456789abcdef";
    (
        hex_chars[(byte >> 4) as usize] as char,
        hex_chars[(byte & 0x0F) as usize] as char,
    )
}

#[derive(Format)]
pub struct MQTTRawData {}

impl AtRequest for MQTTRawData {
    type Response = ();

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CREVHEX=0\r\n").unwrap();
    }
}

impl AtRequest for MQTTPublish {
    type Response = ();

    fn send(&self, writer: &mut ModemWriter) {
        let mut buffer: [u8; 4] = [0; 4];
        hex::encode_to_slice(b"hi", &mut buffer).unwrap();

        writer.write_str("AT+CMQPUB=0,\"test\",1,0,0,4,\"").unwrap();
        writer.write_full_blocking(&buffer);
        writer.write_str("\"\r\n").unwrap();
    }
}

#[derive(Format)]
pub struct MQTTSubscribe {}

impl AtRequest for MQTTSubscribe {
    type Response = ();

    fn send(&self, writer: &mut ModemWriter) {
        writer.write_str("AT+CMQSUB=0,\"test\",1").unwrap();
        writer.write_str("\r\n").unwrap();
    }
}
