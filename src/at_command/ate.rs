use crate::at_command::AtRequest;
use crate::AtError;
use defmt::Format;
use embedded_io::Write;

#[derive(Format, Clone, Copy)]
#[repr(u8)]
pub enum Echo {
    Disable = 0,
    Enable = 1,
}

#[derive(Format, Clone, Copy)]
pub struct AtEcho {
    pub status: Echo,
}

impl AtRequest for AtEcho {
    type Response = Result<(), AtError>;

    fn send<T: Write>(&self, writer: &mut T) {
        let status = self.status as u8;
        writer.write("ATE".as_bytes()).unwrap();
        writer.write(&[status]).unwrap();
        writer.write("\r\n".as_bytes()).unwrap();
    }
}
