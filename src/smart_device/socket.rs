use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use crate::DeviceResult;
use crate::error::DeviceError;
use crate::report::Report;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocketState {
    On,
    Off,
}

impl fmt::Display for SocketState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::On => formatter.write_str("ON"),
            Self::Off => formatter.write_str("OFF"),
        }
    }
}

impl FromStr for SocketState {
    type Err = DeviceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "ON" => Ok(Self::On),
            "OFF" => Ok(Self::Off),
            other => Err(DeviceError::Protocol(format!(
                "unknown socket state '{other}'"
            ))),
        }
    }
}

pub trait Socket: fmt::Debug + Send + Sync {
    const DEFAULT_INACTIVE_POWER: f32 = 0.0;

    fn turn_on(&self) -> DeviceResult<()>;

    fn turn_off(&self) -> DeviceResult<()>;

    fn get_state(&self) -> DeviceResult<SocketState>;

    fn get_power(&self) -> DeviceResult<f32>;
}

#[derive(Debug)]
pub struct EmulatedPowerSocket {
    state: Mutex<SocketState>,
    active_power: f32,
}

impl EmulatedPowerSocket {
    pub fn new(active_power: f32) -> Self {
        Self {
            state: Mutex::new(SocketState::Off),
            active_power,
        }
    }

    pub fn turn_on(&self) -> DeviceResult<()> {
        *self.lock_state()? = SocketState::On;
        Ok(())
    }

    pub fn turn_off(&self) -> DeviceResult<()> {
        *self.lock_state()? = SocketState::Off;
        Ok(())
    }

    pub fn get_state(&self) -> DeviceResult<SocketState> {
        Ok(*self.lock_state()?)
    }

    pub fn get_power(&self) -> DeviceResult<f32> {
        match self.get_state()? {
            SocketState::On => Ok(self.active_power),
            SocketState::Off => Ok(Self::DEFAULT_INACTIVE_POWER),
        }
    }

    fn lock_state(&self) -> DeviceResult<MutexGuard<'_, SocketState>> {
        self.state
            .lock()
            .map_err(|_| DeviceError::LockPoisoned("emulated socket state"))
    }
}

impl Socket for EmulatedPowerSocket {
    fn turn_on(&self) -> DeviceResult<()> {
        Self::turn_on(self)
    }

    fn turn_off(&self) -> DeviceResult<()> {
        Self::turn_off(self)
    }

    fn get_state(&self) -> DeviceResult<SocketState> {
        Self::get_state(self)
    }

    fn get_power(&self) -> DeviceResult<f32> {
        Self::get_power(self)
    }
}

impl Report for EmulatedPowerSocket {
    fn report(&self) -> DeviceResult<String> {
        Ok(format!(
            "Socket {{ state: {}, power: {:.2} W }}",
            self.get_state()?,
            self.get_power()?
        ))
    }
}

#[derive(Clone)]
pub struct NetworkPowerSocket {
    address: String,
    timeout: Duration,
}

impl NetworkPowerSocket {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            timeout: Duration::from_secs(2),
        }
    }

    pub fn with_timeout(address: impl Into<String>, timeout: Duration) -> Self {
        Self {
            address: address.into(),
            timeout,
        }
    }

    pub fn turn_on(&self) -> DeviceResult<()> {
        self.expect_ok("ON")
    }

    pub fn turn_off(&self) -> DeviceResult<()> {
        self.expect_ok("OFF")
    }

    pub fn get_state(&self) -> DeviceResult<SocketState> {
        self.send_command("STATE")?.parse()
    }

    pub fn get_power(&self) -> DeviceResult<f32> {
        let response = self.send_command("POWER")?;
        response
            .parse::<f32>()
            .map_err(|_| DeviceError::Protocol(format!("invalid power response '{response}'")))
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    fn expect_ok(&self, command: &'static str) -> DeviceResult<()> {
        let response = self.send_command(command)?;

        if response == "OK" {
            Ok(())
        } else {
            Err(DeviceError::Protocol(format!(
                "command {command} returned '{response}'"
            )))
        }
    }

    fn send_command(&self, command: &str) -> DeviceResult<String> {
        let mut stream = TcpStream::connect(&self.address)
            .map_err(|error| DeviceError::io("connect TCP", error))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|error| DeviceError::io("set TCP read timeout", error))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|error| DeviceError::io("set TCP write timeout", error))?;
        writeln!(stream, "{command}")
            .map_err(|error| DeviceError::io("write TCP command", error))?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        let bytes = reader
            .read_line(&mut response)
            .map_err(|error| DeviceError::io("read TCP response", error))?;

        if bytes == 0 {
            return Err(DeviceError::Protocol("empty socket response".to_string()));
        }

        Ok(response.trim().to_string())
    }
}

impl fmt::Debug for NetworkPowerSocket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NetworkPowerSocket")
            .field("address", &self.address)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Socket for NetworkPowerSocket {
    fn turn_on(&self) -> DeviceResult<()> {
        Self::turn_on(self)
    }

    fn turn_off(&self) -> DeviceResult<()> {
        Self::turn_off(self)
    }

    fn get_state(&self) -> DeviceResult<SocketState> {
        Self::get_state(self)
    }

    fn get_power(&self) -> DeviceResult<f32> {
        Self::get_power(self)
    }
}

impl Report for NetworkPowerSocket {
    fn report(&self) -> DeviceResult<String> {
        Ok(format!(
            "Socket {{ state: {}, power: {:.2} W, transport: TCP, address: {} }}",
            self.get_state()?,
            self.get_power()?,
            self.address
        ))
    }
}
