use std::env;

use green::{
    cli,
    common::error_code::ErrorCode,
    interpreter::execute,
    lexer::lexical_analyzer,
    parser::ast,
    utils::{error_message::ErrorMessage, misc}
};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let config = match cli::args::Config::new(&args) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };
    
    let content = match misc::load_file_content(&config.file_path) {
        Ok(content) => content,
        Err(_) => {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorCode::Io001,
                &[("file_name", &config.file_path)]
            )?)
        }
    };

    let tokens = match lexical_analyzer::lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorCode::Lex001,
                &[("message", &e)]
            )?)
        }
    };

    let ast = match ast::parse(tokens){
        Ok(node) => node,
        Err(e) => {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorCode::Parse001, &[("message", &e)]
            )?)
        }
    };
    ast.print(0);

    if config.option == cli::args::RunMode::Execute {
        if let Err(e) = execute::execute(&ast) {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorCode::Runtime001, &[("message", &e)]
            )?)
        }
    }

    return Ok(())
}
