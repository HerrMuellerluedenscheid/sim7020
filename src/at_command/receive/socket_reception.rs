use crate::at_command::receive::UnsolicitedMessage;
use crate::AtError;

/// Max size that can be received from the socket.
const MAX_SOCKET_RECEPTION_SIZE: usize = 512;

/// Contains the data received from a socket
pub struct SocketDataReception {
    pub socket_id: u8,
    pub data: heapless::Vec<u8, MAX_SOCKET_RECEPTION_SIZE>,
}

/// Transforms the given hex byte into a nibble
fn hex_to_nibble(x: u8) -> u8 {
    debug_assert!(
        x.is_ascii_digit() || (b'a'..=b'f').contains(&x) || (b'A'..=b'F').contains(&x),
        "The given byte is not a valid hexadecimal number."
    );
    match x {
        b'0'..=b'9' => x - b'0',
        b'a'..=b'f' => x - b'a' + 10,
        b'A'..=b'F' => x - b'A' + 10,
        _ => unreachable!(),
    }
}

impl UnsolicitedMessage for SocketDataReception {
    fn decode(data: &[u8]) -> Result<Self, AtError> {
        let (socket_id, _, data) = at_commands::parser::CommandParser::parse(data)
            .trim_whitespace()
            .expect_identifier(b"+CSONMI: ")
            .trim_whitespace()
            .expect_int_parameter()
            .expect_int_parameter()
            .expect_raw_string()
            .finish()?;

        debug_assert!(data.len() <= MAX_SOCKET_RECEPTION_SIZE);

        let data = data.as_bytes().chunks(2).map(|x| {
            let left_nibble = hex_to_nibble(x[0]);
            let right_nibble = hex_to_nibble(x[1]);
            left_nibble << 4 | right_nibble
        });

        let result = Self {
            socket_id: socket_id as u8,
            data: heapless::Vec::from_iter(data),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode_socket_data_reception() {
        let data = b"+CSONMI: 1,32,0123456789abcdef";

        let socket_reception = SocketDataReception::decode(&data[..]).unwrap();

        assert_eq!(socket_reception.socket_id, 1);
        assert_eq!(
            socket_reception.data,
            &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
        );
    }
}
