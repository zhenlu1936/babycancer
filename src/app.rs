use crate::*;

pub fn get_line() -> Result<String, clap::Error> {
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

pub fn get_args(line: String) -> Result<Args, clap::Error> {
    let tokens = line.split_whitespace();
    let argv = std::iter::once("myprog").chain(tokens);
    let args = match Args::try_parse_from(argv) {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(args)
}

pub fn execute_command(args: Args) -> Result<(), io::Error> {
    match &args.command {
        Some(Commands::Exit) => {
            println!("Exiting the program.");
            std::process::exit(0);
        }

        Some(Commands::Backup(args)) => backup::command_backup(args),

        Some(Commands::Config(args)) => config::command_config(args),

		Some(Commands::Reset(args)) => config::command_reset(args),

        None => {
			println!("No command provided. Use --help for more information.");
			Ok(())
		}
    }
}
