mod assembler;
mod statement;
mod instructions;
mod directives;
mod codegen_utils;
use std::{env, fs};
use colored::Colorize;

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
            error(e.to_string());
            std::process::exit(1);
        });

    fs::write(output_path, bytecode)
        .unwrap_or_else(|e| {
            error(format!("failed to write to output file: {}", e.to_string()));
            std::process::exit(2);
        });
}

fn error(error_message: String) {
    let error_title = "error:".red().bold();
    eprintln!("{} {}", error_title, error_message);
}

fn warning(message: String, line_number: usize) {
    let warning = "warning:".yellow().bold();
    eprintln!("{} line {}: {}", warning, line_number, message);
}
