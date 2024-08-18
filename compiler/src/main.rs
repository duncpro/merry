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
    
    if cli.input_path.is_dir() {
        compile_dir(cli.input_path, cli.output_path, &cli.head)?;
        return Ok(());
    }

    if cli.input_path.is_file() {
        compile_file(cli.input_path, cli.output_path, &cli.head)?;
        return Ok(());
    }
    
    return Ok(());
}
