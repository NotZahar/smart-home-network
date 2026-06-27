use sh::simulators::SocketSimulator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(address) = std::env::args().nth(1) else {
        eprintln!("Usage: socket-simulator <listen-address>");
        std::process::exit(2);
    };

    let mut simulator = SocketSimulator::bind(address, 100.0, 15.0)?;
    println!("Socket simulator listening on {}", simulator.local_addr()?);
    simulator.run()?;

    Ok(())
}
