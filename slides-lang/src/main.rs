use std::path::PathBuf;

use clap::Parser;
use slides_lang::compiler::DebugLang;

#[derive(Debug, clap::Parser)]
enum Command {
    Run {
        file: PathBuf,
        #[clap(short, long, default_value = "out")]
        output: PathBuf,
        #[clap(short, long = "dbg", default_value = "")]
        debug: DebugLang,
    },
    Format {
        file: PathBuf,
        #[clap(long)]
        dry: bool,
        #[clap(short, long = "dbg", default_value = "")]
        debug: DebugLang,
    },
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    match command {
        Command::Run {
            file,
            output,
            debug,
        } => {
            slides_lang::compiler::compile_project(file, output, debug)?;
        }
        Command::Format { file, dry, debug } => {
            slides_lang::formatter::format_file(file, dry, debug)?;
        }
    }
    Ok(())
}
