use biblatex::{Bibliography, ChunksExt, Entry};
use read_input::prelude::*;
use strsim;
use bunt;

use super::{Algorithm, Config};

// Takes a vec of Bibliography and returns a single Bibliography file with repetitions deleted
// as well as the number of repetitions that were deleted
pub fn unify_bibliography(bibliographies: Vec<Bibliography>, config: &Config) -> Bibliography {
    println!("Unifiying bibliography...");
    let mut unified_bibliography = Bibliography::new();
    let mut repetitions_found = 0;
    for bibliography in bibliographies {
        repetitions_found +=
            add_bibliography_to_unified(bibliography, &mut unified_bibliography, config);
    }
    println!(
        "Found {} repetitions in the bibliography.",
        repetitions_found
    );
    unified_bibliography
}

// Adds a Bibliography to another unified Bibliography file. Checks for repetitions in the process.
fn add_bibliography_to_unified(
    to_add: Bibliography,
    unified_bibliography: &mut Bibliography,
    config: &Config,
) -> i32 {
    // to_add will be consumed by this function
    let mut repetitions = 0;
    // For each entry in the bibliography to add
    for entry in to_add.into_iter() {
        let mut add_entry = true;
        let mut delete_prev = false;
        let mut delete_prev_key = String::new();

        // Compare it to each entry already added to the unified bibliography
        for prev_entry in unified_bibliography.iter() {
            match compare_entries(prev_entry, &entry, config) {
                // If KeepBoth maintain the defaults (add entry and dont delete prev)
                // Do not break (continue looking for similaritiy with the next entries)
                ComparisonResult::KeepBoth => continue,
                // If KeepPrev, do not add and do not delete prev (and break)
                ComparisonResult::KeepPrev => {
                    repetitions += 1;
                    add_entry = false;
                    break;
                }
                // If KeepEntry, do add and do delete prev (and break)
                ComparisonResult::KeepEntry => {
                    repetitions += 1;
                    delete_prev = true;
                    delete_prev_key = prev_entry.key.clone();
                    // Cannot be a reference bc otherwise unified_bib will keep borrowed immutably
                    // I found no better way to do this than to clone this value
                    break;
                }
            }
        }

        // Need to delete before adding the new one, just in case KeepEntry and both have the
        // same key
        if delete_prev {
            unified_bibliography.remove(&delete_prev_key);
        }
        if add_entry {
            add_entry_to_bibliography(entry, unified_bibliography);
            // entry is owned, the to_add Bibliography will be consumed after the outer loop ends
        }
    }
    repetitions
}

#[derive(Debug, PartialEq)]
enum ComparisonResult {
    KeepBoth,
    KeepPrev,
    KeepEntry,
}

// Checks if two entries are similar. If they are, decides what to do
fn compare_entries(prev_entry: &Entry, entry: &Entry, config: &Config) -> ComparisonResult {
    // Both have the same key
    if prev_entry.key == entry.key {
        if !config.silent {
            bunt::println!("{$bold+red}The following entries have the same key:{/$}");
            bunt::println!("{$green}Note: If you wish to keep both, the key to the second entry will be automatically changed.{/$}\n");
        }
        return decide_which_to_keep(prev_entry, entry, config);
    }

    // Both have the doi field set
    if let (Some(prev_doi), Some(entry_doi)) = (&prev_entry.doi(), &entry.doi()) {
        // Same doi is condidered the same entry
        if prev_doi == entry_doi {
            // decide_which_to_keep returns false only if the user decides to preserve prev_entry
            if !config.silent {
                bunt::println!("{$bold+red}The following entries have the same DOI:{/$}\n");
            }
            return decide_which_to_keep(prev_entry, entry, config);
        }
    }

    // Both have the title field set
    if let (Some(prev_title), Some(entry_title)) = (&prev_entry.title(), &entry.title()) {
        // Turn them into Strings instead of the default [&Chunk]
        let prev_title = prev_title.format_verbatim();
        let entry_title = entry_title.format_verbatim();
        // First check for equality between the titles. If they are equal should return true
        // independently of the similarity threshold (for every metric, will be 1).
        // Should be much faster than actually running the strsim algorithms.
        if prev_title == entry_title {
            // decide_which_to_keep returns false only if the user decides to preserve prev_entry
            if !config.silent {
                bunt::println!("{$bold+red}The following entries have the same title:{/$}\n");
            }
            return decide_which_to_keep(prev_entry, entry, config);
        } else if config.similarity_threshold < 1.0
            && test_title_similarity(&prev_title, &entry_title, config)
        {
            // decide_which_to_keep returns false only if the user decides to preserve prev_entry
            if !config.silent {
                bunt::println!("{$bold+red}The following entries have the similar titles:{/$}\n");
            }
            return decide_which_to_keep(prev_entry, entry, config);
        }
    }

    ComparisonResult::KeepBoth
}

