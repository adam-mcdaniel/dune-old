extern crate dune;
use dune::{Error, Shell};

fn main() -> Result<(), Error> {
    Shell::new().run();
    Ok(())
}
