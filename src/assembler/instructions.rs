//! Code generation functions for instructions

use super::statement::Statement;
use crate::assembler;
use crate::assembler::SymbolTable;
use crate::split_u16;

pub fn cls(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(0)?;
    Ok(split_u16!(0x00E0))
}

pub fn ret(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(0)?;
    Ok(split_u16!(0x00EE))
}


pub fn sys(
    statement: &Statement,
    symbol_table: &SymbolTable
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    Ok(split_u16!(0x0000 | statement.parse_addr_or_label(0, symbol_table)?))  // 0x0nnn
}

pub fn jp(
    statement: &Statement,
    symbol_table: &SymbolTable
) -> Result<Vec<u8>, assembler::Error> {
    match statement.n_arguments() {
        1 => Ok(split_u16!(0x1000 | statement.parse_addr_or_label(0, symbol_table)?)),  // 0x1nnn
        2 => {
            let register = statement.parse_register(0)?;
            let address = statement.parse_addr_or_label(1, symbol_table)?;
            if register != 0 {  // Only V0 is allowed
                return Err(statement.invalid_argument(0));
            }
            Ok(split_u16!(0xB000 | address))  // 0xBnnn
        }
        _ => Err(statement.invalid_argument_count(
            statement.n_arguments(), &[1, 2]
        ))
    }
}

pub fn call(
    statement: &Statement,
    symbol_table: &SymbolTable
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    Ok(split_u16!(0x2000 | statement.parse_addr_or_label(0, symbol_table)?))  // 0x2nnn
}

pub fn se(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    let x = statement.parse_register(0)?;
    statement
        .parse_number(1, 8)                                // SE Vx, byte
        .map(|byte| split_u16!(0x3000 | (x << 8) | byte))  // 0x3xkk
        .or_else(|_| {
            let y = statement.parse_register(1)?;         // SE Vx, Vy
            Ok(split_u16!(0x5000 | (x << 8) | (y << 4)))  // 0x5xy0
        })
}

pub fn sne(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    let x = statement.parse_register(0)?;
    statement
        .parse_number(1, 8)                                // SNE Vx, byte
        .map(|byte| split_u16!(0x4000 | (x << 8) | byte))  // 0x4xkk
        .or_else(|_| {
            let y = statement.parse_register(1)?;         // SNE Vx, Vy
            Ok(split_u16!(0x9000 | (x << 8) | (y << 4)))  // 0x9xy0
        })
}

pub fn ld(
    statement: &Statement,
    symbol_table: &SymbolTable
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    let address = statement.parse_addr_or_label(1, symbol_table);
    let x = statement.parse_register(0);
    let y = statement.parse_register(1);

    match statement.argument(0)? {
        "I"   => Ok(split_u16!(0xA000 | address?)),   // LD I, addr   0xAnnn
        "DT"  => Ok(split_u16!(0xF015 | (y? << 8))),  // LD DT, Vy    0xFy15
        "ST"  => Ok(split_u16!(0xF018 | (y? << 8))),  // LD ST, Vy    0xFy18
        "F"   => Ok(split_u16!(0xF029 | (y? << 8))),  // LD F, Vy     0xFy29
        "B"   => Ok(split_u16!(0xF033 | (y? << 8))),  // LD B, Vy     0xFy33
        "[I]" => Ok(split_u16!(0xF055 | (y? << 8))),  // LD [I], Vy   0xFy55
        _ => {
            let x = x?;
            match statement.argument(1)? {
                "DT"  => Ok(split_u16!(0xF007 | (x << 8))),  // LD Vx, DT   0xFx07
                "K"   => Ok(split_u16!(0xF00A | (x << 8))),  // LD Vx, K    0xFx0A
                "[I]" => Ok(split_u16!(0xF065 | (x << 8))),  // LD Vx, [I]  0xFx65
                _ => {
                    statement
                        .parse_number(1, 8)
                        .map(|byte| split_u16!(0x6000 | (x << 8) | byte))            // LD Vx, byte  0x6xkk
                        .or_else(|_| Ok(split_u16!(0x8000 | (x << 8) | (y? << 4))))  // LD Vx, Vy    0x8xy0
                }
            }
        }
    }
}

pub fn add(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    if statement.argument(0)? == "I" {
        let x = statement.parse_register(1)?;  // ADD I, Vx
        Ok(split_u16!(0xF01E | (x << 8)))      // 0xFx1E
    } else {
        let x = statement.parse_register(0)?;
        statement
            .parse_number(1, 8)                                // ADD Vx, byte
            .map(|byte| split_u16!(0x7000 | (x << 8) | byte))  // 0x7xkk
            .or_else(|_| {
                let y = statement.parse_register(1)?;         // ADD Vx, Vy
                Ok(split_u16!(0x8004 | (x << 8) | (y << 4)))  // 0x8xy4
            })
    }
}

pub fn sub(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8005 | (x << 8) | (y << 4)))  // 0x8xy5
}

pub fn subn(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8007 | (x << 8) | (y << 4)))  // 0x8xy7
}

pub fn or(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8001 | (x << 8) | (y << 4)))  // 0x8xy1
}

pub fn and(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8002 | (x << 8) | (y << 4)))  // 0x8xy2
}

pub fn xor(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8003 | (x << 8) | (y << 4)))  // 0x8xy3
}

pub fn shr(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x8006 | (x << 8) | (y << 4)))  // 0x8xy6
}

pub fn shl(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    let (x, y) = statement.parse_only_two_registers()?;
    Ok(split_u16!(0x800E | (x << 8) | (y << 4)))  // 0x8xyE
}

pub fn rnd(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    let x = statement.parse_register(0)?;
    let byte = statement.parse_number(1, 8)?;
    Ok(split_u16!(0xC000 | (x << 8) | byte))  // 0xCxkk
}

pub fn drw(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(3)?;
    let x = statement.parse_register(0)?;
    let y = statement.parse_register(1)?;
    let nibble = statement.parse_number(2, 4)?;
    Ok(split_u16!(0xD000 | (x << 8) | (y << 4) | nibble))  // 0xDxyn
}

pub fn skp(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    let x = statement.parse_register(0)?;
    Ok(split_u16!(0xE09E | (x << 8)))  // 0xEx9E
}

pub fn sknp(statement: &Statement) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    let x = statement.parse_register(0)?;
    Ok(split_u16!(0xE0A1 | (x << 8)))  // 0xExA1
}
