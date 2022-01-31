use std::path::{Path, PathBuf};
use std::{ffi::OsStr, fs, io};

use biblatex::{Bibliography, ChunksExt, Entry};
use strsim;

#[derive(Debug)]
pub enum Algorithm {
    Levenshtein,
    DamerauLevenshtein,
    Jaro,
    JaroWinkler,
    SorensenDice,
}

#[derive(Debug)]
pub struct Config {
    pub path_dir: PathBuf,
    pub similarity_threshold: f64,
    pub algorithm: Algorithm,
}

pub fn run(mut config: Config) -> Result<(), io::Error> {
    // Get the bibliographies
    let filepaths = get_filepaths(config.path_dir.as_path())?;
    let bibliographies = get_files(&filepaths)?;
    if bibliographies.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "No .bib files in the specified directory",
        ));
    }
    let bibliographies = get_bibliographies(filepaths, bibliographies);

    // Unify the bibliography
    let unified_bibliography = unify_bibliography(bibliographies, &config);

    // Write the result to a file
    config.path_dir.push("[bib_unifier]bibliography.bib");
    fs::write(&config.path_dir, unified_bibliography.to_bibtex_string())?;
    println!("Unified bibliography was written to {:?}.", config.path_dir);
    Ok(())
}

// Read a directory path and return a vec of the .bib filepaths (i.e. PathBuf's) inside it
fn get_filepaths(path_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut bib_filepaths = vec![];

    for path in fs::read_dir(path_dir)? {
        let path = path?.path();
        if include_path(path.as_path()) {
            bib_filepaths.push(path);
        }
    }
    Ok(bib_filepaths)
}

// Check whether a file needs to be included in the unification or not
// (has to be a .bib file and not begin with [bib_unifier])
fn include_path(path: &Path) -> bool {
    // Check that the extension is .bib
    if let Some("bib") = path.extension().and_then(OsStr::to_str) {
        // Check that the file is not a previous output of the program itself
        // i.e. that it does not begin with [bib_unifier]
        if let Some(filename) = path.file_name().and_then(OsStr::to_str) {
            if !filename.starts_with("[bib_unifier]") {
                return true;
            }
        }
    }
    false
}

// Given a vec of PathBufs, return a vec of the file contents
fn get_files(filepaths: &Vec<PathBuf>) -> io::Result<Vec<String>> {
    let mut files = vec![];
    for path in filepaths.iter() {
        files.push(fs::read_to_string(path)?);
    }
    Ok(files)
}

// Given a vec of Strings (contents of the .bib files), return a vec of Bibliography
fn get_bibliographies(filepaths: Vec<PathBuf>, file_contents: Vec<String>) -> Vec<Bibliography> {
    let mut bibliographies = vec![];
    for (idx, file_content) in file_contents.into_iter().enumerate() {
        match Bibliography::parse(&file_content) {
            Some(bibliography) => bibliographies.push(bibliography),
            None => eprintln!(
                "File {:?} could not be processed and was ignored.",
                filepaths[idx]
            ),
        }
    }
    bibliographies
}

// Takes a vec of Bibliography and returns a single Bibliography file with repetitions deleted
// as well as the number of repetitions that were deleted
fn unify_bibliography(bibliographies: Vec<Bibliography>, config: &Config) -> Bibliography {
    let mut unified_bibliography = Bibliography::new();
    let mut repetitions_found = 0;
    for bibliography in bibliographies {
        repetitions_found += add_to_unified(bibliography, &mut unified_bibliography, config);
    }
    println!(
        "Found {} repetitions in the bibliography.",
        repetitions_found
    );
    unified_bibliography
}

