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
use embassy_rp::bind_interrupts;
use embassy_time::Timer;

use sim7020::nonblocking::AsyncModem;
use sim7020::{at_command, AtError};

use embassy_rp::adc::Adc;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart};
use sim7020::at_command::cmee::ReportMobileEquipmentErrorSetting;
use sim7020::at_command::AtResponse;

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
    let mut power_pin = Output::new(p.PIN_14, Level::Low);
    let mut wake_pin = Output::new(p.PIN_17, Level::Low);
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
    info!("resetting modem");
    power_pin.set_low();
    led.set_low();
    Timer::after_millis(500).await;
    led.set_high();
    power_pin.set_high();
    Timer::after_millis(1000).await;
    info!("resetting modem. done");

    // reader und writer sind nun async. Heisst
    let mut modem = AsyncModem::new(&mut writer, &mut reader).await.unwrap();

    modem
        .verbosity(ReportMobileEquipmentErrorSetting::EnabledVerbose)
        .await
        .unwrap();

    modem
        .send_and_wait_reply(at_command::at_creg::AtCregError {})
        .await
        .expect("TODO: panic message");
    Timer::after_millis(10000).await;

    info!("Enter pin");
    modem
        .send_and_wait_reply(at_command::at_cpin::PINRequired {})
        .await
        .expect("TODO: panic message");

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
        .send_and_wait_reply(at_command::at_creg::NetworkRegistration {})
        .await
        .unwrap();

    let _ = modem
        .send_and_wait_reply(at_command::ntp::StartQueryNTP {
            ip_addr: "202.112.29.82",
        })
        .await
        .or_else(|e| {
            warn!("failed starting ntp connection. Connection already established?");
            return Err(e);
        });

    modem
        .send_and_wait_reply(at_command::ntp::NTPTime {})
        .await
        .unwrap();

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTRawData {
            data_format: at_command::mqtt::MQTTDataFormat::Bytes,
        })
        .await
        .unwrap();

    match modem
        .send_and_wait_reply(at_command::mqtt::MQTTSessionSettings::new(
            "88.198.226.54",
            1883,
        ))
        .await
    {
        Ok(AtResponse::MQTTSessionCreated(mqtt_id)) => {
            info!("connected mqtt session {}", mqtt_id);
            modem
                .send_and_wait_reply(at_command::mqtt::MQTTConnectionSettings {
                    mqtt_id,
                    version: at_command::mqtt::MQTTVersion::MQTT311,
                    client_id: "sdo92u34oij",
                    keepalive_interval: 120,
                    clean_session: false,
                    will_flag: false,
                    username: "marius",
                    password: "Haufenhistory",
                })
                .await
                .unwrap();
            Timer::after_millis(1000).await;

            modem
                .send_and_wait_reply(at_command::mqtt::MQTTPublish {
                    mqtt_id,                          // AT+CMQNEW response
                    topic: "test",                    // length max 128b
                    qos: 1,                           // 0 | 1 | 2
                    retained: false,                  // 0 | 1
                    dup: false,                       // 0 | 1
                    message: b"hello world via mqtt", // as hex
                })
                .await
                .unwrap();
            Timer::after_millis(2000).await;

            // close mqtt connection again
            modem
                .send_and_wait_reply(at_command::mqtt::CloseMQTTConnection { mqtt_id })
                .await
                .unwrap();
        }
        Err(e) => warn!("failed connecting mqtt"),
        _ => {}
    };
    Timer::after_millis(500).await;

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
        led.set_high();
        Timer::after_secs(1).await;
        led.set_low();
        Timer::after_secs(1).await;

        modem
            .send_and_wait_reply(at_command::at_cgatt::GPRSServiceStatus {})
            .await
            .unwrap();
        modem
            .send_and_wait_reply(at_command::at_csq::SignalQualityReport {})
            .await
            .unwrap();
    }
}
