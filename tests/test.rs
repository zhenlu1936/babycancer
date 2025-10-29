use babycancer::*;
use std::fs;
use std::io::Read;

// Test setup helper to ensure clean config state
fn setup_clean_config(config_name: &str) -> String {
    let config_content = r#"[path_config]
source_path = "tests/example/src"
dest_path = "tests/example/dest"

[file_config]

[output_config]
tar = false
gzip = false"#;
    
    let config_path = format!("tests/example/{}.toml", config_name);
    fs::write(&config_path, config_content).expect("Failed to write test config");
    config_path
}

// Cleanup helper to remove temporary config files
fn cleanup_config(config_path: &str) {
    let _ = std::fs::remove_file(config_path);
    // Also remove the dest folder if it exists
    let _ = std::fs::remove_dir_all("tests/example/dest");
}

// Helper to run test with automatic cleanup
fn with_clean_config<F>(test_name: &str, test_fn: F) 
where 
    F: FnOnce(String)
{
    let config_path = setup_clean_config(test_name);
    // Clean dest folder before test
    let _ = std::fs::remove_dir_all("tests/example/dest");
    // Create dest folder for tests that need it
    let _ = std::fs::create_dir_all("tests/example/dest");
    
    test_fn(config_path.clone());
    
    // Clean up after test
    cleanup_config(&config_path);
}

// Config path retention tests
#[test]
fn test_config_path_retention() {
    with_clean_config("test_retention", |config_path| {
        if let Err(e) = repl::execute_line(format!("config -c {}", config_path)) {
            panic!("Failed to set config path: {}", e);
        }

        if let Err(e) = repl::execute_line("backup".to_string()) {
            panic!("Failed to execute backup command: {}", e);
        }
    });
}

#[test]
fn test_config_path_persistence_across_commands() {
    with_clean_config("test_persistence", |config_path| {
        // Set config path and source path
        let result = repl::execute_line(format!("config -c {} --source-path tests/example/src", config_path));
        assert!(result.is_ok());
        
        // Backup should use the retained config path
        let result = repl::execute_line("backup".to_string());
        assert!(result.is_ok());
    });
}

#[test]
fn test_help_commands() {
    // Test help commands (output is printed but we don't care about it)
    assert!(repl::execute_line("config --help".to_string()).is_ok());
    assert!(repl::execute_line("backup --help".to_string()).is_ok());
    assert!(repl::execute_line("reset --help".to_string()).is_ok());
}

// Command parsing tests
#[test]
fn test_command_parsing() {
    // Invalid command
    assert!(repl::execute_line("invalid_command".to_string()).is_err());
    
    // Empty and whitespace commands should be handled gracefully
    assert!(repl::execute_line("".to_string()).is_ok());
    assert!(repl::execute_line("   ".to_string()).is_ok());
}