// Adds a Bibliography to another unified Bibliography file. Checks for repetitions in the process.
fn add_to_unified(
    to_add: Bibliography,
    unified_bibliography: &mut Bibliography,
    config: &Config,
) -> i32 {
    // to_add will be consumed by this function
    let mut repetitions = 0;
    for mut entry in to_add.into_iter() {
        if is_present(&entry, unified_bibliography, config) {
            //println!("Repeated entry: {:?}", &entry);
            repetitions += 1
        } else {
            // If it is not present, we add it to the unified bibliography
            // First check if the citation key is already present
            if unified_bibliography.get(&entry.key).is_some() {
                // If it is, get a new key, otherwise it won't be added correctly
                entry.key = get_new_citation_key(&entry.key, &unified_bibliography);
            }
            // Add it
            unified_bibliography.insert(entry);
        }
    }
    repetitions
}

// Checks if an entry is already present in a Bibliography, with a given similarity threshold
fn is_present(entry: &Entry, bibliography: &Bibliography, config: &Config) -> bool {
    let entry_doi = entry.doi();
    let entry_title = entry.title();

    for prev_entry in bibliography.iter() {
        // Both have the doi field set
        if let (Some(e_doi), Some(p_doi)) = (&entry_doi, &prev_entry.doi()) {
            // Same doi considers it the same entry
            if e_doi == p_doi {
                return true;
            }
        }

        // Both have the title field set
        if let (Some(e_title), Some(p_title)) = (&entry_title, &prev_entry.title()) {
            // Turn them into Strings instead of the default [&Chunk]
            let e_title = e_title.format_verbatim();
            let p_title = p_title.format_verbatim();

            // First check for equality between the titles. If they are equal should return true
            // independently of the similarity threshold (for every metric, will be 1).
            // Should be much faster than actually running the strsim algorithms.
            if e_title == p_title {
                return true;
            } else if config.similarity_threshold < 1.0
                && test_similarity(&e_title, &p_title, config)
            {
                return true;
            }
        }
    }
    false
}

fn test_similarity(title1: &str, title2: &str, config: &Config) -> bool {
    let similarity = match config.algorithm {
        Algorithm::Levenshtein => strsim::normalized_levenshtein(title1, title2),
        Algorithm::DamerauLevenshtein => strsim::normalized_damerau_levenshtein(title1, title2),
        Algorithm::Jaro => strsim::jaro(title1, title2),
        Algorithm::JaroWinkler => strsim::jaro_winkler(title1, title2),
        Algorithm::SorensenDice => strsim::sorensen_dice(title1, title2),
    };
    // if similarity >= config.similarity_threshold {
    //     println!("{} ::: {}", title1, title2);
    // }
    similarity >= config.similarity_threshold
}

