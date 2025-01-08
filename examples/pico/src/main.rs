//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.

#![no_std]
#![no_main]

use defmt_rtt as _;

use bsp::entry;
use core::fmt::Debug;
use defmt::*;
use embedded_hal::digital::OutputPin;
use panic_probe as _;
use sim7020::at_command::http::HttpMethod::GET;
use sim7020::at_command::AtResponse;
use sim7020::AtError;
use sim7020::Modem;
use sim7020::Write;
use sim7020::{at_command, Read};

use fugit::RateExtU32;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionUart, Pins},
    pac,
    sio::Sio,
    uart::{self, DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Clock,
};
use cortex_m::asm::delay;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m::prelude::_embedded_hal_blocking_i2c_Read;

use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{Pin, PullDown};
// use rp_pico::hal::uart::{ReadErrorType, Reader, Writer};
use rp_pico::pac::UART0;
use sim7020::at_command::mqtt::{
    MQTTConnectionSettings, MQTTError, MQTTSessionSettings, MQTTVersion, Mqtt,
};
use sim7020::at_command::network_information::NetworkMode;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("resetting modem");
    let mut power_pin = pins.gpio14.into_push_pull_output();
    let mut wake_pin = pins.gpio17.into_push_pull_output();
    let mut led_pin = pins.led.into_push_pull_output();
    power_pin.set_low().unwrap();
    led_pin.set_low().unwrap();
    delay.delay_ms(500);
    led_pin.set_high().unwrap();
    power_pin.set_high().unwrap();
    delay.delay_ms(3000);
    info!("resetting modem... done");

    // Set up UART on GP0 and GP1 (Pico pins 1 and 2)
    let pins_uart = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    // Need to perform clock init before using UART or it will freeze.
    let uart = UartPeripheral::new(pac.UART0, pins_uart, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Seven, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let (mut reader, mut writer) = uart.split();

    let mut modem = Modem::new(&mut writer, &mut reader).unwrap();

    modem.set_flow_control().unwrap();
    modem.enable_numeric_errors().unwrap();
    'outer: loop {
        info!("waiting for operator");
        let network_information = modem
            .send_and_wait_reply(&at_command::network_information::NetworkInformation {})
            .unwrap();
        match network_information {
            AtResponse::NetworkInformationState(_, _, operator) => {
                if let Some(operator) = operator {
                    info!("operator available: {:?}", operator);
                    break 'outer;
                }
            }
            _ => continue,
        }
        delay.delay_ms(1000);
    }

    delay.delay_ms(2000);

    modem
        .send_and_wait_reply(&at_command::at_cpin::PINRequired {})
        .expect("TODO: panic message");

    let response = modem
        .send_and_wait_reply(&at_command::model_identification::ModelIdentification {})
        .unwrap();
    info!("model id: {}", response);

    modem
        .send_and_wait_reply(&at_command::at_creg::AtCreg {})
        .unwrap();

    let response = modem
        .send_and_wait_reply(&at_command::cgcontrdp::PDPContextReadDynamicsParameters {})
        .unwrap();

    info!("response: {}", response);

    let _ = modem
        .send_and_wait_reply(&at_command::ntp::StartQueryNTP {
            ip_addr: "202.112.29.82",
        })
        .or_else(|e| {
            warn!("failed starting ntp connection. Connection already established?");
            return Err(e);
        });
    delay.delay_ms(2000);
    let _ = modem
        .send_and_wait_reply(&at_command::ntp::StopQueryNTP {})
        .or_else(|e| {
            warn!("failed stopping ntp connection. Connection already established?");
            return Err(e);
        });

    modem
        .send_and_wait_reply(&at_command::ntp::NTPTime {})
        .unwrap();

    // if let Err(e) = test_http_connection(&mut modem) {
    //     error!("http test failed");
    // }
    delay.delay_ms(2000);

    modem
        .send_and_wait_reply(&at_command::network_information::NetworkInformation {})
        .unwrap();

    test_mqtt_connection(&mut modem, &mut delay).unwrap();
    delay.delay_ms(2000);

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

    info!("receive loop");

    loop {
        let gprs_status = modem
            .send_and_wait_reply(&at_command::at_cgatt::GPRSServiceStatus {})
            .unwrap();
        info!("gprs status: {}", gprs_status);
        let signal_quality = modem
            .send_and_wait_reply(&at_command::at_csq::SignalQualityReport {})
            .unwrap();
        info!("signal quality: {}", signal_quality);
        delay.delay_ms(5000);
    }
}

