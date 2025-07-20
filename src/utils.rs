use std::process;

/// Generate a random identifier for ICMP packets
pub fn generate_identifier() -> u16 {
    use rand::Rng;
    rand::thread_rng().gen_range(1..=65535)
}

/// Check if the current process has administrator privileges on Windows
pub fn check_admin_privileges() -> bool {
    // On Windows, we can try to create a raw socket to check privileges
    crate::icmp::socket::check_raw_socket_privileges()
}

/// Detailed privilege check with error reporting
pub fn check_privileges_detailed() -> anyhow::Result<()> {
    // First check if we can create a raw socket
    match crate::icmp::socket::IcmpSocket::new(false) {
        Ok(_) => {
            //println!("✅ IPv4 Raw Socket权限检查通过");
            return Ok(());
        }
        Err(e) => {
            eprintln!("❌ IPv4 Raw Socket Check Failed: {}", e);
        }
    }

    // Try IPv6
    match crate::icmp::socket::IcmpSocket::new(true) {
        Ok(_) => {
            //println!("✅ IPv6 Raw Socket权限检查通过");
            return Ok(());
        }
        Err(e) => {
            eprintln!("❌ IPv6 Raw Socket Check Failed: {}", e);
        }
    }

    // If both fail, provide detailed error information
    Err(anyhow::anyhow!(
        "无法创建Raw Socket。可能的原因:\n\
        1. 需要管理员权限 - 请以管理员身份运行\n\
        2. Windows防火墙阻止 - 请检查防火墙设置\n\
        3. 杀毒软件阻止 - 请临时禁用杀毒软件\n\
        4. 组策略限制 - 请检查本地安全策略\n\
        5. 网络驱动问题 - 请更新网络驱动程序\n\n\
        请运行 diagnose_permissions.ps1 进行详细诊断"
    ))
}

/// Print error message and exit with error code
pub fn exit_with_error(message: &str, code: i32) -> ! {
    eprintln!("ruping: {}", message);
    process::exit(code);
}

/// Print warning message
pub fn print_warning(message: &str) {
    eprintln!("警告: {}", message);
}

/// Validate ping parameters
pub fn validate_ping_params(
    size: Option<u32>,
    count: Option<u32>,
    timeout: Option<u32>,
    ttl: Option<u32>,
) -> anyhow::Result<()> {
    if let Some(size) = size {
        if size > 65500 {
            return Err(anyhow::anyhow!("数据包大小过大，最大值为 65500 字节"));
        }
    }
    
    if let Some(count) = count {
        if count == 0 {
            return Err(anyhow::anyhow!("计数值必须大于 0"));
        }
        if count > 4294967295 {
            return Err(anyhow::anyhow!("计数值过大"));
        }
    }
    
    if let Some(timeout) = timeout {
        if timeout == 0 {
            return Err(anyhow::anyhow!("超时值必须大于 0"));
        }
        if timeout > 4294967295 {
            return Err(anyhow::anyhow!("超时值过大"));
        }
    }
    
    if let Some(ttl) = ttl {
        if ttl == 0 || ttl > 255 {
            return Err(anyhow::anyhow!("TTL 值必须在 1-255 范围内"));
        }
    }
    
    Ok(())
}

/// Format time duration for display
pub fn format_time(ms: f64) -> String {
    if ms < 1.0 {
        "<1ms".to_string()
    } else {
        format!("{:.0}ms", ms)
    }
}

/// Handle Ctrl+C signal for graceful shutdown
pub fn setup_signal_handler() -> tokio::sync::oneshot::Receiver<()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        let _ = tx.send(());
    });
    
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier_generation() {
        let id1 = generate_identifier();
        let id2 = generate_identifier();
        // Very unlikely to be the same
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_parameter_validation() {
        // Valid parameters
        assert!(validate_ping_params(Some(32), Some(4), Some(4000), Some(64)).is_ok());
        
        // Invalid size
        assert!(validate_ping_params(Some(70000), None, None, None).is_err());
        
        // Invalid count
        assert!(validate_ping_params(None, Some(0), None, None).is_err());
        
        // Invalid timeout
        assert!(validate_ping_params(None, None, Some(0), None).is_err());
        
        // Invalid TTL
        assert!(validate_ping_params(None, None, None, Some(0)).is_err());
        assert!(validate_ping_params(None, None, None, Some(256)).is_err());
    }
    
    #[test]
    fn test_time_formatting() {
        assert_eq!(format_time(0.5), "<1ms");
        assert_eq!(format_time(1.0), "1ms");
        assert_eq!(format_time(15.7), "16ms");
        assert_eq!(format_time(100.0), "100ms");
    }
}
