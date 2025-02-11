use clap::Parser;

use green::{
    analyzer::semantic, cli, error::{
        error_code::ErrorCode, error_context::ErrorContext, error_message::ErrorMessage
    }, interpreter::execute, lexer::lexical_analyzer, parser::parser, utils::{ast_to_json::JsonData, misc}
};

fn main() -> Result<(), String> {
    let mut error_flag = false;

    let cli = cli::args::Cli::parse();
    
    let content = match misc::load_file_content(&cli.file) {
        Ok(content) => content,
        Err(_) => {
            return Err(ErrorMessage::global().get_error_message(
                ErrorContext::new(
                    ErrorCode::Io001,
                    None, None,
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
            let error_msg = ErrorMessage::global().get_error_message(error)?;
            println!("{}", error_msg);
        }
        return Err("error".to_string())
    }

    let (ast, errors) = parser::parse(tokens);

    if !errors.is_empty() {
        error_flag = true;
    }

    if error_flag {
        for error in errors {
            let error_msg = ErrorMessage::global().get_error_message(error)?;
            println!("{}", error_msg);
        }
        return Err("error".to_string())
    }

    let semantic = match semantic::semantic(&ast) {
        Ok(semantic) => semantic,
        Err(errors) => {
            for error in errors {
                let error_msg = ErrorMessage::global().get_error_message(error)?;
                println!("{}", error_msg);
            }
            return Err("error".to_string())
        }
    };

    // dbg!(&semantic);

    if cli.analyze {
        let _ = JsonData::new(semantic);
    } else {
        if let Err(e) = execute::execute(&semantic) {
            return Err(ErrorMessage::global().get_error_message(
                ErrorContext::new(
                    ErrorCode::Runtime001,
                    None, None,
                    vec![("message", &e)],
                )
            )?)
        }
    }

    return Ok(())
}
