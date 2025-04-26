use crate::*;

pub fn byte(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn word(
    statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    statement.assert_n_arguments(1)?;
    Ok(split_u16!(statement.parse_number(0)?))
}

pub fn text(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn fill(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn space(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn _include(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
}

pub fn _macro(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn endmacro(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn define(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn _if(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn _else(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn endif(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn warn(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}

pub fn _error(
    _statement: &Statement, 
    _symbol_table: &SymbolTable
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::Unimplemented)
}
