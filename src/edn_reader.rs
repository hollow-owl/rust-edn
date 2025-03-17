use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    f64::{INFINITY, NAN, NEG_INFINITY},
    iter::Peekable,
    str::Chars,
};

use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use num::{BigInt, BigRational, Integer, Num, ToPrimitive};
use ordered_float::OrderedFloat;
use ordermap::OrderMap;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum Edn {
    Nil,
    Bool(bool),
    String(String),
    Char(char),
    Symbol(String),
    Keyword(String),
    Int(i64),
    BigInt(BigInt),
    Float(OrderedFloat<f64>),
    BigDecimal(BigDecimal),
    BigRational(BigRational),
    List(Vec<Edn>),
    Vec(Vec<Edn>),
    Set(BTreeSet<Edn>),
    Map(BTreeMap<Edn, Edn>),
    TaggedElement(String, Box<Edn>),
}
use Edn::{Bool, Char, Float, Int, Keyword, Map, Nil, Set, Symbol, TaggedElement};

type ReaderIter<'a> = Peekable<Chars<'a>>;
type EdnRet = Option<Edn>;
type Reader = fn(&mut ReaderIter, char) -> EdnRet;

lazy_static! {
    static ref symbolPat: Regex = Regex::new("^[:]?([\\D&&[^/]].*/)?(/|[\\D&&[^/]][^/]*)").unwrap();
    // static ref symbolPat: Regex = Regex::new(r"[:]?((?:[^0-9/].*/)?(/|[^0-9/][^/]*))").unwrap();
    static ref intPat: Regex = Regex::new("^([-+]?)(?:(0)|([1-9][0-9]*)|0[xX]([0-9A-Fa-f]+)|0([0-7]+)|([1-9][0-9]?)[rR]([0-9A-Za-z]+)|0[0-9]+)(N)?").unwrap();
    static ref ratioPat: Regex = Regex::new("^([-+]?[0-9]+)/([0-9]+)").unwrap();
    static ref floatPat: Regex = Regex::new("^([-+]?[0-9]+(\\.[0-9]*)?([eE][-+]?[0-9]+)?)(M)?").unwrap();

    static ref MACROS: HashMap<char, Reader> = {
        let mut macros = HashMap::new();
        macros.insert('"', read_string as Reader);
        macros.insert(';', read_comment as Reader);
        macros.insert('^', read_meta as Reader);
        macros.insert('(', read_list as Reader);
        macros.insert(')', read_unmatched_delimiter as Reader);
        macros.insert('[', read_vector as Reader);
        macros.insert(']', read_unmatched_delimiter as Reader);
        macros.insert('{', read_map as Reader);
        macros.insert('}', read_unmatched_delimiter as Reader);
        macros.insert('\\', read_character as Reader);
        macros.insert('#', read_dispatch as Reader);
        macros
    };
    static ref DISPATCH_MACROS: HashMap<char, Reader> = {
        let mut map = HashMap::new();
        map.insert('#', read_symbolic_value as Reader);
        map.insert('^', read_meta as Reader);
        map.insert('{', read_set as Reader);
        map.insert('<', read_unreadable as Reader);
        map.insert('_', read_discard as Reader);
        map.insert(':', read_namespace_map as Reader);
        map
    };
}

pub fn read_str(s: String) -> EdnRet {
    let mut reader = s.chars().peekable();
    let out = read(&mut reader, true, Edn::Nil, false);
    out
}

pub fn read(
    reader: &mut ReaderIter,
    eof_is_error: bool,
    eof_value: Edn,
    is_recursive: bool,
) -> EdnRet {
    loop {
        // dbg!(reader.clone().collect::<String>());
        // Skip whitespace
        while reader.peek().map(|&x| is_whitespace(x))? {
            let _ = reader.next()?;
        }
        let ch = reader.next()?;

        if ch.is_digit(10) {
            let n = read_number(reader, ch);
            return Some(n);
        }

        if let Some(macro_) = MACROS.get(&ch) {
            let ret = macro_(reader, ch);
            if ret.is_none() {
                continue;
            }
            return ret;
        }

        if ch == '+' || ch == '-' {
            if reader.peek()?.is_digit(10) {
                let n = read_number(reader, ch);
                return Some(n);
            }
        }

        let token = read_token(reader, ch, true)?;
        return interpret_token(token);
    }
}

