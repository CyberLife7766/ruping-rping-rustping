use clap::{Arg, ArgAction, Command};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct PingArgs {
    pub targets: Vec<String>,
    pub targets_file: Option<String>,
    pub cidrs: Vec<String>,
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
    pub concurrency: usize,
    pub interval_ms: u64,
    pub deadline_sec: Option<u64>,
    // 输出控制
    pub json_output: bool,
    pub csv_output: bool,
    pub summary_only: bool,
    pub quiet: bool,
    pub include_replies: bool,
    // 结构化输出控制
    pub output_path: Option<String>,
    pub pretty_json: bool,
    pub csv_no_headers: bool,
}

impl Default for PingArgs {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            targets_file: None,
            cidrs: Vec::new(),
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
            concurrency: 64,
            interval_ms: 1000,
            deadline_sec: None,
            json_output: false,
            csv_output: false,
            summary_only: false,
            quiet: false,
            include_replies: false,
            output_path: None,
            pretty_json: false,
            csv_no_headers: false,
        }
    }
}

pub fn build_cli() -> Command {
    Command::new("ruping")
        .version("0.2.0")
        .about("A Rust implementation of Windows ping command")
        .arg(
            Arg::new("target")
                .help("Target hostname or IP address (supports multiple)")
                .required(false)
                .num_args(1..)
                .value_name("TARGETS")
        )
        .arg(
            Arg::new("file")
                .long("file")
                .help("Read targets from file, one per line")
                .value_name("PATH")
        )
        .arg(
            Arg::new("cidr")
                .long("cidr")
                .help("Add targets from CIDR (IPv4), e.g. 192.168.1.0/30; can be repeated or comma-separated")
                .value_name("CIDR")
                .num_args(1..)
                .value_delimiter(',')
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
        .arg(
            Arg::new("concurrency")
                .short('P')
                .long("concurrency")
                .help("Max concurrent hosts (1-256)")
                .value_name("N")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .help("Per-host send interval in milliseconds (default 1000)")
                .value_name("ms")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("deadline")
                .long("deadline")
                .help("Global deadline in seconds")
                .value_name("sec")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output results in JSON format (suppresses per-reply printing)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .help("Output results in CSV format (suppresses per-reply printing)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("summary_only")
                .long("summary-only")
                .help("Only print per-host and overall summaries (no per-reply lines)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("quiet")
                .long("quiet")
                .help("Suppress per-reply lines (headers still printed unless --summary-only)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include_replies")
                .long("include-replies")
                .help("Include each reply details in JSON/CSV output")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_name("path")
                .help("Write JSON/CSV output to file (otherwise prints to stdout)")
        )
        .arg(
            Arg::new("pretty")
                .long("pretty")
                .help("Pretty-print JSON output (indent/newlines)")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("headers")
                .long("headers")
                .value_name("mode")
                .help("CSV headers mode: all|none (default: all)")
                .default_value("all")
        )
}

pub fn parse_args() -> anyhow::Result<PingArgs> {
    let matches = build_cli().get_matches();
    
    let mut args = PingArgs::default();
    
    if let Some(ts) = matches.get_many::<String>("target") {
        args.targets = ts.cloned().collect();
    }
    if let Some(file) = matches.get_one::<String>("file") {
        args.targets_file = Some(file.clone());
    }
    if let Some(cidr_vals) = matches.get_many::<String>("cidr") {
        args.cidrs = cidr_vals.cloned().collect();
    }
    args.continuous = matches.get_flag("continuous");
    args.resolve_addresses = matches.get_flag("resolve");
    args.dont_fragment = matches.get_flag("dont_fragment");
    args.reverse_route = matches.get_flag("reverse_route");
    args.hyper_v = matches.get_flag("hyper_v");
    args.force_ipv4 = matches.get_flag("force_ipv4");
    args.force_ipv6 = matches.get_flag("force_ipv6");
    args.json_output = matches.get_flag("json");
    args.csv_output = matches.get_flag("csv");
    args.summary_only = matches.get_flag("summary_only");
    args.quiet = matches.get_flag("quiet");
    args.include_replies = matches.get_flag("include_replies");
    if let Some(path) = matches.get_one::<String>("output") { args.output_path = Some(path.clone()); }
    args.pretty_json = matches.get_flag("pretty");
    args.csv_no_headers = matches.get_one::<String>("headers").map(|s| s == "none").unwrap_or(false);
    
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
    
    if let Some(cc) = matches.get_one::<u32>("concurrency") {
        let v = (*cc).clamp(1, 256) as usize;
        args.concurrency = v;
    }
    if let Some(iv) = matches.get_one::<u64>("interval") {
        args.interval_ms = *iv;
    }
    if let Some(dl) = matches.get_one::<u64>("deadline") {
        args.deadline_sec = Some(*dl);
    }

    // Validation
    if args.force_ipv4 && args.force_ipv6 {
        return Err(anyhow::anyhow!("Cannot force both IPv4 and IPv6"));
    }
    
    if args.continuous && args.count.is_some() {
        args.count = None; // Continuous mode overrides count
    }

    // Ensure we have at least one target source (positional, file, or cidr)
    if args.targets.is_empty() && args.targets_file.is_none() && args.cidrs.is_empty() {
        return Err(anyhow::anyhow!("No targets provided. Specify targets, --file, or --cidr."));
    }
    // Validate output
    if args.json_output && args.csv_output {
        return Err(anyhow::anyhow!("--json and --csv cannot be used together"));
    }
    
    Ok(args)
}
