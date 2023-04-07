#![feature(decl_macro)]

use std::{error::Error, path::PathBuf, sync::Arc};

use clap::Parser;
use kabina_db::{
    collection_files, runtime::Runtime, Cause, Executable, ResolveRootFiles, RuntimeTask,
    SharedDatabase, ToolchainObject, ToolchainResolve, TransformApply,
};

use kabina_rt::DenoRuntime;
use parking_lot::RwLock;

mod daemon;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Command {
    Build {
        #[arg(long)]
        schema: PathBuf,
        #[arg(long)]
        collection: String,
    },
    #[clap(subcommand)]
    Daemon(Daemon),
}

#[derive(clap::Parser, Debug)]
enum Daemon {
    Start {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Command::parse();

    tracing_subscriber::fmt::init();

    match args {
        Command::Build {
            mut schema,
            collection: bundle,
        } => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            if schema.is_relative() {
                schema = std::env::current_dir()?.join(schema)
            }

            let db = Arc::new(RwLock::new(kabina_db::Database::new()));

            let mut runtime = Box::new(rt.block_on(DenoRuntime::new(db.clone())));

            // Populating the schema from TS
            let schema = rt.block_on(runtime.load_schema(schema));

            let collection = {
                let db = db.read();
                let collections = schema.collections(&*db);
                let collection = collections
                    .iter()
                    .find(|b| (*b).name(&*db) == bundle)
                    .unwrap()
                    .clone();

                collection
            };

            rt.block_on(drive!(runtime, collection_files(db, schema, collection)));
        }
        Command::Daemon(Daemon::Start {}) => daemon::daemon_start(),
    }

    Ok(())
}

pub macro drive($rt:expr, $func:ident($db:expr, $($arg:expr),+)) {
    async { loop {
        #[allow(unused_assignments)]
        let mut tasks = Vec::new();
        match $func(&*$db.read(), $($arg),+) {
            Ok(result) => {
                break result;
            }
            Err(Cause::Pending) => {
                tasks = $func::accumulated::<RuntimeTask>(
                    &*$db.read(), $($arg),+
                );
            }
            Err(e) => panic!("{:?}", e),
        }

        for task in tasks {
            // TODO: parallel
            drive_task(&*task, &$db, &mut $rt).await
        }
    } }
}

pub async fn drive_task(task: &dyn Executable, db: &SharedDatabase, rt: &mut Box<impl Runtime>) {
    if let Some(task) = task.downcast_ref::<ResolveRootFiles>() {
        task.resolve(&mut db.write())
    } else if let Some(task) = task.downcast_ref::<TransformApply>() {
        rt.transform(task).await;
    } else if let Some(task) = task.downcast_ref::<ToolchainResolve>() {
        let binary = task.toolchain.binary(&*db.read());
        match which::which(&binary) {
            Ok(path) => {
                tracing::info!("[ToolchainResolve] Resolved {:?} to {:?}", binary, path);
                task.resolve(&mut *db.write(), Ok(ToolchainObject { binary: path }))
            }
            Err(e) => task.resolve(&mut *db.write(), Err(Cause::from_err(e))),
        }
    }
}
