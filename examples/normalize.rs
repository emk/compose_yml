//! Parse a docker-compose.yml file and print it to standard output in
//! normalized format.

extern crate compose_yml;

use compose_yml::v2 as dc;
use std::io::{self, Write};

fn normalize() -> dc::Result<()> {
    let file = try!(dc::File::read(io::stdin()));
    try!(file.write(&mut io::stdout()));
    Ok(())
}

fn main() {
    if let Err(ref err) = normalize() {
        write!(io::stderr(), "Error: {}", err).unwrap();
    }
}
