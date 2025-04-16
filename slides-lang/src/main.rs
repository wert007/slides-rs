use std::path::PathBuf;

use clap::Parser;
use notify::Watcher;
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
    Watch {
        file: PathBuf,
        #[clap(short, long, default_value = "out")]
        output: PathBuf,
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
        Command::Watch {
            file,
            output,
            debug,
        } => {
            watch(file, output, debug)?;
        }
    }
    Ok(())
}

fn watch(file: PathBuf, output: PathBuf, debug: DebugLang) -> Result<(), anyhow::Error> {
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    match slides_lang::compiler::compile_project(&file, &output, debug) {
        Ok(result) => {
            dbg!(&result);
            for file in result.used_files {
                watcher.watch(&file, notify::RecursiveMode::NonRecursive)?;
            }
        }
        Err(err) => {
            eprintln!("Error occured, continueing: {err}");
        }
    }
    for event in rx {
        let _event = event?;
        match slides_lang::compiler::compile_project(&file, &output, debug) {
            Ok(result) => {
                for file in result.used_files {
                    watcher.watch(&file, notify::RecursiveMode::NonRecursive)?;
                }
            }
            Err(err) => {
                eprintln!("Error occured, continueing: {err}");
            }
        }
    }
    Ok(())
}
