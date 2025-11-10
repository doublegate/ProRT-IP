# Sprint 6.4: Adaptive Tuning & Memory-Mapped I/O Prep (QW-1 + QW-3 Prep)

**Status:** 📋 Planned (Q2 2026)
**Effort Estimate:** 10-14 hours
**Timeline:** Weeks 7-8 (2 weeks)
**Dependencies:** Sprint 6.3 (Network Optimization) COMPLETE
**Priority:** MEDIUM (Secondary Path)

## Sprint Overview

### Deliverables
1. **QW-1: Adaptive Batch Size Tuning** - 15-30% throughput gain (ROI 5.33)
2. **QW-3 Preparation: Memory-Mapped I/O Infrastructure** - Foundation for Sprint 6.6
3. **Auto-Tuning Configuration System** - Platform-specific defaults
4. **Performance Monitoring Dashboard** - Real-time tuning visualization (TUI integration)
5. **Documentation** - 28-ADAPTIVE-TUNING-GUIDE.md

### Strategic Value
- Highest ROI optimization (5.33) - automatic parameter tuning reduces operator expertise requirements
- Enables ProRT-IP to self-optimize for diverse network conditions (LAN vs WAN, fast vs slow targets)
- Memory-mapped I/O foundation for Sprint 6.6 (20-50% memory reduction)
- Differentiates from competitors requiring manual tuning (Masscan, ZMap)

### Integration Points
- **BatchSender/Receiver (Sprint 6.3):** Adaptive batch size control
- **EventBus:** Emit tuning decision events for TUI visualization
- **TUI Dashboard (Sprint 6.2):** Display current tuning parameters
- **Configuration System:** Platform-specific defaults, user overrides

---

## Task Breakdown

### Task Area 1: Adaptive Batch Size Tuning (QW-1) (5-6 hours)

**Task 1.1: Design adaptive tuning algorithm**
- File: `prtip-scanner/src/network/adaptive_tuner.rs`
- Algorithm: AIMD (Additive Increase, Multiplicative Decrease)
  - Start: batch_size = 64 (conservative)
  - Success: batch_size += 16 (additive increase) every 10 batches
  - Failure: batch_size *= 0.5 (multiplicative decrease) on packet loss
  - Max: 1024 (Linux limit), Min: 1 (fallback)
```rust
pub struct AdaptiveTuner {
    current_batch_size: usize,
    min_batch_size: usize,
    max_batch_size: usize,
    success_count: usize,
    failure_count: usize,
    increase_threshold: usize,  // Batches before increase
}

impl AdaptiveTuner {
    pub fn on_success(&mut self, packets_sent: usize) {
        self.success_count += 1;
        
        if self.success_count >= self.increase_threshold {
            // Additive increase
            self.current_batch_size = (self.current_batch_size + 16)
                .min(self.max_batch_size);
            self.success_count = 0;
        }
    }
    
    pub fn on_failure(&mut self) {
        self.failure_count += 1;
        
        // Multiplicative decrease
        self.current_batch_size = (self.current_batch_size / 2)
            .max(self.min_batch_size);
        self.success_count = 0;  // Reset success counter
    }
    
    pub fn get_batch_size(&self) -> usize {
        self.current_batch_size
    }
}
```
- **Estimated Time:** 2h

**Task 1.2: Detect packet loss and network congestion**
```rust
// Integrate with BatchSender
impl BatchSender {
    pub async fn send_batch_adaptive(&mut self, packets: &[Vec<u8>]) -> io::Result<AdaptiveResult> {
        let batch_size = self.tuner.get_batch_size();
        let sent = self.send_batch(&packets[..batch_size]).await?;
        
        if sent < batch_size {
            // Partial send = network congestion or buffer full
            self.tuner.on_failure();
            Ok(AdaptiveResult::PartialSend { sent, requested: batch_size })
        } else {
            self.tuner.on_success(sent);
            Ok(AdaptiveResult::FullSend { sent })
        }
    }
}
```
- Monitor socket buffer usage: `getsockopt(SO_SNDBUF)` for send buffer size
- Detect congestion: partial sends, EAGAIN frequency, RTT increases
- **Estimated Time:** 1.5h

