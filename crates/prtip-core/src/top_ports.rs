//! Port priority lists for `-F` and `--top-ports`
//!
//! # What this ordering is
//!
//! An **editorial ranking of IANA port assignments**, produced by applying a
//! written rule ([`tools/gen-top-ports/RULE.md`]) to a committed snapshot of the
//! IANA Service Name and Transport Protocol Port Number Registry. The rule
//! groups each assignment into one of thirteen service families using the
//! registry's own service names and descriptions, ranks those families by how
//! much an operator learns from finding one open, and interleaves them so that
//! any prefix of the list spans every family. Ties are broken by ascending port
//! number and by nothing else.
//!
//! # What this ordering is not
//!
//! **It is not a frequency ranking.** No input to it is a measurement, and
//! ProRT-IP makes no claim about how often any of these ports is open in
//! practice. Two consequences, both intentional:
//!
//! - `-F` and `--top-ports N` select a *different set of ports* than Nmap,
//!   Masscan, RustScan or any other scanner whose list derives from measured
//!   frequency data. Results are not comparable port-for-port.
//! - There is no hit-rate or coverage guarantee. If a specific port matters,
//!   name it with `-p`.
//!
//! The reason is licensing, and it is not a workaround for something better:
//! `nmap-services` is NPSL-covered and cannot be read or derived from by a work
//! distributed under plain GPL-3.0, while scans.io and Censys both restrict
//! their data to non-commercial use, which GPL-3.0 cannot carry. No lawfully
//! redistributable measured-frequency dataset was available, so ProRT-IP ships
//! an ordering it can justify from first principles instead of one it cannot
//! ship. `RULE.md` records the full reasoning.
//!
//! # Regenerating
//!
//! ```text
//! python3 tools/gen-top-ports/generate.py           # rewrite the arrays
//! python3 tools/gen-top-ports/generate.py --check   # CI gate
//! python3 tools/gen-top-ports/generate.py --explain 20
//! ```
//!
//! [`tools/gen-top-ports/RULE.md`]: https://github.com/doublegate/ProRT-IP/blob/main/tools/gen-top-ports/RULE.md

// BEGIN GENERATED: gen-top-ports -- edit tools/gen-top-ports/, not this block
//
// Regenerate with:  python3 tools/gen-top-ports/generate.py
// CI gate:          python3 tools/gen-top-ports/generate.py --check
//
// Source: tools/gen-service-probes/data/iana-ports.tsv (committed IANA
// Service Name and Transport Protocol Port Number Registry snapshot).
// Rule:   tools/gen-top-ports/RULE.md
//
// This is an editorial ranking of registry assignments, not a frequency
// ranking. No input to it is a measurement.

/// The first 100 ports of ProRT-IP's editorial priority ordering (`-F`).
///
/// Produced by applying `tools/gen-top-ports/RULE.md` to a committed snapshot
/// of the IANA Service Name and Transport Protocol Port Number Registry.
///
/// **This is not a frequency ranking.** No claim is made about how often any
/// of these ports is open in practice, and the selection differs from Nmap's
/// `-F` and from every other scanner whose list comes from measured data.
pub const PRIORITY_PORTS_100: &[u16] = &[
    22, 80, 20, 65, 25, 42, 31, 111, 537, 119, 92, 67, 23, 280, 21, 66, 50, 43, 49, 123, 554, 194,
    515, 68, 89, 443, 69, 118, 58, 53, 56, 135, 649, 433, 1314, 179, 107, 488, 115, 150, 109, 63,
    88, 161, 1071, 529, 2081, 363, 222, 591, 139, 156, 110, 101, 113, 192, 1300, 563, 2291, 500,
    512, 593, 152, 400, 143, 105, 221, 193, 1718, 1815, 3096, 546, 513, 598, 247, 446, 209, 137,
    300, 391, 1719, 1883, 3396, 547, 541, 623, 311, 523, 220, 138, 389, 530, 1720, 2059, 3910, 604,
    830, 631, 445, 590,
];

