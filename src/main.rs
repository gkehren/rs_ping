use std::env;
use std::time::Duration;
use rs_ping::{Pinger, PingError, parse_args};

fn main() -> Result<(), PingError> {
    let args: Vec<String> = env::args().collect();

    let ip = parse_args(&args).map_err(|e| PingError::InvalidAddress(e.to_string()))?;

    let mut pinger = Pinger::new(ip)
    .with_count(4)
    .with_interval(Duration::from_secs(1));
    println!("PING {} ({}): {} data bytes", ip, ip, 56);
    pinger.run()?;

    Ok(())
}