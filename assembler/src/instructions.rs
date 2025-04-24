use crate::*;

pub fn cls(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Ok(0x00E0)
}

pub fn ret(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Ok(0x00EE)
}

pub fn sys(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn jp(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn call(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn se(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn sne(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn ld(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn add(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn or(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn and(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn xor(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn sub(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn shr(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn subn(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn shl(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn rnd(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn drw(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn skp(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn sknp(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Opcode, AssembleError> {
    Err(AssembleError::Unimplemented)
}
