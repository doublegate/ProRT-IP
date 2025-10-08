//! OS detection 16-probe sequence implementation
//!
//! This module implements the 16-probe sequence for OS fingerprinting:
//! - 6 TCP SYN probes to open port (SEQ)
//! - 2 ICMP echo requests (IE1, IE2)
//! - 1 ECN probe
//! - 6 unusual TCP probes (T2-T7)
//! - 1 UDP probe to closed port (U1)
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::os_probe::OsProbeEngine;
//! use std::net::Ipv4Addr;
//!
//! # async fn example() -> Result<(), prtip_core::Error> {
//! let engine = OsProbeEngine::new(
//!     Ipv4Addr::new(192, 168, 1, 1),
//!     80,   // open port
//!     9999  // closed port
//! );
//!
//! let results = engine.send_probes().await?;
//! # Ok(())
//! # }
//! ```

use prtip_core::{Error, ProbeResults};
use prtip_network::packet_builder::{TcpFlags, TcpOption, TcpPacketBuilder, UdpPacketBuilder};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep;

/// OS probe engine for sending 16-probe sequence
pub struct OsProbeEngine {
    /// Target IP address
    target: Ipv4Addr,
    /// Open TCP port on target
    open_port: u16,
    /// Closed TCP port on target
    closed_port: u16,
    /// Source IP (auto-detected)
    source_ip: Ipv4Addr,
}

/// Results from a single TCP probe
#[derive(Debug, Clone)]
pub struct TcpProbeResult {
    /// Initial Sequence Number from response
    pub isn: u32,
    /// IP ID from response
    pub ip_id: u16,
    /// TCP window size from response
    pub window: u16,
    /// TCP options from response
    pub options: Vec<TcpOption>,
    /// Response flags
    pub flags: u8,
    /// TTL from response
    pub ttl: u8,
    /// Don't Fragment bit
    pub df: bool,
    /// Timestamp when probe was sent
    pub timestamp: Instant,
}

/// Results from ICMP echo probe
#[derive(Debug, Clone)]
pub struct IcmpProbeResult {
    /// IP ID from response
    pub ip_id: u16,
    /// TTL from response
    pub ttl: u8,
    /// Don't Fragment bit
    pub df: bool,
    /// Code in response
    pub code: u8,
}

impl OsProbeEngine {
    /// Create new OS probe engine
    pub fn new(target: Ipv4Addr, open_port: u16, closed_port: u16) -> Self {
        Self {
            target,
            open_port,
            closed_port,
            source_ip: Ipv4Addr::new(0, 0, 0, 0), // Auto-detect
        }
    }

    /// Send all 16 probes and collect results
    pub async fn send_probes(&self) -> Result<ProbeResults, Error> {
        let mut results = ProbeResults::default();

        // SEQ: 6 TCP SYN probes to open port (100ms apart)
        let mut seq_results = Vec::new();
        for i in 0..6 {
            if i > 0 {
                sleep(Duration::from_millis(100)).await;
            }

            let probe = self.build_seq_probe(i)?;
            // TODO: Send probe and capture response
            // For now, create placeholder result
            seq_results.push(TcpProbeResult {
                isn: 0,
                ip_id: 0,
                window: 0,
                options: Vec::new(),
                flags: 0,
                ttl: 0,
                df: false,
                timestamp: Instant::now(),
            });
        }

        // Analyze SEQ results
        results.seq = Some(self.analyze_seq_results(&seq_results));

        // IE: 2 ICMP echo requests
        let ie1_probe = self.build_icmp_echo_probe(0, 0)?;
        let ie2_probe = self.build_icmp_echo_probe(4, 9)?;
        // TODO: Send and capture

        results.ie = Some(HashMap::new());

        // ECN: Explicit Congestion Notification probe
        let ecn_probe = self.build_ecn_probe()?;
        // TODO: Send and capture

        results.ecn = Some(HashMap::new());

        // T2-T7: Unusual TCP probes to various ports
        let t2_probe = self.build_t2_probe()?; // NULL flags to open port
        let t3_probe = self.build_t3_probe()?; // SYN+FIN+URG+PSH to open port
        let t4_probe = self.build_t4_probe()?; // ACK to open port
        let t5_probe = self.build_t5_probe()?; // SYN to closed port
        let t6_probe = self.build_t6_probe()?; // ACK to closed port
        let t7_probe = self.build_t7_probe()?; // FIN+PSH+URG to closed port
                                               // TODO: Send and capture

        results.t2 = Some(HashMap::new());
        results.t3 = Some(HashMap::new());
        results.t4 = Some(HashMap::new());
        results.t5 = Some(HashMap::new());
        results.t6 = Some(HashMap::new());
        results.t7 = Some(HashMap::new());

        // U1: UDP probe to closed port
        let u1_probe = self.build_u1_probe()?;
        // TODO: Send and capture

        results.u1 = Some(HashMap::new());

        Ok(results)
    }

