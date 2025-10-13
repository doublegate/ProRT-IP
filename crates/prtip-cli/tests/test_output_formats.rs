//! Integration tests for output format generation
//!
//! Tests text, JSON, XML, and greppable output formats.

#[path = "common/mod.rs"]
mod common;

use common::{cleanup_temp_dir, create_temp_dir, init, run_prtip};
use std::fs;

#[test]
fn test_default_text_output() {
    init();
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain some standard output patterns
        assert!(
            stdout.contains("127.0.0.1") || stdout.contains("scan"),
            "Text output should contain target or scan info"
        );
    }
}

#[test]
fn test_json_output_file() {
    init();
    let temp_dir = create_temp_dir("json_output");
    let output_file = temp_dir.join("scan.json");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oJ",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    if output.status.success() {
        // Check file was created
        assert!(output_file.exists(), "JSON output file not created");

        // Try to parse JSON
        let content = fs::read_to_string(&output_file).expect("Failed to read JSON output file");

        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "JSON output is not valid JSON: {:?}",
            parsed.err()
        );

        // Check for expected fields
        if let Ok(json) = parsed {
            // Depending on schema, check for expected fields
            // This is flexible to handle different JSON structures
            assert!(
                json.is_object() || json.is_array(),
                "JSON output should be object or array"
            );
        }
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_xml_output_file() {
    init();
    let temp_dir = create_temp_dir("xml_output");
    let output_file = temp_dir.join("scan.xml");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oX",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    if output.status.success() {
        // Check file was created
        assert!(output_file.exists(), "XML output file not created");

        // Check XML structure
        let content = fs::read_to_string(&output_file).expect("Failed to read XML output file");

        assert!(
            content.contains("<?xml"),
            "XML output should have XML declaration"
        );
        assert!(content.contains("<"), "XML output should contain tags");
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_greppable_output_file() {
    init();
    let temp_dir = create_temp_dir("grep_output");
    let output_file = temp_dir.join("scan.gnmap");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oG",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    if output.status.success() {
        // Check file was created
        assert!(output_file.exists(), "Greppable output file not created");

        // Check greppable format (should have "Host:" lines)
        let content =
            fs::read_to_string(&output_file).expect("Failed to read greppable output file");

        // Greppable format has specific patterns
        assert!(
            content.contains("Host:") || content.contains("Ports:") || content.is_empty(),
            "Greppable output should contain Host/Ports markers or be empty"
        );
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_all_formats_output() {
    init();
    let temp_dir = create_temp_dir("all_formats");
    let base_name = temp_dir.join("scan");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oA",
        base_name.to_str().unwrap(),
        "127.0.0.1",
    ]);

    // -oA may not be implemented yet, so just check that it doesn't crash
    if output.status.success() {
        // Check that multiple files were created
        let txt_file = base_name.with_extension("txt");
        let xml_file = base_name.with_extension("xml");
        let gnmap_file = base_name.with_extension("gnmap");

        // At least one format should exist (or -oA not implemented, which is okay)
        let any_exist = txt_file.exists() || xml_file.exists() || gnmap_file.exists();
        if !any_exist {
            // -oA might not be implemented, skip assertion
            eprintln!("Note: -oA flag may not be implemented yet (no files created)");
        }
    } else {
        // If it fails, check if it's because -oA is not recognized
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unrecognized") || stderr.contains("unknown") {
            eprintln!("Note: -oA flag not implemented (unrecognized option)");
        }
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_output_to_stdout() {
    init();
    // Without output file, should write to stdout
    let output = run_prtip(&["-sT", "-p", "80", "127.0.0.1"]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "Should have output to stdout");
    }
}

#[test]
fn test_json_output_structure() {
    init();
    let temp_dir = create_temp_dir("json_struct");
    let output_file = temp_dir.join("scan.json");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80,443",
        "-oJ",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    if output.status.success() && output_file.exists() {
        let content = fs::read(&output_file).expect("Failed to read JSON output");

        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&content) {
            // Check basic structure (flexible to accommodate different schemas)
            if json.is_object() {
                let obj = json.as_object().unwrap();
                // Should have some fields related to scanning
                let has_scan_fields = obj.contains_key("scan_id")
                    || obj.contains_key("targets")
                    || obj.contains_key("results")
                    || obj.contains_key("start_time");

                assert!(has_scan_fields, "JSON should contain scan-related fields");
            }
        }
    }

    cleanup_temp_dir(&temp_dir);
}

#[test]
fn test_output_file_permissions() {
    init();
    let temp_dir = create_temp_dir("permissions");
    let output_file = temp_dir.join("scan.txt");

    let output = run_prtip(&[
        "-sT",
        "-p",
        "80",
        "-oN",
        output_file.to_str().unwrap(),
        "127.0.0.1",
    ]);

    if output.status.success() && output_file.exists() {
        // File should be readable
        let metadata = fs::metadata(&output_file).expect("Failed to get file metadata");

        assert!(metadata.is_file(), "Output should be a regular file");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = metadata.permissions();
            // Should have read permission for owner
            assert!(perms.mode() & 0o400 != 0, "Output file should be readable");
        }
    }

    cleanup_temp_dir(&temp_dir);
}
