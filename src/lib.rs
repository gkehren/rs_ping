//! A library Rust to send ICMP Echo (ping) request

// Module declaration
mod error;
mod stats;
mod config;
mod pinger;

use std::{net::IpAddr, time::Duration};

use dns_lookup::lookup_host;
// Public re-export of necessary elements
pub use error::PingError;
pub use stats::PingStats;
pub use config::PingConfig;
pub use pinger::Pinger;

// Custom type Result
pub type Result<T> = std::result::Result<T, PingError>;

pub struct PingOpts {
    pub target: IpAddr,
    pub count: Option<u32>,
    pub interval: Option<Duration>,
}

pub fn parse_args(args: &[String]) -> Result<PingOpts> {
    // Check if we have one arg
    if args.len() < 2 {
        return Err(PingError::InvalidAddress("Invalid number of arguments".to_string()));
    }

    let mut opts = PingOpts {
        target: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        count: None,
        interval: None,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                i += 1;
                if i >= args.len() {
                    return Err(PingError::InvalidAddress("Missing count value".to_string()));
                }
                opts.count = Some(args[i].parse().map_err(|_| PingError::InvalidAddress("Invalid count value".to_string()))?);
            }
            "-i" => {
                i += 1;
                if i >= args.len() {
                    return Err(PingError::InvalidAddress("Missing interval value".to_string()));
                }
                let secs: f64 = args[i].parse().map_err(|_| PingError::InvalidAddress("Invalid interval value".to_string()))?;
                opts.interval = Some(Duration::from_secs_f64(secs));
            }
            arg => {
                // Parse target (IP or hostname)
                match arg.parse::<IpAddr>() {
                    Ok(ip) => match ip {
                            IpAddr::V4(_) => opts.target = ip,
                            IpAddr::V6(_) => return Err(PingError::InvalidAddress("IPv6 isn't supported yet".to_string())),
                    }
                    Err(_) => {
                        match lookup_host(arg) {
                            Ok(ips) => {
                                opts.target = ips.into_iter()
                                    .find(|ip| ip.is_ipv4())
                                    .ok_or_else(|| PingError::InvalidAddress("No IPv4 address found".to_string()))?;
                            }
                            Err(_) => return Err(PingError::InvalidAddress("Could not resolve hostname".to_string())),
                        }
                    }
                }
            }
        }
        i += 1;
    }

    Ok(opts)
}

// integration test
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::time::Duration;

    #[test]
    fn test_ping_config() {
        let config = PingConfig::default()
            .with_count(4)
            .with_interval(Duration::from_secs(1));

        assert_eq!(config.count(), Some(4));
        assert_eq!(config.interval(), Duration::from_secs(1));
    }

    #[test]
    fn test_parse_valid_ipv4() {
        let args = vec![
            String::from("program"),
            String::from("127.0.0.1"),
        ];
        let result = crate::parse_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().target, IpAddr::from_str("127.0.0.1").unwrap());
    }

    #[test]
    fn test_parse_invalid_ip() {
        let args = vec![
            String::from("program"),
            String::from("invalid-ip"),
        ];
        let result = crate::parse_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ipv6() {
        let args = vec![
            String::from("program"),
            String::from("::1"),
        ];
        let result = crate::parse_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_ping_statistics() {
        let mut stats = PingStats::new();

        // Test initial state
        assert_eq!(stats.packets_sent(), 0);
        assert_eq!(stats.packets_received(), 0);

        // Test packet recording
        stats.record_sent();
        assert_eq!(stats.packets_sent(), 1);

        // Test RTT recording
        let rtt = Duration::from_millis(100);
        stats.record_received(rtt);
        assert_eq!(stats.packets_received(), 1);
        assert_eq!(stats.get_min_rtt(), Some(rtt));
        assert_eq!(stats.get_max_rtt(), Some(rtt));
    }

    #[test]
    fn test_packet_loss_calculation() {
        let mut stats = PingStats::new();

        // Send 4 packets, receive 3
        for _ in 0..4 {
            stats.record_sent();
        }
        for _ in 0..3 {
            stats.record_received(Duration::from_millis(100));
        }

        assert_eq!(stats.get_loss_percetange(), 25.0);
    }

    #[test]
    fn test_ping_error_handling() {
        let invalid_ip = "256.256.256.256";
        let args = vec![
            String::from("program"),
            String::from(invalid_ip),
        ];

        match parse_args(&args) {
            Ok(_) => panic!("Should have failed with invalid IP"),
            Err(e) => match e {
                PingError::InvalidAddress(msg) => assert!(msg.contains("Could not resolve hostname")),
                _ => panic!("Wrong error variant"),
            },
        }
    }

    #[test]
    fn ttest_resolve_hostname() {
        let args = vec![
            String::from("program"),
            String::from("localhost"),
        ];
        let result = crate::parse_args(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().target, IpAddr::from_str("127.0.0.1").unwrap());
    }

    #[test]
    fn test_parse_args_with_options() {
        let args = vec![
            String::from("program"),
            String::from("-c"),
            String::from("5"),
            String::from("-i"),
            String::from("2"),
            String::from("8.8.8.8"),
        ];
        let opts = parse_args(&args).unwrap();
        assert_eq!(opts.count, Some(5));
        assert_eq!(opts.interval, Some(Duration::from_secs(2)));
        assert_eq!(opts.target, IpAddr::from_str("8.8.8.8").unwrap());
    }

    #[test]
    fn test_parse_args_missing_value() {
        let args = vec![
            String::from("program"),
            String::from("-c"),
        ];
        let result = parse_args(&args);
        assert!(result.is_err());
    }
}