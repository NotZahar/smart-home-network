use std::fmt;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::DeviceResult;
use crate::error::DeviceError;
use crate::report::Report;

pub trait Thermometer: fmt::Debug + Send + Sync {
    fn get_temperature(&self) -> DeviceResult<f32>;
}

#[derive(Debug)]
pub struct EmulatedThermometer {
    temperature: Mutex<f32>,
}

impl EmulatedThermometer {
    pub fn new(temperature: f32) -> Self {
        Self {
            temperature: Mutex::new(temperature),
        }
    }

    pub fn set_temperature(&self, temperature: f32) -> DeviceResult<()> {
        *self
            .temperature
            .lock()
            .map_err(|_| DeviceError::LockPoisoned("emulated thermometer temperature"))? =
            temperature;
        Ok(())
    }

    pub fn get_temperature(&self) -> DeviceResult<f32> {
        self.temperature
            .lock()
            .map(|guard| *guard)
            .map_err(|_| DeviceError::LockPoisoned("emulated thermometer temperature"))
    }
}

impl Thermometer for EmulatedThermometer {
    fn get_temperature(&self) -> DeviceResult<f32> {
        Self::get_temperature(self)
    }
}

impl Report for EmulatedThermometer {
    fn report(&self) -> DeviceResult<String> {
        Ok(format!(
            "Thermometer {{ temperature: {:.2} C }}",
            self.get_temperature()?
        ))
    }
}

pub struct NetworkThermometer {
    local_addr: SocketAddr,
    last_temperature: Arc<Mutex<Option<f32>>>,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl NetworkThermometer {
    pub fn bind(address: impl ToSocketAddrs) -> DeviceResult<Self> {
        let socket =
            UdpSocket::bind(address).map_err(|error| DeviceError::io("bind UDP", error))?;
        socket
            .set_nonblocking(true)
            .map_err(|error| DeviceError::io("set UDP nonblocking mode", error))?;
        let local_addr = socket
            .local_addr()
            .map_err(|error| DeviceError::io("read UDP local address", error))?;
        let last_temperature = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_temperature = Arc::clone(&last_temperature);
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            receive_temperatures(socket, worker_temperature, worker_shutdown);
        });

        Ok(Self {
            local_addr,
            last_temperature,
            shutdown,
            worker: Some(worker),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn get_temperature(&self) -> DeviceResult<f32> {
        let guard = self
            .last_temperature
            .lock()
            .map_err(|_| DeviceError::LockPoisoned("network thermometer temperature"))?;

        guard.ok_or(DeviceError::NoTemperature)
    }

    pub fn wait_for_temperature(&self, timeout: Duration) -> DeviceResult<f32> {
        let started_at = Instant::now();

        while started_at.elapsed() < timeout {
            match self.get_temperature() {
                Ok(temperature) => return Ok(temperature),
                Err(DeviceError::NoTemperature) => thread::sleep(Duration::from_millis(10)),
                Err(error) => return Err(error),
            }
        }

        Err(DeviceError::NoTemperature)
    }
}

impl Drop for NetworkThermometer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl fmt::Debug for NetworkThermometer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NetworkThermometer")
            .field("local_addr", &self.local_addr)
            .finish_non_exhaustive()
    }
}

impl Thermometer for NetworkThermometer {
    fn get_temperature(&self) -> DeviceResult<f32> {
        Self::get_temperature(self)
    }
}

impl Report for NetworkThermometer {
    fn report(&self) -> DeviceResult<String> {
        Ok(format!(
            "Thermometer {{ temperature: {:.2} C, transport: UDP, local_addr: {} }}",
            self.get_temperature()?,
            self.local_addr
        ))
    }
}

fn receive_temperatures(
    socket: UdpSocket,
    last_temperature: Arc<Mutex<Option<f32>>>,
    shutdown: Arc<AtomicBool>,
) {
    let mut buffer = [0_u8; 128];

    while !shutdown.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buffer) {
            Ok((bytes_read, _sender)) => {
                if let Ok(message) = std::str::from_utf8(&buffer[..bytes_read])
                    && let Ok(temperature) = message.trim().parse::<f32>()
                {
                    match last_temperature.lock() {
                        Ok(mut guard) => *guard = Some(temperature),
                        Err(_) => break,
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => thread::sleep(Duration::from_millis(5)),
        }
    }
}
