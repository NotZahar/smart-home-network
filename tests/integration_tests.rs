use std::collections::HashMap;
use std::net::{TcpListener, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use sh::home::{Home, SmartHome};
use sh::make_room;
use sh::report::Report;
use sh::room::Room;
use sh::simulators::{SocketSimulator, ThermometerSimulator, ThermometerSimulatorConfig};
use sh::smart_device::{
    Device, EmulatedPowerSocket, EmulatedThermometer, NetworkPowerSocket, NetworkThermometer,
    SocketState,
};

#[test]
fn emulated_socket_switches_state_and_power() {
    let socket = EmulatedPowerSocket::new(120.0);

    assert_eq!(socket.get_state().unwrap(), SocketState::Off);
    assert_eq!(socket.get_power().unwrap(), 0.0);

    socket.turn_on().unwrap();

    assert_eq!(socket.get_state().unwrap(), SocketState::On);
    assert_eq!(socket.get_power().unwrap(), 120.0);

    socket.turn_off().unwrap();

    assert_eq!(socket.get_state().unwrap(), SocketState::Off);
}

#[test]
fn network_socket_works_with_nonblocking_simulator() {
    let simulator_thread = start_socket_simulator();
    let first_client = NetworkPowerSocket::new(simulator_thread.address.clone());
    let second_client = NetworkPowerSocket::new(simulator_thread.address.clone());

    first_client.turn_on().unwrap();

    assert_eq!(second_client.get_state().unwrap(), SocketState::On);
    assert_eq!(first_client.get_power().unwrap(), 115.0);

    second_client.turn_off().unwrap();

    assert_eq!(first_client.get_state().unwrap(), SocketState::Off);
    assert_eq!(second_client.get_power().unwrap(), 0.0);
}

#[test]
fn network_socket_reports_connection_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    drop(listener);

    let socket = NetworkPowerSocket::with_timeout(address, Duration::from_millis(100));

    assert!(socket.get_power().is_err());
}

#[test]
fn network_thermometer_returns_last_udp_temperature() {
    let thermometer = NetworkThermometer::bind("127.0.0.1:0").unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    sender
        .send_to(b"24.75\n", thermometer.local_addr())
        .unwrap();

    assert_eq!(
        thermometer
            .wait_for_temperature(Duration::from_secs(1))
            .unwrap(),
        24.75
    );
}

#[test]
fn thermometer_simulator_sends_temperatures_to_network_thermometer() {
    let thermometer = NetworkThermometer::bind("127.0.0.1:0").unwrap();
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let address = thermometer.local_addr().to_string();
    let handle = thread::spawn(move || {
        let mut simulator = ThermometerSimulator::new(address, Duration::from_millis(20)).unwrap();
        simulator.run_until(&thread_shutdown).unwrap();
    });

    let temperature = thermometer
        .wait_for_temperature(Duration::from_secs(1))
        .unwrap();

    shutdown.store(true, Ordering::Relaxed);
    handle.join().unwrap();

    assert!((20.0..=30.0).contains(&temperature));
}

#[test]
fn thermometer_config_is_read_from_key_value_file() {
    let config = ThermometerSimulatorConfig::parse(
        "
        # comments are ignored
        target_addr = 127.0.0.1:5000
        period_ms = 250
        ",
    )
    .unwrap();

    assert_eq!(config.target_addr, "127.0.0.1:5000");
    assert_eq!(config.period, Duration::from_millis(250));
}

#[test]
fn smart_home_report_contains_device_data() {
    let socket = EmulatedPowerSocket::new(90.0);
    let thermometer = EmulatedThermometer::new(21.5);

    socket.turn_on().unwrap();

    let room = make_room!(
        "socket" => socket,
        "thermometer" => thermometer,
    );
    let home = SmartHome::new(HashMap::from([("living".to_string(), room)]));
    let report = home.report().unwrap();

    assert!(report.contains("Home"));
    assert!(report.contains("socket"));
    assert!(report.contains("90.00"));
    assert!(report.contains("21.50"));
}

#[test]
fn room_and_home_manage_devices() {
    let mut room = make_room!("socket" => EmulatedPowerSocket::new(50.0));
    let mut home = SmartHome::new(HashMap::new());

    assert!(room.get_device("socket").is_some());

    room.add_device(
        "thermometer".to_string(),
        EmulatedThermometer::new(22.0).into(),
    );
    room.remove_device("socket");

    assert!(room.get_device("socket").is_none());
    assert!(room.get_device("thermometer").is_some());

    home.add_room("office".to_string(), room);

    assert!(home.get_room("office").is_some());
    assert!(home.get_device("office", "thermometer").is_ok());
    assert!(home.get_device("missing", "thermometer").is_err());
    assert!(home.get_device("office", "missing").is_err());
}

#[test]
fn device_from_conversions_select_expected_variants() {
    let socket: Device = EmulatedPowerSocket::new(10.0).into();
    let thermometer: Device = EmulatedThermometer::new(20.0).into();

    assert!(matches!(socket, Device::EmulatedSocket(_)));
    assert!(matches!(thermometer, Device::EmulatedThermometer(_)));
}

struct SocketSimulatorThread {
    address: String,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<sh::DeviceResult<()>>>,
}

impl Drop for SocketSimulatorThread {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            handle.join().unwrap().unwrap();
        }
    }
}

fn start_socket_simulator() -> SocketSimulatorThread {
    let mut simulator = SocketSimulator::bind("127.0.0.1:0", 100.0, 15.0).unwrap();
    let address = simulator.local_addr().unwrap().to_string();
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = Arc::clone(&shutdown);
    let handle = thread::spawn(move || simulator.run_until(&thread_shutdown));

    thread::sleep(Duration::from_millis(50));

    SocketSimulatorThread {
        address,
        shutdown,
        handle: Some(handle),
    }
}
