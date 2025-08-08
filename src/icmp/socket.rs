use socket2::{Domain, Protocol, Socket, Type};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use crate::icmp::{IcmpPacket, IcmpResponse};

pub struct IcmpSocket {
    socket: Socket,
    is_ipv6: bool,
}

impl IcmpSocket {
    pub fn new(is_ipv6: bool) -> anyhow::Result<Self> {
        let domain = if is_ipv6 { Domain::IPV6 } else { Domain::IPV4 };
        let protocol = if is_ipv6 { Protocol::ICMPV6 } else { Protocol::ICMPV4 };
        
        let socket = Socket::new(domain, Type::RAW, Some(protocol))
            .map_err(|e| anyhow::anyhow!("Failed to create raw socket: {}. Administrator privileges may be required.", e))?;
        
        // Set socket options
        socket.set_nonblocking(false)?; // Use blocking mode for simplicity

        // Note: On Windows, Raw ICMP sockets should NOT include IP header
        // The OS will handle the IP header automatically
        
        Ok(Self { socket, is_ipv6 })
    }
    
    pub async fn send_ping(
        &self,
        target: IpAddr,
        identifier: u16,
        sequence: u16,
        payload_size: usize,
        timeout_ms: u32,
    ) -> anyhow::Result<IcmpResponse> {
        let packet = IcmpPacket::new_echo_request(identifier, sequence, payload_size, self.is_ipv6);
        let packet_bytes = packet.to_bytes();
        
        let target_addr = match target {
            IpAddr::V4(addr) => SocketAddr::new(IpAddr::V4(addr), 0),
            IpAddr::V6(addr) => SocketAddr::new(IpAddr::V6(addr), 0),
        };
        
        let start_time = Instant::now();

        // Send the packet
        log::debug!("Sending ICMP packet to {}: {} bytes", target, packet_bytes.len());
        self.socket.send_to(&packet_bytes, &target_addr.into())?;
        log::debug!("ICMP packet sent successfully");
        
        // Wait for response with timeout
        let timeout_duration = Duration::from_millis(timeout_ms as u64);
        
        match timeout(timeout_duration, self.receive_response(identifier, sequence)).await {
            Ok(Ok(response)) => {
                let elapsed = start_time.elapsed();
                Ok(IcmpResponse {
                    source: response.source,
                    bytes: payload_size as u32,
                    time_ms: elapsed.as_secs_f64() * 1000.0,
                    ttl: response.ttl,
                    sequence,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Request timed out")),
        }
    }
    
    async fn receive_response(&self, expected_id: u16, expected_seq: u16) -> anyhow::Result<IcmpResponse> {
        use std::mem::MaybeUninit;

        loop {
            // Use tokio's async socket operations
            let socket_ref = &self.socket;
            let (bytes_received, source_addr, buffer_data) = tokio::task::spawn_blocking({
                let socket = socket_ref.try_clone()?;
                move || {
                    log::debug!("Waiting for ICMP response...");
                    let mut buffer: [MaybeUninit<u8>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
                    let result = socket.recv_from(&mut buffer)?;
                    log::debug!("Received {} bytes from {}", result.0, result.1.as_socket().map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()));
                    // Convert MaybeUninit to initialized data
                    let mut data = vec![0u8; result.0];
                    for i in 0..result.0 {
                        data[i] = unsafe { buffer[i].assume_init() };
                    }
                    Ok::<(usize, socket2::SockAddr, Vec<u8>), std::io::Error>((result.0, result.1, data))
                }
            }).await??;

            let source_ip = match source_addr.as_socket() {
                Some(addr) => addr.ip(),
                None => continue,
            };

            // Parse the received packet
            // On Windows, Raw ICMP socket may or may not include IP header
            // Try both with and without IP header offset
            log::debug!("Analyzing packet: {} bytes, first byte: 0x{:02x}", bytes_received, buffer_data[0]);

            let icmp_data = if bytes_received >= 28 && buffer_data[0] >> 4 == 4 {
                // Looks like we have an IP header (version 4)
                log::debug!("Packet includes IP header, skipping 20 bytes");
                &buffer_data[20..bytes_received] // Skip 20-byte IPv4 header
            } else if bytes_received >= 8 {
                // No IP header, direct ICMP data
                log::debug!("Packet is direct ICMP data");
                &buffer_data[0..bytes_received]
            } else {
                log::debug!("Packet too short: {} bytes", bytes_received);
                continue; // Packet too short
            };

            log::debug!("ICMP data: {} bytes, type: {}, code: {}",
                       icmp_data.len(),
                       if icmp_data.len() > 0 { icmp_data[0] } else { 0 },
                       if icmp_data.len() > 1 { icmp_data[1] } else { 0 });

            match IcmpPacket::from_bytes(icmp_data) {
                Ok(packet) => {
                    if packet.is_echo_reply(self.is_ipv6)
                        && packet.identifier == expected_id
                        && packet.sequence == expected_seq {

                        let ttl = if self.is_ipv6 {
                            64 // Default for IPv6, would need to parse hop limit from IPv6 header
                        } else if bytes_received >= 28 && buffer_data[0] >> 4 == 4 {
                            buffer_data[8] // TTL field in IPv4 header
                        } else {
                            64 // Default TTL when IP header not available
                        };

                        return Ok(IcmpResponse {
                            source: source_ip,
                            bytes: packet.payload.len() as u32,
                            time_ms: 0.0, // Will be calculated by caller
                            ttl: ttl as u32,
                            sequence: packet.sequence,
                        });
                    }
                }
                Err(_) => continue, // Invalid packet, keep listening
            }
        }
    }
    
    pub fn set_ttl(&self, ttl: u32) -> anyhow::Result<()> {
        if self.is_ipv6 {
            // IPv6 uses unicast hop limit
            self.socket
                .set_unicast_hops_v6(ttl)
                .map_err(|e| anyhow::anyhow!("Failed to set IPv6 hop limit: {}", e))?;
        } else {
            // IPv4 TTL
            self.socket
                .set_ttl(ttl)
                .map_err(|e| anyhow::anyhow!("Failed to set IPv4 TTL: {}", e))?;
        }
        Ok(())
    }
    
    pub fn bind_to_interface(&self, source_addr: IpAddr) -> anyhow::Result<()> {
        let bind_addr = match source_addr {
            IpAddr::V4(addr) => SocketAddr::new(IpAddr::V4(addr), 0),
            IpAddr::V6(addr) => SocketAddr::new(IpAddr::V6(addr), 0),
        };
        
        self.socket.bind(&bind_addr.into())?;
        Ok(())
    }
}

// Helper function to check if raw socket privileges are available
pub fn check_raw_socket_privileges() -> bool {
    match IcmpSocket::new(false) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_raw_socket_creation() {
        // This test may fail without administrator privileges
        match IcmpSocket::new(false) {
            Ok(_) => println!("Raw socket creation successful"),
            Err(e) => println!("Raw socket creation failed: {}", e),
        }
    }
    
    #[test]
    fn test_privilege_check() {
        let has_privileges = check_raw_socket_privileges();
        println!("Has raw socket privileges: {}", has_privileges);
    }
}
