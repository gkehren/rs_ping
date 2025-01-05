use std::time::Duration;

#[derive(Debug, Default)]
pub struct PingStats {
    packets_sent: u32,
    packets_received: u32,
    min_rtt: Option<Duration>,
    max_rtt: Option<Duration>,
    total_rtt: Duration,
}

impl PingStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn packets_sent(&self) -> u32 { self.packets_sent }
    pub fn packets_received(&self) -> u32 { self.packets_received }
    pub fn get_min_rtt(&self) -> Option<Duration> { self.min_rtt }
    pub fn get_max_rtt(&self) -> Option<Duration> { self.max_rtt }

    pub fn record_sent(&mut self) {
        self.packets_sent += 1;
    }

    pub fn record_received(&mut self, rtt: Duration) {
        self.packets_received += 1;
        self.update_rtt(rtt);
    }

    pub fn get_loss_percetange(&self) -> f64 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        ((self.packets_sent - self.packets_received) as f64 / self.packets_sent as f64) * 100.0
    }

    pub fn update_rtt(&mut self, rtt: Duration) {
        self.min_rtt = Some(self.min_rtt.map_or(rtt, |min| min.min(rtt)));
        self.max_rtt = Some(self.max_rtt.map_or(rtt, |max| max.max(rtt)));
        self.total_rtt += rtt;
    }

    pub fn get_avg_rtt(&self) -> Option<Duration> {
        if self.packets_received > 0 {
            Some(self.total_rtt / self.packets_received)
        } else {
            None
        }
    }

    fn duration_as_ms(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    pub fn format_rtt(&self) -> String {
        match (self.min_rtt, self.get_avg_rtt(), self.max_rtt) {
            (Some(min), Some(avg), Some(max)) => {
                format!("{:.3}/{:.3}/{:.3}",
                    Self::duration_as_ms(min),
                    Self::duration_as_ms(avg),
                    Self::duration_as_ms(max)
                )
            },
            _ => String::from("---/---/--- ms")
        }
    }
}