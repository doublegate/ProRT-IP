//! Thread pinning and CPU affinity for NUMA optimization
//!
//! Provides thread-to-core pinning using `sched_setaffinity` to improve
//! cache locality and reduce cross-socket latency.

use super::error::{NumaError, Result};
use super::topology::NumaTopology;
use nix::sched::{sched_getaffinity, sched_setaffinity, CpuSet};
use nix::unistd::Pid;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, info};

/// NUMA-aware thread pinning manager
///
/// Manages CPU core allocation and thread pinning across NUMA nodes.
/// Tracks allocated cores to avoid double-allocation and contention.
///
/// # Thread Safety
///
/// This struct is Send + Sync and can be shared across threads via Arc.
///
/// # Example
///
/// ```no_run
/// use prtip_network::numa::{NumaTopology, NumaManager};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let topology = NumaTopology::detect()?;
/// let manager = NumaManager::new(topology)?;
///
/// // Pin TX thread to core on node 0 (near NIC)
/// let tx_core = manager.allocate_core(Some(0))?;
/// manager.pin_current_thread(tx_core)?;
///
/// // Pin RX thread to different core on node 0
/// let rx_core = manager.allocate_core(Some(0))?;
/// manager.pin_current_thread(rx_core)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct NumaManager {
    /// NUMA topology
    topology: NumaTopology,
    /// Cores already allocated (protected by mutex)
    allocated_cores: Arc<Mutex<Vec<usize>>>,
}

impl NumaManager {
    /// Create new NUMA manager
    ///
    /// # Arguments
    ///
    /// * `topology` - Detected NUMA topology
    ///
    /// # Returns
    ///
    /// - `Ok(NumaManager)` - Manager created successfully
    /// - `Err(NumaError::NotAvailable)` - Single-node system (NUMA not needed)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::numa::{NumaTopology, NumaManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let topology = NumaTopology::detect()?;
    ///
    /// if topology.is_multi_node() {
    ///     let manager = NumaManager::new(topology)?;
    ///     println!("NUMA manager created for {} nodes", manager.node_count());
    /// } else {
    ///     println!("Single-node system, NUMA not needed");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(topology: NumaTopology) -> Result<Self> {
        if !topology.is_multi_node() {
            return Err(NumaError::NotAvailable);
        }

