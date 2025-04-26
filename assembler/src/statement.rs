use crate::*;
use crate::assembler::{OpcodeAddress, SymbolTable};

#[derive(Debug, Clone)]
pub struct Statement<'a> {
    lexemes: Vec<&'a str>,
    line_number: usize,
}

impl Statement<'_> {
    pub fn new(lexemes: Vec<&str>, line_number: usize) -> Statement {
        Statement { lexemes, line_number }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn n_arguments(&self) -> usize {
        self.lexemes.len() - 1
    }

    pub fn argument(&self, argument_index: usize) -> &str {
        self.lexemes[argument_index + 1]
    }

    pub fn instruction(&self) -> &str {
        self.lexemes[0]
    }

    pub fn parse_number(
        &self, argument_index: usize, max_n_bits: usize
    ) -> Result<u16, assembler::Error> {
        let lexeme = self.argument(argument_index);
        let num = if lexeme.starts_with("0x") {
            u16::from_str_radix(&lexeme[2..], 16)
        } else if lexeme.starts_with("0b") {
            u16::from_str_radix(&lexeme[2..], 2)
        } else {
            lexeme.parse::<u16>()
        };
        match num {
            Ok(num) => {
                let max: u16 = u16::MAX >> (16 - max_n_bits);
                if num > max {
                    Err(assembler::Error::ArgumentOverflow {
                        instruction: self.instruction().to_string(),
                        line_number: self.line_number(),
                        argument: num,
                        expected_n_bits: max_n_bits
                    })
                } else {
                    Ok(num)
                }
            },
            Err(_) => Err(self.invalid_argument(argument_index))
        }
    }

    pub fn parse_register(&self, argument_index: usize) -> Result<u16, assembler::Error> {
        let lexeme = self.argument(argument_index);
        let error = self.invalid_argument(argument_index);
        if lexeme.len() == 2 && lexeme.starts_with('V') {
            let register_char = lexeme.chars().nth(1).unwrap();
            let register = register_char.to_digit(16)
                .ok_or_else(|| error)? as u16;
            Ok(register)
        } else {
            Err(error)
        }
    }

    pub fn parse_only_two_registers(&self) -> Result<(u16, u16), assembler::Error> {
        self.assert_n_arguments(2)?;
        let x = self.parse_register(0)?;
        let y = self.parse_register(1)?;
        Ok((x, y))
    }

    pub fn parse_label(
        &self,
        argument_index: usize,
        symbol_table: &SymbolTable
    ) -> Result<OpcodeAddress, assembler::Error> {
        let lexeme = self.argument(argument_index);
        symbol_table
            .get(lexeme)
            .copied()
            .map(|x| x + 0x200)  // offset for ROM
            .ok_or_else(|| self.invalid_argument(argument_index))
    }

    pub fn parse_addr_or_label(
        &self,
        argument_index: usize,
        symbol_table: &SymbolTable
    ) -> Result<OpcodeAddress, assembler::Error> {
        self.parse_number(argument_index, 12)
            .or_else(|_| self.parse_label(argument_index, symbol_table))
    }

    // TODO: seems ugly. how do I not delete spaces and commas inside quotes?
    pub fn parse_string(&self, argument_index: usize) -> Result<String, assembler::Error> {
        let lexeme = self.argument(argument_index);
        if !lexeme.starts_with('"') {
            return Err(self.invalid_argument(argument_index));
        }
        for i in argument_index..self.n_arguments() - 1 {
            if self.argument(i).ends_with('"') {
                return Ok(self.lexemes[argument_index..i].join(" ").trim_matches('"').to_string());
            }
        };
        Err(self.invalid_argument(argument_index))
    }

    pub fn assert_n_arguments(&self, n: usize) -> Result<(), assembler::Error> {
        let n_arguments = self.n_arguments();
        if n_arguments != n {
            return Err(self.invalid_argument_count(n_arguments, vec![n]));
        }
        Ok(())
    }

    pub fn invalid_argument(&self, argument_index: usize) -> assembler::Error {
        assembler::Error::InvalidArgument {
            argument: self.lexemes[argument_index + 1].to_string(),
            line_number: self.line_number
        }
    }

    pub fn invalid_argument_count(
        &self,
        n_arguments: usize,
        expected: Vec<usize>  // e.g. 1 OR 2 arguments are expected (JP)
    ) -> assembler::Error {
        assembler::Error::InvalidArgumentCount {
            instruction: self.instruction().to_string(),
            line_number: self.line_number,
            n_arguments,
            expected
        }
    }
}
