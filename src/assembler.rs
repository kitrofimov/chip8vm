//! Converts CHIP-8 assembly code into bytecode in a two-pass manner
//! 
//! The first pass resolves labels and generates a symbol table, while
//! the second pass generates the bytecode.
//! 
//! The syntax for the assembler is taken from this specification:
//! <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM>
//! 
//! Comments start with a semicolon (`;`) and continue to the end of the line,
//! may start both at the beginning or at the end of a line. Comments are fully
//! ignored by the assembler.
//! 
//! Labels, instructions and directives can be indented as you wish. Labels
//! are case-sensitive, while instructions and directives are not. Labels are
//! defined with a colon at the end of the line, for example:
//! 
//! ```ignore
//! label:
//!     LD V0, 0  ; set V0 to 0
//!     loop:
//!         ADD V0, 1  ; increment V0 by 1
//!         SE V0, 10  ; if V0 is equal to 10, jump to the label "done"
//!         JP loop  ; jump to the label "loop"
//! done:
//!     CLS
//! ```
//! 
//! The assembler supports the following instructions:
//! - CLS: clear the display
//! - RET: return from a subroutine
//! - SYS: jump to a machine code routine (useless)
//! - JP: jump to an address or label
//! - CALL addr: call a subroutine (address or label)
//! - SE: skip next instruction if VX equals byte or register (skip equal)
//!     - SE Vx, Vy
//!     - SE Vx, byte
//! - SNE: skip next instruction if VX does not equal byte or register (skip not equal) 
//!     - SNE Vx, Vy
//!     - SNE Vx, byte
//! - LD: load a value into a register:
//!     - LD Vx, byte: load the byte into register Vx
//!     - LD Vx, Vy: load the value of register Vy into register Vx
//!     - LD I, addr: load the address `addr` or a label into register I
//!     - LD Vx, DT: load the value of delay timer into register Vx
//!     - LD Vx, K: load the value of the key pressed into register Vx
//!     - LD DT, Vx: load the value of register Vx into the delay timer
//!     - LD ST, Vx: load the value of register Vx into the sound timer
//!     - LD F, Vx: load the location of the sprite for digit Vx into register F
//!     - LD B, Vx: store the value of register Vx in BCD format in memory locations I, I+1, and I+2
//!     - LD \[I\], Vx: store the values of registers V0..Vx in memory locations I, I+1, ...
//!     - LD Vx, \[I\]: load the values of memory locations I, I+1, ... into registers V0..Vx
//! - ADD: add a value to a register
//!     - ADD Vx, Vy
//!     - ADD Vx, byte
//! - OR Vx, Vy: bitwise OR a value with a register
//! - AND Vx, Vy: bitwise AND a value with a register
//! - XOR Vx, Vy: bitwise XOR a value with a register
//! - SUB Vx, Vy: subtract a value from a register
//! - SUBN Vx, Vy: subtract the value of a register from another
//! - SHR Vx, Vy: shift a register right
//! - SHL Vx, Vy: shift a register left
//! - RND Vx, byte: generate a random number, AND it with the byte, and store it in a register
//! - DRW Vx, Vy, nibble: draw a sprite with a height of nibble pixels at a position (Vx, Vy)
//! - SKP Vx: skip next instruction if key with the value of VX is pressed
//! - SKNP Vx: skip next instruction if key with the value of VX is not pressed
//! 
//! The assembler supports the following directives:
//! - .BYTE byte, .DB byte: store a single byte
//! - .WORD word, .DW word: store a 16-bit word (2 bytes)
//! - .TEXT string, .ASCII string: store a string
//! - .FILL n, byte, .DB byte: fill the memory with a value
//! - .SPACE n: reserve a number of bytes
//! - .INCLUDE path: include a file
//! - .WARN message: print a warning
//! - .ERROR message: prints an error

pub mod codegen_utils;
pub mod directives;
pub mod instructions;
pub mod statement; 

use regex::Regex;
use colored::Colorize;
use std::{fmt, fs};
use std::collections::HashMap;
use statement::{Statement, TokenSpan};
use instructions::*;
use directives::*;

/// The address of an instruction in the bytecode
pub type OpcodeAddress = u16;

/// A symbol table is a mapping of labels to their addresses in the bytecode
pub type SymbolTable = HashMap<String, OpcodeAddress>;

const BYTES_PER_INSTRUCTION: u16 = 2;

/// Assemble a file into a vector of bytes
pub fn assemble_from_file(path: &str) -> Result<Vec<u8>, Error> {
    let source = fs::read_to_string(path).map_err(|_| Error::ReadError {
        path: path.to_string(),
    })?;
    assemble(&source)
}

/// Assemble source code string into a vector of bytes
pub fn assemble(source: &str) -> Result<Vec<u8>, Error> {
    let preprocessed = preprocess(source);
    let (symbol_table, unresolved) = first_pass(&preprocessed)?;
    second_pass(&symbol_table, &unresolved)
}

