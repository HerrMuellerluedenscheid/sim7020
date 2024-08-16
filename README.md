# SIM7020 Driver for NB-IoT Communication

![Crates.io Version](https://img.shields.io/crates/v/sim7020)

This driver does not implement all available AT commands of the [SIM7020 modem](https://www.waveshare.com/pico-sim7020e-nb-iot.htm).
It's enough though, to get an NTP time stamp and send data via HTTP and MQTT.

Check the **[pico example](./examples/pico/src/main.rs)** to see e.g. how to send payloads with HTTP and MQTT.

Feel free to open an issue if you need support for specific other functionality.