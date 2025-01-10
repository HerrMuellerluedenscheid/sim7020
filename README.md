# SIM7020 Driver for NB-IoT Communication 🦀

![https://crates.io/crates/sim7020](https://img.shields.io/crates/v/sim7020)

This driver does not implement all available AT commands of the [SIM7020 modem](https://www.waveshare.com/pico-sim7020e-nb-iot.htm).
It's enough though, to check the connectivity, get **time stamps** and send data via **HTTP** and **MQTT**.

Check the **[pico example](./examples/pico/src/main.rs)** for a basic example. I used the raspberrypi Pico (rp2040)
along with the header board from [Waveshare](https://www.waveshare.com/wiki/Pico-SIM7020E-NB-IoT) and a sim card from
[1nce](https://1nce.com/).

## Async Support

Enable async support through the **non-blocking** feature flag. This is WIP. Checkout the [embassy pico example](./examples/pico-embassy/src/main.rs).

Feel free to open an issue if you need support for specific other functionality.