        Ok(Self {
            topology,
            allocated_cores: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Allocate a CPU core from the specified NUMA node
    ///
    /// Returns the first available (unallocated) core from the requested node.
    /// Cores are tracked to avoid double-allocation and contention.
    ///
    /// # Arguments
    ///
    /// * `preferred_node` - NUMA node ID (None = any node)
    ///
    /// # Returns
    ///
    /// - `Ok(usize)` - Allocated core ID
    /// - `Err(NumaError)` - No cores available or invalid node
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::numa::{NumaTopology, NumaManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let topology = NumaTopology::detect()?;
    /// # let manager = NumaManager::new(topology)?;
    /// // Allocate core on node 0 (near NIC)
    /// let core = manager.allocate_core(Some(0))?;
    /// println!("Allocated core {} on node 0", core);
    /// # Ok(())
    /// # }
    /// ```
    pub fn allocate_core(&self, preferred_node: Option<usize>) -> Result<usize> {
        let mut allocated = self.allocated_cores.lock();

        match &self.topology {
            NumaTopology::MultiNode { cores_per_node, .. } => {
                // Determine target node
                let node = preferred_node.unwrap_or(0);

                // Get cores for this node
                let cores = cores_per_node.get(&node).ok_or_else(|| {
                    NumaError::InvalidNode(format!(
                        "Node {} not found (available: {:?})",
                        node,
                        cores_per_node.keys().collect::<Vec<_>>()
                    ))
                })?;

                // Find first unallocated core
                for &core in cores {
                    if !allocated.contains(&core) {
                        allocated.push(core);
                        debug!("Allocated core {} from NUMA node {}", core, node);
                        return Ok(core);
                    }
                }

                Err(NumaError::Pinning(format!(
                    "No available cores on NUMA node {} (all {} cores allocated)",
                    node,
                    cores.len()
                )))
            }
            NumaTopology::SingleNode => Err(NumaError::NotAvailable),
        }
    }

    /// Pin current thread to specified CPU core
    ///
    /// Uses `sched_setaffinity` to pin the calling thread to a specific core.
    /// Requires `CAP_SYS_NICE` capability on Linux.
    ///
    /// # Arguments
    ///
    /// * `core_id` - CPU core ID (from `allocate_core()`)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Thread pinned successfully
    /// - `Err(NumaError::Pinning)` - Permission denied or invalid core
    ///
    /// # Errors
    ///
    /// **Permission denied:** Requires `CAP_SYS_NICE` capability.
    /// ```bash
    /// sudo setcap cap_sys_nice+ep /usr/bin/prtip
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// use prtip_network::numa::{NumaTopology, NumaManager};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let topology = NumaTopology::detect()?;
    /// # let manager = NumaManager::new(topology)?;
    /// let core = manager.allocate_core(Some(0))?;
    ///
    /// // Pin current thread
    /// if let Err(e) = manager.pin_current_thread(core) {
    ///     eprintln!("Warning: Failed to pin thread: {}", e);
    ///     // Continue without pinning (graceful degradation)
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn pin_current_thread(&self, core_id: usize) -> Result<()> {
        pin_thread_to_core(core_id)
    }

    /// Get number of NUMA nodes
    pub fn node_count(&self) -> usize {
        self.topology.node_count()
    }

    /// Get total cores across all nodes
    pub fn total_cores(&self) -> usize {
        self.topology.total_cores()
    }

    /// Get number of allocated cores
    pub fn allocated_count(&self) -> usize {
        self.allocated_cores.lock().len()
    }
}

/// Pin current thread to specified CPU core
///
/// Low-level function to pin the calling thread to a specific CPU core.
/// Uses `sched_setaffinity` system call.
///
/// # Arguments
///
/// * `core_id` - CPU core ID (0-based)
///
/// # Returns
///
/// - `Ok(())` - Thread pinned successfully
/// - `Err(NumaError::Pinning)` - Failed to pin (permission or invalid core)
///
/// # Platform Support
///
/// - **Linux:** Supported via `sched_setaffinity`
/// - **macOS/Windows:** Not supported (compile-time error)
///
/// # Example
///
/// ```no_run
/// use prtip_network::numa::pin_thread_to_core;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Pin to core 2
/// pin_thread_to_core(2)?;
/// println!("Thread pinned to core 2");
/// # Ok(())
/// # }
/// ```
pub fn pin_thread_to_core(core_id: usize) -> Result<()> {
    let mut cpu_set = CpuSet::new();

    // Set the target core
    cpu_set
        .set(core_id)
        .map_err(|e| NumaError::InvalidCore(format!("Invalid core ID {}: {}", core_id, e)))?;

    // Pin current thread (PID 0 = calling thread)
    sched_setaffinity(Pid::from_raw(0), &cpu_set).map_err(|e| {
        let msg = format!("Failed to set CPU affinity for core {}: {}", core_id, e);

        // Check for common permission error
        if e == nix::errno::Errno::EPERM {
            NumaError::Pinning(format!(
                "{}. Requires CAP_SYS_NICE capability. Run: sudo setcap cap_sys_nice+ep /usr/bin/prtip",
                msg
            ))
        } else {
            NumaError::Pinning(msg)
        }
    })?;

    info!("Thread pinned to CPU core {}", core_id);
    Ok(())
}

/// Get current thread's CPU affinity
///
/// Returns the list of CPU cores the calling thread is allowed to run on.
///
/// # Returns
///
/// - `Ok(Vec<usize>)` - List of core IDs
/// - `Err(NumaError::Pinning)` - Failed to query affinity
///
/// # Example
///
/// ```no_run
/// use prtip_network::numa::get_current_affinity;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let cores = get_current_affinity()?;
/// println!("Thread can run on cores: {:?}", cores);
/// # Ok(())
/// # }
/// ```
pub fn get_current_affinity() -> Result<Vec<usize>> {
    let cpu_set = sched_getaffinity(Pid::from_raw(0))
        .map_err(|e| NumaError::Pinning(format!("Failed to get CPU affinity: {}", e)))?;

    let mut cores = Vec::new();
    for cpu in 0..CpuSet::count() {
        if cpu_set.is_set(cpu).unwrap_or(false) {
            cores.push(cpu);
        }
    }

    Ok(cores)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_numa_manager_requires_multi_node() {
        let single_node = NumaTopology::SingleNode;
        let result = NumaManager::new(single_node);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumaError::NotAvailable));
    }

    #[test]
    fn test_numa_manager_creation() {
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0, 1, 2, 3]);
        cores_per_node.insert(1, vec![4, 5, 6, 7]);