fn test_mqtt_connection<T, U>(
    mut modem: &mut Modem<T, U>,
    delay: &mut cortex_m::delay::Delay,
) -> Result<(), AtError>
where
    T: Write,
    U: Read,
{
    let connection = MQTTSessionSettings::new("88.198.226.54", 1883);

    loop {
        delay.delay_ms(1000);
        info!("creating mqtt session.");

        let mqtt = Mqtt::new(&connection);
        let mqtt_session = mqtt
            .create_session(&mut modem)
            .map_err(|_| AtError::MqttFailure);
        if let Err(e) = mqtt_session {
            info!("mqtt session creation failed: {:?}. Retry", e);
            continue;
        }
        let mqtt_session = mqtt_session.unwrap();
        delay.delay_ms(1000);
        let mqtt_connection = mqtt_session
            .connect(
                &MQTTConnectionSettings {
                    mqtt_id: 0, // mqtt_id should be taken from session!
                    version: MQTTVersion::MQTT31,
                    client_id: "nbiot",
                    keepalive_interval: 0,
                    clean_session: false,
                    will_flag: false,
                    username: "marius",
                    password: "Haufenhistory",
                },
                &mut modem,
            )
            .unwrap();
        delay.delay_ms(1000);

        match mqtt_connection.publish(
            &at_command::mqtt::MQTTMessage {
                topic: "test",                    // length max 128b
                qos: 1,                           // 0 | 1 | 2
                retained: false,                  // 0 | 1
                dup: false,                       // 0 | 1
                message: b"hello world via mqtt", // as hex
            },
            &mut modem,
        ) {
            Ok(..) => {
                info!("sent message")
            }
            Err(e) => {
                warn!("failed to sent message: {:?}", e);
                continue;
            }
        };
        delay.delay_ms(1000);
        let mqtt = mqtt_connection.disconnect(&mut modem).unwrap();
        delay.delay_ms(10000);
    }

    // connected_mqtt.
    // let connected = connection.connect(modem)?;
    // connected.disconnect();

    // modem
    //     .send_and_wait_reply(&at_command::mqtt::CloseMQTTConnection { mqtt_id: 0 })
    //     .or_else(|e| {
    //         warn!("failed closing mqtt connection");
    //         return Err(e);
    //     }).expect("TODO: panic message");
    //
    // modem
    //     .send_and_wait_reply(at_command::mqtt::MQTTRawData {
    //         data_format: at_command::mqtt::MQTTDataFormat::Bytes,
    //     })
    //     .unwrap();
    //
    // if let AtResponse::MQTTSessionCreated(mqtt_id) =
    //     modem.send_and_wait_reply(at_command::mqtt::MQTTConnectionSettings {
    //         server: "88.198.226.54",
    //         port: 1883,
    //         timeout_ms: 5000,
    //         buffer_size: 600,
    //         context_id: None,
    //     })?
    // {
    //     modem.send_and_wait_reply(at_command::mqtt::MQTTConnect {
    //         mqtt_id,
    //         version: at_command::mqtt::MQTTVersion::MQTT311,
    //         client_id: "sdo92u34oij",
    //         keepalive_interval: 120,
    //         clean_session: false,
    //         will_flag: false,
    //         username: "marius",
    //         password: "Haufenhistory",
    //     })?;
    //     delay.delay_ms(500);
    //     modem.send_and_wait_reply(at_command::mqtt::MQTTPublish {
    //         mqtt_id,
    //         topic: "test",                    // length max 128b
    //         qos: 1,                           // 0 | 1 | 2
    //         retained: false,                  // 0 | 1
    //         dup: false,                       // 0 | 1
    //         message: b"hello world via mqtt", // as hex
    //     })?;
    //     modem
    //         .send_and_wait_reply(at_command::mqtt::CloseMQTTConnection { mqtt_id })
    //         .unwrap();
    // }

    Ok(())
}

fn test_http_connection<T, U>(modem: &mut Modem<T, U>) -> Result<(), AtError>
where
    T: Write,
    U: Read,
{
    // To test this you can start a server e.g. using python with `python3 -m http.server 8000`
    // if this errors, most likely the session count is exhausted (max 4)
    let _ = modem.send_and_wait_reply(&at_command::http::GetHttpSessions {})?;

    let result = modem.send_and_wait_reply(&at_command::http::CreateHttpSession {
        host: "http://88.198.226.54:8000",
        user: None,
        password: None,
    })?;

    info!("created http session: {}", result);
    if let AtResponse::HTTPSessionCreated(client_id) = result {
        // if this errors, most likely the server did not respond
        info!("connecting:");
        modem.send_and_wait_reply(&at_command::http::HttpConnect { client_id })?;
        info!("sending:");

        modem.send_and_wait_reply(&at_command::http::HttpSend {
            client_id,
            method: GET,
            path: "/hello/world",
        })?;

        let _ = modem.send_and_wait_reply(&at_command::http::GetHttpSessions {})?;

        modem.send_and_wait_reply(&at_command::http::HttpDisconnect { client_id })?;

        modem.send_and_wait_reply(&at_command::http::HttpDestroy { client_id })?;
    }
    Ok(())
}
