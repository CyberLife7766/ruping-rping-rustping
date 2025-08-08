mod cli;
mod dns;
mod icmp;
mod stats;
mod utils;
mod netif;

use icmp::IcmpSocket;
use stats::PingStatistics;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() {
    // 启用调试日志
    if std::env::var("RUST_LOG").is_ok() { env_logger::init(); }

    // 解析参数
    let args = match cli::parse_args() { Ok(a) => a, Err(e) => { utils::exit_with_error(&format!("参数解析错误: {}", e), 1); } };

    // 参数校验
    if let Err(e) = utils::validate_ping_params(args.size, args.count, args.timeout, args.ttl) {
        utils::exit_with_error(&e.to_string(), 1);
    }

    // 构建目标集合
    let mut targets: Vec<String> = Vec::new();
    if !args.targets.is_empty() { targets.extend(args.targets.clone()); }
    if let Some(file) = &args.targets_file { targets.extend(read_targets_from_file(file)); }
    for c in &args.cidrs { targets.extend(expand_cidr_ipv4(c)); }

    // 去重并保持插入顺序
    let mut seen: HashSet<String> = HashSet::new();
    targets.retain(|t| seen.insert(t.to_string()));
    if targets.is_empty() { utils::exit_with_error("未提供任何目标。", 1); }

    // 并发解析目标为 IP
    let mut jobs: VecDeque<HostJob> = VecDeque::new();
    for t in targets {
        let prefer_v4 = args.force_ipv4;
        let prefer_v6 = args.force_ipv6;
        let ip = match t.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(_) => match dns::resolve_hostname(&t, prefer_v4, prefer_v6).await {
                Ok(ip) => ip,
                Err(e) => { eprintln!("无法解析主机名 '{}': {}", t, e); continue; }
            }
        };
        jobs.push_back(HostJob { name: t, ip, is_ipv6: ip.is_ipv6() });
    }
    if jobs.is_empty() { utils::exit_with_error("没有可用的可解析目标。", 1); }

    // 并发调度
    let concurrency = args.concurrency.min(256).max(1);
    let payload_size = args.size.unwrap_or(32) as usize;
    let timeout_ms = args.timeout.unwrap_or(4000);
    let per_host_interval = Duration::from_millis(args.interval_ms.max(1));
    let count = if args.continuous { u32::MAX } else { args.count.unwrap_or(4) };

    // 全局截止时间
    let overall_future = async {
        let mut set: JoinSet<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)> = JoinSet::new();
        let mut in_flight: usize = 0;
        let mut results: Vec<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)> = Vec::new();

        // 启动初始批次
        while in_flight < concurrency && !jobs.is_empty() {
            let job = jobs.pop_front().unwrap();
            spawn_host_task(&args, job, payload_size, timeout_ms, per_host_interval, count, &mut set).await;
            in_flight += 1;
        }

        // 轮询完成并继续派发
        while let Some(res) = set.join_next().await {
            match res {
                Ok((name, ip, stats, reps)) => { results.push((name, ip, stats, reps)); },
                Err(e) => { eprintln!("任务执行失败: {}", e); }
            }
            in_flight -= 1;
            if let Some(job) = jobs.pop_front() {
                spawn_host_task(&args, job, payload_size, timeout_ms, per_host_interval, count, &mut set).await;
                in_flight += 1;
            } else if in_flight == 0 {
                break;
            }
        }

        results
    };

    let results: Vec<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)> = if let Some(deadline_sec) = args.deadline_sec {
        match timeout(Duration::from_secs(deadline_sec), overall_future).await {
            Ok(res) => res,
            Err(_) => {
                eprintln!("达到全局截止时间，停止剩余任务。");
                // 截止超时时，已完成的结果会通过已完成 join 返回；此处返回空集合（已完成的在 Ok 分支），为简化，这里返回空
                Vec::new()
            }
        }
    } else {
        overall_future.await
    };

    // 总体汇总与输出
    let mut total = PingStatistics::new();
    for (_name, _ip, s, _) in &results { total.merge_from(s); }

    if args.json_output {
        // JSON 输出
        let mut s = build_json(&results, &total, args.include_replies, args.pretty_json);
        if let Some(path) = &args.output_path {
            if let Err(e) = fs::write(path, &s) { eprintln!("写入文件失败: {}", e); }
        } else {
            println!("{}", s);
        }
        return;
    }
    if args.csv_output {
        // CSV 输出
        let s = build_csv(&results, &total, args.include_replies, args.csv_no_headers);
        if let Some(path) = &args.output_path {
            if let Err(e) = fs::write(path, &s) { eprintln!("写入文件失败: {}", e); }
        } else {
            print!("{}", s);
        }
        return;
    }

    println!("\n===== 总体统计信息 =====");
    println!("数据包: 已发送 = {}, 已接收 = {}, 丢失 = {} ({:.0}% 丢失)",
        total.packets_sent, total.packets_received, total.packets_lost, total.loss_percentage());
    if total.packets_received > 0 {
        let min_time = if total.min_time.is_finite() { total.min_time } else { 0.0 };
        println!("往返行程的估计时间(毫秒): 最短 = {:.0}ms，最长 = {:.0}ms，平均 = {:.0}ms",
            min_time, total.max_time, total.average_time());
        println!(
            "    P50 = {:.0}ms，P90 = {:.0}ms，P99 = {:.0}ms{}",
            total.p50(), total.p90(), total.p99(),
            if total.packets_received > 1 {
                format!("，Jitter = {:.1}ms，StdDev = {:.1}ms", total.jitter(), total.std_deviation())
            } else { String::new() }
        );
    }
}

