use ::std::fs;

fn main() {
    let file1_path = "test_files/test1.bib";
    let file2_path = "test_files/test2.bib";

    let file1 = fs::read_to_string(file1_path).expect("Could not read file1");
    println!("FILE 1: {}", file1);

    let file2 = fs::read_to_string(file2_path).expect("Could not read file2");
    println!("FILE 2: {}", file2);
}