/// The first 1000 ports of the same ordering (`--top-ports 1000`).
///
/// `PRIORITY_PORTS_100` is a prefix of this list by construction, so
/// [`get_priority_ports`] can serve any `n <= 1000` as a simple prefix.
///
/// **This is not a frequency ranking.** See [`PRIORITY_PORTS_100`].
pub const PRIORITY_PORTS_1000: &[u16] = &[
    22, 80, 20, 65, 25, 42, 31, 111, 537, 119, 92, 67, 23, 280, 21, 66, 50, 43, 49, 123, 554, 194,
    515, 68, 89, 443, 69, 118, 58, 53, 56, 135, 649, 433, 1314, 179, 107, 488, 115, 150, 109, 63,
    88, 161, 1071, 529, 2081, 363, 222, 591, 139, 156, 110, 101, 113, 192, 1300, 563, 2291, 500,
    512, 593, 152, 400, 143, 105, 221, 193, 1718, 1815, 3096, 546, 513, 598, 247, 446, 209, 137,
    300, 391, 1719, 1883, 3396, 547, 541, 623, 311, 523, 220, 138, 389, 530, 1720, 2059, 3910, 604,
    830, 631, 445, 590, 406, 261, 464, 567, 1755, 2191, 3911, 647, 902, 664, 487, 888, 465, 520,
    514, 601, 1790, 2218, 3951, 847, 903, 777, 548, 1114, 587, 597, 636, 602, 2000, 2227, 4088,
    1080, 992, 1001, 564, 1157, 993, 853, 749, 685, 2292, 2868, 5309, 1194, 1174, 1128, 574, 1159,
    995, 1052, 750, 1035, 2415, 2980, 8100, 1293, 1197, 1129, 608, 1186, 1396, 1337, 1614, 1232,
    2427, 3331, 40404, 1701, 1494, 1183, 873, 1201, 1397, 1512, 1615, 1427, 2517, 3423, 8609, 1723,
    1970, 1184, 989, 1388, 1398, 1870, 1812, 1440, 2727, 3615, 1985, 1973, 1760, 990, 1433, 2246,
    2164, 1813, 1605, 2979, 3714, 1994, 2122, 2069, 1120, 1434, 2593, 3415, 2083, 1712, 3242, 4109,
    2644, 2179, 2301, 1150, 1498, 3007, 3849, 2139, 1847, 3771, 4411, 2865, 2198, 2381, 1155, 1525,
    3264, 4321, 2147, 1900, 4307, 4450, 2876, 2512, 2688, 1758, 1527, 3332, 5352, 2334, 1906, 4569,
    4664, 3265, 2513, 2784, 1818, 1529, 4190, 5353, 2392, 1993, 5004, 4692, 3318, 2564, 3106, 2049,
    1571, 4405, 5354, 2478, 2039, 5005, 4788, 3503, 2598, 3443, 2257, 1630, 5355, 2821, 2374, 5059,
    4803, 3568, 2654, 3570, 2529, 1748, 6701, 3113, 2377, 5060, 5222, 3653, 2897, 3631, 2811, 1754,
    8953, 3207, 2420, 5061, 5269, 3663, 3083, 3702, 3020, 1808, 9956, 3269, 2514, 7411, 5271, 3772,
    3389, 3816, 3305, 1809, 3407, 2697, 8417, 5298, 3850, 3454, 3840, 3713, 1830, 3411, 3241, 8554,
    5573, 3928, 3468, 3930, 4049, 1862, 3710, 3347, 9100, 5597, 3949, 3533, 3941, 4050, 2005, 3799,
    3421, 9750, 5670, 4112, 3696, 4035, 4672, 2073, 3833, 3427, 10116, 5671, 4133, 3897, 4036,
    4687, 2273, 4032, 3446, 11164, 5672, 4370, 4089, 4590, 5233, 2278, 4129, 3479, 11720, 6697,
    4500, 4151, 4848, 6619, 2439, 5635, 3482, 13823, 6901, 5150, 4334, 4849, 6620, 2481, 6268,
    3532, 22335, 7631, 7674, 4914, 5248, 6621, 2482, 6269, 3668, 4621, 7672, 7675, 4915, 5280,
    6622, 2483, 6301, 3686, 8433, 7673, 8001, 5024, 5357, 7000, 2484, 7004, 3819, 8040, 8567, 5161,
    5443, 7117, 2638, 7847, 3935, 8090, 8899, 5162, 5554, 8148, 2690, 9002, 3937, 8883, 16665,
    5231, 5985, 9800, 2786, 9021, 3954, 9119, 17234, 5631, 5986, 9802, 3306, 27999, 4153, 9122,
    22305, 5632, 5988, 20048, 3308, 4335, 9123, 22343, 5900, 5989, 20049, 3309, 4336, 9955, 38201,
    6252, 5990, 21554, 3352, 4403, 11235, 44123, 6623, 6122, 22537, 3630, 4404, 12012, 4754, 6789,
    6443, 37601, 3835, 4406, 12013, 4755, 7228, 6480, 3841, 4413, 32767, 6634, 7229, 6770, 3891,
    4414, 38800, 6635, 7279, 6771, 3938, 4421, 44818, 6636, 8688, 6788, 4135, 4460, 10111, 7802,
    9535, 6842, 4136, 4552, 8503, 9555, 7443, 4137, 4727, 12009, 11110, 7627, 4427, 4739, 16666,
    12302, 7677, 4430, 4740, 30004, 17235, 8008, 4570, 4742, 24754, 8080, 4900, 4774, 30003, 8084,
    4950, 5063, 43000, 8088, 4999, 5229, 4980, 8118, 5029, 5689, 8243, 5102, 5780, 8280, 5155,
    5987, 8443, 5343, 5993, 8444, 5432, 6343, 8765, 5433, 6513, 8800, 5505, 6514, 8910, 5575, 6515,
    8989, 5629, 6556, 8990, 5677, 7272, 8991, 5984, 8117, 9294, 6379, 8161, 9295, 6446, 8181, 9389,
    6624, 9005, 9443, 6640, 10161, 9444, 7002, 10162, 9762, 7003, 11161, 9988, 7474, 12321, 10880,
    7574, 12322, 11165, 7687, 13218, 11175, 7981, 13832, 11371, 7982, 18242, 16992, 8070, 23333,
    16993, 8102, 32769, 20002, 8432, 42510, 20003, 8997, 45000, 24680, 9088, 45001, 27504, 9089,
    47001, 44323, 9093, 49001, 9212, 5474, 9306, 7040, 9628, 9286, 9981, 11171, 10160, 11430,
    11211, 11877, 12005, 40853, 12006, 12007, 12008, 13785, 19790, 25100, 27017, 33060, 38000,
    38638, 1, 5, 7, 9, 11, 13, 17, 18, 19, 27, 29, 33, 37, 38, 39, 41, 44, 45, 46, 48, 52, 54, 55,
    62, 64, 70, 71, 72, 73, 74, 76, 78, 79, 82, 83, 84, 85, 86, 90, 91, 93, 94, 95, 96, 97, 98, 99,
    102, 103, 104, 106, 108, 112, 116, 117, 120, 121, 122, 124, 125, 126, 127, 128, 129, 130, 131,
    132, 133, 134, 136, 140, 141, 142, 144, 145, 146, 147, 148, 149, 151, 153, 154, 155, 157, 158,
    159, 160, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178,
    180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 195, 196, 197, 198, 199, 200, 201,
    202, 203, 204, 205, 206, 207, 208, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 223, 224,
    242, 243, 244, 245, 246, 248, 256, 257, 259, 260, 262, 263, 264, 265, 266, 267, 268, 269, 271,
    281, 282, 283, 284, 286, 287, 308, 309, 310, 312, 313, 314, 315, 316, 317, 318, 319, 320, 321,
    322, 323, 324, 333, 344, 345, 346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358,
    360, 361, 362, 364, 365, 366, 367, 368, 369, 370, 371, 372, 373, 374, 375, 376, 377, 378, 379,
    380, 381, 382, 383, 384, 385, 386, 387, 388, 390, 392, 393, 394, 395, 396, 397, 398, 399, 401,
    402, 403, 404, 405, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421,
    422, 423, 424, 425, 426, 427, 428, 429, 430, 431, 432, 434, 435, 436, 437, 438, 439, 440, 441,
    442, 444, 447, 448, 449, 450, 451, 452, 453, 454, 455, 456, 457, 458, 459, 460, 461, 462, 463,
    466, 467, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477, 478, 479, 480, 481, 482, 483, 484,
    485, 486, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 501, 502, 503, 504, 505, 506,
    507, 508, 509, 510, 511, 516, 517, 518, 519, 521, 522, 524, 525, 526,
];

