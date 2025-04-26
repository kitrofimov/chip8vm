use crate::*;
use crate::statement::*;

pub fn byte(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    Ok(vec![statement.parse_number(0, 8)? as u8])
}

pub fn word(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    Ok(split_u16!(statement.parse_number(0, 16)?))
}

pub fn text(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    Ok(statement.parse_string(0)?.into_bytes())
}

pub fn fill(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(2)?;
    let n = statement.parse_number(0, 16)?;
    let byte = statement.parse_number(1, 8)? as u8;
    Ok(vec![byte; n as usize])
}

pub fn space(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    statement.assert_n_arguments(1)?;
    Ok(vec![0x00; statement.parse_number(0, 16)? as usize])
}

pub fn _include(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    let path = statement.parse_string(0)?;
    assembler::assemble_from_file(&path).map_err(|e| assembler::Error::IncludeError {
        path, line_number: statement.line_number(), error: Box::new(e)
    })
}

pub fn warn(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    // TODO: reimplement this when I decide about logging
    println!(
        "WARNING: {}; line {}",
        statement.parse_string(0).unwrap_or("<no message>".to_string()),
        statement.line_number()
    );
    Ok(vec![])
}

pub fn _error(
    statement: &Statement,
) -> Result<Vec<u8>, assembler::Error> {
    Err(assembler::Error::UserError {
        message: statement.parse_string(0).unwrap_or("<no message>".to_string()),
        line_number: statement.line_number()
    })
}
