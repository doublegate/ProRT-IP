//! Git-style multi-page help system for ProRT-IP
//!
//! Provides categorized help output similar to Git's help system, improving
//! discoverability for 50+ CLI flags and reducing cognitive load for users.

use colored::*;
use std::collections::HashMap;

/// Main help system with categorized topics
pub struct HelpSystem {
    categories: HashMap<String, HelpCategory>,
}

/// A category of help topics
pub struct HelpCategory {
    name: String,
    description: String,
    detailed_help: String,
}

/// A single example scenario
pub struct Example {
    title: String,
    command: String,
    description: String,
}

impl HelpSystem {
    /// Create a new help system with all categories initialized
    pub fn new() -> Self {
        let mut categories = HashMap::new();

        // Initialize all 9 categories
        categories.insert("scan-types".to_string(), Self::scan_types_category());
        categories.insert(
            "host-discovery".to_string(),
            Self::host_discovery_category(),
        );
        categories.insert("port-specs".to_string(), Self::port_specs_category());
        categories.insert("timing".to_string(), Self::timing_category());
        categories.insert(
            "service-detection".to_string(),
            Self::service_detection_category(),
        );
        categories.insert("os-detection".to_string(), Self::os_detection_category());
        categories.insert("output".to_string(), Self::output_category());
        categories.insert("stealth".to_string(), Self::stealth_category());
        categories.insert("misc".to_string(), Self::misc_category());

        Self { categories }
    }

    /// Display all categories with brief descriptions
    pub fn show_categories(&self) {
        println!("{}", "ProRT-IP WarScan - Help Topics".bright_white().bold());
        println!("{}", "=".repeat(60).bright_cyan());
        println!();
        println!("Usage: prtip help <topic>");
        println!("       prtip help examples");
        println!();
        println!("{}", "Available topics:".bright_white().bold());
        println!();

        // Sort categories for consistent display
        let mut sorted_categories: Vec<_> = self.categories.values().collect();
        sorted_categories.sort_by_key(|c| &c.name);

        for category in sorted_categories {
            println!(
                "  {}  {}",
                category.name.bright_yellow().bold(),
                category.description.dimmed()
            );
        }

        println!();
        println!("{}", "Examples:".bright_white().bold());
        println!("  prtip help scan-types      # Show scan type options");
        println!("  prtip help timing           # Show timing and performance options");
        println!("  prtip help examples         # Show common usage examples");
        println!();
        println!("{}", "=".repeat(60).bright_cyan());
    }

    /// Display detailed help for a specific topic
    pub fn show_topic(&self, topic: &str) {
        // Case-insensitive lookup
        let topic_lower = topic.to_lowercase();

        if let Some(category) = self.categories.get(&topic_lower) {
            println!(
                "{}",
                format!("ProRT-IP Help: {}", category.name)
                    .bright_white()
                    .bold()
            );
            println!("{}", "=".repeat(60).bright_cyan());
            println!();
            println!("{}", category.detailed_help);
            println!();
            println!("{}", "=".repeat(60).bright_cyan());
        } else {
            println!("{}", "Error: Unknown help topic".red().bold());
            println!();
            println!("Unknown topic: '{}'", topic.yellow());
            println!();
            println!("Run 'prtip help' to see all available topics.");
        }
    }

    /// Display common usage examples
    pub fn show_examples(&self) {
        let examples = Self::get_examples();

        println!(
            "{}",
            "ProRT-IP WarScan - Usage Examples".bright_white().bold()
        );
        println!("{}", "=".repeat(60).bright_cyan());
        println!();
        println!("Common scanning scenarios and their commands:");
        println!();

        for (i, example) in examples.iter().enumerate() {
            println!(
                "{}",
                format!("{}. {}", i + 1, example.title)
                    .bright_white()
                    .bold()
            );
            println!("   {}", example.command.bright_cyan());
            println!("   {}", example.description.dimmed());
            println!();
        }

        println!("{}", "=".repeat(60).bright_cyan());
        println!();
        println!("For more details on specific topics, run: prtip help <topic>");
    }

    // ========================================================================
    // Category Definitions
    // ========================================================================

    fn scan_types_category() -> HelpCategory {
        HelpCategory {
            name: "scan-types".to_string(),
            description: "TCP/UDP scan techniques (SYN, Connect, FIN, NULL, Xmas, ACK, UDP)"
                .to_string(),
            detailed_help: format!(
                "{}\n\n\
                ProRT-IP supports 7 different scan types, each with specific use cases:\n\n\
                {} {} (TCP SYN Scan)\n\
                  Fast, stealthy half-open scanning. Sends SYN packets without completing\n\
                  the TCP handshake. Requires raw socket privileges (root/Administrator).\n\
                  Leaves no connection logs on target system.\n\n\
                  Usage: prtip -sS <target>\n\
                  Best for: Default scanning, penetration testing, stealth operations\n\n\
                {} {} (TCP Connect Scan)\n\
                  Full TCP 3-way handshake using OS's connect() syscall. No special\n\
                  privileges required. More detectable but universally compatible.\n\n\
                  Usage: prtip -sT <target>\n\
                  Best for: Unprivileged scans, compatibility, when raw sockets unavailable\n\n\
                {} {} (UDP Scan)\n\
                  Probes UDP services by sending protocol-specific payloads. Slower due\n\
                  to ICMP rate limiting (10-100x slower than TCP). Requires raw sockets.\n\n\
                  Usage: prtip -sU -p 53,161,500 <target>\n\
                  Best for: DNS, SNMP, NetBIOS discovery, VPN detection\n\n\
                {} {} (FIN Scan)\n\
                  Stealth scan sending FIN flag. RFC 793: closed ports send RST, open\n\
                  ports ignore. May bypass simple firewalls. Fails on Windows/Cisco.\n\n\
                  Usage: prtip -sF <target>\n\
                  Best for: IDS/firewall evasion, stealth reconnaissance\n\n\
                {} {} (NULL Scan)\n\
                  Stealth scan with no TCP flags set. RFC 793 compliance: closed ports\n\
                  send RST, open ports ignore. Fails on Windows/Cisco.\n\n\
                  Usage: prtip -sN <target>\n\
                  Best for: IDS/firewall evasion, testing RFC compliance\n\n\
                {} {} (Xmas Scan)\n\
                  Stealth scan with FIN+PSH+URG flags (\"lights up like a Christmas tree\").\n\
                  Same principles as NULL/FIN scans. Fails on Windows/Cisco.\n\n\
                  Usage: prtip -sX <target>\n\
                  Best for: IDS/firewall evasion, creative stealth testing\n\n\
                {} {} (ACK Scan)\n\
                  Firewall rule mapping. Sends ACK packets to determine if ports are\n\
                  filtered. Doesn't determine open/closed state, only filtered/unfiltered.\n\n\
                  Usage: prtip -sA -p 80,443 <target>\n\
                  Best for: Firewall testing, ACL enumeration, infrastructure mapping\n\n\
                {}\n\
                • SYN scan is fastest and most stealthy (default for privileged users)\n\
                • Connect scan is most reliable (default for unprivileged users)\n\
                • UDP scan is slowest but essential for UDP services\n\
                • FIN/NULL/Xmas bypass some firewalls but fail on modern systems\n\
                • ACK scan maps firewall rules, not port states\n\n\
                {}\n\
                Raw socket scans (SYN, UDP, FIN, NULL, Xmas, ACK) require root/Administrator\n\
                privileges. ProRT-IP will automatically fall back to Connect scan if running\n\
                unprivileged.",
                "SCAN TYPES".bright_white().bold(),
                "-sS".bright_cyan().bold(),
                "SYN Scan".bright_yellow(),
                "-sT".bright_cyan().bold(),
                "Connect Scan".bright_yellow(),
                "-sU".bright_cyan().bold(),
                "UDP Scan".bright_yellow(),
                "-sF".bright_cyan().bold(),
                "FIN Scan".bright_yellow(),
                "-sN".bright_cyan().bold(),
                "NULL Scan".bright_yellow(),
                "-sX".bright_cyan().bold(),
                "Xmas Scan".bright_yellow(),
                "-sA".bright_cyan().bold(),
                "ACK Scan".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "PRIVILEGES".bright_white().bold(),
            ),
        }
    }

