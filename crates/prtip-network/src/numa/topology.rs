//! NUMA topology detection
//!
//! Detects NUMA nodes and CPU core layout using hwloc library.
//! Falls back gracefully to single-node on non-NUMA systems or unsupported platforms.

#[cfg(all(target_os = "linux", feature = "numa"))]
use super::error::NumaError;
use super::error::Result;
use std::collections::HashMap;
use tracing::debug;
#[cfg(all(target_os = "linux", feature = "numa"))]
use tracing::{info, warn};

/// NUMA topology information
///
/// Represents the NUMA architecture of the system:
/// - `SingleNode`: UMA system or NUMA unavailable (fallback)
/// - `MultiNode`: Multi-socket system with NUMA nodes
#[derive(Debug, Clone)]
pub enum NumaTopology {
    /// Single NUMA node (UMA system or fallback)
    SingleNode,
    /// Multiple NUMA nodes with core mapping
    MultiNode {
        /// Number of NUMA nodes detected
        node_count: usize,
        /// Map from NUMA node ID to list of CPU core IDs
        cores_per_node: HashMap<usize, Vec<usize>>,
    },
}

impl NumaTopology {
    /// Detect NUMA topology
    ///
    /// Attempts to detect the NUMA topology of the system using hwloc.
    /// Falls back to `SingleNode` on:
    /// - Non-Linux platforms
    /// - Single-socket systems
    /// - Detection failures
    ///
    /// # Returns
    ///
    /// - `Ok(NumaTopology)` - Detected topology (or fallback)
    /// - `Err(NumaError)` - Only on critical failures (rare)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::numa::NumaTopology;
    ///
    /// let topology = NumaTopology::detect().unwrap();
    /// println!("Detected {} NUMA node(s)", topology.node_count());
    /// ```
    pub fn detect() -> Result<Self> {
        #[cfg(all(target_os = "linux", feature = "numa"))]
        {
            match Self::detect_linux() {
                Ok(topo) => Ok(topo),
                Err(e) => {
                    warn!("NUMA detection failed: {}, falling back to single-node", e);
                    Ok(Self::SingleNode)
                }
            }
        }

        #[cfg(not(all(target_os = "linux", feature = "numa")))]
        {
            debug!("NUMA not supported on this platform, using single-node");
            Ok(Self::SingleNode)
        }
    }

    /// Detect NUMA topology on Linux using hwloc
    #[cfg(all(target_os = "linux", feature = "numa"))]
    fn detect_linux() -> Result<Self> {
        use hwlocality::{object::types::ObjectType, Topology};

        // Initialize hwloc topology
        let topo = match Topology::new() {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    "Failed to initialize hwloc topology: {}, falling back to single-node",
                    e
                );
                return Ok(Self::SingleNode);
            }
        };

        // Get NUMA nodes
        let numa_depth = match topo.depth_or_below_for_type(ObjectType::NUMANode) {
            Ok(depth) => depth,
            Err(_) => {
                warn!("Failed to query NUMA depth, falling back to single-node");
                return Ok(Self::SingleNode);
            }
        };

        let node_objs: Vec<_> = topo.objects_at_depth(numa_depth).collect();
        let node_count = node_objs.len();

        debug!(
            "hwloc detected {} NUMA nodes at depth {}",
            node_count, numa_depth
        );

        if node_count <= 1 {
            info!("Single NUMA node detected, optimization not needed");
            return Ok(Self::SingleNode);
        }

        // Build core mapping for each NUMA node
        let mut cores_per_node: HashMap<usize, Vec<usize>> = HashMap::new();

        for node_obj in node_objs.iter() {
            let node_id = match node_obj.os_index() {
                Some(idx) => idx as usize,
                None => {
                    warn!("NUMA node without os_index, skipping");
                    continue;
                }
            };
            let cores = Self::get_cores_for_node(&topo, node_obj)?;

            debug!("NUMA node {}: {} cores ({:?})", node_id, cores.len(), cores);

            cores_per_node.insert(node_id, cores);
        }

        info!(
            "NUMA topology detected: {} nodes, {} total cores",
            node_count,
            cores_per_node.values().map(|v| v.len()).sum::<usize>()
        );

        Ok(Self::MultiNode {
            node_count,
            cores_per_node,
        })
    }

    /// Get list of CPU cores for a specific NUMA node
    #[cfg(all(target_os = "linux", feature = "numa"))]
    fn get_cores_for_node(
        topo: &hwlocality::Topology,
        node_obj: &hwlocality::object::TopologyObject,
    ) -> Result<Vec<usize>> {
        use hwlocality::object::types::ObjectType;

        let mut cores = Vec::new();

        // Get cpuset for this NUMA node
        let cpuset = node_obj
            .cpuset()
            .ok_or_else(|| NumaError::Detection("NUMA node has no cpuset".to_string()))?;

        // Find PU (Processing Unit / logical CPU) depth
        let pu_depth = match topo.depth_or_below_for_type(ObjectType::PU) {
            Ok(depth) => depth,
            Err(_) => {
                return Err(NumaError::Detection("Failed to query PU depth".to_string()));
            }
        };

        let pu_objs: Vec<_> = topo.objects_at_depth(pu_depth).collect();

        // Iterate through all PUs and check if they belong to this node
        for pu_obj in pu_objs {
            if let Some(pu_cpuset) = pu_obj.cpuset() {
                // Check if this PU is in the node's cpuset
                // Use is_set() to check if any bit in the PU's cpuset is set in node's cpuset
                let mut is_in_node = false;
                for i in 0..pu_cpuset.weight() as usize {
                    if cpuset.is_set(i as u32) && pu_cpuset.is_set(i as u32) {
                        is_in_node = true;
                        break;
                    }
                }

                if is_in_node {
                    if let Some(os_idx) = pu_obj.os_index() {
                        cores.push(os_idx as usize);
                    }
                }
            }
        }

        if cores.is_empty() {
            return Err(NumaError::Detection(
                "No cores found for NUMA node".to_string(),
            ));
        }

        Ok(cores)
    }

    /// Check if this is a multi-node NUMA system
    ///
    /// # Returns
    ///
    /// - `true` - Multi-socket NUMA system
    /// - `false` - Single-node UMA system
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::numa::NumaTopology;
    ///
    /// let topology = NumaTopology::detect().unwrap();
    /// if topology.is_multi_node() {
    ///     println!("NUMA optimization available");
    /// }
    /// ```
    pub fn is_multi_node(&self) -> bool {
        matches!(self, Self::MultiNode { .. })
    }

    /// Get number of NUMA nodes
    ///
    /// # Returns
    ///
    /// - `1` - Single node (or fallback)
    /// - `2+` - Number of NUMA nodes
    pub fn node_count(&self) -> usize {
        match self {
            Self::SingleNode => 1,
            Self::MultiNode { node_count, .. } => *node_count,
        }
    }

    /// Get cores for a specific NUMA node
    ///
    /// # Arguments
    ///
    /// * `node_id` - NUMA node ID (0-based)
    ///
    /// # Returns
    ///
    /// - `Some(&[usize])` - List of core IDs for this node
    /// - `None` - Invalid node ID or single-node system
    pub fn cores_for_node(&self, node_id: usize) -> Option<&[usize]> {
        match self {
            Self::SingleNode => None,
            Self::MultiNode { cores_per_node, .. } => {
                cores_per_node.get(&node_id).map(|v| v.as_slice())
            }
        }
    }

    /// Get total number of cores across all nodes
    pub fn total_cores(&self) -> usize {
        match self {
            Self::SingleNode => {
                // Fall back to detecting available parallelism
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            }
            Self::MultiNode { cores_per_node, .. } => {
                cores_per_node.values().map(|v| v.len()).sum()
            }
        }
    }
}

