use std::io;

use anyhow::{Context, Result};
use clap::Parser;

use bib_unifier;

fn main() -> Result<()>{
    let config = bib_unifier::Config::parse();

    bib_unifier::run(config).with_context(|| "An error occured and the program was terminated")?;
    Ok(())
}
