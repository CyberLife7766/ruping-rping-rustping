use dns_lookup::{lookup_host, lookup_addr};
use std::net::IpAddr;

pub async fn resolve_hostname(hostname: &str, force_ipv4: bool, force_ipv6: bool) -> anyhow::Result<IpAddr> {
    // First try to parse as IP address
    if let Ok(ip) = hostname.parse::<IpAddr>() {
        // Validate IP version constraints
        match (ip, force_ipv4, force_ipv6) {
            (IpAddr::V4(_), false, true) => {
                return Err(anyhow::anyhow!("IPv4 address provided but IPv6 forced"));
            }
            (IpAddr::V6(_), true, false) => {
                return Err(anyhow::anyhow!("IPv6 address provided but IPv4 forced"));
            }
            _ => return Ok(ip),
        }
    }
    
    // Perform DNS lookup
    let addresses = tokio::task::spawn_blocking({
        let hostname = hostname.to_string();
        move || lookup_host(&hostname)
    }).await??;
    
    if addresses.is_empty() {
        return Err(anyhow::anyhow!("No addresses found for hostname: {}", hostname));
    }
    
    // Filter addresses based on IP version preference
    let filtered_addresses: Vec<IpAddr> = addresses
        .into_iter()
        .filter(|addr| {
            match (addr, force_ipv4, force_ipv6) {
                (IpAddr::V4(_), false, true) => false,  // IPv4 but IPv6 forced
                (IpAddr::V6(_), true, false) => false,  // IPv6 but IPv4 forced
                _ => true,
            }
        })
        .collect();
    
    if filtered_addresses.is_empty() {
        let version = if force_ipv4 { "IPv4" } else { "IPv6" };
        return Err(anyhow::anyhow!("No {} addresses found for hostname: {}", version, hostname));
    }
    
    // Prefer IPv4 by default (Windows ping behavior)
    let preferred_address = if !force_ipv6 {
        filtered_addresses.iter().find(|addr| addr.is_ipv4()).copied()
            .or_else(|| filtered_addresses.first().copied())
    } else {
        filtered_addresses.iter().find(|addr| addr.is_ipv6()).copied()
            .or_else(|| filtered_addresses.first().copied())
    };
    
    preferred_address.ok_or_else(|| anyhow::anyhow!("No suitable address found"))
}

pub async fn reverse_lookup(ip: IpAddr) -> Option<String> {
    tokio::task::spawn_blocking(move || {
        lookup_addr(&ip).ok()
    }).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ip_address_parsing() {
        let result = resolve_hostname("8.8.8.8", false, false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8.8.8.8".parse::<IpAddr>().unwrap());
    }
    
    #[tokio::test]
    async fn test_hostname_resolution() {
        let result = resolve_hostname("google.com", false, false).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_ipv4_force() {
        let result = resolve_hostname("google.com", true, false).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ipv4());
    }
    
    #[tokio::test]
    async fn test_reverse_lookup() {
        let ip = "8.8.8.8".parse::<IpAddr>().unwrap();
        let result = reverse_lookup(ip).await;
        // This may or may not succeed depending on DNS configuration
        println!("Reverse lookup result: {:?}", result);
    }
}