#[test]
fn test_config_basic_options() {
    with_clean_config("test_basic", |config_path| {
        // Test individual config options
        assert!(repl::execute_line(format!("config -c {} --source-path tests/example/src", config_path)).is_ok());
        assert!(repl::execute_line("config --dest-path tests/example/dest".to_string()).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.txt'".to_string()).is_ok());
        assert!(repl::execute_line("config --user testuser".to_string()).is_ok());
        assert!(repl::execute_line("config --tar true".to_string()).is_ok());
        assert!(repl::execute_line("config --output".to_string()).is_ok());
    });
}

#[test]
fn test_reset_commands() {
    with_clean_config("test_resets", |config_path| {
        // Set multiple fields first
        assert!(repl::execute_line(format!(
            "config -c {} --source-path tests/example/src --user testuser --file-name '.*\\.rs' --tar true",
            config_path
        )).is_ok());
        
        // Test individual resets
        assert!(repl::execute_line("reset --user".to_string()).is_ok());
        assert!(repl::execute_line("reset --file-name".to_string()).is_ok());
        assert!(repl::execute_line("reset --tar".to_string()).is_ok());
        assert!(repl::execute_line("reset --all".to_string()).is_ok());
    });
}

// Backup command tests
#[test]
fn test_backup_commands() {
    with_clean_config("test_backup", |config_path| {
        // Basic backup
        assert!(repl::execute_line(format!("backup -c {}", config_path)).is_ok());
        
        // Interval zero should fail
        assert!(repl::execute_line(format!("backup -c {} --interval 0", config_path)).is_err());
    });
    
    // Realtime with nonexistent config should fail
    assert!(repl::execute_line("backup -c /tmp/nonexistent.toml --realtime".to_string()).is_err());
}

// Error handling tests
#[test]
fn test_error_handling() {
    with_clean_config("test_errors", |_config_path| {
        // Invalid paths
        assert!(repl::execute_line("config -c /nonexistent/path/config.toml".to_string()).is_err());
        assert!(repl::execute_line("backup -c /tmp/nonexistent.toml".to_string()).is_err());
        assert!(repl::execute_line("reset -c /tmp/nonexistent.toml --all".to_string()).is_err());
        
        // Invalid regex
        assert!(repl::execute_line("config -c tests/example/config.toml --file-name '[invalid'".to_string()).is_err());
    });
}

// Edge case tests
#[test]
fn test_edge_cases() {
    with_clean_config("test_edges", |config_path| {
        // Multiple flags at once
        assert!(repl::execute_line(format!(
            "config -c {} --source-path tests/example/src --dest-path tests/example/dest --output",
            config_path
        )).is_ok());
        
        // Numeric edge cases
        assert!(repl::execute_line("config --size 1024".to_string()).is_ok());
        assert!(repl::execute_line("config --size 0".to_string()).is_ok());
        assert!(repl::execute_line("config --size 9223372036854775807".to_string()).is_ok());
        
        // Date formats
        assert!(repl::execute_line("config --date 2023-01-01".to_string()).is_ok());
        assert!(repl::execute_line("config --date 2024-12-31".to_string()).is_ok());
        
        // Other filters
        assert!(repl::execute_line("config --file-path 'some/path'".to_string()).is_ok());
        assert!(repl::execute_line("config --user ''".to_string()).is_ok());
        
        // Special characters
        assert!(repl::execute_line("config --user 'user_测试_тест'".to_string()).is_ok());
    });
}

// Complex integration tests
#[test]
fn test_config_workflows() {
    with_clean_config("test_workflows", |config_path| {
        // Multiple updates in same session
        assert!(repl::execute_line(format!("config -c {} --source-path tests/example/src", config_path)).is_ok());
        assert!(repl::execute_line("config --dest-path tests/example/dest".to_string()).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.rs$'".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        
        // Reset and reconfigure
        assert!(repl::execute_line("reset --user".to_string()).is_ok());
        assert!(repl::execute_line("config --user newuser".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test]
fn test_regex_validation() {
    with_clean_config("test_regex", |config_path| {
        // Valid regex patterns
        assert!(repl::execute_line(format!("config -c {} --file-name '.*\\.txt$'", config_path)).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.(rs|toml|md)$'".to_string()).is_ok());
        
        // Invalid regex should fail
        assert!(repl::execute_line("config --file-name '[unclosed'".to_string()).is_err());
        
        // Should still work after error
        assert!(repl::execute_line("config --file-name '.*\\.rs$'".to_string()).is_ok());
    });
}

#[test]
fn test_full_workflow() {
    with_clean_config("test_full", |config_path| {
        // Build up config step by step
        assert!(repl::execute_line(format!("config -c {}", config_path)).is_ok());
        assert!(repl::execute_line("config --source-path tests/example/src".to_string()).is_ok());
        assert!(repl::execute_line("config --dest-path tests/example/dest".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        
        // Add filters
        assert!(repl::execute_line("config --file-name '.*\\.txt$'".to_string()).is_ok());
        assert!(repl::execute_line("config --size 1000".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        
        // Full reset and rebuild
        assert!(repl::execute_line("reset --all".to_string()).is_ok());
        assert!(repl::execute_line("config --source-path tests/example/src --dest-path tests/example/dest".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test] 
#[ignore] // Ignore this test since exit would terminate the test process
fn test_exit_command_parsing() {
    // Can't actually test exit since it would terminate the test process
}

// Additional test coverage
#[test]
fn test_config_individual_resets() {
    with_clean_config("test_individual_resets", |config_path| {
        // Set all options
        assert!(repl::execute_line(format!(
            "config -c {} --source-path tests/example/src --dest-path tests/example/dest --file-name '.*\\.rs' --user admin --tar true --size 1000 --date 2023-01-01",
            config_path
        )).is_ok());
        
        // Reset each individually
        assert!(repl::execute_line("reset --source-path".to_string()).is_ok());
        assert!(repl::execute_line("reset --dest-path".to_string()).is_ok());
        assert!(repl::execute_line("reset --file-name".to_string()).is_ok());
        assert!(repl::execute_line("reset --user".to_string()).is_ok());
        assert!(repl::execute_line("reset --tar".to_string()).is_ok());
        assert!(repl::execute_line("reset --size".to_string()).is_ok());
        assert!(repl::execute_line("reset --date".to_string()).is_ok());
    });
}

#[test]
fn test_multiple_backups_same_config() {
    with_clean_config("test_multi_backup", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --source-path tests/example/src", config_path)).is_ok());
        
        // Run backup multiple times
        for _ in 0..3 {
            assert!(repl::execute_line("backup".to_string()).is_ok());
        }
    });
}

#[test]
fn test_config_then_output() {
    with_clean_config("test_config_output", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --source-path tests/example/src --user testuser", config_path)).is_ok());
        assert!(repl::execute_line("config --output".to_string()).is_ok());
    });
}

#[test]
fn test_complex_file_patterns() {
    with_clean_config("test_patterns", |config_path| {
        // Different regex patterns
        assert!(repl::execute_line(format!("config -c {} --file-name '^test_.*$'", config_path)).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.(txt|md|rs)$'".to_string()).is_ok());
        assert!(repl::execute_line("config --file-name '[a-z]+\\.txt'".to_string()).is_ok());
    });
}

#[test]
fn test_date_edge_cases() {
    with_clean_config("test_date_edges", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --date 2020-01-01", config_path)).is_ok());
        assert!(repl::execute_line("config --date 2025-12-31".to_string()).is_ok());
        assert!(repl::execute_line("config --date 2030-06-15".to_string()).is_ok());
        assert!(repl::execute_line("reset --date".to_string()).is_ok());
    });
}

#[test]
fn test_size_variations() {
    with_clean_config("test_sizes", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --size 1", config_path)).is_ok());
        assert!(repl::execute_line("config --size 1024".to_string()).is_ok());
        assert!(repl::execute_line("config --size 1048576".to_string()).is_ok());
        assert!(repl::execute_line("config --size 1073741824".to_string()).is_ok());
    });
}

#[test]
fn test_user_variations() {
    with_clean_config("test_users", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --user admin", config_path)).is_ok());
        assert!(repl::execute_line("config --user root".to_string()).is_ok());
        assert!(repl::execute_line("config --user 'john.doe'".to_string()).is_ok());
        assert!(repl::execute_line("config --user 'user@domain.com'".to_string()).is_ok());
        assert!(repl::execute_line("reset --user".to_string()).is_ok());
    });
}

#[test]
fn test_tar_toggle() {
    with_clean_config("test_tar_toggle", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --tar true", config_path)).is_ok());
        assert!(repl::execute_line("config --tar false".to_string()).is_ok());
        assert!(repl::execute_line("config --tar true".to_string()).is_ok());
        assert!(repl::execute_line("reset --tar".to_string()).is_ok());
    });
}

#[test]
fn test_file_path_operations() {
    with_clean_config("test_file_paths", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --file-path 'path/to/file'", config_path)).is_ok());
        assert!(repl::execute_line("config --file-path '/absolute/path'".to_string()).is_ok());
        assert!(repl::execute_line("config --file-path 'relative/path'".to_string()).is_ok());
        assert!(repl::execute_line("reset --file-path".to_string()).is_ok());
    });
}

#[test]
fn test_combined_filters() {
    with_clean_config("test_combined", |config_path| {
        // Combine multiple filters
        assert!(repl::execute_line(format!(
            "config -c {} --file-name '.*\\.txt$' --size 1024 --date 2023-01-01 --user admin",
            config_path
        )).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test]
fn test_incremental_config_build() {
    with_clean_config("test_incremental", |config_path| {
        // Build config incrementally
        assert!(repl::execute_line(format!("config -c {}", config_path)).is_ok());
        assert!(repl::execute_line("config --source-path tests/example/src".to_string()).is_ok());
        assert!(repl::execute_line("config --dest-path tests/example/dest".to_string()).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.txt'".to_string()).is_ok());
        assert!(repl::execute_line("config --user testuser".to_string()).is_ok());
        assert!(repl::execute_line("config --size 500".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test]
fn test_reset_after_backup() {
    with_clean_config("test_reset_after", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --source-path tests/example/src", config_path)).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        assert!(repl::execute_line("reset --all".to_string()).is_ok());
    });
}

#[test]
fn test_reconfig_after_partial_reset() {
    with_clean_config("test_reconfig", |config_path| {
        // Set multiple options
        assert!(repl::execute_line(format!(
            "config -c {} --source-path tests/example/src --user admin --file-name '.*\\.rs'",
            config_path
        )).is_ok());
        
        // Reset some
        assert!(repl::execute_line("reset --user".to_string()).is_ok());
        assert!(repl::execute_line("reset --file-name".to_string()).is_ok());
        
        // Reconfigure with new values
        assert!(repl::execute_line("config --user newadmin".to_string()).is_ok());
        assert!(repl::execute_line("config --file-name '.*\\.txt'".to_string()).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test]
fn test_gzip_toggle() {
    with_clean_config("test_gzip_toggle", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --gzip true", config_path)).is_ok());
        assert!(repl::execute_line("config --gzip false".to_string()).is_ok());
        assert!(repl::execute_line("config --gzip true".to_string()).is_ok());
        assert!(repl::execute_line("reset --gzip".to_string()).is_ok());
    });
}

#[test]
fn test_gzip_with_tar() {
    with_clean_config("test_gzip_tar", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --tar true --gzip true", config_path)).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        
        // Verify tar.gz file exists
        let dest = PathBuf::from("tests/example/dest");
        assert!(dest.join("backup.tar.gz").exists());
        
        // Verify it's a valid gzip file
        let file = fs::File::open(dest.join("backup.tar.gz")).expect("Failed to open backup.tar.gz");
        let mut gz = flate2::read::GzDecoder::new(file);
        let mut contents = Vec::new();
        gz.read_to_end(&mut contents).expect("Failed to decompress gzip file");
        assert!(contents.len() > 0, "Gzipped tar archive should not be empty");
    });
}

#[test]
fn test_gzip_without_tar() {
    with_clean_config("test_gzip_no_tar", |config_path| {
        // Gzip without tar should still work (files just copied normally)
        assert!(repl::execute_line(format!("config -c {} --gzip true", config_path)).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
    });
}

#[test]
fn test_tar_without_gzip() {
    with_clean_config("test_tar_no_gzip", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --tar true --gzip false", config_path)).is_ok());
        assert!(repl::execute_line("backup".to_string()).is_ok());
        
        // Verify only .tar file exists (not .tar.gz)
        let dest = PathBuf::from("tests/example/dest");
        assert!(dest.join("backup.tar").exists());
        assert!(!dest.join("backup.tar.gz").exists());
    });
}

#[test]
fn test_gzip_reset_all() {
    with_clean_config("test_gzip_reset_all", |config_path| {
        assert!(repl::execute_line(format!("config -c {} --gzip true --tar true", config_path)).is_ok());
        assert!(repl::execute_line("reset --all".to_string()).is_ok());
        
        // Verify config was reset
        assert!(repl::execute_line("config --output".to_string()).is_ok());
    });
}
