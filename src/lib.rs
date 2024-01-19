use std::collections::HashMap;

use micro_rdk::DoCommand;
use micro_rdk::common::{
    config::ConfigType,
    registry::{ComponentRegistry, Dependency, RegistryError},
    sensor::{
        GenericReadingsResult, Readings, Sensor, SensorResult, SensorT, SensorType, TypedReadingsResult,
    },
    status::Status,
};

pub fn register_models(_registry: &mut ComponentRegistry) -> anyhow::Result<(), RegistryError> {
    registry.register_sensor("adafruit-scd30", &AdafruitSCD30::from_config)?;
    log::debug!("adafruit-scd-30 sensor registration ok");
    Ok(())
}

pub struct AdafruitSCD30;



impl AdafruitSCD30 {
    pub fn from_config(_cfg: ConfigType, _deps: Vec<Dependency>) -> anyhow::Result<SensorType> {
        println!("adafruit-scd30 sensor instantiated from config");
        Ok(Arc::new(Mutex::new(Self {})))
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

impl SensorT<f64> for AdafruitSCD30 {
    fn get_readings(&self) -> anyhow::Result<TypedReadingsResult<f64>> {
        let mut x = HashMap::new();
        x.insert("fake_temp".to_string(), 75.0);
        println!("scd30 - get readings OK");
        Ok(x)
    }
}

impl Status for AdafruitSCD30 {
    fn get_status(&self) -> anyhow::Result<Option<micro_rdk::google::protobuf::Struct>> {
        log::debug!("wifi-rssi sensor - get status called");
        Ok(Some(micro_rdk::google::protobuf::Struct {
            fields: HashMap::new(),
        }))
    }
}