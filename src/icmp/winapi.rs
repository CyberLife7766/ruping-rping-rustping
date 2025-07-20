// Windows ICMP API implementation as fallback
// This uses the Windows ICMP.dll which doesn't require Raw Socket privileges

use std::net::IpAddr;
use std::time::Instant;
use crate::icmp::IcmpResponse;

// Windows ICMP API structures and functions
#[repr(C)]
struct IpOptionInformation {
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: *mut u8,
}

#[repr(C)]
struct IcmpEchoReply {
    address: u32,
    status: u32,
    rtt: u32,
    data_size: u16,
    reserved: u16,
    data: *mut u8,
    options: IpOptionInformation,
}

// External Windows API functions
unsafe extern "system" {
    fn IcmpCreateFile() -> *mut std::ffi::c_void;
    fn IcmpCloseHandle(icmp_handle: *mut std::ffi::c_void) -> bool;
    fn IcmpSendEcho(
        icmp_handle: *mut std::ffi::c_void,
        destination_address: u32,
        request_data: *const u8,
        request_size: u16,
        request_options: *const IpOptionInformation,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32,
    ) -> u32;
}

pub struct WinApiIcmpSocket {
    handle: *mut std::ffi::c_void,
}

impl WinApiIcmpSocket {
    pub fn new() -> anyhow::Result<Self> {
        let handle = unsafe { IcmpCreateFile() };
        if handle.is_null() {
            return Err(anyhow::anyhow!("Failed to create ICMP handle"));
        }
        
        Ok(Self { handle })
    }
    
    pub async fn send_ping(
        &self,
        target: IpAddr,
        _identifier: u16, // Not used in Windows API
        _sequence: u16,   // Not used in Windows API
        payload_size: usize,
        timeout_ms: u32,
        ttl: Option<u32>,
    ) -> anyhow::Result<IcmpResponse> {
        // Only IPv4 is supported by Windows ICMP API
        let ipv4_addr = match target {
            IpAddr::V4(addr) => addr,
            IpAddr::V6(_) => {
                return Err(anyhow::anyhow!("IPv6 not supported by Windows ICMP API"));
            }
        };
        
        let payload = vec![0x61u8; payload_size]; // Fill with 'a' like Windows ping
        
        let options = IpOptionInformation {
            ttl: ttl.unwrap_or(128) as u8,
            tos: 0,
            flags: 0,
            options_size: 0,
            options_data: std::ptr::null_mut(),
        };
        
        // Reply buffer needs to be large enough for reply structure + data
        let reply_size = std::mem::size_of::<IcmpEchoReply>() + payload_size + 8;
        let mut reply_buffer = vec![0u8; reply_size];
        
        let start_time = Instant::now();
        
        let result = unsafe {
            IcmpSendEcho(
                self.handle,
                u32::from(ipv4_addr),
                payload.as_ptr(),
                payload.len() as u16,
                &options,
                reply_buffer.as_mut_ptr(),
                reply_size as u32,
                timeout_ms,
            )
        };
        
        let elapsed = start_time.elapsed();
        
        if result == 0 {
            return Err(anyhow::anyhow!("ICMP request failed or timed out"));
        }
        
        // Parse the reply
        let reply = unsafe { &*(reply_buffer.as_ptr() as *const IcmpEchoReply) };
        
        if reply.status != 0 {
            return Err(anyhow::anyhow!("ICMP error status: {}", reply.status));
        }
        
        Ok(IcmpResponse {
            source: target,
            bytes: payload_size as u32,
            time_ms: elapsed.as_secs_f64() * 1000.0,
            ttl: options.ttl as u32,
            sequence: 0, // Windows API doesn't provide sequence number
        })
    }
}

impl Drop for WinApiIcmpSocket {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                IcmpCloseHandle(self.handle);
            }
        }
    }
}

// Test if Windows ICMP API is available
pub fn is_winapi_available() -> bool {
    match WinApiIcmpSocket::new() {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_winapi_availability() {
        let available = is_winapi_available();
        println!("Windows ICMP API available: {}", available);
    }
    
    #[tokio::test]
    async fn test_winapi_ping() {
        if let Ok(socket) = WinApiIcmpSocket::new() {
            let target = "127.0.0.1".parse().unwrap();
            match socket.send_ping(target, 0, 0, 32, 4000, None).await {
                Ok(response) => {
                    println!("WinAPI ping successful: {:?}", response);
                }
                Err(e) => {
                    println!("WinAPI ping failed: {}", e);
                }
            }
        }
    }
}
