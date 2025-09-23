use babycancer::*;

#[test]
fn test_placeholder() {
    // Placeholder test to ensure the tests module is recognized.
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_args() {
    let line = String::from("config -");
    let args = app::get_args(line);
	assert!(args.is_ok());
    assert!(app::execute_command(args.unwrap()).is_ok());
}