use std::env;
use std::fs;
use std::collections::HashMap;

const BYTES_PER_INSTRUCTION: u16 = 2;

#[derive(Debug)]
struct Statement {
    statement: String,
    line_number: usize,
    instruction_address: OpcodeAddress
}

#[derive(Debug)]
struct AssembleError {
    message: String,
    bad_statement: Statement
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

    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.ends_with(':') {
            let label = line.trim_end_matches(':');
            labels.insert(label.to_string(), address);
        } else {
            unresolved.push(Statement {
                statement: line.to_string(),
                line_number: line_num,
                instruction_address: address
            });
            address += BYTES_PER_INSTRUCTION;
        }
    }

    (labels, unresolved)
}

fn second_pass(symbol_table: &SymbolTable, unresolved: &Vec<Statement>) -> Result<Vec<Opcode>, AssembleError> {
    unimplemented!();
}
