use std::{fs::File, io::Read};
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

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

fn read_files(dirs: Vec<&str>) -> Vec<Result<PathBuf, (PathBuf, pest::error::Error<Rule>)>> {
    let dirs: Vec<_> = dirs.into_iter().filter(|x| x.ends_with("edn")).collect();
    let dirs = dirs
        .into_iter()
        .map(Path::new)
        .map(|x| {
            visit_dirs(x, &|y| {
                read_edn_file(&y.path())
                    .map(|_| y.path())
                    .map_err(|e| (y.path(), e))
            })
        })
        .flatten()
        .collect();
    dirs
}

fn read_edn_file(file: &Path) -> Result<(), pest::error::Error<Rule>> {
    let mut file = File::open(file).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let _ = EdnParser::parse(Rule::edn, &contents)?;
    Ok(())
}

fn visit_dirs<T>(dir: &Path, cb: &dyn Fn(&DirEntry) -> T) -> Vec<T> {
    let mut out = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                out.extend(visit_dirs(&path, cb).into_iter());
            } else {
                // println!("{:?}", &entry);
                out.push(cb(&entry));
            }
        }
    }
    out
}
fn main() {
    // let mut file = File::open("test/learnxiny.edn").unwrap();
    // let mut contents = String::new();
    // file.read_to_string(&mut contents).unwrap();
    // let a = EdnParser::parse(Rule::edn,&contents).unwrap();
    // dbg!(a);
}

#[cfg(test)]
mod tests {

    use super::*;
    const VALID_FOLDERS: [&str; 4] = [
        "./test",
        "./examples/edn-tests/valid-edn",
        "./examples/edn-tests/performance",
        "./examples/antlr-grammars-v4/edn/examples",
    ];
    const INVALID_FOLDERS: [&str; 1] = ["./examples/edn-tests/invalid-edn"];

    #[test]
    fn test_edn_files() {
        let a = read_files(VALID_FOLDERS.into())
            .into_iter()
            .filter_map(|x| x.err())
            .map(|x| dbg!(x.0))
            .count();
        assert_eq!(a, 0);
    }

    #[test]
    fn test_invalid_edn_files() {
        let a = read_files(INVALID_FOLDERS.into())
            .into_iter()
            .filter_map(|x| x.ok())
            .map(|x| dbg!(x))
            .count();
        assert_eq!(a, 0);
    }
}
