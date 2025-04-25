use std::{fmt, fs};
use std::collections::HashMap;
use crate::statement::Statement;
use crate::instructions::*;
use crate::directives::*;

pub type OpcodeAddress = u16;
pub type SymbolTable = HashMap<String, OpcodeAddress>;

const BYTES_PER_INSTRUCTION: u16 = 2;

pub fn assemble_from_file(path: &str) -> Result<Vec<u8>, Error> {
    let source = fs::read_to_string(path).map_err(|_| Error::ReadError {
        path: path.to_string(),
    })?;
    assemble(&source)
}

pub fn assemble(source: &str) -> Result<Vec<u8>, Error> {
    let (symbol_table, unresolved) = first_pass(source);
    second_pass(&symbol_table, &unresolved)
}

fn first_pass(source: &str) -> (SymbolTable, Vec<Statement>) {
    let mut labels = HashMap::new();
    let mut unresolved = Vec::new();
    let mut address: OpcodeAddress = 0;

    for (line_index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.ends_with(':') {
            let label = line.trim_end_matches(':');
            labels.insert(label.to_string(), address);
        } else {
            unresolved.push(Statement::new(
                line.split_whitespace().map(|s| s.trim_matches(',')).collect(),
                line_index + 1
            ));
            if line.starts_with(".") {  // Assembler directive
                continue;  // TODO: need to know how many bytes it takes
            } else {
                address += BYTES_PER_INSTRUCTION;
            }
        }
    }

    (labels, unresolved)
}

fn second_pass(
    symbol_table: &SymbolTable, 
    unresolved: &Vec<Statement>
) -> Result<Vec<u8>, Error> {
    let mut bytecode = Vec::new();
    for statement in unresolved {
        let bytes = parse_statement(&statement, &symbol_table)?;
        bytecode.push(bytes);
    }
    Ok(bytecode.into_iter().flatten().collect())
}

fn parse_statement(
    statement: &Statement, 
    symbol_table: &SymbolTable
) -> Result<Vec<u8>, Error> {
    let opcode = match statement.instruction().to_uppercase().as_str() {
        // INSTRUCTIONS
        "CLS"  =>  cls(statement),
        "RET"  =>  ret(statement),
        "SYS"  =>  sys(statement, symbol_table),
        "JP"   =>   jp(statement, symbol_table),
        "CALL" => call(statement, symbol_table),
        "SE"   =>   se(statement),
        "SNE"  =>  sne(statement),
        "LD"   =>   ld(statement, symbol_table),
        "ADD"  =>  add(statement),
        "OR"   =>   or(statement),
        "AND"  =>  and(statement),
        "XOR"  =>  xor(statement),
        "SUB"  =>  sub(statement),
        "SHR"  =>  shr(statement),
        "SUBN" => subn(statement),
        "SHL"  =>  shl(statement),
        "RND"  =>  rnd(statement),
        "DRW"  =>  drw(statement),
        "SKP"  =>  skp(statement),
        "SKNP" => sknp(statement),
        // ASSEMBLER DIRECTIVES
        ".BYTE" | ".DB"    =>     byte(statement),
        ".WORD" | ".DW"    =>     word(statement),
        ".TEXT" | ".ASCII" =>     text(statement),
        ".FILL"            =>     fill(statement),
        ".SPACE"           =>    space(statement),
        ".INCLUDE"         => _include(statement),
        ".WARN"            =>     warn(statement),
        ".ERROR"           =>   _error(statement),
        _ => Err(Error::UnknownInstruction {
            instruction: statement.instruction().to_string(),
            line_number: statement.line_number()
        })
    }?;
    Ok(opcode)
}

#[derive(Debug)]
pub enum Error {
    UnknownInstruction { instruction: String, line_number: usize },
    InvalidArgument { argument: String, line_number: usize },
    InvalidArgumentCount {
        instruction: String,
        line_number: usize,
        n_arguments: usize,
        expected: Vec<usize>,
    },
    UserError {
        message: String,
        line_number: usize,
    },
    ReadError { path: String },
    IncludeError { line_number: usize, error: Box<Error> }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::UnknownInstruction { instruction, line_number } => {
                format!("Unknown instruction \"{}\" at line {}", instruction, line_number)
            }
            Error::InvalidArgument { argument, line_number } => {
                format!("Invalid argument \"{}\" at line {}", argument, line_number)
            }
            Error::InvalidArgumentCount { 
                instruction, line_number, n_arguments, expected
            } => {
                format!(
                    "Invalid argument count for instruction \"{}\" at line {}: found {}, expected {:?}",
                    instruction, line_number, n_arguments, expected
                )
            }
            Error::UserError { message, line_number } => {
                format!("Error at line {}: {}", line_number, message)
            }
            Error::ReadError { path } => {
                format!("Failed to read file {}", path)
            }
            Error::IncludeError { path, line_number, error } => {
                format!("In file {} included at line {}: {}", path, line_number, error)
            }
        };
        write!(f, "{}", message)
    }
}
