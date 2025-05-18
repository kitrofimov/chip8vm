//! Pretty-printing messages to the console

use colored::Colorize;

/// Pretty-print an error message to the console
pub fn error(error_message: String) {
    let error_title = "error:".red().bold();
    eprintln!("{} {}", error_title, error_message);
}

/// Pretty-print a warning message to the console
pub fn warning(message: String, line_number: usize) {
    let warning = "warning:".yellow().bold();
    eprintln!("{} line {}: {}", warning, line_number, message);
}
