use std::path::PathBuf;
use std::{ffi::OsStr, fs};

use anyhow;
use anyhow::Context;
use clap::{ArgEnum, Parser};

mod parsing;
use crate::parsing::*;
mod unify;
use crate::unify::*;

#[derive(Debug, Clone, ArgEnum)]
pub enum Algorithm {
    Levenshtein,
    DamerauLevenshtein,
    Jaro,
    JaroWinkler,
    SorensenDice,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    // Directory where the .bib files are located
    #[clap(
        value_name = "PATH",
        help = "Directory where the .bib files are located"
    )]
    pub path_dir: PathBuf,

    #[clap(
        short,
        long,
        value_name = "PATH",
        validator = validate_output,
        help = "Path (directory + filename) to the desired output file",
        display_order = 0
    )]
    pub output: Option<PathBuf>,

    // To test for similarity between titles, establish a threshold between 0.0 and 1.0
    #[clap(
        short = 't',
        long = "threshold",
        default_value_t = 1.0,
        validator = validate_threshold,
        help = "Value between 0 and 1 to compare entry titles",
        display_order = 2
    )]
    pub similarity_threshold: f64,

    // Algorithm used
    #[clap(
        short,
        long,
        arg_enum,
        default_value_t = Algorithm::Levenshtein,
        help="Algorithm to use to compare similarity",
        display_order = 3
    )]
    pub algorithm: Algorithm,

    // Will not ask for input regarding which entry to keep
    #[clap(
        short,
        long,
        help = "If present, will not ask for input regarding which repeated entry to keep",
        display_order = 1
    )]
    pub silent: bool,

    #[clap(
    short,
    long,
    help = "Default format for entries is bibtex. Setting this flag changes it to biblatex",
    display_order = 4
    )]
    pub biblatex: bool,
}

fn validate_threshold(v: &str) -> Result<(), String> {
    if let Ok(num) = v.parse::<f64>() {
        if num >= 0.0 && num <= 1.0 {
            return Ok(());
        }
    }
    Err(String::from(
        "Threshold must be a valid number between 0 and 1 (e.g. 0.75)",
    ))
}
fn validate_output(v: &str) -> Result<(), String> {
    if let Ok(path) = v.parse::<PathBuf>() {
        if let (Some(_filename), Some(extension)) = (
            path.file_name().and_then(OsStr::to_str),
            path.extension().and_then(OsStr::to_str),
        ) {
            if extension == "bib" {
                return Ok(());
            }
        }
    }
    Err(String::from("Output must be a path to a .bib file"))
}

pub fn run(mut config: Config) -> anyhow::Result<()> {
    // Get the bibliographies
    let filepaths = get_filepaths(config.path_dir.as_path())
        .with_context(|| "A problem was encountered with the input path")?;
    let bibliographies =
        get_files(&filepaths).with_context(|| "A problem was encountered with the input files")?;
    anyhow::ensure!(
        bibliographies.len() > 0,
        "No .bib files in the specified input directory"
    );
    let bibliographies = get_bibliographies(filepaths, bibliographies);

    // Unify the bibliography
    let unified_bibliography = unify_bibliography(bibliographies, &config);

    // Write the result to a file
    // By default, the output path is the input path plus the following file name
    config.path_dir.push("[bib_unifier]bibliography.bib");
    let mut path = config.path_dir.as_path();
    // If the user entered a different output path, change that:
    if let Some(output_path) = &config.output {
        path = output_path.as_path()
    }

    let bibliography_string = match config.biblatex {
        true => unified_bibliography.to_biblatex_string(),
        false => unified_bibliography.to_bibtex_string(),
    };
    fs::write(path, bibliography_string).with_context(|| {
        "A problem was encountered when writing the unified bibliography to the file"
    })?;
    println!("Unified bibliography was written to {:?}.", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use biblatex::Bibliography;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_run() {
        let config = Config {
            path_dir: PathBuf::from(r"bib_files/test_files/"),
            similarity_threshold: 0.7,
            algorithm: Algorithm::Levenshtein,
            silent: true,
            output: None,
            bibtex: false,
        };
        if let Err(_) = run(config) {
            panic!("Error running")
        }

        // Read the output file and check that it has 7 entries
        // (the 6 from test.bib + 1 from rep_in_file.bib)
        let file =
            fs::read_to_string("bib_files/test_files/[bib_unifier]bibliography.bib").unwrap();
        let bibliography = Bibliography::parse(&file).unwrap();
        assert_eq!(bibliography.len(), 7);
    }
}
