mod cli;
mod dns;
mod icmp;
mod stats;
mod utils;

use icmp::IcmpSocket;
use stats::PingStatistics;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Enable debug logging if RUST_LOG is set
    if std::env::var("RUST_LOG").is_ok() {
        env_logger::init();
    }

    // Parse command line arguments
    let args = match cli::parse_args() {
        Ok(args) => args,
        Err(e) => {
            utils::exit_with_error(&format!("参数解析错误: {}", e), 1);
        }
    };

    // Check administrator privileges with detailed error reporting
    if let Err(e) = utils::check_privileges_detailed() {
        utils::exit_with_error(&e.to_string(), 1);
    }

    // Validate parameters
    if let Err(e) = utils::validate_ping_params(args.size, args.count, args.timeout, args.ttl) {
        utils::exit_with_error(&e.to_string(), 1);
    }

    // Resolve target hostname
    let target_ip = match dns::resolve_hostname(&args.target, args.force_ipv4, args.force_ipv6).await {
        Ok(ip) => ip,
        Err(e) => {
            utils::exit_with_error(&format!("无法解析主机名 '{}': {}", args.target, e), 1);
        }
    };

    // Determine IP version
    let is_ipv6 = target_ip.is_ipv6();

    // Create ICMP socket
    let socket = match IcmpSocket::new(is_ipv6) {
        Ok(socket) => socket,
        Err(e) => {
            utils::exit_with_error(&format!("无法创建 ICMP 套接字: {}", e), 1);
        }
    };

    // Bind to source address if specified
    if let Some(source_addr) = args.source_address {
        if let Err(e) = socket.bind_to_interface(source_addr) {
            utils::exit_with_error(&format!("无法绑定到源地址 {}: {}", source_addr, e), 1);
        }
    }

    // Initialize statistics
    let mut stats = PingStatistics::new();
    let identifier = utils::generate_identifier();
    let payload_size = args.size.unwrap_or(32) as usize;
    let timeout_ms = args.timeout.unwrap_or(4000);

    // Print header
    println!("{}", stats.format_header(&args.target, &target_ip.to_string(), payload_size as u32));

    // Setup signal handler for Ctrl+C
    let mut shutdown_signal = utils::setup_signal_handler();

    // Main ping loop
    let mut sequence = 1u16;
    let count = args.count.unwrap_or(u32::MAX);

    for i in 0..count {
        // Check for shutdown signal
        if shutdown_signal.try_recv().is_ok() {
            break;
        }

        stats.record_sent();

        match socket.send_ping(target_ip, identifier, sequence, payload_size, timeout_ms).await {
            Ok(response) => {
                stats.record_received(response.time_ms);

                // Reverse lookup if requested
                let resolved_name = if args.resolve_addresses {
                    dns::reverse_lookup(response.source).await
                } else {
                    None
                };

                println!("{}", stats.format_response(&response, &args.target, resolved_name.as_deref()));
            }
            Err(e) => {
                stats.record_lost();
                println!("请求超时。");
                if e.to_string().contains("timed out") {
                    // This is expected for timeouts
                } else {
                    eprintln!("错误: {}", e);
                }
            }
        }

        sequence = sequence.wrapping_add(1);

        // Don't sleep after the last packet or if continuous mode is interrupted
        if !args.continuous && i < count - 1 {
            sleep(Duration::from_millis(1000)).await;
        } else if args.continuous {
            sleep(Duration::from_millis(1000)).await;
        }
    }

    // Print statistics
    println!("{}", stats.format_summary(&args.target));
}
