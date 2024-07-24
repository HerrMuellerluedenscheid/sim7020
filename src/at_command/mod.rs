pub mod at;
pub mod at_cgatt;
pub mod at_cgmm;
pub mod at_cpin;
pub mod at_creg;
pub mod at_csq;
pub mod at_cstt;
pub mod ate;
pub mod ati;
pub mod cgcontrdp;
pub mod mqtt;
pub mod network_information;
pub mod ntp;

pub trait AtResponse {
    fn parse(response: &str) {}
}

pub trait AtRequest {
    type Response;

    fn send<T: embedded_io::Write>(&self, writer: &mut T);
}