    /// Build SEQ probe (TCP SYN to open port)
    fn build_seq_probe(&self, seq_num: u8) -> Result<Vec<u8>, Error> {
        let mut builder = TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(20000 + seq_num as u16)
            .dest_port(self.open_port)
            .flags(TcpFlags::SYN)
            .window(1024 << seq_num) // Different window sizes
            .sequence(rand::random());

        // Different TCP options for each probe
        match seq_num {
            0 => {
                builder = builder
                    .add_option(TcpOption::Mss(1460))
                    .add_option(TcpOption::Nop)
                    .add_option(TcpOption::WindowScale(10))
                    .add_option(TcpOption::Nop)
                    .add_option(TcpOption::Nop)
                    .add_option(TcpOption::Timestamp {
                        tsval: rand::random(),
                        tsecr: 0,
                    });
            }
            1 => {
                builder = builder.add_option(TcpOption::Mss(1400));
            }
            2 => {
                builder = builder
                    .add_option(TcpOption::Nop)
                    .add_option(TcpOption::Nop)
                    .add_option(TcpOption::Timestamp {
                        tsval: rand::random(),
                        tsecr: 0,
                    });
            }
            3 => {
                builder = builder.add_option(TcpOption::WindowScale(7));
            }
            4 => {
                builder = builder.add_option(TcpOption::SackPermitted);
            }
            _ => {}
        }

        Ok(builder.build()?)
    }

    /// Build ICMP echo probe
    fn build_icmp_echo_probe(&self, _tos: u8, _code: u8) -> Result<Vec<u8>, Error> {
        // TODO: Implement ICMP packet builder
        // For now, return minimal packet
        Ok(vec![])
    }

