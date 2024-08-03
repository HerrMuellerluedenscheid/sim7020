# SIM7020 Driver for NB-IoT Communication

Based on the [rp2040-project-template](https://github.com/rp-rs/rp2040-project-template).

This driver currently supports only a subset of the available AT commands of the [SIM7020 modem](https://www.waveshare.com/pico-sim7020e-nb-iot.htm). It's enough to get an NTP time stamp and send data via mqtt:

```rust

    modem
        .send_and_wait_reply(at_command::ntp::StartNTPConnection {}).or_else(|e| {
        warn!("failed starting ntp connection. Connection already established?" );
        return Err(e)
    });

    modem
        .send_and_wait_reply(at_command::ntp::NTPTime {})
        .unwrap();

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTRawData {})
        .unwrap();

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTRawData {
            data_format: at_command::mqtt::MQTTDataFormat::Bytes,
        })
        .unwrap();

    match modem.send_and_wait_reply(at_command::mqtt::NewMQTTConnection {
        server: "88.198.111.21",
        port: 1883,
        timeout_ms: 5000,
        buffer_size: 600,
        context_id: None,
    }) {
        Ok(_) => info!("connected mqtt"),
        Err(e) => warn!("failed connecting mqtt"),
    };

    modem
        .send_and_wait_reply(at_command::mqtt::MQTTConnect {
            mqtt_id: 0,
            version: at_command::mqtt::MQTTVersion::MQTT311,
            client_id: "0",
            keepalive_interval: 120,
            clean_session: false,
            will_flag: false,
            username: "someuser",
            password: "somepassword"
        })
        .unwrap();
```

Feel free to open an issue if you need support for specific other functionality.