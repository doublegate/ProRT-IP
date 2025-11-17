//! Integration tests for command history and replay functionality

use prtip_cli::history::{HistoryEntry, HistoryManager};
use tempfile::TempDir;

/// Helper to create a temporary history manager for testing
fn create_temp_manager() -> (HistoryManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.json");

    let manager = HistoryManager::with_path(history_path);

    (manager, temp_dir)
}

#[test]
fn test_full_workflow_add_list_replay() {
    let (mut manager, _temp) = create_temp_manager();

    // Add three scan commands
    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sS".to_string(),
                "-p".to_string(),
                "80,443".to_string(),
                "192.168.1.1".to_string(),
            ],
            "SYN scan of 192.168.1.1: 2 open ports",
            0,
        )
        .unwrap();

    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sV".to_string(),
                "target.com".to_string(),
            ],
            "Service detection scan: 5 services detected",
            0,
        )
        .unwrap();

    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sT".to_string(),
                "10.0.0.0/24".to_string(),
            ],
            "Connect scan of 10.0.0.0/24: 10 hosts, 15 open ports",
            0,
        )
        .unwrap();

    // Verify history size
    assert_eq!(manager.len(), 3);
    assert!(!manager.is_empty());

    // List all entries
    let entries = manager.list_entries();
    assert_eq!(entries.len(), 3);

    // Verify first entry
    assert_eq!(entries[0].command, "prtip -sS -p 80,443 192.168.1.1");
    assert_eq!(entries[0].summary, "SYN scan of 192.168.1.1: 2 open ports");
    assert_eq!(entries[0].exit_code, 0);

    // Get last entry
    let last = manager.get_last().unwrap();
    assert_eq!(last.command, "prtip -sT 10.0.0.0/24");

    // Replay second entry
    let entry_to_replay = manager.get_entry(1).unwrap();
    HistoryManager::validate_replay(entry_to_replay).unwrap();

    let replay_args = HistoryManager::rebuild_command(entry_to_replay, None);
    assert_eq!(replay_args.len(), 3);
    assert_eq!(replay_args[0], "prtip");
    assert_eq!(replay_args[1], "-sV");
    assert_eq!(replay_args[2], "target.com");

    // Replay with modifications
    let modified_args =
        HistoryManager::rebuild_command(entry_to_replay, Some(vec!["--verbose", "-p", "1-1000"]));
    assert_eq!(modified_args.len(), 6);
    assert_eq!(modified_args[3], "--verbose");
    assert_eq!(modified_args[4], "-p");
    assert_eq!(modified_args[5], "1-1000");
}

#[test]
fn test_persistence_across_manager_instances() {
    // Save current env var state and unset for this test
    // This test specifically validates file persistence, which requires actual file I/O
    let old_disable = std::env::var("PRTIP_DISABLE_HISTORY").ok();
    std::env::remove_var("PRTIP_DISABLE_HISTORY");

    let temp_dir = TempDir::new().unwrap();

    // Set up temporary home directory
    std::env::set_var("HOME", temp_dir.path());

    // Create first manager and add entries
    {
        let mut manager = HistoryManager::new(true).unwrap();

        manager
            .add_entry(
                vec![
                    "prtip".to_string(),
                    "-sS".to_string(),
                    "target1".to_string(),
                ],
                "Scan 1",
                0,
            )
            .unwrap();

        manager
            .add_entry(
                vec![
                    "prtip".to_string(),
                    "-sT".to_string(),
                    "target2".to_string(),
                ],
                "Scan 2",
                0,
            )
            .unwrap();

        assert_eq!(manager.len(), 2);
    } // Manager dropped here

    // Create second manager and verify entries persisted
    {
        let manager = HistoryManager::new(true).unwrap();

        assert_eq!(manager.len(), 2);
        let entries = manager.list_entries();
        assert_eq!(entries[0].summary, "Scan 1");
        assert_eq!(entries[1].summary, "Scan 2");
    }

    // Cleanup
    std::env::remove_var("HOME");

    // Restore env var state
    match old_disable {
        Some(val) => std::env::set_var("PRTIP_DISABLE_HISTORY", val),
        None => std::env::remove_var("PRTIP_DISABLE_HISTORY"),
    }
}

