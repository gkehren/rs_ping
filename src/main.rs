use std::env;
use rs_ping::{Pinger, PingError, parse_args};

fn main() -> Result<(), PingError> {
    let args: Vec<String> = env::args().collect();
    let opts = parse_args(&args)?;

    let mut pinger = Pinger::new(opts.target);
    if let Some(count) = opts.count {
        pinger = pinger.with_count(count);
    }
    if let Some(interval) = opts.interval {
        pinger = pinger.with_interval(interval);
    }

    println!("PING {} ({}): {} data bytes", opts.target, opts.target, 56);
    pinger.run()?;

    Ok(())
}