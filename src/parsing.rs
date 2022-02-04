use std::path::{Path, PathBuf};
use std::{ffi::OsStr, fs, io};

use biblatex::{Bibliography, ChunksExt};

// Read a directory path and return a vec of the .bib filepaths (i.e. PathBuf's) inside it
pub fn get_filepaths(path_dir: &Path) -> io::Result<Vec<PathBuf>> {
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
pub fn include_path(path: &Path) -> bool {
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
pub fn get_files(filepaths: &Vec<PathBuf>) -> io::Result<Vec<String>> {
    let mut files = vec![];
    for path in filepaths.iter() {
        files.push(fs::read_to_string(path)?);
    }
    Ok(files)
}

// Given a vec of Strings (contents of the .bib files), return a vec of Bibliography
pub fn get_bibliographies(filepaths: Vec<PathBuf>, file_contents: Vec<String>) -> Vec<Bibliography> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parsing() {
        let file1 = fs::read_to_string("bib_files/test_files/test1.bib").unwrap();
        let file2 = fs::read_to_string("bib_files/test_files/test2.bib").unwrap();
        let bibliography1 = Bibliography::parse(&file1).unwrap();
        let bibliography2 = Bibliography::parse(&file2).unwrap();
        assert_eq!(bibliography1.len(), 8);
        assert_eq!(bibliography2.len(), 6);
        assert!(bibliography1.get("lalala").is_none());
        let prior = bibliography1
            .get("Prior1960")
            .expect("No entry with key Prior1960");
        let prior_title = prior.title().unwrap().format_verbatim();
        assert_eq!(&prior_title, "The Runabout Inference-Ticket")
    }
}