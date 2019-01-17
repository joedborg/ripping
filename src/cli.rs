use clap::{App, Arg};
pub use clap::value_t;

pub fn main<'a>() -> clap::ArgMatches<'a> {
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
                .short("n")
                .help("Number of pings to send")
                .takes_value(true)
                .default_value("3")
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .short("t")
                .help("Timeout of the pings")
                .takes_value(true)
                .default_value("5")
        )
        .arg(
            Arg::with_name("host")
                .index(1)
                .required(true)
                .help("Host to send pings at")
                .takes_value(true)
        );

    return app.get_matches();
}
