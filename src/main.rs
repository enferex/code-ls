extern crate clap;
use clap::{App, Arg};
use std::path::Path;
mod cscope;

fn main() {
    let args = App::new("code-ls")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .short("f")
                .help("cscope database file (it must be uncompressed).")
                .required(true),
        )
        .get_matches();

    let fname = args.value_of("file").unwrap();
    match cscope::parse_database(&Path::new(fname)) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
