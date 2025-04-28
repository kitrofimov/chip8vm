use std::{env, fs};
use colored::Colorize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.ch8> <output.asm>", args[0]);
        return;
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let bytecode = fs::read(input_path)
        .unwrap_or_else(|e| {
            error(format!("failed to read input file: {}", e.to_string()));
            std::process::exit(1);
        });
    let asm = disassemble(bytecode);

    fs::write(output_path, asm)
        .unwrap_or_else(|e| {
            error(format!("failed to write to output file: {}", e.to_string()));
            std::process::exit(2);
        });
}

fn disassemble(bytecode: Vec<u8>) -> String {
    unimplemented!();
}

fn error(error_message: String) {
    let error_title = "error:".red().bold();
    eprintln!("{} {}", error_title, error_message);
}