**Task 1.3: Platform-specific defaults**
- File: `prtip-scanner/src/network/platform_defaults.rs`
- Linux: max_batch=1024, initial=64, increase_threshold=10
- macOS/BSD: max_batch=1 (no batching), initial=1
- Windows: max_batch=1 (no batching), initial=1
```rust
pub fn get_platform_defaults() -> TunerConfig {
    #[cfg(target_os = "linux")]
    return TunerConfig {
        initial_batch_size: 64,
        min_batch_size: 1,
        max_batch_size: 1024,
        increase_threshold: 10,
        decrease_factor: 0.5,
    };
    
    #[cfg(not(target_os = "linux"))]
    return TunerConfig {
        initial_batch_size: 1,
        min_batch_size: 1,
        max_batch_size: 1,
        increase_threshold: 1,
        decrease_factor: 1.0,
    };
}
```
- **Estimated Time:** 0.5h

**Task 1.4: Emit tuning events to EventBus**
```rust
// New event type
#[derive(Debug, Clone)]
pub struct AdaptiveTuningEvent {
    pub timestamp: Instant,
    pub old_batch_size: usize,
    pub new_batch_size: usize,
    pub reason: TuningReason,  // Success, Failure, Congestion
    pub success_count: usize,
    pub failure_count: usize,
}

// Emit on batch size change
if old_batch_size != new_batch_size {
    event_bus.publish(Event::AdaptiveTuning(AdaptiveTuningEvent {
        timestamp: Instant::now(),
        old_batch_size,
        new_batch_size,
        reason: if success { TuningReason::Success } else { TuningReason::Failure },
        success_count: self.tuner.success_count,
        failure_count: self.tuner.failure_count,
    }));
}
```
- TUI displays tuning graph: batch size over time
- **Estimated Time:** 1h

**Task 1.5: Write unit tests**
- Test AIMD algorithm (additive increase, multiplicative decrease)
- Test success threshold (10 batches before increase)
- Test failure recovery (decrease then stabilize)
- Test min/max limits (batch size ∈ [1, 1024])
- Test platform defaults (Linux vs macOS)
- **Target:** 10-12 tests
- **Estimated Time:** 1h

---

### Task Area 2: Memory-Mapped I/O Infrastructure (QW-3 Prep) (3-4 hours)

**Task 2.1: Design memory-mapped result file format**
- File: `prtip-core/src/output/mmap_format.rs`
- Binary format for efficient memory-mapped access:
```
Header (64 bytes):
  - Magic: "PRTIP\0\0\0" (8 bytes)
  - Version: u32 (4 bytes)
  - Entry count: u64 (8 bytes)
  - Entry size: u32 (4 bytes, fixed 128 bytes)
  - Reserved: 40 bytes

Entry (128 bytes):
  - Target IP: [u8; 16] (IPv6-compatible, 16 bytes)
  - Port: u16 (2 bytes)
  - Protocol: u8 (1 byte, 0=TCP, 1=UDP)
  - State: u8 (1 byte, 0=closed, 1=open, 2=filtered)
  - Service name: [u8; 32] (32 bytes, null-terminated)
  - Service version: [u8; 32] (32 bytes, null-terminated)
  - Banner: [u8; 40] (40 bytes, truncated)
  - Timestamp: u64 (8 bytes, UNIX timestamp)
  - Reserved: 2 bytes (alignment)
```
- Fixed-size entries enable O(1) random access
- **Estimated Time:** 1.5h

