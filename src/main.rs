use std::{fs::File, io::Read};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "edn.pest"] // relative to src
struct EdnParser;
// [:]?([\D&&[^/]].*/)?([\D&&[^/]][^/]*)
// static Pattern symbolPat = Pattern.compile("[:]?([\\D&&[^/]].*/)?([\\D&&[^/]][^/]*)");
// ([-+]?)(?:(0)|([1-9][0-9]*)|0[xX]([0-9A-Fa-f]+)|0([0-7]+)|([1-9][0-9]?)[rR]([0-9A-Za-z]+)|0[0-9]+)(N)?
// static Pattern intPat =
// 		Pattern.compile(
// 				"([-+]?)(?:(0)|([1-9][0-9]*)|0[xX]([0-9A-Fa-f]+)|0([0-7]+)|([1-9][0-9]?)[rR]([0-9A-Za-z]+)|0[0-9]+)(N)?");
// ([-+]?[0-9]+)/([0-9]+)
// static Pattern ratioPat = Pattern.compile("([-+]?[0-9]+)/([0-9]+)");
// ([-+]?[0-9]+(\.[0-9]*)?([eE][-+]?[0-9]+)?)(M)?
// static Pattern floatPat = Pattern.compile("([-+]?[0-9]+(\\.[0-9]*)?([eE][-+]?[0-9]+)?)(M)?");
// static final Symbol SLASH = Symbol.intern("/");

fn main() {
    let mut file = File::open("test/learnxiny.edn").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let a = EdnParser::parse(Rule::edn,&contents).unwrap();
    dbg!(a);
}
