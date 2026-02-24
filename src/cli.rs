use clap::{Arg, Command};

pub fn main() -> clap::ArgMatches {
    Command::new("ripping")
        .about("Ripping is the ping toolbox.")
        .author("Joe Borg <joe@josephb.org>")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("number")
                .long("number")
                .short('n')
                .help("Number of pings to send")
                .value_parser(clap::value_parser!(u64))
                .default_value("3"),
        )
        .arg(
            Arg::new("timeout")
                .long("timeout")
                .short('t')
                .help("Timeout of the pings in ms")
                .value_parser(clap::value_parser!(u64))
                .default_value("5000"),
        )
        .arg(
            Arg::new("size")
                .long("size")
                .short('s')
                .help("Size of the pings")
                .value_parser(clap::value_parser!(u64))
                .default_value("56"),
        )
        .arg(
            Arg::new("plot")
                .long("plot")
                .short('p')
                .help("Show latency plot")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("host")
                .index(1)
                .required(true)
                .help("Host to send pings at"),
        )
        .get_matches()
}
