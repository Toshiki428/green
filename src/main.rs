use clap::Parser;

use green::{
    cli, error::{
        error_code::ErrorCode, error_context::ErrorContext, error_message::ErrorMessage
    },interpreter::execute, lexer::lexical_analyzer, parser::ast, utils::{ast_to_json::ast_to_json, misc}
};

fn main() -> Result<(), String> {
    let mut error_flag = false;

    let cli = cli::args::Cli::parse();
    
    let content = match misc::load_file_content(&cli.file) {
        Ok(content) => content,
        Err(_) => {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Io001,
                    0, 0,
                    vec![("file_name", &cli.file)],
                )
            )?)
        }
    };


    let (tokens, errors) = lexical_analyzer::lex(&content);
    if !errors.is_empty() {
        error_flag = true;
    }
    if error_flag {
        for error in errors {
            let error_msg = ErrorMessage::global().get_error_message_with_location(error)?;
            println!("{}", error_msg);
        }
        return Err("error".to_string())
    }

    let (ast, errors) = ast::parse(tokens);

    ast.print(0);
    if !errors.is_empty() {
        error_flag = true;
    }

    if error_flag {
        for error in errors {
            let error_msg = ErrorMessage::global().get_error_message_with_location(error)?;
            println!("{}", error_msg);
        }
        return Err("error".to_string())
    }

    if cli.analyze {
        let _ = ast_to_json(ast);
    } else {
        if let Err(e) = execute::execute(&ast) {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime001,
                    0, 0,
                    vec![("message", &e)],
                )
            )?)
        }
    }

    return Ok(())
}
