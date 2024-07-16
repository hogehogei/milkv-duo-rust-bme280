use std::backtrace::Backtrace;
use std::error::Error;
use i2cdev::core::I2CDevice;

// BME280 temperature compensation registers "dig_T*"
const BME280_TEMPERATURE_COMP_REG: [u8; 6] = [
    0x88, 0x89, 
    0x8A, 0x8B,
    0x8C, 0x8D
];

// BME280 pressure compensation registers "dig_P*"
const BME280_PRESSURE_COMP_REG: [u8; 18] = [
    0x8E, 0x8F,
    0x90, 0x91,
    0x92, 0x93,
    0x94, 0x95,
    0x96, 0x97,
    0x98, 0x99,
    0x9A, 0x9B,
    0x9C, 0x9D,
    0x9E, 0x9F,
];

// BME280 humidity compensation registers "dig_H*"
const BME280_HUMIDITY_COMP_REG_HI: [u8; 1] = [
    0xA1
];
const BME280_HUMIDITY_COMP_REG_LO: [u8; 7] = [
    0xE1, 0xE2,
    0xE3,
    0xE4, 0xE5, 0xE6,
    0xE7,
];

// BME280 read pressure register "press"
const BME280_PRESS_REG: [u8; 3] = [
    0xF7, 0xF8, 0xF9,
];

// BME280 read temperature register "temp"
const BME280_TEMP_REG: [u8; 3] = [
    0xFA, 0xFB, 0xFC,
];

// BME280 read humidity register "hum"
const BME280_HUM_REG: [u8; 2] = [
    0xFD, 0xFE,
];

pub struct BME280<T: I2CDevice>
{
    i2cdev: T,
    slave_address: u16,
}

pub struct CompTemperature
{
    t1: u16,
    t2: i16,
    t3: i16
}

impl CompTemperature
{
    pub fn new() -> Self
    {
        Self { t1: 0, t2: 0, t3: 0 }
    }
}

pub struct CompPressure
{
    p1: u16,
    p2: i16,
    p3: i16,
    p4: i16,
    p5: i16,
    p6: i16,
    p7: i16,
    p8: i16,
    p9: i16,
}

impl CompPressure
{
    pub fn new() -> Self
    {
        Self { p1: 0, p2: 0, p3: 0, p4: 0, p5: 0, p6: 0, p7: 0, p8: 0, p9: 0 }
    }
}

pub struct CompHumidity
{
    h1: u8,
    h2: i16,
    h3: u8,
    h4: i16,
    h5: i16,
    h6: i8,
}

impl CompHumidity
{
    pub fn new() -> Self
    {
        Self { h1: 0, h2: 0, h3: 0, h4: 0, h5: 0, h6: 0 }
    }
}

pub struct CompensationData
{
    pub temperature: CompTemperature,
    pub pressure: CompPressure,
    pub humidity: CompHumidity,
}

impl CompensationData
{
    pub fn new() -> Self
    {
        Self { temperature: CompTemperature::new(), pressure: CompPressure::new(), humidity: CompHumidity::new() }
    }
}

pub struct EnvData
{
    pub pressure: i32,
    pub temperature: i32,
    pub humidity: i32,
}

