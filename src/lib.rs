use ::std::fs;

use biblatex::{Bibliography, ChunksExt, Entry};

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
    pub filepaths: Vec<String>,
    pub similarity_threshold: f64,
    pub algorithm: Algorithm,
}

pub fn run(config: Config) {
    // todo fix later on, we want to pass a directory not file paths. Also, do not panic!
    // Get the file as string for each of the filepaths in config.filepaths (as an iterator)
    let files = config
        .filepaths
        .iter()
        .map(|filepath| fs::read_to_string(filepath).unwrap());
    // Get the parsed Bibliography from each file string (as an iterator)
    let bibliographies = files.map(|file| Bibliography::parse(&file).unwrap());

    // Unify the bibliography
    // todo move both this and the above to separate functions. See the types in the signatures
    let mut unified_bibliography = Bibliography::new();
    let mut repetitions_found = 0;
    for bibliography in bibliographies {
        repetitions_found += add_to_unified(bibliography, &mut unified_bibliography);
    }

    println!(
        "Repetitions deleted: {}. Final number of entries: {}",
        repetitions_found,
        unified_bibliography.len()
    );
}

// Adds a Bibliography to another unified Bibliography file. Checks for repetitions in the process.
fn add_to_unified(to_add: Bibliography, unified_bibliography: &mut Bibliography) -> i32 {
    // to_add will be consumed by this function
    let mut repetitions = 0;
    for mut entry in to_add.into_iter() {
        if is_present(&entry, unified_bibliography) {
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
fn is_present(entry: &Entry, bibliography: &Bibliography) -> bool {
    let entry_title = entry.title().unwrap().format_verbatim();
    // format_verbatim is necessary to get it as a String instead of [&Chunk]
    for prev_entry in bibliography.iter() {
        if entry_title == prev_entry.title().unwrap().format_verbatim() {
            return true;
        }
    }
    false
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
        let file1 = fs::read_to_string("test_files/test1.bib").expect("Could not read file1");
        let file2 = fs::read_to_string("test_files/test2.bib").expect("Could not read file2");
        let bibliography1 = Bibliography::parse(&file1).expect("Could not parse file1");
        let bibliography2 = Bibliography::parse(&file2).expect("Could not parse file2");
        (bibliography1, bibliography2)
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
        assert_eq!(repetitions2, 8);  // because bibliography1 has 8 entries, 1 is repeated

        // bibliography2 has 2 repetitions (with bibliography 1) -not counting similar entries
        // todo adjust for similar entries later
        assert_eq!(bibliography2.len(), 6);
        let repetitions3 = add_to_unified(bibliography2, &mut unified_bibliography);
        assert_eq!(unified_bibliography.len(), 11);
        assert_eq!(repetitions3, 2);
    }
}