fn test_title_similarity(title1: &str, title2: &str, config: &Config) -> bool {
    let similarity = match config.algorithm {
        Algorithm::Levenshtein => strsim::normalized_levenshtein(title1, title2),
        Algorithm::DamerauLevenshtein => strsim::normalized_damerau_levenshtein(title1, title2),
        Algorithm::Jaro => strsim::jaro(title1, title2),
        Algorithm::JaroWinkler => strsim::jaro_winkler(title1, title2),
        Algorithm::SorensenDice => strsim::sorensen_dice(title1, title2),
    };
    similarity >= config.similarity_threshold
}

// Given two entries which we have previously decided are similar, decides whether to keep
// the old one (only return false), the new one (delete the old one and return true), or both (just return true)
fn decide_which_to_keep(prev_entry: &Entry, entry: &Entry, config: &Config) -> ComparisonResult {
    // In silent mode, always retain the old entry
    if config.silent {
        return ComparisonResult::KeepPrev;
    }

    // Check if both entries are equal in all fields (and again, retain the old one)
    if prev_entry == entry {
        return ComparisonResult::KeepPrev;
    }

    // Otherwise, ask which
    let (prev_entry_string, entry_string) = match config.biblatex {
        true => (prev_entry.to_biblatex_string(), entry.to_biblatex_string()),
        false => (prev_entry.to_bibtex_string(), entry.to_bibtex_string()),
    };
    bunt::println!(
        "{$green}1-{/$} {}\n\n{$green}2-{/$} {}\n\n{$blue}Do you wish to keep the first (1), the second (2) or both (3)?{/$}",
        prev_entry_string,
        entry_string
    );
    let input: u32 = input()
        .repeat_msg("Enter your choice: ")
        .err("The value must be either 1, 2 or 3.")
        .min_max(1, 3)
        .get();
    println!();

    if input == 3 {
        // 3 means keep both
        return ComparisonResult::KeepBoth;
    } else if input == 1 {
        // 1 means keep the old entry
        return ComparisonResult::KeepPrev;
    } else {
        // 2 means keep the new one
        return ComparisonResult::KeepEntry;
    }
}

