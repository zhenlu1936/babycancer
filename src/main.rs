use babycancer::*;

fn main() {
    loop {
        print!("babycancer> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let tokens = line.split_whitespace();
        let argv = std::iter::once("myprog").chain(tokens);

        let args = match Args::try_parse_from(argv) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        match &args.command {
            Some(Commands::Exit) => {
                println!("Exiting the program.");
                break;
            }

            Some(Commands::Backup(args)) => backup::command_backup(args),

            Some(Commands::Config(args)) => config::command_config(args),

            None => {}
        }
    }
}
