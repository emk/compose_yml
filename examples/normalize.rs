//! Parse a docker-compose.yml file and print it to standard output in
//! normalized format.

extern crate docker_compose;

use docker_compose::v2 as dc;
use std::error;
use std::io::{self, Write};

/// A catch-all error type to which any other error may be coerced by
/// `try!` (provided it implements `Send` and `Sync`, which most types
/// do).
pub type Error = Box<error::Error+Send+Sync>;

fn normalize() -> Result<(), Error> {
    let file = try!(dc::File::read(io::stdin()));
    try!(file.write(&mut io::stdout()));
    Ok(())
}

fn main() {
    if let Err(ref err) = normalize() {
        write!(io::stderr(), "Error: {}", err).unwrap();
    }
}
