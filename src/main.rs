use babycancer::*;

fn main() {
    loop {
        print!("babycancer> ");

        let line = match app::get_line() {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        let args = match app::get_args(line) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        match app::execute_command(args) {
            Ok(_) => continue,
            Err(e) => {
                eprintln!("Error executing command: {}", e);
                continue;
            }
        };
    }
}
