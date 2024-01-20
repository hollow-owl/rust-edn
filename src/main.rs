use std::{fs::File, io::Read};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "edn.pest"] // relative to src
struct EdnParser;
// symbol
// [:]?([\D&&[^/]].*/)?([\D&&[^/]][^/]*)
// int
// ([-+]?)(?:(0)|([1-9][0-9]*)|0[xX]([0-9A-Fa-f]+)|0([0-7]+)|([1-9][0-9]?)[rR]([0-9A-Za-z]+)|0[0-9]+)(N)?
// ratio
// ([-+]?[0-9]+)/([0-9]+)
// float
// ([-+]?[0-9]+(\.[0-9]*)?([eE][-+]?[0-9]+)?)(M)?

// pub enum Value {
//     Nil,
//     Symbol(String),
//     Keyword(String),
//     String(String),
//     Bool(Bool),
//     Char(Char),
//     Int(usize),
//     Float(f64),
//     List(Vec<Value>),
//     Vec(Vec<Value>),
//     Map(HashMap<Value,Value>),
//     Set(Set<Value>),
//     // TaggedElement(tag,value)
// }

fn main() {
    let mut file = File::open("test/learnxiny.edn").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let a = EdnParser::parse(Rule::edn,&contents).unwrap();
    dbg!(a);
}

#[cfg(test)]
mod tests {

    use std::{path::PathBuf, fs};
    use pest::error::Error;
    use walkdir::WalkDir;
    
    use super::*;
    
    const VALID_FOLDERS: [&str; 4] = [
        "./test",
        "./examples/edn-tests/valid-edn",
        "./examples/edn-tests/performance",
        "./examples/antlr-grammars-v4/edn/examples",
    ];
    const INVALID_FOLDERS: [&str; 1] = ["./examples/edn-tests/invalid-edn"];

    fn walk_dir(folders: Vec<&str>) -> Vec<(PathBuf, Result<(), Error<Rule>>)> {
        let files = folders
        .iter()
        .flat_map(WalkDir::new)
        .flatten();

        let mut results = Vec::new();
        for entry in files {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "edn") {
                let file = fs::read_to_string(entry.path()).unwrap();
                let parse_result = EdnParser::parse(Rule::edn, &file).map(|_| ());
                results.push((entry.path().to_owned(), parse_result));
            }
        }
        results
    }

    fn all_ok(results: impl Iterator<Item = (PathBuf, Result<(),Error<Rule>>)>) {
        let mut has_err = false;
        for (path, err) in results {
            if err.is_err() {
                dbg!(path);
                let err = err.unwrap_err();
                println!("{}",err);
                has_err = true;
            }
        }
        assert!(!has_err)
    }

    fn all_err(results: impl Iterator<Item= (PathBuf, Result<(),Error<Rule>>)>) {
        let mut has_ok = false;
        for (path, err) in results {
            if err.is_ok() {
                dbg!(path);
                has_ok = true;
            }
        }
        assert!(!has_ok);
    }

    #[test]
    fn test_edn_files() {
        all_ok(walk_dir(VALID_FOLDERS.into()).into_iter());
    }

    #[test]
    fn test_invalid_edn_files() {
        all_err(walk_dir(INVALID_FOLDERS.into()).into_iter());
    }
}
