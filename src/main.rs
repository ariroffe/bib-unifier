use anyhow::{Result};
use clap::Parser;

use bib_unifier;

fn main() -> Result<()>{
    let config = bib_unifier::Config::parse();

    bib_unifier::run(config)?;
    Ok(())
}
