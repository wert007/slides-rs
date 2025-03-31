use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, clap::Parser)]
enum Command {
    Run {
        file: PathBuf,
        #[clap(short, long, default_value = "out")]
        output: PathBuf,
    },
    Format {
        file: PathBuf,
        #[clap(long)]
        dry: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    match command {
        Command::Run { file, output } => {
            slides_lang::compiler::compile_project(file, output)?;
        }
        Command::Format { file, dry } => {
            slides_lang::formatter::format_file(file, dry)?;
        }
    }
    Ok(())
}
