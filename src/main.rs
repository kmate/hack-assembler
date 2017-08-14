extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use clap::{Arg, App};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::process::exit;

mod codegen;
mod inst;
mod parser;
mod symtab;

fn existing_file(path: String) -> Result<(), String> {
    let info = fs::metadata(path).map_err(|e| e.description().to_string())?;
    if info.is_file() {
        Ok(())
    } else {
        Err(String::from("input file does not exist"))
    }
}

fn read_input(input_option: Option<&str>) -> io::Result<String> {
    let mut buffer = String::new();
    if let Some(path) = input_option {
        let mut file = File::open(path)?;
        file.read_to_string(&mut buffer)?;
    } else {
        io::stdin().read_to_string(&mut buffer)?;
    }
    Ok(buffer)
}

fn write_output(output_option: Option<&str>, buffer: String) -> io::Result<()> {
    if let Some(path) = output_option {
        let mut file = File::create(path)?;
        file.write_all(buffer.as_bytes())?;
    } else {
        print!("{}", buffer);
        io::stdout().flush().ok();
    }
    Ok(())
}

macro_rules! catch {
    ($x: expr, $msg: expr) => {{
        $x.unwrap_or_else(|error| {
            io::stderr().write_fmt(format_args!(concat!($msg, ": {}\n"), error)).ok();
            exit(1);
        })
    }};
}

fn main() {
    let matches = App::new("Hack Assembler")
        .version("1.0")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file to use")
                .takes_value(true)
                .validator(existing_file),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the output file to use")
                .takes_value(true),
        )
        .get_matches();

    let buffer = catch!(read_input(matches.value_of("input")), "Input error");
    let mut table = symtab::SymbolTable::new();
    let lines = parser::preprocess(&buffer);
    catch!(
        parser::collect_labels(&lines, &mut table),
        "Unable to collect labels"
    );
    let insts = lines
        .iter()
        .filter(|line| parser::label_name(line).is_none())
        .map(|line| {
            catch!(parser::parse_inst(line, &mut table), "Parse error")
        });
    let code = insts
        .map(|inst| {
            format!(
                "{:016b}",
                catch!(codegen::compile(inst), "Compilation error")
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    catch!(
        write_output(matches.value_of("output"), code),
        "Output error"
    );
}
