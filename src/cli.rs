use clap::{Arg, ArgAction, Command};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct PingArgs {
    pub target: String,
    pub continuous: bool,
    pub resolve_addresses: bool,
    pub count: Option<u32>,
    pub size: Option<u32>,
    pub dont_fragment: bool,
    pub ttl: Option<u32>,
    pub tos: Option<u32>,
    pub record_route: Option<u32>,
    pub timestamp: Option<u32>,
    pub loose_source_route: Option<Vec<String>>,
    pub strict_source_route: Option<Vec<String>>,
    pub timeout: Option<u32>,
    pub reverse_route: bool,
    pub source_address: Option<IpAddr>,
    pub interface: Option<String>,
    pub compartment: Option<u32>,
    pub hyper_v: bool,
    pub force_ipv4: bool,
    pub force_ipv6: bool,
}

impl Default for PingArgs {
    fn default() -> Self {
        Self {
            target: String::new(),
            continuous: false,
            resolve_addresses: false,
            count: Some(4), // Windows default
            size: Some(32), // Windows default
            dont_fragment: false,
            ttl: None,
            tos: None,
            record_route: None,
            timestamp: None,
            loose_source_route: None,
            strict_source_route: None,
            timeout: Some(4000), // Windows default 4 seconds
            reverse_route: false,
            source_address: None,
            interface: None,
            compartment: None,
            hyper_v: false,
            force_ipv4: false,
            force_ipv6: false,
        }
    }
}

pub fn build_cli() -> Command {
    Command::new("ruping")
        .version("0.1.0")
        .about("A Rust implementation of Windows ping command")
        .arg(
            Arg::new("target")
                .help("Target hostname or IP address")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("continuous")
                .short('t')
                .help("Ping the specified host until stopped")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("resolve")
                .short('a')
                .help("Resolve addresses to hostnames")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("count")
                .short('n')
                .help("Number of echo requests to send")
                .value_name("count")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("size")
                .short('l')
                .help("Send buffer size")
                .value_name("size")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("dont_fragment")
                .short('f')
                .help("Set Don't Fragment flag in packet (IPv4-only)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ttl")
                .short('i')
                .help("Time To Live")
                .value_name("TTL")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("tos")
                .short('v')
                .help("Type Of Service (IPv4-only, deprecated)")
                .value_name("TOS")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("record_route")
                .short('r')
                .help("Record route for count hops (IPv4-only)")
                .value_name("count")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("timestamp")
                .short('s')
                .help("Timestamp for count hops (IPv4-only)")
                .value_name("count")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("loose_source_route")
                .short('j')
                .help("Loose source route along host-list (IPv4-only)")
                .value_name("host-list")
                .value_delimiter(',')
        )
        .arg(
            Arg::new("strict_source_route")
                .short('k')
                .help("Strict source route along host-list (IPv4-only)")
                .value_name("host-list")
                .value_delimiter(',')
        )
        .arg(
            Arg::new("timeout")
                .short('w')
                .help("Timeout in milliseconds to wait for each reply")
                .value_name("timeout")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("reverse_route")
                .short('R')
                .help("Test reverse route also (IPv6-only)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("source_address")
                .short('S')
                .help("Source address to use")
                .value_name("srcaddr")
                .value_parser(clap::value_parser!(IpAddr))
        )
        .arg(
            Arg::new("interface")
                .long("iface")
                .help("Network interface name or index to bind as source")
                .value_name("iface")
        )
        .arg(
            Arg::new("compartment")
                .short('c')
                .help("Routing compartment identifier")
                .value_name("compartment")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("hyper_v")
                .short('p')
                .help("Ping a Hyper-V Network Virtualization provider address")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("force_ipv4")
                .short('4')
                .help("Force using IPv4")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("force_ipv6")
                .short('6')
                .help("Force using IPv6")
                .action(ArgAction::SetTrue)
        )
}

pub fn parse_args() -> anyhow::Result<PingArgs> {
    let matches = build_cli().get_matches();
    
    let mut args = PingArgs::default();
    
    args.target = matches.get_one::<String>("target").unwrap().clone();
    args.continuous = matches.get_flag("continuous");
    args.resolve_addresses = matches.get_flag("resolve");
    args.dont_fragment = matches.get_flag("dont_fragment");
    args.reverse_route = matches.get_flag("reverse_route");
    args.hyper_v = matches.get_flag("hyper_v");
    args.force_ipv4 = matches.get_flag("force_ipv4");
    args.force_ipv6 = matches.get_flag("force_ipv6");
    
    if let Some(count) = matches.get_one::<u32>("count") {
        args.count = Some(*count);
    }
    
    if let Some(size) = matches.get_one::<u32>("size") {
        args.size = Some(*size);
    }
    
    if let Some(ttl) = matches.get_one::<u32>("ttl") {
        args.ttl = Some(*ttl);
    }
    
    if let Some(tos) = matches.get_one::<u32>("tos") {
        args.tos = Some(*tos);
    }
    
    if let Some(record_route) = matches.get_one::<u32>("record_route") {
        args.record_route = Some(*record_route);
    }
    
    if let Some(timestamp) = matches.get_one::<u32>("timestamp") {
        args.timestamp = Some(*timestamp);
    }
    
    if let Some(timeout) = matches.get_one::<u32>("timeout") {
        args.timeout = Some(*timeout);
    }
    
    if let Some(source_address) = matches.get_one::<IpAddr>("source_address") {
        args.source_address = Some(*source_address);
    }

    if let Some(iface) = matches.get_one::<String>("interface") {
        args.interface = Some(iface.clone());
    }
    
    if let Some(compartment) = matches.get_one::<u32>("compartment") {
        args.compartment = Some(*compartment);
    }
    
    if let Some(hosts) = matches.get_many::<String>("loose_source_route") {
        args.loose_source_route = Some(hosts.cloned().collect());
    }
    
    if let Some(hosts) = matches.get_many::<String>("strict_source_route") {
        args.strict_source_route = Some(hosts.cloned().collect());
    }
    
    // Validation
    if args.force_ipv4 && args.force_ipv6 {
        return Err(anyhow::anyhow!("Cannot force both IPv4 and IPv6"));
    }
    
    if args.continuous && args.count.is_some() {
        args.count = None; // Continuous mode overrides count
    }
    
    Ok(args)
}
