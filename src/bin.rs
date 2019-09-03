extern crate dune;
use dune::{Error, Shell, LOGO};

fn main() -> Result<(), Error> {
    println!("{}", LOGO);
    Shell::new().run();
    Ok(())
}
