# Sprint 6.6: Advanced Features & Memory-Mapped I/O (QW-3 Complete)

**Status:** 📋 Planned (Q2 2026)
**Effort Estimate:** 15-20 hours
**Timeline:** Weeks 10-11 (2 weeks)
**Dependencies:** Sprint 6.4 (Adaptive Tuning) COMPLETE, Sprint 6.5 (Interactive Selection) COMPLETE
**Priority:** MEDIUM (Critical Path)

## Sprint Overview

### Deliverables
1. **QW-3 Completion: Memory-Mapped I/O** - 20-50% memory reduction (ROI 3.75)
2. **Scan History & Resume** - Pause/resume scans, historical scan logs
3. **Export Enhancements** - Multiple formats (CSV, PDF, HTML reports)
4. **Real-Time Filtering** - Filter live results during scan
5. **Performance Dashboard** - CPU/memory/network metrics visualization

### Strategic Value
- Memory-mapped I/O enables large-scale scans (10M+ targets) on constrained systems
- Scan history provides audit trail for compliance (GDPR, PCI-DSS)
- Resume capability prevents data loss on network interruptions
- Export formats enable integration with reporting tools (SIEM, ticketing)

### Integration Points
- **Mmap Infrastructure (Sprint 6.4):** Complete memory-mapped I/O implementation
- **EventBus:** ScanPauseEvent, ScanResumeEvent, ExportCompleteEvent
- **TUI Dashboard:** Display performance metrics in real-time
- **Configuration System:** Scan history storage location

---

## Task Breakdown

### Task Area 1: Memory-Mapped I/O Completion (QW-3) (6-8 hours)

**Task 1.1: Integrate mmap writer with scanner**
- File: `prtip-scanner/src/output/mmap_output.rs`
```rust
pub struct MmapOutputHandler {
    writer: MmapResultWriter,
    event_bus: Arc<EventBus>,
    buffer: Vec<ScanResult>,
    buffer_size: usize,
}

impl MmapOutputHandler {
    pub async fn handle_result(&mut self, result: ScanResult) -> io::Result<()> {
        self.buffer.push(result);
        
        // Flush buffer when full
        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> io::Result<()> {
        for result in self.buffer.drain(..) {
            self.writer.write_entry(&result)?;
        }
        self.writer.flush()?;
        
        // Emit event for TUI
        self.event_bus.publish(Event::MmapFlushed {
            entries_written: self.buffer_size,
        });
        
        Ok(())
    }
}
```
- Buffer size: 1K entries (trade-off: memory vs flush frequency)
- **Estimated Time:** 2h

**Task 1.2: Add mmap reader for result analysis**
```rust
pub struct MmapResultReader {
    mmap: Mmap,
    entry_count: usize,
    entry_size: usize,
}

impl MmapResultReader {
    pub fn open(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        let (entry_count, entry_size) = Self::parse_header(&mmap)?;
        
        Ok(Self {
            mmap,
            entry_count,
            entry_size,
        })
    }
    
    pub fn get_entry(&self, index: usize) -> Option<ScanResult> {
        if index >= self.entry_count {
            return None;
        }
        
        let offset = HEADER_SIZE + (index * self.entry_size);
        Some(Self::deserialize_entry(&self.mmap[offset..]))
    }
    
    pub fn iter(&self) -> MmapResultIterator {
        MmapResultIterator {
            reader: self,
            current: 0,
        }
    }
}

pub struct MmapResultIterator<'a> {
    reader: &'a MmapResultReader,
    current: usize,
}

impl<'a> Iterator for MmapResultIterator<'a> {
    type Item = ScanResult;
    
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.get_entry(self.current)?;
        self.current += 1;
        Some(result)
    }
}
```
- **Estimated Time:** 2h

