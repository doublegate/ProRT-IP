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
use prtip_network::{
    packet_builder::{TcpFlags, TcpOption, TcpPacketBuilder, UdpPacketBuilder},
    PacketCapture,
};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, trace};

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
    /// Packet capture for sending/receiving
    capture: Arc<Mutex<Option<Box<dyn PacketCapture>>>>,
    /// Response timeout
    timeout: Duration,
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
            capture: Arc::new(Mutex::new(None)),
            timeout: Duration::from_secs(2),
        }
    }

    /// Initialize packet capture
    pub fn with_capture(self, capture: Box<dyn PacketCapture>) -> Self {
        *self.capture.lock().unwrap() = Some(capture);
        self
    }

    /// Set source IP address
    pub fn with_source_ip(mut self, ip: Ipv4Addr) -> Self {
        self.source_ip = ip;
        self
    }

    /// Set response timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
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
            let timestamp = Instant::now();

            // Send probe and capture response
            match self.send_and_capture_tcp(&probe, self.open_port).await {
                Ok(result) => {
                    seq_results.push(TcpProbeResult {
                        isn: result.isn,
                        ip_id: result.ip_id,
                        window: result.window,
                        options: result.options,
                        flags: result.flags,
                        ttl: result.ttl,
                        df: result.df,
                        timestamp,
                    });
                }
                Err(e) => {
                    debug!("SEQ probe {} failed: {}", i, e);
                    // Add placeholder result on error
                    seq_results.push(TcpProbeResult {
                        isn: 0,
                        ip_id: 0,
                        window: 0,
                        options: Vec::new(),
                        flags: 0,
                        ttl: 0,
                        df: false,
                        timestamp,
                    });
                }
            }
        }

        // Analyze SEQ results
        results.seq = Some(self.analyze_seq_results(&seq_results));

        // IE: 2 ICMP echo requests
        let ie1_probe = self.build_icmp_echo_probe(0, 0)?;
        let ie2_probe = self.build_icmp_echo_probe(4, 9)?;

        let mut ie_data = HashMap::new();
        // Send IE1 probe
        match self.send_and_capture_icmp(&ie1_probe).await {
            Ok(result) => {
                ie_data.insert("IE1_IPID".to_string(), format!("{:X}", result.ip_id));
                ie_data.insert("IE1_TTL".to_string(), format!("{}", result.ttl));
                ie_data.insert(
                    "IE1_DF".to_string(),
                    if result.df { "Y" } else { "N" }.to_string(),
                );
            }
            Err(e) => {
                debug!("IE1 probe failed: {}", e);
            }
        }

        // Send IE2 probe
        match self.send_and_capture_icmp(&ie2_probe).await {
            Ok(result) => {
                ie_data.insert("IE2_IPID".to_string(), format!("{:X}", result.ip_id));
                ie_data.insert("IE2_TTL".to_string(), format!("{}", result.ttl));
                ie_data.insert(
                    "IE2_DF".to_string(),
                    if result.df { "Y" } else { "N" }.to_string(),
                );
            }
            Err(e) => {
                debug!("IE2 probe failed: {}", e);
            }
        }

        results.ie = Some(ie_data);

        // ECN: Explicit Congestion Notification probe
        let ecn_probe = self.build_ecn_probe()?;

        let mut ecn_data = HashMap::new();
        match self.send_and_capture_tcp(&ecn_probe, self.open_port).await {
            Ok(result) => {
                ecn_data.insert("ECN_FLAGS".to_string(), format!("{:02X}", result.flags));
                ecn_data.insert("ECN_WIN".to_string(), format!("{:X}", result.window));
                ecn_data.insert("ECN_TTL".to_string(), format!("{}", result.ttl));
            }
            Err(e) => {
                debug!("ECN probe failed: {}", e);
            }
        }

        results.ecn = Some(ecn_data);

        // T2-T7: Unusual TCP probes to various ports
        let t2_probe = self.build_t2_probe()?; // NULL flags to open port
        let t3_probe = self.build_t3_probe()?; // SYN+FIN+URG+PSH to open port
        let t4_probe = self.build_t4_probe()?; // ACK to open port
        let t5_probe = self.build_t5_probe()?; // SYN to closed port
        let t6_probe = self.build_t6_probe()?; // ACK to closed port
        let t7_probe = self.build_t7_probe()?; // FIN+PSH+URG to closed port

        // T2 probe
        let mut t2_data = HashMap::new();
        match self.send_and_capture_tcp(&t2_probe, self.open_port).await {
            Ok(result) => {
                t2_data.insert("T2_FLAGS".to_string(), format!("{:02X}", result.flags));
                t2_data.insert("T2_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T2 probe failed: {}", e),
        }
        results.t2 = Some(t2_data);

        // T3 probe
        let mut t3_data = HashMap::new();
        match self.send_and_capture_tcp(&t3_probe, self.open_port).await {
            Ok(result) => {
                t3_data.insert("T3_FLAGS".to_string(), format!("{:02X}", result.flags));
                t3_data.insert("T3_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T3 probe failed: {}", e),
        }
        results.t3 = Some(t3_data);

        // T4 probe
        let mut t4_data = HashMap::new();
        match self.send_and_capture_tcp(&t4_probe, self.open_port).await {
            Ok(result) => {
                t4_data.insert("T4_FLAGS".to_string(), format!("{:02X}", result.flags));
                t4_data.insert("T4_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T4 probe failed: {}", e),
        }
        results.t4 = Some(t4_data);

        // T5 probe
        let mut t5_data = HashMap::new();
        match self.send_and_capture_tcp(&t5_probe, self.closed_port).await {
            Ok(result) => {
                t5_data.insert("T5_FLAGS".to_string(), format!("{:02X}", result.flags));
                t5_data.insert("T5_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T5 probe failed: {}", e),
        }
        results.t5 = Some(t5_data);

        // T6 probe
        let mut t6_data = HashMap::new();
        match self.send_and_capture_tcp(&t6_probe, self.closed_port).await {
            Ok(result) => {
                t6_data.insert("T6_FLAGS".to_string(), format!("{:02X}", result.flags));
                t6_data.insert("T6_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T6 probe failed: {}", e),
        }
        results.t6 = Some(t6_data);

        // T7 probe
        let mut t7_data = HashMap::new();
        match self.send_and_capture_tcp(&t7_probe, self.closed_port).await {
            Ok(result) => {
                t7_data.insert("T7_FLAGS".to_string(), format!("{:02X}", result.flags));
                t7_data.insert("T7_WIN".to_string(), format!("{:X}", result.window));
            }
            Err(e) => debug!("T7 probe failed: {}", e),
        }
        results.t7 = Some(t7_data);

        // U1: UDP probe to closed port
        let u1_probe = self.build_u1_probe()?;

        let mut u1_data = HashMap::new();
        match self.send_and_capture_udp(&u1_probe, self.closed_port).await {
            Ok(result) => {
                u1_data.insert("U1_IPID".to_string(), format!("{:X}", result.ip_id));
                u1_data.insert("U1_TTL".to_string(), format!("{}", result.ttl));
                u1_data.insert(
                    "U1_DF".to_string(),
                    if result.df { "Y" } else { "N" }.to_string(),
                );
            }
            Err(e) => {
                debug!("U1 probe failed: {}", e);
            }
        }

        results.u1 = Some(u1_data);

        Ok(results)
    }

    /// Send TCP probe and capture response
    async fn send_and_capture_tcp(
        &self,
        packet: &[u8],
        expected_port: u16,
    ) -> Result<TcpProbeResult, Error> {
        // Send packet
        if let Some(ref mut capture) = *self.capture.lock().unwrap() {
            capture
                .send_packet(packet)
                .map_err(|e| Error::Network(format!("Failed to send probe: {}", e)))?;

            trace!("Sent TCP probe to {}:{}", self.target, expected_port);

            // Wait for response
            let start = Instant::now();
            while start.elapsed() < self.timeout {
                if let Some(response) = capture
                    .receive_packet(100)
                    .map_err(|e| Error::Network(format!("Failed to receive: {}", e)))?
                {
                    // Parse response packet
                    if let Some(result) = self.parse_tcp_response(&response, expected_port) {
                        return Ok(result);
                    }
                }
            }

            Err(Error::Network(
                "Timeout waiting for TCP response".to_string(),
            ))
        } else {
            Err(Error::Config("Packet capture not initialized".to_string()))
        }
    }

    /// Send ICMP probe and capture response
    async fn send_and_capture_icmp(&self, packet: &[u8]) -> Result<IcmpProbeResult, Error> {
        // Send packet
        if let Some(ref mut capture) = *self.capture.lock().unwrap() {
            capture
                .send_packet(packet)
                .map_err(|e| Error::Network(format!("Failed to send probe: {}", e)))?;

            trace!("Sent ICMP probe to {}", self.target);

            // Wait for response
            let start = Instant::now();
            while start.elapsed() < self.timeout {
                if let Some(response) = capture
                    .receive_packet(100)
                    .map_err(|e| Error::Network(format!("Failed to receive: {}", e)))?
                {
                    // Parse response packet
                    if let Some(result) = self.parse_icmp_response(&response) {
                        return Ok(result);
                    }
                }
            }

            Err(Error::Network(
                "Timeout waiting for ICMP response".to_string(),
            ))
        } else {
            Err(Error::Config("Packet capture not initialized".to_string()))
        }
    }

    /// Send UDP probe and capture ICMP unreachable response
    async fn send_and_capture_udp(
        &self,
        packet: &[u8],
        expected_port: u16,
    ) -> Result<IcmpProbeResult, Error> {
        // Send packet
        if let Some(ref mut capture) = *self.capture.lock().unwrap() {
            capture
                .send_packet(packet)
                .map_err(|e| Error::Network(format!("Failed to send probe: {}", e)))?;

            trace!("Sent UDP probe to {}:{}", self.target, expected_port);

            // Wait for ICMP unreachable response
            let start = Instant::now();
            while start.elapsed() < self.timeout {
                if let Some(response) = capture
                    .receive_packet(100)
                    .map_err(|e| Error::Network(format!("Failed to receive: {}", e)))?
                {
                    // Parse ICMP response
                    if let Some(result) = self.parse_icmp_response(&response) {
                        return Ok(result);
                    }
                }
            }

            Err(Error::Network(
                "Timeout waiting for ICMP response".to_string(),
            ))
        } else {
            Err(Error::Config("Packet capture not initialized".to_string()))
        }
    }

    /// Parse TCP response packet
    fn parse_tcp_response(&self, packet: &[u8], expected_port: u16) -> Option<TcpProbeResult> {
        use pnet_packet::ethernet::{EtherTypes, EthernetPacket};
        use pnet_packet::ipv4::Ipv4Packet;
        use pnet_packet::tcp::TcpPacket;
        use pnet_packet::Packet;

        // Parse Ethernet frame
        let eth = EthernetPacket::new(packet)?;
        if eth.get_ethertype() != EtherTypes::Ipv4 {
            return None;
        }

        // Parse IP packet
        let ip = Ipv4Packet::new(eth.payload())?;
        if ip.get_source() != self.target {
            return None;
        }

        // Parse TCP packet
        let tcp = TcpPacket::new(ip.payload())?;
        if tcp.get_source() != expected_port {
            return None;
        }

        // Extract TCP options
        let mut options = Vec::new();
        for opt_bytes in tcp.get_options_iter() {
            if let Some(opt) = Self::parse_tcp_option(&opt_bytes) {
                options.push(opt);
            }
        }

        Some(TcpProbeResult {
            isn: tcp.get_sequence(),
            ip_id: ip.get_identification(),
            window: tcp.get_window(),
            options,
            flags: tcp.get_flags(),
            ttl: ip.get_ttl(),
            df: (ip.get_flags() & 0x02) != 0,
            timestamp: Instant::now(),
        })
    }

    /// Parse ICMP response packet
    fn parse_icmp_response(&self, packet: &[u8]) -> Option<IcmpProbeResult> {
        use pnet_packet::ethernet::{EtherTypes, EthernetPacket};
        use pnet_packet::icmp::IcmpPacket;
        use pnet_packet::ipv4::Ipv4Packet;
        use pnet_packet::Packet;

        // Parse Ethernet frame
        let eth = EthernetPacket::new(packet)?;
        if eth.get_ethertype() != EtherTypes::Ipv4 {
            return None;
        }

        // Parse IP packet
        let ip = Ipv4Packet::new(eth.payload())?;
        if ip.get_source() != self.target {
            return None;
        }

        // Parse ICMP packet
        let icmp = IcmpPacket::new(ip.payload())?;

        Some(IcmpProbeResult {
            ip_id: ip.get_identification(),
            ttl: ip.get_ttl(),
            df: (ip.get_flags() & 0x02) != 0,
            code: icmp.get_icmp_code().0,
        })
    }

    /// Parse TCP option from raw bytes
    fn parse_tcp_option(bytes: &pnet_packet::tcp::TcpOptionPacket) -> Option<TcpOption> {
        use pnet_packet::tcp::TcpOptionNumbers;
        use pnet_packet::Packet;

        match bytes.get_number() {
            TcpOptionNumbers::MSS => {
                let payload = bytes.payload();
                if payload.len() >= 2 {
                    let mss = u16::from_be_bytes([payload[0], payload[1]]);
                    Some(TcpOption::Mss(mss))
                } else {
                    None
                }
            }
            TcpOptionNumbers::WSCALE => {
                let payload = bytes.payload();
                if !payload.is_empty() {
                    Some(TcpOption::WindowScale(payload[0]))
                } else {
                    None
                }
            }
            TcpOptionNumbers::SACK_PERMITTED => Some(TcpOption::SackPermitted),
            TcpOptionNumbers::TIMESTAMPS => {
                let payload = bytes.payload();
                if payload.len() >= 8 {
                    let tsval =
                        u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    let tsecr =
                        u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                    Some(TcpOption::Timestamp { tsval, tsecr })
                } else {
                    None
                }
            }
            TcpOptionNumbers::NOP => Some(TcpOption::Nop),
            TcpOptionNumbers::EOL => Some(TcpOption::Eol),
            _ => None,
        }
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
    fn build_icmp_echo_probe(&self, tos: u8, code: u8) -> Result<Vec<u8>, Error> {
        use pnet_packet::icmp::IcmpPacket;
        use pnet_packet::icmp::{echo_request, IcmpCode, IcmpTypes};
        use pnet_packet::ipv4::Ipv4Packet;
        use pnet_packet::MutablePacket;

        // ICMP echo request packet
        let mut icmp_buffer = vec![0u8; 64]; // ICMP header (8) + data (56)

        {
            let mut icmp_packet = echo_request::MutableEchoRequestPacket::new(&mut icmp_buffer)
                .ok_or_else(|| Error::Network("Failed to create ICMP packet".to_string()))?;

            icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
            icmp_packet.set_icmp_code(IcmpCode(code));
            icmp_packet.set_identifier(rand::random());
            icmp_packet.set_sequence_number(rand::random());

            // Fill payload with random data
            let payload = icmp_packet.payload_mut();
            for byte in payload.iter_mut() {
                *byte = rand::random();
            }
        }

        // Calculate and set checksum (using cloned buffer for calculation)
        let checksum = pnet_packet::icmp::checksum(&IcmpPacket::new(&icmp_buffer.clone()).unwrap());
        {
            let mut icmp_packet = echo_request::MutableEchoRequestPacket::new(&mut icmp_buffer)
                .ok_or_else(|| Error::Network("Failed to create ICMP packet".to_string()))?;
            icmp_packet.set_checksum(checksum);
        }

        // Build IP packet with ICMP payload
        let total_len = 20 + icmp_buffer.len(); // IP header (20) + ICMP packet
        let mut ip_buffer = vec![0u8; total_len];

        {
            let mut ip_packet = pnet_packet::ipv4::MutableIpv4Packet::new(&mut ip_buffer)
                .ok_or_else(|| Error::Network("Failed to create IP packet".to_string()))?;

            ip_packet.set_version(4);
            ip_packet.set_header_length(5); // 5 * 4 = 20 bytes
            ip_packet.set_dscp(tos >> 2);
            ip_packet.set_ecn(tos & 0x03);
            ip_packet.set_total_length(total_len as u16);
            ip_packet.set_identification(rand::random());
            ip_packet.set_flags(0x02); // Don't Fragment
            ip_packet.set_fragment_offset(0);
            ip_packet.set_ttl(64);
            ip_packet.set_next_level_protocol(pnet_packet::ip::IpNextHeaderProtocols::Icmp);
            ip_packet.set_source(self.source_ip);
            ip_packet.set_destination(self.target);

            // Copy ICMP packet to IP payload
            ip_packet.set_payload(&icmp_buffer);
        }

        // Calculate and set IP checksum (using cloned buffer for calculation)
        let checksum = pnet_packet::ipv4::checksum(&Ipv4Packet::new(&ip_buffer.clone()).unwrap());
        {
            let mut ip_packet = pnet_packet::ipv4::MutableIpv4Packet::new(&mut ip_buffer)
                .ok_or_else(|| Error::Network("Failed to create IP packet".to_string()))?;
            ip_packet.set_checksum(checksum);
        }

        Ok(ip_buffer)
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

        // SP: ISN counter rate (sequence predictability)
        if !deltas.is_empty() {
            let avg_delta = deltas.iter().map(|&d| d as f64).sum::<f64>() / deltas.len() as f64;
            let variance = deltas
                .iter()
                .map(|&d| {
                    let diff = d as f64 - avg_delta;
                    diff * diff
                })
                .sum::<f64>()
                / deltas.len() as f64;
            let std_dev = variance.sqrt();

            // Categorize predictability based on standard deviation
            let sp = if std_dev < 100.0 {
                "0" // Very predictable
            } else if std_dev < 1000.0 {
                "1-4" // Somewhat predictable
            } else if std_dev < 10000.0 {
                "5-10" // Moderately random
            } else {
                "11+" // Highly random
            };
            seq_data.insert("SP".to_string(), sp.to_string());
        }

        // CI: IP ID counter increments (closed port IP ID)
        let ci_pattern = Self::analyze_ip_id_pattern(&ip_ids);
        seq_data.insert("CI".to_string(), ci_pattern.clone());

        // II: Incremental IP ID (similar to CI but for all responses)
        seq_data.insert("II".to_string(), ci_pattern);

        // SS: TCP timestamp option presence
        let has_timestamps = results.iter().any(|r| {
            r.options
                .iter()
                .any(|opt| matches!(opt, TcpOption::Timestamp { .. }))
        });
        seq_data.insert(
            "SS".to_string(),
            if has_timestamps { "S" } else { "U" }.to_string(),
        );

        // TS: TCP timestamp values (if present)
        if has_timestamps {
            let ts_values: Vec<u32> = results
                .iter()
                .filter_map(|r| {
                    r.options.iter().find_map(|opt| {
                        if let TcpOption::Timestamp { tsval, .. } = opt {
                            Some(*tsval)
                        } else {
                            None
                        }
                    })
                })
                .collect();

            if ts_values.len() >= 2 {
                // Calculate timestamp increments
                let mut ts_deltas = Vec::new();
                for i in 1..ts_values.len() {
                    ts_deltas.push(ts_values[i].wrapping_sub(ts_values[i - 1]));
                }

                let avg_ts_delta =
                    ts_deltas.iter().map(|&d| d as f64).sum::<f64>() / ts_deltas.len() as f64;

                // Categorize timestamp behavior
                let ts = if avg_ts_delta < 10.0 {
                    "U" // Not used or near-zero
                } else if avg_ts_delta < 100.0 {
                    "1" // 1-100 Hz
                } else if avg_ts_delta < 1000.0 {
                    "2" // 100-1000 Hz (typical: 100-200 Hz)
                } else {
                    "7" // >1000 Hz
                };
                seq_data.insert("TS".to_string(), ts.to_string());
            }
        }

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