#[test]
fn test_error_handling_invalid_replay() {
    let (manager, _temp) = create_temp_manager();

    // Try to get non-existent entry
    assert!(manager.get_entry(0).is_none());
    assert!(manager.get_entry(100).is_none());

    // Create invalid entry (no args)
    let invalid_entry = HistoryEntry::new(Vec::new(), "Test", 0);
    assert!(HistoryManager::validate_replay(&invalid_entry).is_err());

    // Create entry with wrong command
    let wrong_command = HistoryEntry::new(
        vec!["nmap".to_string(), "-sS".to_string(), "target".to_string()],
        "Wrong command",
        0,
    );
    assert!(HistoryManager::validate_replay(&wrong_command).is_err());
}

#[test]
fn test_clear_and_rebuild() {
    let (mut manager, _temp) = create_temp_manager();

    // Add entries
    for i in 0..10 {
        manager
            .add_entry(
                vec![
                    "prtip".to_string(),
                    "-sS".to_string(),
                    format!("target{}", i),
                ],
                format!("Scan {}", i),
                0,
            )
            .unwrap();
    }

    assert_eq!(manager.len(), 10);

    // Clear history
    manager.clear().unwrap();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());

    // Rebuild history
    for i in 0..5 {
        manager
            .add_entry(
                vec![
                    "prtip".to_string(),
                    "-sT".to_string(),
                    format!("newtarget{}", i),
                ],
                format!("New scan {}", i),
                0,
            )
            .unwrap();
    }

    assert_eq!(manager.len(), 5);
    let first = manager.get_entry(0).unwrap();
    assert_eq!(first.summary, "New scan 0");
}

#[test]
fn test_exit_code_tracking() {
    let (mut manager, _temp) = create_temp_manager();

    // Add successful scan
    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sS".to_string(),
                "target1".to_string(),
            ],
            "Successful scan",
            0,
        )
        .unwrap();

    // Add failed scan
    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sS".to_string(),
                "invalid-target".to_string(),
            ],
            "Failed to resolve hostname",
            1,
        )
        .unwrap();

    let entries = manager.list_entries();
    assert_eq!(entries[0].exit_code, 0);
    assert_eq!(entries[1].exit_code, 1);

    // Verify display formatting includes exit code
    let success_display = entries[0].format_display(0);
    assert!(success_display.contains("✓"));

    let error_display = entries[1].format_display(1);
    assert!(error_display.contains("✗(1)"));
}

#[test]
fn test_timestamp_tracking() {
    use std::thread;
    use std::time::Duration;

    let (mut manager, _temp) = create_temp_manager();

    // Add first entry
    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sS".to_string(),
                "target1".to_string(),
            ],
            "Scan 1",
            0,
        )
        .unwrap();

    let first_timestamp = manager.get_entry(0).unwrap().timestamp;

    // Wait a bit
    thread::sleep(Duration::from_millis(10));

    // Add second entry
    manager
        .add_entry(
            vec![
                "prtip".to_string(),
                "-sT".to_string(),
                "target2".to_string(),
            ],
            "Scan 2",
            0,
        )
        .unwrap();

    let second_timestamp = manager.get_entry(1).unwrap().timestamp;

    // Second timestamp should be after first
    assert!(second_timestamp > first_timestamp);
}

#[test]
fn test_json_serialization_format() {
    let temp_dir = TempDir::new().unwrap();
    let history_path = temp_dir.path().join("history.json");

    {
        let mut manager = HistoryManager::with_path(history_path.clone());

        manager
            .add_entry(
                vec!["prtip".to_string(), "-sS".to_string(), "target".to_string()],
                "Test scan",
                0,
            )
            .unwrap();
    }

    // Read raw JSON
    let json_content = std::fs::read_to_string(&history_path).unwrap();

    // Verify it's valid JSON
    let parsed: Vec<HistoryEntry> = serde_json::from_str(&json_content).unwrap();
    assert_eq!(parsed.len(), 1);

    // Verify JSON contains expected fields
    assert!(json_content.contains("\"timestamp\""));
    assert!(json_content.contains("\"command\""));
    assert!(json_content.contains("\"args\""));
    assert!(json_content.contains("\"summary\""));
    assert!(json_content.contains("\"exit_code\""));
}