**Task 2.2: Create memory-mapped file writer**
```rust
use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;

pub struct MmapResultWriter {
    file: File,
    mmap: MmapMut,
    entry_count: usize,
    max_entries: usize,
}

impl MmapResultWriter {
    pub fn new(path: &str, max_entries: usize) -> io::Result<Self> {
        let file_size = HEADER_SIZE + (max_entries * ENTRY_SIZE);
        
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        file.set_len(file_size as u64)?;
        
        let mut mmap = unsafe {
            MmapOptions::new().len(file_size).map_mut(&file)?
        };
        
        // Initialize header
        Self::write_header(&mut mmap, max_entries);
        
        Ok(Self {
            file,
            mmap,
            entry_count: 0,
            max_entries,
        })
    }
    
    pub fn write_entry(&mut self, entry: &ScanResult) -> io::Result<()> {
        if self.entry_count >= self.max_entries {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Memory-mapped file full",
            ));
        }
        
        let offset = HEADER_SIZE + (self.entry_count * ENTRY_SIZE);
        Self::serialize_entry(&mut self.mmap[offset..], entry);
        
        self.entry_count += 1;
        Ok(())
    }
    
    pub fn flush(&mut self) -> io::Result<()> {
        self.mmap.flush()?;
        Ok(())
    }
}
```
- Auto-grow file if max_entries exceeded (double size)
- **Estimated Time:** 2h

**Task 2.3: Write unit tests**
- Test file creation with various sizes (1K, 10K, 100K entries)
- Test entry serialization/deserialization
- Test auto-grow on overflow
- Test header parsing
- **Target:** 6-8 tests
- **Estimated Time:** 1h

---

### Task Area 3: Auto-Tuning Configuration System (2-3 hours)

**Task 3.1: Create configuration schema**
- File: `prtip-core/src/config/tuning_config.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    // Batch size tuning
    pub adaptive_batching: bool,
    pub initial_batch_size: usize,
    pub max_batch_size: usize,
    pub increase_threshold: usize,
    
    // Rate limiting tuning
    pub adaptive_rate_limit: bool,
    pub initial_pps: u64,
    pub max_pps: u64,
    
    // Memory-mapped I/O
    pub use_mmap: bool,
    pub mmap_max_entries: usize,
    
    // Platform overrides
    pub platform_defaults: bool,
}

impl Default for TuningConfig {
    fn default() -> Self {
        let platform = get_platform_defaults();
        Self {
            adaptive_batching: true,
            initial_batch_size: platform.initial_batch_size,
            max_batch_size: platform.max_batch_size,
            increase_threshold: 10,
            adaptive_rate_limit: false,  // Future work
            initial_pps: 1000,
            max_pps: 100_000,
            use_mmap: false,  // Enabled in Sprint 6.6
            mmap_max_entries: 1_000_000,
            platform_defaults: true,
        }
    }
}
```
- **Estimated Time:** 1h

**Task 3.2: Integrate with CLI flags**
- File: `prtip/src/cli.rs`
- Add flags: `--adaptive-batch`, `--max-batch-size`, `--no-adaptive`, `--use-mmap`
```rust
#[derive(Parser)]
pub struct Cli {
    /// Enable adaptive batch size tuning (default: true on Linux)
    #[arg(long, default_value_t = true)]
    pub adaptive_batch: bool,
    
    /// Maximum batch size for sendmmsg (default: platform-specific)
    #[arg(long)]
    pub max_batch_size: Option<usize>,
    
    /// Disable all adaptive tuning
    #[arg(long)]
    pub no_adaptive: bool,
    
    /// Use memory-mapped I/O for results (experimental)
    #[arg(long)]
    pub use_mmap: bool,
}
```
- **Estimated Time:** 1h

**Task 3.3: Write configuration tests**
- Test default config (platform-specific)
- Test CLI override (--max-batch-size 512)
- Test disable adaptive (--no-adaptive)
- **Target:** 5-6 tests
- **Estimated Time:** 0.5h

---

### Task Area 4: TUI Performance Monitoring (1-2 hours)

**Task 4.1: Create AdaptiveTuningWidget**
- File: `prtip-tui/src/widgets/adaptive_tuning.rs`
- Display current tuning parameters:
  - Batch size: [Current: 256, Range: 64-1024]
  - Success rate: 95.2% (last 100 batches)
  - Tuning events: +16 (10 success), -128 (congestion detected)
- Line graph: Batch size over time (last 60 seconds)
- **Estimated Time:** 1.5h

