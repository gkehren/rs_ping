use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PingConfig {
	count: Option<u32>,		// Number of pings to send (None = inf)
	interval: Duration,		// Interval between two pings
	timeout: Duration,		// waiting duration for reply
	ttl: u8,				// Time To Live
	packet_size: usize,		// Packet size in bytes
}

impl Default for PingConfig {
	fn default() -> Self {
		Self {
			count: None,
			interval: Duration::from_secs(1),
			timeout: Duration::from_secs(2),
			ttl: 64,
			packet_size: 56,
		}
	}
}

impl PingConfig {
	pub fn new() -> Self {
		Self::default()
	}

	// Getters
	pub fn count(&self) -> Option<u32> { self.count }
	pub fn interval(&self) -> Duration { self.interval }
	pub fn timeout(&self) -> Duration { self.timeout }
	pub fn ttl(&self) -> u8 { self.ttl }
	pub fn packet_size(&self) -> usize { self.packet_size }

	// Builder methods
	pub fn with_count(mut self, count: u32) -> Self {
		self.count = Some(count);
		self
	}

	pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}