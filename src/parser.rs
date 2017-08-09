use inst::Inst;
use inst::Inst::*;
use regex::Regex;
use std::iter::Iterator;
use symtab::SymbolTable;

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    UndefinedSymbol(String),
    UnknownInst(&'a str),
}

use self::ParseError::*;

// TODO define error impl for ParseError

fn preprocess(text: &str) -> Vec<String> {
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

fn label_name(line: &str) -> Option<&str> {
    if line.starts_with('(') && line.ends_with(')') {
        Some(line.trim_matches(|c| '(' == c || ')' == c))
    } else {
        None
    }
}

pub fn collect_labels(text: &str, table: &mut SymbolTable) {
    let lines = preprocess(text);
    for (address, line) in lines.iter().enumerate() {
        label_name(line).map(|label| table.bind(label, address as u16));
    }
}

lazy_static! {
    static ref A_INST: Regex = Regex::new(r"^@(?P<symbol>\pL+)$").unwrap();
    static ref C_INST: Regex = Regex::new(concat!(r"^((?P<dest>[AMD]{1,3})\s*=\s*)?",
                                                  r"(?P<comp>[\-\+\|&!01ADM]+)",
                                                  r"(\s*;\s*(?P<jump>[EGJLMNPQT]{3}))?$")).unwrap();
}

fn parse_inst<'a, 'b>(line: &'a str, table: &'b SymbolTable) -> Result<Inst<'a>, ParseError<'a>> {
    if let Some(parts) = A_INST.captures(line) {
        let symbol = &parts["symbol"];
        let address = table.resolve(symbol).ok_or(
            UndefinedSymbol(symbol.to_string()),
        )?;
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
        collect_labels("(a)\nb\n \n(c)\nd", &mut table);
        assert_eq!(Some(0), table.resolve("a"));
        assert_eq!(Some(2), table.resolve("c"));
    }

    #[test]
    fn parse_a_inst() {
        let mut table = SymbolTable::new();
        assert_eq!(
            Err(UndefinedSymbol("X".to_string())),
            parse_inst("@X", &table)
        );
        table.bind("X", 42).ok();
        assert_eq!(Ok(AInst { address: 42 }), parse_inst("@X", &table));
    }

    #[test]
    fn parse_c_inst() {
        let table = SymbolTable::new();
        assert_eq!(
            Ok(CInst {
                comp: "A",
                dest: None,
                jump: None,
            }),
            parse_inst("A", &table)
        );
        assert_eq!(
            Ok(CInst {
                comp: "1",
                dest: Some("M"),
                jump: None,
            }),
            parse_inst("M = 1", &table)
        );
        assert_eq!(
            Ok(CInst {
                comp: "D",
                dest: None,
                jump: Some("JMP"),
            }),
            parse_inst("D ; JMP", &table)
        );
    }

    #[test]
    fn parse_unknown_inst() {
        let table = SymbolTable::new();
        assert_eq!(Err(UnknownInst(";=;=")), parse_inst(";=;=", &table));
    }
}