    fn host_discovery_category() -> HelpCategory {
        HelpCategory {
            name: "host-discovery".to_string(),
            description: "Host discovery options (ping scans, ARP, ICMP, TCP/UDP ping)".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Determine which hosts are online before port scanning:\n\n\
                {} {}\n\
                  Skip host discovery and treat all targets as online. Useful when hosts\n\
                  don't respond to ping but have open ports (firewall blocks ICMP).\n\n\
                  Usage: prtip -Pn <target>\n\
                  Alias: prtip --skip-ping <target>\n\n\
                {} {}\n\
                  Only perform host discovery, don't port scan. Quickly determine which\n\
                  hosts are up in a network range before full scanning.\n\n\
                  Usage: prtip --ping-only 10.0.0.0/8\n\n\
                {} {} (ARP Ping)\n\
                  ARP-based discovery for local network (same subnet). Fastest and most\n\
                  reliable for local networks. Automatically used for local networks.\n\n\
                  Usage: prtip -PR 192.168.1.0/24\n\
                  Best for: Local network discovery (same broadcast domain)\n\n\
                {} {} (TCP SYN Ping)\n\
                  Send TCP SYN packets to discover hosts. Default port 80. More reliable\n\
                  than ICMP for hosts behind firewalls that block ICMP.\n\n\
                  Usage: prtip -PS <target>       # Port 80\n\
                         prtip -PS21,22,80 <target>  # Multiple ports\n\n\
                {} {} (TCP ACK Ping)\n\
                  Send TCP ACK packets for discovery. May bypass stateless firewalls\n\
                  that block SYN packets. Default port 80.\n\n\
                  Usage: prtip -PA <target>\n\
                         prtip -PA443 <target>\n\n\
                {} {} (UDP Ping)\n\
                  Send UDP packets for discovery. Useful for hosts that don't respond\n\
                  to TCP or ICMP. Default port 40125 (uncommon port).\n\n\
                  Usage: prtip -PU <target>\n\
                         prtip -PU53,161 <target>\n\n\
                {} {} (ICMP Echo Ping)\n\
                  Standard ICMP echo request (traditional ping). Works for most hosts\n\
                  but often blocked by firewalls.\n\n\
                  Usage: prtip -PE <target>\n\n\
                {} {} (ICMP Timestamp Ping)\n\
                  ICMP timestamp request. Alternative when echo is blocked. Some hosts\n\
                  respond to timestamp but not echo requests.\n\n\
                  Usage: prtip -PP <target>\n\n\
                {}\n\
                • Use ARP ping (-PR) for local networks (fastest, most accurate)\n\
                • Use TCP SYN ping (-PS) for internet hosts (bypasses ICMP blocks)\n\
                • Combine multiple techniques: prtip -PS -PA -PE <target>\n\
                • Skip ping (-Pn) when targets are known to be up\n\
                • Use --ping-only for quick network reconnaissance\n\n\
                {}\n\
                Default behavior: ProRT-IP performs automatic host discovery using\n\
                appropriate methods based on target network (ARP for local, TCP/ICMP\n\
                for remote). Use --no-ping or -Pn to disable.",
                "HOST DISCOVERY".bright_white().bold(),
                "-Pn".bright_cyan().bold(),
                "(No Ping)".bright_yellow(),
                "--ping-only".bright_cyan().bold(),
                "(Ping Only)".bright_yellow(),
                "-PR".bright_cyan().bold(),
                "(ARP Ping)".bright_yellow(),
                "-PS".bright_cyan().bold(),
                "(TCP SYN Ping)".bright_yellow(),
                "-PA".bright_cyan().bold(),
                "(TCP ACK Ping)".bright_yellow(),
                "-PU".bright_cyan().bold(),
                "(UDP Ping)".bright_yellow(),
                "-PE".bright_cyan().bold(),
                "(ICMP Echo)".bright_yellow(),
                "-PP".bright_cyan().bold(),
                "(ICMP Timestamp)".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "DEFAULT BEHAVIOR".bright_white().bold(),
            ),
        }
    }

    fn port_specs_category() -> HelpCategory {
        HelpCategory {
            name: "port-specs".to_string(),
            description: "Port specification (ranges, top ports, randomization)".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Control which ports to scan:\n\n\
                {} {}\n\
                  Specify ports to scan. Supports ranges, lists, and combinations.\n\n\
                  Usage: prtip -p 80                  # Single port\n\
                         prtip -p 1-1000             # Range\n\
                         prtip -p 80,443,8080        # List\n\
                         prtip -p 1-1000,8000-9000   # Multiple ranges\n\
                         prtip -p-                    # All 65535 ports\n\n\
                {} {} (Fast Scan)\n\
                  Scan only the top 100 most common ports from nmap-services database.\n\
                  Dramatically faster than default 1-1000 range (~66ms vs ~200ms).\n\n\
                  Usage: prtip -F <target>\n\
                  Equivalent to: prtip --top-ports 100 <target>\n\n\
                {} {} (Top Ports)\n\
                  Scan the N most common ports based on nmap-services frequency database.\n\
                  Balances speed vs comprehensiveness.\n\n\
                  Usage: prtip --top-ports 10 <target>     # Quickest (10 ports)\n\
                         prtip --top-ports 100 <target>    # Fast (100 ports)\n\
                         prtip --top-ports 1000 <target>   # Thorough (1000 ports)\n\
                         prtip --top-ports 5000 <target>   # Comprehensive\n\n\
                {} {} (Port Ratio)\n\
                  Scan ports more common than specified ratio (0.0-1.0). Advanced option\n\
                  for fine-grained control over which ports to scan based on frequency.\n\n\
                  Usage: prtip --port-ratio 0.5 <target>   # Top 50%% of ports\n\
                         prtip --port-ratio 0.1 <target>   # Top 10%% of ports\n\n\
                {} {} (No Randomize)\n\
                  Scan ports in sequential order (1, 2, 3...) instead of randomized.\n\
                  Slightly faster but more detectable by IDS/IPS systems.\n\n\
                  Usage: prtip -r <target>\n\n\
                {}\n\
                • Default: 1-1000 ports (balanced speed and coverage)\n\
                • Quick scan: -F or --top-ports 100 (66ms for common ports)\n\
                • Thorough scan: --top-ports 1000 (200ms, covers 99%% of services)\n\
                • Comprehensive: -p- (all 65535 ports, ~37ms for 10K ports)\n\
                • Targeted: -p 80,443,8080,8443 (web services only)\n\n\
                {}\n\
                ProRT-IP scans ports in random order by default to evade IDS/IPS detection.\n\
                Use -r flag to scan sequentially (faster but more detectable).\n\n\
                {}\n\
                Top 10 ports cover ~70%% of internet services\n\
                Top 100 ports cover ~95%% of internet services\n\
                Top 1000 ports cover ~99%% of internet services\n\
                Full 65535 ports required only for comprehensive security audits",
                "PORT SPECIFICATION".bright_white().bold(),
                "-p <ports>".bright_cyan().bold(),
                "(Port List)".bright_yellow(),
                "-F".bright_cyan().bold(),
                "(Fast Scan)".bright_yellow(),
                "--top-ports <N>".bright_cyan().bold(),
                "(Top Ports)".bright_yellow(),
                "--port-ratio <ratio>".bright_cyan().bold(),
                "(Port Ratio)".bright_yellow(),
                "-r".bright_cyan().bold(),
                "(No Randomize)".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "RANDOMIZATION".bright_white().bold(),
                "PORT COVERAGE".bright_white().bold(),
            ),
        }
    }

    fn timing_category() -> HelpCategory {
        HelpCategory {
            name: "timing".to_string(),
            description: "Timing templates and rate limiting (T0-T5, retries, delays, rates)".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Control scan speed and network behavior:\n\n\
                {} {} (Timing Templates)\n\
                  Predefined timing profiles balancing speed vs stealth.\n\n\
                  -T0  Paranoid:    5 min delay, serial scanning (IDS evasion)\n\
                  -T1  Sneaky:      15 sec delay, serial scanning (slower IDS evasion)\n\
                  -T2  Polite:      0.4 sec delay, less aggressive (low bandwidth)\n\
                  -T3  Normal:      Default balanced timing (recommended)\n\
                  -T4  Aggressive:  Fast scanning for fast networks\n\
                  -T5  Insane:      Very fast, may overwhelm target/network\n\n\
                  Usage: prtip -T4 <target>\n\n\
                {} {}\n\
                  Maximum number of times to retransmit probes that receive no response.\n\
                  Higher values improve accuracy for lossy networks but slow scans.\n\n\
                  Usage: prtip --max-retries 5 <target>\n\
                  Default: 10 retries\n\
                  Range: 0-10\n\n\
                {} {}\n\
                  Give up on host after specified time. Prevents wasting time on\n\
                  unresponsive hosts.\n\n\
                  Usage: prtip --host-timeout 30m <target>  # 30 minutes\n\
                         prtip --host-timeout 5h <target>   # 5 hours\n\n\
                {} {}\n\
                  Wait specified time between probes to same host. Useful for avoiding\n\
                  rate limiting or IDS/IPS detection.\n\n\
                  Usage: prtip --scan-delay 100ms <target>  # 100 milliseconds\n\
                         prtip --scan-delay 1s <target>     # 1 second\n\n\
                {} {}\n\
                  Cap on scan delay between probes. Prevents probe delays from growing\n\
                  too large due to network congestion.\n\n\
                  Usage: prtip --max-scan-delay 500ms <target>\n\n\
                {} {}\n\
                  Send at least N packets per second. Ensures minimum scan rate even\n\
                  when network conditions vary.\n\n\
                  Usage: prtip --min-rate 100 <target>      # 100 pps minimum\n\
                         prtip --min-rate 1000 <target>     # 1000 pps minimum\n\n\
                {} {}\n\
                  Send at most N packets per second. Useful for avoiding network\n\
                  congestion or respecting bandwidth limits.\n\n\
                  Usage: prtip --max-rate 10000 <target>    # 10K pps maximum\n\
                         prtip --max-rate 100000 <target>   # 100K pps maximum\n\n\
                {}\n\
                • Local network: -T4 (aggressive, ~66ms for 1K ports)\n\
                • Internet scans: -T3 (normal, balanced)\n\
                • Stealth scans: -T0 or -T1 (slow, evasive)\n\
                • Rate-limited targets: --max-rate 1000 (avoid overwhelming)\n\
                • Fast networks: -T5 with --min-rate 50000 (maximum speed)\n\n\
                {}\n\
                  Timing template sets multiple parameters:\n\
                  • Initial probe timeout\n\
                  • Maximum probe timeout\n\
                  • Maximum retries\n\
                  • Parallelism hints\n\n\
                  Individual flags (--max-retries, --scan-delay, etc.) override template settings.\n\n\
                {}\n\
                  ProRT-IP can achieve 1M+ packets per second on modern hardware with\n\
                  appropriate settings:\n\
                  • -T5 --min-rate 100000 --max-concurrent 1000\n\
                  • Requires sufficient ulimit (--ulimit 10000)\n\
                  • Best for internet-scale sweeps",
                "TIMING AND PERFORMANCE".bright_white().bold(),
                "-T<0-5>".bright_cyan().bold(),
                "(Timing Templates)".bright_yellow(),
                "--max-retries <N>".bright_cyan().bold(),
                "(Max Retries)".bright_yellow(),
                "--host-timeout <time>".bright_cyan().bold(),
                "(Host Timeout)".bright_yellow(),
                "--scan-delay <time>".bright_cyan().bold(),
                "(Scan Delay)".bright_yellow(),
                "--max-scan-delay <time>".bright_cyan().bold(),
                "(Max Delay)".bright_yellow(),
                "--min-rate <N>".bright_cyan().bold(),
                "(Min Rate)".bright_yellow(),
                "--max-rate <N>".bright_cyan().bold(),
                "(Max Rate)".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "TEMPLATE BEHAVIOR".bright_white().bold(),
                "PERFORMANCE TIPS".bright_white().bold(),
            ),
        }
    }

    fn service_detection_category() -> HelpCategory {
        HelpCategory {
            name: "service-detection".to_string(),
            description: "Service version detection (intensity, probes, TLS support)".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Identify services running on open ports:\n\n\
                {} {}\n\
                  Enable service version detection. Connects to open ports and sends\n\
                  protocol-specific probes to identify services and versions.\n\n\
                  Usage: prtip -sV <target>\n\
                         prtip -sV -p 80,443 <target>\n\n\
                {} {}\n\
                  Control how many probes to send per port (0-9). Higher intensity\n\
                  increases accuracy but takes longer.\n\n\
                  0:  Light probing (fastest, ~50%% accuracy)\n\
                  1-6: Increasing probe count\n\
                  7:  Default (balanced, ~80%% accuracy)\n\
                  8-9: Comprehensive probing (slowest, ~95%% accuracy)\n\n\
                  Usage: prtip -sV --version-intensity 9 <target>    # Max accuracy\n\
                         prtip -sV --version-intensity 3 <target>    # Fast\n\n\
                {} {}\n\
                  Disable TLS/SSL service detection. Faster but misses HTTPS, SMTPS,\n\
                  IMAPS, and other TLS-wrapped services.\n\n\
                  Usage: prtip -sV --no-tls <target>\n\
                  Performance: ~30%% faster but 20-30%% lower detection rate\n\n\
                {} {}\n\
                  Use custom service probe database instead of embedded nmap-service-probes.\n\
                  Useful for detecting proprietary or uncommon services.\n\n\
                  Usage: prtip -sV --probe-db custom-probes.txt <target>\n\n\
                {}\n\
                ProRT-IP uses the same nmap-service-probes database as Nmap (187 probes)\n\
                for accurate service detection. The probe database contains protocol-specific\n\
                signatures for identifying 1000+ services.\n\n\
                {}\n\
                1. Open port discovered\n\
                2. Connect to port\n\
                3. Send NULL probe (wait for banner)\n\
                4. If no match, send protocol-specific probes\n\
                5. Match response against regex patterns\n\
                6. Extract service name and version\n\n\
                {}\n\
                • Quick scan: -sV --version-intensity 3 (3-5 probes per port)\n\
                • Balanced: -sV (default, 7 probes per port)\n\
                • Accurate: -sV --version-intensity 9 (all probes)\n\
                • Fast mode: -sV --no-tls (skip TLS for speed)\n\
                • Combine with OS detection: -sV -O (comprehensive fingerprinting)\n\n\
                {}\n\
                  prtip -sV -p 22,80,443 192.168.1.1\n\
                  # Might detect:\n\
                  # 22/tcp   open  ssh      OpenSSH 8.9p1\n\
                  # 80/tcp   open  http     nginx 1.22.0\n\
                  # 443/tcp  open  https    nginx 1.22.0 (TLS)\n\n\
                {}\n\
                Service detection adds ~2-3 seconds per open port (depending on intensity).\n\
                For large scans, consider using -sV only on interesting ports rather than\n\
                all ports to maintain performance.",
                "SERVICE DETECTION".bright_white().bold(),
                "-sV".bright_cyan().bold(),
                "(Version Detection)".bright_yellow(),
                "--version-intensity <0-9>".bright_cyan().bold(),
                "(Intensity)".bright_yellow(),
                "--no-tls".bright_cyan().bold(),
                "(Disable TLS)".bright_yellow(),
                "--probe-db <file>".bright_cyan().bold(),
                "(Custom Probes)".bright_yellow(),
                "PROBE DATABASE".bright_white().bold(),
                "DETECTION PROCESS".bright_white().bold(),
                "RECOMMENDATIONS".bright_white().bold(),
                "EXAMPLE OUTPUT".bright_white().bold(),
                "PERFORMANCE".bright_white().bold(),
            ),
        }
    }

    fn os_detection_category() -> HelpCategory {
        HelpCategory {
            name: "os-detection".to_string(),
            description: "OS fingerprinting and accuracy settings".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Determine target operating system:\n\n\
                {} {}\n\
                  Enable OS fingerprinting. Analyzes TCP/IP stack behavior to identify\n\
                  operating system. Requires at least one open and one closed port.\n\n\
                  Usage: prtip -O <target>\n\
                         prtip -sS -O <target>\n\n\
                {} {}\n\
                  Only attempt OS detection on hosts with at least one open port.\n\
                  Improves accuracy and reduces wasted effort on hosts with all closed ports.\n\n\
                  Usage: prtip -O --osscan-limit <target>\n\n\
                {} {}\n\
                  Enable all detection methods: OS detection + service detection + progress bar.\n\
                  Equivalent to: -O -sV --progress\n\n\
                  Usage: prtip -A <target>\n\
                         prtip -A -p 1-1000 <target>\n\n\
                {}\n\
                ProRT-IP uses TCP/IP fingerprinting with 16 probes sent to open and closed\n\
                ports. Responses are analyzed for:\n\
                • TCP window size\n\
                • TCP options\n\
                • TCP timestamp behavior\n\
                • ICMP responses\n\
                • IP TTL values\n\
                • Packet fragmentation handling\n\n\
                Fingerprints are matched against a database of 2,600+ OS signatures.\n\n\
                {}\n\
                • At least one open TCP port required\n\
                • At least one closed TCP port required\n\
                • Raw packet privileges needed (root/Administrator)\n\
                • Some hosts actively resist fingerprinting (SYN cookies, etc.)\n\
                • NAT/firewalls may obscure results\n\n\
                {}\n\
                  prtip -O -sS -p 1-1000 192.168.1.1\n\
                  # Might detect:\n\
                  # OS: Linux 5.10-5.19 (95%% confidence)\n\
                  # OS: Ubuntu 22.04 or Debian 11 (90%% confidence)\n\n\
                {}\n\
                • Use with SYN scan: -sS -O (most accurate)\n\
                • Combine with service detection: -A (comprehensive)\n\
                • Limit to promising hosts: --osscan-limit (faster)\n\
                • Scan enough ports: -p 1-1000 or more (better accuracy)\n\n\
                {}\n\
                OS detection is probabilistic, not definitive. Results show confidence\n\
                percentage based on signature match quality. Accuracy ranges from 70-95%%\n\
                depending on target OS and network conditions.",
                "OS DETECTION".bright_white().bold(),
                "-O".bright_cyan().bold(),
                "(OS Detection)".bright_yellow(),
                "--osscan-limit".bright_cyan().bold(),
                "(Limit to Open Ports)".bright_yellow(),
                "-A".bright_cyan().bold(),
                "(Aggressive Mode)".bright_yellow(),
                "FINGERPRINTING METHOD".bright_white().bold(),
                "REQUIREMENTS & LIMITATIONS".bright_white().bold(),
                "EXAMPLE OUTPUT".bright_white().bold(),
                "RECOMMENDATIONS".bright_white().bold(),
                "ACCURACY".bright_white().bold(),
            ),
        }
    }

    fn output_category() -> HelpCategory {
        HelpCategory {
            name: "output".to_string(),
            description: "Output formats (text, JSON, XML, greppable) and filtering".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Control output format and filtering:\n\n\
                {} {}\n\
                  Normal text output to file. Human-readable format similar to terminal.\n\n\
                  Usage: prtip -oN scan.txt <target>\n\
                  Alias: prtip --output-normal scan.txt <target>\n\n\
                {} {}\n\
                  XML output to file. Machine-parseable, Nmap-compatible format.\n\n\
                  Usage: prtip -oX scan.xml <target>\n\
                  Alias: prtip --output-xml scan.xml <target>\n\n\
                {} {}\n\
                  Greppable output to file. One line per host for easy parsing with\n\
                  grep, awk, sed, or shell scripts.\n\n\
                  Usage: prtip -oG scan.gnmap <target>\n\
                  Format: Host: <ip> (<hostname>) Ports: <port>/<state>/<protocol>/<service>\n\n\
                {} {}\n\
                  Output all three formats with given basename.\n\
                  Creates: <basename>.txt, <basename>.xml, <basename>.gnmap\n\n\
                  Usage: prtip -oA scan-results <target>\n\
                  Creates: scan-results.txt, scan-results.xml, scan-results.gnmap\n\n\
                {} {}\n\
                  Show only open (or possibly open) ports in output. Filters closed and\n\
                  filtered ports. Dramatically reduces output size for large scans.\n\n\
                  Usage: prtip --open <target>\n\
                         prtip -sS -p- --open <target>    # All ports, show only open\n\n\
                {} {}\n\
                  Show all packets sent and received. Very verbose, useful for debugging\n\
                  and understanding scan behavior.\n\n\
                  Usage: prtip --packet-trace <target>\n\n\
                {} {}\n\
                  Display reason why port is in particular state (SYN-ACK, RST, timeout, etc.).\n\
                  Useful for understanding firewall behavior.\n\n\
                  Usage: prtip --reason <target>\n\
                  Output: 80/tcp open http reason: syn-ack\n\n\
                {} {}\n\
                  Print scan statistics every N seconds during scan.\n\n\
                  Usage: prtip --stats-every 5s <target>    # Stats every 5 seconds\n\
                         prtip --stats-every 1m <target>    # Stats every minute\n\n\
                {} {}\n\
                  Increase verbosity level. More verbose output includes internal details.\n\n\
                  -v:   Basic verbose (info level)\n\
                  -vv:  More verbose (debug level)\n\
                  -vvv: Maximum verbosity (trace level)\n\n\
                  Usage: prtip -vv <target>\n\n\
                {} {}\n\
                  Suppress banner and non-essential output. Useful for scripting.\n\n\
                  Usage: prtip -q <target>\n\n\
                {}\n\
                • JSON output: -o json --output-file scan.json (API integration)\n\
                • XML output: -oX scan.xml (tool compatibility)\n\
                • Greppable: -oG scan.gnmap (shell scripting)\n\
                • Show only interesting: --open (reduce noise)\n\
                • All formats: -oA basename (comprehensive archiving)\n\n\
                {}\n\
                  # Large scan, only open ports\n\
                  prtip -p- --open 192.168.1.0/24\n\n\
                  # JSON output for API\n\
                  prtip -sV -o json --output-file api-results.json target.com\n\n\
                  # All formats for audit\n\
                  prtip -A -oA audit-2024-01-15 target-network.com",
                "OUTPUT FORMATS".bright_white().bold(),
                "-oN <file>".bright_cyan().bold(),
                "(Normal Text)".bright_yellow(),
                "-oX <file>".bright_cyan().bold(),
                "(XML)".bright_yellow(),
                "-oG <file>".bright_cyan().bold(),
                "(Greppable)".bright_yellow(),
                "-oA <basename>".bright_cyan().bold(),
                "(All Formats)".bright_yellow(),
                "--open".bright_cyan().bold(),
                "(Open Ports Only)".bright_yellow(),
                "--packet-trace".bright_cyan().bold(),
                "(Packet Trace)".bright_yellow(),
                "--reason".bright_cyan().bold(),
                "(Show Reason)".bright_yellow(),
                "--stats-every <time>".bright_cyan().bold(),
                "(Statistics)".bright_yellow(),
                "-v/-vv/-vvv".bright_cyan().bold(),
                "(Verbosity)".bright_yellow(),
                "-q".bright_cyan().bold(),
                "(Quiet)".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "EXAMPLES".bright_white().bold(),
            ),
        }
    }

    fn stealth_category() -> HelpCategory {
        HelpCategory {
            name: "stealth".to_string(),
            description: "Stealth and evasion (decoys, fragmentation, source port)".to_string(),
            detailed_help: format!(
                "{}\n\n\
                Evade IDS/IPS detection and firewall filtering:\n\n\
                {} {}\n\
                  Use specified source port for scanning. Some firewalls allow traffic\n\
                  from specific source ports (53 for DNS, 20 for FTP-DATA).\n\n\
                  Usage: prtip -g 53 <target>           # Source port 53 (DNS)\n\
                         prtip --source-port 20 <target>  # Source port 20 (FTP)\n\n\
                {} {}\n\
                  Generate random decoy source IPs to hide real scanning source.\n\
                  Makes it harder to identify the actual attacker.\n\n\
                  Usage: prtip -D RND:10 <target>       # 10 random decoys\n\
                         prtip -D decoy1,ME,decoy2 <target>  # Specific decoys\n\n\
                {} {}\n\
                  Use slow timing templates to evade IDS/IPS detection.\n\n\
                  -T0: Paranoid (5 min between probes)\n\
                  -T1: Sneaky (15 sec between probes)\n\
                  -T2: Polite (0.4 sec between probes)\n\n\
                  Usage: prtip -T0 <target>    # Paranoid (very slow)\n\
                         prtip -T1 <target>    # Sneaky (slow)\n\n\
                {} {}\n\
                  Use stealth scan types that may bypass simple firewalls.\n\n\
                  -sF:  FIN scan (no flags set in initial probe)\n\
                  -sN:  NULL scan (all flags cleared)\n\
                  -sX:  Xmas scan (FIN+PSH+URG flags)\n\n\
                  Usage: prtip -sF -T1 <target>    # Stealth FIN + slow timing\n\n\
                {} {}\n\
                  Add delays between probes to avoid rate-based detection.\n\n\
                  Usage: prtip --scan-delay 100ms <target>  # 100ms between probes\n\
                         prtip --scan-delay 1s <target>     # 1 second delay\n\n\
                {} {}\n\
                  Don't randomize port scan order. Scan sequentially (1, 2, 3...).\n\
                  Slightly faster but more detectable.\n\n\
                  Usage: prtip -r <target>\n\
                  Note: NOT recommended for stealth (more detectable)\n\n\
                {}\n\
                ProRT-IP randomizes port scan order by default to avoid sequential\n\
                patterns that IDS/IPS systems detect. Use -r flag to disable (not\n\
                recommended for stealth operations).\n\n\
                {}\n\
                • Source port spoofing: -g 53 (DNS port)\n\
                • Slow scan: -T0 or -T1 (evade rate-based detection)\n\
                • Stealth scan type: -sF, -sN, -sX (bypass simple firewalls)\n\
                • Add delays: --scan-delay 500ms (avoid IDS triggers)\n\
                • Decoys: -D RND:10 (hide among noise)\n\
                • Randomize: Default behavior (don't use -r for stealth)\n\n\
                {}\n\
                  # Maximum stealth scan\n\
                  prtip -sF -T0 -g 53 -D RND:10 --scan-delay 5s <target>\n\n\
                  # Moderate stealth (faster)\n\
                  prtip -sS -T2 -g 53 --scan-delay 100ms <target>\n\n\
                {}\n\
                • Stealth scanning is slower (T0 can take hours for 1000 ports)\n\
                • Some techniques require raw packet privileges\n\
                • FIN/NULL/Xmas scans fail on Windows and Cisco devices\n\
                • Decoys may be traced back via network topology analysis\n\
                • Source port spoofing may not work through NAT/firewalls\n\n\
                {}\n\
                Stealth scanning is for authorized security testing only. Unauthorized\n\
                port scanning may be illegal. Always obtain proper authorization.",
                "STEALTH AND EVASION".bright_white().bold(),
                "-g <port>".bright_cyan().bold(),
                "(Source Port)".bright_yellow(),
                "-D <decoys>".bright_cyan().bold(),
                "(Decoy Scan)".bright_yellow(),
                "-T0/-T1/-T2".bright_cyan().bold(),
                "(Slow Timing)".bright_yellow(),
                "-sF/-sN/-sX".bright_cyan().bold(),
                "(Stealth Scans)".bright_yellow(),
                "--scan-delay".bright_cyan().bold(),
                "(Probe Delay)".bright_yellow(),
                "-r".bright_cyan().bold(),
                "(Sequential)".bright_yellow(),
                "PORT RANDOMIZATION".bright_white().bold(),
                "RECOMMENDATIONS".bright_white().bold(),
                "EXAMPLES".bright_white().bold(),
                "LIMITATIONS".bright_white().bold(),
                "LEGAL WARNING".bright_white().bold(),
            ),
        }
    }

    fn misc_category() -> HelpCategory {
        HelpCategory {
            name: "misc".to_string(),
            description: "Miscellaneous options (verbosity, interfaces, privileges, version)"
                .to_string(),
            detailed_help: format!(
                "{}\n\n\
                Miscellaneous options and utilities:\n\n\
                {} {}\n\
                  Print version information and exit.\n\n\
                  Usage: prtip --version\n\n\
                {} {}\n\
                  List available network interfaces with IP addresses, MAC addresses,\n\
                  and status. Useful for selecting correct interface for scanning.\n\n\
                  Usage: prtip --iflist 192.168.1.1 (ignored target required by CLI)\n\n\
                {} {}\n\
                  Use specified network interface for scanning. Useful when multiple\n\
                  interfaces are available.\n\n\
                  Usage: prtip --interface eth0 <target>\n\
                         prtip --interface wlan0 <target>\n\n\
                {} {}\n\
                  Force use of raw ethernet frames instead of IP packets. Advanced option\n\
                  for custom packet crafting.\n\n\
                  Usage: prtip --send-eth <target>\n\n\
                {} {}\n\
                  Force use of IP packets instead of raw ethernet frames. Default for\n\
                  most scans.\n\n\
                  Usage: prtip --send-ip <target>\n\n\
                {} {}\n\
                  Assume user has privileges for raw sockets. Skips privilege checks.\n\
                  Use when running as root/Administrator.\n\n\
                  Usage: prtip --privileged <target>\n\n\
                {} {}\n\
                  Assume user does NOT have privileges. Forces TCP connect scan.\n\
                  Mutually exclusive with --privileged.\n\n\
                  Usage: prtip --unprivileged <target>\n\n\
                {} {} / {}\n\
                  Never perform DNS resolution. Faster but IP addresses only.\n\n\
                  Usage: prtip -n <target>\n\
                         prtip --no-dns <target>\n\n\
                {} {}\n\
                  Increase verbosity of output. More -v flags = more verbose.\n\n\
                  -v:   Basic verbose (warnings and info)\n\
                  -vv:  More verbose (debug information)\n\
                  -vvv: Maximum verbosity (trace everything)\n\n\
                  Usage: prtip -vv <target>\n\n\
                {} {}\n\
                  Quiet mode. Suppress banner and non-essential output.\n\n\
                  Usage: prtip -q <target>\n\n\
                {} {}\n\
                  Show progress bar during scan. Useful for long-running scans.\n\n\
                  Usage: prtip --progress <target>\n\n\
                {} {}\n\
                  Disable progress bar output. Useful for scripting or piped output.\n\n\
                  Usage: prtip --no-progress <target>\n\n\
                {} {}\n\
                  Adjust file descriptor limit (Unix only). Allows more concurrent connections.\n\n\
                  Usage: prtip --ulimit 10000 <target>\n\
                  Minimum: 100\n\
                  Recommended: 2x your max-concurrent value\n\n\
                {}\n\
                • Check interfaces: --iflist (before scanning)\n\
                • Increase verbosity: -vv (for debugging)\n\
                • Suppress output: -q (for scripting)\n\
                • Show progress: --progress (for long scans)\n\
                • Adjust limits: --ulimit 10000 (for large scans)\n\n\
                {}\n\
                  # List interfaces\n\
                  prtip --iflist target.com\n\n\
                  # Use specific interface\n\
                  prtip --interface eth0 192.168.1.0/24\n\n\
                  # Verbose scan with progress\n\
                  prtip -vv --progress target.com\n\n\
                  # Quiet scan for scripts\n\
                  prtip -q -oN results.txt target.com",
                "MISCELLANEOUS".bright_white().bold(),
                "--version".bright_cyan().bold(),
                "(Version)".bright_yellow(),
                "--iflist".bright_cyan().bold(),
                "(Interface List)".bright_yellow(),
                "--interface <iface>".bright_cyan().bold(),
                "(Select Interface)".bright_yellow(),
                "--send-eth".bright_cyan().bold(),
                "(Ethernet Frames)".bright_yellow(),
                "--send-ip".bright_cyan().bold(),
                "(IP Packets)".bright_yellow(),
                "--privileged".bright_cyan().bold(),
                "(Assume Privileged)".bright_yellow(),
                "--unprivileged".bright_cyan().bold(),
                "(Assume Unprivileged)".bright_yellow(),
                "-n".bright_cyan().bold(),
                "(No DNS)".bright_yellow(),
                "--no-dns".bright_yellow(),
                "-v/-vv/-vvv".bright_cyan().bold(),
                "(Verbosity)".bright_yellow(),
                "-q".bright_cyan().bold(),
                "(Quiet)".bright_yellow(),
                "--progress".bright_cyan().bold(),
                "(Progress Bar)".bright_yellow(),
                "--no-progress".bright_cyan().bold(),
                "(No Progress)".bright_yellow(),
                "--ulimit <limit>".bright_cyan().bold(),
                "(File Descriptor Limit)".bright_yellow(),
                "RECOMMENDATIONS".bright_white().bold(),
                "EXAMPLES".bright_white().bold(),
            ),
        }
    }

    // ========================================================================
    // Examples
    // ========================================================================

    fn get_examples() -> Vec<Example> {
        vec![
            Example {
                title: "Basic SYN scan of local network".to_string(),
                command: "prtip -sS 192.168.1.0/24".to_string(),
                description: "Fast TCP SYN scan of all hosts in 192.168.1.0/24 subnet".to_string(),
            },
            Example {
                title: "Service detection on specific ports".to_string(),
                command: "prtip -sV -p 22,80,443 target.com".to_string(),
                description: "Detect SSH, HTTP, and HTTPS service versions".to_string(),
            },
            Example {
                title: "OS detection with SYN scan".to_string(),
                command: "prtip -sS -O 192.168.1.1".to_string(),
                description: "Fingerprint operating system using TCP/IP stack analysis".to_string(),
            },
            Example {
                title: "Stealth FIN scan with slow timing".to_string(),
                command: "prtip -sF -T0 target.com".to_string(),
                description: "Stealthy scan using FIN packets and paranoid timing (very slow)"
                    .to_string(),
            },
            Example {
                title: "Fast scan of top 100 ports".to_string(),
                command: "prtip -F 192.168.1.1".to_string(),
                description: "Quick scan of 100 most common ports (~66ms)".to_string(),
            },
            Example {
                title: "Full port scan (all 65535 ports)".to_string(),
                command: "prtip -p- 192.168.1.1".to_string(),
                description: "Comprehensive scan of every possible TCP port".to_string(),
            },
            Example {
                title: "UDP scan of DNS and SNMP ports".to_string(),
                command: "prtip -sU -p 53,161,500 192.168.1.0/24".to_string(),
                description: "Scan UDP services (slower due to ICMP rate limiting)".to_string(),
            },
            Example {
                title: "Aggressive scan with all detection".to_string(),
                command: "prtip -A -p 1-1000 192.168.1.0/24".to_string(),
                description: "Enable OS detection, service detection, and progress bar".to_string(),
            },
            Example {
                title: "Save results in XML format".to_string(),
                command: "prtip -sS -p 80,443 -oX scan.xml 10.0.0.0/24".to_string(),
                description: "Export results in Nmap-compatible XML for tool integration"
                    .to_string(),
            },
            Example {
                title: "Save results in all formats".to_string(),
                command: "prtip -sS -p 80,443 -oA scan-results 192.168.1.0/24".to_string(),
                description: "Create .txt, .xml, and .gnmap files with scan results".to_string(),
            },
            Example {
                title: "Scan top 1000 ports with service detection".to_string(),
                command: "prtip --top-ports 1000 -sV target.com".to_string(),
                description: "Thorough scan of 1000 most common ports with version detection"
                    .to_string(),
            },
            Example {
                title: "Show only open ports from large scan".to_string(),
                command: "prtip --open -p- 192.168.1.0/24".to_string(),
                description: "Scan all ports but only display open results (reduce noise)"
                    .to_string(),
            },
            Example {
                title: "ACK scan for firewall rule mapping".to_string(),
                command: "prtip -sA -p 80,443 target.com".to_string(),
                description: "Determine if ports are filtered by firewall (not open/closed)"
                    .to_string(),
            },
            Example {
                title: "Scan with source port spoofing".to_string(),
                command: "prtip -g 53 -sS target.com".to_string(),
                description: "Use source port 53 (DNS) to bypass firewall rules".to_string(),
            },
            Example {
                title: "Multiple targets with different formats".to_string(),
                command: "prtip -sS target1.com target2.com 192.168.1.1".to_string(),
                description: "Scan multiple hosts (hostnames and IPs)".to_string(),
            },
            Example {
                title: "Performance tuning for fast networks".to_string(),
                command: "prtip -T4 --min-rate 1000 10.0.0.0/24".to_string(),
                description: "Aggressive timing with minimum 1000 packets/sec rate".to_string(),
            },
            Example {
                title: "Detailed service detection with high intensity".to_string(),
                command: "prtip -sV --version-intensity 9 -p 1-1000 target.com".to_string(),
                description: "Maximum accuracy service detection (slowest but most thorough)"
                    .to_string(),
            },
            Example {
                title: "Greppable output for shell scripting".to_string(),
                command: "prtip -sS -p 80,443 -oG scan.gnmap 192.168.1.0/24".to_string(),
                description: "One line per host, easy to parse with grep/awk/sed".to_string(),
            },
            Example {
                title: "Skip host discovery (assume all online)".to_string(),
                command: "prtip -Pn -sS -p 80,443 192.168.1.0/24".to_string(),
                description: "Don't ping, treat all hosts as up (bypasses ping blocks)".to_string(),
            },
            Example {
                title: "Scan with progress bar for long scans".to_string(),
                command: "prtip --progress -p- target.com".to_string(),
                description: "Show real-time progress bar during full port scan".to_string(),
            },
            Example {
                title: "IPv6 scan".to_string(),
                command: "prtip -6 -sS fe80::1".to_string(),
                description: "Scan IPv6 address (use -6 flag for IPv6 mode)".to_string(),
            },
            Example {
                title: "Combine stealth techniques".to_string(),
                command: "prtip -sF -T1 -g 53 --scan-delay 500ms target.com".to_string(),
                description: "Maximum stealth: FIN scan + slow timing + source port + delay"
                    .to_string(),
            },
            Example {
                title: "List network interfaces".to_string(),
                command: "prtip --iflist target.com".to_string(),
                description: "Show all available network interfaces with IP/MAC addresses"
                    .to_string(),
            },
        ]
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_initialization() {
        let help = HelpSystem::new();
        assert_eq!(help.categories.len(), 9);
        assert!(help.categories.contains_key("scan-types"));
        assert!(help.categories.contains_key("host-discovery"));
        assert!(help.categories.contains_key("port-specs"));
        assert!(help.categories.contains_key("timing"));
        assert!(help.categories.contains_key("service-detection"));
        assert!(help.categories.contains_key("os-detection"));
        assert!(help.categories.contains_key("output"));
        assert!(help.categories.contains_key("stealth"));
        assert!(help.categories.contains_key("misc"));
    }

    #[test]
    fn test_help_categories_list() {
        let help = HelpSystem::new();
        // Should not panic when displaying categories
        help.show_categories();
    }

    #[test]
    fn test_help_topic_display_valid() {
        let help = HelpSystem::new();
        // Should not panic for valid topic
        help.show_topic("scan-types");
        help.show_topic("timing");
        help.show_topic("output");
    }

    #[test]
    fn test_help_topic_case_insensitive() {
        let help = HelpSystem::new();
        // Should work with different cases
        help.show_topic("SCAN-TYPES");
        help.show_topic("Scan-Types");
        help.show_topic("scan-types");
    }

    #[test]
    fn test_help_invalid_topic() {
        let help = HelpSystem::new();
        // Should not panic for invalid topic
        help.show_topic("invalid-topic-name");
    }

    #[test]
    fn test_help_examples() {
        let help = HelpSystem::new();
        // Should not panic when displaying examples
        help.show_examples();
    }

    #[test]
    fn test_examples_count() {
        let examples = HelpSystem::get_examples();
        assert!(examples.len() >= 20, "Should have at least 20 examples");
    }

    #[test]
    fn test_examples_content() {
        let examples = HelpSystem::get_examples();
        // Verify examples have required fields
        for example in &examples {
            assert!(
                !example.title.is_empty(),
                "Example title should not be empty"
            );
            assert!(
                !example.command.is_empty(),
                "Example command should not be empty"
            );
            assert!(
                !example.description.is_empty(),
                "Example description should not be empty"
            );
            assert!(
                example.command.starts_with("prtip"),
                "Example command should start with 'prtip'"
            );
        }
    }

    #[test]
    fn test_category_help_text_length() {
        let help = HelpSystem::new();
        for category in help.categories.values() {
            // Each category should have substantial help text (at least 500 characters)
            assert!(
                category.detailed_help.len() >= 500,
                "Category '{}' should have at least 500 characters of help text",
                category.name
            );
        }
    }

    #[test]
    fn test_help_empty_input() {
        let help = HelpSystem::new();
        // Test with empty string (should behave like invalid topic)
        help.show_topic("");
    }
}
