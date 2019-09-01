extern crate dune;
use dune::{program, Error, Execute, Shell};


fn main() -> Result<(), Error> {
    Shell::new().run();
    Ok(())
}
