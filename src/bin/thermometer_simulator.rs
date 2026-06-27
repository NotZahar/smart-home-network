use sh::simulators::ThermometerSimulator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(config_path) = std::env::args().nth(1) else {
        eprintln!("Usage: thermometer-simulator <config-file>");
        std::process::exit(2);
    };

    let mut simulator = ThermometerSimulator::from_file(config_path)?;
    println!("Thermometer simulator started");
    simulator.run()?;

    Ok(())
}
