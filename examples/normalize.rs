//! Parse a docker-compose.yml file and print it to standard output in
//! normalized format.

use compose_yml::v2 as dc;
use std::io::{self, Write};

fn normalize() -> dc::Result<()> {
    let file = dc::File::read(io::stdin())?;
    file.write(&mut io::stdout())?;
    Ok(())
}

fn main() {
    if let Err(ref err) = normalize() {
        write!(io::stderr(), "Error: {}", err).unwrap();
    }
}
