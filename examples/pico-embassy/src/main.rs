//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]
use defmt_rtt as _;

use core::fmt::Debug;
use defmt::*;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, gpio};
use embassy_time::Timer;
use gpio::{Level, Output};

use sim7020::at_command;
use sim7020::nonblocking::Modem;

use embassy_rp::adc::Adc;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart};

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
const BUFFER_SIZE: usize = 128;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    info!("Program start");
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let adc = Adc::new_blocking(p.ADC, Default::default());

    let mut interrupt_pin = Input::new(p.PIN_26, Pull::Down);

    let mut tx_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let mut rx_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let nbiot_rx = p.PIN_1;
    let nbiot_tx = p.PIN_0;
    let uart = BufferedUart::new(
        p.UART0,
        Irqs,
        nbiot_tx,
        nbiot_rx,
        &mut tx_buffer,
        &mut rx_buffer,
        Default::default(),
    );

    let (mut reader, mut writer) = uart.split();

    // reader und writer sind nun async. Heisst
    let mut modem = Modem {
        writer: &mut writer,
        reader: &mut reader,
    };

    modem
        .send_and_wait_reply(at_command::at_cpin::PINRequired {})
        .await
        .expect("TODO: panic message");
    // modem
    //     .send_and_wait_reply(at_command::ate::AtEcho {
    //         status: at_command::ate::Echo::Disable,
    //     })
    //     .await
    //     .unwrap();
    //
    // // modem.send_and_wait_reply(at_command::at::At {}).unwrap();
    modem
        .send_and_wait_reply(at_command::model_identification::ModelIdentification {})
        .await
        .unwrap();
    //
    // // this blocks completely
    // // modem.send_and_wait_reply(at_command::network_information::NetworkInformationAvailable{}).unwrap();
    //
    modem
        .send_and_wait_reply(at_command::at_creg::AtCreg {})
        .await
        .unwrap();

    // modem
    //     .send_and_wait_reply(at_command::cgcontrdp::PDPContextReadDynamicsParameters {})
    //     .unwrap();
    //
    // modem
    //     .send_and_wait_reply(at_command::ntp::StartNTPConnection {
    //         ip_addr: "202.112.29.82"
    //     })
    //     .or_else(|e| {
    //         warn!("failed starting ntp connection. Connection already established?");
    //         return Err(e);
    //     });
    //
    // modem
    //     .send_and_wait_reply(at_command::ntp::NTPTime {})
    //     .unwrap();
    // modem
    //     .send_and_wait_reply(at_command::mqtt::CloseMQTTConnection {})
    //     .or_else(|e| {
    //         warn!("failed closing mqtt connection");
    //         return Err(e);
    //     });
    //
    // modem
    //     .send_and_wait_reply(at_command::mqtt::MQTTRawData {
    //         data_format: at_command::mqtt::MQTTDataFormat::Bytes,
    //     })
    //     .unwrap();
    //
    // match modem.send_and_wait_reply(at_command::mqtt::NewMQTTConnection {
    //     server: "88.198.226.54",
    //     port: 1883,
    //     timeout_ms: 5000,
    //     buffer_size: 600,
    //     context_id: None,
    // }) {
    //     Ok(_) => info!("connected mqtt"),
    //     Err(e) => warn!("failed connecting mqtt"),
    // };
    // Timer::after_millis(500).await;
    //
    // modem
    //     .send_and_wait_reply(at_command::mqtt::MQTTConnect {
    //         mqtt_id: 0,
    //         version: at_command::mqtt::MQTTVersion::MQTT311,
    //         client_id: "sdo92u34oij",
    //         keepalive_interval: 120,
    //         clean_session: false,
    //         will_flag: false,
    //         username: "marius",
    //         password: "Haufenhistory",
    //     })
    //     .unwrap();
    // Timer::after_millis(1000).await;
    //
    // modem
    //     .send_and_wait_reply(at_command::mqtt::MQTTPublish {
    //         mqtt_id: 0,                      // AT+CMQNEW response
    //         topic: "test",                   // length max 128b
    //         qos: 1,                          // 0 | 1 | 2
    //         retained: false,                 // 0 | 1
    //         dup: false,                      // 0 | 1
    //         message: "hello world via mqtt", // as hex
    //     })
    //     .unwrap();
    // Timer::after_millis(2000).await;
    //
    // // close mqtt connection again
    // modem
    //     .send_and_wait_reply(at_command::mqtt::CloseMQTTConnection {})
    //     .unwrap();

    // Setting the APN fails:
    // match modem.send_and_wait_reply(at_command::at_cstt::SetAPNUserPassword::new().with_apn("iot.1nce.net")){
    //     Ok(result) => info!("set apn"),
    //     Err(e) => error!("failed setting apn"),
    // }
    //
    // modem.send_and_wait_reply(at_command::at_cstt::GetAPNUserPassword{})
    //     .unwrap();

    // // at_command::at::At::send(&writer);
    // // at_command::at_creg::AtCreg::send(&writer);
    //
    // // writer.write_full_blocking(b"ATE0\r\n");
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    info!("receive loop");

    loop {
        // info!("led on!");
        // led.set_high();
        // Timer::after_secs(1).await;
        //
        // info!("led off!");
        // led.set_low();
        // Timer::after_secs(1).await;
        //
        // modem
        //     .send_and_wait_reply(at_command::at_cgatt::GPRSServiceStatus {})
        //     .unwrap();
        // modem
        //     .send_and_wait_reply(at_command::at_csq::SignalQualityReport {})
        //     .unwrap();
        // Timer::after_millis(3000).await;
        // match modem.reader.read() {
        //     Ok(13) => {
        //         index = 0;
        //         buffer = [0; BUFFER_SIZE];
        //     }
        //     Ok(byte) => {
        //         buffer[index] = byte;
        //         index += 1;
        //         if index == BUFFER_SIZE - 1 {
        //             error!("BUFFER_SIZE was not large enough")
        //         }
        //     }
        //     Err(e) => {
        //         // error!("no data")
        //     }
        // }
    }
}

// End of file
