use ::std::fs;

use biblatex::Bibliography;
use biblatex::ChunksExt;

fn main() {
    let file1_path = "test_files/test1.bib";
    let file2_path = "test_files/test2.bib";

    // todo Change the expect
    let file1 = fs::read_to_string(file1_path).expect("Could not read file1");
    let file2 = fs::read_to_string(file2_path).expect("Could not read file2");

    // todo Change the expect
    let bibliography1 = Bibliography::parse(&file1).expect("Could not parse file1");
    let bibliography2 = Bibliography::parse(&file2).expect("Could not parse file2");

    let first_entry = bibliography2.get_resolved("CERVR-PMTI").unwrap();
    println!("FIRST ENTRY TITLE: {:?}", first_entry.title().unwrap().format_verbatim());
}

