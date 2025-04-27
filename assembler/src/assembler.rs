use regex::Regex;
use colored::Colorize;
use std::{fmt, fs};
use std::collections::HashMap;
use crate::statement::{Statement, TokenSpan};
use crate::instructions::*;
use crate::directives::*;

pub type OpcodeAddress = u16;
pub type SymbolTable = HashMap<String, OpcodeAddress>;

const BYTES_PER_INSTRUCTION: u16 = 2;
const REGEX_LEXING: &str = r#""[^"]*"|[^,\s]+"#;
const REGEX_MACRO: &str = r"(?s)\.macro\s+(?P<name>\w+)\s+(?P<args>[^\n]+)\s*\n(?P<body>.*?)\n\.endmacro";

pub fn assemble_from_file(path: &str) -> Result<Vec<u8>, Error> {
    let source = fs::read_to_string(path).map_err(|_| Error::ReadError {
        path: path.to_string(),
    })?;
    assemble(&source)
}

pub fn assemble(source: &str) -> Result<Vec<u8>, Error> {
    let preprocessed = preprocess(source)?;
    let (symbol_table, unresolved) = first_pass(&preprocessed)?;
    second_pass(&symbol_table, &unresolved)
}

#[derive(Debug)]
struct Macro {
    name: String,
    args: Vec<String>,
    body: String,
}

fn preprocess(source: &str) -> Result<String, Error> {
    let mut preprocessed = source
        .lines()
        .map(|line| line.splitn(2, ';').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let macro_re = Regex::new(REGEX_MACRO).unwrap();
    let mut macros = Vec::<Macro>::new();
    for cap in macro_re.captures_iter(&preprocessed) {
        let name = cap["name"].to_string();
        let args: Vec<String> = cap["args"].split_whitespace().map(|s| s.to_string()).collect();
        let body = cap["body"].to_string();

        macros.push(Macro { name, args, body });
    }
    preprocessed = macro_re.replace_all(&preprocessed, "").to_string();

    for Macro { name, args: definition_args, body } in macros {
        let macro_usage_re = Regex::new(&format!(r"{}\s+(?P<args>[^\n]+)", name)).unwrap();
        let mut error = None;
        preprocessed = macro_usage_re.replace_all(&preprocessed, |caps: &regex::Captures| {
            let lexing_re = Regex::new(REGEX_LEXING).unwrap();
            let args = lexing_re.captures_iter(&caps["args"]).map(|cap| cap[0].to_string()).collect::<Vec<_>>();

            // This is the definition of the macro, not a usage
            if args == definition_args {
                return body.clone();
            }

            if args.len() != definition_args.len() {
                error = Some(Error::InvalidArgumentCount {
                    instruction: name.clone(),
                    n_arguments: args.len(),
                    expected: vec![definition_args.len()],
                    extra_argument_spans: vec![],  // TODO ?
                    line_number: 0,  // TODO ?
                    line: "".to_string()  // TODO ?
                });
            }

            let mut modified_body = body.clone();
            for i in 0..args.len() {
                let placeholder = format!("${}", definition_args[i]);
                modified_body = modified_body.replace(&placeholder, &args[i]);
            }

            modified_body
        }).to_string();

        if let Some(err) = error {
            return Err(err);
        }
    }

    preprocessed = preprocessed
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(preprocessed)
}

fn first_pass(preprocessed: &str) -> Result<(SymbolTable, Vec<Statement>), Error> {
    let mut labels = HashMap::new();
    let mut unresolved = Vec::new();
    let mut address: OpcodeAddress = 0;

    for (line_index, line) in preprocessed.lines().enumerate() {
        if line.ends_with(':') {
            let label = line.trim_end_matches(':');
            labels.insert(label.to_string(), address);
        } else {
            let re = Regex::new(REGEX_LEXING).unwrap();
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

#[derive(Debug)]
pub enum Error {
    UnknownInstruction {
        instruction: String,
        instruction_span: TokenSpan,
        line_number: usize,
        line: String
    },
    InvalidArgument {
        argument: String,
        argument_span: TokenSpan,
        line_number: usize,
        line: String
    },
    InvalidArgumentCount {
        instruction: String,
        n_arguments: usize,
        expected: Vec<usize>,
        extra_argument_spans: Vec<TokenSpan>,
        line_number: usize,
        line: String
    },
    UserError {
        message: String,
        line_number: usize,
        line: String
    },
    ReadError {
        path: String
    },
    IncludeError {
        path: String,
        error: Box<Error>,
        line_number: usize,
        line: String
    },
    ArgumentOverflow {
        argument: u16,
        argument_span: TokenSpan,
        expected_n_bits: usize,
        line_number: usize,
        line: String
    },
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
                    "invalid argument count for \"{}\" at line {}: found {}, expected {:?}",
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