**Task 1.3: Add mmap compression (optional)**
- Use zstd for 40-60% size reduction
- Compress in background thread (doesn't block scanning)
- Trade-off: CPU vs disk space
```rust
pub struct CompressedMmapWriter {
    base_writer: MmapResultWriter,
    compressor: ZstdEncoder,
    compression_level: i32,  // 1-22, default 3
}
```
- **Estimated Time:** 2h

**Task 1.4: Benchmark mmap vs standard output**
```bash
# Memory usage comparison
/usr/bin/time -v prtip -sS -p 80 10.0.0.0/16 -oN standard.txt
/usr/bin/time -v prtip -sS -p 80 10.0.0.0/16 --use-mmap -oM mmap.bin

# Expected:
# Standard: ~500MB peak memory (65K IPs × ~8KB/entry)
# Mmap: ~100MB peak memory (20-50% reduction)
```
- **Estimated Time:** 1h

**Task 1.5: Write unit tests**
- Test mmap write/read round-trip (1K entries)
- Test mmap iterator (verify order)
- Test buffer flush (ensure no data loss)
- Test compression (verify decompression)
- **Target:** 10-12 tests
- **Estimated Time:** 1.5h

---

### Task Area 2: Scan History & Resume (4-5 hours)

**Task 2.1: Design scan history storage**
- File: `prtip-core/src/history/scan_history.rs`
- Storage: SQLite database at `~/.prtip/history.db`
- Schema:
```sql
CREATE TABLE scans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    status TEXT NOT NULL,  -- running, paused, completed, failed
    target_spec TEXT NOT NULL,
    scan_config TEXT NOT NULL,  -- JSON
    results_path TEXT,
    total_targets INTEGER,
    targets_scanned INTEGER,
    ports_discovered INTEGER
);

CREATE TABLE scan_checkpoints (
    scan_id INTEGER,
    checkpoint_time TEXT NOT NULL,
    targets_scanned INTEGER,
    state_data BLOB,  -- Serialized scanner state
    FOREIGN KEY (scan_id) REFERENCES scans(id)
);
```
- **Estimated Time:** 1.5h

**Task 2.2: Implement pause/resume functionality**
```rust
pub struct ScanCheckpoint {
    pub scan_id: u64,
    pub targets_scanned: usize,
    pub scanner_state: Vec<u8>,  // Serialized RawSocketScanner state
}

impl Scanner {
    pub async fn pause(&mut self) -> io::Result<ScanCheckpoint> {
        // Stop sending new packets
        self.paused = true;
        
        // Wait for in-flight packets to complete
        self.drain_response_queue().await?;
        
        // Serialize current state
        let state = bincode::serialize(&self)?;
        
        let checkpoint = ScanCheckpoint {
            scan_id: self.scan_id,
            targets_scanned: self.progress.targets_scanned,
            scanner_state: state,
        };
        
        // Save checkpoint to database
        self.history.save_checkpoint(&checkpoint).await?;
        
        // Emit event
        self.event_bus.publish(Event::ScanPaused {
            scan_id: self.scan_id,
            checkpoint_time: Utc::now(),
        });
        
        Ok(checkpoint)
    }
    
    pub async fn resume(checkpoint: ScanCheckpoint) -> io::Result<Self> {
        // Deserialize scanner state
        let mut scanner: Self = bincode::deserialize(&checkpoint.scanner_state)?;
        
        // Resume scanning
        scanner.paused = false;
        
        // Emit event
        scanner.event_bus.publish(Event::ScanResumed {
            scan_id: checkpoint.scan_id,
            resume_time: Utc::now(),
        });
        
        Ok(scanner)
    }
}
```
- **Estimated Time:** 2.5h

**Task 2.3: TUI pause/resume controls**
- Keyboard: `p` pauses scan, `r` resumes scan
- Display pause status in progress widget: "PAUSED - Press 'r' to resume"
- Show checkpoint time and targets scanned at pause
- **Estimated Time:** 1h

**Task 2.4: CLI scan history commands**
```bash
# List scan history
prtip history list

# Resume most recent scan
prtip history resume

# Resume specific scan
prtip history resume --scan-id 42

# Delete old scans
prtip history clean --older-than 30d
```
- **Estimated Time:** 1h

**Task 2.5: Write unit tests**
- Test checkpoint save/load round-trip
- Test pause → resume (state preserved)
- Test scan history queries (list, get by ID)
- **Target:** 8-10 tests
- **Estimated Time:** 1h

---

### Task Area 3: Export Enhancements (3-4 hours)

**Task 3.1: Add CSV export**
- File: `prtip-core/src/output/csv_export.rs`
- Format: IP, Port, Protocol, State, Service, Version, Banner
```rust
pub struct CsvExporter {
    writer: csv::Writer<File>,
}

impl CsvExporter {
    pub fn export(&mut self, results: &[ScanResult]) -> io::Result<()> {
        for result in results {
            self.writer.write_record(&[
                result.target.to_string(),
                result.port.to_string(),
                result.protocol.to_string(),
                result.state.to_string(),
                result.service.clone().unwrap_or_default(),
                result.version.clone().unwrap_or_default(),
                result.banner.clone().unwrap_or_default(),
            ])?;
        }
        self.writer.flush()?;
        Ok(())
    }
}
```
- **Estimated Time:** 1h

**Task 3.2: Add HTML report generation**
```rust
pub struct HtmlReporter {
    template: Template,
}

impl HtmlReporter {
    pub fn generate(&self, scan: &ScanHistory, results: &[ScanResult]) -> String {
        let context = json!({
            "scan_id": scan.id,
            "start_time": scan.start_time,
            "end_time": scan.end_time,
            "total_targets": scan.total_targets,
            "ports_discovered": scan.ports_discovered,
            "results": results,
        });
        
        self.template.render("scan_report", &context).unwrap()
    }
}
```
- Template: `templates/scan_report.html` (Bootstrap styling)
- Sections: Summary, Statistics, Discovered Hosts, Service Breakdown
- **Estimated Time:** 2h

**Task 3.3: Add PDF export (optional)**
- Use printpdf crate
- Simple table format (IP, Port, Service)
- Logo and header/footer
- **Estimated Time:** 1.5h (skip if time-constrained)

**Task 3.4: TUI export menu**
- Keyboard: `e` opens export menu
- Select format: Plain Text, JSON, XML, CSV, HTML, PDF
- Enter filename
- Progress bar for large exports
- **Estimated Time:** 1h

**Task 3.5: Write unit tests**
- Test CSV export (verify format)
- Test HTML report generation (verify sections)
- Test export with 10K results (performance)
- **Target:** 6-8 tests
- **Estimated Time:** 0.5h

---

### Task Area 4: Real-Time Filtering (2-3 hours)

**Task 4.1: Add live result filter**
- File: `prtip-tui/src/filters/live_filter.rs`
```rust
pub struct LiveFilter {
    pub port_range: Option<(u16, u16)>,
    pub protocols: Option<Vec<Protocol>>,
    pub states: Option<Vec<PortState>>,
    pub service_regex: Option<Regex>,
    pub min_confidence: Option<f64>,  // For service detection
}

impl LiveFilter {
    pub fn matches(&self, result: &ScanResult) -> bool {
        if let Some((min, max)) = self.port_range {
            if result.port < min || result.port > max {
                return false;
            }
        }
        
        if let Some(ref protocols) = self.protocols {
            if !protocols.contains(&result.protocol) {
                return false;
            }
        }
        
        if let Some(ref states) = self.states {
            if !states.contains(&result.state) {
                return false;
            }
        }
        
        if let Some(ref regex) = self.service_regex {
            if !result.service.as_ref().map_or(false, |s| regex.is_match(s)) {
                return false;
            }
        }
        
        true
    }
}
```
- **Estimated Time:** 1.5h

**Task 4.2: Apply filter to port table widget**
```rust
impl PortTableWidget {
    pub fn set_filter(&mut self, filter: LiveFilter) {
        self.filter = Some(filter);
        // Re-filter existing results
        self.filtered_results = self.all_results.iter()
            .filter(|r| self.filter.as_ref().unwrap().matches(r))
            .cloned()
            .collect();
    }
}
```
- Filter applied to new results as they arrive (EventBus subscription)
- Display filter status: "Showing 42/1000 results (filtered)"
- **Estimated Time:** 1h

**Task 4.3: TUI filter builder UI**
- Keyboard: `f` opens filter builder
- Dialog with checkboxes/inputs for each filter criterion
- Preview: "This filter will match ~500 results"
- **Estimated Time:** 1h

**Task 4.4: Write unit tests**
- Test filter matching (port range, protocols, states)
- Test service regex filter
- Test combined filters (AND logic)
- **Target:** 5-6 tests
- **Estimated Time:** 0.5h

---

### Task Area 5: Performance Dashboard (2-3 hours)

**Task 5.1: Create PerformanceWidget**
- File: `prtip-tui/src/widgets/performance.rs`
- Display metrics:
  - CPU usage (per-core)
  - Memory usage (RSS, heap)
  - Network I/O (bytes sent/received)
  - Disk I/O (mmap flushes)
```rust
pub struct PerformanceWidget {
    cpu_usage: VecDeque<f64>,     // Last 60s
    memory_usage: VecDeque<usize>, // Last 60s
    network_io: VecDeque<(u64, u64)>, // (sent, recv) last 60s
}

impl PerformanceWidget {
    pub fn update(&mut self, metrics: &SystemMetrics) {
        self.cpu_usage.push_back(metrics.cpu_percent);
        self.memory_usage.push_back(metrics.memory_rss);
        self.network_io.push_back((metrics.bytes_sent, metrics.bytes_received));
        
        // Keep last 60 samples (1 per second)
        if self.cpu_usage.len() > 60 {
            self.cpu_usage.pop_front();
            self.memory_usage.pop_front();
            self.network_io.pop_front();
        }
    }
}
```
- **Estimated Time:** 1.5h

**Task 5.2: Collect system metrics**
```rust
use sysinfo::{System, SystemExt, ProcessExt};

pub struct MetricsCollector {
    system: System,
    pid: u32,
}

impl MetricsCollector {
    pub fn collect(&mut self) -> SystemMetrics {
        self.system.refresh_all();
        
        let process = self.system.process(self.pid.into()).unwrap();
        
        SystemMetrics {
            cpu_percent: process.cpu_usage() as f64,
            memory_rss: process.memory(),
            bytes_sent: /* network stats */,
            bytes_received: /* network stats */,
        }
    }
}
```
- Poll every 1 second
- Emit PerformanceMetricsEvent to EventBus
- **Estimated Time:** 1h

**Task 5.3: Render performance graphs**
- Use `ratatui::widgets::Chart` for line graphs
- CPU: 0-100% range
- Memory: 0-max_rss range (MB)
- Network: bytes/sec (log scale)
- **Estimated Time:** 1h

**Task 5.4: Write unit tests**
- Test metrics collection
- Test ring buffer management
- **Target:** 4-5 tests
- **Estimated Time:** 0.5h

---

## Definition of Done

### Functional Requirements
- [ ] Memory-mapped I/O working (20-50% memory reduction verified)
- [ ] Scan pause/resume functional (state preserved)
- [ ] Scan history stored in SQLite database
- [ ] CSV, HTML exports working
- [ ] Live result filtering applied in real-time
- [ ] Performance dashboard displays CPU/memory/network metrics

### Quality Requirements
- [ ] 43-51 tests passing (100% success rate)
- [ ] Zero clippy warnings (--deny warnings)
- [ ] Zero rustdoc warnings
- [ ] Code formatted with `cargo fmt`
- [ ] Memory leak testing (long-running scan, stable memory)

### Documentation Requirements
- [ ] 30-ADVANCED-FEATURES-GUIDE.md complete (1,500-2,000 lines)
- [ ] Rustdoc comments for all public APIs
- [ ] Export format examples
- [ ] Pause/resume workflow documented

### Performance Requirements
- [ ] Memory reduction: 20-50% (mmap vs standard)
- [ ] Export time (10K results): <5s for CSV, <10s for HTML
- [ ] Checkpoint save time: <500ms
- [ ] Resume time: <1s
- [ ] Filter latency: <50ms to apply

---

## Testing Plan

### Unit Tests (30-35 tests)
```bash
cargo test -p prtip-core output::mmap_
cargo test -p prtip-core history::
cargo test -p prtip-core output::csv_
cargo test -p prtip-tui filters::live_filter
cargo test -p prtip-tui widgets::performance
```

**Test Cases:**
1-12. Mmap: write/read, iterator, compression, benchmarks
13-17. History: checkpoint save/load, pause/resume, queries
18-23. Export: CSV format, HTML sections, performance
24-28. LiveFilter: port range, protocols, states, service regex, combined
29-33. Performance: metrics collection, ring buffer, graph rendering

### Integration Tests (13-16 tests)
```bash
cargo test -p prtip --test integration_advanced
```

**Test Cases:**
1. Full Scan: mmap vs standard output (verify memory reduction)
2. Pause/Resume: pause at 50%, resume, verify completion
3. Scan History: list scans, get by ID, clean old scans
4. Export CSV: 10K results, verify format
5. Export HTML: verify all sections present
6. Live Filter: apply during scan, verify filtered results
7. Performance Dashboard: verify metrics update every 1s
8. EventBus: MmapFlushed, ScanPaused, ScanResumed events
9. Memory Leak: 10-minute scan, verify stable memory
10. Checkpoint Recovery: crash simulation, resume from checkpoint

### Benchmark Tests
```bash
# Memory usage comparison
hyperfine --warmup 3 \
  'prtip -sS -p 80 10.0.0.0/16 -oN standard.txt' \
  'prtip -sS -p 80 10.0.0.0/16 --use-mmap -oM mmap.bin' \
  --export-json results/mmap_vs_standard.json

# Export performance
hyperfine --warmup 3 \
  'prtip export --format csv results.bin' \
  'prtip export --format html results.bin' \
  --export-json results/export_perf.json
```

### Manual Testing Checklist
- [ ] **Mmap:** Verify 20-50% memory reduction (with /usr/bin/time -v)
- [ ] **Pause:** Press `p` during scan, verify pause status displayed
- [ ] **Resume:** Press `r` after pause, verify scan continues
- [ ] **History:** Run `prtip history list`, verify scan listed
- [ ] **Export CSV:** Open in Excel/LibreOffice, verify columns
- [ ] **Export HTML:** Open in browser, verify styling
- [ ] **Filter:** Apply "TCP only" filter, verify UDP results hidden
- [ ] **Performance:** Verify CPU/memory graphs update in TUI
- [ ] **Long Scan:** 30-minute scan, verify stable memory (no leaks)

---

## Dependencies

### External Crates
- `memmap2 = "0.9"` - Memory-mapped file I/O
- `zstd = "0.13"` - Compression (optional)
- `rusqlite = "0.30"` - Scan history database
- `csv = "1.3"` - CSV export
- `tera = "1.19"` - HTML template engine
- `printpdf = "0.6"` - PDF export (optional)
- `sysinfo = "0.30"` - System metrics collection

### Internal Dependencies
- **Sprint 6.4 (Adaptive Tuning):** Mmap infrastructure
- **Sprint 6.5 (Interactive Selection):** Export foundation
- **prtip-scanner:** Scanner state serialization

---

## Risk Mitigation

### Risk 1: Mmap File Corruption
**Impact:** High | **Probability:** Low
**Mitigation:**
- Flush every 1K entries (limit data loss)
- Add file integrity check (checksum in header)
- Test with simulated crashes (kill -9)

### Risk 2: Checkpoint State Size
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Compress checkpoint state (zstd)
- Store only essential state (exclude cached data)
- Limit checkpoint frequency (every 10K targets, not every packet)

### Risk 3: Export Performance (Large Results)
**Impact:** Medium | **Probability:** Medium
**Mitigation:**
- Stream results to file (don't load all in memory)
- Progress bar for long exports (>10s)
- Abort on user request (Ctrl+C during export)

---

## Resources

### Documentation
- **memmap2:** https://docs.rs/memmap2/
- **rusqlite:** https://docs.rs/rusqlite/
- **tera templates:** https://tera.netlify.app/

### Reference Implementations
- **Nmap Resume:** --resume option (uses XML state file)
- **Masscan Pause:** pacer.c (no resume, just rate limiting)

---

## Sprint Completion Report Template

```markdown
# Sprint 6.6 Completion Report

**Date:** [YYYY-MM-DD]
**Actual Duration:** [X] hours
**Status:** ✅ COMPLETE / ⚠️ PARTIAL / ❌ INCOMPLETE

## Deliverables Status
- [ ] Memory-Mapped I/O Completion (QW-3)
- [ ] Scan History & Resume
- [ ] Export Enhancements (CSV, HTML, PDF)
- [ ] Real-Time Filtering
- [ ] Performance Dashboard

## Test Results
- Unit Tests: [X/35] passing
- Integration Tests: [X/16] passing

## Performance Metrics
- Memory Reduction: [X]% (target: 20-50%)
- Export Time (10K results): CSV [X]s, HTML [X]s
- Checkpoint Save Time: [X]ms (target: <500ms)

## Issues Encountered
1. [Issue description] - **Resolution:** [How resolved]

## Lessons Learned
- [Key insight from mmap implementation]

## Next Sprint Preparation
- Dependencies ready for Sprint 6.7: ✅/❌
```

---

**This sprint adds enterprise-grade features (audit trail, resume, export). Prioritize data integrity (no corruption) over performance - operators trust ProRT-IP with sensitive scan data.**
