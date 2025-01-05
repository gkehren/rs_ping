use std::{net::IpAddr, time::Duration};
use socket2::{Socket, Domain, Type, Protocol};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{PingConfig, PingError, PingStats};

pub struct Pinger {
    target: IpAddr,
    config: PingConfig,
    stats: PingStats,
    socket: Option<Socket>, // raw socket from socket2
    current_ping_start: Option<std::time::Instant>,
    running: Arc<AtomicBool>,
}

impl Pinger {
    pub fn new(target: IpAddr) -> Self {
        Self {
            target,
            config: PingConfig::default(),
            stats: PingStats::new(),
            socket: None,
            current_ping_start: None,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    // builder-pattern config
    pub fn with_count(mut self, count: u32) -> Self {
        self.config = self.config.with_count(count);
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.config = self.config.with_interval(interval);
        self
    }

    // socket init
    pub fn init(&mut self) -> Result<(), PingError> {
        let domain = if self.target.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        };

        let socket = Socket::new(
            domain,
            Type::RAW, // Need raw socket
            Some(Protocol::ICMPV4)
        )?;

        self.socket = Some(socket);
        Ok(())
    }

    // main method to start ping
    pub fn run(&mut self) -> Result<(), PingError> {
        self.init()?;

        let running = self.running.clone();
        ctrlc::set_handler(move || {
            running.store(false, Ordering::SeqCst);
        })?;

        let mut seq = 0;
        while self.shoud_continue(seq) && self.running.load(Ordering::SeqCst) {
            self.send_ping(seq)?;
            self.receive_pong()?;
            seq += 1;
            std::thread::sleep(self.config.interval());
        }

        self.print_statistics();
        Ok(())
    }

    fn shoud_continue(&self, seq: u32) -> bool {
        match self.config.count() {
            Some(count) => seq < count,
            None => true,
        }
    }

    fn send_ping(&mut self, seq: u32) -> Result<(), PingError> {
        let socket = self.socket.as_ref().ok_or(PingError::SocketNotInitialized)?;

        // ICMP Header
        let mut buf = vec![0u8; 8];
        buf[0] = 8; // Type: Echo Request
        buf[1] = 0; // Code
        buf[4..6].copy_from_slice(&(seq as u16).to_be_bytes()); // Sequence number

        // Checksum
        let checksum = Self::calculate_checksum(&buf);
        buf[2..4].copy_from_slice(&checksum.to_be_bytes());

        let start = std::time::Instant::now();

        // Send
        let addr = Self::make_socket_addr(self.target);
        socket.send_to(&buf, &addr)?;

        self.stats.record_sent();
        self.current_ping_start = Some(start);
        Ok(())
    }

    fn receive_pong(&mut self) -> Result<(), PingError> {
        let socket = self.socket.as_ref().ok_or(PingError::SocketNotInitialized)?;

        let mut buf = vec![std::mem::MaybeUninit::uninit(); 64];
        let (len, _addr) = socket.recv_from(&mut buf)?;
        let buf = &buf[..len];
        let buf: Vec<u8> = buf.iter().map(|x| unsafe { x.assume_init() }).collect();

        if len >= 20 + 8 {
            let icmp_type = buf[20];
            if icmp_type == 0 {
                if let Some(start) = self.current_ping_start.take() {
                    let rtt = start.elapsed();
                    self.stats.record_received(rtt);
                    println!("{} bytes from {}: icmp_seq={} ttl={} time={:.3} ms",
                        self.config.packet_size() + 8,
                        self.target,
                        u16::from_be_bytes([buf[24], buf[25]]),
                        buf[8],
                        rtt.as_secs_f64() * 1000.0,
                    );
                }
                Ok(())
            } else {
                Err(PingError::InvalidResponse)
            }
        } else {
            Err(PingError::InvalidResponse)
        }
    }

    fn print_statistics(&self) {
        println!("\n--- {} ping statistics ---", self.target);
        println!("{} packets transmitted, {} received, {}% packet loss",
            self.stats.packets_sent(),
            self.stats.packets_received(),
            self.stats.get_loss_percetange(),
        );
        println!("rtt min/avg/max = {} ms", self.stats.format_rtt());
    }

    fn calculate_checksum(buf: &[u8]) -> u16 {
        let mut sum = 0u32;
        let len = buf.len();
        let mut i = 0;

        while i < len - 1 {
            sum += ((buf[i] as u32) << 8) | (buf[i + 1] as u32);
            i += 2;
        }

        if len % 2 == 1 {
            sum += (buf[len - 1] as u32) << 8;
        }

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !sum as u16
    }

    fn make_socket_addr(addr: IpAddr) -> socket2::SockAddr {
        std::net::SocketAddr::new(addr, 0).into()
    }
}