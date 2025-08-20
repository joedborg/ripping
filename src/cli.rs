pub use clap::value_t;
use clap::{App, Arg};

pub fn main() -> clap::ArgMatches {
    let app = App::new("ripping")
        .about(
            "\
                Ripping is the ping toolbox.\
            ",
        )
        .author("Joe Borg <joe@josephb.org>")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("number")
                .long("number")
                .short('n')
                .help("Number of pings to send")
                .takes_value(true)
                .default_value("3"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .short('t')
                .help("Timeout of the pings in ms")
                .takes_value(true)
                .default_value("5000"),
        )
        .arg(
            Arg::with_name("size")
                .long("size")
                .short('s')
                .help("Size of the pings")
                .takes_value(true)
                .default_value("56"),
        )
        .arg(
            Arg::with_name("plot")
                .long("plot")
                .short('p')
                .help("Show latency plot")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("host")
                .index(1)
                .required(true)
                .help("Host to send pings at")
                .takes_value(true),
        );

    return app.get_matches();
}
