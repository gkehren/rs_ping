//! A library Rust to send ICMP Echo (ping) request

// Module declaration
mod error;
mod stats;
mod config;
mod pinger;

use std::net::IpAddr;

use dns_lookup::lookup_host;
// Public re-export of necessary elements
pub use error::PingError;
pub use stats::PingStats;
pub use config::PingConfig;
pub use pinger::Pinger;

// Custom type Result
pub type Result<T> = std::result::Result<T, PingError>;

pub fn parse_args(args: &[String]) -> Result<IpAddr> {
    // Check if we have one arg
    if args.len() != 2 {
        return Err(PingError::InvalidAddress("Invalid number of arguments".to_string()));
    }

    // try to parse ip address
    match args[1].parse::<IpAddr>() {
        Ok(ip) => {
            // check if it's a ipv4 address
            match ip {
                IpAddr::V4(_) => Ok(ip),
                IpAddr::V6(_) => Err(PingError::InvalidAddress("IPv6 isn't supported yet".to_string())),
            }
        }
        Err(_) => {
            match lookup_host(&args[1]) {
                Ok(ips) => {
                    ips.into_iter()
                        .find(|ip| ip.is_ipv4())
                        .ok_or_else(|| PingError::InvalidAddress("No IPv4 address found".to_string()))
                }
                Err(_) => Err(PingError::InvalidAddress("Could not resolve hostname".to_string())),
            }
        }
    }
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
        assert_eq!(
            result.unwrap(),
            IpAddr::from_str("127.0.0.1").unwrap()
        );
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
        assert_eq!(result.unwrap(), IpAddr::from([127, 0, 0, 1]));
    }
}