
use crate::common::i2c::I2cHandleType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use micro_rdk::DoCommand;
use micro_rdk::common::{
    config::ConfigType,
    registry::{ComponentRegistry, Dependency, RegistryError},
    sensor::{
        GenericReadingsResult, Readings, Sensor, SensorResult, SensorT, SensorType, TypedReadingsResult,
    },
    status::Status,
};

pub fn register_models(registry: &mut ComponentRegistry) -> anyhow::Result<(), RegistryError> {
    registry.register_sensor("adafruit-scd30", &AdafruitSCD30::from_config)?;
    println!("adafruit-scd-30 sensor registration ok");
    Ok(())
}

const RESET_COMMAND: u8 = 0xD304;
const READ_COMMAND: u8 = 0x0300;

fn _get_command_bytes(command: u8) -> [u8; 2] {
    [command >> 8, command & 0xFF]
}


#[derive(DoCommand)]
pub struct AdafruitSCD30 {
    i2c_handle: I2cHandleType,
    i2c_address: u8,
}

impl AdafruitSCD30 {
    pub fn new(mut i2c_handle: I2cHandleType, i2c_address: u8) -> anyhow::Result<Self> {
        // let bytes: [u8; 2] = [RESET_COMMAND >> 8, RESET_COMMAND & 0xFF];
        let bytes = _get_command_bytes(RESET_COMMAND);
        i2c_handle.write_i2c(i2c_address, &bytes)?;
        Ok(Self {
            i2c_handle,
            i2c_address,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn from_config(
        cfg: ConfigType,
        dependencies: Vec<Dependency>,
    ) -> anyhow::Result<MovementSensorType> {
        let board = get_board_from_dependencies(dependencies);
        if board.is_none() {
            return Err(anyhow::anyhow!(
                "actual board is required to be passed to configure AdafruitSCD30"
            ));
        }
        let board_unwrapped = board.unwrap();
        let i2c_handle: I2cHandleType;
        if let Ok(i2c_name) = cfg.get_attribute::<String>("i2c_bus") {
            i2c_handle = board_unwrapped.get_i2c_by_name(i2c_name)?;
        } else {
            return Err(anyhow::anyhow!(
                "i2c_bus is a required config attribute for AdafruitSCD30"
            ));
        };
        if let Ok(use_alt_address) = cfg.get_attribute::<bool>("use_alt_i2c_address") {
            if use_alt_address {
                return Ok(Arc::new(Mutex::new(ADXL345::new(i2c_handle, 29)?)));
            }
            Ok(Arc::new(Mutex::new(ADXL345::new(i2c_handle, 83)?)))
        } else {
            Ok(Arc::new(Mutex::new(ADXL345::new(i2c_handle, 83)?)))
        }
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        // put the MPU in the sleep state
        let off_data: [u8; 2] = [STANDBY_MODE_REGISTER, 0];
        if let Err(err) = self.i2c_handle.write_i2c(self.i2c_address, &off_data) {
            return Err(anyhow::anyhow!("AdafruitSCD30 sleep command failed: {:?}", err));
        };
        Ok(())
    }

}

impl Drop for AdafruitSCD30 {
    fn drop(&mut self) {
        if let Err(err) = self.close() {
            log::error!("adxl-345 close failure: {:?}", err)
        };
    }
}

impl Sensor for AdafruitSCD30 {}

impl Readings for AdafruitSCD30 {
    fn get_generic_readings(&mut self) -> anyhow::Result<GenericReadingsResult> {
        Ok(self
            .get_readings()?
            .into_iter()
            .map(|v| (v.0, SensorResult::<f64> { value: v.1 }.into()))
            .collect())
    }
}

fn get_reading_from_bytes(reading: &[u8; 18], start: usize) -> anyhow::Result<f32> {
    let first_slice = reading[start..start + 2];
    let second_slice = reading[start + 3..start + 5];
    let combined = [
        first_slice[0],
        first_slice[1],
        second_slice[0],
        second_slice[1],
    ];
    Ok(f32::from_be_bytes(combined.try_into()?))
}

impl SensorT<f32> for AdafruitSCD30 {
    /**
     * Reference from: https://github.com/viamrobotics/micro-rdk/blob/2b9d95885f89e3512a9f54309596b27803409321/src/common/adxl345.rs#L124C4-L130C6 
     */
    fn get_readings(&self) -> anyhow::Result<TypedReadingsResult<f32>> {
        let mut x = HashMap::new();
        // x.insert("fake_temp".to_string(), 75.0);
        // println!("scd30 - get readings OK");
        // Ok(x)
        // let register_write: [u8; 1] = [READING_START_REGISTER];
        // let mut result: [u8; 6] = [0; 6];
        // self.i2c_handle
        //     .write_read_i2c(self.i2c_address, &register_write, &mut result)?;
        // Ok(get_co2_from_reading(&result))
        let command_bytes = _get_command_bytes(READ_COMMAND);
        let mut result: [u8; 18] = [0; 18];
        self.i2c_handle.write_read_i2c(self.i2c_address, &command_bytes, &mut result)?;
        // now results filled w/ what we need to unpack
        // let co2_reading = get_co2_from_reading(&result);
        let co2_reading = get_reading_from_bytes(&result, 0);
        let temp_reading = get_reading_from_bytes(&result, 6);
        let humidity_reading = get_reading_from_bytes(&result, 12);

        x.insert("co2".to_string(), co2_reading);
        x.insert("temp".to_string(), temp_reading);
        x.insert("humidity".to_string(), humidity_reading);
        Ok(x)
    }
}

impl Status for AdafruitSCD30 {
    fn get_status(&self) -> anyhow::Result<Option<micro_rdk::google::protobuf::Struct>> {
        println!("wifi-rssi sensor - get status called");
        Ok(Some(micro_rdk::google::protobuf::Struct {
            fields: HashMap::new(),
        }))
    }
}