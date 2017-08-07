use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::error::Error;

const INITIAL_ENTRIES: [(&str, i16); 22] = [
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
    ("THIS", 2),
    ("THAT", 3),
    ("SCREEN", 16384),
    ("KBD", 24576),
];

#[derive(Debug, PartialEq)]
pub struct BindError {
    symbol: String,
}

impl Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to rebind symbol {}", self.symbol)
    }
}

impl Error for BindError {
    fn description(&self) -> &str {
        self.symbol.as_str()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl BindError {
    fn new(symbol: &str) -> BindError {
        BindError { symbol: symbol.to_owned() }
    }
}

pub struct SymbolTable {
    table: HashMap<String, i16>,
}

impl SymbolTable {
    pub fn initial() -> SymbolTable {
        let mut table = SymbolTable { table: HashMap::new() };
        for entry in INITIAL_ENTRIES.iter() {
            table.bind(entry.0, entry.1).ok();
        }
        table
    }

    pub fn bind(&mut self, symbol: &str, address: i16) -> Result<(), BindError> {
        if self.contains(symbol) {
            Err(BindError::new(symbol))
        } else {
            self.table.insert(symbol.to_string(), address);
            Ok(())
        }
    }

    pub fn contains(&self, symbol: &str) -> bool {
        self.table.contains_key(symbol)
    }

    pub fn resolve(&self, symbol: &str) -> Option<i16> {
        self.table.get(symbol).map(|&x| x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_initial_symbols() {
        let table = SymbolTable::initial();
        assert!(table.contains("SP"));
        assert_eq!(Some(0), table.resolve("SP"));
    }

    #[test]
    fn does_not_contain_missing_symbol() {
        let table = SymbolTable::initial();
        assert!(!table.contains("something"));
    }

    #[test]
    fn resolves_added_symbol() {
        let mut table = SymbolTable::initial();
        assert_eq!(Ok(()), table.bind("something", 42));
        assert!(table.contains("something"));
        assert_eq!(Some(42), table.resolve("something"));
    }

    #[test]
    fn is_case_sensitive() {
        let mut table = SymbolTable::initial();
        assert_eq!(Ok(()), table.bind("lowercase", 1337));
        assert!(!table.contains("LOWERCASE"));
    }

    #[test]
    fn bind_to_existing_is_error() {
        let mut table = SymbolTable::initial();
        assert_eq!(Err(BindError::new("SP")), table.bind("SP", 42));
    }
}
