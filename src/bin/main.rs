use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use sh::home::SmartHome;
use sh::make_room;
use sh::report::Report;
use sh::simulators::{SocketSimulator, ThermometerSimulator};
use sh::smart_device::{NetworkPowerSocket, NetworkThermometer};

fn main() {
    if let Err(error) = run() {
        eprintln!("Example setup error: {error}");
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let start_simulators = std::env::args().any(|arg| arg == "--start-simulators");
    let mut socket_addr =
        configured_addr("SMART_HOME_SOCKET_ADDR", start_simulators, "127.0.0.1:4000");
    let thermometer_addr = configured_addr(
        "SMART_HOME_THERMOMETER_ADDR",
        start_simulators,
        "127.0.0.1:4001",
    );

    let mut simulator_threads = Vec::new();

    if start_simulators {
        let socket_simulator = spawn_socket_simulator(socket_addr.clone())?;
        socket_addr = socket_simulator.address.clone();
        simulator_threads.push(socket_simulator);
    }

    let thermometer = match NetworkThermometer::bind(&thermometer_addr) {
        Ok(thermometer) => thermometer,
        Err(error) => {
            println!("Error: failed to start network thermometer: {error}");
            return Ok(());
        }
    };

    if start_simulators {
        simulator_threads.push(spawn_thermometer_simulator(
            thermometer.local_addr().to_string(),
        )?);
    }

    let socket = NetworkPowerSocket::new(socket_addr);

    if let Err(error) = socket.turn_on() {
        println!("Error: failed to control socket: {error}");
    }

    let _ = thermometer.wait_for_temperature(temperature_wait_timeout());

    let room = make_room!(
        "network_socket" => socket,
        "network_thermometer" => thermometer,
    );
    let home = SmartHome::new(HashMap::from([("lab".to_string(), room)]));

    match home.report() {
        Ok(report) => {
            println!("=== Network smart home ===");
            println!("{report}");
        }
        Err(error) => {
            println!("Error: failed to get device data: {error}");
        }
    }

    drop(simulator_threads);

    Ok(())
}

fn configured_addr(env_name: &str, start_simulators: bool, default_addr: &str) -> String {
    std::env::var(env_name).unwrap_or_else(|_| {
        if start_simulators {
            "127.0.0.1:0".to_string()
        } else {
            default_addr.to_string()
        }
    })
}

fn temperature_wait_timeout() -> Duration {
    let wait_ms = std::env::var("SMART_HOME_TEMPERATURE_WAIT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(2_000);

    Duration::from_millis(wait_ms)
}

struct SimulatorThread {
    address: String,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<sh::DeviceResult<()>>>,
}

impl Drop for SimulatorThread {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_socket_simulator(address: String) -> Result<SimulatorThread, Box<dyn Error>> {
    let mut simulator = SocketSimulator::bind(address, 100.0, 15.0)?;
    let address = simulator.local_addr()?.to_string();
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let handle = thread::spawn(move || simulator.run_until(&thread_shutdown));

    thread::sleep(Duration::from_millis(50));

    Ok(SimulatorThread {
        address,
        shutdown,
        handle: Some(handle),
    })
}

fn spawn_thermometer_simulator(address: String) -> Result<SimulatorThread, Box<dyn Error>> {
    let mut simulator = ThermometerSimulator::new(address, Duration::from_millis(50))?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let handle = thread::spawn(move || simulator.run_until(&thread_shutdown));

    Ok(SimulatorThread {
        address: String::new(),
        shutdown,
        handle: Some(handle),
    })
}
