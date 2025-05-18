use colored::Colorize;

pub fn error(error_message: String) {
    let error_title = "error:".red().bold();
    eprintln!("{} {}", error_title, error_message);
}

pub fn warning(message: String, line_number: usize) {
    let warning = "warning:".yellow().bold();
    eprintln!("{} line {}: {}", warning, line_number, message);
}
