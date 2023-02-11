use std::{error::Error, path::PathBuf, sync::Arc};

use clap::Parser;
use kabina_db::DebugWithDb;

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

            let db = Arc::new(kabina_db::Database::new());

            // Populating the schema from TS
            let schema = rt.block_on(kabina_rt::invoke(schema, db.clone()));

            let groups = schema.file_groups(&*db);

            for group in groups.iter() {
                println!(
                    "Files in {:?}: {:#?}",
                    *group,
                    kabina_db::file_group_files(&*db, schema, *group)
                        .into_iter()
                        .map(|f| f.into_debug_all(&*db))
                        .collect::<Vec<_>>()
                )
            }

            let transforms = schema.transforms(&*db);

            for t in transforms.iter() {
                println!(
                    "Dependencies in {:?}: {:#?}",
                    *t,
                    kabina_db::transform_inputs(&*db, *t)
                )
            }
        }
    }

    Ok(())
}
