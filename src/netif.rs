use anyhow::Context;
use std::net::IpAddr;

fn is_loopback_v4(ip: &std::net::Ipv4Addr) -> bool {
    ip.octets()[0] == 127
}

fn is_link_local_v6(ip: &std::net::Ipv6Addr) -> bool {
    // fe80::/10
    let seg0 = ip.segments()[0];
    (seg0 & 0xffc0) == 0xfe80
}

/// 根据网卡名称或索引选择一个合适的源地址（按需要的协议族）
/// iface: 网卡友好名/描述/适配器名，或数值索引（ifIndex）
/// want_ipv6: 目标是否为 IPv6
pub fn find_source_ip_for_iface(iface: &str, want_ipv6: bool) -> anyhow::Result<IpAddr> {
    let adapters = ipconfig::get_adapters().context("无法获取本机网卡列表")?;

    // 尝试按索引匹配
    let want_index: Option<u32> = iface.parse::<u32>().ok();
    let iface_lower = iface.to_ascii_lowercase();

    let mut matched = adapters.into_iter().filter(|ad| {
        if let Some(idx) = want_index {
            // 当前 ipconfig 接口提供 ipv6_if_index；优先用此索引匹配
            ad.ipv6_if_index() == idx
        } else {
            ad.friendly_name().to_ascii_lowercase() == iface_lower
                || ad.description().to_ascii_lowercase() == iface_lower
                || ad.adapter_name().to_ascii_lowercase() == iface_lower
        }
    });

    let adapter = matched
        .next()
        .with_context(|| format!("未找到匹配的网卡: '{}'", iface))?;

    // 选取同协议族的单播地址
    let mut candidates: Vec<IpAddr> = adapter
        .ip_addresses()
        .iter()
        .filter(|ip| match (want_ipv6, *ip) {
            (true, IpAddr::V6(_)) => true,
            (false, IpAddr::V4(_)) => true,
            _ => false,
        })
        .cloned()
        .collect();

    // 优先选择更“可路由”的地址：
    if want_ipv6 {
        candidates.sort_by_key(|ip| match ip {
            IpAddr::V6(v6) => {
                if is_link_local_v6(v6) { 1 } else { 0 }
            }
            _ => 2,
        });
    } else {
        candidates.sort_by_key(|ip| match ip {
            IpAddr::V4(v4) => {
                if is_loopback_v4(v4) { 1 } else { 0 }
            }
            _ => 2,
        });
    }

    candidates
        .into_iter()
        .next()
        .with_context(|| format!("网卡 '{}' 未找到可用的{}地址", iface, if want_ipv6 { "IPv6" } else { "IPv4" }))
}