**Task 4.2: Subscribe to AdaptiveTuningEvent**
```rust
let mut tuning_rx = app.event_bus.subscribe_typed::<AdaptiveTuningEvent>();

tokio::select! {
    Some(tuning) = tuning_rx.recv() => {
        app.ui_state.adaptive_widget.add_event(tuning);
        // Update graph with new batch size
        app.ui_state.adaptive_widget.update_graph(tuning.new_batch_size);
    }
}
```
- **Estimated Time:** 0.5h

**Task 4.3: Write widget tests**
- Test event subscription
- Test graph updates
- **Target:** 3-4 tests
- **Estimated Time:** 0.5h

---

### Task Area 5: Documentation (1-2 hours)

**Task 5.1: Create adaptive tuning guide**
- File: `docs/28-ADAPTIVE-TUNING-GUIDE.md` (800-1,000 lines)
- Sections:
  1. Overview (why adaptive tuning, expected gains)
  2. AIMD Algorithm (additive increase, multiplicative decrease)
  3. Platform Defaults (Linux, macOS, Windows)
  4. Configuration Options (CLI flags, config file)
  5. Performance Monitoring (TUI widget)
  6. Memory-Mapped I/O Preparation (Sprint 6.6 foundation)
  7. Troubleshooting (packet loss, congestion)
- **Estimated Time:** 1.5h

**Task 5.2: Update CHANGELOG.md**
- Add entry for Sprint 6.4 completion
- Highlight: 15-30% throughput gain, automatic tuning
- **Estimated Time:** 0.5h

---

## Definition of Done

### Functional Requirements
- [ ] Adaptive batch size tuning working (AIMD algorithm)
- [ ] Platform-specific defaults applied automatically
- [ ] Memory-mapped I/O file format defined and tested
- [ ] TUI widget displays tuning parameters in real-time
- [ ] CLI flags for configuration (`--adaptive-batch`, `--max-batch-size`, `--use-mmap`)
- [ ] EventBus emits AdaptiveTuningEvent on parameter changes

### Quality Requirements
- [ ] 24-28 tests passing (100% success rate)
- [ ] Zero clippy warnings (--deny warnings)
- [ ] Zero rustdoc warnings
- [ ] Code formatted with `cargo fmt`
- [ ] No regressions in existing scan accuracy

### Documentation Requirements
- [ ] 28-ADAPTIVE-TUNING-GUIDE.md complete (800-1,000 lines)
- [ ] Rustdoc comments for all public APIs
- [ ] Configuration examples in guide
- [ ] TUI widget usage documented

### Performance Requirements
- [ ] Throughput improvement: 15-30% (60K pps → 69K-78K pps)
- [ ] Adaptive tuning overhead: <1% (negligible)
- [ ] Batch size stabilization: <30 seconds to optimal
- [ ] Memory-mapped I/O file creation: <100ms for 1M entries

---

## Testing Plan

### Unit Tests (15-18 tests)
```bash
cargo test -p prtip-scanner network::adaptive_
cargo test -p prtip-core output::mmap_
cargo test -p prtip config::tuning_
```

**Test Cases:**
1. AdaptiveTuner: additive increase after 10 successes
2. AdaptiveTuner: multiplicative decrease on failure
3. AdaptiveTuner: min/max limits enforced
4. AdaptiveTuner: success counter reset on failure
5. AdaptiveTuner: platform defaults (Linux)
6. AdaptiveTuner: platform defaults (macOS)
7. MmapResultWriter: create file with 1K entries
8. MmapResultWriter: write entry at offset 0
9. MmapResultWriter: write entry at offset 1000
10. MmapResultWriter: auto-grow on overflow
11. MmapResultWriter: header parsing
12. MmapFormat: serialize/deserialize entry
13. TuningConfig: default config (platform-specific)
14. TuningConfig: CLI override (--max-batch-size)
15. TuningConfig: disable adaptive (--no-adaptive)

### Integration Tests (9-10 tests)
```bash
cargo test -p prtip-scanner --test integration_adaptive
```

