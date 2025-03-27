use std::path::PathBuf;

use clap::Parser;
mod compiler;

#[derive(Debug, clap::Parser)]
struct Args {
    file: PathBuf,
    #[clap(short, long, default_value = "out")]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    compiler::compile_project(args.file, args.output)?;
    Ok(())
}
