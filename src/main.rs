use std::path::PathBuf;
use std::io;

use bib_unifier;

fn main() -> Result<(), io::Error>{
    let config = bib_unifier::Config {
        path_dir: PathBuf::from(r"bib_files/test_files/"),
        similarity_threshold: 0.95,
        algorithm: bib_unifier::Algorithm::Levenshtein,
    };

    bib_unifier::run(config)?;
    Ok(())
}