fn interpret_token(token: String) -> EdnRet {
    match token.as_str() {
        "nil" => Some(Nil),
        "true" => Some(Bool(true)),
        "false" => Some(Bool(false)),
        s => {
            let ret = match_symbol(s);
            match ret {
                Some(sym) => Some(sym),
                None => panic!("Invalid token: {s}"),
            }
        }
    }
}

fn match_symbol(s: &str) -> EdnRet {
    let caps = symbolPat.captures(s);
    if let Some(caps) = caps {
        let ns = caps.get(1);
        let name = caps.get(2).unwrap().as_str();
        if ns.is_some_and(|ns| ns.as_str().ends_with(":/"))
            || name.ends_with(":")
            || s[1..].contains("::")
        {
            return None;
        }
        if s.starts_with("::") {
            return None;
        }
        let is_keyword = s.starts_with(":");
        let sym = s[if is_keyword { 1 } else { 0 }..].to_string();
        if is_keyword {
            return Some(Keyword(sym));
        } else {
            return Some(Symbol(sym));
        }
    } else {
        return None;
    }
}

// Readers

fn read_number(reader: &mut ReaderIter, ch: char) -> Edn {
    let mut s = ch.to_string();

    loop {
        match reader.peek() {
            None => break,
            Some(&ch) if is_whitespace(ch) || is_macro(ch) => break,
            Some(&ch) => {
                s.push(ch);
                let _ = reader.next();
            }
        }
    }

    let n = match_number(&s);
    if n.is_none() {
        panic!("Invalid number: {s}");
    }
    return n.unwrap();
}

fn read_token(reader: &mut ReaderIter, ch: char, lead_constituent: bool) -> Option<String> {
    if lead_constituent && non_constituent(ch) {
        panic!("Invalid leading leading character: {ch}");
    }
    let mut out = ch.to_string();

    loop {
        let ch = reader.peek();
        match ch {
            None => return Some(out),
            Some(&ch) if is_whitespace(ch) || is_terminating_macro(ch) => return Some(out),
            Some(&ch) if non_constituent(ch) => panic!("Invalid contituent character: {ch}"),
            Some(&ch) => {
                out.push(ch);
                let _ = reader.next().unwrap();
            }
        }
    }
}

// Macros
fn read_string(reader: &mut ReaderIter, double_quote: char) -> EdnRet {
    if double_quote != '"' {
        unreachable!("Started reading string with {double_quote} but it should always be a \"");
    }

    let mut out = String::new();
    loop {
        let ch = match reader.next().expect("EOF while reading string") {
            '"' => break,
            '\\' => {
                // escape
                match reader.next().expect("EOF while reading string") {
                    't' => '\t',
                    'r' => '\r',
                    'n' => '\n',
                    '\\' => '\\',
                    '"' => '"',
                    'b' => '\u{08}',
                    'f' => '\u{0C}',
                    'u' => {
                        let ch = reader.next().unwrap();
                        if !ch.is_digit(16) {
                            panic!("Unvalid unicode escape: \\u{ch}");
                        }
                        read_unicode_char(reader, ch, 16, 4, true).unwrap()
                    }
                    ch => {
                        if ch.is_digit(10) {
                            let ch = read_unicode_char(reader, ch, 8, 3, false).unwrap();
                            if (ch as u32) > 0o377 {
                                panic!("Octal escape sequence must be in range [0, 377].")
                            }
                            ch
                        } else {
                            panic!("Unsupported escape character: \\{ch}");
                        }
                    }
                }
            }
            ch => ch,
        };
        out.push(ch);
    }
    return Some(Edn::String(out));
}

fn read_unicode_char(
    reader: &mut ReaderIter,
    ch: char,
    base: u32,
    length: i32,
    exact: bool,
) -> Option<char> {
    let mut uc = {
        let uc = ch.to_digit(base);
        if let None = uc {
            panic!("Invalid digit: {ch}");
        }
        uc.unwrap()
    };
    let mut i = 0;
    for curr in 0..length {
        i = curr;
        let ch = reader.peek();
        match ch {
            None => break,
            Some(&ch) if is_whitespace(ch) || is_macro(ch) => break,
            Some(&ch) => {
                let _ = reader.next();
                let d = ch.to_digit(base);
                match d {
                    None => panic!("Invalid digit: {ch}"),
                    Some(d) => uc = uc * base + d,
                }
            }
        }
    }
    if i != length && exact {
        panic!("Invalid character length: {i}, should be: {length}");
    }
    return char::from_u32(uc);
}

