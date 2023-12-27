use std::{fs::File, io::Read};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "edn.pest"] // relative to src
struct EdnParser;

fn main() {
    let mut file = File::open("test/learnxiny.edn").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let a = EdnParser::parse(Rule::edn,&contents).unwrap();
    dbg!(a);
}
