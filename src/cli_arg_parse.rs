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
            _ => Err(format!("不正なオプション {}", s))
        }
    }
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, String> {
        let (option_str, file_path) = match args.len() {
            3 => (args[1].clone(), args[2].clone()),
            2 => ("-exe".to_string(), args[1].clone()),
            1 => ("-exe".to_string(), "main.grn".to_string()),
            _ => return Err("不正なコマンドライン引数の数".to_string()),
        };

        let option = RunMode::from_str(&option_str)?;
        Ok(Config { option, file_path })
    }
}
