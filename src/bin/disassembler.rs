use std::{env, fs};
use chip8vm::logging::error;

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
    let mut result = String::new();
    let mut i = 0;

    while i < bytecode.len() {
        if i + 1 >= bytecode.len() {
            result.push_str(&format!(".byte 0x{:02X}\n", bytecode[i]));
            break;
        }

        let high = bytecode[i] as u16;
        let low = bytecode[i + 1] as u16;
        let opcode = (high << 8) | low;

        let line = decode_instruction(opcode);
        result.push_str(&line);
        result.push('\n');

        i += 2;
    }

    result
}

fn decode_instruction(opcode: u16) -> String {
    let nibbles = (
        (opcode & 0xF000) >> 12,
        (opcode & 0x0F00) >> 8,
        (opcode & 0x00F0) >> 4,
        (opcode & 0x000F)
    );

    let nnn = opcode & 0x0FFF;
    let kk = (opcode & 0x00FF) as u8;
    let x = ((opcode & 0x0F00) >> 8) as u8;
    let y = ((opcode & 0x00F0) >> 4) as u8;
    let n = (opcode & 0x000F) as u8;

    match nibbles {
        (0x0, 0x0, 0xE, 0x0) => "CLS".to_string(),
        (0x0, 0x0, 0xE, 0xE) => "RET".to_string(),
        (0x0, _, _, _)       => format!("SYS 0x{:03X}",         nnn),
        (0x1, _, _, _)       => format!("JP 0x{:03X}",          nnn),
        (0x2, _, _, _)       => format!("CALL 0x{:03X}",        nnn),
        (0x3, _, _, _)       => format!("SE V{:X}, 0x{:02X}",   x, kk),
        (0x4, _, _, _)       => format!("SNE V{:X}, 0x{:02X}",  x, kk),
        (0x5, _, _, 0x0)     => format!("SE V{:X}, V{:X}",      x, y),
        (0x6, _, _, _)       => format!("LD V{:X}, 0x{:02X}",   x, kk),
        (0x7, _, _, _)       => format!("ADD V{:X}, 0x{:02X}",  x, kk),
        (0x8, _, _, 0x0)     => format!("LD V{:X}, V{:X}",      x, y),
        (0x8, _, _, 0x1)     => format!("OR V{:X}, V{:X}",      x, y),
        (0x8, _, _, 0x2)     => format!("AND V{:X}, V{:X}",     x, y),
        (0x8, _, _, 0x3)     => format!("XOR V{:X}, V{:X}",     x, y),
        (0x8, _, _, 0x4)     => format!("ADD V{:X}, V{:X}",     x, y),
        (0x8, _, _, 0x5)     => format!("SUB V{:X}, V{:X}",     x, y),
        (0x8, _, _, 0x6)     => format!("SHR V{:X}",            x),
        (0x8, _, _, 0x7)     => format!("SUBN V{:X}, V{:X}",    x, y),
        (0x8, _, _, 0xE)     => format!("SHL V{:X}",            x),
        (0x9, _, _, 0x0)     => format!("SNE V{:X}, V{:X}",     x, y),
        (0xA, _, _, _)       => format!("LD I, 0x{:03X}",       nnn),
        (0xB, _, _, _)       => format!("JP V0, 0x{:03X}",      nnn),
        (0xC, _, _, _)       => format!("RND V{:X}, 0x{:02X}",  x, kk),
        (0xD, _, _, _)       => format!("DRW V{:X}, V{:X}, {}", x, y, n),
        (0xE, _, 0x9, 0xE)   => format!("SKP V{:X}",            x),
        (0xE, _, 0xA, 0x1)   => format!("SKNP V{:X}",           x),
        (0xF, _, 0x0, 0x7)   => format!("LD V{:X}, DT",         x),
        (0xF, _, 0x0, 0xA)   => format!("LD V{:X}, K",          x),
        (0xF, _, 0x1, 0x5)   => format!("LD DT, V{:X}",         x),
        (0xF, _, 0x1, 0x8)   => format!("LD ST, V{:X}",         x),
        (0xF, _, 0x1, 0xE)   => format!("ADD I, V{:X}",         x),
        (0xF, _, 0x2, 0x9)   => format!("LD F, V{:X}",          x),
        (0xF, _, 0x3, 0x3)   => format!("LD B, V{:X}",          x),
        (0xF, _, 0x5, 0x5)   => format!("LD [I], V{:X}",        x),
        (0xF, _, 0x6, 0x5)   => format!("LD V{:X}, [I]",        x),
        _ => format!(".word 0x{:04X}", opcode),
    }
}
