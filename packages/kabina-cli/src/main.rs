use std::{error::Error, path::PathBuf};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    Build {
        #[arg(short, long)]
        schema: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Command::parse();

    tracing_subscriber::fmt::init();

    match args {
        Command::Build { mut schema } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            if schema.is_relative() {
                schema = std::env::current_dir()?.join(schema)
            }

            rt.block_on(kabina_rt::invoke(schema));
        }
    }

    Ok(())
}
