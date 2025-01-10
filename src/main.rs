use std::env;

use green::utils;
use green::cli_arg_parse;
use green::lexical_analyzer;
use green::parser;
use green::interpreter;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = match cli_arg_parse::Config::new(&args) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };
    
    let content = match utils::load_file_content(&config.file_path) {
        Ok(content) => content,
        Err(_) => {
            match utils::get_error_message("FILE001", &[("file_name", &config.file_path)]) {
                Ok(message) => return Err(message),
                Err(message) => return Err(message),
            }
        }
    };

    let tokens = match lexical_analyzer::lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            match utils::get_error_message("LEX001", &[("message", &e)]) {
                Ok(message) => return Err(message),
                Err(message) => return Err(message),
            }
        }
    };

    let ast = match parser::parse(tokens){
        Ok(node) => node,
        Err(e) => {
            match utils::get_error_message("PARSE001", &[("message", &e)]) {
                Ok(message) => return Err(message),
                Err(message) => return Err(message),
            }
        }
    };

    if config.option == cli_arg_parse::RunMode::Execute {
        if let Err(e) = interpreter::execute(&ast) {
            match utils::get_error_message("RUNTIME001", &[("message", &e)]) {
                Ok(message) => return Err(message),
                Err(message) => return Err(message),
            }
        }
    }

    return Ok(())
}
