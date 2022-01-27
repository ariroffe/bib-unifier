use ::std::fs;

use biblatex::{Bibliography, ChunksExt, Entry};

fn main() {
    let file1_path = "test_files/test1.bib";
    let file2_path = "test_files/test2.bib";

    let file1 = fs::read_to_string(file1_path).expect("Could not read file1");
    let file2 = fs::read_to_string(file2_path).expect("Could not read file2");

    let bibliography1 = Bibliography::parse(&file1).expect("Could not parse file1");
    let bibliography2 = Bibliography::parse(&file2).expect("Could not parse file2");

    let mut unified_bibliography = Bibliography::new();
    add_bibliography_to_unified(bibliography1, &mut unified_bibliography);
    add_bibliography_to_unified(bibliography2, &mut unified_bibliography);
    println!("Final number of entries: {}", unified_bibliography.len());
    // println!("\n\n FINAL BIBLIOGRAPHY: {:?}", unified_bibliography);
}

fn add_bibliography_to_unified(
    to_add: Bibliography,  // to_add will be consumed by this function
    unified: &mut Bibliography,
) -> &mut Bibliography {
    for mut entry in to_add.into_iter() {
        if !is_present(&entry, unified) {
            entry.key = get_new_citation_key(&entry.key, &unified);
            unified.insert(entry);
        }
    }
    unified
}

// Checks if an entry is already present in a Bibliography
// Should check if the title is present (either identically or similarly, depending on config)
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

// If the Entry is not repeated, we need to assign it a citation key that is not already present
fn get_new_citation_key(old_key: &str, bibliography: &Bibliography) -> String {
    // If the key is not present return it as a String
    if bibliography.get(&old_key).is_none() {
        return old_key.to_owned();
    }
    // If the key is already present
    let mut try_num: u8 = 1;
    loop {
        // Create a new String with form "oldkey_num"
        let new_key = format!("{}_{}", old_key, try_num);
        // If it already exists, sum 1 to the number and try again. Else return the new string
        if bibliography.get(&new_key).is_some() {
            try_num += 1;
        } else {
            return new_key;
        }
    }
}