struct HostJob { name: String, ip: IpAddr, is_ipv6: bool }

async fn spawn_host_task(
    args: &cli::PingArgs,
    job: HostJob,
    payload_size: usize,
    timeout_ms: u32,
    per_host_interval: Duration,
    count: u32,
    set: &mut JoinSet<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)>,
) {
    let name = job.name.clone();
    let ip = job.ip;
    let is_ipv6 = job.is_ipv6;
    let ttl_opt = args.ttl;
    let source_addr = args.source_address;
    let iface = args.interface.clone();
    let resolve_addrs = args.resolve_addresses;
    let print_replies = !(args.summary_only || args.quiet || args.json_output || args.csv_output);
    let print_headers = !(args.summary_only || args.json_output || args.csv_output);
    let print_summaries = !(args.json_output || args.csv_output);
    let include_replies = args.include_replies;

    set.spawn(async move {
        let mut stats = PingStatistics::new();
        let identifier = utils::generate_identifier();
        let mut sequence: u16 = 1;
        let mut replies: Vec<(u16, Option<f64>, String)> = Vec::new();

        // 打印头部
        if print_headers {
            println!("{}", stats.format_header(&name, &ip.to_string(), payload_size as u32));
        }

        // 优先尝试 RAW socket
        #[allow(unused_mut)]
        let mut raw_socket = IcmpSocket::new(is_ipv6).ok();

        // 绑定与 TTL（仅 RAW 可用）
        if let Some(sock) = &raw_socket {
            if let Some(sa) = source_addr {
                if let Err(e) = sock.bind_to_interface(sa) { utils::print_warning(&format!("无法绑定到源地址 {}: {}", sa, e)); }
            } else if let Some(ifn) = &iface {
                match netif::find_source_ip_for_iface(ifn, is_ipv6) {
                    Ok(src) => { if let Err(e) = sock.bind_to_interface(src) { utils::print_warning(&format!("无法绑定到网卡 {} 的源地址 {}: {}", ifn, src, e)); } },
                    Err(e) => { utils::print_warning(&format!("根据网卡 '{}' 选择源地址失败: {}", ifn, e)); }
                }
            }
            if let Some(ttl) = ttl_opt { if let Err(e) = sock.set_ttl(ttl) { utils::print_warning(&format!("设置 {} 失败: {}", if is_ipv6 { "Hop Limit(IPv6)" } else { "TTL(IPv4)" }, e)); } }
        }

        // WinAPI 回退（仅 IPv4）
        #[cfg(windows)]
        let winapi_fallback = if raw_socket.is_none() && !is_ipv6 {
            match icmp::winapi::WinApiIcmpSocket::new() { Ok(s) => Some(s), Err(e) => { utils::print_warning(&format!("WinAPI 回退创建失败: {}", e)); None } }
        } else { None };

        for i in 0..count {
            stats.record_sent();
            let send_res = if let Some(sock) = &raw_socket {
                sock.send_ping(ip, identifier, sequence, payload_size, timeout_ms).await
            } else {
                #[cfg(windows)]
                {
                    if let Some(ws) = &winapi_fallback {
                        ws.send_ping(ip, identifier, sequence, payload_size, timeout_ms, ttl_opt).await
                    } else {
                        Err(anyhow::anyhow!("无法创建 ICMP 套接字且无可用回退"))
                    }
                }
                #[cfg(not(windows))]
                { Err(anyhow::anyhow!("无法创建 ICMP 套接字")) }
            };

            match send_res {
                Ok(response) => {
                    // RAW 的时间在 send_ping 中已计算；WinAPI 我们也返回了 time_ms
                    stats.record_received(response.time_ms);
                    if include_replies {
                        replies.push((sequence, Some(response.time_ms), "ok".to_string()));
                    }
                    if print_replies {
                        let resolved_name = if resolve_addrs { dns::reverse_lookup(response.source).await } else { None };
                        println!("{}", stats.format_response(&response, &name, resolved_name.as_deref()));
                    }
                }
                Err(e) => {
                    stats.record_lost();
                    if include_replies {
                        let out = if e.to_string().contains("timed out") { "timeout" } else { "error" };
                        replies.push((sequence, None, out.to_string()));
                    }
                    if print_replies {
                        if e.to_string().contains("timed out") { println!("请求超时。"); } else { eprintln!("错误: {}", e); }
                    }
                }
            }

            sequence = sequence.wrapping_add(1);
            // 间隔控制
            if i < count - 1 || count == u32::MAX { sleep(per_host_interval).await; }
        }

        // 每主机总结
        if print_summaries {
            println!("{}", stats.format_summary(&name));
        }
        (name, ip.to_string(), stats, replies)
    });
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn build_json(results: &Vec<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)>, total: &PingStatistics, include_replies: bool, pretty: bool) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"schema\":\"ruping-stats\",\n  \"version\":1,\n  \"hosts\":[\n");
    for (idx, (name, ip, s, reps)) in results.iter().enumerate() {
        if idx > 0 { out.push_str(",\n"); }
        out.push_str("  {\"name\":\"");
        out.push_str(&json_escape(name));
        out.push_str("\",\"ip\":\"");
        out.push_str(&json_escape(ip));
        out.push_str("\",\"stats\":{");
        out.push_str(&format!("\"sent\":{},", s.packets_sent));
        out.push_str(&format!("\"received\":{},", s.packets_received));
        out.push_str(&format!("\"lost\":{},", s.packets_lost));
        out.push_str(&format!("\"loss_pct\":{:.2},", s.loss_percentage()));
        out.push_str(&format!("\"min\":{:.3},", if s.min_time.is_finite() { s.min_time } else { 0.0 }));
        out.push_str(&format!("\"avg\":{:.3},", s.average_time()));
        out.push_str(&format!("\"max\":{:.3},", s.max_time));
        out.push_str(&format!("\"p50\":{:.3},", s.p50()));
        out.push_str(&format!("\"p90\":{:.3},", s.p90()));
        out.push_str(&format!("\"p99\":{:.3},", s.p99()));
        out.push_str(&format!("\"jitter\":{:.3},", s.jitter()));
        out.push_str(&format!("\"stddev\":{:.3}", s.std_deviation()));
        if include_replies {
            out.push_str(",\"replies\":[");
            for (i, (seq, rtt, outcome)) in reps.iter().enumerate() {
                if i > 0 { out.push(','); }
                match rtt {
                    Some(v) => {
                        out.push_str(&format!("{{\"seq\":{},\"time_ms\":{:.3},\"outcome\":\"{}\"}}", seq, v, json_escape(outcome)));
                    }
                    None => {
                        out.push_str(&format!("{{\"seq\":{},\"time_ms\":null,\"outcome\":\"{}\"}}", seq, json_escape(outcome)));
                    }
                }
            }
            out.push(']');
        }
        out.push_str("}}\n");
    }
    out.push_str("],\n  \"overall\":{");
    out.push_str(&format!("\"sent\":{},", total.packets_sent));
    out.push_str(&format!("\"received\":{},", total.packets_received));
    out.push_str(&format!("\"lost\":{},", total.packets_lost));
    out.push_str(&format!("\"loss_pct\":{:.2},", total.loss_percentage()));
    out.push_str(&format!("\"min\":{:.3},", if total.min_time.is_finite() { total.min_time } else { 0.0 }));
    out.push_str(&format!("\"avg\":{:.3},", total.average_time()));
    out.push_str(&format!("\"max\":{:.3},", total.max_time));
    out.push_str(&format!("\"p50\":{:.3},", total.p50()));
    out.push_str(&format!("\"p90\":{:.3},", total.p90()));
    out.push_str(&format!("\"p99\":{:.3},", total.p99()));
    out.push_str(&format!("\"jitter\":{:.3},", total.jitter()));
    out.push_str(&format!("\"stddev\":{:.3}", total.std_deviation()));
    out.push_str("}\n}\n");
    if !pretty {
        // 紧凑化：移除换行与多余空格
        let mut compact = String::with_capacity(out.len());
        for line in out.lines() { compact.push_str(line.trim()); }
        compact
    } else {
        out
    }
}

