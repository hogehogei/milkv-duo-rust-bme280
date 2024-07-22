#![feature(error_generic_member_access)]

use i2cdev::core::I2CDevice;
use i2cdev::linux::*;
use anyhow::{Result, Context};

use std::{env, thread, fs, time::Duration};
use chrono::{DateTime, Local, SecondsFormat};
use rumqttc::{Client, MqttOptions, TlsConfiguration, Transport, QoS};
use serde::{Serialize, Deserialize};

mod bme280;
use crate::bme280::BME280;

const BME280_SLAVE_ADDR: u16 = 0x76;

#[derive(Serialize, Deserialize, Debug)]
struct Payload<'a> {
    timestamp: &'a str,
    temperature: f32
}

fn create_i2cdev() -> Result<impl I2CDevice>
{
    let device_name = "/dev/i2c-1";
    let i2cdev = LinuxI2CDevice::new(device_name, BME280_SLAVE_ADDR)
                        .with_context(|| format!("create_i2cdev() failed. device_name={device_name}, slave_addr={BME280_SLAVE_ADDR}"))?;
    Ok(i2cdev)
}

fn read_temperature() -> Result<f32, anyhow::Error>
{
    tracing::info!("Start BME280 read temperature.");

    let dev = create_i2cdev()?;
    let mut bme280 = BME280::new(dev, BME280_SLAVE_ADDR);
    bme280.initialize().with_context(|| "BME280 initialize failed.")?;

    let compensation_data = bme280.read_compensation()?;
    let env_data = bme280.read_env_measured()?;
    let temperature = bme280::calc_temperature(compensation_data.temperature, env_data.temperature);

    tracing::info!("Read temperature: temp={temperature:?}");

    Ok(temperature)
}

fn aws_mqtt_publish() -> Result<(), anyhow::Error>
{
    let client_id = env::var("AWS_IOT_CLIENT_ID").with_context(|| "AWS_IOT_CLIENT_ID is undefined.")?;
    let aws_iot_endpoint = env::var("AWS_IOT_ENDPOINT").with_context(|| "AWS_IOT_ENDPOINT is undefined.")?;
    let ca_path = "AmazonRootCA1.pem";
    let client_cert_path = "milkv-duo-test.cert.pem";
    let client_key_path = "milkv-duo-test.private.key";

    let ca = fs::read(ca_path).with_context(|| format!("Read CA failed. path={ca_path:?}"))?;
    let client_cert = fs::read(client_cert_path).with_context(|| format!("Read client cert failed. path={client_cert_path:?}"))?;
    let client_key = fs::read(client_key_path).with_context(|| format!("Read client key failed. path={client_key_path:?}"))?;

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca,
        alpn: None,
        client_auth: Some((client_cert, client_key)),
    });
    let mut mqtt_options = MqttOptions::new(client_id, aws_iot_endpoint, 8883);
    mqtt_options
        .set_transport(transport)
        .set_keep_alive(Duration::from_secs(10));

    let (mqtt_client, mut connection) = Client::new(mqtt_options, 10);

    let sleep_time = Duration::from_secs(10);
    thread::spawn(move || {
        loop {
            let dt: DateTime<Local> = Local::now();
            let timestamp = dt.to_rfc3339_opts(SecondsFormat::Millis, true);
            let temperature = read_temperature();
            if let Err(e) = temperature {
                tracing::error!("Read temperature failed. err={e:?}");
                thread::sleep(sleep_time);
                continue;
            }

            let payload = Payload { timestamp: &timestamp, temperature: temperature.unwrap() };
            let payload_str = serde_json::to_string(&payload).unwrap();
            let topic = "iot/topic";

            let _ = mqtt_client.publish(topic, QoS::AtLeastOnce, false, payload_str.clone())
                               .inspect_err( |err| tracing::error!("Publish failed. err={err:?}, topic={topic:?}, payload_str={payload_str:?}") );
            thread::sleep(sleep_time);
        }
    });

    for (i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(ref t) => {
                tracing::info!("notification={notification:?}");
            },
            Err(ref e) => {
                tracing::error!("notification={notification:?}");
            }
        }
    }

    Ok(())
}

#[tracing::instrument]
fn main() -> Result<(), anyhow::Error> 
{
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let result = aws_mqtt_publish();
    match result {
        Ok(t) => Ok(t),
        Err(e) => {
            tracing::error!("Program abort with error = {e:}");
            Err(e)
        },
    }
}