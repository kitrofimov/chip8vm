use crate::*;
use crate::assembler::{OpcodeAddress, SymbolTable};

#[derive(Debug, Clone, Copy)]
pub struct TokenSpan {
    start: usize,
    end: usize,
}

impl TokenSpan {
    pub fn new(start: usize, end: usize) -> TokenSpan {
        TokenSpan { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone)]
pub struct Statement<'a> {
    instruction: &'a str,
    instruction_span: TokenSpan,
    arguments: Vec<&'a str>,
    argument_spans: Vec<TokenSpan>,
    line_number: usize,
    line: &'a str
}

impl<'a> Statement<'a> {
    pub fn new(
        instruction: &'a str,
        instruction_span: TokenSpan,
        arguments: Vec<&'a str>,
        argument_spans: Vec<TokenSpan>,
        line_number: usize,
        line: &'a str
    ) -> Statement<'a> {
        Statement {
            instruction,
            instruction_span,
            arguments,
            argument_spans,
            line_number,
            line
        }
    }

    pub fn instruction(&self) -> &str {
        self.instruction
    }

    pub fn instruction_span(&self) -> TokenSpan {
        self.instruction_span
    }

    pub fn n_arguments(&self) -> usize {
        self.arguments.len()
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn line(&self) -> String {
        self.line.to_string()
    }

    pub fn argument(&self, argument_index: usize) -> Result<&str, assembler::Error> {
        self.arguments.get(argument_index)
            .ok_or(assembler::Error::InvalidArgumentIndex {
                requested_index: argument_index,
                n_arguments: self.arguments.len()
            })
            .copied()
    }

    pub fn parse_number(
        &self, argument_index: usize, max_n_bits: usize
    ) -> Result<u16, assembler::Error> {
        let lexeme = self.argument(argument_index)?;
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
                        argument: num,
                        argument_span: self.argument_spans[argument_index],
                        expected_n_bits: max_n_bits,
                        line_number: self.line_number(),
                        line: self.line()
                    })
                } else {
                    Ok(num)
                }
            },
            Err(_) => Err(self.invalid_argument(argument_index))
        }
    }

    pub fn parse_register(&self, argument_index: usize) -> Result<u16, assembler::Error> {
        let lexeme = self.argument(argument_index)?;
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
        let lexeme = self.argument(argument_index)?;
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

    pub fn parse_string(&self, argument_index: usize) -> Result<String, assembler::Error> {
        let lexeme = self.argument(argument_index)?;
        Ok(lexeme.trim_matches('"').to_string())
    }

    pub fn assert_n_arguments(&self, n: usize) -> Result<(), assembler::Error> {
        let n_arguments = self.arguments.len();
        if n_arguments != n {
            return Err(self.invalid_argument_count(n_arguments, &[n]));
        }
        Ok(())
    }

    pub fn invalid_argument(&self, argument_index: usize) -> assembler::Error {
        assembler::Error::InvalidArgument {
            argument: self.arguments[argument_index].to_string(),
            argument_span: self.argument_spans[argument_index],
            line_number: self.line_number,
            line: self.line()
        }
    }

    pub fn invalid_argument_count(
        &self,
        n_arguments: usize,
        expected: &[usize]
    ) -> assembler::Error {
        let max_expected = *expected.iter().max().unwrap_or(&0);
        let extra_argument_spans = self.argument_spans
            .get(max_expected..)
            .unwrap_or(&[])
            .to_vec();
    
        assembler::Error::InvalidArgumentCount {
            instruction: self.instruction.to_string(),
            n_arguments,
            expected: expected.to_vec(),
            extra_argument_spans,
            line_number: self.line_number,
            line: self.line(),
        }
    }
}