**Test Cases:**
1. Full Scan: adaptive tuning vs fixed batch (measure improvement)
2. EventBus: AdaptiveTuningEvent emitted on batch size change
3. TUI Integration: AdaptiveTuningWidget updates in real-time
4. Configuration: CLI flags override defaults
5. Platform Detection: Linux uses batching, macOS uses fallback
6. Congestion Handling: batch size decreases on packet loss
7. Recovery: batch size increases after stabilization
8. Memory-Mapped I/O: write 1K entries, verify file size
9. Memory-Mapped I/O: read back entries, verify correctness

### Manual Testing Checklist
- [ ] **Linux:** Adaptive tuning increases batch size over time (64 → 256)
- [ ] **Linux:** Batch size decreases on packet loss (256 → 128)
- [ ] **macOS:** Graceful fallback to batch_size=1
- [ ] **TUI:** AdaptiveTuningWidget displays current parameters
- [ ] **TUI:** Graph shows batch size over time (last 60s)
- [ ] **CLI:** `--max-batch-size 512` limits batch size to 512
- [ ] **CLI:** `--no-adaptive` disables tuning (fixed batch size)
- [ ] **Mmap:** Create file with 1M entries (<100ms)
- [ ] **Mmap:** Verify file size (64 bytes header + 1M × 128 bytes entries = ~128MB)

---

## Dependencies

### External Crates
- `memmap2 = "0.9"` - Memory-mapped file I/O
- `serde = { version = "1.0", features = ["derive"] }` - Configuration serialization

### Internal Dependencies
- **Sprint 6.3 (Network Optimization):** BatchSender/Receiver integration
- **Sprint 6.2 (Live Dashboard):** TUI widget framework
- **Sprint 5.5.3 (EventBus):** AdaptiveTuningEvent distribution

---

## Risk Mitigation

### Risk 1: AIMD Oscillation
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Use conservative increase threshold (10 batches)
- Monitor success rate (>90% = stable)
- Allow manual override (`--max-batch-size`)

### Risk 2: Memory-Mapped File Corruption
**Impact:** High | **Probability:** Low
**Mitigation:**
- Flush mmap on every 1K entries
- Use fixed-size entries (no variable-length fields)
- Add checksum to header (future work)

### Risk 3: Platform-Specific Bugs
**Impact:** Low | **Probability:** Low
**Mitigation:**
- Test on all 3 platforms (Linux, macOS, Windows)
- Graceful fallback on unsupported features

---

## Resources

### Documentation
- **AIMD Algorithm:** RFC 5681 (TCP Congestion Control)
- **Memory-Mapped I/O:** `man 2 mmap`
- **memmap2 crate:** https://docs.rs/memmap2/

### Reference Implementations
- **TCP Congestion Control:** Linux kernel net/ipv4/tcp_cubic.c
- **Masscan Adaptive Rate Limiting:** transmit.c (adaptive pps)

---

## Sprint Completion Report Template

```markdown
# Sprint 6.4 Completion Report

**Date:** [YYYY-MM-DD]
**Actual Duration:** [X] hours
**Status:** ✅ COMPLETE / ⚠️ PARTIAL / ❌ INCOMPLETE

## Deliverables Status
- [ ] Adaptive Batch Size Tuning (AIMD)
- [ ] Memory-Mapped I/O Infrastructure
- [ ] Auto-Tuning Configuration System
- [ ] TUI Performance Monitoring Widget
- [ ] Documentation (28-ADAPTIVE-TUNING-GUIDE.md)

## Test Results
- Unit Tests: [X/18] passing
- Integration Tests: [X/10] passing

## Performance Metrics
- Throughput Improvement: [X]% (target: 15-30%)
- Batch Size Stabilization: [X]s (target: <30s)
- Mmap File Creation: [X]ms (target: <100ms)

## Issues Encountered
1. [Issue description] - **Resolution:** [How resolved]

## Lessons Learned
- [Key insight from AIMD tuning]

## Next Sprint Preparation
- Dependencies ready for Sprint 6.5: ✅/❌
```

---

**This sprint delivers automatic performance tuning - operators no longer need expertise to optimize ProRT-IP. Focus on stability (no oscillation) and smooth integration with existing systems.**