        let topology = NumaTopology::MultiNode {
            node_count: 2,
            cores_per_node,
        };

        let manager = NumaManager::new(topology).unwrap();
        assert_eq!(manager.node_count(), 2);
        assert_eq!(manager.total_cores(), 8);
        assert_eq!(manager.allocated_count(), 0);
    }

    #[test]
    fn test_core_allocation() {
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0, 1, 2, 3]);
        cores_per_node.insert(1, vec![4, 5, 6, 7]);

        let topology = NumaTopology::MultiNode {
            node_count: 2,
            cores_per_node,
        };

        let manager = NumaManager::new(topology).unwrap();

        // Allocate from node 0
        let core1 = manager.allocate_core(Some(0)).unwrap();
        assert!(core1 <= 3); // Should be from node 0

        // Allocate from node 1
        let core2 = manager.allocate_core(Some(1)).unwrap();
        assert!((4..=7).contains(&core2)); // Should be from node 1

        // Verify allocation count
        assert_eq!(manager.allocated_count(), 2);
    }

    #[test]
    fn test_core_allocation_avoids_duplicates() {
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0, 1]);

        let topology = NumaTopology::MultiNode {
            node_count: 1,
            cores_per_node,
        };

        let manager = NumaManager::new(topology).unwrap();

        // Allocate both cores
        let core1 = manager.allocate_core(Some(0)).unwrap();
        let core2 = manager.allocate_core(Some(0)).unwrap();

        // Should be different cores
        assert_ne!(core1, core2);

        // Third allocation should fail (exhausted)
        let result = manager.allocate_core(Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_node_allocation() {
        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, vec![0, 1]);

        let topology = NumaTopology::MultiNode {
            node_count: 1,
            cores_per_node,
        };

        let manager = NumaManager::new(topology).unwrap();

        // Try to allocate from non-existent node
        let result = manager.allocate_core(Some(99));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NumaError::InvalidNode(_)));
    }

    #[test]
    fn test_get_current_affinity() {
        // This test should work on any Linux system
        let result = get_current_affinity();

        // May fail on non-Linux or containers with restricted permissions
        if let Ok(cores) = result {
            println!("Current thread affinity: {:?}", cores);
            assert!(!cores.is_empty());
        } else {
            println!("get_current_affinity failed (expected on non-Linux or restricted env)");
        }
    }

    #[test]
    fn test_pin_thread_to_core() {
        // This test requires CAP_SYS_NICE capability
        // It may fail with EPERM in CI environments
        let result = pin_thread_to_core(0);

        match result {
            Ok(_) => {
                println!("Thread pinned successfully");

                // Verify pinning
                if let Ok(cores) = get_current_affinity() {
                    assert_eq!(cores, vec![0]);
                }
            }
            Err(NumaError::Pinning(msg)) if msg.contains("CAP_SYS_NICE") => {
                println!("Thread pinning requires privileges (expected in CI)");
            }
            Err(e) => {
                println!(
                    "Thread pinning failed: {} (may be expected in CI/containers)",
                    e
                );
            }
        }
    }

    #[test]
    fn test_numa_manager_concurrent_allocation() {
        use std::sync::Arc;
        use std::thread;

        let mut cores_per_node = HashMap::new();
        cores_per_node.insert(0, (0..8).collect());

        let topology = NumaTopology::MultiNode {
            node_count: 1,
            cores_per_node,
        };

        let manager = Arc::new(NumaManager::new(topology).unwrap());

        // Spawn multiple threads allocating cores
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let mgr = Arc::clone(&manager);
                thread::spawn(move || mgr.allocate_core(Some(0)))
            })
            .collect();

        // Collect results
        let mut allocated_cores = Vec::new();
        for handle in handles {
            if let Ok(Ok(core)) = handle.join() {
                allocated_cores.push(core);
            }
        }

        // Should have allocated 4 different cores
        assert_eq!(allocated_cores.len(), 4);

        // All cores should be unique
        allocated_cores.sort();
        allocated_cores.dedup();
        assert_eq!(allocated_cores.len(), 4);
    }
}
