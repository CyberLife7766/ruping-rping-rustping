use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PingStatistics {
    pub packets_sent: u32,
    pub packets_received: u32,
    pub packets_lost: u32,
    pub min_time: f64,
    pub max_time: f64,
    pub total_time: f64,
    pub start_time: Instant,
    // 采样与抖动/方差相关
    samples: Vec<f64>,
    last_time_ms: Option<f64>,
    jitter_sum: f64,
    jitter_count: u32,
}

impl PingStatistics {
    pub fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            min_time: f64::INFINITY,
            max_time: 0.0,
            total_time: 0.0,
            start_time: Instant::now(),
            samples: Vec::new(),
            last_time_ms: None,
            jitter_sum: 0.0,
            jitter_count: 0,
        }
    }
    
    pub fn record_sent(&mut self) {
        self.packets_sent += 1;
    }
    
    pub fn record_received(&mut self, time_ms: f64) {
        self.packets_received += 1;
        self.total_time += time_ms;

        if time_ms < self.min_time {
            self.min_time = time_ms;
        }

        if time_ms > self.max_time {
            self.max_time = time_ms;
        }

        // 保存样本
        self.samples.push(time_ms);
        // 更新抖动（相邻样本差的绝对值均值）
        if let Some(prev) = self.last_time_ms {
            self.jitter_sum += (time_ms - prev).abs();
            self.jitter_count += 1;
        }
        self.last_time_ms = Some(time_ms);
    }
    
    pub fn record_lost(&mut self) {
        self.packets_lost += 1;
    }
    
    pub fn loss_percentage(&self) -> f64 {
        if self.packets_sent == 0 {
            return 0.0;
        }
        (self.packets_lost as f64 / self.packets_sent as f64) * 100.0
    }
    
    pub fn average_time(&self) -> f64 {
        if self.packets_received == 0 {
            return 0.0;
        }
        self.total_time / self.packets_received as f64
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.samples.is_empty() { return 0.0; }
        let mut v = self.samples.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len();
        // 最近邻法
        let rank = (p * (n as f64 - 1.0)).round() as usize;
        v[rank]
    }

    pub fn p50(&self) -> f64 { self.percentile(0.50) }
    pub fn p90(&self) -> f64 { self.percentile(0.90) }
    pub fn p99(&self) -> f64 { self.percentile(0.99) }

    pub fn std_deviation(&self) -> f64 {
        let n = self.samples.len();
        if n < 2 { return 0.0; }
        let mean = self.average_time();
        let var = self.samples.iter().map(|&x| {
            let d = x - mean; d * d
        }).sum::<f64>() / (n as f64);
        var.sqrt()
    }

    pub fn jitter(&self) -> f64 {
        if self.jitter_count == 0 { 0.0 } else { self.jitter_sum / self.jitter_count as f64 }
    }

    // 将另一份统计合并到当前对象，用于总体聚合
    pub fn merge_from(&mut self, other: &PingStatistics) {
        self.packets_sent += other.packets_sent;
        self.packets_received += other.packets_received;
        self.packets_lost += other.packets_lost;
        if other.min_time.is_finite() { self.min_time = self.min_time.min(other.min_time); }
        self.max_time = self.max_time.max(other.max_time);
        self.total_time += other.total_time;
        // 合并样本以支持总体百分位与标准差
        if !other.samples.is_empty() {
            self.samples.extend_from_slice(&other.samples);
        }
        // 合并抖动（按样本-1 计数加权）
        self.jitter_sum += other.jitter_sum;
        self.jitter_count += other.jitter_count;
    }
    
    pub fn format_summary(&self, target: &str) -> String {
        let loss_percent = self.loss_percentage();
        
        let mut summary = format!(
            "\n{} 的 Ping 统计信息:\n    数据包: 已发送 = {}, 已接收 = {}, 丢失 = {} ({:.0}% 丢失),\n",
            target,
            self.packets_sent,
            self.packets_received,
            self.packets_lost,
            loss_percent
        );
        
        if self.packets_received > 0 {
            let min_time = if self.min_time == f64::INFINITY { 0.0 } else { self.min_time };
            summary.push_str(&format!(
                "往返行程的估计时间(以毫秒为单位):\n    最短 = {:.0}ms，最长 = {:.0}ms，平均 = {:.0}ms\n",
                min_time,
                self.max_time,
                self.average_time()
            ));

            // 高级统计
            if self.samples.len() >= 1 {
                summary.push_str(&format!(
                    "    P50 = {:.0}ms，P90 = {:.0}ms，P99 = {:.0}ms",
                    self.p50(), self.p90(), self.p99()
                ));
                if self.samples.len() >= 2 {
                    summary.push_str(&format!(
                        "，Jitter = {:.1}ms，StdDev = {:.1}ms\n",
                        self.jitter(), self.std_deviation()
                    ));
                } else {
                    summary.push('\n');
                }
            }
        }
        
        summary
    }
    
    pub fn format_response(&self, response: &crate::icmp::IcmpResponse, _target: &str, resolved_name: Option<&str>) -> String {
        let source_display = if let Some(name) = resolved_name {
            format!("{} [{}]", name, response.source)
        } else {
            response.source.to_string()
        };
        
        let time_display = if response.time_ms < 1.0 {
            "<1ms".to_string()
        } else {
            format!("{:.0}ms", response.time_ms)
        };
        
        format!(
            "来自 {} 的回复: 字节={} 时间={} TTL={}",
            source_display,
            response.bytes,
            time_display,
            response.ttl
        )
    }
    
    pub fn format_header(&self, target: &str, resolved_ip: &str, payload_size: u32) -> String {
        if target == resolved_ip {
            format!("正在 Ping {} 具有 {} 字节的数据:", target, payload_size)
        } else {
            format!("正在 Ping {} [{}] 具有 {} 字节的数据:", target, resolved_ip, payload_size)
        }
    }
}

