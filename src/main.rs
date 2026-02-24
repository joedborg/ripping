mod cli;
mod ping;

fn main() {
    let matches = cli::main();

    let number = *matches.get_one::<u64>("number").unwrap();
    let timeout = *matches.get_one::<u64>("timeout").unwrap();
    let size = *matches.get_one::<u64>("size").unwrap();
    let plot = matches.get_flag("plot");
    let host = matches.get_one::<String>("host").unwrap();

    ping::run(host, number, timeout, size, plot);
}
