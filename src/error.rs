use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum DeviceError {
    Io {
        operation: &'static str,
        source: io::Error,
    },
    Protocol(String),
    InvalidConfig(String),
    NoTemperature,
    LockPoisoned(&'static str),
    Report {
        target: String,
        source: Box<DeviceError>,
    },
}

impl DeviceError {
    pub fn io(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }

    pub fn report(target: impl Into<String>, source: DeviceError) -> Self {
        Self::Report {
            target: target.into(),
            source: Box::new(source),
        }
    }
}

impl fmt::Display for DeviceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
            Self::Protocol(message) => write!(formatter, "protocol error: {message}"),
            Self::InvalidConfig(message) => write!(formatter, "invalid config: {message}"),
            Self::NoTemperature => write!(formatter, "temperature has not been received yet"),
            Self::LockPoisoned(target) => write!(formatter, "shared state lock poisoned: {target}"),
            Self::Report { target, source } => write!(formatter, "{target}: {source}"),
        }
    }
}

impl Error for DeviceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Report { source, .. } => Some(source),
            Self::Protocol(_)
            | Self::InvalidConfig(_)
            | Self::NoTemperature
            | Self::LockPoisoned(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum HomeError {
    RoomNotFound(String),
    DeviceNotFound(String),
}

impl fmt::Display for HomeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoomNotFound(room) => write!(formatter, "room not found: {room}"),
            Self::DeviceNotFound(device) => write!(formatter, "device not found: {device}"),
        }
    }
}

impl Error for HomeError {}