// Gets a new, non-repeated, citation key for an Entry
fn get_new_citation_key(old_key: &str, bibliography: &Bibliography) -> String {
    let mut try_num: u8 = 1;
    loop {
        // Create a new String with form "oldkey_trynum" (e.g. "Roffe2021_1")
        let new_key = format!("{}_{}", old_key, try_num);
        // If it already exists, sum 1 to the number and try again. Else return the new string
        if bibliography.get(&new_key).is_some() {
            try_num += 1;
        } else {
            return new_key;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Bibliography, Bibliography) {
        let file1 =
            fs::read_to_string("bib_files/test_files/test1.bib").expect("Could not read file1");
        let file2 =
            fs::read_to_string("bib_files/test_files/test2.bib").expect("Could not read file2");
        let bibliography1 = Bibliography::parse(&file1).expect("Could not parse file1");
        let bibliography2 = Bibliography::parse(&file2).expect("Could not parse file2");
        (bibliography1, bibliography2)
    }

    #[test]
    fn test_get_filepaths() {
        let path_dir = PathBuf::from("bib_files/test_files/");
        let filepaths = get_filepaths(path_dir.as_path()).unwrap();
        assert_eq!(filepaths.len(), 2);
        assert_eq!(
            filepaths[0].to_str().unwrap(),
            "bib_files/test_files/test1.bib"
        );
        assert_eq!(
            filepaths[1].to_str().unwrap(),
            "bib_files/test_files/test2.bib"
        );
    }

    #[test]
    fn test_include_path() {
        assert!(include_path(
            PathBuf::from("bib_files/test_files/test1.bib").as_path()
        ));
        assert!(!include_path(
            PathBuf::from("bib_files/test_files/test1.txt").as_path()
        ));
        assert!(!include_path(
            PathBuf::from("bib_files/test_files/[bib_unifier]test1.bib").as_path()
        ));
    }

    #[test]
    fn test_no_bib_files() {
        // Check that the program fails appropriately when there are no .bib files in the directory
        let config = Config {
            path_dir: PathBuf::from(r"bib_files/no_bib_files/"),
            similarity_threshold: 0.95,
            algorithm: Algorithm::Levenshtein,
        };
        let result = run(config).map_err(|e| e.kind());
        assert_eq!(result, Err(io::ErrorKind::Other))
    }

    #[test]
    fn test_parsing() {
        let (bibliography1, bibliography2) = setup();
        assert_eq!(bibliography1.len(), 8);
        assert_eq!(bibliography2.len(), 6);
        assert!(bibliography1.get("lalala").is_none());
        let prior = bibliography1
            .get("Prior1960")
            .expect("No entry with key Prior1960");
        let prior_title = prior.title().unwrap().format_verbatim();
        assert_eq!(&prior_title, "The Runabout Inference-Ticket")
    }

    #[test]
    fn test_is_present() {
        let (bibliography1, bibliography2) = setup();

        // Identical title
        let montague = bibliography1
            .get("Montague1973QuantificationOrdinaryEnglish")
            .unwrap();
        assert!(is_present(montague, &bibliography2));
        let frege = bibliography1
            .get("FregeGrundlagen")
            .expect("No entry with key FregeGrundlagen");
        assert!(!is_present(frege, &bibliography2));

        // Similar title (for now should return false, todo change to true later on)
        let bps = bibliography1
            .get("BPS2018-WIAPL")
            .expect("No entry with key BPS2018-WIAPL");
        assert!(!is_present(bps, &bibliography2));
        let carnap = bibliography1
            .get("Carnap1942")
            .expect("No entry with key Carnap1942");
        assert!(!is_present(carnap, &bibliography2));
    }

    #[test]
    fn test_get_new_citation_key() {
        let (mut bibliography1, bibliography2) = setup();

        assert_eq!(
            get_new_citation_key("Carnap1942", &bibliography1),
            String::from("Carnap1942_1")
        );

        // Lets get Carnap1942 from bibliography2, insert it into 1 again, withe key Carnap1942_1
        let mut carnap2 = bibliography2
            .get_resolved("Carnap1942")
            .expect("No entry with key Carnap1942");
        carnap2.key = String::from("Carnap1942_1");
        bibliography1.insert(carnap2);
        // Now get_new_citation_key should return "Carnap1942_2"
        assert_eq!(
            get_new_citation_key("Carnap1942", &bibliography1),
            String::from("Carnap1942_2")
        );
    }

    #[test]
    fn test_add_to_unified() {
        let (bibliography1, bibliography2) = setup();
        let bibliography1_copy = bibliography1.clone();
        let mut unified_bibliography = Bibliography::new();

        // bibliography1 has 1 repeated entry inside
        assert_eq!(bibliography1.len(), 8);
        let repetitions1 = add_to_unified(bibliography1, &mut unified_bibliography);
        assert_eq!(unified_bibliography.len(), 7);
        assert_eq!(repetitions1, 1);

        // If we attempt to add bibliography1 again, we should not get any new entries
        let repetitions2 = add_to_unified(bibliography1_copy, &mut unified_bibliography);
        assert_eq!(unified_bibliography.len(), 7);
        assert_eq!(repetitions2, 8); // because bibliography1 has 8 entries, 1 is repeated

        // bibliography2 has 2 repetitions (with bibliography 1) -not counting similar entries
        // todo adjust for similar entries later
        assert_eq!(bibliography2.len(), 6);
        let repetitions3 = add_to_unified(bibliography2, &mut unified_bibliography);
        assert_eq!(unified_bibliography.len(), 11);
        assert_eq!(repetitions3, 2);
    }
}