// END GENERATED: gen-top-ports

/// Deprecated alias for [`PRIORITY_PORTS_100`].
///
/// Renamed because "top" implied a frequency ranking that this list has never
/// legitimately had. The contents also changed: the ordering is now derived
/// from the IANA registry, not from measured-frequency data.
#[deprecated(
    since = "1.1.0",
    note = "renamed to PRIORITY_PORTS_100; the list is an IANA-derived editorial ranking, not a frequency ranking"
)]
pub const TOP_100_PORTS: &[u16] = PRIORITY_PORTS_100;

/// Deprecated alias for [`PRIORITY_PORTS_1000`].
///
/// See [`TOP_100_PORTS`] for why it was renamed.
#[deprecated(
    since = "1.1.0",
    note = "renamed to PRIORITY_PORTS_1000; the list is an IANA-derived editorial ranking, not a frequency ranking"
)]
pub const TOP_1000_PORTS: &[u16] = PRIORITY_PORTS_1000;

/// Get the first `n` ports of ProRT-IP's editorial priority ordering.
///
/// For `n <= 1000` this is a prefix of [`PRIORITY_PORTS_1000`]. For larger `n`
/// the list is padded with ports above 1000 that it does not already contain,
/// ascending, until `n` entries are available.
///
/// The ordering is derived from IANA port assignments, **not** from measured
/// frequency. See the [module documentation](self) before treating position in
/// this list as evidence of anything.
///
/// # Arguments
///
/// * `n` - Number of ports to return
///
/// # Examples
///
/// ```
/// // The first entry of every family, in family order
/// let ports = prtip_core::top_ports::get_priority_ports(12);
/// assert_eq!(ports.len(), 12);
/// assert_eq!(ports[0], 22); // SSH  - first of the remote-access family
/// assert_eq!(ports[1], 80); // HTTP - first of the web family
///
/// // Fast scan
/// let ports = prtip_core::top_ports::get_priority_ports(100);
/// assert_eq!(ports.len(), 100);
/// assert!(ports.contains(&443)); // HTTPS
/// assert!(ports.contains(&53)); // DNS
/// ```
pub fn get_priority_ports(n: usize) -> Vec<u16> {
    if n == 0 {
        return Vec::new();
    }

    if n <= PRIORITY_PORTS_100.len() {
        // Fast path: the 100-port prefix
        PRIORITY_PORTS_100[..n].to_vec()
    } else if n <= PRIORITY_PORTS_1000.len() {
        // Medium path: the 1000-port list
        PRIORITY_PORTS_1000[..n].to_vec()
    } else {
        // Fallback: everything ranked, then sequential ports to reach n.
        // Beyond 1000 the registry gives no further ranking signal, so the
        // padding is deliberately dumb.
        let mut ports = PRIORITY_PORTS_1000.to_vec();

        for port in (PRIORITY_PORTS_1000.len() as u16 + 1)..=65535 {
            if !ports.contains(&port) {
                ports.push(port);
                if ports.len() >= n {
                    break;
                }
            }
        }

        ports.truncate(n);
        ports
    }
}

