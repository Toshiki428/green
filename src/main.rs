use std::env;

use green::{
    cli,
    common::error_code::ErrorCode,
    interpreter::execute,
    lexer::lexical_analyzer,
    parser::ast,
    utils::{error_message::ErrorMessage, misc}
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match cli::args::Config::new(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    
    let content = match misc::load_file_content(&config.file_path) {
        Ok(content) => content,
        Err(_) => {
            match ErrorMessage::global().get_error_message(
                &ErrorCode::Io001,
                &[("file_name", &config.file_path)]
            ) {
                Ok(message) => eprintln!("{}", message),
                Err(message) => eprintln!("{}", message),
            }
            return;
        }
    };

    let tokens = match lexical_analyzer::lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            match ErrorMessage::global().get_error_message(
                &ErrorCode::Lex001,
                &[("message", &e)]
            ) {
                Ok(message) => eprintln!("{}", message),
                Err(message) => eprintln!("{}", message),
            }
            return;
        }
    };

    let ast = match ast::parse(tokens){
        Ok(node) => node,
        Err(e) => {
            match ErrorMessage::global().get_error_message(
                &ErrorCode::Parse001, &[("message", &e)]
            ) {
                Ok(message) => eprintln!("{}", message),
                Err(message) => eprintln!("{}", message),
            }
            return;
        }
    };
    ast.print(0);

    if config.option == cli::args::RunMode::Execute {
        if let Err(e) = execute::execute(&ast) {
            match ErrorMessage::global().get_error_message(
                &ErrorCode::Runtime001, &[("message", &e)]
            ) {
                Ok(message) => eprintln!("{}", message),
                Err(message) => eprintln!("{}", message),
            }
            return;
        }
    }
}
