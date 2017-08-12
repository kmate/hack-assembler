use std::collections::HashMap;
use std::clone::Clone;
use std::fmt;
use std::fmt::Display;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum BindError<'a> {
    AlreadyBound { symbol: &'a str },
    TooManyBindings,
}

use self::BindError::*;

impl<'a> Display for BindError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AlreadyBound { symbol } => write!(f, "Unable to rebind symbol {}", symbol),
            TooManyBindings => write!(f, "Too many bindings"),
        }
    }
}

impl<'a> Error for BindError<'a> {
    fn description(&self) -> &str {
        "bind error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

#[derive(Clone)]
pub struct SymbolTable {
    entries: HashMap<String, u16>,
    next_local: u16,
}

lazy_static! {
    static ref INITIAL_TABLE: SymbolTable = {
        let initial_entries: [(&str, u16); 23] = [
            ("R0", 0),
            ("R1", 1),
            ("R2", 2),
            ("R3", 3),
            ("R4", 4),
            ("R5", 5),
            ("R6", 6),
            ("R7", 7),
            ("R8", 8),
            ("R9", 9),
            ("R10", 10),
            ("R11", 11),
            ("R12", 12),
            ("R13", 13),
            ("R14", 14),
            ("R15", 15),
            ("SP", 0),
            ("LCL", 1),
            ("ARG", 2),
            ("THIS", 3),
            ("THAT", 4),
            ("SCREEN", 16384),
            ("KBD", 24576),
        ];

        let mut table = SymbolTable { entries: HashMap::new(), next_local: 16 };
        for entry in initial_entries.iter() {
            table.bind(entry.0, entry.1).ok();
        }
        table
    };
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        INITIAL_TABLE.clone()
    }

    pub fn bind<'a>(&mut self, symbol: &'a str, address: u16) -> Result<u16, BindError<'a>> {
        if self.contains(symbol) {
            Err(AlreadyBound { symbol: symbol })
        } else {
            self.entries.insert(symbol.to_string(), address);
            Ok(address)
        }
    }

    fn contains(&self, symbol: &str) -> bool {
        self.entries.contains_key(symbol)
    }

    pub fn resolve(&self, symbol: &str) -> Option<u16> {
        self.entries.get(symbol).map(|&x| x)
    }

    pub fn resolve_or_bind<'a>(&mut self, symbol: &'a str) -> Result<u16, BindError<'a>> {
        self.resolve(symbol).map(Ok).unwrap_or_else(|| {
            if self.next_local == <u16>::max_value() {
                return Err(TooManyBindings);
            }
            let address = self.next_local;
            self.next_local += 1;
            self.bind(symbol, address)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_initial_symbols() {
        let table = SymbolTable::new();
        assert!(table.contains("SP"));
        assert_eq!(Some(0), table.resolve("SP"));
    }

    #[test]
    fn does_not_contain_missing_symbol() {
        let table = SymbolTable::new();
        assert!(!table.contains("something"));
    }

    #[test]
    fn resolves_added_symbol() {
        let mut table = SymbolTable::new();
        assert_eq!(Ok(42), table.bind("something", 42));
        assert!(table.contains("something"));
        assert_eq!(Some(42), table.resolve("something"));
    }

    #[test]
    fn is_case_sensitive() {
        let mut table = SymbolTable::new();
        assert_eq!(Ok(1337), table.bind("lowercase", 1337));
        assert!(!table.contains("LOWERCASE"));
    }

    #[test]
    fn bind_to_existing_is_error() {
        let mut table = SymbolTable::new();
        assert_eq!(Err(AlreadyBound { symbol: "SP" }), table.bind("SP", 42));
    }

    #[test]
    fn resolve_or_bind() {
        let mut table = SymbolTable::new();
        table.bind("A", 1).ok();
        assert_eq!(Ok(1), table.resolve_or_bind("A"));
        assert_eq!(Ok(16), table.resolve_or_bind("B"));
        assert_eq!(Ok(17), table.resolve_or_bind("C"));
        for address in 18..<u16>::max_value() {
            table.resolve_or_bind(format!("X{}", address).as_str()).ok();
        }
        assert_eq!(Err(TooManyBindings), table.resolve_or_bind("Z"));
    }
}
