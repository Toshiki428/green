use clap::{Parser, ArgGroup};

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(group(
    ArgGroup::new("mode")
        .required(false)
        .args(&["execute", "analyze"])
))]
pub struct Cli {
    /// The input file to process
    #[arg(default_value_t = String::from("main.grn"))]
    pub file: String,

    /// Execute the script (default behavior)
    #[arg(short, long)]
    pub execute: bool,

    /// Analyze the script without execution
    #[arg(short, long)]
    pub analyze: bool,
}
