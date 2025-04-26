mod instructions;
mod directives;

use std::{env, error, fmt, fs};
use std::collections::HashMap;
use instructions::*;
use directives::*;

const BYTES_PER_INSTRUCTION: u16 = 2;

#[derive(Debug, Clone)]
struct Statement<'a> {
    lexemes: Vec<&'a str>,
    line_number: usize,
}

#[derive(Debug)]
enum AssembleError {
    UnknownInstruction { instruction: String, line_number: usize },
    Unimplemented  // TODO: get rid of this
}

impl error::Error for AssembleError {}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            AssembleError::UnknownInstruction { instruction, line_number } => {
                format!("Unknown opcode {} at line {}", instruction, line_number)
            }
            AssembleError::Unimplemented => "Unimplemented opcode".to_string(),
        };
        write!(f, "{}", message)
    }
}

type OpcodeAddress = u16;  // Do not need more because of limited ROM size
type Opcode = u16;
type SymbolTable = HashMap<String, OpcodeAddress>;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.asm> <output.ch8>", args[0]);
        return;
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let source = fs::read_to_string(input_path).expect("Failed to read input file");
    let opcodes = assemble(&source).expect("Failed to assemble");

    let bytecode: Vec<u8> = opcodes.iter()
        .flat_map(|word| word.to_be_bytes())
        .collect();
    fs::write(output_path, bytecode).expect("Failed to write output file");
}

fn assemble(source: &str) -> Result<Vec<Opcode>, AssembleError> {
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
            unresolved.push(Statement {
                lexemes: line.split_whitespace().map(|s| s.trim_matches(',')).collect(),
                line_number: line_index + 1
            });
            address += BYTES_PER_INSTRUCTION;
        }
    }

    (labels, unresolved)
}

fn second_pass(
    symbol_table: &SymbolTable, 
    unresolved: &Vec<Statement>
) -> Result<Vec<Opcode>, AssembleError> {
    let mut opcodes = Vec::new();
    for statement in unresolved {
        let opcode = parse_statement(&statement, &symbol_table)?;
        opcodes.push(opcode);
    }
    Ok(opcodes)
}

fn parse_statement(
    statement: &Statement, 
    symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    let opcode = match statement.lexemes[0].to_uppercase().as_str() {
        // INSTRUCTIONS
        "CLS"  =>  cls(statement, symbol_table),
        "RET"  =>  ret(statement, symbol_table),
        "SYS"  =>  sys(statement, symbol_table),
        "JP"   =>   jp(statement, symbol_table),
        "CALL" => call(statement, symbol_table),
        "SE"   =>   se(statement, symbol_table),
        "SNE"  =>  sne(statement, symbol_table),
        "LD"   =>   ld(statement, symbol_table),
        "ADD"  =>  add(statement, symbol_table),
        "OR"   =>   or(statement, symbol_table),
        "AND"  =>  and(statement, symbol_table),
        "XOR"  =>  xor(statement, symbol_table),
        "SUB"  =>  sub(statement, symbol_table),
        "SHR"  =>  shr(statement, symbol_table),
        "SUBN" => subn(statement, symbol_table),
        "SHL"  =>  shl(statement, symbol_table),
        "RND"  =>  rnd(statement, symbol_table),
        "DRW"  =>  drw(statement, symbol_table),
        "SKP"  =>  skp(statement, symbol_table),
        "SKNP" => sknp(statement, symbol_table),
        // ASSEMBLER DIRECTIVES
        ".BYTE" | ".DB"    =>     byte(statement, symbol_table),
        ".WORD" | ".DW"    =>     word(statement, symbol_table),
        ".TEXT" | ".ASCII" =>     text(statement, symbol_table),
        ".FILL"            =>     fill(statement, symbol_table),
        ".SPACE"           =>    space(statement, symbol_table),
        ".INCLUDE"         => _include(statement, symbol_table),
        ".MACRO"           =>   _macro(statement, symbol_table),
        ".ENDMACRO"        => endmacro(statement, symbol_table),
        ".DEFINE"          =>   define(statement, symbol_table),
        ".IF"              =>      _if(statement, symbol_table),
        ".ELSE"            =>    _else(statement, symbol_table),
        ".ENDIF"           =>    endif(statement, symbol_table),
        ".WARN"            =>     warn(statement, symbol_table),
        ".ERROR"           =>   _error(statement, symbol_table),
        _ => Err(AssembleError::UnknownInstruction {
            instruction: statement.lexemes[0].to_string(),
            line_number: statement.line_number
        })
    }?;

    Ok(opcode)
}
