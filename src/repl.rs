use crate::*;

pub fn run() {
    loop {
        print!("babycancer> ");

        let line = match get_line() {
            Ok(l) => l,
            Err(e) => {
                // For clap DisplayHelp/DisplayVersion, e's Display already prints the help/version.
                eprintln!("Error: {}", e);
                continue;
            }
        };

        if let Err(e) = execute_line(line) {
            // Avoid prefixing with "Error:" because clap::Error display already contains
            // the formatted error/help. Printing just the error avoids duplicated help text.
            eprintln!("Error: {}", e);
        }
    }
}

pub fn execute_line(line: String) -> Result<(), Box<dyn std::error::Error>> {
    // Parse args; handle help/version specially to avoid double-printing.
    match command::get_args(line) {
        Ok(args) => {
            return command::execute_command(args)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
        }
        Err(e) => {
            use clap::error::ErrorKind;
            match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    // Print once here and treat as success so the caller doesn't print again.
                    let _ = e.print();
                    return Ok(());
                }
                _ => {
                    return Err(Box::new(e));
                }
            }
        }
    }
}

fn get_line() -> Result<String, clap::Error> {
    io::stdout().flush().unwrap();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::Io,
            "Failed to read line",
        ));
    }
    Ok(line)
}
