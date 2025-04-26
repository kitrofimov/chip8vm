use crate::*;

pub fn byte(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    statement.assert_n_arguments(1)?;
    Ok(vec![statement.parse_number(0)? as u8])
}

pub fn word(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    statement.assert_n_arguments(1)?;
    Ok(split_u16!(statement.parse_number(0)?))
}

pub fn text(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    Ok(statement.parse_string(0)?.into_bytes())
}

pub fn fill(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    statement.assert_n_arguments(2)?;
    let n = statement.parse_number(0)?;
    let byte = statement.parse_number(1)? as u8;
    Ok(vec![byte; n as usize])
}

pub fn space(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    statement.assert_n_arguments(1)?;
    Ok(vec![0x00; statement.parse_number(0)? as usize])
}

pub fn _include(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    let path = statement.parse_string(0)?;
    assemble_from_file(&path).map_err(|e| AssembleError::IncludeError {
        line_number: statement.line_number,
        error: Box::new(e)
    })
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
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    // TODO: reimplement this when I decide about logging
    println!(
        "WARNING: {}; line {}",
        statement.parse_string(0).unwrap_or("<no message>".to_string()),
        statement.line_number
    );
    Ok(vec![])
}

pub fn _error(
    statement: &Statement,
) -> Result<Vec<u8>, AssembleError> {
    Err(AssembleError::UserError {
        message: statement.parse_string(0).unwrap_or("<no message>".to_string()),
        line_number: statement.line_number,
    })
}
