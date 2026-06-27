use std::fmt;

use crate::DeviceResult;
use crate::report::Report;
use crate::smart_device::{
    EmulatedPowerSocket, EmulatedThermometer, NetworkPowerSocket, NetworkThermometer,
};

pub enum Device {
    Socket(NetworkPowerSocket),
    EmulatedSocket(EmulatedPowerSocket),
    Thermometer(NetworkThermometer),
    EmulatedThermometer(EmulatedThermometer),
}

impl From<NetworkPowerSocket> for Device {
    fn from(socket: NetworkPowerSocket) -> Self {
        Self::Socket(socket)
    }
}

impl From<EmulatedPowerSocket> for Device {
    fn from(socket: EmulatedPowerSocket) -> Self {
        Self::EmulatedSocket(socket)
    }
}

impl From<NetworkThermometer> for Device {
    fn from(thermometer: NetworkThermometer) -> Self {
        Self::Thermometer(thermometer)
    }
}

impl From<EmulatedThermometer> for Device {
    fn from(thermometer: EmulatedThermometer) -> Self {
        Self::EmulatedThermometer(thermometer)
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Socket(socket) => formatter
                .debug_tuple("Device::Socket")
                .field(socket)
                .finish(),
            Self::EmulatedSocket(socket) => formatter
                .debug_tuple("Device::EmulatedSocket")
                .field(socket)
                .finish(),
            Self::Thermometer(thermometer) => formatter
                .debug_tuple("Device::Thermometer")
                .field(thermometer)
                .finish(),
            Self::EmulatedThermometer(thermometer) => formatter
                .debug_tuple("Device::EmulatedThermometer")
                .field(thermometer)
                .finish(),
        }
    }
}

impl Report for Device {
    fn report(&self) -> DeviceResult<String> {
        match self {
            Self::Socket(socket) => socket.report(),
            Self::EmulatedSocket(socket) => socket.report(),
            Self::Thermometer(thermometer) => thermometer.report(),
            Self::EmulatedThermometer(thermometer) => thermometer.report(),
        }
    }
}
