mod edn_reader;
use std::{
    fs,
    io::{self, Read, Write},
    process::{Command, Stdio},
};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Nil,
    Bool(bool),
    String(String),
    Char(char),
    Symbol(String),
    Keyword(String),
    Int(usize),
    Float(f64),
    List(Vec<Value>),
    Vec(Vec<Value>),
    // Map(HashMap<String,Value>), // TODO: arbitrary Values as keys
    // Set(HashSet<Value>), // TODO: arbitrary Sets
    TaggedElement(String, Box<Value>), // #sym val
}

fn pair_to_value(pair: Pair<Rule>) -> Value {
    dbg!(&pair);
    match pair.as_rule() {
        Rule::edn => pair_to_value(pair.into_inner().next().unwrap()),
        Rule::symbol => {
            let sym = pair.as_str();
            match sym {
                "nil" => Value::Nil,
                "false" => Value::Bool(false),
                "true" => Value::Bool(true),
                _ => Value::Symbol(sym.to_owned()),
            }
        }
        Rule::keyword => {
            let ksym = pair.as_str();
            Value::Keyword(ksym.to_owned())
        }
        Rule::string => {
            let mut pair = pair.into_inner();
            let inner = pair.next().unwrap().as_str();
            Value::String(inner.to_owned())
        }
        Rule::character => {
            dbg!(pair);
            unimplemented!()
        }
        Rule::list => Value::List(pair.into_inner().map(pair_to_value).collect()),
        Rule::vector => Value::Vec(pair.into_inner().map(pair_to_value).collect()),
        // Rule::set => Value::Map(pair.into_inner().map(pair_to_value).collect()),
        // Rule::set => Value::Set(pair.into_inner().map(pair_to_value).collect()),
        Rule::tagged_element => {
            let mut pairs = pair.into_inner();
            let tag = pairs.next().unwrap().as_str().to_owned();
            let expr = pair_to_value(pairs.next().unwrap());
            Value::TaggedElement(tag, Box::new(expr))
        }
        _ => unimplemented!(),
    }
}

fn pest_edn() {
    let file = fs::read_to_string("test/learnxiny.edn").expect("could not read file");
    let file = "#_ (a b c)";
    let parsed = EdnParser::parse(Rule::edn, &file).unwrap();
    dbg!(&parsed);
    for edn in parsed {
        dbg!(pair_to_value(edn));
    }
}

fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        if is_match(&input) {
            println!("Match");
        } else {
            println!("ERROR: {input} does not match")
        }
    }
}

fn test_file(path: &str) {
    let file = fs::read_to_string(path).expect("could not read file");
    if is_match(file.trim()) {
        println!("Match");
    } else {
        println!("ERROR: File {path} does not match")
    }
}

fn is_match(input: &str) -> bool {
    let input = input.trim().to_owned();
    let clojure_edn = clojure_edn(input.as_str());
    let edn = edn_reader::read_str(input);
    dbg!(&edn);
    matches!((clojure_edn, edn), (Some(_), Some(_)) | (None, None))
}

fn clojure_edn(input: &str) -> Option<String> {
    let mut child = Command::new("clojure")
        .arg("-M")
        .arg("-e")
        .arg("(clojure.edn/read *in*)")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Clojure Process");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout
            .read_to_string(&mut output)
            .expect("Failed to read stdout");
    }

    let mut err = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr
            .read_to_string(&mut err)
            .expect("Failed to read stderr");
    }
    err = err
        .strip_prefix(
            "Picked up _JAVA_OPTIONS: -Djava.util.prefs.userRoot=/home/user/.config/java\n",
        )
        .expect("Missing java stuff")
        .to_string();
    // dbg!((&input, &output, &err));
    if !err.is_empty() {
        dbg!(err);
        // todo!("Error messages");
        None
    } else {
        dbg!(&output);
        Some(output)
    }
}

fn main() {
    repl();
    // test_file("test/learnxiny.edn");
}

#[cfg(test)]
mod tests {

    use pest::error::Error;
    use std::{fs, path::PathBuf};
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
        let files = folders.iter().flat_map(WalkDir::new).flatten();

        let mut results = Vec::new();
        for entry in files {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |e| e == "edn")
            {
                let file = fs::read_to_string(entry.path()).unwrap();
                let parse_result = EdnParser::parse(Rule::edn, &file).map(|_| ());
                results.push((entry.path().to_owned(), parse_result));
            }
        }
        results
    }

    fn all_ok(results: impl Iterator<Item = (PathBuf, Result<(), Error<Rule>>)>) {
        let mut has_err = false;
        for (path, err) in results {
            if err.is_err() {
                dbg!(path);
                let err = err.unwrap_err();
                println!("{}", err);
                has_err = true;
            }
        }
        assert!(!has_err)
    }

    fn all_err(results: impl Iterator<Item = (PathBuf, Result<(), Error<Rule>>)>) {
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