impl EnvData
{
    pub fn new() -> Self
    {
        Self { pressure: 0, temperature: 0, humidity: 0 }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BME280ErrorType {
    #[error("Failed initialize.\n{}", backtrace)]
    InitializeError {
        backtrace: Backtrace
    },
    #[error("Failed read data.\n{}", backtrace)]
    ReadError {
        backtrace: Backtrace
    },
    #[error("Failed write data.\n{}", backtrace)]
    WriteError {
        backtrace: Backtrace
    },
}

impl<T> BME280<T>
where 
    T: I2CDevice
{
    pub fn new(dev: T, slave_addr: u16) -> Self
    {
        Self { i2cdev: dev, slave_address: slave_addr }
    }

    pub fn write(&mut self, register_addr: u8, value: u8) -> Result<&mut Self, BME280ErrorType>
    {
        self.i2cdev.smbus_write_byte_data(register_addr, value)
                   .map_err(|e| {
                        BME280ErrorType::WriteError { backtrace: Backtrace::force_capture() }
                   })?;
        Ok(self)
    }

    pub fn read(&mut self, register_addr: u8, buf: &mut [u8]) -> Result<&mut Self, BME280ErrorType>
    {
        let read_data = self.i2cdev.smbus_read_i2c_block_data(register_addr, buf.len() as u8)
                                     .map_err(|e| {
                                        BME280ErrorType::ReadError { backtrace: Backtrace::force_capture() }
                                     })?;
        for i in 0..buf.len() {
            buf[i] = read_data[i];
        }

        Ok(self)
    }

    pub fn initialize(&mut self) -> Result<&mut Self, BME280ErrorType>
    {
        let mut reg: u8;
        let mut value: u8;

        // set "config(0xF5)" register
        // t_sb[2:0]   = 0.5ms(000)
        // filter[2:0] = filter x16(100)
        // spi3w_en[0] = not enable 3wire SPI(0)
        // value = |000|100|*|0|
        reg = 0xF5;
        value = 0x10;
        self.write(reg, value)
            .map_err(|e| BME280ErrorType::InitializeError { backtrace: Backtrace::force_capture() })?;

        // set "ctrl_meas(0xF4)" register
        // osrs_t[2:0]     = oversamplingx2(010)
        // osrs_p[2:0]     = oversamplingx16(101)
        // mode[1:0]       = normal mode(11)
        // value = |010|101|11|
        reg = 0xF4;
        value = 0x57;
        self.write(reg, value)
            .map_err(|e| BME280ErrorType::InitializeError { backtrace: Backtrace::force_capture() })?;
    
        // set "ctrl_hum(0xF2)" register
        // osrs_h[2:0]  = oversamplingx1(001)
        // value = |*****|001|
        reg = 0xF2;
        value = 0x01;
        self.write(reg, value)
            .map_err(|e| BME280ErrorType::InitializeError { backtrace: Backtrace::force_capture() })?;

        Ok(self)
    }

    pub fn read_env_measured(&mut self) -> Result<EnvData, BME280ErrorType>
    {
        let mut reg_press = [0_u8; BME280_PRESS_REG.len()];
        let mut reg_temp = [0_u8; BME280_TEMP_REG.len()];
        let mut reg_hum = [0_u8; BME280_HUM_REG.len()];

        // read pressure data
        self.read(BME280_PRESS_REG[0], &mut reg_press)?;
        // read temperature data
        self.read(BME280_TEMP_REG[0], &mut reg_temp)?;
        // read humidity data
        self.read(BME280_HUM_REG[0], &mut reg_hum)?;

        let mut envdata = EnvData::new();

        envdata.pressure = ((reg_press[0] as u32) << 16 | (reg_press[1] as u32) << 8 | (reg_press[2] as u32)) as i32;
        envdata.pressure >>= 4;
        envdata.temperature = ((reg_temp[0] as u32) << 16 | (reg_temp[1] as u32) << 8 | (reg_temp[2] as u32)) as i32;
        envdata.temperature >>= 4;
        envdata.humidity = ((reg_hum[0] as u32) << 8 | (reg_hum[1] as u32)) as i32;

        Ok(envdata)
    }

    pub fn read_compensation(&mut self) -> Result<CompensationData, BME280ErrorType>
    {
        let mut reg_t = [0_u8; BME280_TEMPERATURE_COMP_REG.len()];
        let mut reg_p = [0_u8; BME280_PRESSURE_COMP_REG.len()];
        let mut reg_h = [0_u8; BME280_HUMIDITY_COMP_REG_HI.len() + BME280_HUMIDITY_COMP_REG_LO.len()];
    
        let mut dig_t =  CompTemperature::new();
        let mut dig_p = CompPressure::new();
        let mut dig_h = CompHumidity::new();
    
        // read temperature compensation data
        self.read(BME280_TEMPERATURE_COMP_REG[0], &mut reg_t)?;
        // read pressure compensation data
        self.read(BME280_PRESSURE_COMP_REG[0], &mut reg_p)?;
        // read humidity compensation hi data
        self.read(BME280_HUMIDITY_COMP_REG_HI[0], &mut reg_h)?;
        self.read(BME280_HUMIDITY_COMP_REG_LO[0], &mut reg_h[BME280_HUMIDITY_COMP_REG_HI.len()..])?;
    
        // ok. format compensation data.
        dig_t.t1 = (reg_t[0] as u16) | ((reg_t[1] as u16) << 8);
        dig_t.t2 = ((reg_t[2] as u16) | ((reg_t[3] as u16) << 8)) as i16;
        dig_t.t3 = ((reg_t[4] as u16) | ((reg_t[5] as u16) << 8)) as i16;
    
        dig_p.p1 = (reg_p[0] as u16) | ((reg_p[1] as u16) << 8);
        dig_p.p2 = ((reg_p[2] as u16)| ((reg_p[3] as u16) << 8)) as i16;
        dig_p.p3 = ((reg_p[4] as u16)| ((reg_p[5] as u16) << 8)) as i16;
        dig_p.p4 = ((reg_p[6] as u16)| ((reg_p[7] as u16) << 8)) as i16;
        dig_p.p5 = ((reg_p[8] as u16)| ((reg_p[9] as u16) << 8)) as i16;
        dig_p.p6 = ((reg_p[10] as u16)| ((reg_p[11] as u16) << 8)) as i16;
        dig_p.p7 = ((reg_p[12] as u16)| ((reg_p[13] as u16) << 8)) as i16;
        dig_p.p8 = ((reg_p[14] as u16)| ((reg_p[15] as u16) << 8)) as i16;
        dig_p.p9 = ((reg_p[16] as u16)| ((reg_p[17] as u16) << 8)) as i16;
    
        dig_h.h1 = reg_h[0];
        dig_h.h2 = ((reg_h[1] as u16) | ((reg_h[2] as u16) << 8)) as i16;
        dig_h.h3 = reg_h[3];
        dig_h.h4 = (((reg_h[4] as u16) << 4 as u16) | ((reg_h[5] as u16) & 0x0F)) as i16;
        dig_h.h5 = (((reg_h[5] as u16) >> 4 as u16) | ((reg_h[6] as u16) << 8)) as i16;
        dig_h.h6 = reg_h[7] as i8;
    
        let mut compdata = CompensationData::new();
        compdata.temperature = dig_t;
        compdata.pressure = dig_p;
        compdata.humidity = dig_h;
    
        Ok(compdata)
    }
}

pub fn calc_temperature(compt: CompTemperature, env_temperature: i32) -> f32
{
    let adc_t: i32;
    let (var1, var2, t): (i32, i32, i32);
    let t_fine_out: i32;

    adc_t = env_temperature;

    var1 = ((((adc_t >> 3) - ((compt.t1 as i32) << 1))) * (compt.t2 as i32)) >> 11;
    var2 = (((((adc_t >> 4) - (compt.t1 as i32)) * ((adc_t >> 4) - (compt.t1 as i32))) >> 12) * (compt.t3 as i32)) >> 14;
    
    t_fine_out = var1 + var2;
    t = ((t_fine_out * 5) + 128) >> 8;

    return (t as f32) / 100.0_f32;
}