/// Deprecated alias for [`get_priority_ports`].
///
/// Renamed because "top" implied a frequency ranking. Behaviour is identical.
#[deprecated(
    since = "1.1.0",
    note = "renamed to get_priority_ports; the ordering is an IANA-derived editorial ranking, not a frequency ranking"
)]
pub fn get_top_ports(n: usize) -> Vec<u16> {
    get_priority_ports(n)
}

/// Convert list of ports to a comma-separated port specification string
///
/// Used to convert a port list into a format compatible with `PortRange::parse()`
///
/// # Examples
///
/// ```
/// let ports = vec![80, 443, 8080];
/// let spec = prtip_core::top_ports::ports_to_spec(&ports);
/// assert_eq!(spec, "80,443,8080");
/// ```
pub fn ports_to_spec(ports: &[u16]) -> String {
    ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ports_100_defined() {
        assert_eq!(PRIORITY_PORTS_100.len(), 100);
    }

    #[test]
    fn test_priority_ports_1000_defined() {
        assert_eq!(PRIORITY_PORTS_1000.len(), 1000);
        // The 100-list must be a prefix of the 1000-list, not merely a subset:
        // get_priority_ports() relies on the prefix property.
        assert_eq!(&PRIORITY_PORTS_1000[..100], PRIORITY_PORTS_100);
    }

    #[test]
    fn test_priority_ports_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for port in PRIORITY_PORTS_1000 {
            assert!(seen.insert(port), "duplicate port {} in the list", port);
        }
    }

    #[test]
    fn test_priority_ports_are_valid() {
        // Port 0 is not a scannable assignment and must never be ranked.
        for port in PRIORITY_PORTS_1000 {
            assert!(*port > 0, "port 0 must not appear in the list");
        }
    }

    #[test]
    fn test_get_priority_ports_zero() {
        let ports = get_priority_ports(0);
        assert_eq!(ports.len(), 0);
    }

    #[test]
    fn test_get_priority_ports_small() {
        let ports = get_priority_ports(10);
        assert_eq!(ports.len(), 10);
        assert_eq!(ports, &PRIORITY_PORTS_100[..10]);
    }

    #[test]
    fn test_get_priority_ports_100() {
        let ports = get_priority_ports(100);
        assert_eq!(ports.len(), 100);
        assert_eq!(ports, PRIORITY_PORTS_100);
    }

    #[test]
    fn test_get_priority_ports_500() {
        let ports = get_priority_ports(500);
        assert_eq!(ports.len(), 500);
        assert_eq!(ports, &PRIORITY_PORTS_1000[..500]);
    }

    #[test]
    fn test_get_priority_ports_1000() {
        let ports = get_priority_ports(1000);
        assert_eq!(ports.len(), 1000);
        assert_eq!(ports, PRIORITY_PORTS_1000);
    }

    #[test]
    fn test_get_priority_ports_excessive() {
        let ports = get_priority_ports(2000);
        assert_eq!(ports.len(), 2000);
        for port in PRIORITY_PORTS_1000 {
            assert!(ports.contains(port));
        }
    }

    #[test]
    fn test_ports_to_spec_empty() {
        let ports = vec![];
        assert_eq!(ports_to_spec(&ports), "");
    }

    #[test]
    fn test_ports_to_spec_single() {
        let ports = vec![80];
        assert_eq!(ports_to_spec(&ports), "80");
    }

    #[test]
    fn test_ports_to_spec_multiple() {
        let ports = vec![80, 443, 8080];
        assert_eq!(ports_to_spec(&ports), "80,443,8080");
    }

    #[test]
    fn test_ports_to_spec_unsorted() {
        let ports = vec![443, 80, 8080];
        assert_eq!(ports_to_spec(&ports), "443,80,8080");
    }

    /// Ports whose family ranks them into the 100-port prefix.
    ///
    /// This is a regression guard on the generated output, not a claim that
    /// these are the ports most likely to be open. The rule that puts them here
    /// is `tools/gen-top-ports/RULE.md`; if a registry refresh moves one, the
    /// right response is to read the diff, not to pin the port.
    #[test]
    fn test_expected_ports_in_priority_100() {
        let expected = [
            20,   // ftp-data     - file-sharing
            21,   // ftp          - file-sharing
            22,   // ssh          - remote-access
            23,   // telnet       - remote-access
            25,   // smtp         - mail
            53,   // domain       - name-service
            80,   // http         - web
            110,  // pop3         - mail
            111,  // sunrpc       - management-discovery
            135,  // epmap        - management-discovery
            139,  // netbios-ssn  - file-sharing
            143,  // imap         - mail
            161,  // snmp         - management-discovery
            389,  // ldap         - directory-auth
            443,  // https        - web
            445,  // microsoft-ds - file-sharing
            515,  // printer      - print
            554,  // rtsp         - voice-video
            631,  // ipp          - web
            1883, // mqtt         - messaging
        ];
        for port in &expected {
            assert!(
                PRIORITY_PORTS_100.contains(port),
                "port {} should be in PRIORITY_PORTS_100",
                port
            );
        }
    }

    /// Ports a frequency-ordered list would rank highly and this one does not.
    ///
    /// Pinned deliberately: it is the honesty gate for the module's central
    /// claim. If this ever starts failing because these ports drifted into the
    /// first hundred, either the rule changed or someone reintroduced a
    /// frequency-derived ordering, and both need review.
    #[test]
    fn test_ordering_is_not_frequency_derived() {
        for port in &[3389u16, 8080] {
            assert!(
                !PRIORITY_PORTS_100.contains(port),
                "port {} is in PRIORITY_PORTS_100; the ordering is supposed to be \
                 IANA-derived, and an IANA-derived ordering does not rank it there",
                port
            );
            assert!(
                PRIORITY_PORTS_1000.contains(port),
                "port {} should still be within PRIORITY_PORTS_1000",
                port
            );
        }
    }

    #[test]
    fn test_deprecated_aliases_match() {
        #[allow(deprecated)]
        {
            assert_eq!(TOP_100_PORTS, PRIORITY_PORTS_100);
            assert_eq!(TOP_1000_PORTS, PRIORITY_PORTS_1000);
            assert_eq!(get_top_ports(50), get_priority_ports(50));
        }
    }
}
