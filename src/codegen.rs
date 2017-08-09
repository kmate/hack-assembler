use inst::Inst;
use inst::Inst::*;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::error::Error;

type LookupTable = HashMap<&'static str, u16>;

lazy_static! {
    static ref COMP_TABLE: LookupTable = {
        let mut table = HashMap::new();
        table.insert("0",   0b0101010);
        table.insert("1",   0b0111111);
        table.insert("-1",  0b0111010);
        table.insert("D",   0b0001100);
        table.insert("A",   0b0110000);
        table.insert("!D",  0b0001101);
        table.insert("!A",  0b0110001);
        table.insert("-D",  0b0001111);
        table.insert("-A",  0b0110011);
        table.insert("D+1", 0b0011111);
        table.insert("A+1", 0b0110111);
        table.insert("D-1", 0b0001110);
        table.insert("A-1", 0b0110010);
        table.insert("D+A", 0b0000010);
        table.insert("D-A", 0b0010011);
        table.insert("A-D", 0b0000111);
        table.insert("D&A", 0b0000000);
        table.insert("D|A", 0b0010101);
        table.insert("M",   0b1110000);
        table.insert("!M",  0b1110001);
        table.insert("-M",  0b1110011);
        table.insert("M+1", 0b1110111);
        table.insert("M-1", 0b1110010);
        table.insert("D+M", 0b1000010);
        table.insert("D-M", 0b1010011);
        table.insert("M-D", 0b1000111);
        table.insert("D&M", 0b1000000);
        table.insert("D|M", 0b1010101);
        table
    };

    static ref DEST_TABLE: LookupTable = {
        let mut table = HashMap::new();
        table.insert("M",   0b001);
        table.insert("D",   0b010);
        table.insert("MD",  0b011);
        table.insert("A",   0b100);
        table.insert("AM",  0b101);
        table.insert("AD",  0b110);
        table.insert("AMD", 0b111);
        table
    };

    static ref JUMP_TABLE: LookupTable = {
        let mut table = HashMap::new();
        table.insert("JGT", 0b001);
        table.insert("JEQ", 0b010);
        table.insert("JGE", 0b011);
        table.insert("JLT", 0b100);
        table.insert("JNE", 0b101);
        table.insert("JLE", 0b110);
        table.insert("JMP", 0b111);
        table
    };
}


#[derive(Debug, PartialEq)]
pub enum MissInfo<'a> {
    Comp(&'a str),
    Dest(&'a str),
    Jump(&'a str),
}

use self::MissInfo::*;

impl<'a> Display for MissInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Comp(comp) => write!(f, "computation `{}'", comp),
            Dest(dest) => write!(f, "destination `{}'", dest),
            Jump(jump) => write!(f, "jump specifiction `{}'", jump),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CompileError<'a> {
    LookupMiss(MissInfo<'a>),
}

use self::CompileError::*;

impl<'a> Display for CompileError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LookupMiss(ref info) => write!(f, "Lookup table miss: {}", info),
        }
    }
}

impl<'a> Error for CompileError<'a> {
    fn description(&self) -> &str {
        "compilation error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

pub fn compile(inst: Inst) -> Result<u16, CompileError> {
    match inst {
        AInst { address } => Ok(address as u16 & 0x7FFFu16),
        CInst { comp, dest, jump } => {
            let c = COMP_TABLE.get(&comp).ok_or(LookupMiss(Comp(comp)))?;
            let d = match dest {
                None => 0,
                Some(dest) => *DEST_TABLE.get(dest).ok_or(LookupMiss(Dest(dest)))?,
            };
            let j = match jump {
                None => 0,
                Some(jump) => *JUMP_TABLE.get(jump).ok_or(LookupMiss(Jump(jump)))?,
            };
            Ok(0xE000u16 + (c << 6) + (d << 3) + j)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_a_inst() {
        assert_eq!(Ok(42), compile(AInst { address: 42 }));
        assert_eq!(Ok(1), compile(AInst { address: (1 << 15) + 1 }));
    }

    #[test]
    fn compile_c_inst() {
        assert_eq!(
            Ok(0b1111010101000000),
            compile(CInst {
                comp: "D|M",
                dest: None,
                jump: None,
            })
        );
        assert_eq!(
            Ok(0b1110010101101011),
            compile(CInst {
                comp: "D|A",
                dest: Some("AM"),
                jump: Some("JGE"),
            })
        );
    }

    #[test]
    fn compile_errors() {
        assert_eq!(
            Err(LookupMiss(Comp("UNKNOWN"))),
            compile(CInst {
                comp: "UNKNOWN",
                dest: None,
                jump: None,
            })
        );
        assert_eq!(
            Err(LookupMiss(Dest("UNKNOWN"))),
            compile(CInst {
                comp: "D|M",
                dest: Some("UNKNOWN"),
                jump: None,
            })
        );
        assert_eq!(
            Err(LookupMiss(Jump("UNKNOWN"))),
            compile(CInst {
                comp: "D|M",
                dest: None,
                jump: Some("UNKNOWN"),
            })
        );
    }
}
