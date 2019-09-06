extern crate dune;
use dune::{Error, Shell, INFO, LOGO};

fn main() -> Result<(), Error> {
    println!("{}\n{}", INFO, LOGO);
    Shell::new().run();
    Ok(())
}
