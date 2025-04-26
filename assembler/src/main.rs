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

type Number = u16;
type Register = u16;

impl Statement<'_> {
    fn n_arguments(&self) -> usize {
        self.lexemes.len() - 1
    }

    fn argument(&self, argument_index: usize) -> &str {
        self.lexemes[argument_index + 1]
    }

    fn instruction(&self) -> &str {
        self.lexemes[0]
    }

    // TODO: where I am using this function, there is no overflow check
    fn parse_number(&self, argument_index: usize) -> Result<Number, AssembleError> {
        let lexeme = self.argument(argument_index);
        if lexeme.starts_with("0x") {
            u16::from_str_radix(&lexeme[2..], 16)
        } else if lexeme.starts_with("0b") {
            u16::from_str_radix(&lexeme[2..], 2)
        } else {
            lexeme.parse::<Number>()
        }.map_err(|_| self.invalid_argument(argument_index))
    }

    fn parse_register(&self, argument_index: usize) -> Result<Register, AssembleError> {
        let lexeme = self.argument(argument_index);
        let error = self.invalid_argument(argument_index);
        if lexeme.len() == 2 && lexeme.starts_with('V') {
            let register_char = lexeme.chars().nth(1).unwrap();
            let register = register_char.to_digit(16)
                .ok_or_else(|| error)? as u16;
            Ok(register)
        } else {
            Err(error)
        }
    }

    fn parse_only_two_registers(&self) -> Result<(Register, Register), AssembleError> {
        self.assert_n_arguments(2)?;
        let x = self.parse_register(0)?;
        let y = self.parse_register(1)?;
        Ok((x, y))
    }

    fn assert_n_arguments(&self, n: usize) -> Result<(), AssembleError> {
        let n_arguments = self.n_arguments();
        if n_arguments != n {
            return Err(self.invalid_argument_count(n_arguments, vec![n]));
        }
        Ok(())
    }

    fn invalid_argument(&self, argument_index: usize) -> AssembleError {
        AssembleError::InvalidArgument {
            argument: self.lexemes[argument_index + 1].to_string(),
            line_number: self.line_number
        }
    }

    fn invalid_argument_count(
        &self, 
        n_arguments: usize, 
        expected: Vec<usize>  // e.g. 1 OR 2 arguments are expected
    ) -> AssembleError {
        AssembleError::InvalidArgumentCount {
            instruction: self.instruction().to_string(),
            line_number: self.line_number,
            n_arguments,
            expected
        }
    }
}

#[derive(Debug)]
enum AssembleError {
    UnknownInstruction { instruction: String, line_number: usize },
    InvalidArgument { argument: String, line_number: usize },
    InvalidArgumentCount {
        instruction: String,
        line_number: usize,
        n_arguments: usize,
        expected: Vec<usize>,
    },
    Unimplemented  // TODO: get rid of this
}

impl error::Error for AssembleError {}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            AssembleError::UnknownInstruction { instruction, line_number } => {
                format!("Unknown instruction {} at line {}", instruction, line_number)
            }
            AssembleError::InvalidArgument { argument, line_number } => {
                format!("Invalid argument {} at line {}", argument, line_number)
            }
            AssembleError::InvalidArgumentCount { 
                instruction, line_number, n_arguments, expected 
            } => {
                format!(
                    "Invalid argument count for opcode {} at line {}: found {}, expected {:?}",
                    instruction, line_number, n_arguments, expected
                )
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
