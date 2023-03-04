use std::{error::Error, path::PathBuf, sync::Arc};

use clap::Parser;
use kabina_db::{
    bundle_files, runtime::Runtime, ApplyTransform, Cause, ResolveRootFiles, RuntimeTask,
};

use kabina_rt::DenoRuntime;
use parking_lot::RwLock;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    Build {
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        bundle: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Command::parse();

    tracing_subscriber::fmt::init();

    match args {
        Command::Build { mut schema, bundle } => {
            let tokio = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            if schema.is_relative() {
                schema = std::env::current_dir()?.join(schema)
            }

            let db = Arc::new(RwLock::new(kabina_db::Database::new()));

            let mut runtime = Box::new(DenoRuntime::new(db.clone()));

            // Populating the schema from TS
            let schema = tokio.block_on(runtime.load_schema(schema));

            let bundle = {
                let db = db.read();
                let bundles = schema.bundles(&*db);
                let bundle = bundles
                    .iter()
                    .find(|b| (*b).name(&*db) == bundle)
                    .unwrap()
                    .clone();

                bundle
            };

            loop {
                let mut tasks = Vec::new();
                match bundle_files(&*db.read(), schema, bundle) {
                    Ok(files) => {
                        println!("Bundle files {:?}", files);
                        break;
                    }
                    Err(Cause::Pending) => {
                        tasks = kabina_db::bundle_files::accumulated::<RuntimeTask>(
                            &*db.read(),
                            schema,
                            bundle,
                        );
                    }
                    Err(e) => panic!("{:?}", e),
                }

                for task in tasks {
                    if let Some(task) = task.downcast_ref::<ResolveRootFiles>() {
                        task.resolve(&mut db.write())
                    } else if let Some(task) = task.downcast_ref::<ApplyTransform>() {
                        tokio.block_on(runtime.transform(task));
                    }
                }
            }
        }
    }

    Ok(())
}