fn read_comment(reader: &mut ReaderIter, semicolon: char) -> EdnRet {
    assert_eq!(semicolon, ';');
    loop {
        match reader.next() {
            None => break,
            Some(ch) if ch == '\n' || ch == '\r' => break,
            _ => continue,
        }
    }
    None
}
fn read_list(reader: &mut ReaderIter, ch: char) -> EdnRet {
    let list = read_delimited_list(')', reader, true);
    return Some(Edn::List(list));
}

fn read_unmatched_delimiter(reader: &mut ReaderIter, ch: char) -> EdnRet {
    panic!("Unmatched Delimiter: {ch}");
}
fn read_vector(reader: &mut ReaderIter, ch: char) -> EdnRet {
    let vec = read_delimited_list(']', reader, true);
    return Some(Edn::Vec(vec));
}
fn read_map(reader: &mut ReaderIter, ch: char) -> EdnRet {
    assert_eq!(ch, '{');
    let vec = read_delimited_list('}', reader, true);
    if vec.len().is_odd() {
        panic!("Map literal must contain an even number of forms");
    } else {
        let map = vec
            .chunks_exact(2)
            .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
            .collect();
        Some(Map(map))
    }
}

fn read_character(reader: &mut ReaderIter, backslash: char) -> EdnRet {
    assert_eq!(backslash, '\\');
    let token = reader.next().and_then(|x| read_token(reader, x, false));
    let c = match token.as_deref() {
        Some(t) if t.len() == 1 => t.chars().next().unwrap(),
        Some("newline") => '\n',
        Some("tab") => '\t',
        Some("backspace") => '\u{08}',
        Some("formfeed") => '\u{0C}',
        Some("return") => '\r',
        Some(t) if t.starts_with("u") => {
            let c = read_unicode_char_from_token(t, 1, 4, 16);
            match c {
                None => panic!("Invalid character constant: \\{t}"),
                Some(c) if (c as u32) >= 0xD800 && (c as u32) <= 0xDFFF => {
                    panic!("Invalid character constant: \\u{c}")
                }
                Some(c) => c,
            }
        }
        Some(t) if t.starts_with("o") => {
            todo!()
        }
        Some(t) => panic!("Unsupported character: \\{t}"),
        None => panic!("Unsupported character: \\ TODO"),
    };
    Some(Char(c))
}

fn read_dispatch(reader: &mut ReaderIter, hash: char) -> EdnRet {
    assert_eq!(hash, '#');
    let ch = *reader.peek().expect("EOF while reading character");
    if let Some(macro_) = DISPATCH_MACROS.get(&ch) {
        let ch = reader.next().unwrap();
        return macro_(reader, ch);
    } else if ch.is_alphabetic() {
        return read_tagged(reader, ch);
    } else {
        panic!("No dispatch macro for: {ch}")
    }
}

fn read_tagged(reader: &mut ReaderIter, ch: char) -> EdnRet {
    let name = read(reader, true, Nil, false)?;
    if let Symbol(name) = name {
        let o = read(reader, true, Nil, true)?;
        if !vec!["uuid", "inst"].contains(&name.as_str()) {
            panic!("No reader function for tag {name}");
        }
        Some(TaggedElement(name, Box::new(o)))
    } else {
        panic!("Reader tag must be a symbol");
    }
}

// Dispatch Macros
fn read_symbolic_value(reader: &mut ReaderIter, quote: char) -> EdnRet {
    assert_eq!(quote, '#');
    let edn = read(reader, true, Nil, true)?;
    let out = match edn {
        Symbol(s) => match s.as_ref() {
            "Inf" => Edn::Float(INFINITY.into()),
            "-Inf" => Edn::Float(NEG_INFINITY.into()),
            "NaN" => Edn::Float(NAN.into()),
            _ => panic!("Unkown symbolic value: ##{s}"),
        },
        _ => panic!("Invalid token: ##{edn:?}"),
    };
    Some(out)
}

