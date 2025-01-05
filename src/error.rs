use std::fmt;
use std::io;

#[derive(Debug)]
pub enum PingError {
    SocketError(io::Error),
    InvalidAddress(String),
    Timeout,
    PacketError(String),
    SocketNotInitialized,
    InvalidResponse,
    SignalError(String),
}

impl fmt::Display for PingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PingError::SocketError(e) => write!(f, "Socket error: {}", e),
            PingError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            PingError::Timeout => write!(f, "Timeout"),
            PingError::PacketError(e) => write!(f, "Packet error: {}", e),
            PingError::SocketNotInitialized => write!(f, "Socket not initialized"),
            PingError::InvalidResponse => write!(f, "Invalid response"),
            PingError::SignalError(e) => write!(f, "Signal error: {}", e),
        }
    }
}

impl std::error::Error for PingError {}

impl From<io::Error> for PingError {
    fn from(error: io::Error) -> Self {
        PingError::SocketError(error)
    }
}

impl From<ctrlc::Error> for PingError {
    fn from(error: ctrlc::Error) -> Self {
        PingError::SignalError(error.to_string())
    }
}