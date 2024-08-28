use std::path::PathBuf;

use clap::Parser;
use merry_compiler::compile::{compile_dir, compile_file};

#[derive(Parser)]
pub struct Cli {
    input_path: PathBuf,
    output_path: PathBuf,
    #[arg(short, long)]
    head: Option<PathBuf>
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut input_path = std::env::current_dir()?;
    input_path.push(&cli.input_path);
  
    if input_path.is_dir() {
        compile_dir(input_path, cli.output_path, &cli.head)?;
        return Ok(());
    }

    if input_path.is_file() {
        std::fs::create_dir_all(cli.output_path.parent().unwrap())?;
        compile_file(input_path, cli.output_path, &cli.head)?;
        return Ok(());
    }
    
    return Ok(());
}
