mod cli;
mod ping;

fn main() {
    let matches = cli::main();

    let number = cli::value_t!(matches.value_of("number"), u64).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let timeout = cli::value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let size = cli::value_t!(matches.value_of("size"), u64).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let plot = matches.is_present("plot");

    let host = matches.value_of("host").unwrap();

    ping::run(host, number, timeout, size, plot);
}
