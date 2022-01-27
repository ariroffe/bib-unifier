use bib_unifier;

fn main() {
    let file1_path = "test_files/test1.bib";
    let file2_path = "test_files/test2.bib";

    bib_unifier::run(file1_path, file2_path);
}
