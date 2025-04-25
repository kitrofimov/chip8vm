mod assembler;
mod statement;
mod instructions;
mod directives;
mod codegen_utils;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.asm> <output.ch8>", args[0]);
        return;
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let bytecode = assembler::assemble_from_file(&input_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to assemble! {}", e);
            std::process::exit(1);
        });

    fs::write(output_path, bytecode)
        .unwrap_or_else(|_| {
            eprintln!("Failed to write to output file: {}", output_path);
            std::process::exit(2);
        });
}
