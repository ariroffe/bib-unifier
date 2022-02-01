use std::io;

use clap::Parser;

use bib_unifier;

fn main() -> Result<(), io::Error>{
    let config = bib_unifier::Config::parse();

    bib_unifier::run(config)?;
    Ok(())
}