impl Default for PingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icmp::IcmpResponse;
    use std::net::IpAddr;
    
    #[test]
    fn test_statistics_calculation() {
        let mut stats = PingStatistics::new();
        
        stats.record_sent();
        stats.record_received(10.5);
        
        stats.record_sent();
        stats.record_received(20.3);
        
        stats.record_sent();
        stats.record_lost();
        
        assert_eq!(stats.packets_sent, 3);
        assert_eq!(stats.packets_received, 2);
        assert_eq!(stats.packets_lost, 1);
        assert!((stats.loss_percentage() - 33.333333333333336).abs() < 0.0001);
        assert_eq!(stats.average_time(), 15.4);
        assert_eq!(stats.min_time, 10.5);
        assert_eq!(stats.max_time, 20.3);
    }
    
    #[test]
    fn test_response_formatting() {
        let stats = PingStatistics::new();
        let response = IcmpResponse {
            source: "8.8.8.8".parse::<IpAddr>().unwrap(),
            bytes: 32,
            time_ms: 15.7,
            ttl: 64,
            sequence: 1,
        };
        
        let formatted = stats.format_response(&response, "8.8.8.8", None);
        assert!(formatted.contains("来自 8.8.8.8 的回复"));
        assert!(formatted.contains("字节=32"));
        assert!(formatted.contains("时间=16ms"));
        assert!(formatted.contains("TTL=64"));
    }
    
    #[test]
    fn test_summary_formatting() {
        let mut stats = PingStatistics::new();
        stats.record_sent();
        stats.record_received(10.0);
        stats.record_sent();
        stats.record_lost();
        
        let summary = stats.format_summary("8.8.8.8");
        assert!(summary.contains("8.8.8.8 的 Ping 统计信息"));
        assert!(summary.contains("已发送 = 2"));
        assert!(summary.contains("已接收 = 1"));
        assert!(summary.contains("丢失 = 1"));
        assert!(summary.contains("50% 丢失"));
    }
}
