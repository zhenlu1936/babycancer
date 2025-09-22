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

            Some(Commands::Backup(args)) => backup::command_backup(args)
                .unwrap_or_else(|err| eprintln!("Backup command failed: {}", err)),

            Some(Commands::Config(args)) => config::command_config(args)
                .unwrap_or_else(|err| eprintln!("Config command failed: {}", err)),

            Some(Commands::Reset(args)) => config::command_reset(args)
                .unwrap_or_else(|err| eprintln!("Reset command failed: {}", err)),

            Some(Commands::TimedBackup(args)) => backup::command_timed_backup(args)
                .unwrap_or_else(|err| eprintln!("Timed backup command failed: {}", err)),

            Some(Commands::RealtimeBackup(args)) => backup::command_realtime_backup(args)
                .unwrap_or_else(|err| eprintln!("Realtime backup command failed: {}", err)),

            None => {}
        }
    }
}
