use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use crate::DeviceResult;
use crate::error::DeviceError;
use crate::smart_device::SocketState;

#[derive(Debug)]
pub struct SocketSimulator {
    listener: TcpListener,
    state: SimulatorSocketState,
    clients: Vec<Client>,
    tick_delay: Duration,
}

impl SocketSimulator {
    pub fn bind(
        address: impl ToSocketAddrs,
        active_power: f32,
        active_power_offset: f32,
    ) -> DeviceResult<Self> {
        let listener =
            TcpListener::bind(address).map_err(|error| DeviceError::io("bind TCP", error))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| DeviceError::io("set TCP listener nonblocking mode", error))?;

        Ok(Self {
            listener,
            state: SimulatorSocketState::new(active_power, active_power_offset),
            clients: Vec::new(),
            tick_delay: Duration::from_millis(5),
        })
    }

    pub fn local_addr(&self) -> DeviceResult<SocketAddr> {
        self.listener
            .local_addr()
            .map_err(|error| DeviceError::io("read TCP local address", error))
    }

    pub fn run(&mut self) -> DeviceResult<()> {
        loop {
            self.tick()?;
            thread::sleep(self.tick_delay);
        }
    }

    pub fn run_until(&mut self, shutdown: &std::sync::atomic::AtomicBool) -> DeviceResult<()> {
        while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            self.tick()?;
            thread::sleep(self.tick_delay);
        }

        Ok(())
    }

    fn tick(&mut self) -> DeviceResult<()> {
        self.accept_clients()?;

        let mut clients = std::mem::take(&mut self.clients);

        for mut client in clients.drain(..) {
            client.read_commands(&mut self.state);
            client.flush_output();

            if !client.is_closed() {
                self.clients.push(client);
            }
        }

        Ok(())
    }

    fn accept_clients(&mut self) -> DeviceResult<()> {
        loop {
            match self.listener.accept() {
                Ok((stream, _address)) => {
                    stream.set_nonblocking(true).map_err(|error| {
                        DeviceError::io("set TCP stream nonblocking mode", error)
                    })?;
                    self.clients.push(Client::new(stream));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => return Err(DeviceError::io("accept TCP client", error)),
            }
        }
    }
}

#[derive(Debug)]
struct SimulatorSocketState {
    state: SocketState,
    active_power: f32,
    active_power_offset: f32,
}

impl SimulatorSocketState {
    fn new(active_power: f32, active_power_offset: f32) -> Self {
        Self {
            state: SocketState::Off,
            active_power,
            active_power_offset,
        }
    }

    fn handle_command(&mut self, command: &str) -> String {
        match command.trim().to_ascii_uppercase().as_str() {
            "ON" => {
                self.state = SocketState::On;
                "OK".to_string()
            }
            "OFF" => {
                self.state = SocketState::Off;
                "OK".to_string()
            }
            "STATE" => self.state.to_string(),
            "POWER" => format!("{:.2}", self.power()),
            unknown => format!("ERR unknown command '{unknown}'"),
        }
    }

    fn power(&self) -> f32 {
        match self.state {
            SocketState::On => self.active_power + self.active_power_offset,
            SocketState::Off => 0.0,
        }
    }
}

#[derive(Debug)]
struct Client {
    stream: TcpStream,
    input: Vec<u8>,
    output: Vec<u8>,
    closed: bool,
}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            input: Vec::new(),
            output: Vec::new(),
            closed: false,
        }
    }

    fn is_closed(&self) -> bool {
        self.closed && self.output.is_empty()
    }

    fn read_commands(&mut self, state: &mut SimulatorSocketState) {
        let mut buffer = [0_u8; 512];

        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    self.closed = true;
                    break;
                }
                Ok(bytes_read) => {
                    self.input.extend_from_slice(&buffer[..bytes_read]);
                    self.process_commands(state);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => {
                    self.closed = true;
                    break;
                }
            }
        }
    }

    fn process_commands(&mut self, state: &mut SimulatorSocketState) {
        while let Some(newline_index) = self.input.iter().position(|byte| *byte == b'\n') {
            let line = self.input.drain(..=newline_index).collect::<Vec<_>>();
            let command = String::from_utf8_lossy(&line);
            let response = state.handle_command(command.trim());
            self.output.extend_from_slice(response.as_bytes());
            self.output.push(b'\n');
        }
    }

    fn flush_output(&mut self) {
        while !self.output.is_empty() {
            match self.stream.write(&self.output) {
                Ok(0) => {
                    self.closed = true;
                    break;
                }
                Ok(bytes_written) => {
                    self.output.drain(..bytes_written);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => {
                    self.closed = true;
                    break;
                }
            }
        }
    }
}
