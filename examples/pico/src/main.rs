//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.

#![no_std]
#![no_main]

use defmt_rtt as _;

use sim7020::at_command;
use sim7020::Modem;

use bsp::entry;
use core::fmt::Debug;
use defmt::*;
use embedded_hal::digital::OutputPin;
use panic_probe as _;

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
use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_serial_Read};
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1};
use rp_pico::hal::gpio::{Pin, PullDown};
use rp_pico::hal::uart::{Reader, Writer};
use rp_pico::pac::UART0;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
const BUFFER_SIZE: usize = 128;

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

    // Set up UART on GP0 and GP1 (Pico pins 1 and 2)
    let pins_uart = (
        pins.gpio0.into_function::<FunctionUart>(),
        pins.gpio1.into_function::<FunctionUart>(),
    );

    // Need to perform clock init before using UART or it will freeze.
    let uart = UartPeripheral::new(pac.UART0, pins_uart, &mut pac.RESETS)
        .enable(
            // UartConfig::new(9600.Hz(), DataBits::Eight, Option::from(Parity::Odd), StopBits::Two),
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    let (mut reader, mut writer) = uart.split();

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    //
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead.
    // One way to do that is by using [embassy](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_blinky.rs)
    //
    // If you have a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here. Don't forget adding an appropriate resistor
    // in series with the LED.
    info!("send");
    let mut index = 0;
    let mut led_pin = pins.led.into_push_pull_output();

    // pico-sim7020E-NB-IOT specific
    // GP14 -> PWR: pull down to shutdown
    // GP17 -> DTR: wake up module

    let mut modem = Modem {
        writer: &mut writer,
        reader: &mut reader,
    };

    modem
        .send_and_wait_reply(at_command::at_cpin::PINRequired {})
        .expect("TODO: panic message");
    modem
        .send_and_wait_reply(at_command::ate::AtEcho {
            status: at_command::ate::Echo::Disable,
        })
        .unwrap();

    modem
        .send_and_wait_reply(at_command::model_identification::ModelIdentification {})
        .unwrap();

    // todo: this blocks completely
    // modem.send_and_wait_reply(at_command::network_information::NetworkInformationAvailable{}).unwrap();

    modem
        .send_and_wait_reply(at_command::at_creg::AtCreg {})
        .unwrap();

    modem
        .send_and_wait_reply(at_command::cgcontrdp::PDPContextReadDynamicsParameters {})
        .unwrap();

    modem
        .send_and_wait_reply(at_command::ntp::StartNTPConnection {
            ip_addr: "202.112.29.82",
        })
        .or_else(|e| {
            warn!("failed starting ntp connection. Connection already established?");
            return Err(e);
        });

    modem
        .send_and_wait_reply(at_command::ntp::NTPTime {})
        .unwrap();
    modem
        .send_and_wait_reply(at_command::mqtt::CloseMQTTConnection {})
        .or_else(|e| {
            warn!("failed closing mqtt connection");
            return Err(e);
        });

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTRawData {
            data_format: at_command::mqtt::MQTTDataFormat::Bytes,
        })
        .unwrap();

    match modem.send_and_wait_reply(at_command::mqtt::NewMQTTConnection {
        server: "88.198.226.54",
        port: 1883,
        timeout_ms: 5000,
        buffer_size: 600,
        context_id: None,
    }) {
        Ok(_) => info!("connected mqtt"),
        Err(e) => warn!("failed connecting mqtt"),
    };
    delay.delay_ms(500);

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTConnect {
            mqtt_id: 0,
            version: at_command::mqtt::MQTTVersion::MQTT311,
            client_id: "sdo92u34oij",
            keepalive_interval: 120,
            clean_session: false,
            will_flag: false,
            username: "marius",
            password: "Haufenhistory",
        })
        .unwrap();
    delay.delay_ms(1000);

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTPublish {
            mqtt_id: 0,                      // AT+CMQNEW response
            topic: "test",                   // length max 128b
            qos: 1,                          // 0 | 1 | 2
            retained: false,                 // 0 | 1
            dup: false,                      // 0 | 1
            message: "hello world via mqtt", // as hex
        })
        .unwrap();
    delay.delay_ms(2000);
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
        modem
            .send_and_wait_reply(at_command::at_cgatt::GPRSServiceStatus {})
            .unwrap();
        modem
            .send_and_wait_reply(at_command::at_csq::SignalQualityReport {})
            .unwrap();
        delay.delay_ms(5000);
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
