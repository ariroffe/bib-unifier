use ::std::fs;

use biblatex::{Bibliography, ChunksExt, Entry};

pub fn run(file1_path: &str, file2_path: &str) {
    let file1 = fs::read_to_string(file1_path).expect("Could not read file1");
    let file2 = fs::read_to_string(file2_path).expect("Could not read file2");

    let bibliography1 = Bibliography::parse(&file1).expect("Could not parse file1");
    let bibliography2 = Bibliography::parse(&file2).expect("Could not parse file2");

    let mut unified_bibliography = Bibliography::new();
    let mut repetitions_found = 0;
    repetitions_found += add_bibliography_to_unified(bibliography1, &mut unified_bibliography);
    repetitions_found += add_bibliography_to_unified(bibliography2, &mut unified_bibliography);
    println!(
        "Repetitions deleted: {}. Final number of entries: {}",
        repetitions_found,
        unified_bibliography.len()
    );
    // println!("\n\n FINAL BIBLIOGRAPHY: {:?}", unified_bibliography);
}

// Adds a Bibliography to another unified Bibliography file. Checks for repetitions in the process.
fn add_bibliography_to_unified(
    to_add: Bibliography,
    unified_bibliography: &mut Bibliography,
) -> i32 {
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
            println!("Repeated title: {}", entry_title);
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
