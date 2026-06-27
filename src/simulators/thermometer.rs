use std::fs;
use std::net::UdpSocket;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::DeviceResult;
use crate::error::DeviceError;
use crate::utils::random::{RandomGenerator, SimpleRandomGenerator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThermometerSimulatorConfig {
    pub target_addr: String,
    pub period: Duration,
}

impl ThermometerSimulatorConfig {
    pub fn from_file(path: impl AsRef<Path>) -> DeviceResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|error| DeviceError::io("read thermometer simulator config", error))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> DeviceResult<Self> {
        let mut target_addr = None;
        let mut period_ms = None;

        for (line_index, raw_line) in content.lines().enumerate() {
            let line = raw_line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line.split_once('=').ok_or_else(|| {
                DeviceError::InvalidConfig(format!(
                    "line {} must have key=value format",
                    line_index + 1
                ))
            })?;

            match key.trim() {
                "target_addr" | "address" => target_addr = Some(value.trim().to_string()),
                "period_ms" => {
                    period_ms = Some(value.trim().parse::<u64>().map_err(|_| {
                        DeviceError::InvalidConfig(format!(
                            "line {} has invalid period_ms",
                            line_index + 1
                        ))
                    })?);
                }
                unknown => {
                    return Err(DeviceError::InvalidConfig(format!(
                        "line {} has unknown key '{unknown}'",
                        line_index + 1
                    )));
                }
            }
        }

        let target_addr = target_addr
            .ok_or_else(|| DeviceError::InvalidConfig("missing target_addr value".to_string()))?;
        let period_ms = period_ms
            .ok_or_else(|| DeviceError::InvalidConfig("missing period_ms value".to_string()))?;

        if period_ms == 0 {
            return Err(DeviceError::InvalidConfig(
                "period_ms must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            target_addr,
            period: Duration::from_millis(period_ms),
        })
    }
}

#[derive(Debug)]
pub struct ThermometerSimulator {
    socket: UdpSocket,
    target_addr: String,
    period: Duration,
    random_temperature_generator: SimpleRandomGenerator<f32>,
}

impl ThermometerSimulator {
    pub fn new(target_addr: impl Into<String>, period: Duration) -> DeviceResult<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|error| DeviceError::io("bind UDP sender", error))?;
        socket
            .set_nonblocking(true)
            .map_err(|error| DeviceError::io("set UDP sender nonblocking mode", error))?;

        Ok(Self {
            socket,
            target_addr: target_addr.into(),
            period,
            random_temperature_generator: SimpleRandomGenerator::new(),
        })
    }

    pub fn from_config(config: ThermometerSimulatorConfig) -> DeviceResult<Self> {
        Self::new(config.target_addr, config.period)
    }

    pub fn from_file(path: impl AsRef<Path>) -> DeviceResult<Self> {
        Self::from_config(ThermometerSimulatorConfig::from_file(path)?)
    }

    pub fn run(&mut self) -> DeviceResult<()> {
        loop {
            self.send_temperature()?;
            thread::sleep(self.period);
        }
    }

    pub fn run_until(&mut self, shutdown: &std::sync::atomic::AtomicBool) -> DeviceResult<()> {
        while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            self.send_temperature()?;
            thread::sleep(self.period);
        }

        Ok(())
    }

    fn send_temperature(&mut self) -> DeviceResult<()> {
        let packet = format!("{:.2}\n", self.next_temperature());

        match self.socket.send_to(packet.as_bytes(), &self.target_addr) {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
            Err(error) => Err(DeviceError::io("send UDP temperature", error)),
        }
    }

    fn next_temperature(&mut self) -> f32 {
        self.random_temperature_generator.generate(20.0, 30.0)
    }
}
