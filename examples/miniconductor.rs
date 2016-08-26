//! Parse a docker-compose.yml file and print it to standard output in
//! normalized format.  Try running:
//!
//! ```sh
//! minicondutor docker-compose.in.yml docker-compose.yml
//! ```

extern crate docker_compose;

use docker_compose::v2 as dc;
use std::error;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

/// A catch-all error type to which any other error may be coerced by
/// `try!` (provided it implements `Send` and `Sync`, which most types
/// do).  A handy Rust idiom.
pub type Error = Box<error::Error+Send+Sync>;

/// Create an error using a format string and arguments.
macro_rules! err {
    ($( $e:expr ),*) => (From::from(format!($( $e ),*)));
}

/// Update a `docker-compose.yml` file in place.
fn update(file: &mut dc::File) {

}

/// Our real `main` function.  This is a standard wrapper pattern: we put
/// all the real logic in a function that returns `Result` so that we can
/// use `try!` to handle errors, and we reserve `main` just for error
/// handling.
fn run() -> Result<(), Error> {
    // Parse arguments.
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(err!("Usage: miniconductor <infile> <outfile>"));
    }
    let in_path = Path::new(&args[1]);
    let out_path = Path::new(&args[2]);

    // Transform our file.
    let mut file = try!(dc::File::read_from_path(in_path));
    update(&mut file);
    try!(file.write_to_path(out_path));

    Ok(())
}

fn main() {
    if let Err(ref err) = run() {
        // We use `unwrap` here to turn I/O errors into application panics.
        // If we can't print a message to stderr without an I/O error,
        // the situation is hopeless.
        write!(io::stderr(), "Error: {}", err).unwrap();
        process::exit(1);
    }
}
