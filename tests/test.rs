use babycancer::*;

#[test]
fn test_help() {
    if let Err(e) = repl::execute_line("config -c tests/example/config.toml".to_string()) {
        panic!("Failed to set config path: {}", e);
    }

    if let Err(e) = repl::execute_line("backup".to_string()) {
        panic!("Failed to execute backup command: {}", e);
    }
}
