use std::{env, fs};
use chip8vm::logging::error;
use chip8vm::assembler;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.asm> <output.ch8>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let bytecode = assembler::assemble_from_file(&input_path)
        .unwrap_or_else(|e| {
            error(e.to_string());
            std::process::exit(2);
        });

    fs::write(output_path, bytecode)
        .unwrap_or_else(|e| {
            error(format!("failed to write to output file: {}", e.to_string()));
            std::process::exit(3);
        });
}
