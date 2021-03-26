extern crate clap;
extern crate prettytable;
use clap::{App, Arg};
use std::path::Path;
mod cscope;

fn main() -> Result<(), std::io::Error> {
    let args = App::new("cscopetree")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .short("f")
                .help("cscope database file.")
                .required(true),
        )
        .get_matches();

    let fname = args.value_of("file").unwrap();
    cscope::parse_database(&Path::new(fname))
}
