#[macro_export]
macro_rules! make_room {
    ( $($device_name:expr => $device:expr),* $(,)? ) => {
        {
            let mut devices = std::collections::HashMap::new();
            $(
                devices.insert($device_name.to_string(), $device.into());
            )*
            $crate::room::SmartRoom::new(devices)
        }
    };
}

pub mod error;
pub mod home;
pub mod report;
pub mod room;
pub mod simulators;
pub mod smart_device;

mod utils;

pub type DeviceResult<T> = Result<T, error::DeviceError>;
