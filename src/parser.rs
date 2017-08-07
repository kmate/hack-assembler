use std::iter::Iterator;
use super::symtab::SymbolTable;

pub struct Parser;

impl Parser {
    fn new() -> Parser {
        Parser
    }

    fn preprocess(&self, text: &str) -> Vec<String> {
        text.lines()
            .map(|line| {
                line.replace(|c: char| c.is_whitespace(), "")
                    .split("//")
                    .next()
                    .unwrap()
                    .to_string()
            })
            .filter(|line| !line.is_empty())
            .collect()
    }

    fn label_name<'a>(&self, line: &'a str) -> Option<&'a str> {
        if line.starts_with('(') && line.ends_with(')') {
            Some(line.trim_matches(|c| '(' == c || ')' == c))
        } else {
            None
        }
    }

    fn collect_labels(&self, text: &str, table: &mut SymbolTable) {
        let lines = self.preprocess(text);
        for (address, line) in lines.iter().enumerate() {
            self.label_name(line).map(|label| {
                table.bind(label, address as i16)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespaces_trimmed() {
        let parser = Parser::new();
        assert_eq!(
            vec!["a", "b", "cd"],
            parser.preprocess(" a\t \n\t b\r\n c d")
        );
    }

    #[test]
    fn comments_removed() {
        let parser = Parser::new();
        assert_eq!(vec!["b"], parser.preprocess("// x\n\t b // y\r\n // c d"))
    }

    #[test]
    fn label_detected() {
        let parser = Parser::new();
        assert_eq!(None, parser.label_name("not-a-label"));
        assert_eq!(Some("label"), parser.label_name("(label)"));
    }

    #[test]
    fn labels_collected() {
        let parser = Parser::new();
        let mut table = SymbolTable::initial();
        parser.collect_labels("(a)\nb\n\n(c)\nd", &mut table);
        assert_eq!(Some(0), table.resolve("a"));
        assert_eq!(Some(2), table.resolve("c"));
    }
}
