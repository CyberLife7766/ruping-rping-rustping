use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Cursor, Read};

pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMPV6_ECHO_REQUEST: u8 = 128;
pub const ICMPV6_ECHO_REPLY: u8 = 129;

#[derive(Debug, Clone)]
pub struct IcmpPacket {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: Vec<u8>,
}

impl IcmpPacket {
    pub fn new_echo_request(identifier: u16, sequence: u16, payload_size: usize, is_ipv6: bool) -> Self {
        let icmp_type = if is_ipv6 { ICMPV6_ECHO_REQUEST } else { ICMP_ECHO_REQUEST };
        let payload = vec![0x61; payload_size]; // Fill with 'a' characters like Windows ping
        
        let mut packet = Self {
            icmp_type,
            code: 0,
            checksum: 0,
            identifier,
            sequence,
            payload,
        };
        
        packet.calculate_checksum();
        packet
    }
    
    pub fn from_bytes(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 8 {
            return Err(anyhow::anyhow!("ICMP packet too short"));
        }
        
        let mut cursor = Cursor::new(data);
        let icmp_type = cursor.read_u8()?;
        let code = cursor.read_u8()?;
        let checksum = cursor.read_u16::<BigEndian>()?;
        let identifier = cursor.read_u16::<BigEndian>()?;
        let sequence = cursor.read_u16::<BigEndian>()?;
        
        let mut payload = Vec::new();
        cursor.read_to_end(&mut payload)?;
        
        Ok(Self {
            icmp_type,
            code,
            checksum,
            identifier,
            sequence,
            payload,
        })
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.payload.len());
        bytes.write_u8(self.icmp_type).unwrap();
        bytes.write_u8(self.code).unwrap();
        bytes.write_u16::<BigEndian>(self.checksum).unwrap();
        bytes.write_u16::<BigEndian>(self.identifier).unwrap();
        bytes.write_u16::<BigEndian>(self.sequence).unwrap();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
    
    pub fn calculate_checksum(&mut self) {
        self.checksum = 0;
        let bytes = self.to_bytes();
        self.checksum = Self::compute_checksum(&bytes);
    }
    
    fn compute_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        
        // Sum all 16-bit words
        while i < data.len() - 1 {
            let word = ((data[i] as u16) << 8) | (data[i + 1] as u16);
            sum += word as u32;
            i += 2;
        }
        
        // Add the last byte if the length is odd
        if i < data.len() {
            sum += (data[i] as u32) << 8;
        }
        
        // Add the carry
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        // One's complement
        !sum as u16
    }
    
    pub fn is_echo_reply(&self, is_ipv6: bool) -> bool {
        let expected_type = if is_ipv6 { ICMPV6_ECHO_REPLY } else { ICMP_ECHO_REPLY };
        self.icmp_type == expected_type
    }
    
    pub fn verify_checksum(&self) -> bool {
        let bytes = self.to_bytes();
        Self::compute_checksum(&bytes) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_icmp_packet_creation() {
        let packet = IcmpPacket::new_echo_request(1234, 1, 32, false);
        assert_eq!(packet.icmp_type, ICMP_ECHO_REQUEST);
        assert_eq!(packet.code, 0);
        assert_eq!(packet.identifier, 1234);
        assert_eq!(packet.sequence, 1);
        assert_eq!(packet.payload.len(), 32);
    }
    
    #[test]
    fn test_icmp_packet_serialization() {
        let packet = IcmpPacket::new_echo_request(1234, 1, 32, false);
        let bytes = packet.to_bytes();
        let parsed = IcmpPacket::from_bytes(&bytes).unwrap();
        
        assert_eq!(packet.icmp_type, parsed.icmp_type);
        assert_eq!(packet.code, parsed.code);
        assert_eq!(packet.identifier, parsed.identifier);
        assert_eq!(packet.sequence, parsed.sequence);
        assert_eq!(packet.payload, parsed.payload);
    }
    
    #[test]
    fn test_checksum_calculation() {
        let mut packet = IcmpPacket::new_echo_request(1234, 1, 32, false);
        packet.calculate_checksum();
        assert!(packet.verify_checksum());
    }
}
