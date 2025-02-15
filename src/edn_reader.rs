use std::{collections::HashMap, str::Chars};

use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use num::{BigInt, BigRational, Num, ToPrimitive};
use pushback_iter::PushBackIterator;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug)]
pub enum Edn {
    Nil,
    Bool(bool),
    String(String),
    Char(char),
    Symbol(String),
    Keyword(String),
    Int(i64),
    BigInt(BigInt),
    Float(f64),
    BigDecimal(BigDecimal),
    BigRational(BigRational),
    List(Vec<Edn>),
    Vec(Vec<Edn>),
    TaggedElement(String, Box<Edn>),
}
use Edn::{Char, Float, Int, Keyword, Nil, Symbol};

type ReaderIter<'a> = PushBackIterator<Chars<'a>>;
type Reader = fn(&mut ReaderIter, char) -> Edn;

lazy_static! {
    static ref symbolPat: Regex = Regex::new("[:]?([\\D&&[^/]].*/)?(/|[\\D&&[^/]][^/]*)").unwrap();
    static ref intPat: Regex = Regex::new("^([-+]?)(?:(0)|([1-9][0-9]*)|0[xX]([0-9A-Fa-f]+)|0([0-7]+)|([1-9][0-9]?)[rR]([0-9A-Za-z]+)|0[0-9]+)(N)?$").unwrap();
    static ref ratioPat: Regex = Regex::new("^([-+]?[0-9]+)/([0-9]+)$").unwrap();
    static ref floatPat: Regex = Regex::new("^([-+]?[0-9]+(\\.[0-9]*)?([eE][-+]?[0-9]+)?)(M)?$").unwrap();

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
        map.insert('#', read_symbolic as Reader);
        map.insert('#', read_symbolic_value as Reader);
        map.insert('^', read_meta as Reader);
        map.insert('{', read_set as Reader);
        map.insert('<', read_unreadable as Reader);
        map.insert('_', read_discard as Reader);
        map.insert(':', read_namespace_map as Reader);
        map
    };
}

pub fn read(s: String) -> Option<Edn> {
    let s = s.clone();
    let mut reader = PushBackIterator::from(s.chars().into_iter());

    loop {
        // Skip whitespace
        while reader.peek().map(|&x| is_whitespace(x))? {
            let _ = reader.next()?;
        }
        let ch = reader.next()?;

        if ch.is_digit(10) {
            let n = read_number(&mut reader, ch);
            return Some(n);
        }

        if let Some(macro_) = MACROS.get(&ch) {
            let ret = macro_(&mut reader, ch);
            // if macro is noop
            continue;
            // return Some(ret);
        }

        if ch == '+' || ch == '-' {
            if reader.peek()?.is_digit(10) {
                let n = read_number(&mut reader, ch);
                return Some(n);
            }
        }

        let token = read_token(&mut reader, ch, true)?;
        return interpret_token(token);
    }
}

fn interpret_token(token: String) -> Option<Edn> {
    Some(Edn::String(token))
}

// Readers

fn read_number(reader: &mut ReaderIter, ch: char) -> Edn {
    let mut s = ch.to_string();

    loop {
        match reader.peek() {
            None => break,
            Some(&ch) if is_whitespace(ch) && is_macro(ch) => break,
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
            Some(&ch) if is_whitespace(ch) && is_terminating_macro(ch) => return Some(out),
            Some(&ch) if non_constituent(ch) => panic!("Invalid contituent character: {ch}"),
            Some(&ch) => {
                out.push(ch);
                let _ = reader.next().unwrap();
            }
        }
    }
}

// Macros
fn read_string(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}

fn read_comment(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_list(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_unmatched_delimiter(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_vector(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_map(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_character(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_dispatch(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}

// Dispatch Macros
fn read_symbolic(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_symbolic_value(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_meta(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_set(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_unreadable(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_discard(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}
fn read_namespace_map(reader: &mut ReaderIter, ch: char) -> Edn {
    Nil
}

// Matches
fn match_number(s: &str) -> Option<Edn> {
    // Integer
    let caps = intPat.captures(s);
    if let Some(caps) = caps {
        if caps.get(2).is_some() {
            if caps.get(8).is_some() {
                return Some(Edn::String("BigInt.ZERO".to_string()));
            } else {
                return Some(Edn::Int(0));
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
            return Some(Edn::Int(n));
        } else {
            return Some(Edn::BigInt(bn));
        }
    }

    let caps = floatPat.captures(s);
    if let Some(caps) = caps {
        if let Some(m) = caps.get(4) {
            let bd = BigDecimal::from_str(caps.get(1).unwrap().as_str()).unwrap();
            return Some(Edn::BigDecimal(bd));
        } else {
            return Some(Float(s.parse().unwrap()));
        }
    }

    let caps = ratioPat.captures(s);
    if let Some(caps) = caps {
        let ratio = BigRational::from_str(s).unwrap();
        return Some(Edn::BigRational(ratio));
    }
    None
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
