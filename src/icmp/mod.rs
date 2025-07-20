pub mod packet;
pub mod socket;

#[cfg(windows)]
pub mod winapi;

pub use packet::*;
pub use socket::*;

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct IcmpResponse {
    pub source: IpAddr,
    pub bytes: u32,
    pub time_ms: f64,
    pub ttl: u32,
    pub sequence: u16,
}

#[derive(Debug)]
pub enum IcmpError {
    DestinationUnreachable,
    TimeExceeded,
    ParameterProblem,
    SourceQuench,
    Redirect,
    Unknown(u8),
}

impl std::fmt::Display for IcmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcmpError::DestinationUnreachable => write!(f, "Destination host unreachable"),
            IcmpError::TimeExceeded => write!(f, "Request timed out"),
            IcmpError::ParameterProblem => write!(f, "Parameter problem"),
            IcmpError::SourceQuench => write!(f, "Source quench"),
            IcmpError::Redirect => write!(f, "Redirect"),
            IcmpError::Unknown(code) => write!(f, "Unknown ICMP error: {}", code),
        }
    }
}

impl std::error::Error for IcmpError {}