fn preprocess(source: &str) -> String {
    source
        .lines()
        .map(|line| line.splitn(2, ';').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_pass(source: &str) -> Result<(SymbolTable, Vec<Statement>), Error> {
    let mut labels = HashMap::new();
    let mut unresolved = Vec::new();
    let mut address: OpcodeAddress = 0;

    for (line_index, line) in source.lines().enumerate() {
        if line.ends_with(':') {
            let label = line.trim_end_matches(':');
            labels.insert(label.to_string(), address);
        } else {
            let re = Regex::new(r#""[^"]*"|[^,\s]+"#).unwrap();
            let mut lexemes = Vec::new();
            let mut spans = Vec::new();
            for mat in re.find_iter(line) {
                lexemes.push(mat.as_str());
                spans.push(TokenSpan::new(mat.start(), mat.end()));
            }

            let statement = Statement::new(
                lexemes[0],
                spans[0],
                lexemes[1..].to_vec(),
                spans[1..].to_vec(),
                line_index + 1,
                line
            );

            if line.starts_with(".") {
                // Here we need to know the output size of the directive to not mess
                // up the offsets. Essentially, we do double work here, but it would
                // be a useless hassle to try to avoid it!
                let n_bytes = parse_statement(&statement, &labels)?.len();
                address += n_bytes as u16;
            } else {
                address += BYTES_PER_INSTRUCTION;
            }

            unresolved.push(statement);
        }
    }

    Ok((labels, unresolved))
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
        // TODO: macros and conditionals?
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
            instruction_span: statement.instruction_span(),
            line_number: statement.line_number(),
            line: statement.line()
        })
    }?;
    Ok(opcode)
}

/// An error that can occur during the assembly process
#[derive(Debug)]
pub enum Error {
    /// An unknown instruction was encountered
    UnknownInstruction {
        instruction: String,
        instruction_span: TokenSpan,
        line_number: usize,
        line: String
    },
    /// An invalid argument was encountered
    InvalidArgument {
        argument: String,
        argument_span: TokenSpan,
        line_number: usize,
        line: String
    },
    /// An invalid number of arguments was passed
    InvalidArgumentCount {
        instruction: String,
        n_arguments: usize,
        expected: Vec<usize>,
        extra_argument_spans: Vec<TokenSpan>,
        line_number: usize,
        line: String
    },
    /// An error generated with `.ERROR` direcrive
    UserError {
        message: String,
        line_number: usize,
        line: String
    },
    /// A file could not be read
    ReadError {
        path: String
    },
    /// An error in a file included with `.INCLUDE` directive
    IncludeError {
        path: String,
        error: Box<Error>,
        line_number: usize,
        line: String
    },
    /// An argument overflowed the allowed number of bits (e.g. a byte was
    /// passed instead of a nibble)
    ArgumentOverflow {
        argument: u16,
        argument_span: TokenSpan,
        expected_n_bits: usize,
        line_number: usize,
        line: String
    },
    /// An internal error. If you see this, this is probably an internal bug
    /// in the assembler itself. Submit a GitHub issue!
    InvalidArgumentIndex {
        requested_index: usize,
        n_arguments: usize
    }
}

impl std::error::Error for Error {}

fn underline_spans(line: &str, spans: Vec<&TokenSpan>) -> String {
    let mut underline = vec![' '; line.len()];
    for span in spans {
        for i in span.start()..span.end() {
            if i < underline.len() {
                underline[i] = '^';
            }
        }
    }
    underline.into_iter().collect()
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (message, line, line_number, underlined_spans) = match self {
            Error::UnknownInstruction { instruction, instruction_span, line_number, line } => (
                format!("unknown instruction \"{}\" at line {}", instruction, line_number),
                Some(line), Some(line_number), vec![instruction_span]
            ),
            Error::InvalidArgument { argument, argument_span, line_number, line } => (
                format!("invalid argument \"{}\" at line {}", argument, line_number),
                Some(line), Some(line_number), vec![argument_span]
            ),
            Error::InvalidArgumentCount {instruction, line_number, n_arguments, expected, extra_argument_spans, line} => (
                format!(
                    "invalid argument count for instruction \"{}\" at line {}: found {}, expected {:?}",
                    instruction, line_number, n_arguments, expected
                ),
                Some(line), Some(line_number), extra_argument_spans.iter().collect()
            ),
            Error::UserError { message, line_number, line } => (
                format!("line {}: {}", line_number, message),
                Some(line), Some(line_number), vec![]
            ),
            Error::ReadError { path } => (
                format!("failed to read file {}", path),
                None, None, vec![]
            ),
            Error::IncludeError { path, line_number, error, line } => (
                format!("in file {} included at line {}: {}", path, line_number, error),
                Some(line), Some(line_number), vec![]
            ),
            Error::ArgumentOverflow { line_number, argument, argument_span, expected_n_bits, line } => (
                format!(
                    "argument overflow at line {}: maximum allowed value {} (to fit in {} bits), got {}", 
                    line_number, ((1u32 << 16) - 1) >> (16 - expected_n_bits), expected_n_bits, argument
                ),
                Some(line), Some(line_number), vec![argument_span]
            ),
            Error::InvalidArgumentIndex { requested_index, n_arguments } => (
                format!(
                    "invalid argument index {} requested; number of arguments: {}. If you see this, this \
                    is probably an internal bug in the assembler itself. Please, submit a GitHub issue.",
                    requested_index, n_arguments
                ),
                None, None, vec![]
            )
        };
        writeln!(f, "{}", message)?;
        if let (Some(line), Some(line_number)) = (line, line_number) {
            write!(f, "{}\t{}", line_number, line)?;
            if underlined_spans.len() != 0 {
                writeln!(f, "")?;
                write!(f, "\t{}", underline_spans(line, underlined_spans).green())?;
            }
        }
        Ok(())
    }
}