fn read_meta(reader: &mut ReaderIter, carrot: char) -> EdnRet {
    assert_eq!(carrot, '^');
    // TODO: Implement metadata
    let meta = read(reader, true, Nil, true)?;
    match meta {
        Symbol(_) | Edn::String(_) | Keyword(_) | Map(_) => {}
        _ => panic!("Metadata must be Symbol,Keyword,String or Map"),
    }
    read(reader, true, Nil, true)
}

fn read_set(reader: &mut ReaderIter, ch: char) -> EdnRet {
    assert_eq!(ch, '{');
    let vec = read_delimited_list('}', reader, true);
    Some(Set(vec.into_iter().collect()))
}

fn read_unreadable(reader: &mut ReaderIter, ch: char) -> EdnRet {
    panic!("Unreadable form");
}

fn read_discard(reader: &mut ReaderIter, ch: char) -> EdnRet {
    assert_eq!(ch, '_');
    read(reader, true, Nil, true);
    None
}

fn read_namespace_map(reader: &mut ReaderIter, ch: char) -> EdnRet {
    assert_eq!(ch, ':');
    let sym = read(reader, true, Nil, false);
    let namespace = {
        if let Some(Symbol(sym)) = sym {
            let (ns, name) = sym_split(&sym)?;
            if !ns.is_none() {
                panic!("Namespaced map must specify a valid namespace: {sym}");
            }
            Some(name.to_string())
        } else {
            None
        }
    };
    // Skip whitespace
    while reader.peek().is_some_and(|&x| is_whitespace(x)) {
        let _ = reader.next();
    }
    let ch = reader.next().expect("EOF while reading");
    if ch != '{' {
        panic!("Namespaced map must specify a map");
    }
    let vec = read_delimited_list('}', reader, true);
    if vec.len().is_odd() {
        panic!("Map literal must contain an even number of forms");
    } else {
        let map = vec
            .chunks_exact(2)
            .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
            .map(|chunk| {
                if let Some(namespace) = namespace.as_deref() {
                    let new_key = match chunk.0 {
                        Keyword(kw) => {
                            let (ns, name) = sym_split(&kw).unwrap();
                            match ns {
                                Some(ns) if ns == "_/" => Keyword(format!("{name}")),
                                None => Keyword(format!("{namespace}/{name}")),
                                _ => Keyword(kw),
                            }
                        }
                        Symbol(sym) => {
                            let (ns, name) = sym_split(&sym).unwrap();
                            match ns {
                                Some(ns) if ns == "_/" => Symbol(format!("{name}")),
                                None => Symbol(format!("{namespace}/{name}")),
                                _ => Symbol(sym),
                            }
                        }
                        _ => chunk.0,
                    };
                    (new_key, chunk.1)
                } else {
                    chunk
                }
            })
            .collect();
        // TODO: Construct output map
        Some(Map(map))
    }
}

// Matches
fn match_number(s: &str) -> EdnRet {
    // Integer
    let caps = intPat.captures(s);
    if let Some(caps) = caps {
        if caps.get(2).is_some() {
            if caps.get(8).is_some() {
                return Some(Edn::String("BigInt.ZERO".to_string()));
            } else {
                return Some(Int(0));
            }
        }
        let negate = caps.get(1).unwrap().as_str() == "-";
        let mut n = String::new();
        let mut radix = 10;
        if let Some(m) = caps.get(3) {
            n = m.as_str().into();
            radix = 10;
        }
        if let Some(m) = caps.get(4) {
            n = m.as_str().into();
            radix = 16;
        }
        if let Some(m) = caps.get(5) {
            n = m.as_str().into();
            radix = 8;
        }
        if let Some(m) = caps.get(7) {
            n = m.as_str().into();
            radix = caps.get(6).map(|x| x.as_str().parse()).unwrap().unwrap();
        }

        if n == String::new() {
            return None;
        }
        let mut bn = BigInt::from_str_radix(&n, radix).unwrap();
        if negate {
            bn *= -1;
        }
        if caps.get(8).is_some() {
            return Some(Edn::BigInt(bn));
        }
        if let Some(n) = bn.to_i64() {
            return Some(Int(n));
        } else {
            return Some(Edn::BigInt(bn));
        }
    }

    let caps = floatPat.captures(s);
    if let Some(caps) = caps {
        if caps.get(4).is_some() {
            let bd = BigDecimal::from_str(caps.get(1).unwrap().as_str()).unwrap();
            return Some(Edn::BigDecimal(bd));
        } else {
            return Some(Float(s.parse().unwrap()));
        }
    }

    let caps = ratioPat.captures(s);
    if caps.is_some() {
        let ratio = BigRational::from_str(s).unwrap();
        return Some(Edn::BigRational(ratio));
    }
    None
}

