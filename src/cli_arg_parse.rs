use crate::utils;

pub struct Config {
    pub option: RunMode,
    pub file_path: String,
}

#[derive(PartialEq)]
pub enum RunMode {
    Execute,
    Analysis,
}

impl RunMode {
    fn from_str(s: &str) -> Result<RunMode, String> {
        match s {
            "-exe" => Ok(RunMode::Execute),
            "-ana" => Ok(RunMode::Analysis),
            _ => {
                let message = utils::get_error_message("CMD001", &[("option", s)])?;
                Err(message)
            }
        }
    }
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, String> {
        let (option_str, file_path) = match args.len() {
            3 => (args[1].clone(), args[2].clone()),
            2 => ("-exe".to_string(), args[1].clone()),
            1 => ("-exe".to_string(), "main.grn".to_string()),
            _ => return Err(utils::get_error_message("CMD002", &[])?),
        };

        let option = RunMode::from_str(&option_str)?;
        Ok(Config { option, file_path })
    }
}
