#![feature(error_generic_member_access)]

use std::io;
use std::sync::WaitTimeoutResult;
use i2cdev::core::I2CDevice;
use i2cdev::linux::*;
use anyhow::{Result, Context};

mod bme280;
use crate::bme280::BME280;

const BME280_SLAVE_ADDR: u16 = 0x76;

fn create_i2cdev() -> Result<impl I2CDevice>
{
    let device_name = "/dev/i2c-1";
    let i2cdev = LinuxI2CDevice::new(device_name, BME280_SLAVE_ADDR)
                        .with_context(|| format!("create_i2cdev() failed. device_name={}, slave_addr={}", device_name, BME280_SLAVE_ADDR))?;
    Ok(i2cdev)
}

fn read_temperature() -> Result<(), anyhow::Error>
{
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("BME280 temperature reader.");
    let dev = create_i2cdev()?;
    let mut bme280 = BME280::new(dev, BME280_SLAVE_ADDR);
    bme280.initialize().with_context(|| "BME280 initialize failed.")?;

    let compensation_data = bme280.read_compensation()?;
    let env_data = bme280.read_env_measured()?;
    let temperature = bme280::calc_temperature(compensation_data.temperature, env_data.temperature);

    println!("temperature: {:?}", temperature);

    Ok(())
}

#[tracing::instrument]
fn main() -> Result<(), anyhow::Error> 
{
    let result = read_temperature();
    match result {
        Ok(t) => Ok(t),
        Err(e) => {
            Err(e)
        },
    }
}