    /// Build ECN probe (SYN with ECN flags)
    fn build_ecn_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30000)
            .dest_port(self.open_port)
            .flags(TcpFlags::SYN.combine(TcpFlags::ECE).combine(TcpFlags::CWR))
            .window(65535)
            .sequence(rand::random())
            .add_option(TcpOption::Mss(1460))
            .add_option(TcpOption::WindowScale(10))
            .build()?)
    }

    /// Build T2 probe (NULL flags to open port)
    fn build_t2_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30002)
            .dest_port(self.open_port)
            .flags(TcpFlags::empty())
            .window(128)
            .sequence(rand::random())
            .build()?)
    }

    /// Build T3 probe (SYN+FIN+URG+PSH to open port)
    fn build_t3_probe(&self) -> Result<Vec<u8>, Error> {
        let flags = TcpFlags::SYN
            .combine(TcpFlags::FIN)
            .combine(TcpFlags::URG)
            .combine(TcpFlags::PSH);

        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30003)
            .dest_port(self.open_port)
            .flags(flags)
            .window(256)
            .sequence(rand::random())
            .build()?)
    }

    /// Build T4 probe (ACK to open port)
    fn build_t4_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30004)
            .dest_port(self.open_port)
            .flags(TcpFlags::ACK)
            .window(1024)
            .sequence(rand::random())
            .acknowledgment(rand::random())
            .build()?)
    }

    /// Build T5 probe (SYN to closed port)
    fn build_t5_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30005)
            .dest_port(self.closed_port)
            .flags(TcpFlags::SYN)
            .window(31337)
            .sequence(rand::random())
            .build()?)
    }

    /// Build T6 probe (ACK to closed port)
    fn build_t6_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30006)
            .dest_port(self.closed_port)
            .flags(TcpFlags::ACK)
            .window(32768)
            .sequence(rand::random())
            .acknowledgment(rand::random())
            .build()?)
    }

    /// Build T7 probe (FIN+PSH+URG to closed port)
    fn build_t7_probe(&self) -> Result<Vec<u8>, Error> {
        let flags = TcpFlags::FIN.combine(TcpFlags::PSH).combine(TcpFlags::URG);

        Ok(TcpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(30007)
            .dest_port(self.closed_port)
            .flags(flags)
            .window(65535)
            .sequence(rand::random())
            .build()?)
    }

    /// Build U1 probe (UDP to closed port)
    fn build_u1_probe(&self) -> Result<Vec<u8>, Error> {
        Ok(UdpPacketBuilder::new()
            .source_ip(self.source_ip)
            .dest_ip(self.target)
            .source_port(40000)
            .dest_port(self.closed_port)
            .payload(b"ProRT-IP".to_vec())
            .build()?)
    }

    /// Analyze SEQ probe results to extract ISN patterns
    fn analyze_seq_results(&self, results: &[TcpProbeResult]) -> HashMap<String, String> {
        let mut seq_data = HashMap::new();

        if results.len() < 2 {
            return seq_data;
        }

        // Calculate ISN deltas
        let mut deltas = Vec::new();
        for i in 1..results.len() {
            let delta = results[i].isn.wrapping_sub(results[i - 1].isn);
            deltas.push(delta);
        }

        // Calculate GCD of deltas
        let gcd = Self::calculate_gcd_vec(&deltas);
        seq_data.insert("GCD".to_string(), format!("{:X}", gcd));

        // Calculate ISN rate (ISR)
        if results.len() >= 2 {
            let time_diff = results
                .last()
                .unwrap()
                .timestamp
                .duration_since(results.first().unwrap().timestamp)
                .as_secs_f64();

            if time_diff > 0.0 {
                let isn_diff = results
                    .last()
                    .unwrap()
                    .isn
                    .wrapping_sub(results.first().unwrap().isn);
                let isr = (isn_diff as f64 / time_diff) as u32;
                seq_data.insert("ISR".to_string(), format!("{:X}", isr));
            }
        }

        // Analyze IP ID generation pattern
        let ip_ids: Vec<u16> = results.iter().map(|r| r.ip_id).collect();
        let ti_pattern = Self::analyze_ip_id_pattern(&ip_ids);
        seq_data.insert("TI".to_string(), ti_pattern);

        // TODO: Add more SEQ analysis (SP, CI, II, SS, TS)

        seq_data
    }

    /// Calculate GCD of a vector of numbers
    fn calculate_gcd_vec(numbers: &[u32]) -> u32 {
        if numbers.is_empty() {
            return 1;
        }

        let mut result = numbers[0];
        for &num in &numbers[1..] {
            result = Self::gcd(result, num);
        }
        result
    }

    /// Calculate GCD of two numbers
    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    /// Analyze IP ID generation pattern
    fn analyze_ip_id_pattern(ip_ids: &[u16]) -> String {
        if ip_ids.len() < 2 {
            return "U".to_string(); // Unknown
        }

        // Check for all zeros
        if ip_ids.iter().all(|&id| id == 0) {
            return "Z".to_string();
        }

        // Check for incremental pattern
        let mut is_incremental = true;
        for i in 1..ip_ids.len() {
            let diff = ip_ids[i].wrapping_sub(ip_ids[i - 1]);
            if diff == 0 || diff > 1000 {
                is_incremental = false;
                break;
            }
        }

        if is_incremental {
            "I".to_string() // Incremental
        } else {
            "RI".to_string() // Random incremental
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(OsProbeEngine::gcd(12, 8), 4);
        assert_eq!(OsProbeEngine::gcd(48, 18), 6);
        assert_eq!(OsProbeEngine::gcd(100, 50), 50);
    }

    #[test]
    fn test_gcd_vec() {
        assert_eq!(OsProbeEngine::calculate_gcd_vec(&[12, 18, 24]), 6);
        assert_eq!(OsProbeEngine::calculate_gcd_vec(&[10, 15, 20]), 5);
    }

    #[test]
    fn test_ip_id_pattern_zero() {
        let ip_ids = vec![0, 0, 0, 0];
        assert_eq!(OsProbeEngine::analyze_ip_id_pattern(&ip_ids), "Z");
    }

    #[test]
    fn test_ip_id_pattern_incremental() {
        let ip_ids = vec![100, 101, 102, 103];
        assert_eq!(OsProbeEngine::analyze_ip_id_pattern(&ip_ids), "I");
    }

    #[test]
    fn test_ip_id_pattern_random() {
        let ip_ids = vec![100, 5000, 200, 8000];
        assert_eq!(OsProbeEngine::analyze_ip_id_pattern(&ip_ids), "RI");
    }

    #[test]
    fn test_build_seq_probe() {
        let engine = OsProbeEngine::new(Ipv4Addr::new(192, 168, 1, 1), 80, 9999);

        let probe = engine.build_seq_probe(0).unwrap();
        assert!(!probe.is_empty());
    }

    #[test]
    fn test_build_ecn_probe() {
        let engine = OsProbeEngine::new(Ipv4Addr::new(192, 168, 1, 1), 80, 9999);

        let probe = engine.build_ecn_probe().unwrap();
        assert!(!probe.is_empty());
    }

    #[test]
    fn test_build_unusual_probes() {
        let engine = OsProbeEngine::new(Ipv4Addr::new(192, 168, 1, 1), 80, 9999);

        assert!(engine.build_t2_probe().is_ok()); // NULL
        assert!(engine.build_t3_probe().is_ok()); // SYN+FIN+URG+PSH
        assert!(engine.build_t4_probe().is_ok()); // ACK
        assert!(engine.build_t5_probe().is_ok()); // SYN to closed
        assert!(engine.build_t6_probe().is_ok()); // ACK to closed
        assert!(engine.build_t7_probe().is_ok()); // FIN+PSH+URG to closed
    }

    #[test]
    fn test_build_u1_probe() {
        let engine = OsProbeEngine::new(Ipv4Addr::new(192, 168, 1, 1), 80, 9999);

        let probe = engine.build_u1_probe().unwrap();
        assert!(!probe.is_empty());
    }
}
