use std::collections::HashMap;

use crate::DeviceResult;
use crate::error::DeviceError;
use crate::report::Report;
use crate::smart_device::Device;

pub trait Room {
    fn new(devices: HashMap<String, Device>) -> Self;

    fn get_device(&self, key: &str) -> Option<&Device>;

    fn get_device_mut(&mut self, key: &str) -> Option<&mut Device>;

    fn add_device(&mut self, key: String, device: Device);

    fn remove_device(&mut self, key: &str);
}

#[derive(Debug)]
pub struct SmartRoom {
    devices: HashMap<String, Device>,
}

impl SmartRoom {
    pub fn new(devices: HashMap<String, Device>) -> Self {
        Self { devices }
    }
}

impl Room for SmartRoom {
    fn new(devices: HashMap<String, Device>) -> Self {
        Self::new(devices)
    }

    fn get_device(&self, key: &str) -> Option<&Device> {
        self.devices.get(key)
    }

    fn get_device_mut(&mut self, key: &str) -> Option<&mut Device> {
        self.devices.get_mut(key)
    }

    fn add_device(&mut self, key: String, device: Device) {
        self.devices.insert(key, device);
    }

    fn remove_device(&mut self, key: &str) {
        self.devices.remove(key);
    }
}

impl Report for SmartRoom {
    fn report(&self) -> DeviceResult<String> {
        let mut result = String::new();

        for (device_name, device) in &self.devices {
            let device_report = device
                .report()
                .map_err(|error| DeviceError::report(format!("device '{device_name}'"), error))?;
            result.push_str(&format!("Device '{device_name}': {device_report}\n"));
        }

        Ok(result)
    }
}