fn read_delimited_list(delim: char, reader: &mut ReaderIter, is_recursive: bool) -> Vec<Edn> {
    let mut list = Vec::new();
    loop {
        // Skip whitespace
        while reader.peek().is_some_and(|&x| is_whitespace(x)) {
            let _ = reader.next();
        }
        match reader.peek() {
            None => panic!("EOF while reading"),
            Some(&ch) if ch == delim => {
                let _ = reader.next();
                break;
            }
            Some(&ch) => {
                if let Some(macro_) = MACROS.get(&ch) {
                    let _ = reader.next();
                    let ret = macro_(reader, ch);
                    if let Some(ret) = ret {
                        list.push(ret);
                    }
                } else if let Some(o) = read(reader, true, Nil, is_recursive) {
                    list.push(o)
                }
            }
        }
    }
    list
}

fn read_unicode_char_from_token(
    token: &str,
    offset: usize,
    length: usize,
    base: u32,
) -> Option<char> {
    if token.len() != offset + length {
        panic!("Invalid unicode character: \\{token}");
    }

    let mut uc = 0;
    for d in token.chars().skip(offset) {
        match d.to_digit(base) {
            None => panic!("Invalid digit: {d}"),
            Some(d) => {
                uc = uc * base + d;
            }
        }
    }

    char::from_u32(uc)
}
// Utils
fn non_constituent(ch: char) -> bool {
    ch == '@' || ch == '`' || ch == '~'
}

fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() || ch == ','
}

fn is_terminating_macro(ch: char) -> bool {
    ch != '#' && ch != '\'' && is_macro(ch)
}

fn is_macro(ch: char) -> bool {
    MACROS.get(&ch).is_some()
}

impl fmt::Display for Edn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nil => write!(f, "nil"),
            Bool(b) => write!(f, "{b}"),
            Edn::String(s) => write!(
                f,
                "\"{}\"",
                s.replace("\r\n", "\n")
                    .chars()
                    .flat_map(|c| c.escape_default())
                    .collect::<String>()
                    .replace("\\'", "'")
            ),
            Char(c) => write!(f, "\\{c}"),
            Symbol(s) => write!(f, "{s}"),
            Keyword(k) => write!(f, ":{k}"),
            Int(n) => write!(f, "{n}"),
            Edn::BigInt(n) => write!(f, "{n}N"),
            Float(n) => write!(f, "{n}"),
            Edn::BigDecimal(n) => write!(f, "{n}M"),
            Edn::BigRational(n) => write!(f, "{n}R"),
            Edn::List(vec) => write_delimited_list(f, "(", vec, ")"),
            Edn::Vec(vec) => write_delimited_list(f, "[", vec, "]"),
            Set(btree_set) => write_delimited_list(f, "#{", btree_set, "}"),
            Map(map) => {
                write!(f, "{{")?;
                let fmt_map = map
                    .iter()
                    .map(|(k, v)| format!("{k} {v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{fmt_map}")?;
                write!(f, "}}")
            }
            TaggedElement(tag, edn) => write!(f, "#{tag} {edn}"),
        }
    }
}

fn write_delimited_list<'a, I: IntoIterator<Item = &'a Edn>>(
    f: &mut std::fmt::Formatter<'_>,
    start: &str,
    iter: I,
    end: &str,
) -> Result<(), std::fmt::Error> {
    write!(f, "{start}")?;
    for (i, item) in iter.into_iter().enumerate() {
        if i > 0 {
            write!(f, " ")?;
        }
        write!(f, "{item}")?;
    }
    write!(f, "{end}")
}

fn sym_split(sym: &str) -> Option<(Option<&str>, &str)> {
    let caps = symbolPat.captures(sym)?;
    Some((
        caps.get(1).map(|x| x.as_str()),
        caps.get(2).unwrap().as_str(),
    ))
}