// Adds an Entry to a Bibliography, checking that the key is not repeated
fn add_entry_to_bibliography(mut entry: Entry, bibliography: &mut Bibliography) {
    // If it is not present, we add it to the unified bibliography
    // First check if the citation key is already present
    if bibliography.get(&entry.key).is_some() {
        // If it is, get a new key, otherwise it won't be added correctly
        entry.key = get_new_citation_key(&entry.key, &bibliography);
    }
    // Add it
    bibliography.insert(entry);
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
    use std::fs;
    use std::path::PathBuf;

    fn setup() -> (Bibliography, Config) {
        let file = fs::read_to_string("bib_files/test_files/test.bib").unwrap();
        let bibliography = Bibliography::parse(&file).unwrap();
        let config = Config {
            path_dir: PathBuf::from(r"bib_files/test_files/"),
            similarity_threshold: 1.0,
            algorithm: Algorithm::Levenshtein,
            silent: true,
            output: None,
            bibtex: false,
        };
        (bibliography, config)
    }

    #[test]
    fn test_add_to_unified() {
        let (bibliography1, config) = setup();
        let bibliography1_copy = bibliography1.clone();
        let mut unified_bibliography = Bibliography::new();

        let repetitions =
            add_bibliography_to_unified(bibliography1, &mut unified_bibliography, &config);
        assert_eq!(unified_bibliography.len(), 6);
        assert_eq!(repetitions, 0);

        // If we attempt to add bibliography1 again, we should not get any new entries
        let repetitions2 =
            add_bibliography_to_unified(bibliography1_copy, &mut unified_bibliography, &config);
        assert_eq!(unified_bibliography.len(), 6);
        assert_eq!(repetitions2, 6);
    }

    #[test]
    fn test_rep_in_file() {
        let file = fs::read_to_string("bib_files/test_files/rep_in_file.bib").unwrap();
        let bibliography1 = Bibliography::parse(&file).unwrap();
        let config = Config {
            path_dir: PathBuf::from(r"bib_files/test_files/"),
            similarity_threshold: 1.0,
            algorithm: Algorithm::Levenshtein,
            silent: true,
            output: None,
            bibtex: false,
        };
        let mut bibliography = Bibliography::new();

        // It should delete one repetition (the one present in bibliography1)
        assert_eq!(
            add_bibliography_to_unified(bibliography1, &mut bibliography, &config),
            1
        )
    }

    #[test]
    fn test_all_same_fields() {
        let (mut bibliography1, mut config) = setup();
        config.silent = false;

        let file = fs::read_to_string("bib_files/test_files/all_same_fields.bib").unwrap();
        let bibliography2 = Bibliography::parse(&file).unwrap();

        let montague1 = bibliography1
            .get("Montague1973QuantificationOrdinaryEnglish")
            .unwrap();
        let montague2 = bibliography2
            .get("Montague1973QuantificationOrdinaryEnglish")
            .unwrap();
        // Should keep the first entry even if silent is false since all fields are identical
        assert_eq!(
            compare_entries(montague1, montague2, &config),
            ComparisonResult::KeepPrev
        );

        // It deletes one repetition
        assert_eq!(
            add_bibliography_to_unified(bibliography2, &mut bibliography1, &config),
            1
        )
    }

    #[test]
    fn test_only_same_key() {
        let (mut bibliography1, config) = setup();

        let file = fs::read_to_string("bib_files/test_files/only_same_key.bib").unwrap();
        let bibliography2 = Bibliography::parse(&file).unwrap();

        let frege1 = bibliography1.get("FregeGrundlagen").unwrap();
        let frege2 = bibliography2.get("FregeGrundlagen").unwrap();
        assert_eq!(
            compare_entries(frege1, frege2, &config),
            ComparisonResult::KeepPrev
        );

        assert_eq!(
            add_bibliography_to_unified(bibliography2, &mut bibliography1, &config),
            1
        )
    }

    #[test]
    fn test_only_same_doi() {
        let (mut bibliography1, config) = setup();

        let file = fs::read_to_string("bib_files/test_files/only_same_doi.bib").unwrap();
        let bibliography2 = Bibliography::parse(&file).unwrap();

        let hardgree1 = bibliography1.get("Hardegree2005completeness").unwrap();
        let hardgree2 = bibliography2.get("Hardegree2005completeness1").unwrap();
        assert_eq!(
            compare_entries(hardgree1, hardgree2, &config),
            ComparisonResult::KeepPrev
        );

        assert_eq!(
            add_bibliography_to_unified(bibliography2, &mut bibliography1, &config),
            1
        )
    }

    #[test]
    fn test_only_same_title() {
        let (mut bibliography1, config) = setup();

        let file = fs::read_to_string("bib_files/test_files/only_same_title.bib").unwrap();
        let bibliography2 = Bibliography::parse(&file).unwrap();

        let prior1 = bibliography1.get("Prior1960").unwrap();
        let prior2 = bibliography2.get("Prior1961").unwrap();
        assert_eq!(
            compare_entries(prior1, prior2, &config),
            ComparisonResult::KeepPrev
        );

        assert_eq!(
            add_bibliography_to_unified(bibliography2, &mut bibliography1, &config),
            1
        )
    }

    #[test]
    fn test_similar_title() {
        let (mut bibliography1, mut config) = setup();

        let file = fs::read_to_string("bib_files/test_files/similar_title.bib").unwrap();
        let bibliography2 = Bibliography::parse(&file).unwrap();

        // With a similarity threshold of 1, it should keep both
        let carnap1 = bibliography1.get("Carnap1942").unwrap();
        let carnap2 = bibliography2.get("Carnap1942_1").unwrap();
        assert_eq!(
            compare_entries(carnap1, carnap2, &config),
            ComparisonResult::KeepBoth
        );

        // With a similarity threshold of 0.7, it should keep only the first
        config.similarity_threshold = 0.7;
        let carnap1 = bibliography1.get("Carnap1942").unwrap();
        let carnap2 = bibliography2.get("Carnap1942_1").unwrap();
        assert_eq!(
            compare_entries(carnap1, carnap2, &config),
            ComparisonResult::KeepPrev
        );

        assert_eq!(
            add_bibliography_to_unified(bibliography2, &mut bibliography1, &config),
            2
        )
    }

    #[test]
    fn test_get_new_citation_key() {
        let (mut bibliography1, _config) = setup();

        assert_eq!(
            get_new_citation_key("Carnap1942", &bibliography1),
            String::from("Carnap1942_1")
        );

        // Lets get Carnap1942, insert it into 1 again, withe key Carnap1942_1
        let mut carnap2 = bibliography1.get_resolved("Carnap1942").unwrap().clone();
        carnap2.key = String::from("Carnap1942_1");
        bibliography1.insert(carnap2);
        // Now get_new_citation_key should return "Carnap1942_2"
        assert_eq!(
            get_new_citation_key("Carnap1942", &bibliography1),
            String::from("Carnap1942_2")
        );
    }
}
