use bib_unifier;

fn main() {
    // let file1_path = "test_files/test1.bib";
    // let file2_path = "test_files/test2.bib";

    let config = bib_unifier::Config{
        filepaths: vec![String::from("test_files/test1.bib"), String::from("test_files/test2.bib")],
        similarity_threshold: 0.95,
        algorithm: bib_unifier::Algorithm::Levenshtein,
    };

    bib_unifier::run(config);
}
