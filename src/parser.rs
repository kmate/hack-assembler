use inst::Inst;
use inst::Inst::*;
use regex::Regex;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::iter::Iterator;
use std::num::ParseIntError;
use symtab::{BindError, SymbolTable};

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    InvalidAddress,
    BindError(BindError<'a>),
    UnknownInst(&'a str),
}

use self::ParseError::*;

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidAddress => write!(f, "unable to parse address"),
            BindError(ref error) => write!(f, "unable to bind symbol to address: {}", error),
            UnknownInst(line) => write!(f, "unknown instruction: {}", line),
        }
    }
}

impl<'a> Error for ParseError<'a> {
    fn description(&self) -> &str {
        "parse error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl<'a> From<ParseIntError> for ParseError<'a> {
    fn from(_: ParseIntError) -> Self {
        InvalidAddress
    }
}

impl<'a> From<BindError<'a>> for ParseError<'a> {
    fn from(error: BindError<'a>) -> Self {
        BindError(error)
    }
}

type CleanLines = Vec<String>;

pub fn preprocess(text: &str) -> CleanLines {
    text.lines()
        .map(|line| {
            line.replace(|c: char| c.is_whitespace(), "")
                .split("//")
                .next()
                .unwrap()
                .trim()
                .to_string()
        })
        .filter(|line| !line.is_empty())
        .collect()
}

lazy_static! {
    static ref LABEL: Regex = Regex::new(r"\(\s*(?P<label>\pL[\pL\d_\.\$]*)\s*\)").unwrap();
    static ref A_INST: Regex = Regex::new(r"^@((?P<address>\d+)|(?P<symbol>\pL[\pL\d_\.\$]*))$").unwrap();
    static ref C_INST: Regex = Regex::new(concat!(r"^((?P<dest>[AMD]{1,3})\s*=\s*)?",
                                                  r"(?P<comp>[\-\+\|&!01ADM]+)",
                                                  r"(\s*;\s*(?P<jump>[EGJLMNPQT]{3}))?$")).unwrap();
}

pub fn label_name(line: &str) -> Option<&str> {
    if let Some(parts) = LABEL.captures(line) {
        Some(parts.name("label").unwrap().as_str())
    } else {
        None
    }
}

pub fn collect_labels(lines: &CleanLines, table: &mut SymbolTable) {
    let mut label_count = 0;
    for (row, line) in lines.iter().enumerate() {
        let address = row as u16 - label_count;
        if let Some(label) = label_name(line) {
            // TODO handle bind errors
            table.bind(label, address).ok();
            label_count += 1;
        }
    }
}

pub fn parse_inst<'a, 'b>(
    line: &'a str,
    table: &'b mut SymbolTable,
) -> Result<Inst<'a>, ParseError<'a>> {
    if let Some(parts) = A_INST.captures(line) {
        let address = if let Some(symbol) = parts.name("symbol") {
            table.resolve_or_bind(symbol.as_str())?
        } else {
            parts.name("address").unwrap().as_str().parse::<u16>()?
        };
        Ok(AInst { address: address })
    } else if let Some(parts) = C_INST.captures(line) {
        (Ok(CInst {
            comp: parts.name("comp").unwrap().as_str(),
            dest: parts.name("dest").map(|x| x.as_str()),
            jump: parts.name("jump").map(|x| x.as_str()),
        }))
    } else {
        Err(UnknownInst(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespaces_trimmed() {
        assert_eq!(vec!["a", "b", "cd"], preprocess(" a\t \n\t b\r\n c d "));
    }

    #[test]
    fn comments_removed() {
        assert_eq!(vec!["b"], preprocess("// x\n\t b // y\r\n // c d"))
    }

    #[test]
    fn label_detected() {
        assert_eq!(None, label_name("not-a-label"));
        assert_eq!(Some("label"), label_name("(label)"));
    }

    #[test]
    fn labels_collected() {
        let mut table = SymbolTable::new();
        let lines = preprocess("(a)\nb\nc\n \n(d)\ne");
        collect_labels(&lines, &mut table);
        assert_eq!(Some(0), table.resolve("a"));
        assert_eq!(Some(2), table.resolve("d"));
    }

    #[test]
    fn parse_a_inst() {
        let mut table = SymbolTable::new();
        assert_eq!(Ok(AInst { address: 42 }), parse_inst("@42", &mut table));
        assert_eq!(Err(InvalidAddress), parse_inst("@70000", &mut table));
        table.bind("X", 42).ok();
        assert_eq!(Ok(AInst { address: 42 }), parse_inst("@X", &mut table));
        assert_eq!(Ok(AInst { address: 16 }), parse_inst("@Y", &mut table));
    }

    #[test]
    fn parse_c_inst() {
        let mut table = SymbolTable::new();
        assert_eq!(
            Ok(CInst {
                comp: "A",
                dest: None,
                jump: None,
            }),
            parse_inst("A", &mut table)
        );
        assert_eq!(
            Ok(CInst {
                comp: "1",
                dest: Some("M"),
                jump: None,
            }),
            parse_inst("M = 1", &mut table)
        );
        assert_eq!(
            Ok(CInst {
                comp: "D",
                dest: None,
                jump: Some("JMP"),
            }),
            parse_inst("D ; JMP", &mut table)
        );
    }

    #[test]
    fn parse_unknown_inst() {
        let mut table = SymbolTable::new();
        assert_eq!(Err(UnknownInst(";=;=")), parse_inst(";=;=", &mut table));
    }
}
