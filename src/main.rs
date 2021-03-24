extern crate clap;
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
    if let Err(e) = cscope::parse_database(&Path::new(fname)) {
        println!("Error detected.");
        return Err(e);
    }
    Ok(())
}
