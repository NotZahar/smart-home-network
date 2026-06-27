use std::collections::HashMap;
use std::fmt;

use crate::DeviceResult;
use crate::error::{DeviceError, HomeError};
use crate::report::Report;
use crate::room::{Room, SmartRoom};
use crate::smart_device::Device;

pub trait Home {
    fn new(rooms: HashMap<String, SmartRoom>) -> Self;

    fn get_room(&self, key: &str) -> Option<&SmartRoom>;

    fn get_room_mut(&mut self, key: &str) -> Option<&mut SmartRoom>;

    fn add_room(&mut self, key: String, room: SmartRoom);

    fn remove_room(&mut self, key: &str);

    fn get_device(&self, room_key: &str, device_key: &str) -> Result<&Device, HomeError>;

    fn get_device_mut(
        &mut self,
        room_key: &str,
        device_key: &str,
    ) -> Result<&mut Device, HomeError>;
}

#[derive(Debug)]
pub struct SmartHome {
    rooms: HashMap<String, SmartRoom>,
}

impl SmartHome {
    pub fn new(rooms: HashMap<String, SmartRoom>) -> Self {
        Self { rooms }
    }
}

impl Home for SmartHome {
    fn new(rooms: HashMap<String, SmartRoom>) -> Self {
        Self::new(rooms)
    }

    fn get_room(&self, key: &str) -> Option<&SmartRoom> {
        self.rooms.get(key)
    }

    fn get_room_mut(&mut self, key: &str) -> Option<&mut SmartRoom> {
        self.rooms.get_mut(key)
    }

    fn add_room(&mut self, key: String, room: SmartRoom) {
        self.rooms.insert(key, room);
    }

    fn remove_room(&mut self, key: &str) {
        self.rooms.remove(key);
    }

    fn get_device(&self, room_key: &str, device_key: &str) -> Result<&Device, HomeError> {
        let room = self
            .rooms
            .get(room_key)
            .ok_or_else(|| HomeError::RoomNotFound(room_key.to_string()))?;

        room.get_device(device_key)
            .ok_or_else(|| HomeError::DeviceNotFound(device_key.to_string()))
    }

    fn get_device_mut(
        &mut self,
        room_key: &str,
        device_key: &str,
    ) -> Result<&mut Device, HomeError> {
        let room = self
            .rooms
            .get_mut(room_key)
            .ok_or_else(|| HomeError::RoomNotFound(room_key.to_string()))?;

        room.get_device_mut(device_key)
            .ok_or_else(|| HomeError::DeviceNotFound(device_key.to_string()))
    }
}

impl Report for SmartHome {
    fn report(&self) -> DeviceResult<String> {
        let mut result = String::from("Home:\n");

        for (name, room) in &self.rooms {
            result.push_str(&format!(" - Room '{name}':\n"));
            let room_report = room
                .report()
                .map_err(|error| DeviceError::report(format!("room '{name}'"), error))?;

            for line in room_report.lines() {
                result.push_str(&format!("    - {line}\n"));
            }
        }

        Ok(result)
    }
}

impl fmt::Display for SmartHome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.report() {
            Ok(report) => formatter.write_str(&report),
            Err(error) => write!(formatter, "failed to build report: {error}"),
        }
    }
}