fn build_csv(results: &Vec<(String, String, PingStatistics, Vec<(u16, Option<f64>, String)>)>, total: &PingStatistics, include_replies: bool, no_headers: bool) -> String {
    let mut out = String::new();
    if !no_headers {
        out.push_str("scope,name,ip,sent,received,lost,loss_pct,min,avg,max,p50,p90,p99,jitter,stddev\n");
        if include_replies { out.push_str("scope,name,ip,seq,time_ms,outcome\n"); }
    }
    for (name, ip, s, reps) in results {
        out.push_str(&format!(
            "host,{},{},{},{},{},{:.2},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}\n",
            name, ip,
            s.packets_sent, s.packets_received, s.packets_lost, s.loss_percentage(),
            if s.min_time.is_finite() { s.min_time } else { 0.0 }, s.average_time(), s.max_time,
            s.p50(), s.p90(), s.p99(), s.jitter(), s.std_deviation()
        ));
        if include_replies {
            for (seq, rtt, outcome) in reps {
                match rtt {
                    Some(v) => out.push_str(&format!("reply,{},{},{},{:.3},{}\n", name, ip, seq, v, outcome)),
                    None => out.push_str(&format!("reply,{},{},{},,{}\n", name, ip, seq, outcome)),
                }
            }
        }
    }
    out.push_str(&format!(
        "overall,,,{},{},{},{:.2},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}\n",
        total.packets_sent, total.packets_received, total.packets_lost, total.loss_percentage(),
        if total.min_time.is_finite() { total.min_time } else { 0.0 }, total.average_time(), total.max_time,
        total.p50(), total.p90(), total.p99(), total.jitter(), total.std_deviation()
    ));
    out
}

fn read_targets_from_file(path: &str) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|s| s.to_string())
            .collect(),
        Err(e) => { eprintln!("读取目标文件失败 {}: {}", path, e); Vec::new() }
    }
}

fn expand_cidr_ipv4(cidr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 { return out; }
    let base: Ipv4Addr = match parts[0].parse() { Ok(ip) => ip, Err(_) => return out };
    let prefix: u32 = match parts[1].parse() { Ok(p) => p, Err(_) => return out };
    if prefix > 32 { return out; }
    let base_u32 = u32::from(base) & (!0u32 << (32 - prefix));
    let host_count = if prefix == 32 { 1 } else { 1u64 << (32 - prefix) };
    // 简单全量展开（包含网络和广播地址）
    for i in 0..host_count { out.push(Ipv4Addr::from(base_u32.wrapping_add(i as u32)).to_string()); }
    out
}
