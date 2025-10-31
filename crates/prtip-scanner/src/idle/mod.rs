//! Idle Scan (Zombie Scan) Implementation
//!
//! This module implements Nmap's idle scan technique, which enables completely anonymous
//! port scanning by bouncing probes off a "zombie" host. The scanner's IP address never
//! contacts the target directly, making this the most stealthy scan technique available.
//!
//! # How Idle Scan Works
//!
//! 1. **Find a zombie**: A host with predictable IPID (IP Identification) increments
//! 2. **Measure baseline IPID**: Send SYN/ACK to zombie, record IPID in RST response
//! 3. **Spoof SYN packet**: Send SYN to target with zombie's source IP
//! 4. **Target responds to zombie**:
//!    - Open port: Target sends SYN/ACK, zombie replies RST (IPID +2)
//!    - Closed port: Target sends RST, zombie ignores it (IPID +1)
//!    - Filtered port: No response (IPID +0)
//! 5. **Measure post-scan IPID**: Calculate delta to infer port state
//!
//! # Example
//!
//! ```no_run
//! use prtip_scanner::idle::{ZombieDiscovery, IdleScanner, IdleScanConfig};
//! use std::net::IpAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Find suitable zombie in subnet
//! let discovery = ZombieDiscovery::new("192.168.1.0/24".to_string());
//! let zombies = discovery.find_zombies().await?;
//! let best_zombie = zombies.first().expect("No zombie found");
//!
//! // Scan target via zombie
//! let config = IdleScanConfig {
//!     zombie: best_zombie.clone(),
//!     wait_time_ms: 300,
//!     retries: 2,
//!     confidence_threshold: 0.8,
//! };
//!
//! let mut scanner = IdleScanner::new(config)?;
//! let target: IpAddr = "10.0.0.1".parse()?;
//! let results = scanner.scan_ports(target, &[22, 80, 443]).await?;
//!
//! for result in results {
//!     println!("Port {}: {:?} (confidence: {:.2})",
//!         result.port, result.state, result.confidence);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # References
//!
//! - Nmap Idle Scan: https://nmap.org/book/idlescan.html
//! - Original technique by Salvatore Sanfilippo (antirez)
//! - RFC 791 (IP) - IPID field specification
//! - RFC 793 (TCP) - SYN/ACK/RST handshake

pub mod idle_scanner;
pub mod ipid_tracker;
pub mod zombie_discovery;

// Re-export main types
pub use idle_scanner::{IdleScanConfig, IdleScanResult, IdleScanner};
pub use ipid_tracker::{IPIDMeasurement, IPIDPattern, IPIDTracker};
pub use zombie_discovery::{DiscoveryConfig, ZombieCandidate, ZombieDiscovery};