impl Default for NumaTopology {
    fn default() -> Self {
        Self::SingleNode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numa_detection() {
        // Should always succeed (fallback to SingleNode if needed)
        let topo = NumaTopology::detect().unwrap();
        assert!(topo.node_count() > 0);
        assert!(topo.total_cores() > 0);
    }

    #[test]
    fn test_single_node_topology() {
        let topo = NumaTopology::SingleNode;
        assert_eq!(topo.node_count(), 1);
        assert!(!topo.is_multi_node());
        assert!(topo.cores_for_node(0).is_none());
        assert!(topo.total_cores() > 0); // Should detect available parallelism
    }

    #[test]
    fn test_multi_node_topology() {
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0, 1, 2, 3]);
        cores_per_node.insert(1, vec![4, 5, 6, 7]);

        let topo = NumaTopology::MultiNode {
            node_count: 2,
            cores_per_node,
        };

        assert_eq!(topo.node_count(), 2);
        assert!(topo.is_multi_node());
        assert_eq!(topo.total_cores(), 8);
        assert_eq!(topo.cores_for_node(0), Some(&[0, 1, 2, 3][..]));
        assert_eq!(topo.cores_for_node(1), Some(&[4, 5, 6, 7][..]));
        assert_eq!(topo.cores_for_node(2), None);
    }

    #[test]
    fn test_default_topology() {
        let topo = NumaTopology::default();
        assert_eq!(topo.node_count(), 1);
        assert!(!topo.is_multi_node());
    }

    #[test]
    fn test_topology_edge_cases() {
        // Empty cores (should not happen in practice, but test robustness)
        let topo = NumaTopology::MultiNode {
            node_count: 1,
            cores_per_node: HashMap::new(),
        };
        assert_eq!(topo.total_cores(), 0);

        // Single core per node
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0]);
        cores_per_node.insert(1, vec![1]);

        let topo = NumaTopology::MultiNode {
            node_count: 2,
            cores_per_node,
        };
        assert_eq!(topo.total_cores(), 2);
    }

    #[cfg(all(target_os = "linux", feature = "numa"))]
    #[test]
    fn test_linux_numa_detection() {
        // This test only runs on Linux with NUMA feature
        // It may detect SingleNode or MultiNode depending on hardware
        let topo = NumaTopology::detect().unwrap();

        match topo {
            NumaTopology::SingleNode => {
                println!("Single-node system detected (expected on most test systems)");
            }
            NumaTopology::MultiNode {
                node_count,
                cores_per_node,
            } => {
                println!(
                    "Multi-node system detected: {} nodes, {} cores",
                    node_count,
                    cores_per_node.values().map(|v| v.len()).sum::<usize>()
                );

                // Validate detected topology
                assert!(node_count >= 2);
                assert!(!cores_per_node.is_empty());

                for (node_id, cores) in &cores_per_node {
                    println!("  Node {}: {:?}", node_id, cores);
                    assert!(!cores.is_empty());
                }
            }
        }
    }
}
